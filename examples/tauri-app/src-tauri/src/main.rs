// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_frame::FramePluginBuilder;

#[tauri::command]
async fn open_new_window(app: tauri::AppHandle) -> Result<(), String> {
    let _new_window = WebviewWindowBuilder::new(
        &app,
        "new-window",
        WebviewUrl::App("index.html".into())
    )
    .title("New Window")
    .inner_size(500.0, 350.0)
    .decorations(false)  // Set false from start to avoid flash
    .build()
    .map_err(|e| e.to_string())?;

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .plugin(
            FramePluginBuilder::new()
                .titlebar_height(32)
                .auto_titlebar(true)
                .build()
        )
        .invoke_handler(tauri::generate_handler![open_new_window])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
