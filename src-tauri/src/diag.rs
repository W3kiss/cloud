//! Self-diagnostics: file logging, panic capture, error codes and preflight checks.
//!
//! Logs live in `%LOCALAPPDATA%\SetupHub\logs\setup-hub.log` and rotate at 1 MB.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::tweaks::{hidden_command, run_powershell};

static LOG_LOCK: Mutex<()> = Mutex::new(());
const MAX_LOG_BYTES: u64 = 1_000_000;

pub fn data_dir() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    base.join("SetupHub")
}

pub fn log_dir() -> PathBuf {
    data_dir().join("logs")
}

fn log_path() -> PathBuf {
    log_dir().join("setup-hub.log")
}

fn timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (h, m, s) = ((secs / 3600) % 24, (secs / 60) % 60, secs % 60);
    let days = secs / 86_400;
    format!("day{days} {h:02}:{m:02}:{s:02}Z")
}

/// Appends one line to the log. Never panics and never fails the caller.
pub fn log(level: &str, message: &str) {
    let Ok(_guard) = LOG_LOCK.lock() else { return };
    let dir = log_dir();
    if fs::create_dir_all(&dir).is_err() {
        return;
    }
    let path = log_path();
    if fs::metadata(&path).map(|m| m.len() > MAX_LOG_BYTES).unwrap_or(false) {
        let _ = fs::rename(&path, dir.join("setup-hub.1.log"));
    }
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
        let flat = message.replace('\r', "").replace('\n', " | ");
        let _ = writeln!(file, "{} [{level}] {flat}", timestamp());
    }
}

/// Captures panics (including those inside worker threads) into the log.
pub fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        log("PANIC", &info.to_string());
        previous(info);
    }));
    log("INFO", "Setup Hub started");
}

/// Maps a raw tool report to a stable error code with a human hint.
pub fn error_code(report: &str) -> Option<(&'static str, &'static str)> {
    let l = report.to_lowercase();
    let has = |needle: &str| l.contains(needle);
    if has("admin_required") || has("access is denied") || has("0x80070005") {
        Some(("E-PERM-001", "Нужны права администратора."))
    } else if has("0x8a150011") || has("no applicable upgrade") || has("no newer package") {
        Some(("E-WINGET-NOUPDATE", "Обновлений нет."))
    } else if has("1618") || has("another installation is already in progress") {
        Some(("E-LOCK-004", "Идёт другая установка Windows. Повторите позже."))
    } else if has("1603") {
        Some(("E-INSTALL-1603", "Установщик завершился с критической ошибкой."))
    } else if has("0x8a150014") || has("no package found") || has("not found") {
        Some(("E-PATH-002", "Пакет или путь не найден."))
    } else if has("0x8a15000f") || has("failed when opening source") || has("source") && has("reset") {
        Some(("E-WINGET-SRC", "Источник winget повреждён: выполните «Исправить» в диагностике."))
    } else if has("timeout") || has("timed out") || has("could not be resolved") {
        Some(("E-NET-003", "Сетевая ошибка или таймаут."))
    } else if has("0x80070070") || has("no space left") {
        Some(("E-DISK-005", "Недостаточно места на диске."))
    } else if has("could not start winget") {
        Some(("E-WINGET-MISSING", "winget не найден. Установите «Установщик приложений» из Microsoft Store."))
    } else {
        None
    }
}

/// Appends the error code and hint to a failure message and logs it.
pub fn annotate(context: &str, message: String) -> String {
    log("ERROR", &format!("{context}: {message}"));
    match error_code(&message) {
        Some((code, hint)) => format!("[{code}] {hint}\n{message}"),
        None => message,
    }
}

// ------------------------------------------------------------ preflight

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Check {
    pub id: String,
    pub title: String,
    /// "ok" | "warn" | "fail"
    pub status: String,
    pub detail: String,
    pub code: String,
    pub fixable: bool,
}

fn check(id: &str, title: &str, status: &str, detail: impl Into<String>, code: &str, fixable: bool) -> Check {
    Check {
        id: id.into(),
        title: title.into(),
        status: status.into(),
        detail: detail.into(),
        code: code.into(),
        fixable,
    }
}

pub fn run_checks() -> Vec<Check> {
    let mut checks = Vec::new();

    // Administrator rights
    let admin = crate::tweaks::is_admin();
    checks.push(if admin {
        check("admin", "Права администратора", "ok", "Приложение запущено с повышенными правами.", "", false)
    } else {
        check(
            "admin",
            "Права администратора",
            "warn",
            "Часть твиков, служб и запись флешки требуют запуска от имени администратора.",
            "E-PERM-001",
            false,
        )
    });

    // winget
    let winget = crate::installer::winget_version();
    checks.push(match winget {
        Ok(version) => check("winget", "winget", "ok", format!("Версия {version}"), "", false),
        Err(error) => check("winget", "winget", "fail", error, "E-WINGET-MISSING", false),
    });

    // winget sources
    if crate::installer::winget_version().is_ok() {
        let sources = crate::installer::winget_run(&["source", "list"], 60);
        checks.push(match sources {
            Ok(out) if !out.to_lowercase().contains("failed") => {
                check("winget_sources", "Источники winget", "ok", "Источники доступны.", "", true)
            }
            Ok(out) => check("winget_sources", "Источники winget", "fail", out, "E-WINGET-SRC", true),
            Err(error) => check("winget_sources", "Источники winget", "fail", error, "E-WINGET-SRC", true),
        });
    }

    // Network
    checks.push(match network_ok() {
        true => check("network", "Подключение к сети", "ok", "Интернет доступен.", "", false),
        false => check("network", "Подключение к сети", "fail", "Нет доступа в интернет — установка недоступна.", "E-NET-003", false),
    });

    // Disk space (system drive)
    let space = run_powershell("[math]::Round((Get-PSDrive -Name ($env:SystemDrive.TrimEnd(':'))).Free / 1GB, 1)");
    checks.push(match space.ok().and_then(|s| s.trim().replace(',', ".").parse::<f64>().ok()) {
        Some(gb) if gb >= 5.0 => check("disk", "Место на системном диске", "ok", format!("Свободно {gb} ГБ"), "", false),
        Some(gb) => check("disk", "Место на системном диске", "warn", format!("Свободно всего {gb} ГБ"), "E-DISK-005", false),
        None => check("disk", "Место на системном диске", "warn", "Не удалось определить.", "", false),
    });

    // Pending reboot
    let reboot = run_powershell(
        "$p=$false; foreach($k in 'HKLM:\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\WindowsUpdate\\Auto Update\\RebootRequired','HKLM:\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Component Based Servicing\\RebootPending'){ if(Test-Path $k){$p=$true} }; if($p){'yes'}else{'no'}",
    );
    checks.push(match reboot.as_deref().map(str::trim) {
        Ok("yes") => check("reboot", "Ожидается перезагрузка", "warn", "Windows ждёт перезагрузки: установки могут завершаться ошибкой.", "E-INSTALL-1603", false),
        _ => check("reboot", "Ожидается перезагрузка", "ok", "Не требуется.", "", false),
    });

    // Windows Installer busy
    let busy = run_powershell(
        "try { $m=[System.Threading.Mutex]::OpenExisting('Global\\_MSIExecute'); $m.Dispose(); 'busy' } catch { 'free' }",
    );
    checks.push(match busy.as_deref().map(str::trim) {
        Ok("busy") => check("msi", "Windows Installer", "warn", "Сейчас выполняется другая установка (MSI).", "E-LOCK-004", false),
        _ => check("msi", "Windows Installer", "ok", "Свободен.", "", false),
    });

    // Log directory writable
    let dir = log_dir();
    let writable = fs::create_dir_all(&dir).is_ok()
        && OpenOptions::new().create(true).append(true).open(dir.join("probe.tmp")).is_ok();
    let _ = fs::remove_file(dir.join("probe.tmp"));
    checks.push(if writable {
        check("logs", "Папка логов", "ok", dir.display().to_string(), "", false)
    } else {
        check("logs", "Папка логов", "warn", format!("Нет доступа на запись: {}", dir.display()), "E-PERM-001", false)
    });

    // AppData paths
    let missing: Vec<&str> = ["APPDATA", "LOCALAPPDATA"]
        .into_iter()
        .filter(|name| !std::env::var_os(name).map(PathBuf::from).is_some_and(|p| p.is_dir()))
        .collect();
    checks.push(if missing.is_empty() {
        check("appdata", "Папки AppData", "ok", "Найдены.", "", false)
    } else {
        check("appdata", "Папки AppData", "fail", format!("Недоступны: {}", missing.join(", ")), "E-PATH-002", false)
    });

    // Bundled catalog
    checks.push(match crate::catalog::bundled() {
        Ok(catalog) => check("catalog", "Каталог программ", "ok", format!("Записей: {}", catalog.apps.len()), "", false),
        Err(error) => check("catalog", "Каталог программ", "fail", error, "E-PATH-002", false),
    });

    for item in checks.iter().filter(|c| c.status != "ok") {
        log("WARN", &format!("preflight {} -> {} {}", item.id, item.status, item.detail));
    }
    checks
}

fn network_ok() -> bool {
    let script = "try { (Invoke-WebRequest -Uri 'https://www.msftconnecttest.com/connecttest.txt' -UseBasicParsing -TimeoutSec 8).StatusCode } catch { 0 }";
    run_powershell(script).map(|s| s.trim() == "200").unwrap_or(false)
}

/// Applies a safe automatic fix. Returns a human message.
pub fn fix(id: &str) -> Result<String, String> {
    log("INFO", &format!("fix requested: {id}"));
    match id {
        "winget_sources" => {
            crate::installer::winget_run(&["source", "reset", "--force"], 120)?;
            crate::installer::winget_run(&["source", "update"], 120)?;
            Ok("Источники winget сброшены и обновлены.".into())
        }
        _ => Err("для этой проверки нет автоматического исправления".into()),
    }
}

/// Writes a plain-text diagnostic report (checks + log tail) and returns its path.
pub fn export_report() -> Result<String, String> {
    let mut text = String::from("Setup Hub diagnostic report\n===========================\n\n");
    for item in run_checks() {
        text.push_str(&format!("[{}] {} {}: {}\n", item.status, item.code, item.title, item.detail));
    }
    text.push_str("\n--- log tail ---\n");
    if let Ok(log) = fs::read_to_string(log_path()) {
        let lines: Vec<&str> = log.lines().collect();
        let start = lines.len().saturating_sub(200);
        text.push_str(&lines[start..].join("\n"));
    }
    let path = data_dir().join("diagnostic-report.txt");
    fs::create_dir_all(data_dir()).map_err(|e| e.to_string())?;
    fs::write(&path, text).map_err(|e| e.to_string())?;
    let _ = hidden_command("explorer.exe").arg(format!("/select,{}", path.display())).spawn();
    Ok(path.display().to_string())
}
