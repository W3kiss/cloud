param(
    [string]$Binary = "",
    [switch]$NoBackup
)

$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($Binary)) {
    $Binary = Join-Path $root "src-tauri\target\release\setup-hub.exe"
}

if (-not (Test-Path -LiteralPath $Binary)) {
    throw "Release binary was not found: $Binary. Run 'npm run tauri build' first."
}

$binaryItem = Get-Item -LiteralPath $Binary
Write-Host ("Before strip: {0:N2} MB" -f ($binaryItem.Length / 1MB))

$stripPath = $null

$fromPath = Get-Command "llvm-strip" -ErrorAction SilentlyContinue
if ($fromPath) {
    $stripPath = $fromPath.Source
}

if (-not $stripPath) {
    $candidate = Get-ChildItem `
        "$env:USERPROFILE\.rustup\toolchains\*\lib\rustlib\x86_64-pc-windows-msvc\bin\llvm-strip.exe" `
        -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($candidate) {
        $stripPath = $candidate.FullName
    }
}

if (-not $stripPath) {
    $candidate = Get-ChildItem `
        "$env:USERPROFILE\.rustup\toolchains\*\lib\rustlib\x86_64-pc-windows-msvc\bin\rust-objcopy.exe" `
        -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($candidate) {
        $stripPath = $candidate.FullName
    }
}

if (-not $stripPath) {
    $fromPath = Get-Command "strip" -ErrorAction SilentlyContinue
    if ($fromPath) {
        $stripPath = $fromPath.Source
    }
}

if (-not $stripPath) {
    throw "No strip tool found. Install it with: rustup component add llvm-tools-preview"
}

# Keep an untouched copy unless explicitly disabled. Cargo already embeds a
# stripped binary because `strip = "symbols"` is set in [profile.release],
# but this manual pass guarantees the final artifact is clean even when the
# profile is temporarily changed.
if (-not $NoBackup) {
    $backup = "${Binary}.unstripped"
    Copy-Item -LiteralPath $Binary -Destination $backup -Force
    Write-Host "Backup saved: $backup"
}

Write-Host "Strip tool: $stripPath"
& $stripPath --strip-all $Binary
if ($LASTEXITCODE -ne 0) {
    throw "Strip tool exited with code $LASTEXITCODE"
}

$binaryItem = Get-Item -LiteralPath $Binary
Write-Host ("After strip:  {0:N2} MB" -f ($binaryItem.Length / 1MB))
Write-Host "Done: $Binary"
