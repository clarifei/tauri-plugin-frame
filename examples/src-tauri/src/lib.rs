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
                // Titlebar height in pixels
                .titlebar_height(32)
                // Button width in pixels
                .button_width(46)
                // Automatically apply titlebar to all windows
                .auto_titlebar(true)
                // Delay before pressing Alt to hide snap overlay numbers (ms)
                .snap_overlay_delay_ms(10)
                // Close button hover background color
                .close_hover_bg("rgba(196,43,28,1)")
                // Other buttons hover background color
                .button_hover_bg("rgba(255,255,255,0.1)")
                .build(),
        )
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
