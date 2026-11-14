//! "Stripped-down" (beta): aggressive, mostly reversible Windows trimming and
//! gaming-performance tweaks. Everything here is opt-in from the UI and the UI
//! warns the user before anything is applied.

use crate::tweaks::{
    TweakStatus, hidden_command, query_dword, remove_value, run_powershell, run_reg, set_dword,
    set_flag,
};

const STATE_KEY: &str = r"HKCU\Software\SetupHub\Stripped";
const SERVICES_KEY: &str = r"HKLM\SYSTEM\CurrentControlSet\Services";
const GAMES_KEY: &str =
    r"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\Tasks\Games";
const SYSTEM_PROFILE: &str =
    r"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile";

#[derive(Clone, Copy)]
enum Data {
    Dword {
        on: u32,
        off: Option<u32>,
    },
    Text {
        on: &'static str,
        off: Option<&'static str>,
    },
}

struct RegValue {
    key: &'static str,
    name: &'static str,
    data: Data,
}

#[derive(Clone, Copy)]
enum Restore {
    Automatic,
    Delayed,
    Manual,
}

#[derive(Clone, Copy)]
enum Action {
    TelemetryTasks,
    ReservedStorage,
    RemoveApps,
    RemoveOneDrive,
    CleanTemp,
}

enum Kind {
    Registry(&'static [RegValue]),
    Services(&'static [&'static str], Restore),
    PowerPlan,
    Flag(Action),
}

struct Def {
    id: &'static str,
    admin: bool,
    kind: Kind,
    on: &'static str,
    off: &'static str,
}

const fn dword(key: &'static str, name: &'static str, on: u32, off: Option<u32>) -> RegValue {
    RegValue {
        key,
        name,
        data: Data::Dword { on, off },
    }
}

const fn text(
    key: &'static str,
    name: &'static str,
    on: &'static str,
    off: Option<&'static str>,
) -> RegValue {
    RegValue {
        key,
        name,
        data: Data::Text { on, off },
    }
}

static DEFS: &[Def] = &[
    // ---------------------------------------------------------------- FPS
    Def {
        id: "fullscreen_optimizations",
        admin: false,
        kind: Kind::Registry(&[
            dword(
                r"HKCU\System\GameConfigStore",
                "GameDVR_FSEBehaviorMode",
                2,
                None,
            ),
            dword(
                r"HKCU\System\GameConfigStore",
                "GameDVR_HonorUserFSEBehaviorMode",
                1,
                None,
            ),
            dword(
                r"HKCU\System\GameConfigStore",
                "GameDVR_DXGIHonorFSEWindowsCompatible",
                1,
                None,
            ),
            dword(
                r"HKCU\System\GameConfigStore",
                "GameDVR_EFSEFeatureFlags",
                0,
                None,
            ),
        ]),
        on: "Оптимизации во весь экран отключены: игры получают настоящий эксклюзивный режим.",
        off: "Оптимизации во весь экран возвращены к стандартным.",
    },
    Def {
        id: "ultimate_power_plan",
        admin: false,
        kind: Kind::PowerPlan,
        on: "Включена схема питания «Максимальная производительность».",
        off: "Возвращена сбалансированная схема питания.",
    },
    Def {
        id: "power_throttling",
        admin: true,
        kind: Kind::Registry(&[dword(
            r"HKLM\SYSTEM\CurrentControlSet\Control\Power\PowerThrottling",
            "PowerThrottlingOff",
            1,
            None,
        )]),
        on: "Power Throttling отключён: фоновые и игровые процессы не замедляются. Нужна перезагрузка.",
        off: "Power Throttling возвращён к стандартному поведению.",
    },
    Def {
        id: "network_throttling",
        admin: true,
        kind: Kind::Registry(&[
            dword(
                SYSTEM_PROFILE,
                "NetworkThrottlingIndex",
                0xFFFF_FFFF,
                Some(10),
            ),
            dword(SYSTEM_PROFILE, "SystemResponsiveness", 0, Some(20)),
        ]),
        on: "Сетевое ограничение мультимедиа снято, отзывчивость системы повышена.",
        off: "Сетевое ограничение мультимедиа возвращено (значения по умолчанию).",
    },
    Def {
        id: "games_priority",
        admin: true,
        kind: Kind::Registry(&[
            dword(GAMES_KEY, "Priority", 6, Some(2)),
            text(GAMES_KEY, "Scheduling Category", "High", Some("Medium")),
            text(GAMES_KEY, "SFIO Priority", "High", Some("Normal")),
        ]),
        on: "Игровые процессы получили повышенный приоритет в планировщике.",
        off: "Приоритет игровых процессов возвращён к стандартному.",
    },
    Def {
        id: "mouse_acceleration",
        admin: false,
        kind: Kind::Registry(&[
            text(r"HKCU\Control Panel\Mouse", "MouseSpeed", "0", Some("1")),
            text(
                r"HKCU\Control Panel\Mouse",
                "MouseThreshold1",
                "0",
                Some("6"),
            ),
            text(
                r"HKCU\Control Panel\Mouse",
                "MouseThreshold2",
                "0",
                Some("10"),
            ),
        ]),
        on: "Ускорение мыши отключено. Изменение вступит в силу после перезахода в систему.",
        off: "Ускорение мыши возвращено. Изменение вступит в силу после перезахода в систему.",
    },
    Def {
        id: "sticky_keys_prompts",
        admin: false,
        kind: Kind::Registry(&[
            text(
                r"HKCU\Control Panel\Accessibility\StickyKeys",
                "Flags",
                "506",
                Some("510"),
            ),
            text(
                r"HKCU\Control Panel\Accessibility\ToggleKeys",
                "Flags",
                "58",
                Some("62"),
            ),
            text(
                r"HKCU\Control Panel\Accessibility\Keyboard Response",
                "Flags",
                "122",
                Some("126"),
            ),
        ]),
        on: "Окна залипания клавиш и фильтрации ввода отключены — они больше не свернут игру.",
        off: "Окна залипания клавиш и фильтрации ввода возвращены.",
    },
    Def {
        id: "visual_effects_performance",
        admin: false,
        kind: Kind::Registry(&[
            dword(
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\VisualEffects",
                "VisualFXSetting",
                2,
                Some(0),
            ),
            dword(
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
                "EnableTransparency",
                0,
                Some(1),
            ),
            text(
                r"HKCU\Control Panel\Desktop\WindowMetrics",
                "MinAnimate",
                "0",
                Some("1"),
            ),
        ]),
        on: "Визуальные эффекты и прозрачность отключены. Перезайдите в систему.",
        off: "Визуальные эффекты и прозрачность возвращены. Перезайдите в систему.",
    },
    Def {
        id: "background_apps",
        admin: false,
        kind: Kind::Registry(&[
            dword(
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\BackgroundAccessApplications",
                "GlobalUserDisabled",
                1,
                None,
            ),
            dword(
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Search",
                "BackgroundAppGlobalToggle",
                0,
                None,
            ),
        ]),
        on: "Фоновая работа приложений Microsoft Store отключена.",
        off: "Фоновая работа приложений Microsoft Store возвращена.",
    },
    Def {
        id: "ntfs_last_access",
        admin: true,
        kind: Kind::Registry(&[dword(
            r"HKLM\SYSTEM\CurrentControlSet\Control\FileSystem",
            "NtfsDisableLastAccessUpdate",
            0x8000_0001,
            Some(0x8000_0002),
        )]),
        on: "Запись времени последнего доступа к файлам отключена. Нужна перезагрузка.",
        off: "Запись времени доступа возвращена в управление системой. Нужна перезагрузка.",
    },
    Def {
        id: "delivery_optimization",
        admin: true,
        kind: Kind::Registry(&[dword(
            r"HKLM\SOFTWARE\Policies\Microsoft\Windows\DeliveryOptimization",
            "DODownloadMode",
            0,
            None,
        )]),
        on: "Раздача обновлений другим ПК (P2P) отключена — канал не расходуется в играх.",
        off: "Политика оптимизации доставки сброшена.",
    },
    Def {
        id: "widgets_news",
        admin: true,
        kind: Kind::Registry(&[dword(
            r"HKLM\SOFTWARE\Policies\Microsoft\Dsh",
            "AllowNewsAndInterests",
            0,
            None,
        )]),
        on: "Виджеты и новости отключены политикой. Перезапустите Проводник.",
        off: "Политика виджетов снята. Перезапустите Проводник.",
    },
    Def {
        id: "hvci_memory_integrity",
        admin: true,
        kind: Kind::Registry(&[dword(
            r"HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity",
            "Enabled",
            0,
            None,
        )]),
        on: "Целостность памяти (HVCI/VBS) отключена. Нужна перезагрузка.",
        off: "Параметр целостности памяти сброшен к системному. Нужна перезагрузка.",
    },
    // ----------------------------------------------------------- services
    Def {
        id: "svc_sysmain",
        admin: true,
        kind: Kind::Services(&["SysMain"], Restore::Automatic),
        on: "Служба SysMain (Superfetch) отключена.",
        off: "Служба SysMain (Superfetch) включена.",
    },
    Def {
        id: "svc_search_indexing",
        admin: true,
        kind: Kind::Services(&["WSearch"], Restore::Delayed),
        on: "Индексирование Windows Search отключено.",
        off: "Индексирование Windows Search включено.",
    },
    Def {
        id: "svc_maps",
        admin: true,
        kind: Kind::Services(&["MapsBroker"], Restore::Delayed),
        on: "Служба загруженных карт отключена.",
        off: "Служба загруженных карт включена.",
    },
    Def {
        id: "svc_error_reporting",
        admin: true,
        kind: Kind::Services(&["WerSvc"], Restore::Manual),
        on: "Служба отчётов об ошибках Windows отключена.",
        off: "Служба отчётов об ошибках Windows возвращена.",
    },
    Def {
        id: "svc_xbox",
        admin: true,
        kind: Kind::Services(
            &[
                "XblAuthManager",
                "XblGameSave",
                "XboxNetApiSvc",
                "XboxGipSvc",
            ],
            Restore::Manual,
        ),
        on: "Службы Xbox отключены.",
        off: "Службы Xbox возвращены.",
    },
    Def {
        id: "svc_print_spooler",
        admin: true,
        kind: Kind::Services(&["Spooler"], Restore::Automatic),
        on: "Диспетчер печати отключён.",
        off: "Диспетчер печати включён.",
    },
    // ----------------------------------------------------- composite flags
    Def {
        id: "telemetry_tasks",
        admin: true,
        kind: Kind::Flag(Action::TelemetryTasks),
        on: "Задачи телеметрии и сбора данных в планировщике отключены.",
        off: "Задачи телеметрии в планировщике включены.",
    },
    Def {
        id: "reserved_storage",
        admin: true,
        kind: Kind::Flag(Action::ReservedStorage),
        on: "Зарезервированное хранилище отключено — освобождается несколько ГБ после перезагрузки.",
        off: "Зарезервированное хранилище включено.",
    },
    Def {
        id: "remove_bloat_apps",
        admin: true,
        kind: Kind::Flag(Action::RemoveApps),
        on: "Предустановленные приложения удалены.",
        off: "Отметка снята. Удалённые приложения можно вернуть из Microsoft Store.",
    },
    Def {
        id: "remove_onedrive",
        admin: false,
        kind: Kind::Flag(Action::RemoveOneDrive),
        on: "OneDrive удалён. Локальные файлы в папке OneDrive остались на месте.",
        off: "Отметка снята. OneDrive можно установить заново с сайта Microsoft.",
    },
    Def {
        id: "clean_temp",
        admin: true,
        kind: Kind::Flag(Action::CleanTemp),
        on: "Временные файлы и кэш обновлений очищены.",
        off: "Отметка снята.",
    },
];

pub fn list() -> Vec<TweakStatus> {
    DEFS.iter()
        .map(|def| TweakStatus {
            id: def.id.to_string(),
            enabled: is_enabled(def),
            requires_admin: def.admin,
        })
        .collect()
}

pub fn apply(id: &str, enabled: bool) -> Result<String, String> {
    let def = DEFS
        .iter()
        .find(|def| def.id == id)
        .ok_or_else(|| "unknown tweak".to_string())?;

    match &def.kind {
        Kind::Registry(values) => apply_registry(values, enabled)?,
        Kind::Services(names, restore) => apply_services(names, *restore, enabled)?,
        Kind::PowerPlan => apply_power_plan(enabled)?,
        Kind::Flag(action) => return apply_flag(def, *action, enabled),
    }

    Ok(if enabled { def.on } else { def.off }.to_string())
}

fn is_enabled(def: &Def) -> bool {
    match &def.kind {
        Kind::Registry(values) => values.iter().all(value_is_on),
        Kind::Services(names, _) => {
            let starts: Vec<u32> = names
                .iter()
                .filter_map(|name| query_dword(&format!(r"{SERVICES_KEY}\{name}"), "Start"))
                .collect();
            !starts.is_empty() && starts.iter().all(|start| *start == 4)
        }
        Kind::PowerPlan => power_plan_is_fast(),
        Kind::Flag(_) => query_dword(STATE_KEY, def.id) == Some(1),
    }
}

// ------------------------------------------------------------------ registry

fn value_is_on(value: &RegValue) -> bool {
    match value.data {
        Data::Dword { on, .. } => query_dword(value.key, value.name) == Some(on),
        Data::Text { on, .. } => query_text(value.key, value.name).as_deref() == Some(on),
    }
}

fn apply_registry(values: &[RegValue], enabled: bool) -> Result<(), String> {
    for value in values {
        match value.data {
            Data::Dword { on, off } => match (enabled, off) {
                (true, _) => set_dword(value.key, value.name, on)?,
                (false, Some(off)) => set_dword(value.key, value.name, off)?,
                (false, None) => remove_value(value.key, value.name)?,
            },
            Data::Text { on, off } => match (enabled, off) {
                (true, _) => set_text(value.key, value.name, on)?,
                (false, Some(off)) => set_text(value.key, value.name, off)?,
                (false, None) => remove_value(value.key, value.name)?,
            },
        }
    }
    Ok(())
}

fn set_text(key: &str, name: &str, data: &str) -> Result<(), String> {
    run_reg(&["add", key, "/v", name, "/t", "REG_SZ", "/d", data, "/f"])
}

fn query_text(key: &str, name: &str) -> Option<String> {
    let output = hidden_command("reg")
        .args(["query", key, "/v", name])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if let Some(position) = line.find("REG_SZ") {
            return Some(line[position + "REG_SZ".len()..].trim().to_string());
        }
    }
    None
}

// ------------------------------------------------------------------ services

fn apply_services(names: &[&str], restore: Restore, enabled: bool) -> Result<(), String> {
    let list = names
        .iter()
        .map(|name| format!("'{name}'"))
        .collect::<Vec<_>>()
        .join(", ");

    let body = if enabled {
        r#"
    $item = Get-Service -Name $name -ErrorAction SilentlyContinue
    if ($null -ne $item) {
        Set-Service -Name $name -StartupType Disabled -ErrorAction Stop
        Stop-Service -Name $name -Force -ErrorAction SilentlyContinue
    }"#
    } else {
        match restore {
            Restore::Automatic => {
                r#"
    $item = Get-Service -Name $name -ErrorAction SilentlyContinue
    if ($null -ne $item) { Set-Service -Name $name -StartupType Automatic -ErrorAction Stop }"#
            }
            Restore::Manual => {
                r#"
    $item = Get-Service -Name $name -ErrorAction SilentlyContinue
    if ($null -ne $item) { Set-Service -Name $name -StartupType Manual -ErrorAction Stop }"#
            }
            Restore::Delayed => {
                r#"
    $item = Get-Service -Name $name -ErrorAction SilentlyContinue
    if ($null -ne $item) {
        & sc.exe config $name start= delayed-auto | Out-Null
        if ($LASTEXITCODE -ne 0) { throw "sc.exe exited with $LASTEXITCODE" }
    }"#
            }
        }
    };

    let script = format!(
        r#"
$ErrorActionPreference = 'Stop'
try {{
foreach ($name in @({list})) {{{body}
}}
    Write-Output 'SETUP_HUB_OK'
}} catch {{
    Write-Output ('SETUP_HUB_ERR:' + $_.Exception.GetType().Name + ':' + $_.Exception.Message)
    exit 1
}}
"#
    );
    run_powershell(&script).map(|_| ())
}

// --------------------------------------------------------------- power plan

fn power_plan_is_fast() -> bool {
    hidden_command("powercfg")
        .arg("/getactivescheme")
        .output()
        .map(|output| {
            let text = String::from_utf8_lossy(&output.stdout).to_lowercase();
            text.contains("ultimate performance")
                || text.contains("максимальная производительность")
                || text.contains("high performance")
                || text.contains("высокая производительность")
        })
        .unwrap_or(false)
}

fn apply_power_plan(enabled: bool) -> Result<(), String> {
    let script = if enabled {
        r#"
$ErrorActionPreference = 'Stop'
try {
    $guidPattern = '([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})'
    $guid = $null
    $existing = powercfg /list | Where-Object { $_ -match 'Ultimate Performance|Максимальная производительность' } | Select-Object -First 1
    if ($existing -and ($existing -match $guidPattern)) { $guid = $Matches[1] }
    if (-not $guid) {
        $created = powercfg -duplicatescheme e9a42b02-d5df-448d-aa00-03f14749eb61 2>&1 | Out-String
        if ($created -match $guidPattern) { $guid = $Matches[1] }
    }
    if ($guid) {
        powercfg /setactive $guid
        if ($LASTEXITCODE -ne 0) { throw "powercfg exited with $LASTEXITCODE" }
    } else {
        powercfg /setactive SCHEME_MIN
        if ($LASTEXITCODE -ne 0) { throw "powercfg exited with $LASTEXITCODE" }
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
    powercfg /setactive SCHEME_BALANCED
    if ($LASTEXITCODE -ne 0) { throw "powercfg exited with $LASTEXITCODE" }
    Write-Output 'SETUP_HUB_OK'
} catch {
    Write-Output ('SETUP_HUB_ERR:' + $_.Exception.GetType().Name + ':' + $_.Exception.Message)
    exit 1
}
"#
    };
    run_powershell(script).map(|_| ())
}

// ------------------------------------------------------------------- actions

fn apply_flag(def: &Def, action: Action, enabled: bool) -> Result<String, String> {
    let mut message = if enabled { def.on } else { def.off }.to_string();

    match (action, enabled) {
        (Action::TelemetryTasks, _) => {
            run_powershell(&telemetry_tasks_script(enabled)).map(|_| ())?
        }
        (Action::ReservedStorage, _) => {
            run_powershell(&reserved_storage_script(enabled)).map(|_| ())?
        }
        (Action::RemoveApps, true) => {
            run_powershell(REMOVE_APPS_SCRIPT)?;
        }
        (Action::RemoveOneDrive, true) => {
            run_powershell(REMOVE_ONEDRIVE_SCRIPT)?;
        }
        (Action::CleanTemp, true) => {
            let report = run_powershell(CLEAN_TEMP_SCRIPT)?;
            if let Some(freed) = report
                .lines()
                .find_map(|line| line.trim().strip_prefix("FREED_MB:"))
            {
                message = format!(
                    "Очистка завершена. Освобождено примерно {} МБ.",
                    freed.trim()
                );
            }
        }
        (_, false) => {}
    }

    set_flag(STATE_KEY, def.id, enabled)?;
    Ok(message)
}

fn telemetry_tasks_script(disable: bool) -> String {
    let verb = if disable {
        "Disable-ScheduledTask"
    } else {
        "Enable-ScheduledTask"
    };
    format!(
        r#"
$ErrorActionPreference = 'Stop'
try {{
    $tasks = @(
        @('\Microsoft\Windows\Application Experience\', 'Microsoft Compatibility Appraiser'),
        @('\Microsoft\Windows\Application Experience\', 'ProgramDataUpdater'),
        @('\Microsoft\Windows\Application Experience\', 'StartupAppTask'),
        @('\Microsoft\Windows\Customer Experience Improvement Program\', 'Consolidator'),
        @('\Microsoft\Windows\Customer Experience Improvement Program\', 'UsbCeip'),
        @('\Microsoft\Windows\Windows Error Reporting\', 'QueueReporting'),
        @('\Microsoft\Windows\DiskDiagnostic\', 'Microsoft-Windows-DiskDiagnosticDataCollector')
    )
    foreach ($task in $tasks) {{
        {verb} -TaskPath $task[0] -TaskName $task[1] -ErrorAction SilentlyContinue | Out-Null
    }}
    Write-Output 'SETUP_HUB_OK'
}} catch {{
    Write-Output ('SETUP_HUB_ERR:' + $_.Exception.GetType().Name + ':' + $_.Exception.Message)
    exit 1
}}
"#
    )
}

fn reserved_storage_script(disable: bool) -> String {
    let state = if disable { "Disabled" } else { "Enabled" };
    format!(
        r#"
$ErrorActionPreference = 'Stop'
try {{
    & DISM.exe /Online /Set-ReservedStorageState /State:{state} | Out-Null
    if ($LASTEXITCODE -ne 0) {{ throw "DISM exited with $LASTEXITCODE" }}
    Write-Output 'SETUP_HUB_OK'
}} catch {{
    Write-Output ('SETUP_HUB_ERR:' + $_.Exception.GetType().Name + ':' + $_.Exception.Message)
    exit 1
}}
"#
    )
}

const REMOVE_APPS_SCRIPT: &str = r#"
$ErrorActionPreference = 'SilentlyContinue'
$apps = @(
    'Microsoft.BingNews', 'Microsoft.BingWeather', 'Microsoft.BingFinance', 'Microsoft.BingSports',
    'Microsoft.GetHelp', 'Microsoft.Getstarted', 'Microsoft.MicrosoftOfficeHub',
    'Microsoft.MicrosoftSolitaireCollection', 'Microsoft.People', 'Microsoft.WindowsFeedbackHub',
    'Microsoft.YourPhone', 'Microsoft.ZuneVideo', 'Microsoft.549981C3F5F10',
    'Microsoft.MixedReality.Portal', 'Microsoft.3DBuilder', 'Microsoft.Microsoft3DViewer',
    'Microsoft.SkypeApp', 'Microsoft.WindowsMaps', 'Microsoft.Wallet', 'Microsoft.OneConnect',
    'Microsoft.Messaging', 'Microsoft.Print3D', 'Microsoft.PowerAutomateDesktop',
    'Clipchamp.Clipchamp', 'MicrosoftTeams', 'MSTeams', 'Microsoft.Todos',
    '*Disney*', '*Spotify*', '*TikTok*', '*Facebook*', '*Instagram*', '*Netflix*',
    '*LinkedIn*', '*CandyCrush*', '*Twitter*'
)
foreach ($app in $apps) {
    Get-AppxPackage -AllUsers -Name $app | Remove-AppxPackage -AllUsers
    Get-AppxProvisionedPackage -Online | Where-Object { $_.DisplayName -like $app } |
        Remove-AppxProvisionedPackage -Online | Out-Null
}
Write-Output 'SETUP_HUB_OK'
"#;

const REMOVE_ONEDRIVE_SCRIPT: &str = r#"
$ErrorActionPreference = 'SilentlyContinue'
Stop-Process -Name OneDrive -Force
Start-Sleep -Milliseconds 800
$setup = Join-Path $env:SystemRoot 'SysWOW64\OneDriveSetup.exe'
if (-not (Test-Path -LiteralPath $setup)) { $setup = Join-Path $env:SystemRoot 'System32\OneDriveSetup.exe' }
if (Test-Path -LiteralPath $setup) {
    Start-Process -FilePath $setup -ArgumentList '/uninstall' -Wait
}
Remove-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run' -Name 'OneDrive'
Write-Output 'SETUP_HUB_OK'
"#;

const CLEAN_TEMP_SCRIPT: &str = r#"
$ErrorActionPreference = 'SilentlyContinue'
$paths = @($env:TEMP, (Join-Path $env:SystemRoot 'Temp'), (Join-Path $env:SystemRoot 'SoftwareDistribution\Download'))
$before = 0
foreach ($path in $paths) {
    $before += (Get-ChildItem -LiteralPath $path -Recurse -Force -File | Measure-Object -Property Length -Sum).Sum
}
$update = Get-Service -Name wuauserv
$wasRunning = $update -and $update.Status -eq 'Running'
if ($wasRunning) { Stop-Service -Name wuauserv -Force }
foreach ($path in $paths) {
    Get-ChildItem -LiteralPath $path -Force | Remove-Item -Recurse -Force
}
if ($wasRunning) { Start-Service -Name wuauserv }
Clear-DnsClientCache
$after = 0
foreach ($path in $paths) {
    $after += (Get-ChildItem -LiteralPath $path -Recurse -Force -File | Measure-Object -Property Length -Sum).Sum
}
$freed = [math]::Max(0, [math]::Round(($before - $after) / 1MB))
Write-Output ('FREED_MB:' + $freed)
Write-Output 'SETUP_HUB_OK'
"#;

// ------------------------------------------------------------ restore point

pub fn create_restore_point() -> Result<String, String> {
    let script = r#"
$ErrorActionPreference = 'Stop'
try {
    Enable-ComputerRestore -Drive "$env:SystemDrive\"
    Checkpoint-Computer -Description 'Setup Hub: Stripped-down' -RestorePointType 'MODIFY_SETTINGS'
    Write-Output 'SETUP_HUB_OK'
} catch {
    Write-Output ('SETUP_HUB_ERR:' + $_.Exception.GetType().Name + ':' + $_.Exception.Message)
    exit 1
}
"#;
    run_powershell(script)?;
    Ok("Точка восстановления создана. Windows может пропустить её, если за последние 24 часа уже была создана другая.".into())
}
