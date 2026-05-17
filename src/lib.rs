use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

mod desktop;
mod error;
#[cfg(windows)]
mod snap;

pub use desktop::{Frame, WebviewWindowExt};
pub use error::{Error, Result};

#[cfg(windows)]
use std::sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
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
static NATIVE_SNAP_OVERLAY: AtomicBool = AtomicBool::new(true);
#[cfg(windows)]
static CLOSE_HOVER_BG: OnceLock<String> = OnceLock::new();
#[cfg(windows)]
static BUTTON_HOVER_BG_LIGHT: OnceLock<String> = OnceLock::new();
#[cfg(windows)]
static BUTTON_HOVER_BG_DARK: OnceLock<String> = OnceLock::new();

pub struct FramePluginBuilder {
    #[cfg(windows)]
    titlebar_height: u32,
    #[cfg(windows)]
    button_width: u32,
    #[cfg(windows)]
    auto_titlebar: bool,
    #[cfg(windows)]
    snap_overlay: bool,
    #[cfg(windows)]
    close_hover_bg: String,
    #[cfg(windows)]
    button_hover_bg_light: String,
    #[cfg(windows)]
    button_hover_bg_dark: String,
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
            snap_overlay: true,
            #[cfg(windows)]
            close_hover_bg: "rgba(196,43,28,1)".into(),
            #[cfg(windows)]
            button_hover_bg_light: "rgba(0,0,0,0.1)".into(),
            #[cfg(windows)]
            button_hover_bg_dark: "rgba(255,255,255,0.1)".into(),
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
    pub fn snap_overlay(mut self, enabled: bool) -> Self {
        self.snap_overlay = enabled;
        self
    }

    #[cfg(not(windows))]
    pub fn snap_overlay(self, _: bool) -> Self {
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

    /// Set the hover background color for non-close buttons in light mode.
    #[cfg(windows)]
    pub fn button_hover_bg_light(mut self, color: impl Into<String>) -> Self {
        self.button_hover_bg_light = color.into();
        self
    }

    #[cfg(not(windows))]
    pub fn button_hover_bg_light(self, _: impl Into<String>) -> Self {
        self
    }

    /// Set the hover background color for non-close buttons in dark mode.
    #[cfg(windows)]
    pub fn button_hover_bg_dark(mut self, color: impl Into<String>) -> Self {
        self.button_hover_bg_dark = color.into();
        self
    }

    #[cfg(not(windows))]
    pub fn button_hover_bg_dark(self, _: impl Into<String>) -> Self {
        self
    }

    /// Set a single hover background color for non-close buttons (applies to both light and dark mode).
    #[cfg(windows)]
    pub fn button_hover_bg(mut self, color: impl Into<String>) -> Self {
        let c = color.into();
        self.button_hover_bg_light = c.clone();
        self.button_hover_bg_dark = c;
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
        NATIVE_SNAP_OVERLAY.store(self.snap_overlay, Ordering::SeqCst);
        let _ = CLOSE_HOVER_BG.set(self.close_hover_bg);
        let _ = BUTTON_HOVER_BG_LIGHT.set(self.button_hover_bg_light);
        let _ = BUTTON_HOVER_BG_DARK.set(self.button_hover_bg_dark);

        Builder::new("frame")
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
                let button_width = BUTTON_WIDTH.load(Ordering::SeqCst);
                let snap_overlay = NATIVE_SNAP_OVERLAY.load(Ordering::SeqCst);
                let webview = webview.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = webview.eval(build_scripts(height, None));
                    if snap_overlay {
                        // Default right_index=1 (close button is present by default)
                        let _ = crate::snap::install_window(&webview.window(), height, button_width, 1);
                    }
                });
            })
            .build()
    }

    #[cfg(not(windows))]
    pub fn build<R: Runtime>(self) -> TauriPlugin<R> {
        Builder::new("frame")
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
pub(crate) fn snap_overlay_enabled() -> bool {
    NATIVE_SNAP_OVERLAY.load(Ordering::SeqCst)
}


#[cfg(windows)]
pub(crate) fn get_titlebar_height() -> u32 {
    TITLEBAR_HEIGHT.load(Ordering::SeqCst)
}

#[cfg(windows)]
pub(crate) fn get_button_width() -> u32 {
    BUTTON_WIDTH.load(Ordering::SeqCst)
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
    let button_hover_light = BUTTON_HOVER_BG_LIGHT
        .get()
        .map_or("rgba(0,0,0,0.1)", |s| s.as_str());
    let button_hover_dark = BUTTON_HOVER_BG_DARK
        .get()
        .map_or("rgba(255,255,255,0.1)", |s| s.as_str());

    let script_tb = include_str!("js/titlebar.js").replace("\"32px\"", &height_px);
    let mut script_controls = include_str!("js/controls.js")
        .replace("\"32px\"", &height_px)
        .replace("\"46px\"", &width_px)
        .replace("\"__CLOSE_HOVER_BG__\"", &format!("\"{}\"", close_hover))
        .replace("\"__BUTTON_HOVER_BG_LIGHT__\"", &format!("\"{}\"", button_hover_light))
        .replace("\"__BUTTON_HOVER_BG_DARK__\"", &format!("\"{}\"", button_hover_dark));

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