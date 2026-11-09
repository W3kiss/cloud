//! Installed-software management through winget: list, update, uninstall, preset files.

use std::path::Path;

use serde::Serialize;

use crate::installer::{winget_run, with_winget_lock};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Package {
    pub id: String,
    pub name: String,
    pub version: String,
    pub available: String,
    pub source: String,
}

/// Components other apps depend on; removing them breaks software silently.
const PROTECTED_PREFIXES: &[&str] = &[
    "microsoft.vcredist",
    "microsoft.edgewebview2runtime",
    "microsoft.dotnet",
    "microsoft.windowsappruntime",
    "microsoft.ui.xaml",
    "microsoft.vclibs",
    "microsoft.appinstaller",
    "microsoft.desktopappinstaller",
    "microsoft.edge",
];

pub fn is_protected(id: &str) -> bool {
    let lowered = id.to_lowercase();
    PROTECTED_PREFIXES.iter().any(|p| lowered.starts_with(p))
}

fn valid_package_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 200
        && !id.starts_with('-')
        && !id.chars().any(|c| c.is_control() || matches!(c, '"' | '\'' | '`' | '|' | '&' | '<' | '>'))
}

/// Parses the fixed-width table printed by `winget list` / `winget upgrade`.
/// Column offsets come from the header line, which makes it locale independent.
pub fn parse_table(output: &str) -> Vec<Package> {
    let lines: Vec<Vec<char>> = output
        .lines()
        // winget redraws a spinner with bare `\r`; only the text after the last one is real.
        .map(|l| l.trim_end_matches('\r').rsplit('\r').next().unwrap_or("").chars().collect())
        .collect();

    let Some(dash) = lines
        .iter()
        .position(|l| l.len() >= 10 && l.iter().all(|c| *c == '-'))
    else {
        return Vec::new();
    };
    if dash == 0 {
        return Vec::new();
    }
    let header = &lines[dash - 1];

    // A column starts where a non-space follows two spaces (or the line start).
    let mut starts = Vec::new();
    for (i, c) in header.iter().enumerate() {
        if *c != ' ' && (i == 0 || (header[i - 1] == ' ' && (i < 2 || header[i - 2] == ' '))) {
            starts.push(i);
        }
    }
    if starts.len() < 3 {
        return Vec::new();
    }

    let cell = |line: &Vec<char>, index: usize| -> String {
        let from = starts[index].min(line.len());
        let to = starts.get(index + 1).copied().unwrap_or(line.len()).min(line.len());
        line[from..to.max(from)].iter().collect::<String>().trim().to_string()
    };

    let mut packages = Vec::new();
    for line in &lines[dash + 1..] {
        if line.iter().all(|c| c.is_whitespace()) {
            continue;
        }
        let id = cell(line, 1);
        if id.is_empty() || id.contains(' ') {
            continue; // footer such as "3 upgrades available."
        }
        let (available, source) = match starts.len() {
            n if n >= 5 => (cell(line, 3), cell(line, 4)),
            4 => (String::new(), cell(line, 3)),
            _ => (String::new(), String::new()),
        };
        // With four columns the 4th may be "Available" when no source column exists.
        packages.push(Package {
            name: cell(line, 0),
            id,
            version: cell(line, 2),
            available,
            source,
        });
    }
    packages
}

const BASE_FLAGS: [&str; 2] = ["--accept-source-agreements", "--disable-interactivity"];

pub fn list_installed() -> Result<Vec<Package>, String> {
    let mut args = vec!["list"];
    args.extend(BASE_FLAGS);
    let out = with_winget_lock(|| winget_run(&args, 180)).map_err(|e| crate::diag::annotate("winget list", e))?;
    let mut packages = parse_table(&out);
    // Hide the Windows Store/system noise by default: keep named, non-protected entries.
    packages.retain(|p| !p.name.is_empty());
    Ok(packages)
}

pub fn list_updates() -> Result<Vec<Package>, String> {
    let mut args = vec!["upgrade", "--include-unknown"];
    args.extend(BASE_FLAGS);
    let out = match with_winget_lock(|| winget_run(&args, 180)) {
        Ok(out) => out,
        // winget exits non-zero when nothing can be upgraded.
        Err(error) if crate::diag::error_code(&error).is_some_and(|(c, _)| c == "E-WINGET-NOUPDATE") => String::new(),
        Err(error) => return Err(crate::diag::annotate("winget upgrade", error)),
    };
    let mut packages = parse_table(&out);
    packages.retain(|p| !p.available.is_empty());
    Ok(packages)
}

pub fn upgrade_package(id: &str) -> Result<String, String> {
    if !valid_package_id(id) {
        return Err("invalid package id".into());
    }
    crate::diag::log("INFO", &format!("winget upgrade {id}"));
    let mut args = vec![
        "upgrade", "--id", id, "--exact", "--silent", "--accept-package-agreements",
    ];
    args.extend(BASE_FLAGS);

    let mut last = String::new();
    for attempt in 0..3 {
        match with_winget_lock(|| winget_run(&args, 1800)) {
            Ok(_) => return Ok(format!("{id} обновлён")),
            Err(error) => {
                match crate::diag::error_code(&error).map(|(c, _)| c) {
                    Some("E-WINGET-NOUPDATE") => return Ok(format!("{id}: уже последняя версия")),
                    Some("E-LOCK-004") if attempt < 2 => {
                        std::thread::sleep(std::time::Duration::from_secs(20));
                        last = error;
                        continue;
                    }
                    _ => return Err(crate::diag::annotate(&format!("winget upgrade {id}"), error)),
                }
            }
        }
    }
    Err(crate::diag::annotate(&format!("winget upgrade {id}"), last))
}

pub fn uninstall_package(id: &str) -> Result<String, String> {
    if !valid_package_id(id) {
        return Err("invalid package id".into());
    }
    if is_protected(id) {
        return Err(format!("{id} — системный компонент, его удаление заблокировано"));
    }
    crate::diag::log("INFO", &format!("winget uninstall {id}"));
    let mut args = vec!["uninstall", "--id", id, "--exact", "--silent"];
    args.extend(BASE_FLAGS);
    with_winget_lock(|| winget_run(&args, 1200))
        .map(|_| format!("{id} удалён"))
        .map_err(|e| crate::diag::annotate(&format!("winget uninstall {id}"), e))
}

// ------------------------------------------------------------ preset files

const MAX_PRESET_BYTES: u64 = 2 * 1024 * 1024;

fn is_json(path: &Path) -> bool {
    path.extension().is_some_and(|e| e.eq_ignore_ascii_case("json"))
}

pub fn write_preset(path: &str, content: &str) -> Result<(), String> {
    let path = Path::new(path.trim());
    if !is_json(path) {
        return Err("preset file must end with .json".into());
    }
    if content.len() as u64 > MAX_PRESET_BYTES {
        return Err("preset is too large".into());
    }
    serde_json::from_str::<serde_json::Value>(content).map_err(|e| format!("invalid JSON: {e}"))?;
    std::fs::write(path, content).map_err(|e| format!("cannot write preset: {e}"))
}

pub fn read_preset(path: &str) -> Result<String, String> {
    let path = Path::new(path.trim());
    if !is_json(path) {
        return Err("preset file must end with .json".into());
    }
    let meta = std::fs::metadata(path).map_err(|e| format!("cannot read preset: {e}"))?;
    if meta.len() > MAX_PRESET_BYTES {
        return Err("preset is too large".into());
    }
    std::fs::read_to_string(path).map_err(|e| format!("cannot read preset: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(a: &str, b: &str, c: &str, d: &str, e: &str) -> String {
        format!("{a:<18}{b:<22}{c:<9}{d:<11}{e}\n")
    }

    fn sample() -> String {
        let mut out = String::from("\r   - \\ |\r");
        out.push_str(&row("Name", "Id", "Version", "Available", "Source"));
        out.push_str(&"-".repeat(70));
        out.push('\n');
        out.push_str(&row("Notepad++", "Notepad++.Notepad++", "8.6.1", "8.7", "winget"));
        out.push_str(&row("7-Zip", "7zip.7zip", "23.01", "", "winget"));
        out.push_str(&row("Мой софт", "Vendor.Moy", "1.0", "", "winget"));
        out.push_str("2 upgrades available.\n");
        out
    }

    #[test]
    fn parses_table_with_available_column() {
        let items = parse_table(&sample());
        assert_eq!(items.len(), 3);
        assert!(items.iter().any(|p| p.id == "7zip.7zip" && p.version == "23.01"));
        assert!(items.iter().any(|p| p.id == "Notepad++.Notepad++" && p.available == "8.7"));
        assert!(items.iter().any(|p| p.name == "Мой софт" && p.source == "winget"));
        assert!(items.iter().all(|p| !p.id.contains(' ')));
    }

    #[test]
    fn protected_components_are_blocked() {
        assert!(is_protected("Microsoft.VCRedist.2015+.x64"));
        assert!(!is_protected("Mozilla.Firefox"));
    }

    #[test]
    fn rejects_option_like_ids() {
        assert!(!valid_package_id("--force"));
        assert!(valid_package_id("Notepad++.Notepad++"));
    }
}
