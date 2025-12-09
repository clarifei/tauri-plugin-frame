use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use tauri::plugin::{Builder, TauriPlugin};
use tauri::{async_runtime, Emitter, Error, Listener, Runtime, WebviewWindow};

mod commands;

static TITLEBAR_HEIGHT: AtomicU32 = AtomicU32::new(32);
static AUTO_TITLEBAR: AtomicBool = AtomicBool::new(false);

/// Builder for the frame plugin
pub struct FramePluginBuilder {
    titlebar_height: u32,
    auto_titlebar: bool,
}

impl FramePluginBuilder {
    pub fn new() -> Self {
        Self {
            titlebar_height: 32,
            auto_titlebar: false,
        }
    }

    /// Set the titlebar height in pixels (default: 32)
    pub fn titlebar_height(mut self, height: u32) -> Self {
        self.titlebar_height = height;
        self
    }

    /// Automatically apply titlebar to all windows (default: false)
    pub fn auto_titlebar(mut self, auto: bool) -> Self {
        self.auto_titlebar = auto;
        self
    }

    /// Build the plugin
    pub fn build<R: Runtime>(self) -> TauriPlugin<R> {
        TITLEBAR_HEIGHT.store(self.titlebar_height, Ordering::SeqCst);
        AUTO_TITLEBAR.store(self.auto_titlebar, Ordering::SeqCst);

        Builder::new("frame")
            .invoke_handler(tauri::generate_handler![commands::show_snap_overlay])
            .on_page_load(|webview, _| {
                let _ = webview.emit("frame-page-load", ());

                if AUTO_TITLEBAR.load(Ordering::SeqCst) {
                    let height = TITLEBAR_HEIGHT.load(Ordering::SeqCst);
                    let webview = webview.clone();

                    async_runtime::spawn(async move {
                        let _ = webview.eval(build_scripts(height, None));
                    });
                }
            })
            .build()
    }
}

impl Default for FramePluginBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize the frame plugin with default settings
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    FramePluginBuilder::new().build()
}

fn build_scripts(height: u32, controls: Option<Vec<&str>>) -> String {
    let height_px = format!("\"{}px\"", height);

    let script_tb = include_str!("js/titlebar.js").replace("\"32px\"", &height_px);
    let mut script_controls = include_str!("js/controls.js").replace("\"32px\"", &height_px);

    if let Some(ctrl) = controls {
        script_controls = script_controls.replacen(
            "[\"minimize\", \"maximize\", \"close\"]",
            &format!("{:?}", ctrl),
            1,
        );
    }

    format!("{}\n{}", script_tb, script_controls)
}

/// Extensions to [`tauri::WebviewWindow`] to access the frame APIs.
pub trait WebviewWindowExt {
    fn create_overlay_titlebar(&self) -> Result<&WebviewWindow, Error>;
    fn create_overlay_titlebar_with_height(&self, height: u32) -> Result<&WebviewWindow, Error>;
}

impl WebviewWindowExt for WebviewWindow {
    fn create_overlay_titlebar(&self) -> Result<&WebviewWindow, Error> {
        self.create_overlay_titlebar_with_height(TITLEBAR_HEIGHT.load(Ordering::SeqCst))
    }

    fn create_overlay_titlebar_with_height(&self, height: u32) -> Result<&WebviewWindow, Error> {
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

            let _ = win.eval(build_scripts(height, Some(controls)));

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
