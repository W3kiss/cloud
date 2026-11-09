mod backup;
mod catalog;
mod diag;
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
async fn run_diagnostics() -> Vec<diag::Check> {
    tauri::async_runtime::spawn_blocking(diag::run_checks)
        .await
        .unwrap_or_default()
}

#[tauri::command]
async fn fix_diagnostic(id: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || diag::fix(&id))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn export_diagnostics() -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(diag::export_report)
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
fn log_frontend_error(message: String) {
    diag::log("FRONTEND", &message.chars().take(2000).collect::<String>());
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

/// Re-tints the native window (Mica) and caption bar to match the UI theme.
#[tauri::command]
fn set_window_tint(window: tauri::WebviewWindow, r: u8, g: u8, b: u8, a: u8, dark: bool) {
    #[cfg(target_os = "windows")]
    {
        use tauri::window::{Color, Effect, EffectsBuilder};
        let _ = window.set_theme(Some(if dark {
            tauri::Theme::Dark
        } else {
            tauri::Theme::Light
        }));
        let _ = window.set_effects(
            EffectsBuilder::new()
                .effect(Effect::Mica)
                .color(Color(r, g, b, a))
                .build(),
        );
    }
    #[cfg(not(target_os = "windows"))]
    let _ = (window, r, g, b, a, dark);
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
    diag::install_panic_hook();
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
            set_window_tint,
            run_diagnostics,
            fix_diagnostic,
            export_diagnostics,
            log_frontend_error,
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
