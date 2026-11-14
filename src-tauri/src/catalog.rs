use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::protected;

const CATALOG_BLOB: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/catalog.masked"));

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareItem {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub category: String,
    pub kind: String,
    #[serde(default)]
    pub winget_id: Option<String>,
    #[serde(default)]
    pub download_url: Option<String>,
    #[serde(default)]
    pub installer_args: Vec<String>,
    #[serde(default)]
    pub custom_config: BTreeMap<String, String>,
    #[serde(default)]
    pub presets: Vec<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub placeholder: bool,
    #[serde(default)]
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog {
    pub version: u32,
    pub apps: Vec<SoftwareItem>,
}

pub fn bundled() -> Result<Catalog, String> {
    let raw = protected::decode_catalog(CATALOG_BLOB)?;
    parse_catalog(&raw)
}

pub fn find(id: &str) -> Result<SoftwareItem, String> {
    bundled()?
        .apps
        .into_iter()
        .find(|app| app.id == id)
        .ok_or_else(|| format!("application is not present in the catalog: {id}"))
}

fn parse_catalog(raw: &str) -> Result<Catalog, String> {
    let catalog: Catalog =
        serde_json::from_str(raw).map_err(|error| format!("catalog is invalid: {error}"))?;

    if catalog.version == 0 {
        return Err("catalog version must be greater than zero".to_string());
    }

    for app in &catalog.apps {
        validate_app(app)?;
    }

    Ok(catalog)
}

fn validate_app(app: &SoftwareItem) -> Result<(), String> {
    validate_app_id(&app.id)?;

    if !matches!(
        app.category.as_str(),
        "system" | "browsers" | "gaming" | "production"
    ) {
        return Err(format!("catalog category is not allowed: {}", app.category));
    }

    match app.kind.as_str() {
        "winget" => {
            let winget_id = app
                .winget_id
                .as_deref()
                .ok_or_else(|| format!("{} has no winget id", app.id))?;
            validate_winget_id(winget_id)?;
        }
        "download" | "custom" => {
            if let Some(url) = app.download_url.as_deref() {
                validate_https_url(url)?;
            } else if !app.placeholder {
                return Err(format!("{} has no download url", app.id));
            }
        }
        other => return Err(format!("catalog kind is not allowed: {other}")),
    }

    if let Some(digest) = app.sha256.as_deref() {
        if digest.len() != 64 || !digest.chars().all(|ch| ch.is_ascii_hexdigit()) {
            return Err(format!("{} has an invalid sha256 digest", app.id));
        }
    }

    if app.installer_args.len() > 16 {
        return Err(format!("{} has too many installer arguments", app.id));
    }
    for argument in &app.installer_args {
        if argument.len() > 256 || argument.chars().any(|ch| ch.is_control()) {
            return Err(format!("{} has an unsafe installer argument", app.id));
        }
    }

    if app.custom_config.len() > 24 {
        return Err(format!("{} has too many custom settings", app.id));
    }
    for (key, value) in &app.custom_config {
        if key.len() > 64 || value.len() > 512 {
            return Err(format!("{} has an oversized custom setting", app.id));
        }
    }

    for dependency in &app.depends_on {
        validate_winget_id(dependency)?;
    }

    Ok(())
}

pub fn validate_app_id(id: &str) -> Result<(), String> {
    if id.is_empty()
        || id.len() > 64
        || !id
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
    {
        return Err(format!("application id is not allowed: {id}"));
    }
    Ok(())
}

pub fn validate_winget_id(id: &str) -> Result<(), String> {
    let mut chars = id.chars();
    let Some(first) = chars.next() else {
        return Err("winget id is empty".to_string());
    };

    if !first.is_ascii_alphanumeric() || id.len() > 128 {
        return Err(format!("winget id is not allowed: {id}"));
    }

    if chars.any(|ch| !(ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-' | '+'))) {
        return Err(format!("winget id is not allowed: {id}"));
    }

    Ok(())
}

pub fn validate_https_url(url: &str) -> Result<(), String> {
    if url.len() > 2048 || !url.starts_with("https://") {
        return Err("download url must start with https://".to_string());
    }

    if url.contains('@') || url.contains('\\') || url.contains(char::is_whitespace) {
        return Err("download url is not allowed".to_string());
    }

    let host = url
        .strip_prefix("https://")
        .and_then(|rest| rest.split('/').next())
        .unwrap_or_default()
        .split(':')
        .next()
        .unwrap_or_default();

    if host.is_empty() || host == "localhost" || host.ends_with(".localhost") {
        return Err("download url host is not allowed".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_catalog_is_decodable() {
        let catalog = bundled().expect("the masked catalog should decode");
        assert!(catalog.version >= 2);
        assert!(!catalog.apps.is_empty());
        assert!(catalog.apps.iter().any(|app| app.id == "winrar"));
        assert!(catalog.apps.iter().any(|app| app.category == "production"));
    }

    #[test]
    fn winget_ids_may_contain_plus() {
        assert!(validate_winget_id("Notepad++.Notepad++").is_ok());
        assert!(validate_winget_id("Vendor.App; calc").is_err());
        assert!(validate_winget_id("+Vendor.App").is_err());
    }

    #[test]
    fn catalog_urls_are_https() {
        let catalog = bundled().expect("the masked catalog should decode");
        for app in catalog.apps {
            if let Some(url) = app.download_url.as_deref() {
                assert!(validate_https_url(url).is_ok(), "unsafe url in {}", app.id);
            }
        }
    }
}
