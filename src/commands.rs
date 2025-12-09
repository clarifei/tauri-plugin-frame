use enigo::{Enigo, Key, KeyboardControllable};

#[tauri::command]
pub async fn show_snap_overlay() {
    let mut enigo = Enigo::new();
    enigo.key_down(Key::Meta);
    enigo.key_click(Key::Layout('z'));
    enigo.key_up(Key::Meta);

    std::thread::sleep(std::time::Duration::from_millis(10));

    enigo.key_click(Key::Alt);
}
