//! "Windows to USB": writes an installation flash drive straight from the app.
//!
//! Safety rules enforced both here and inside the PowerShell script:
//! * only disks reporting BusType USB are ever listed or written,
//! * boot/system disks are always rejected,
//! * the ISO path must be an existing `.iso` file.
//!
//! "Stripped" variants never ship a modified Windows image. They use the
//! original ISO plus an `autounattend.xml` and a first-logon script that apply
//! the tweaks after Windows is installed.

use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::process::Stdio;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use crate::tweaks::{classify_error, hidden_command, run_powershell};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsbDisk {
    pub number: u32,
    pub name: String,
    pub size_gb: f64,
    pub style: String,
    pub letters: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct UsbProgress {
    percent: f64,
    phase: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Variant {
    Original,
    Bypass,
    Lite,
    Gaming,
    Minimal,
}

impl Variant {
    fn parse(id: &str) -> Option<Self> {
        match id {
            "original" => Some(Self::Original),
            "bypass" => Some(Self::Bypass),
            "lite" => Some(Self::Lite),
            "gaming" => Some(Self::Gaming),
            "minimal" => Some(Self::Minimal),
            _ => None,
        }
    }

    fn has_unattend(self) -> bool {
        self != Self::Original
    }

    fn has_first_logon(self) -> bool {
        matches!(self, Self::Lite | Self::Gaming | Self::Minimal)
    }
}

// ----------------------------------------------------------------- listing

pub fn list_usb_disks() -> Result<Vec<UsbDisk>, String> {
    let script = r##"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
try {
    $items = @(Get-Disk | Where-Object { $_.BusType -eq 'USB' -and -not $_.IsBoot -and -not $_.IsSystem } | ForEach-Object {
        $letters = (Get-Partition -DiskNumber $_.Number -ErrorAction SilentlyContinue | Where-Object { $_.DriveLetter } | ForEach-Object { [string]$_.DriveLetter }) -join ', '
        [pscustomobject]@{
            number = [int]$_.Number
            name = [string]$_.FriendlyName
            sizeGb = [math]::Round($_.Size / 1GB, 1)
            style = [string]$_.PartitionStyle
            letters = [string]$letters
        }
    })
    if ($items.Count -eq 0) { Write-Output '[]' } else { Write-Output (ConvertTo-Json -InputObject $items -Compress) }
} catch {
    Write-Output ('SETUP_HUB_ERR:' + $_.Exception.GetType().Name + ':' + $_.Exception.Message)
    exit 1
}
"##;

    let report = run_powershell(script)?;
    let start = report
        .find('[')
        .ok_or("USB drive list is empty or unreadable")?;
    let end = report.rfind(']').ok_or("USB drive list is unreadable")?;
    serde_json::from_str::<Vec<UsbDisk>>(&report[start..=end])
        .map_err(|error| format!("could not read the USB drive list: {error}"))
}

// ----------------------------------------------------------------- writing

pub fn create_installer(
    app: &AppHandle,
    iso_path: &str,
    disk_number: u32,
    variant_id: &str,
    scheme: &str,
) -> Result<String, String> {
    let variant = Variant::parse(variant_id).ok_or("unknown Windows variant")?;
    let style = match scheme {
        "gpt" => "GPT",
        "mbr" => "MBR",
        _ => return Err("unknown partition scheme".into()),
    };

    if disk_number > 999 {
        return Err("invalid disk number".into());
    }
    if iso_path.chars().any(|ch| ch.is_control()) {
        return Err("invalid ISO path".into());
    }
    let path = Path::new(iso_path);
    let is_iso = path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("iso"));
    if !is_iso || !path.is_file() {
        return Err("ISO file was not found".into());
    }

    let unattend = if variant.has_unattend() {
        unattend_xml(variant.has_first_logon())
    } else {
        String::new()
    };
    let first_logon = if variant.has_first_logon() {
        first_logon_script(variant)
    } else {
        String::new()
    };

    let script = INSTALL_SCRIPT
        .replace("__ISO__", &iso_path.replace('\'', "''"))
        .replace("__DISK__", &disk_number.to_string())
        .replace("__STYLE__", style)
        .replace(
            "__HAS_UNATTEND__",
            if variant.has_unattend() { "1" } else { "0" },
        )
        .replace("__UNATTEND__", &unattend)
        .replace("__FIRSTLOGON__", &first_logon);

    emit(app, 0.0, "Запуск");

    let mut child = hidden_command("powershell")
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-WindowStyle",
            "Hidden",
            "-Command",
            &script,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("could not start powershell: {error}"))?;

    let mut report = String::new();
    let mut finished_ok = false;

    if let Some(stdout) = child.stdout.take() {
        for chunk in BufReader::new(stdout).split(b'\n') {
            let Ok(chunk) = chunk else { break };
            let line = String::from_utf8_lossy(&chunk).trim().to_string();
            if line.is_empty() {
                continue;
            }
            if let Some(rest) = line.strip_prefix("PROGRESS:") {
                let mut parts = rest.splitn(2, ':');
                let percent = parts
                    .next()
                    .and_then(|value| value.trim().parse::<f64>().ok())
                    .unwrap_or(0.0);
                emit(app, percent, parts.next().unwrap_or("").trim());
            } else if line == "SETUP_HUB_OK" {
                finished_ok = true;
            } else {
                report.push_str(&line);
                report.push('\n');
            }
        }
    }

    let mut stderr_text = String::new();
    if let Some(mut stderr) = child.stderr.take() {
        let _ = stderr.read_to_string(&mut stderr_text);
    }
    let status = child
        .wait()
        .map_err(|error| format!("powershell did not finish: {error}"))?;

    if status.success() && finished_ok {
        return Ok("Установочная флешка готова. Загрузитесь с неё через меню выбора устройства (Boot Menu) компьютера.".into());
    }

    report.push_str(&stderr_text);
    Err(classify_error(&report, status.to_string()))
}

fn emit(app: &AppHandle, percent: f64, phase: &str) {
    let _ = app.emit(
        "usb-progress",
        UsbProgress {
            percent,
            phase: phase.to_string(),
        },
    );
}

// ------------------------------------------------------------ PowerShell

const INSTALL_SCRIPT: &str = r##"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$iso = '__ISO__'
$diskNumber = __DISK__
$style = '__STYLE__'
$hasUnattend = __HAS_UNATTEND__
$mounted = $false
function Step([int]$percent, [string]$phase) { Write-Output ('PROGRESS:' + $percent + ':' + $phase) }
try {
    Step 2 'Проверка флешки'
    $disk = Get-Disk -Number $diskNumber -ErrorAction Stop
    if ($disk.BusType -ne 'USB') { throw 'Выбранный диск не является USB-накопителем.' }
    if ($disk.IsBoot -or $disk.IsSystem) { throw 'Нельзя выбрать системный диск.' }
    if ($disk.Size -lt 7GB) { throw 'Для установочной флешки нужно не менее 8 ГБ.' }

    Step 5 'Подключение образа'
    $image = Mount-DiskImage -ImagePath $iso -PassThru -ErrorAction Stop
    $mounted = $true
    $volume = $null
    for ($try = 0; $try -lt 12; $try++) {
        $candidate = $image | Get-Volume -ErrorAction SilentlyContinue
        if ($candidate -and $candidate.DriveLetter) { $volume = $candidate; break }
        Start-Sleep -Milliseconds 700
    }
    if (-not $volume) { throw 'Не удалось подключить ISO-образ.' }
    $src = [string]$volume.DriveLetter + ':\'
    if (-not (Test-Path -LiteralPath (Join-Path $src 'sources')) -or -not (Test-Path -LiteralPath (Join-Path $src 'boot'))) {
        throw 'Это не установочный образ Windows.'
    }
    $wim = Join-Path $src 'sources\install.wim'
    $esd = Join-Path $src 'sources\install.esd'
    $hasWim = Test-Path -LiteralPath $wim
    $hasEsd = Test-Path -LiteralPath $esd
    if (-not ($hasWim -or $hasEsd)) { throw 'В образе нет install.wim или install.esd.' }
    if ($hasEsd -and ((Get-Item -LiteralPath $esd).Length -ge 4294967295)) {
        throw 'install.esd больше 4 ГБ и не помещается на FAT32. Используйте официальный ISO с install.wim.'
    }

    Step 10 'Подготовка флешки'
    Set-Disk -Number $diskNumber -IsOffline $false -ErrorAction SilentlyContinue
    Set-Disk -Number $diskNumber -IsReadOnly $false -ErrorAction SilentlyContinue
    if ($disk.PartitionStyle -ne 'RAW') {
        Clear-Disk -Number $diskNumber -RemoveData -RemoveOEM -Confirm:$false -ErrorAction Stop
    }
    Initialize-Disk -Number $diskNumber -PartitionStyle $style -ErrorAction Stop
    $diskSize = (Get-Disk -Number $diskNumber).Size
    if ($diskSize -gt 32GB) {
        $part = New-Partition -DiskNumber $diskNumber -Size 30GB -AssignDriveLetter -ErrorAction Stop
    } else {
        $part = New-Partition -DiskNumber $diskNumber -UseMaximumSize -AssignDriveLetter -ErrorAction Stop
    }
    if ($style -eq 'MBR') {
        Set-Partition -DiskNumber $diskNumber -PartitionNumber $part.PartitionNumber -IsActive $true -ErrorAction Stop
    }
    Format-Volume -Partition $part -FileSystem FAT32 -NewFileSystemLabel 'SETUPHUB' -Force -Confirm:$false -ErrorAction Stop | Out-Null
    $letter = (Get-Partition -DiskNumber $diskNumber -PartitionNumber $part.PartitionNumber).DriveLetter
    if (-not $letter) { throw 'Не удалось получить букву диска флешки.' }
    $dst = [string]$letter + ':\'

    Step 15 'Копирование файлов Windows'
    $measure = Get-ChildItem -LiteralPath $src -Recurse -File -Force -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -ne 'install.wim' } | Measure-Object -Property Length -Sum
    $total = [double]$measure.Sum
    if ($total -le 0) { $total = 1 }
    $roboArgs = @(('"' + $src.TrimEnd('\') + '\."'), ('"' + $dst.TrimEnd('\') + '\."'), '/E', '/R:1', '/W:1', '/NFL', '/NDL', '/NJH', '/NJS', '/NP', '/XF', 'install.wim')
    $proc = Start-Process -FilePath 'robocopy.exe' -ArgumentList $roboArgs -PassThru -WindowStyle Hidden
    $null = $proc.Handle
    while (-not $proc.HasExited) {
        Start-Sleep -Seconds 2
        $copied = (Get-ChildItem -LiteralPath $dst -Recurse -File -Force -ErrorAction SilentlyContinue | Measure-Object -Property Length -Sum).Sum
        $copied = [double]$copied
        $pct = 15 + [math]::Min(60, [math]::Floor(60 * $copied / $total))
        Step ([int]$pct) 'Копирование файлов Windows'
    }
    $proc.WaitForExit()
    if ($proc.ExitCode -ge 8) { throw ('Копирование завершилось с ошибкой (robocopy ' + $proc.ExitCode + ').') }

    if ($hasWim) {
        Step 78 'Копирование образа install.wim (это самый долгий этап)'
        New-Item -ItemType Directory -Path (Join-Path $dst 'sources') -Force | Out-Null
        $wimSize = (Get-Item -LiteralPath $wim).Length
        if ($wimSize -ge 4294967295) {
            $swm = Join-Path $dst 'sources\install.swm'
            & dism.exe /Split-Image ('/ImageFile:' + $wim) ('/SWMFile:' + $swm) /FileSize:3800 | Out-Null
            if ($LASTEXITCODE -ne 0) { throw ('Не удалось разделить install.wim (DISM ' + $LASTEXITCODE + ').') }
        } else {
            Copy-Item -LiteralPath $wim -Destination (Join-Path $dst 'sources\install.wim') -Force
        }
    }

    if ($style -eq 'MBR') {
        Step 88 'Запись загрузочной области (BIOS)'
        $bootsect = Join-Path $src 'boot\bootsect.exe'
        if (Test-Path -LiteralPath $bootsect) {
            & $bootsect /nt60 ([string]$letter + ':') /force /mbr | Out-Null
        }
    }

    if ($hasUnattend -eq 1) {
        Step 93 'Настройка автоответов и скриптов'
        $unattend = @'
__UNATTEND__
'@
        Set-Content -LiteralPath (Join-Path $dst 'autounattend.xml') -Value $unattend -Encoding UTF8
        $logon = @'
__FIRSTLOGON__
'@
        if ($logon.Trim().Length -gt 0) {
            $oem = Join-Path $dst 'sources\$OEM$\$1\SetupHub'
            New-Item -ItemType Directory -Path $oem -Force | Out-Null
            Set-Content -LiteralPath (Join-Path $oem 'firstlogon.ps1') -Value $logon -Encoding UTF8
        }
    }

    Step 98 'Завершение'
    Write-Output 'PROGRESS:100:Готово'
    Write-Output 'SETUP_HUB_OK'
} catch {
    Write-Output ('SETUP_HUB_ERR:' + $_.Exception.GetType().Name + ':' + $_.Exception.Message)
    exit 1
} finally {
    if ($mounted) { Dismount-DiskImage -ImagePath $iso -ErrorAction SilentlyContinue | Out-Null }
}
"##;

// ------------------------------------------------------------ unattend

fn unattend_xml(with_first_logon: bool) -> String {
    let first_logon = if with_first_logon {
        r##"
  <settings pass="oobeSystem">
    <component name="Microsoft-Windows-Shell-Setup" processorArchitecture="amd64" publicKeyToken="31bf3856ad364e35" language="neutral" versionScope="nonSxS" xmlns:wcm="http://schemas.microsoft.com/WMIConfig/2002/State" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
      <FirstLogonCommands>
        <SynchronousCommand wcm:action="add">
          <Order>1</Order>
          <CommandLine>powershell.exe -NoProfile -ExecutionPolicy Bypass -File C:\SetupHub\firstlogon.ps1</CommandLine>
          <Description>Setup Hub: apply tweaks</Description>
        </SynchronousCommand>
      </FirstLogonCommands>
    </component>
  </settings>"##
    } else {
        ""
    };

    format!(
        r##"<?xml version="1.0" encoding="utf-8"?>
<unattend xmlns="urn:schemas-microsoft-com:unattend">
  <settings pass="windowsPE">
    <component name="Microsoft-Windows-Setup" processorArchitecture="amd64" publicKeyToken="31bf3856ad364e35" language="neutral" versionScope="nonSxS" xmlns:wcm="http://schemas.microsoft.com/WMIConfig/2002/State" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
      <RunSynchronous>
        <RunSynchronousCommand wcm:action="add">
          <Order>1</Order>
          <Path>reg.exe add HKLM\SYSTEM\Setup\LabConfig /v BypassTPMCheck /t REG_DWORD /d 1 /f</Path>
        </RunSynchronousCommand>
        <RunSynchronousCommand wcm:action="add">
          <Order>2</Order>
          <Path>reg.exe add HKLM\SYSTEM\Setup\LabConfig /v BypassSecureBootCheck /t REG_DWORD /d 1 /f</Path>
        </RunSynchronousCommand>
        <RunSynchronousCommand wcm:action="add">
          <Order>3</Order>
          <Path>reg.exe add HKLM\SYSTEM\Setup\LabConfig /v BypassRAMCheck /t REG_DWORD /d 1 /f</Path>
        </RunSynchronousCommand>
        <RunSynchronousCommand wcm:action="add">
          <Order>4</Order>
          <Path>reg.exe add HKLM\SYSTEM\Setup\LabConfig /v BypassCPUCheck /t REG_DWORD /d 1 /f</Path>
        </RunSynchronousCommand>
        <RunSynchronousCommand wcm:action="add">
          <Order>5</Order>
          <Path>reg.exe add HKLM\SYSTEM\Setup\LabConfig /v BypassStorageCheck /t REG_DWORD /d 1 /f</Path>
        </RunSynchronousCommand>
      </RunSynchronous>
    </component>
  </settings>
  <settings pass="specialize">
    <component name="Microsoft-Windows-Deployment" processorArchitecture="amd64" publicKeyToken="31bf3856ad364e35" language="neutral" versionScope="nonSxS" xmlns:wcm="http://schemas.microsoft.com/WMIConfig/2002/State" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
      <RunSynchronous>
        <RunSynchronousCommand wcm:action="add">
          <Order>1</Order>
          <Path>reg.exe add HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\OOBE /v BypassNRO /t REG_DWORD /d 1 /f</Path>
        </RunSynchronousCommand>
      </RunSynchronous>
    </component>
  </settings>{first_logon}
</unattend>"##
    )
}

// ------------------------------------------------- first-logon scripts

const FIRST_LOGON_HEADER: &str = r##"$ErrorActionPreference = 'SilentlyContinue'
$null = New-Item -ItemType Directory -Path 'C:\SetupHub' -Force
Start-Transcript -Path 'C:\SetupHub\firstlogon.log' -Append | Out-Null
function Set-Dw($Path, $Name, $Value) {
    if (-not (Test-Path $Path)) { New-Item -Path $Path -Force | Out-Null }
    New-ItemProperty -Path $Path -Name $Name -PropertyType DWord -Value $Value -Force | Out-Null
}
function Set-Str($Path, $Name, $Value) {
    if (-not (Test-Path $Path)) { New-Item -Path $Path -Force | Out-Null }
    New-ItemProperty -Path $Path -Name $Name -PropertyType String -Value $Value -Force | Out-Null
}
"##;

const FIRST_LOGON_LITE: &str = r##"
# ---- Lite: privacy, ads and preinstalled apps ----
Set-Dw 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\DataCollection' 'AllowTelemetry' 0
Set-Dw 'HKCU:\Software\Microsoft\Windows\CurrentVersion\AdvertisingInfo' 'Enabled' 0
foreach ($n in @('SubscribedContent-338388Enabled', 'SubscribedContent-338389Enabled', 'SubscribedContent-353694Enabled', 'SubscribedContent-353696Enabled', 'SystemPaneSuggestionsEnabled', 'SilentInstalledAppsEnabled', 'SoftLandingEnabled')) {
    Set-Dw 'HKCU:\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager' $n 0
}
Set-Dw 'HKCU:\Software\Policies\Microsoft\Windows\Explorer' 'DisableSearchBoxSuggestions' 1
Set-Dw 'HKCU:\Software\Policies\Microsoft\Windows\WindowsCopilot' 'TurnOffWindowsCopilot' 1
Set-Dw 'HKLM:\SOFTWARE\Policies\Microsoft\Dsh' 'AllowNewsAndInterests' 0
foreach ($s in @('DiagTrack', 'dmwappushservice')) {
    Set-Service -Name $s -StartupType Disabled
    Stop-Service -Name $s -Force
}
$apps = @(
    'Microsoft.BingNews', 'Microsoft.BingWeather', 'Microsoft.GetHelp', 'Microsoft.Getstarted',
    'Microsoft.MicrosoftOfficeHub', 'Microsoft.MicrosoftSolitaireCollection', 'Microsoft.People',
    'Microsoft.WindowsFeedbackHub', 'Microsoft.YourPhone', 'Microsoft.ZuneVideo',
    'Microsoft.MixedReality.Portal', 'Microsoft.3DBuilder', 'Microsoft.Microsoft3DViewer',
    'Microsoft.SkypeApp', 'Microsoft.WindowsMaps', 'Microsoft.Wallet', 'Microsoft.OneConnect',
    'Microsoft.Messaging', 'Microsoft.Print3D', 'Microsoft.PowerAutomateDesktop',
    'Clipchamp.Clipchamp', 'MicrosoftTeams', 'MSTeams', 'Microsoft.Todos',
    '*Disney*', '*Spotify*', '*TikTok*', '*Facebook*', '*Instagram*', '*Netflix*',
    '*LinkedIn*', '*CandyCrush*', '*Twitter*'
)
foreach ($app in $apps) {
    Get-AppxPackage -AllUsers -Name $app | Remove-AppxPackage -AllUsers
    Get-AppxProvisionedPackage -Online | Where-Object { $_.DisplayName -like $app } | Remove-AppxProvisionedPackage -Online | Out-Null
}
"##;

const FIRST_LOGON_GAMING: &str = r##"
# ---- Gaming: FPS and latency ----
$guidPattern = '([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})'
$created = powercfg -duplicatescheme e9a42b02-d5df-448d-aa00-03f14749eb61 2>&1 | Out-String
if ($created -match $guidPattern) { powercfg /setactive $Matches[1] } else { powercfg /setactive SCHEME_MIN }
Set-Dw 'HKCU:\System\GameConfigStore' 'GameDVR_Enabled' 0
Set-Dw 'HKCU:\Software\Microsoft\Windows\CurrentVersion\GameDVR' 'AppCaptureEnabled' 0
Set-Dw 'HKCU:\System\GameConfigStore' 'GameDVR_FSEBehaviorMode' 2
Set-Dw 'HKCU:\System\GameConfigStore' 'GameDVR_HonorUserFSEBehaviorMode' 1
Set-Dw 'HKCU:\System\GameConfigStore' 'GameDVR_DXGIHonorFSEWindowsCompatible' 1
Set-Str 'HKCU:\Control Panel\Mouse' 'MouseSpeed' '0'
Set-Str 'HKCU:\Control Panel\Mouse' 'MouseThreshold1' '0'
Set-Str 'HKCU:\Control Panel\Mouse' 'MouseThreshold2' '0'
Set-Str 'HKCU:\Control Panel\Accessibility\StickyKeys' 'Flags' '506'
Set-Str 'HKCU:\Control Panel\Accessibility\ToggleKeys' 'Flags' '58'
Set-Str 'HKCU:\Control Panel\Accessibility\Keyboard Response' 'Flags' '122'
$profile = 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile'
Set-Dw $profile 'NetworkThrottlingIndex' -1
Set-Dw $profile 'SystemResponsiveness' 0
Set-Dw ($profile + '\Tasks\Games') 'Priority' 6
Set-Str ($profile + '\Tasks\Games') 'Scheduling Category' 'High'
Set-Str ($profile + '\Tasks\Games') 'SFIO Priority' 'High'
Set-Dw 'HKLM:\SYSTEM\CurrentControlSet\Control\Power\PowerThrottling' 'PowerThrottlingOff' 1
foreach ($s in @('SysMain', 'MapsBroker', 'WerSvc')) {
    Set-Service -Name $s -StartupType Disabled
    Stop-Service -Name $s -Force
}
$tasks = @(
    @('\Microsoft\Windows\Application Experience\', 'Microsoft Compatibility Appraiser'),
    @('\Microsoft\Windows\Application Experience\', 'ProgramDataUpdater'),
    @('\Microsoft\Windows\Customer Experience Improvement Program\', 'Consolidator'),
    @('\Microsoft\Windows\Customer Experience Improvement Program\', 'UsbCeip')
)
foreach ($task in $tasks) { Disable-ScheduledTask -TaskPath $task[0] -TaskName $task[1] | Out-Null }
"##;

const FIRST_LOGON_MINIMAL: &str = r##"
# ---- Minimal: lightest possible system ----
Set-Dw 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\VisualEffects' 'VisualFXSetting' 2
Set-Dw 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize' 'EnableTransparency' 0
Set-Str 'HKCU:\Control Panel\Desktop\WindowMetrics' 'MinAnimate' '0'
Set-Dw 'HKCU:\Software\Microsoft\Windows\CurrentVersion\BackgroundAccessApplications' 'GlobalUserDisabled' 1
Set-Dw 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\DeliveryOptimization' 'DODownloadMode' 0
Set-Service -Name 'WSearch' -StartupType Disabled
Stop-Service -Name 'WSearch' -Force
fsutil behavior set disablelastaccess 1 | Out-Null
DISM.exe /Online /Set-ReservedStorageState /State:Disabled | Out-Null
Stop-Process -Name OneDrive -Force
$oneDrive = Join-Path $env:SystemRoot 'SysWOW64\OneDriveSetup.exe'
if (-not (Test-Path -LiteralPath $oneDrive)) { $oneDrive = Join-Path $env:SystemRoot 'System32\OneDriveSetup.exe' }
if (Test-Path -LiteralPath $oneDrive) { Start-Process -FilePath $oneDrive -ArgumentList '/uninstall' -Wait }
"##;

const FIRST_LOGON_FOOTER: &str = "\nStop-Transcript | Out-Null\n";

fn first_logon_script(variant: Variant) -> String {
    let mut script = String::from(FIRST_LOGON_HEADER);
    script.push_str(FIRST_LOGON_LITE);
    if matches!(variant, Variant::Gaming | Variant::Minimal) {
        script.push_str(FIRST_LOGON_GAMING);
    }
    if variant == Variant::Minimal {
        script.push_str(FIRST_LOGON_MINIMAL);
    }
    script.push_str(FIRST_LOGON_FOOTER);
    script
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scripts_never_break_here_strings() {
        for variant in [Variant::Lite, Variant::Gaming, Variant::Minimal] {
            let script = first_logon_script(variant);
            assert!(!script.contains("\n'@"));
        }
        assert!(!unattend_xml(true).contains("\n'@"));
    }

    #[test]
    fn variants_parse() {
        for id in ["original", "bypass", "lite", "gaming", "minimal"] {
            assert!(Variant::parse(id).is_some());
        }
        assert!(Variant::parse("other").is_none());
    }
}
