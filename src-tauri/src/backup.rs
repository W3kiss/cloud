use std::fs;
use std::path::{Path, PathBuf};

pub fn restore_folder(
    source: String,
    scope: String,
    folder_name: String,
) -> Result<String, String> {
    let source_path = PathBuf::from(source.trim());
    if !source_path.is_dir() {
        return Err("source folder was not found".into());
    }
    if is_blocked_source(&source_path) {
        return Err("this source folder cannot be copied".into());
    }

    let folder_name = folder_name.trim();
    if !valid_folder_name(folder_name) {
        return Err("folder name must use english letters, numbers, spaces, dots or dashes".into());
    }

    let root = match scope.as_str() {
        "roaming" => std::env::var("APPDATA").map_err(|_| "APPDATA is missing".to_string())?,
        "local" => {
            std::env::var("LOCALAPPDATA").map_err(|_| "LOCALAPPDATA is missing".to_string())?
        }
        _ => return Err("unknown backup scope".into()),
    };

    let destination = PathBuf::from(root).join(folder_name);
    if paths_overlap(&source_path, &destination) {
        return Err("source and destination overlap".into());
    }

    let copied = copy_dir(&source_path, &destination)?;
    Ok(format!(
        "copied {copied} files to {}",
        destination.display()
    ))
}

fn valid_folder_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if name.len() > 64 || !first.is_ascii_alphanumeric() {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-' | ' '))
        && !name.ends_with('.')
        && !name.ends_with(' ')
}

fn is_blocked_source(path: &Path) -> bool {
    let Ok(canonical) = path.canonicalize() else {
        return true;
    };
    if canonical.parent().is_none() {
        return true;
    }
    let windir = std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".into());
    let source = canonical.to_string_lossy().to_lowercase();
    let windows = PathBuf::from(windir).to_string_lossy().to_lowercase();
    source == windows || source.starts_with(&(windows.clone() + "\\"))
}

fn paths_overlap(source: &Path, destination: &Path) -> bool {
    let source = source
        .canonicalize()
        .unwrap_or_else(|_| source.to_path_buf());
    let destination = destination
        .canonicalize()
        .unwrap_or_else(|_| destination.to_path_buf());
    destination.starts_with(&source) || source.starts_with(&destination)
}

fn copy_dir(source: &Path, destination: &Path) -> Result<u32, String> {
    fs::create_dir_all(destination).map_err(|error| error.to_string())?;
    let mut copied = 0u32;

    for entry in fs::read_dir(source).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let file_type = entry.file_type().map_err(|error| error.to_string())?;
        let target = destination.join(entry.file_name());
        if file_type.is_dir() {
            copied += copy_dir(&entry.path(), &target)?;
        } else if file_type.is_file() {
            fs::copy(entry.path(), target).map_err(|error| error.to_string())?;
            copied += 1;
        }
    }

    Ok(copied)
}

// ====================================================================
// Scan / pack / restore of AppData configuration folders (zip archives)
// ====================================================================

use std::collections::HashSet;
use std::io::{Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

/// Folders that only hold caches / transient data; skipped inside archives.
const CACHE_DIRS: &[&str] = &[
    "cache", "code cache", "gpucache", "gpu cache", "dawncache", "shadercache", "grshadercache",
    "crashpad", "crash reports", "logs", "log", "temp", "tmp", "cache2", "service worker",
    "media cache", "blob_storage", "component_crx_cache", "optimizationguidepredictionmodels",
];

/// Top-level folders that belong to Windows itself or are huge/unsafe to move.
const SKIP_TOP: &[&str] = &[
    "microsoft", "packages", "temp", "programs", "publishers", "d3dscache", "crashdumps",
    "comms", "connecteddevicesplatform", "history", "tempstate", "virtualstore", "setuphub",
    "windowsapps", "pip", "npm-cache", "nuget",
];

const KNOWN: &[(&str, &str)] = &[
    ("code", "Visual Studio Code"),
    ("discord", "Discord"),
    ("telegram desktop", "Telegram"),
    ("obs-studio", "OBS Studio"),
    ("spotify", "Spotify"),
    ("steam", "Steam"),
    ("google", "Google Chrome / Google"),
    ("mozilla", "Firefox"),
    ("opera software", "Opera"),
    ("brave-browser", "Brave"),
    ("bravesoftware", "Brave"),
    ("7-zip", "7-Zip"),
    ("notepad++", "Notepad++"),
    ("vlc", "VLC"),
    ("qbittorrent", "qBittorrent"),
    ("jetbrains", "JetBrains IDE"),
    ("git", "Git"),
    ("docker", "Docker Desktop"),
    ("zoom", "Zoom"),
    ("slack", "Slack"),
    ("whatsapp", "WhatsApp"),
    ("anydesk", "AnyDesk"),
    ("audacity", "Audacity"),
    ("gimp", "GIMP"),
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppFolder {
    pub root: String,
    pub name: String,
    pub size_mb: f64,
    pub files: u32,
    pub modified: u64,
    pub known: Option<String>,
    pub running: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupEntry {
    pub root: String,
    pub name: String,
    pub files: u32,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub version: u32,
    pub created: u64,
    pub computer: String,
    pub entries: Vec<BackupEntry>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupRequest {
    pub root: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BackupProgress {
    percent: f64,
    phase: String,
}

fn emit(app: &AppHandle, percent: f64, phase: &str) {
    let _ = app.emit(
        "backup-progress",
        BackupProgress { percent, phase: phase.to_string() },
    );
}

fn root_dir(root: &str) -> Result<PathBuf, String> {
    let var = match root {
        "roaming" => "APPDATA",
        "local" => "LOCALAPPDATA",
        _ => return Err("unknown backup root".into()),
    };
    std::env::var_os(var)
        .map(PathBuf::from)
        .filter(|p| p.is_dir())
        .ok_or_else(|| format!("{var} is missing"))
}

/// True for symlinks and junctions: never followed (avoids loops and escapes).
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

/// Only plain single path components are allowed in names coming from the UI or an archive.
fn safe_component(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.contains(['/', '\\', ':', '\0'])
        && !name.ends_with('.')
        && !name.ends_with(' ')
}

fn is_cache(name: &str) -> bool {
    CACHE_DIRS.contains(&name.to_lowercase().as_str())
}

/// (files, bytes) with caches excluded and a hard cap to keep scans fast.
fn measure(dir: &Path, include_cache: bool, budget: &mut u32) -> (u32, u64) {
    let mut files = 0u32;
    let mut bytes = 0u64;
    let Ok(read) = fs::read_dir(dir) else { return (0, 0) };
    for entry in read.flatten() {
        if *budget == 0 {
            break;
        }
        let Ok(meta) = entry.metadata() else { continue };
        if is_reparse(&meta) {
            continue;
        }
        if meta.is_dir() {
            if !include_cache && is_cache(&entry.file_name().to_string_lossy()) {
                continue;
            }
            let (f, b) = measure(&entry.path(), include_cache, budget);
            files += f;
            bytes += b;
        } else {
            *budget = budget.saturating_sub(1);
            files += 1;
            bytes += meta.len();
        }
    }
    (files, bytes)
}

fn running_process_names() -> HashSet<String> {
    crate::tweaks::run_powershell("(Get-Process | Select-Object -ExpandProperty ProcessName) -join \"`n\"")
        .map(|out| out.lines().map(|l| l.trim().to_lowercase()).filter(|l| !l.is_empty()).collect())
        .unwrap_or_default()
}

fn is_running(folder: &str, processes: &HashSet<String>) -> bool {
    let key = folder.to_lowercase().replace(' ', "");
    if key.len() < 3 {
        return false;
    }
    processes.iter().any(|p| p == &key || p.starts_with(&key) || (p.len() >= 4 && key.starts_with(p.as_str())))
}

pub fn scan_appdata() -> Result<Vec<AppFolder>, String> {
    let processes = running_process_names();
    let mut found = Vec::new();

    for root in ["roaming", "local"] {
        let Ok(base) = root_dir(root) else { continue };
        let Ok(read) = fs::read_dir(&base) else { continue };
        for entry in read.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if SKIP_TOP.contains(&name.to_lowercase().as_str()) || !safe_component(&name) {
                continue;
            }
            let Ok(meta) = entry.metadata() else { continue };
            if !meta.is_dir() || is_reparse(&meta) {
                continue;
            }
            let mut budget = 40_000u32;
            let (files, bytes) = measure(&entry.path(), false, &mut budget);
            if files == 0 {
                continue;
            }
            let modified = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let lowered = name.to_lowercase();
            found.push(AppFolder {
                root: root.to_string(),
                known: KNOWN.iter().find(|(k, _)| *k == lowered).map(|(_, v)| v.to_string()),
                running: is_running(&name, &processes),
                name,
                size_mb: (bytes as f64 / 1_048_576.0 * 10.0).round() / 10.0,
                files,
                modified,
            });
        }
    }
    found.sort_by(|a, b| b.known.is_some().cmp(&a.known.is_some()).then(a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    Ok(found)
}

fn collect_files(dir: &Path, rel: &Path, include_cache: bool, out: &mut Vec<(PathBuf, PathBuf)>) {
    let Ok(read) = fs::read_dir(dir) else { return };
    for entry in read.flatten() {
        let Ok(meta) = entry.metadata() else { continue };
        if is_reparse(&meta) {
            continue;
        }
        let name = entry.file_name();
        let rel_path = rel.join(&name);
        if meta.is_dir() {
            if !include_cache && is_cache(&name.to_string_lossy()) {
                continue;
            }
            collect_files(&entry.path(), &rel_path, include_cache, out);
        } else {
            out.push((entry.path(), rel_path));
        }
    }
}

fn zip_name(root: &str, name: &str, rel: &Path) -> String {
    let mut parts = vec![root.to_string(), name.to_string()];
    parts.extend(rel.components().map(|c| c.as_os_str().to_string_lossy().to_string()));
    parts.join("/")
}

pub fn create_backup(
    app: &AppHandle,
    items: Vec<BackupRequest>,
    destination: String,
    include_cache: bool,
) -> Result<String, String> {
    if items.is_empty() {
        return Err("nothing selected".into());
    }
    let dest = PathBuf::from(destination.trim());
    if dest.extension().map(|e| !e.eq_ignore_ascii_case("zip")).unwrap_or(true) {
        return Err("backup file must end with .zip".into());
    }
    if let Some(parent) = dest.parent() {
        if !parent.as_os_str().is_empty() && !parent.is_dir() {
            return Err("destination folder was not found".into());
        }
    }

    emit(app, 1.0, "Подсчёт файлов");
    let mut plan: Vec<(BackupRequest, Vec<(PathBuf, PathBuf)>)> = Vec::new();
    for item in items {
        if !safe_component(&item.name) {
            return Err(format!("invalid folder name: {}", item.name));
        }
        let base = root_dir(&item.root)?.join(&item.name);
        if !base.is_dir() {
            return Err(format!("folder not found: {}", base.display()));
        }
        let mut files = Vec::new();
        collect_files(&base, Path::new(""), include_cache, &mut files);
        plan.push((item, files));
    }
    let total: usize = plan.iter().map(|(_, f)| f.len()).sum();
    if total == 0 {
        return Err("selected folders contain no files".into());
    }

    let file = fs::File::create(&dest).map_err(|e| format!("cannot create {}: {e}", dest.display()))?;
    let mut writer = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .large_file(true);

    let mut manifest = Manifest {
        version: 1,
        created: SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0),
        computer: std::env::var("COMPUTERNAME").unwrap_or_default(),
        entries: Vec::new(),
    };

    let mut done = 0usize;
    let mut skipped = 0u32;
    let mut buffer = vec![0u8; 256 * 1024];
    for (item, files) in &plan {
        let mut bytes = 0u64;
        let mut count = 0u32;
        for (path, rel) in files {
            done += 1;
            if done % 25 == 0 {
                emit(app, 2.0 + 96.0 * done as f64 / total as f64, &item.name);
            }
            // Locked or unreadable files are skipped, never fatal.
            let Ok(mut source) = fs::File::open(path) else {
                skipped += 1;
                continue;
            };
            let entry_name = zip_name(&item.root, &item.name, rel);
            if writer.start_file(entry_name, options).is_err() {
                skipped += 1;
                continue;
            }
            loop {
                let read = match source.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => n,
                    Err(_) => {
                        skipped += 1;
                        break;
                    }
                };
                writer.write_all(&buffer[..read]).map_err(|e| e.to_string())?;
                bytes += read as u64;
            }
            count += 1;
        }
        manifest.entries.push(BackupEntry {
            root: item.root.clone(),
            name: item.name.clone(),
            files: count,
            bytes,
        });
    }

    let json = serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?;
    writer.start_file("manifest.json", options).map_err(|e| e.to_string())?;
    writer.write_all(&json).map_err(|e| e.to_string())?;
    writer.finish().map_err(|e| e.to_string())?;

    emit(app, 100.0, "Готово");
    crate::diag::log("INFO", &format!("backup created: {} ({} files, {} skipped)", dest.display(), total, skipped));
    Ok(if skipped > 0 {
        format!("Архив создан: {}. Пропущено занятых файлов: {skipped}.", dest.display())
    } else {
        format!("Архив создан: {}", dest.display())
    })
}

pub fn inspect_backup(path: &str) -> Result<Manifest, String> {
    let file = fs::File::open(path.trim()).map_err(|e| format!("cannot open archive: {e}"))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|_| "this is not a Setup Hub backup archive".to_string())?;
    let mut entry = archive
        .by_name("manifest.json")
        .map_err(|_| "this is not a Setup Hub backup archive (no manifest)".to_string())?;
    let mut text = String::new();
    entry.read_to_string(&mut text).map_err(|e| e.to_string())?;
    let manifest: Manifest = serde_json::from_str(&text).map_err(|_| "backup manifest is damaged".to_string())?;
    if manifest.version != 1 {
        return Err("unsupported backup version".into());
    }
    Ok(manifest)
}

pub fn restore_archive(
    app: &AppHandle,
    path: &str,
    selected: Vec<BackupRequest>,
) -> Result<String, String> {
    let manifest = inspect_backup(path)?;
    let wanted: HashSet<(String, String)> = selected
        .into_iter()
        .filter(|s| manifest.entries.iter().any(|e| e.root == s.root && e.name == s.name))
        .map(|s| (s.root, s.name))
        .collect();
    if wanted.is_empty() {
        return Err("nothing selected to restore".into());
    }

    let file = fs::File::open(path.trim()).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);

    let mut moved_aside: HashSet<(String, String)> = HashSet::new();
    let mut restored = 0u32;
    let mut failed = 0u32;
    let total = archive.len().max(1);

    for index in 0..archive.len() {
        if index % 25 == 0 {
            emit(app, 2.0 + 96.0 * index as f64 / total as f64, "Восстановление");
        }
        let mut entry = match archive.by_index(index) {
            Ok(e) => e,
            Err(_) => {
                failed += 1;
                continue;
            }
        };
        if entry.is_dir() {
            continue;
        }
        // `enclosed_name` rejects absolute paths and `..` (zip-slip protection).
        let Some(rel) = entry.enclosed_name() else { continue };
        let parts: Vec<String> = rel
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .collect();
        if parts.len() < 3 || !parts.iter().all(|p| safe_component(p)) {
            continue;
        }
        let key = (parts[0].clone(), parts[1].clone());
        if !wanted.contains(&key) {
            continue;
        }

        let base = root_dir(&key.0)?.join(&key.1);
        // Keep the user's current config: move it aside once, never overwrite blindly.
        if !moved_aside.contains(&key) {
            if base.exists() {
                let backup = base.with_file_name(format!("{}.setuphub-bak-{stamp}", key.1));
                fs::rename(&base, &backup).map_err(|e| {
                    format!("cannot replace {} (is the app still running?): {e}", base.display())
                })?;
            }
            moved_aside.insert(key.clone());
        }

        let mut target = base;
        for part in &parts[2..] {
            target.push(part);
        }
        if let Some(parent) = target.parent() {
            if fs::create_dir_all(parent).is_err() {
                failed += 1;
                continue;
            }
        }
        match fs::File::create(&target) {
            Ok(mut out) => {
                if std::io::copy(&mut entry, &mut out).is_ok() {
                    restored += 1;
                } else {
                    failed += 1;
                }
            }
            Err(_) => failed += 1,
        }
    }

    emit(app, 100.0, "Готово");
    crate::diag::log("INFO", &format!("backup restored: {restored} files, {failed} failed"));
    if restored == 0 {
        return Err("no files were restored".into());
    }
    Ok(if failed > 0 {
        format!("Восстановлено файлов: {restored}. Не удалось: {failed}. Прежние настройки сохранены в папках *.setuphub-bak-*.")
    } else {
        format!("Восстановлено файлов: {restored}. Прежние настройки сохранены в папках *.setuphub-bak-*.")
    })
}

#[cfg(test)]
mod archive_tests {
    use super::*;

    #[test]
    fn rejects_unsafe_components() {
        for bad in ["", ".", "..", "a/b", "a\\b", "C:", "name.", "name "] {
            assert!(!safe_component(bad), "{bad}");
        }
        assert!(safe_component("Code"));
        assert!(safe_component("My App"));
    }

    #[test]
    fn cache_detection_is_case_insensitive() {
        assert!(is_cache("GPUCache"));
        assert!(!is_cache("User"));
    }
}
