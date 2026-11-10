//! Automatic download of the official Windows ISO.
//!
//! Microsoft has no stable direct links, so the one-time download URL is resolved
//! by the open-source Fido script (https://github.com/pbatard/Fido), pinned to a
//! reviewed commit and verified by SHA-256 before it is ever executed. The ISO
//! itself is then streamed from Microsoft's servers with resume support.

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use serde::Serialize;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter};

use crate::tweaks::{hidden_command, run_powershell};

const FIDO_COMMIT: &str = "266084e3765ab8bf96767bd55c51b0ee70934551";
/// SHA-256 of the script as served by GitHub (LF line endings, UTF-8 BOM)...
const FIDO_SHA256: &str = "89a26b4a8be2721e610ba3f563ec44024e6cf7975938ee5b61d624d2bfbf4643";
/// ...and of the same file with CRLF line endings (what a Windows git checkout yields).
const FIDO_SHA256_CRLF: &str = "ceedef02aca0135fcd98f481d1c386e467432f2973bbbade093205c380330da5";

fn fido_hash_ok(bytes: &[u8]) -> bool {
    let hash = sha256_hex(bytes);
    hash == FIDO_SHA256 || hash == FIDO_SHA256_CRLF
}

const MIN_ISO_BYTES: u64 = 2 * 1024 * 1024 * 1024;

static CANCEL: AtomicBool = AtomicBool::new(false);

pub fn cancel() {
    CANCEL.store(true, Ordering::SeqCst);
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct IsoProgress {
    phase: String,
    percent: f64,
    received_mb: f64,
    total_mb: f64,
    speed_mbs: f64,
}

fn emit(app: &AppHandle, phase: &str, percent: f64, received: u64, total: u64, speed: f64) {
    let _ = app.emit(
        "iso-progress",
        IsoProgress {
            phase: phase.to_string(),
            percent,
            received_mb: (received as f64 / 1_048_576.0).round(),
            total_mb: (total as f64 / 1_048_576.0).round(),
            speed_mbs: (speed * 10.0).round() / 10.0,
        },
    );
}

pub fn iso_dir() -> PathBuf {
    crate::diag::data_dir().join("iso")
}

fn tools_dir() -> PathBuf {
    crate::diag::data_dir().join("tools")
}

fn product_name(product: &str) -> Result<&'static str, String> {
    match product {
        "win11" => Ok("Windows 11"),
        "win10" => Ok("Windows 10"),
        _ => Err("unknown Windows version".into()),
    }
}

fn valid_language(language: &str) -> bool {
    (2..=40).contains(&language.len())
        && language
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '(' | ')' | '-'))
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

/// Makes sure the pinned, hash-checked Fido script is on disk and returns its path.
async fn ensure_fido() -> Result<PathBuf, String> {
    let dir = tools_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("cannot create tools folder: {e}"))?;
    let path = dir.join(format!("Fido-{}.ps1", &FIDO_COMMIT[..8]));

    if let Ok(existing) = fs::read(&path) {
        if fido_hash_ok(&existing) {
            return Ok(path);
        }
    }

    let url = format!("https://raw.githubusercontent.com/pbatard/Fido/{FIDO_COMMIT}/Fido.ps1");
    let client = reqwest::Client::builder()
        .user_agent(concat!("SetupHub/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(Duration::from_secs(20))
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| e.to_string())?;
    let bytes = client
        .get(&url)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| format!("не удалось загрузить помощник Fido: {e}"))?
        .bytes()
        .await
        .map_err(|e| format!("не удалось загрузить помощник Fido: {e}"))?;

    if !fido_hash_ok(&bytes) {
        crate::diag::log("ERROR", &format!("Fido hash was {}", sha256_hex(&bytes)));
        crate::diag::log("ERROR", "Fido hash mismatch, refusing to run it");
        return Err("Контрольная сумма Fido не совпала — запуск отменён из соображений безопасности.".into());
    }
    fs::write(&path, &bytes).map_err(|e| format!("cannot save Fido: {e}"))?;
    Ok(path)
}

/// Runs Fido in command-line mode and returns the one-time download URL.
fn resolve_url(fido: &Path, product: &str, language: &str) -> Result<String, String> {
    let name = product_name(product)?;
    if !valid_language(language) {
        return Err("invalid language".into());
    }

    let mut child = hidden_command("powershell")
        .args(["-NoLogo", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-File"])
        .arg(fido)
        .args([
            "-Win", name, "-Rel", "Latest", "-Ed", "Home/Pro/Edu", "-Lang", language, "-Arch",
            "x64", "-PlatformArch", "x64", "-GetUrl",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("could not start powershell: {e}"))?;

    let mut out = child.stdout.take();
    let mut err = child.stderr.take();
    let t_out = std::thread::spawn(move || {
        let mut s = String::new();
        if let Some(p) = out.as_mut() {
            let mut b = Vec::new();
            let _ = p.read_to_end(&mut b);
            s = String::from_utf8_lossy(&b).to_string();
        }
        s
    });
    let t_err = std::thread::spawn(move || {
        let mut s = String::new();
        if let Some(p) = err.as_mut() {
            let mut b = Vec::new();
            let _ = p.read_to_end(&mut b);
            s = String::from_utf8_lossy(&b).to_string();
        }
        s
    });

    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if started.elapsed() > Duration::from_secs(180) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("Microsoft не ответил вовремя (таймаут)".into());
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(200)),
            Err(e) => return Err(e.to_string()),
        }
    }
    let stdout = t_out.join().unwrap_or_default();
    let stderr = t_err.join().unwrap_or_default();

    if let Some(url) = stdout
        .lines()
        .map(str::trim)
        .rev()
        .find(|l| l.starts_with("https://"))
    {
        return Ok(url.to_string());
    }

    let report = format!("{stdout}\n{stderr}");
    crate::diag::log("ERROR", &format!("Fido failed: {}", report.replace('\n', " | ")));
    let lowered = report.to_lowercase();
    Err(if lowered.contains("invalid windows language") {
        "Для выбранной версии нет такого языка.".to_string()
    } else if lowered.contains("banned") || lowered.contains("blocked") || lowered.contains("715-123130") {
        "Microsoft временно ограничил автоматическое скачивание с этого адреса. Скачайте ISO вручную на сайте Microsoft и выберите файл.".to_string()
    } else {
        let tail: String = report.trim().chars().rev().take(300).collect::<Vec<_>>().into_iter().rev().collect();
        format!("Не удалось получить ссылку на ISO у Microsoft. {tail}")
    })
}

fn free_gb(dir: &Path) -> Option<f64> {
    let letter = dir.to_string_lossy().chars().next().filter(|c| c.is_ascii_alphabetic())?;
    run_powershell(&format!("[math]::Round((Get-PSDrive -Name {letter}).Free / 1GB, 1)"))
        .ok()?
        .trim()
        .replace(',', ".")
        .parse()
        .ok()
}

pub async fn download(app: &AppHandle, product: &str, language: &str) -> Result<String, String> {
    product_name(product)?;
    if !valid_language(language) {
        return Err("invalid language".into());
    }
    CANCEL.store(false, Ordering::SeqCst);

    let dir = iso_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    let slug: String = language.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    let final_path = dir.join(format!("Windows-{product}-{slug}.iso"));
    let part_path = dir.join(format!("Windows-{product}-{slug}.iso.part"));

    // A complete earlier download is reused.
    if fs::metadata(&final_path).map(|m| m.len() >= MIN_ISO_BYTES).unwrap_or(false) {
        emit(app, "Образ уже скачан", 100.0, 0, 0, 0.0);
        return Ok(final_path.display().to_string());
    }

    emit(app, "Запрос ссылки у Microsoft", 0.0, 0, 0, 0.0);
    let fido = ensure_fido().await?;
    let (p, l) = (product.to_string(), language.to_string());
    let url = tauri::async_runtime::spawn_blocking(move || resolve_url(&fido, &p, &l))
        .await
        .map_err(|e| e.to_string())??;

    let parsed = reqwest::Url::parse(&url).map_err(|_| "Microsoft вернул некорректную ссылку".to_string())?;
    let host = parsed.host_str().unwrap_or_default().to_lowercase();
    if parsed.scheme() != "https" || !(host == "microsoft.com" || host.ends_with(".microsoft.com")) {
        crate::diag::log("ERROR", &format!("unexpected ISO host: {host}"));
        return Err("Ссылка ведёт не на сервер Microsoft — скачивание отменено.".into());
    }

    let client = reqwest::Client::builder()
        .user_agent(concat!("SetupHub/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;

    // Resume a partial file if there is one.
    let existing = fs::metadata(&part_path).map(|m| m.len()).unwrap_or(0);
    let mut request = client.get(parsed);
    if existing > 0 {
        request = request.header("Range", format!("bytes={existing}-"));
    }
    let response = request.send().await.map_err(|e| format!("не удалось начать скачивание: {e}"))?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("сервер Microsoft ответил кодом {status}"));
    }
    let resumed = status.as_u16() == 206 && existing > 0;
    let start = if resumed { existing } else { 0 };
    let total = response.content_length().map(|n| n + start).unwrap_or(0);

    if total > 0 {
        let dir_for_check = dir.clone();
        let free = tauri::async_runtime::spawn_blocking(move || free_gb(&dir_for_check))
            .await
            .ok()
            .flatten();
        let need = (total - start) as f64 / 1_073_741_824.0 + 1.0;
        if free.is_some_and(|f| f < need) {
            return Err(format!(
                "Недостаточно места на диске: нужно около {:.1} ГБ, свободно {:.1} ГБ.",
                need,
                free.unwrap_or(0.0)
            ));
        }
    }

    let mut file = if resumed {
        fs::OpenOptions::new().append(true).open(&part_path)
    } else {
        fs::File::create(&part_path)
    }
    .map_err(|e| format!("cannot write {}: {e}", part_path.display()))?;

    let mut stream = response.bytes_stream();
    let mut received = start;
    let began = Instant::now();
    let mut last_emit = Instant::now() - Duration::from_secs(1);
    let mut window = (Instant::now(), received);
    let mut speed = 0.0f64;

    loop {
        if CANCEL.load(Ordering::SeqCst) {
            return Err("Остановлено".into());
        }
        // A stalled connection is reported instead of hanging forever.
        let next = tokio::time::timeout(Duration::from_secs(60), stream.next())
            .await
            .map_err(|_| "соединение с Microsoft прервалось (нет данных 60 секунд); повторите — загрузка продолжится".to_string())?;
        let Some(chunk) = next else { break };
        let chunk = chunk.map_err(|e| format!("загрузка прервана: {e}; повторите — она продолжится"))?;
        file.write_all(&chunk).map_err(|e| format!("ошибка записи на диск: {e}"))?;
        received += chunk.len() as u64;

        if last_emit.elapsed() >= Duration::from_millis(300) {
            let dt = window.0.elapsed().as_secs_f64();
            if dt >= 1.0 {
                speed = (received - window.1) as f64 / 1_048_576.0 / dt;
                window = (Instant::now(), received);
            }
            let percent = if total > 0 { received as f64 / total as f64 * 100.0 } else { 0.0 };
            emit(app, "Скачивание Windows", percent.min(100.0), received, total, speed);
            last_emit = Instant::now();
        }
    }
    file.flush().map_err(|e| e.to_string())?;
    drop(file);

    if total > 0 && received != total {
        return Err("Скачанный файл неполный; повторите — загрузка продолжится".into());
    }
    if received < MIN_ISO_BYTES {
        let _ = fs::remove_file(&part_path);
        return Err("Скачанный файл слишком мал для образа Windows".into());
    }
    fs::rename(&part_path, &final_path).map_err(|e| format!("cannot finalize ISO: {e}"))?;
    crate::diag::log(
        "INFO",
        &format!("ISO downloaded: {} ({} MB in {}s)", final_path.display(), received / 1_048_576, began.elapsed().as_secs()),
    );
    emit(app, "Готово", 100.0, received, total, speed);
    Ok(final_path.display().to_string())
}

/// Removes a cached ISO; only files inside Setup Hub's own ISO folder qualify.
pub fn delete_cached(path: &str) -> Result<(), String> {
    let target = PathBuf::from(path.trim());
    let canonical = target.canonicalize().map_err(|e| e.to_string())?;
    let root = iso_dir().canonicalize().map_err(|e| e.to_string())?;
    let is_iso = canonical.extension().is_some_and(|e| e.eq_ignore_ascii_case("iso"));
    if !canonical.starts_with(&root) || !is_iso {
        return Err("only Setup Hub's own downloaded ISO files can be removed".into());
    }
    fs::remove_file(canonical).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_validation() {
        assert!(valid_language("Russian"));
        assert!(valid_language("English (United States)"));
        assert!(!valid_language("a"));
        assert!(!valid_language("Russian; calc"));
        assert!(!valid_language("a\"b"));
    }

    #[test]
    fn products() {
        assert_eq!(product_name("win11").unwrap(), "Windows 11");
        assert!(product_name("win7").is_err());
    }

    #[test]
    fn fido_hash_constant_is_sha256() {
        assert_eq!(FIDO_SHA256.len(), 64);
        assert_eq!(FIDO_SHA256_CRLF.len(), 64);
        assert!(FIDO_SHA256.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(sha256_hex(b""), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    }
}
