//! System maintenance: services, startup entries, junk cleaner, context-menu manager.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::tweaks::{classify_error, hidden_command, run_powershell};

/// Runs a PowerShell script that prints one JSON array and parses it.
fn ps_json<T: for<'de> Deserialize<'de>>(script: &str) -> Result<Vec<T>, String> {
    let report = run_powershell(script)?;
    let start = report.find('[').ok_or("empty response")?;
    let end = report.rfind(']').ok_or("unreadable response")?;
    serde_json::from_str(&report[start..=end]).map_err(|e| format!("could not read the result: {e}"))
}

/// Escapes a value for a single-quoted PowerShell string.
fn ps_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn plain_text(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value.chars().count() <= max && !value.chars().any(|c| c.is_control())
}

const JSON_PRELUDE: &str = "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8\n$ErrorActionPreference = 'Stop'\n$ProgressPreference = 'SilentlyContinue'\n";

// ================================================================ services

/// Services that must never be changed from a UI (the system stops working without them).
const PROTECTED_SERVICES: &[&str] = &[
    "rpcss", "rpceptmapper", "dcomlaunch", "lsm", "dhcp", "dnscache", "eventlog", "windefend",
    "mpssvc", "bfe", "cryptsvc", "trustedinstaller", "winmgmt", "samss", "plugplay", "power",
    "profsvc", "gpsvc", "nsi", "systemeventsbroker", "timebrokersvc", "staterepository",
    "usermanager", "schedule", "audiosrv", "audioendpointbuilder", "wlansvc", "lanmanworkstation",
    "netprofm", "nlasvc", "keyiso", "sens", "themes", "wscsvc", "securityhealthservice",
    "coremessagingregistrar", "cdpsvc", "brokerinfrastructure", "tokenbroker", "msiserver",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceInfo {
    pub name: String,
    pub display: String,
    pub state: String,
    pub mode: String,
    pub delayed: bool,
    pub path: String,
    #[serde(default)]
    pub third_party: bool,
    #[serde(default)]
    pub protected: bool,
}

pub fn list_services() -> Result<Vec<ServiceInfo>, String> {
    let script = format!(
        "{JSON_PRELUDE}try {{\n\
        $items = @(Get-CimInstance Win32_Service | ForEach-Object {{ [pscustomobject]@{{ name=[string]$_.Name; display=[string]$_.DisplayName; state=[string]$_.State; mode=[string]$_.StartMode; delayed=[bool]$_.DelayedAutoStart; path=[string]$_.PathName }} }})\n\
        if ($items.Count -eq 0) {{ '[]' }} else {{ ConvertTo-Json -InputObject $items -Compress }}\n\
        }} catch {{ Write-Output ('SETUP_HUB_ERR:' + $_.Exception.Message); exit 1 }}"
    );
    let mut items: Vec<ServiceInfo> = ps_json(&script)?;
    for item in &mut items {
        let path = item.path.to_lowercase();
        item.third_party = !path.contains("\\windows\\") && !path.contains("svchost.exe");
        item.protected = PROTECTED_SERVICES.contains(&item.name.to_lowercase().as_str());
    }
    items.sort_by(|a, b| a.display.to_lowercase().cmp(&b.display.to_lowercase()));
    Ok(items)
}

pub fn set_service_mode(name: &str, mode: &str) -> Result<String, String> {
    let valid_name = !name.is_empty()
        && name.len() <= 80
        && name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-' | '@' | '$'));
    if !valid_name {
        return Err("invalid service name".into());
    }
    if PROTECTED_SERVICES.contains(&name.to_lowercase().as_str()) {
        return Err(format!("{name} — системная служба, изменение заблокировано"));
    }
    let value = match mode {
        "auto" => "auto",
        "delayed" => "delayed-auto",
        "manual" => "demand",
        "disabled" => "disabled",
        _ => return Err("unknown start mode".into()),
    };
    let output = hidden_command("sc.exe")
        .args(["config", name, "start=", value])
        .output()
        .map_err(|e| format!("could not start sc.exe: {e}"))?;
    let report = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    if output.status.success() {
        crate::diag::log("INFO", &format!("service {name} -> {mode}"));
        Ok(format!("{name}: режим «{mode}»"))
    } else {
        // `sc` reports "Access is denied" / "Отказано в доступе" without elevation.
        Err(crate::diag::annotate(
            &format!("sc config {name}"),
            classify_error(&report, output.status.to_string()),
        ))
    }
}

// ================================================================ startup

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupItem {
    pub id: String,
    pub name: String,
    pub command: String,
    pub location: String,
    pub enabled: bool,
}

const STARTUP_LOCATIONS: &str = r#"
$locs = @(
  @{ id='hkcu';   kind='reg';    run='HKCU:\Software\Microsoft\Windows\CurrentVersion\Run';                       ap='HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run' },
  @{ id='hklm';   kind='reg';    run='HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Run';                       ap='HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run' },
  @{ id='hklm32'; kind='reg';    run='HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Run';           ap='HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run32' },
  @{ id='fuser';  kind='folder'; run=(Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs\Startup');    ap='HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\StartupFolder' },
  @{ id='fall';   kind='folder'; run=(Join-Path $env:ProgramData 'Microsoft\Windows\Start Menu\Programs\Startup'); ap='HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\StartupFolder' }
)
"#;

pub fn list_startup() -> Result<Vec<StartupItem>, String> {
    let script = format!(
        "{JSON_PRELUDE}{STARTUP_LOCATIONS}\
        try {{\n\
        $out = New-Object System.Collections.ArrayList\n\
        foreach ($l in $locs) {{\n\
          $names = @()\n\
          if ($l.kind -eq 'reg') {{\n\
            if (Test-Path -LiteralPath $l.run) {{ $p = Get-ItemProperty -LiteralPath $l.run; $names = @($p.PSObject.Properties | Where-Object {{ $_.Name -notlike 'PS*' }} | ForEach-Object {{ @{{ n=$_.Name; c=[string]$_.Value }} }}) }}\n\
          }} elseif (Test-Path -LiteralPath $l.run) {{\n\
            $names = @(Get-ChildItem -LiteralPath $l.run -File -Force | Where-Object {{ $_.Name -ne 'desktop.ini' }} | ForEach-Object {{ @{{ n=$_.Name; c=$_.FullName }} }})\n\
          }}\n\
          foreach ($e in $names) {{\n\
            $enabled = $true\n\
            if (Test-Path -LiteralPath $l.ap) {{ $v = (Get-ItemProperty -LiteralPath $l.ap -Name $e.n -ErrorAction SilentlyContinue).($e.n); if ($v -is [byte[]] -and $v.Length -gt 0 -and (($v[0] -band 1) -eq 1)) {{ $enabled = $false }} }}\n\
            [void]$out.Add([pscustomobject]@{{ id=($l.id + '|' + $e.n); name=[string]$e.n; command=[string]$e.c; location=[string]$l.id; enabled=[bool]$enabled }})\n\
          }}\n\
        }}\n\
        if ($out.Count -eq 0) {{ '[]' }} else {{ ConvertTo-Json -InputObject @($out) -Compress }}\n\
        }} catch {{ Write-Output ('SETUP_HUB_ERR:' + $_.Exception.Message); exit 1 }}"
    );
    ps_json(&script)
}

pub fn set_startup_enabled(location: &str, name: &str, enabled: bool) -> Result<String, String> {
    if !["hkcu", "hklm", "hklm32", "fuser", "fall"].contains(&location) {
        return Err("unknown startup location".into());
    }
    if !plain_text(name, 260) {
        return Err("invalid entry name".into());
    }
    let first = if enabled { 2 } else { 3 };
    let script = format!(
        "{JSON_PRELUDE}{STARTUP_LOCATIONS}\
        try {{\n\
        $l = $locs | Where-Object {{ $_.id -eq {loc} }}\n\
        if (-not (Test-Path -LiteralPath $l.ap)) {{ New-Item -Path $l.ap -Force | Out-Null }}\n\
        $bytes = [byte[]]({first},0,0,0,0,0,0,0,0,0,0,0)\n\
        New-ItemProperty -LiteralPath $l.ap -Name {name} -PropertyType Binary -Value $bytes -Force | Out-Null\n\
        Write-Output 'SETUP_HUB_OK'\n\
        }} catch {{ Write-Output ('SETUP_HUB_ERR:' + $_.Exception.GetType().Name + ':' + $_.Exception.Message); exit 1 }}",
        loc = ps_quote(location),
        name = ps_quote(name),
    );
    run_powershell(&script)
        .map(|_| {
            crate::diag::log("INFO", &format!("startup {location}|{name} enabled={enabled}"));
            if enabled { format!("{name}: включено") } else { format!("{name}: отключено") }
        })
        .map_err(|e| crate::diag::annotate("startup toggle", e))
}

// ================================================================ junk cleaner

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JunkCategory {
    pub id: String,
    pub title: String,
    pub hint: String,
    pub size_mb: f64,
    pub files: u32,
    pub requires_admin: bool,
    pub recommended: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanReport {
    pub freed_mb: f64,
    pub skipped_files: u32,
    pub notes: Vec<String>,
}

struct Target {
    id: &'static str,
    title: &'static str,
    hint: &'static str,
    admin: bool,
    recommended: bool,
}

const TARGETS: &[Target] = &[
    Target { id: "user_temp", title: "Временные файлы пользователя", hint: "%TEMP%", admin: false, recommended: true },
    Target { id: "win_temp", title: "Временные файлы Windows", hint: "C:\\Windows\\Temp", admin: true, recommended: true },
    Target { id: "wu_cache", title: "Кэш Центра обновления Windows", hint: "SoftwareDistribution\\Download", admin: true, recommended: true },
    Target { id: "logs", title: "Системные логи и дампы сбоев", hint: "Windows\\Logs, Minidump, CrashDumps", admin: true, recommended: true },
    Target { id: "browser_cache", title: "Кэш браузеров", hint: "Chrome, Edge, Brave, Firefox", admin: false, recommended: true },
    Target { id: "thumbnails", title: "Кэш миниатюр", hint: "thumbcache_*.db", admin: false, recommended: false },
    Target { id: "prefetch", title: "Prefetch", hint: "Windows\\Prefetch (замедлит первый запуск программ)", admin: true, recommended: false },
];

fn env_path(name: &str) -> Option<PathBuf> {
    std::env::var_os(name).map(PathBuf::from).filter(|p| p.is_dir())
}

fn sub_dirs(path: &Path) -> Vec<PathBuf> {
    fs::read_dir(path)
        .map(|r| r.flatten().filter(|e| e.path().is_dir()).map(|e| e.path()).collect())
        .unwrap_or_default()
}

/// Directories whose *contents* are removed. Built here, never taken from the UI.
fn target_dirs(id: &str) -> Vec<PathBuf> {
    let sys = env_path("SystemRoot");
    let local = env_path("LOCALAPPDATA");
    let mut dirs = Vec::new();
    match id {
        "user_temp" => dirs.extend(env_path("TEMP")),
        "win_temp" => dirs.extend(sys.map(|s| s.join("Temp"))),
        "wu_cache" => dirs.extend(sys.map(|s| s.join("SoftwareDistribution").join("Download"))),
        "logs" => {
            if let Some(s) = &sys {
                dirs.push(s.join("Logs").join("CBS"));
                dirs.push(s.join("Minidump"));
                dirs.push(s.join("Logs").join("DISM"));
            }
            dirs.extend(local.map(|l| l.join("CrashDumps")));
        }
        "browser_cache" => {
            if let Some(l) = &local {
                for root in [
                    l.join("Google").join("Chrome").join("User Data"),
                    l.join("Microsoft").join("Edge").join("User Data"),
                    l.join("BraveSoftware").join("Brave-Browser").join("User Data"),
                ] {
                    for profile in sub_dirs(&root) {
                        let n = profile.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                        if n == "Default" || n.starts_with("Profile ") {
                            for cache in ["Cache", "Code Cache", "GPUCache"] {
                                dirs.push(profile.join(cache));
                            }
                        }
                    }
                }
                for profile in sub_dirs(&l.join("Mozilla").join("Firefox").join("Profiles")) {
                    dirs.push(profile.join("cache2"));
                }
            }
        }
        "thumbnails" => dirs.extend(local.map(|l| l.join("Microsoft").join("Windows").join("Explorer"))),
        "prefetch" => dirs.extend(sys.map(|s| s.join("Prefetch"))),
        _ => {}
    }
    dirs.into_iter().filter(|d| d.is_dir()).collect()
}

fn is_reparse(meta: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        meta.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        meta.file_type().is_symlink()
    }
}

fn counts_for(id: &str, name: &str) -> bool {
    // The Explorer folder holds more than thumbnails: only touch the cache databases.
    id != "thumbnails" || (name.to_lowercase().starts_with("thumbcache_") && name.to_lowercase().ends_with(".db"))
        || (id == "thumbnails" && name.to_lowercase().starts_with("iconcache_"))
}

fn measure_dir(id: &str, dir: &Path, top: bool) -> (u32, u64) {
    let (mut files, mut bytes) = (0u32, 0u64);
    let Ok(read) = fs::read_dir(dir) else { return (0, 0) };
    for entry in read.flatten() {
        let Ok(meta) = entry.metadata() else { continue };
        if is_reparse(&meta) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if top && id == "thumbnails" {
            if meta.is_file() && counts_for(id, &name) {
                files += 1;
                bytes += meta.len();
            }
            continue;
        }
        if meta.is_dir() {
            let (f, b) = measure_dir(id, &entry.path(), false);
            files += f;
            bytes += b;
        } else {
            files += 1;
            bytes += meta.len();
        }
    }
    (files, bytes)
}

pub fn scan_junk() -> Vec<JunkCategory> {
    TARGETS
        .iter()
        .map(|t| {
            let (mut files, mut bytes) = (0u32, 0u64);
            for dir in target_dirs(t.id) {
                let (f, b) = measure_dir(t.id, &dir, true);
                files += f;
                bytes += b;
            }
            JunkCategory {
                id: t.id.into(),
                title: t.title.into(),
                hint: t.hint.into(),
                size_mb: (bytes as f64 / 1_048_576.0 * 10.0).round() / 10.0,
                files,
                requires_admin: t.admin,
                recommended: t.recommended,
            }
        })
        .collect()
}

/// Deletes everything inside `dir`, returning (freed bytes, skipped entries).
fn clear_dir(id: &str, dir: &Path, top: bool) -> (u64, u32) {
    let (mut freed, mut skipped) = (0u64, 0u32);
    let Ok(read) = fs::read_dir(dir) else { return (0, 0) };
    for entry in read.flatten() {
        let Ok(meta) = entry.metadata() else { continue };
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if is_reparse(&meta) {
            continue; // never follow junctions/symlinks out of the cache folder
        }
        if top && id == "thumbnails" && !(meta.is_file() && counts_for(id, &name)) {
            continue;
        }
        if meta.is_dir() {
            let (f, s) = clear_dir(id, &path, false);
            freed += f;
            skipped += s;
            if fs::remove_dir(&path).is_err() {
                skipped += 1;
            }
        } else {
            let size = meta.len();
            if fs::remove_file(&path).is_ok() {
                freed += size;
            } else {
                skipped += 1;
            }
        }
    }
    (freed, skipped)
}

fn service_control(action: &str, service: &str) {
    let _ = hidden_command("sc.exe").args([action, service]).output();
}

pub fn clean_junk(ids: Vec<String>) -> Result<CleanReport, String> {
    let admin = crate::tweaks::is_admin();
    let mut freed = 0u64;
    let mut skipped = 0u32;
    let mut notes = Vec::new();

    for id in ids {
        let Some(target) = TARGETS.iter().find(|t| t.id == id) else {
            continue;
        };
        if target.admin && !admin {
            notes.push(format!("«{}»: нужны права администратора — пропущено", target.title));
            continue;
        }
        if id == "wu_cache" {
            service_control("stop", "wuauserv");
            service_control("stop", "bits");
        }
        for dir in target_dirs(&id) {
            let (f, s) = clear_dir(&id, &dir, true);
            freed += f;
            skipped += s;
        }
        if id == "wu_cache" {
            service_control("start", "bits");
            service_control("start", "wuauserv");
        }
    }

    crate::diag::log("INFO", &format!("junk cleaned: {} bytes, {} skipped", freed, skipped));
    Ok(CleanReport {
        freed_mb: (freed as f64 / 1_048_576.0 * 10.0).round() / 10.0,
        skipped_files: skipped,
        notes,
    })
}

pub fn empty_recycle_bin() -> Result<String, String> {
    run_powershell(&format!(
        "{JSON_PRELUDE}try {{ Clear-RecycleBin -Force -ErrorAction Stop; Write-Output 'SETUP_HUB_OK' }} catch {{ Write-Output ('SETUP_HUB_ERR:' + $_.Exception.Message); exit 1 }}"
    ))
    .map(|_| "Корзина очищена".to_string())
}

// ================================================================ context menu

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuItem {
    pub scope: String,
    pub key: String,
    pub label: String,
    pub command: String,
    pub icon: String,
    pub disabled: bool,
    pub extended: bool,
    /// "user" (HKCU) or "machine" (HKLM)
    pub origin: String,
    pub ours: bool,
}

fn scope_path(scope: &str) -> Option<&'static str> {
    match scope {
        "files" => Some("*\\shell"),
        "folders" => Some("Directory\\shell"),
        "desktop" => Some("Directory\\Background\\shell"),
        _ => None,
    }
}

fn valid_key(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= 80
        && key.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-' | ' '))
}

pub fn list_menu() -> Result<Vec<MenuItem>, String> {
    let script = format!(
        "{JSON_PRELUDE}try {{\n\
        $out = New-Object System.Collections.ArrayList\n\
        $scopes = @{{ files='*\\shell'; folders='Directory\\shell'; desktop='Directory\\Background\\shell' }}\n\
        foreach ($s in $scopes.Keys) {{\n\
          foreach ($o in @(@{{ n='user'; r='HKCU:\\Software\\Classes' }}, @{{ n='machine'; r='HKLM:\\SOFTWARE\\Classes' }})) {{\n\
            $base = $o.r + '\\' + $scopes[$s]\n\
            if (-not (Test-Path -LiteralPath $base)) {{ continue }}\n\
            foreach ($k in Get-ChildItem -LiteralPath $base -ErrorAction SilentlyContinue) {{\n\
              $p = Get-ItemProperty -LiteralPath $k.PSPath -ErrorAction SilentlyContinue\n\
              $cmd = (Get-ItemProperty -LiteralPath ($k.PSPath + '\\command') -ErrorAction SilentlyContinue).'(default)'\n\
              $label = [string]$p.MUIVerb; if (-not $label) {{ $label = [string]$p.'(default)' }}; if (-not $label) {{ $label = $k.PSChildName }}\n\
              [void]$out.Add([pscustomobject]@{{ scope=$s; key=[string]$k.PSChildName; label=$label; command=[string]$cmd; icon=[string]$p.Icon; disabled=($null -ne $p.LegacyDisable); extended=($null -ne $p.Extended); origin=$o.n; ours=($k.PSChildName -like 'SetupHub.*') }})\n\
            }}\n\
          }}\n\
        }}\n\
        if ($out.Count -eq 0) {{ '[]' }} else {{ ConvertTo-Json -InputObject @($out) -Compress }}\n\
        }} catch {{ Write-Output ('SETUP_HUB_ERR:' + $_.Exception.Message); exit 1 }}"
    );
    ps_json(&script)
}

fn hive(origin: &str) -> Result<&'static str, String> {
    match origin {
        "user" => Ok("HKCU:\\Software\\Classes"),
        "machine" => Ok("HKLM:\\SOFTWARE\\Classes"),
        _ => Err("unknown registry origin".into()),
    }
}

pub fn add_menu_item(
    scope: &str,
    key: &str,
    label: &str,
    command: &str,
    icon: &str,
    extended: bool,
) -> Result<String, String> {
    let path = scope_path(scope).ok_or("unknown scope")?;
    if !valid_key(key) {
        return Err("key may only use letters, digits, space, '.', '_' and '-'".into());
    }
    if !plain_text(label, 80) || !plain_text(command, 500) {
        return Err("label and command are required".into());
    }
    if !icon.is_empty() && !plain_text(icon, 260) {
        return Err("invalid icon path".into());
    }
    let full_key = format!("SetupHub.{key}");
    let script = format!(
        "{JSON_PRELUDE}try {{\n\
        $base = 'HKCU:\\Software\\Classes\\{path}\\' + {key}\n\
        New-Item -Path $base -Force | Out-Null\n\
        New-ItemProperty -LiteralPath $base -Name 'MUIVerb' -Value {label} -PropertyType String -Force | Out-Null\n\
        if ({icon} -ne '') {{ New-ItemProperty -LiteralPath $base -Name 'Icon' -Value {icon} -PropertyType String -Force | Out-Null }}\n\
        if ({ext}) {{ New-ItemProperty -LiteralPath $base -Name 'Extended' -Value '' -PropertyType String -Force | Out-Null }}\n\
        New-Item -Path ($base + '\\command') -Force | Out-Null\n\
        Set-ItemProperty -LiteralPath ($base + '\\command') -Name '(default)' -Value {command}\n\
        Write-Output 'SETUP_HUB_OK'\n\
        }} catch {{ Write-Output ('SETUP_HUB_ERR:' + $_.Exception.GetType().Name + ':' + $_.Exception.Message); exit 1 }}",
        key = ps_quote(&full_key),
        label = ps_quote(label),
        icon = ps_quote(icon),
        command = ps_quote(command),
        ext = if extended { "$true" } else { "$false" },
    );
    run_powershell(&script)
        .map(|_| {
            crate::diag::log("INFO", &format!("context menu add {scope}/{full_key}"));
            format!("«{label}» добавлено")
        })
        .map_err(|e| crate::diag::annotate("context menu add", e))
}

pub fn set_menu_disabled(scope: &str, key: &str, origin: &str, disabled: bool) -> Result<String, String> {
    let path = scope_path(scope).ok_or("unknown scope")?;
    let root = hive(origin)?;
    if !valid_key(key) && !key.starts_with('{') {
        return Err("invalid key".into());
    }
    let script = format!(
        "{JSON_PRELUDE}try {{\n\
        $k = '{root}\\{path}\\' + {key}\n\
        if (-not (Test-Path -LiteralPath $k)) {{ throw 'not_found' }}\n\
        if ({off}) {{ New-ItemProperty -LiteralPath $k -Name 'LegacyDisable' -Value '' -PropertyType String -Force | Out-Null }} else {{ Remove-ItemProperty -LiteralPath $k -Name 'LegacyDisable' -ErrorAction SilentlyContinue }}\n\
        Write-Output 'SETUP_HUB_OK'\n\
        }} catch {{ Write-Output ('SETUP_HUB_ERR:' + $_.Exception.GetType().Name + ':' + $_.Exception.Message); exit 1 }}",
        key = ps_quote(key),
        off = if disabled { "$true" } else { "$false" },
    );
    run_powershell(&script)
        .map(|_| if disabled { "Пункт отключён".to_string() } else { "Пункт включён".to_string() })
        .map_err(|e| crate::diag::annotate("context menu toggle", e))
}

pub fn remove_menu_item(scope: &str, key: &str) -> Result<String, String> {
    let path = scope_path(scope).ok_or("unknown scope")?;
    // Only our own entries can be deleted; everything else can just be disabled.
    if !key.starts_with("SetupHub.") || !valid_key(key) {
        return Err("удалять можно только пункты, созданные в Setup Hub (остальные можно отключить)".into());
    }
    let script = format!(
        "{JSON_PRELUDE}try {{\n\
        Remove-Item -LiteralPath ('HKCU:\\Software\\Classes\\{path}\\' + {key}) -Recurse -Force\n\
        Write-Output 'SETUP_HUB_OK'\n\
        }} catch {{ Write-Output ('SETUP_HUB_ERR:' + $_.Exception.GetType().Name + ':' + $_.Exception.Message); exit 1 }}",
        key = ps_quote(key),
    );
    run_powershell(&script)
        .map(|_| "Пункт удалён".to_string())
        .map_err(|e| crate::diag::annotate("context menu remove", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ps_quote_doubles_quotes() {
        assert_eq!(ps_quote("a'b"), "'a''b'");
    }

    #[test]
    fn protected_services_are_blocked() {
        assert!(set_service_mode("RpcSs", "disabled").is_err());
        assert!(set_service_mode("bad name;", "manual").is_err());
        assert!(set_service_mode("Spooler", "bogus").is_err());
    }

    #[test]
    fn menu_keys_are_validated() {
        assert!(valid_key("Open Terminal"));
        assert!(!valid_key("a\\b"));
        assert!(remove_menu_item("files", "SomeoneElse").is_err());
    }

    #[test]
    fn startup_rejects_unknown_location() {
        assert!(set_startup_enabled("hkxx", "x", true).is_err());
    }
}
