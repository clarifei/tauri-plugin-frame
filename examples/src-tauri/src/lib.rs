use tauri_plugin_frame::FramePluginBuilder;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            FramePluginBuilder::new()
                .titlebar_height(32)
                .button_width(46)
                .auto_titlebar(true)
                .snap_overlay(true)
                .close_hover_bg("rgba(196,43,28,1)")
                .button_hover_bg_light("rgba(0,0,0,0.1)")
                .button_hover_bg_dark("rgba(255,255,255,0.1)")
                .build(),
        )
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}