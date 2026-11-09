use std::ffi::OsStr;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use serde::Serialize;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter};

use crate::catalog::{self, SoftwareItem};

const MAX_DOWNLOAD_BYTES: u64 = 3 * 1024 * 1024 * 1024;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallProgress {
    pub app_id: String,
    pub percent: Option<f64>,
    pub phase: String,
    pub message: Option<String>,
}

#[cfg(windows)]
fn hidden_command(program: impl AsRef<OsStr>) -> Command {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let mut command = Command::new(program);
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

#[cfg(not(windows))]
fn hidden_command(program: impl AsRef<OsStr>) -> Command {
    Command::new(program)
}

fn emit_progress(
    app: &AppHandle,
    app_id: &str,
    percent: Option<f64>,
    phase: &str,
    message: Option<String>,
) {
    let _ = app.emit(
        "install-progress",
        InstallProgress {
            app_id: app_id.to_string(),
            percent,
            phase: phase.to_string(),
            message,
        },
    );
}

pub async fn install_entry(app: AppHandle, app_id: String) -> Result<String, String> {
    let entry = catalog::find(&app_id)?;

    if entry.placeholder {
        return Err(
            "This entry is a configuration template. Set placeholder to false after filling in the download url."
                .to_string(),
        );
    }

    match entry.kind.as_str() {
        "winget" => install_winget_entry(&app, &entry).await,
        "download" | "custom" => install_from_url(&app, &entry).await,
        other => Err(format!("unsupported install kind: {other}")),
    }
}

/// Kept for compatibility with the first catalog screen.
pub async fn install_winget(winget_id: String) -> Result<String, String> {
    catalog::validate_winget_id(&winget_id)?;
    tauri::async_runtime::spawn_blocking(move || run_winget(&winget_id))
        .await
        .map_err(|error| error.to_string())?
}

async fn install_winget_entry(app: &AppHandle, entry: &SoftwareItem) -> Result<String, String> {
    let winget_id = entry
        .winget_id
        .clone()
        .ok_or_else(|| "winget id is missing".to_string())?;

    emit_progress(app, &entry.id, None, "Установка через winget", None);

    let task_id = winget_id.clone();
    let result = tauri::async_runtime::spawn_blocking(move || run_winget(&task_id))
        .await
        .map_err(|error| error.to_string())?;

    match result {
        Ok(report) => {
            emit_progress(app, &entry.id, Some(100.0), "Готово", None);
            Ok(report)
        }
        Err(error) => {
            emit_progress(
                app,
                &entry.id,
                None,
                "Ошибка установки",
                Some(error.clone()),
            );
            Err(error)
        }
    }
}

fn winget_program() -> String {
    // GUI processes launched from Explorer may have a reduced PATH, so resolve
    // the WindowsApps execution alias explicitly before falling back to PATH.
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        let candidate = PathBuf::from(local)
            .join("Microsoft")
            .join("WindowsApps")
            .join("winget.exe");
        if candidate.is_file() {
            return candidate.display().to_string();
        }
    }
    "winget".to_string()
}

/// Only one winget operation may run at a time: parallel winget/MSI runs race
/// on the Windows Installer mutex and produce 1618/1603 failures.
static WINGET_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Runs winget with a hard timeout (the process tree is killed on expiry).
pub(crate) fn winget_run(args: &[&str], timeout_secs: u64) -> Result<String, String> {
    use std::io::Read;
    use std::process::Stdio;
    use std::time::Instant;

    let mut child = hidden_command(winget_program())
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("could not start winget: {error}"))?;

    // Drain pipes on threads so a chatty process cannot block on a full buffer.
    let mut out = child.stdout.take();
    let mut err = child.stderr.take();
    let out_thread = std::thread::spawn(move || {
        let mut buffer = Vec::new();
        if let Some(pipe) = out.as_mut() {
            let _ = pipe.read_to_end(&mut buffer);
        }
        buffer
    });
    let err_thread = std::thread::spawn(move || {
        let mut buffer = Vec::new();
        if let Some(pipe) = err.as_mut() {
            let _ = pipe.read_to_end(&mut buffer);
        }
        buffer
    });

    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if started.elapsed() > Duration::from_secs(timeout_secs) {
                    let _ = child.kill();
                    let _ = child.wait();
                    crate::diag::log("ERROR", &format!("winget timeout: {args:?}"));
                    return Err("winget timed out".to_string());
                }
                std::thread::sleep(Duration::from_millis(250));
            }
            Err(error) => return Err(format!("could not wait for winget: {error}")),
        }
    };

    let stdout = out_thread.join().unwrap_or_default();
    let stderr = err_thread.join().unwrap_or_default();
    let report = format!(
        "{}{}",
        String::from_utf8_lossy(&stdout),
        String::from_utf8_lossy(&stderr)
    )
    .trim()
    .to_string();

    if status.success() {
        Ok(report)
    } else {
        let code = status.code().map(|c| format!(" (0x{:X})", c as u32)).unwrap_or_default();
        Err(if report.is_empty() {
            format!("winget exited with {status}{code}")
        } else {
            format!("{report}\nwinget exit{code}")
        })
    }
}

pub(crate) fn winget_version() -> Result<String, String> {
    winget_run(&["--version"], 20).map(|v| v.trim().to_string())
}

fn run_winget(winget_id: &str) -> Result<String, String> {
    let _guard = WINGET_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    crate::diag::log("INFO", &format!("winget install {winget_id}"));

    let args = [
        "install",
        "--id",
        winget_id,
        "--exact",
        "--silent",
        "--accept-package-agreements",
        "--accept-source-agreements",
        "--disable-interactivity",
    ];

    let mut last_error = String::new();
    for attempt in 0..3 {
        match winget_run(&args, 1800) {
            Ok(report) => {
                crate::diag::log("INFO", &format!("winget install {winget_id}: ok"));
                return Ok(if report.is_empty() {
                    format!("{winget_id} installed")
                } else {
                    report
                });
            }
            Err(error) => {
                // 1618: another installation is running; wait and retry.
                let busy = matches!(
                    crate::diag::error_code(&error),
                    Some(("E-LOCK-004", _))
                );
                last_error = error;
                if busy && attempt < 2 {
                    crate::diag::log("WARN", &format!("installer busy, retry {}", attempt + 1));
                    std::thread::sleep(Duration::from_secs(20));
                    continue;
                }
                break;
            }
        }
    }
    Err(crate::diag::annotate(&format!("winget install {winget_id}"), last_error))
}

async fn install_from_url(app: &AppHandle, entry: &SoftwareItem) -> Result<String, String> {
    let url = entry
        .download_url
        .clone()
        .ok_or_else(|| "download url is missing".to_string())?;
    catalog::validate_https_url(&url)?;

    emit_progress(app, &entry.id, Some(0.0), "Подключение к серверу", None);
    let path = download_file(app, entry, &url).await?;

    if let Some(expected) = entry.sha256.as_deref() {
        emit_progress(app, &entry.id, Some(93.0), "Проверка SHA-256", None);
        if let Err(error) = verify_sha256(&path, expected) {
            let _ = std::fs::remove_file(&path);
            return Err(error);
        }
    }

    emit_progress(app, &entry.id, Some(96.0), "Запуск установщика", None);

    let installer_path = path.clone();
    let arguments = entry.installer_args.clone();
    let config = serde_json::to_string(&entry.custom_config).unwrap_or_else(|_| "{}".to_string());

    let result = tauri::async_runtime::spawn_blocking(move || {
        run_installer(&installer_path, &arguments, &config)
    })
    .await
    .map_err(|error| error.to_string())?;

    let _ = std::fs::remove_file(&path);

    match result {
        Ok(report) => {
            emit_progress(app, &entry.id, Some(100.0), "Готово", None);
            Ok(report)
        }
        Err(error) => {
            emit_progress(
                app,
                &entry.id,
                None,
                "Ошибка установки",
                Some(error.clone()),
            );
            Err(error)
        }
    }
}

async fn download_file(
    app: &AppHandle,
    entry: &SoftwareItem,
    url: &str,
) -> Result<PathBuf, String> {
    let client = reqwest::Client::builder()
        .user_agent(concat!("SetupHub/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(Duration::from_secs(20))
        .timeout(Duration::from_secs(1800))
        .build()
        .map_err(|error| error.to_string())?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|error| format!("download failed: {error}"))?;

    if !response.status().is_success() {
        return Err(format!("download failed with status {}", response.status()));
    }

    let final_url = response.url().clone();
    if final_url.scheme() != "https" {
        return Err("download was redirected to a non-https address".to_string());
    }

    let total = response.content_length();
    if total.is_some_and(|size| size > MAX_DOWNLOAD_BYTES) {
        return Err("download is larger than the 3 GiB limit".to_string());
    }

    let file_name = file_name_from_url(&final_url)?;
    let directory = std::env::temp_dir().join("setup-hub-downloads");
    std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let path = directory.join(format!("{}-{file_name}", entry.id));

    let mut file = std::fs::File::create(&path).map_err(|error| error.to_string())?;
    let mut stream = response.bytes_stream();
    let mut received = 0u64;
    let mut last_emit = Instant::now() - Duration::from_secs(1);

    while let Some(chunk) = stream.next().await {
        let chunk = match chunk {
            Ok(chunk) => chunk,
            Err(error) => {
                let _ = std::fs::remove_file(&path);
                return Err(format!("download interrupted: {error}"));
            }
        };

        received += chunk.len() as u64;
        if received > MAX_DOWNLOAD_BYTES {
            let _ = std::fs::remove_file(&path);
            return Err("download is larger than the 3 GiB limit".to_string());
        }

        file.write_all(&chunk)
            .map_err(|error| format!("could not write the installer: {error}"))?;

        if last_emit.elapsed() >= Duration::from_millis(120) {
            let percent = total
                .filter(|size| *size > 0)
                .map(|size| (received as f64 / size as f64 * 92.0).min(92.0));
            emit_progress(app, &entry.id, percent, "Загрузка установщика", None);
            last_emit = Instant::now();
        }
    }

    file.flush().map_err(|error| error.to_string())?;
    Ok(path)
}

fn file_name_from_url(url: &reqwest::Url) -> Result<String, String> {
    let raw = url
        .path_segments()
        .and_then(|mut segments| segments.next_back())
        .unwrap_or_default();

    let mut name: String = raw
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-') {
                ch
            } else {
                '_'
            }
        })
        .collect();

    name = name.trim_matches('.').to_string();
    if name.is_empty() {
        return Err("download url has no file name".to_string());
    }
    if name.len() > 120 {
        return Err("download file name is too long".to_string());
    }

    let extension = Path::new(&name)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    if !matches!(extension.as_str(), "exe" | "msi" | "msix" | "appx") {
        return Err("this installer file type is not allowed".to_string());
    }

    Ok(name)
}

fn verify_sha256(path: &Path, expected: &str) -> Result<(), String> {
    let mut file = std::fs::File::open(path).map_err(|error| error.to_string())?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher).map_err(|error| error.to_string())?;
    let actual = format!("{:x}", hasher.finalize());

    if actual.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err("installer checksum does not match the catalog".to_string())
    }
}

fn run_installer(path: &Path, arguments: &[String], config: &str) -> Result<String, String> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    let status = match extension.as_str() {
        "exe" => hidden_command(path)
            .args(arguments)
            .env("SETUP_HUB_APP_CONFIG", config)
            .status()
            .map_err(|error| format!("could not start the installer: {error}"))?,
        "msi" => {
            let mut command = hidden_command("msiexec");
            command
                .arg("/i")
                .arg(path)
                .args(["/qn", "/norestart"])
                .args(arguments)
                .env("SETUP_HUB_APP_CONFIG", config);
            command
                .status()
                .map_err(|error| format!("could not start msiexec: {error}"))?
        }
        "msix" | "appx" => {
            let script = format!(
                "$ErrorActionPreference='Stop'; Add-AppxPackage -Path '{}';",
                path.display().to_string().replace('\'', "''")
            );
            hidden_command("powershell")
                .args([
                    "-NoLogo",
                    "-NoProfile",
                    "-NonInteractive",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-Command",
                    &script,
                ])
                .env("SETUP_HUB_APP_CONFIG", config)
                .status()
                .map_err(|error| format!("could not start the package installer: {error}"))?
        }
        _ => return Err("unsupported installer file type".to_string()),
    };

    if status.success() {
        Ok(format!("installer finished with {}", status))
    } else {
        Err(format!("installer exited with {status}"))
    }
}
