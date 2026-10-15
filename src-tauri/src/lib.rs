//! 190x4 Discord Kit.
//! Режимы запуска одного exe:
//!   (без аргументов)  — окно установщика;
//!   --guard           — фоновый сторож без окна (автозапуск);
//!   --self-update     — тихое OTA-обновление самого Kit и выход.

mod autostart;
mod bd;
mod content;
mod discord;
mod guard;
mod installer;
mod net;
mod paths;
mod state;
mod util;

use serde::Serialize;
use tauri::{AppHandle, Manager};
use tauri_plugin_updater::UpdaterExt;

#[derive(Serialize)]
struct Status {
    app_version: String,
    discord: Vec<discord::Install>,
    bd_installed: bool,
    bd_version: Option<String>,
    content: Vec<content::ItemStatus>,
    guard: bool,
}

#[derive(Serialize)]
struct AppUpdate {
    version: String,
    notes: Option<String>,
}

async fn blocking<T: Send + 'static>(f: impl FnOnce() -> Result<T, String> + Send + 'static) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn get_status(app: AppHandle) -> Result<Status, String> {
    let app_version = app.package_info().version.to_string();
    blocking(move || {
        let state = state::State::load();
        let client = net::client();
        Ok(Status {
            app_version,
            discord: discord::installs(),
            bd_installed: bd::installed(),
            bd_version: state.bd_version.clone(),
            content: content::status(&client, &state),
            guard: autostart::is_enabled(),
        })
    })
    .await
}

#[tauri::command]
async fn install_all(app: AppHandle) -> Result<(), String> {
    blocking(move || installer::install_all(&app)).await
}

#[tauri::command]
async fn uninstall_all(app: AppHandle) -> Result<(), String> {
    blocking(move || installer::uninstall_all(&app)).await
}

#[tauri::command]
async fn restart_discord() -> Result<(), String> {
    blocking(|| {
        installer::restart_discord();
        Ok(())
    })
    .await
}

#[tauri::command]
async fn check_app_update(app: AppHandle) -> Result<Option<AppUpdate>, String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    let update = updater.check().await.map_err(|e| e.to_string())?;
    Ok(update.map(|u| AppUpdate {
        version: u.version,
        notes: u.body,
    }))
}

/// Скачивает и ставит обновление Kit; установщик NSIS перезапустит приложение.
#[tauri::command]
async fn install_app_update(app: AppHandle) -> Result<(), String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    if let Some(update) = updater.check().await.map_err(|e| e.to_string())? {
        update
            .download_and_install(|_, _| {}, || {})
            .await
            .map_err(|e| e.to_string())?;
        app.restart();
    }
    Ok(())
}

async fn silent_self_update(app: &AppHandle) {
    if let Ok(updater) = app.updater() {
        if let Ok(Some(update)) = updater.check().await {
            let _ = update.download_and_install(|_, _| {}, || {}).await;
        }
    }
}

pub fn run() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--guard") {
        guard::run();
        return;
    }
    let self_update = args.iter().any(|a| a == "--self-update");

    let mut builder = tauri::Builder::default();
    if !self_update {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }));
    }
    builder
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            get_status,
            install_all,
            uninstall_all,
            restart_discord,
            check_app_update,
            install_app_update
        ])
        .setup(move |app| {
            if self_update {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    silent_self_update(&handle).await;
                    handle.exit(0);
                });
            } else {
                tauri::WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::App("index.html".into()))
                    .title("190x4 Discord Kit")
                    .inner_size(900.0, 600.0)
                    .resizable(false)
                    .decorations(false)
                    .center()
                    .build()?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("190x4 Discord Kit failed to start");
}
