use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::Command;

use serde::Serialize;

const TELEMETRY_KEY: &str = r"HKLM\SOFTWARE\Policies\Microsoft\Windows\DataCollection";
const OPS_KEY: &str = r"HKCU\Software\SetupHub\Ops";
const SECURITY_KEY: &str = r"HKCU\Software\SetupHub\Security";
const CLASSIC_MENU_KEY: &str =
    r"HKCU\Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}\InprocServer32";
const CLASSIC_MENU_ROOT: &str =
    r"HKCU\Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}";
const GAME_DVR_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR";
const GAME_CONFIG_KEY: &str = r"HKCU\System\GameConfigStore";
const SEARCH_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Search";
const SEARCH_POLICY_KEY: &str = r"HKCU\Software\Policies\Microsoft\Windows\Explorer";
const LONG_PATHS_KEY: &str = r"HKLM\SYSTEM\CurrentControlSet\Control\FileSystem";
const POWER_KEY: &str = r"HKLM\SYSTEM\CurrentControlSet\Control\Power";
const HOOK_MARKER: &str = "# SETUP-HUB-CUSTOM-HOOK";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TweakStatus {
    pub id: String,
    pub enabled: bool,
    pub requires_admin: bool,
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

pub fn list_tweaks() -> Vec<TweakStatus> {
    vec![
        TweakStatus {
            id: "disable_telemetry".into(),
            enabled: query_dword(TELEMETRY_KEY, "AllowTelemetry") == Some(0),
            requires_admin: true,
        },
        TweakStatus {
            id: "classic_context_menu".into(),
            enabled: key_exists(CLASSIC_MENU_KEY),
            requires_admin: false,
        },
        TweakStatus {
            id: "disable_bing_search".into(),
            enabled: query_dword(SEARCH_POLICY_KEY, "DisableSearchBoxSuggestions") == Some(1),
            requires_admin: false,
        },
        TweakStatus {
            id: "disable_game_dvr".into(),
            enabled: query_dword(GAME_CONFIG_KEY, "GameDVR_Enabled") == Some(0)
                && query_dword(GAME_DVR_KEY, "AppCaptureEnabled") == Some(0),
            requires_admin: false,
        },
        TweakStatus {
            id: "enable_long_paths".into(),
            enabled: query_dword(LONG_PATHS_KEY, "LongPathsEnabled") == Some(1),
            requires_admin: true,
        },
        TweakStatus {
            id: "disable_hibernation".into(),
            enabled: query_dword(POWER_KEY, "HibernateEnabled") == Some(0),
            requires_admin: true,
        },
        TweakStatus {
            id: "cleanup_startup".into(),
            enabled: query_dword(OPS_KEY, "StartupCleaned") == Some(1),
            requires_admin: false,
        },
        TweakStatus {
            id: "custom_hook".into(),
            enabled: query_dword(SECURITY_KEY, "AllowCustomScripts") == Some(1),
            requires_admin: false,
        },
    ]
}

pub fn apply(id: &str, enabled: bool) -> Result<String, String> {
    match id {
        "disable_telemetry" => set_telemetry(enabled),
        "classic_context_menu" => set_classic_menu(enabled),
        "disable_bing_search" => set_bing_search(enabled),
        "disable_game_dvr" => set_game_dvr(enabled),
        "enable_long_paths" => set_long_paths(enabled),
        "disable_hibernation" => set_hibernation(enabled),
        "cleanup_startup" => set_startup_cleanup(enabled),
        "custom_hook" => set_custom_hook(enabled),
        _ => Err("unknown tweak".into()),
    }
}

fn set_telemetry(enabled: bool) -> Result<String, String> {
    let script = if enabled {
        r#"
$ErrorActionPreference = 'Stop'
try {
    New-Item -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\DataCollection' -Force | Out-Null
    Set-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\DataCollection' -Name 'AllowTelemetry' -Type DWord -Value 0
    foreach ($service in @('DiagTrack', 'dmwappushservice')) {
        $item = Get-Service -Name $service -ErrorAction SilentlyContinue
        if ($null -ne $item) {
            Set-Service -Name $service -StartupType Disabled -ErrorAction Stop
            Stop-Service -Name $service -Force -ErrorAction SilentlyContinue
        }
    }
    Write-Output 'SETUP_HUB_OK'
} catch {
    Write-Output ('SETUP_HUB_ERR:' + $_.Exception.GetType().Name + ':' + $_.Exception.Message)
    exit 1
}
"#
    } else {
        r#"
$ErrorActionPreference = 'Stop'
try {
    New-Item -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\DataCollection' -Force | Out-Null
    Set-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\DataCollection' -Name 'AllowTelemetry' -Type DWord -Value 1
    $diag = Get-Service -Name 'DiagTrack' -ErrorAction SilentlyContinue
    if ($null -ne $diag) {
        Set-Service -Name 'DiagTrack' -StartupType Automatic -ErrorAction Stop
        Start-Service -Name 'DiagTrack' -ErrorAction SilentlyContinue
    }
    $push = Get-Service -Name 'dmwappushservice' -ErrorAction SilentlyContinue
    if ($null -ne $push) {
        Set-Service -Name 'dmwappushservice' -StartupType Manual -ErrorAction Stop
    }
    Write-Output 'SETUP_HUB_OK'
} catch {
    Write-Output ('SETUP_HUB_ERR:' + $_.Exception.GetType().Name + ':' + $_.Exception.Message)
    exit 1
}
"#
    };

    run_powershell(script)?;

    if enabled {
        Ok("Телеметрия отключена, службы DiagTrack и dmwappushservice остановлены.".into())
    } else {
        Ok("Политика телеметрии возвращена к стандартному значению.".into())
    }
}

fn set_classic_menu(enabled: bool) -> Result<String, String> {
    if enabled {
        run_reg(&["add", CLASSIC_MENU_KEY, "/ve", "/d", "", "/f"])?;
        Ok("Классическое контекстное меню включено. Перезапустите Проводник.".into())
    } else {
        match run_reg(&["delete", CLASSIC_MENU_ROOT, "/f"]) {
            Ok(()) => Ok("Современное меню Windows 11 возвращено.".into()),
            Err(message) if message == "not_found" => Ok("Классическое меню уже отключено.".into()),
            Err(message) => Err(message),
        }
    }
}

fn set_bing_search(enabled: bool) -> Result<String, String> {
    if enabled {
        set_dword(SEARCH_POLICY_KEY, "DisableSearchBoxSuggestions", 1)?;
        set_dword(SEARCH_KEY, "BingSearchEnabled", 0)?;
        set_dword(SEARCH_KEY, "CortanaConsent", 0)?;
        Ok("Веб-результаты и подсказки Bing в меню «Пуск» отключены. Перезапустите Проводник."
            .into())
    } else {
        match run_reg(&[
            "delete",
            SEARCH_POLICY_KEY,
            "/v",
            "DisableSearchBoxSuggestions",
            "/f",
        ]) {
            Ok(()) => {}
            Err(message) if message == "not_found" => {}
            Err(message) => return Err(message),
        }
        set_dword(SEARCH_KEY, "BingSearchEnabled", 1)?;
        set_dword(SEARCH_KEY, "CortanaConsent", 1)?;
        Ok("Веб-поиск и подсказки Bing в меню «Пуск» включены.".into())
    }
}

fn set_game_dvr(enabled: bool) -> Result<String, String> {
    let value = if enabled { 0 } else { 1 };
    set_dword(GAME_CONFIG_KEY, "GameDVR_Enabled", value)?;
    set_dword(GAME_DVR_KEY, "AppCaptureEnabled", value)?;

    if enabled {
        Ok("Фоновая запись Game DVR и Game Bar отключена.".into())
    } else {
        Ok("Фоновая запись Game DVR и Game Bar включена.".into())
    }
}

fn set_long_paths(enabled: bool) -> Result<String, String> {
    set_dword(LONG_PATHS_KEY, "LongPathsEnabled", u32::from(enabled))?;

    if enabled {
        Ok("Поддержка длинных путей включена. Изменение вступит в силу после перезагрузки."
            .into())
    } else {
        Ok("Поддержка длинных путей отключена.".into())
    }
}

fn set_hibernation(enabled: bool) -> Result<String, String> {
    let mode = if enabled { "off" } else { "on" };
    let output = hidden_command("powercfg")
        .args(["/h", mode])
        .output()
        .map_err(|error| format!("could not start powercfg: {error}"))?;

    if output.status.success() {
        return Ok(if enabled {
            "Гибернация и быстрый запуск отключены. Место на системном диске освобождено.".into()
        } else {
            "Гибернация и быстрый запуск включены.".into()
        });
    }

    let report = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Err(classify_error(&report, output.status.to_string()))
}

fn set_startup_cleanup(enabled: bool) -> Result<String, String> {
    if !enabled {
        set_flag(OPS_KEY, "StartupCleaned", false)?;
        return Ok("Флаг очистки сброшен. Удалённые записи не восстанавливаются.".into());
    }

    let script = r#"
$ErrorActionPreference = 'SilentlyContinue'
$names = @('OneDrive', 'MicrosoftTeams', 'com.squirrel.Teams.Teams', 'MicrosoftEdgeAutoLaunch', 'Spotify', 'Copilot')
foreach ($name in $names) {
    Remove-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run' -Name $name -ErrorAction SilentlyContinue
    Remove-ItemProperty -Path 'HKLM:\Software\Microsoft\Windows\CurrentVersion\Run' -Name $name -ErrorAction SilentlyContinue
}
$startup = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs\Startup'
foreach ($shortcut in @('OneDrive.lnk', 'Microsoft Teams.lnk')) {
    $path = Join-Path $startup $shortcut
    if (Test-Path -LiteralPath $path) {
        Remove-Item -LiteralPath $path -Force -ErrorAction SilentlyContinue
    }
}
New-Item -Path 'HKCU:\Software\SetupHub\Ops' -Force | Out-Null
Set-ItemProperty -Path 'HKCU:\Software\SetupHub\Ops' -Name 'StartupCleaned' -Type DWord -Value 1
Write-Output 'SETUP_HUB_OK'
"#;

    run_powershell(script)?;
    Ok("Автозапуск очищен: OneDrive, Teams, Edge autolaunch и Spotify.".into())
}

fn set_custom_hook(enabled: bool) -> Result<String, String> {
    if !enabled {
        set_flag(SECURITY_KEY, "AllowCustomScripts", false)?;
        return Ok("Пользовательский хук отключён.".into());
    }

    let path = hook_path()?;
    let escaped_path = path.display().to_string().replace('\'', "''");
    let script = format!(
        r#"
$ErrorActionPreference = 'Stop'
try {{
    $path = '{path}'
    if (-not (Test-Path -LiteralPath $path)) {{
        New-Item -ItemType Directory -Path (Split-Path -LiteralPath $path) -Force | Out-Null
        @'
{marker}
# Add your idempotent automation below.
Write-Output 'Setup Hub custom hook is active.'
'@ | Set-Content -LiteralPath $path -Encoding UTF8
    }}
    $content = Get-Content -LiteralPath $path -Raw
    if (-not $content.StartsWith('{marker}')) {{
        throw 'Custom hook is missing the approval marker.'
    }}
    if ((Get-Item -LiteralPath $path).Length -gt 262144) {{
        throw 'Custom hook is larger than 256 KiB.'
    }}
    & $path
    if ($null -ne $LASTEXITCODE -and $LASTEXITCODE -ne 0) {{
        throw "Custom hook exited with $LASTEXITCODE"
    }}
    Write-Output 'SETUP_HUB_OK'
}} catch {{
    Write-Output ('SETUP_HUB_ERR:' + $_.Exception.GetType().Name + ':' + $_.Exception.Message)
    exit 1
}}
"#,
        path = escaped_path,
        marker = HOOK_MARKER
    );

    run_powershell(&script)?;
    set_flag(SECURITY_KEY, "AllowCustomScripts", true)?;
    Ok(format!("Хук выполнен. Файл: {}", path.display()))
}

pub fn hook_path() -> Result<PathBuf, String> {
    let appdata = std::env::var("APPDATA").map_err(|_| "APPDATA is missing".to_string())?;
    Ok(PathBuf::from(appdata)
        .join("SetupHub")
        .join("scripts")
        .join("custom.ps1"))
}

pub fn restart_explorer() -> Result<String, String> {
    let script = r#"
$ErrorActionPreference = 'SilentlyContinue'
Stop-Process -Name explorer -Force
Start-Sleep -Milliseconds 700
Start-Process explorer.exe
Write-Output 'SETUP_HUB_OK'
"#;
    run_powershell(script)?;
    Ok("Проводник перезапущен.".into())
}

pub fn is_admin() -> bool {
    // The high mandatory integrity SID is present only in an elevated token.
    // `net session` is unreliable on systems where the Server service is off,
    // so we inspect the token directly through `whoami /groups`.
    hidden_command("whoami")
        .args(["/groups"])
        .output()
        .map(|output| {
            output.status.success()
                && String::from_utf8_lossy(&output.stdout).contains("S-1-16-12288")
        })
        .unwrap_or(false)
}

pub fn restart_elevated() -> Result<(), String> {
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    let escaped = executable.display().to_string().replace('\'', "''");
    let script = format!("Start-Process -FilePath '{escaped}' -Verb RunAs");

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
        .spawn()
        .map_err(|error| format!("could not request elevation: {error}"))?;

    std::process::exit(0);
}

fn run_powershell(script: &str) -> Result<String, String> {
    let output = hidden_command("powershell")
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-WindowStyle",
            "Hidden",
            "-Command",
            script,
        ])
        .output()
        .map_err(|error| format!("could not start powershell: {error}"))?;

    let report = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let report = report.trim().to_string();

    if output.status.success() && !report.contains("SETUP_HUB_ERR:") {
        return Ok(report);
    }

    Err(classify_error(&report, output.status.to_string()))
}

fn run_reg(args: &[&str]) -> Result<(), String> {
    let output = hidden_command("reg")
        .args(args)
        .output()
        .map_err(|error| error.to_string())?;

    if output.status.success() {
        return Ok(());
    }

    let report = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Err(classify_error(&report, output.status.to_string()))
}

fn set_dword(key: &str, value: &str, data: u32) -> Result<(), String> {
    let data = data.to_string();
    run_reg(&[
        "add", key, "/v", value, "/t", "REG_DWORD", "/d", &data, "/f",
    ])
}

fn set_flag(key: &str, value: &str, enabled: bool) -> Result<(), String> {
    if enabled {
        run_reg(&["add", key, "/v", value, "/t", "REG_DWORD", "/d", "1", "/f"])
    } else {
        match run_reg(&["delete", key, "/v", value, "/f"]) {
            Ok(()) => Ok(()),
            Err(message) if message == "not_found" => Ok(()),
            Err(message) => Err(message),
        }
    }
}

fn query_dword(key: &str, value: &str) -> Option<u32> {
    let output = hidden_command("reg")
        .args(["query", key, "/v", value])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if !line.contains("REG_DWORD") {
            continue;
        }
        let token = line.split_whitespace().last()?;
        let hex = token.trim_start_matches("0x").trim_start_matches("0X");
        return u32::from_str_radix(hex, 16).ok();
    }
    None
}

fn key_exists(key: &str) -> bool {
    hidden_command("reg")
        .args(["query", key])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn classify_error(report: &str, status: String) -> String {
    let trimmed = report.trim();
    let lowered = trimmed.to_lowercase();

    if lowered.contains("access is denied")
        || lowered.contains("access denied")
        || lowered.contains("unauthorizedaccess")
        || lowered.contains("administrator privileges")
        || lowered.contains("requires administrator")
        || lowered.contains("elevation")
        || lowered.contains("отказано в доступе")
        || lowered.contains("нет прав")
        || lowered.contains("недостаточно прав")
    {
        return "admin_required".to_string();
    }

    if lowered.contains("unable to find")
        || lowered.contains("not found")
        || lowered.contains("не удается найти")
        || lowered.contains("не найден")
    {
        return "not_found".to_string();
    }

    if let Some(marker) = lowered.find("setup_hub_err:") {
        return trimmed[marker + "setup_hub_err:".len()..]
            .trim()
            .to_string();
    }

    if trimmed.is_empty() {
        format!("command exited with {status}")
    } else {
        trimmed.to_string()
    }
}
