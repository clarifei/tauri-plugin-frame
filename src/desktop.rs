use tauri::{Runtime, WebviewWindow};

use crate::error::Result;

pub struct Frame<R: Runtime> {
    #[allow(dead_code)]
    app: tauri::AppHandle<R>,
}

impl<R: Runtime> Frame<R> {
    pub fn new(app: tauri::AppHandle<R>) -> Self {
        Self { app }
    }

    #[cfg(windows)]
    pub fn titlebar_height(&self) -> u32 {
        crate::get_titlebar_height()
    }

    #[cfg(not(windows))]
    pub fn titlebar_height(&self) -> u32 {
        32
    }

    #[cfg(windows)]
    pub fn auto_titlebar(&self) -> bool {
        crate::get_auto_titlebar()
    }

    #[cfg(not(windows))]
    pub fn auto_titlebar(&self) -> bool {
        false
    }
}

pub trait WebviewWindowExt {
    fn create_overlay_titlebar(&self) -> Result<&WebviewWindow>;
    fn create_overlay_titlebar_with_height(&self, height: u32) -> Result<&WebviewWindow>;
}

#[cfg(windows)]
impl WebviewWindowExt for WebviewWindow {
    fn create_overlay_titlebar(&self) -> Result<&WebviewWindow> {
        self.create_overlay_titlebar_with_height(crate::get_titlebar_height())
    }

    fn create_overlay_titlebar_with_height(&self, height: u32) -> Result<&WebviewWindow> {
        use tauri::Listener;

        self.set_decorations(false)?;

        let win = self.clone();
        self.listen("frame-page-load", move |event| {
            let controls: Vec<&str> = [
                win.is_minimizable().unwrap_or(false).then_some("minimize"),
                (win.is_maximizable().unwrap_or(false) && win.is_resizable().unwrap_or(false))
                    .then_some("maximize"),
                win.is_closable().unwrap_or(false).then_some("close"),
            ]
            .into_iter()
            .flatten()
            .collect();

            let _ = win.eval(crate::build_scripts(height, Some(controls)));

            let win2 = win.clone();
            win.on_window_event(move |e| {
                if matches!(e, tauri::WindowEvent::CloseRequested { .. }) {
                    win2.unlisten(event.id());
                }
            });
        });

        Ok(self)
    }
}

#[cfg(not(windows))]
impl WebviewWindowExt for WebviewWindow {
    fn create_overlay_titlebar(&self) -> Result<&WebviewWindow> {
        Ok(self)
    }

    fn create_overlay_titlebar_with_height(&self, _height: u32) -> Result<&WebviewWindow> {
        Ok(self)
    }
}
