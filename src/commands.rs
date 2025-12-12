use crate::error::Result;

#[cfg(windows)]
#[tauri::command]
pub async fn show_snap_overlay() -> Result<()> {
    let info = os_info::get();
    let version = info.version();

    let is_win11 = match version {
        os_info::Version::Semantic(major, _, patch) => {
            *major >= 11 || (*major == 10 && *patch >= 22000)
        }
        _ => false,
    };

    if !is_win11 {
        return Ok(());
    }

    use std::time::Duration;

    use enigo::{
        Direction::{Click, Press, Release},
        Enigo, Key, Keyboard, Settings,
    };

    use crate::get_snap_overlay_delay_ms;

    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| eyre::eyre!("{e}"))?;

    enigo
        .key(Key::Meta, Press)
        .map_err(|e| eyre::eyre!("{e}"))?;
    enigo
        .key(Key::Unicode('z'), Click)
        .map_err(|e| eyre::eyre!("{e}"))?;
    enigo
        .key(Key::Meta, Release)
        .map_err(|e| eyre::eyre!("{e}"))?;

    std::thread::sleep(Duration::from_millis(get_snap_overlay_delay_ms()));

    enigo.key(Key::Alt, Press).map_err(|e| eyre::eyre!("{e}"))?;
    std::thread::sleep(Duration::from_millis(50));
    enigo
        .key(Key::Alt, Release)
        .map_err(|e| eyre::eyre!("{e}"))?;

    Ok(())
}

#[cfg(not(windows))]
#[tauri::command]
pub async fn show_snap_overlay() -> Result<()> {
    Ok(())
}
