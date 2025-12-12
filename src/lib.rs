use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

mod commands;
mod desktop;
mod error;

pub use desktop::{Frame, WebviewWindowExt};
pub use error::{Error, Result};

#[cfg(windows)]
use std::sync::{
    atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
    OnceLock,
};

#[cfg(windows)]
use tauri::Emitter;

#[cfg(windows)]
static TITLEBAR_HEIGHT: AtomicU32 = AtomicU32::new(32);
#[cfg(windows)]
static BUTTON_WIDTH: AtomicU32 = AtomicU32::new(46);
#[cfg(windows)]
static AUTO_TITLEBAR: AtomicBool = AtomicBool::new(false);
#[cfg(windows)]
static SNAP_OVERLAY_DELAY_MS: AtomicU64 = AtomicU64::new(10);
#[cfg(windows)]
static CLOSE_HOVER_BG: OnceLock<String> = OnceLock::new();
#[cfg(windows)]
static BUTTON_HOVER_BG: OnceLock<String> = OnceLock::new();

pub struct FramePluginBuilder {
    #[cfg(windows)]
    titlebar_height: u32,
    #[cfg(windows)]
    button_width: u32,
    #[cfg(windows)]
    auto_titlebar: bool,
    #[cfg(windows)]
    snap_overlay_delay_ms: u64,
    #[cfg(windows)]
    close_hover_bg: String,
    #[cfg(windows)]
    button_hover_bg: String,
}

impl Default for FramePluginBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl FramePluginBuilder {
    pub fn new() -> Self {
        Self {
            #[cfg(windows)]
            titlebar_height: 32,
            #[cfg(windows)]
            button_width: 46,
            #[cfg(windows)]
            auto_titlebar: false,
            #[cfg(windows)]
            snap_overlay_delay_ms: 10,
            #[cfg(windows)]
            close_hover_bg: "rgba(196,43,28,1)".into(),
            #[cfg(windows)]
            button_hover_bg: "rgba(0,0,0,0.2)".into(),
        }
    }

    #[cfg(windows)]
    pub fn titlebar_height(mut self, height: u32) -> Self {
        self.titlebar_height = height;
        self
    }

    #[cfg(not(windows))]
    pub fn titlebar_height(self, _: u32) -> Self {
        self
    }

    #[cfg(windows)]
    pub fn button_width(mut self, width: u32) -> Self {
        self.button_width = width;
        self
    }

    #[cfg(not(windows))]
    pub fn button_width(self, _: u32) -> Self {
        self
    }

    #[cfg(windows)]
    pub fn auto_titlebar(mut self, auto: bool) -> Self {
        self.auto_titlebar = auto;
        self
    }

    #[cfg(not(windows))]
    pub fn auto_titlebar(self, _: bool) -> Self {
        self
    }

    #[cfg(windows)]
    pub fn snap_overlay_delay_ms(mut self, delay: u64) -> Self {
        self.snap_overlay_delay_ms = delay;
        self
    }

    #[cfg(not(windows))]
    pub fn snap_overlay_delay_ms(self, _: u64) -> Self {
        self
    }

    #[cfg(windows)]
    pub fn close_hover_bg(mut self, color: impl Into<String>) -> Self {
        self.close_hover_bg = color.into();
        self
    }

    #[cfg(not(windows))]
    pub fn close_hover_bg(self, _: impl Into<String>) -> Self {
        self
    }

    #[cfg(windows)]
    pub fn button_hover_bg(mut self, color: impl Into<String>) -> Self {
        self.button_hover_bg = color.into();
        self
    }

    #[cfg(not(windows))]
    pub fn button_hover_bg(self, _: impl Into<String>) -> Self {
        self
    }

    #[cfg(windows)]
    pub fn build<R: Runtime>(self) -> TauriPlugin<R> {
        TITLEBAR_HEIGHT.store(self.titlebar_height, Ordering::SeqCst);
        BUTTON_WIDTH.store(self.button_width, Ordering::SeqCst);
        AUTO_TITLEBAR.store(self.auto_titlebar, Ordering::SeqCst);
        SNAP_OVERLAY_DELAY_MS.store(self.snap_overlay_delay_ms, Ordering::SeqCst);
        let _ = CLOSE_HOVER_BG.set(self.close_hover_bg);
        let _ = BUTTON_HOVER_BG.set(self.button_hover_bg);

        Builder::new("frame")
            .invoke_handler(tauri::generate_handler![commands::show_snap_overlay])
            .setup(|app, _| {
                app.manage(Frame::new(app.clone()));
                Ok(())
            })
            .on_page_load(|webview, _| {
                let _ = webview.emit("frame-page-load", ());
                if !AUTO_TITLEBAR.load(Ordering::SeqCst) {
                    return;
                }
                let height = TITLEBAR_HEIGHT.load(Ordering::SeqCst);
                let webview = webview.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = webview.eval(build_scripts(height, None));
                });
            })
            .build()
    }

    #[cfg(not(windows))]
    pub fn build<R: Runtime>(self) -> TauriPlugin<R> {
        Builder::new("frame")
            .invoke_handler(tauri::generate_handler![commands::show_snap_overlay])
            .setup(|app, _| {
                app.manage(Frame::new(app.clone()));
                Ok(())
            })
            .build()
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    FramePluginBuilder::new().build()
}

#[cfg(windows)]
pub(crate) fn get_snap_overlay_delay_ms() -> u64 {
    SNAP_OVERLAY_DELAY_MS.load(Ordering::SeqCst)
}

#[cfg(windows)]
pub(crate) fn get_titlebar_height() -> u32 {
    TITLEBAR_HEIGHT.load(Ordering::SeqCst)
}

#[cfg(windows)]
pub(crate) fn get_auto_titlebar() -> bool {
    AUTO_TITLEBAR.load(Ordering::SeqCst)
}

#[cfg(windows)]
pub(crate) fn build_scripts(height: u32, controls: Option<Vec<&str>>) -> String {
    let height_px = format!("\"{}px\"", height);
    let width_px = format!("\"{}px\"", BUTTON_WIDTH.load(Ordering::SeqCst));
    let close_hover = CLOSE_HOVER_BG
        .get()
        .map_or("rgba(196,43,28,1)", |s| s.as_str());
    let button_hover = BUTTON_HOVER_BG
        .get()
        .map_or("rgba(0,0,0,0.2)", |s| s.as_str());

    let script_tb = include_str!("js/titlebar.js").replace("\"32px\"", &height_px);
    let mut script_controls = include_str!("js/controls.js")
        .replace("\"32px\"", &height_px)
        .replace("\"46px\"", &width_px)
        .replace("rgba(196,43,28,1)", close_hover)
        .replace("rgba(0,0,0,0.2)", button_hover);

    if let Some(ctrl) = controls {
        script_controls = script_controls.replacen(
            "[\"minimize\", \"maximize\", \"close\"]",
            &format!("{:?}", ctrl),
            1,
        );
    }

    format!("{}\n{}", script_tb, script_controls)
}

pub trait FrameExt<R: Runtime> {
    fn frame(&self) -> &Frame<R>;
}

impl<R: Runtime, T: Manager<R>> FrameExt<R> for T {
    fn frame(&self) -> &Frame<R> {
        self.state::<Frame<R>>().inner()
    }
}
