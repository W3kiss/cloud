mod backup;
mod catalog;
mod installer;
mod protected;
mod stripped;
mod tweaks;
mod usb;

use catalog::Catalog;
use tweaks::TweakStatus;

#[tauri::command]
fn bundled_catalog() -> Result<Catalog, String> {
    catalog::bundled()
}

#[tauri::command]
fn exit_app(app: tauri::AppHandle) {
    app.exit(0);
}

#[tauri::command]
async fn install_winget(winget_id: String) -> Result<String, String> {
    installer::install_winget(winget_id).await
}

#[tauri::command]
async fn install_entry(app: tauri::AppHandle, app_id: String) -> Result<String, String> {
    installer::install_entry(app, app_id).await
}

#[tauri::command]
fn tweak_states() -> Vec<TweakStatus> {
    tweaks::list_tweaks()
}

#[tauri::command]
async fn apply_tweak(id: String, enabled: bool) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || tweaks::apply(&id, enabled))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn stripped_states() -> Result<Vec<TweakStatus>, String> {
    tauri::async_runtime::spawn_blocking(stripped::list)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn apply_stripped(id: String, enabled: bool) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || stripped::apply(&id, enabled))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn create_restore_point() -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(stripped::create_restore_point)
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn list_usb_disks() -> Result<Vec<usb::UsbDisk>, String> {
    tauri::async_runtime::spawn_blocking(usb::list_usb_disks)
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn create_usb_installer(
    app: tauri::AppHandle,
    iso_path: String,
    disk_number: u32,
    variant: String,
    scheme: String,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        usb::create_installer(&app, &iso_path, disk_number, &variant, &scheme)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn restart_explorer() -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(tweaks::restart_explorer)
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn admin_status() -> bool {
    tauri::async_runtime::spawn_blocking(tweaks::is_admin)
        .await
        .unwrap_or(false)
}

#[tauri::command]
async fn restart_elevated() -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(tweaks::restart_elevated)
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
fn custom_hook_path() -> Result<String, String> {
    tweaks::hook_path().map(|path| path.display().to_string())
}

#[tauri::command]
async fn restore_backup(
    source: String,
    scope: String,
    folder_name: String,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || backup::restore_folder(source, scope, folder_name))
        .await
        .map_err(|error| error.to_string())?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            #[cfg(target_os = "windows")]
            {
                use tauri::Manager;
                use tauri::window::{Color, Effect, EffectsBuilder};
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_effects(
                        EffectsBuilder::new()
                            .effect(Effect::Mica)
                            .color(Color(16, 17, 20, 230))
                            .build(),
                    );
                }
            }
            #[cfg(not(target_os = "windows"))]
            let _ = app;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            bundled_catalog,
            exit_app,
            install_winget,
            install_entry,
            tweak_states,
            apply_tweak,
            stripped_states,
            apply_stripped,
            create_restore_point,
            list_usb_disks,
            create_usb_installer,
            restart_explorer,
            admin_status,
            restart_elevated,
            custom_hook_path,
            restore_backup
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
