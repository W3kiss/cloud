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
