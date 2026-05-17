use std::{collections::HashMap, sync::Mutex};

use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use tauri::{Emitter, Runtime, WebviewWindow, Window};
use windows_sys::Win32::{
    Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM},
    Graphics::Gdi::{GetStockObject, ScreenToClient, HBRUSH, NULL_BRUSH},
    System::LibraryLoader::GetModuleHandleW,
    UI::{
        Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
        HiDpi::GetDpiForWindow,
        Input::KeyboardAndMouse::{TrackMouseEvent, TME_LEAVE, TME_NONCLIENT, TRACKMOUSEEVENT},
        WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, GetClientRect, RegisterClassExW,
            SetWindowPos, CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, HTMAXBUTTON,
            HWND_TOP, SWP_ASYNCWINDOWPOS, SWP_SHOWWINDOW, WM_CLOSE, WM_CREATE,
            WM_DPICHANGED, WM_NCHITTEST, WM_NCLBUTTONDOWN, WM_NCLBUTTONUP,
            WM_NCMOUSELEAVE, WM_NCMOUSEMOVE, WM_SIZE, WNDCLASSEXW, WS_CHILD,
            WS_CLIPSIBLINGS, WS_OVERLAPPED, WS_VISIBLE,
        },
    },
};

use crate::error::Result;

const SNAP_CLASS: &[u16] = &[
    b'T' as u16,
    b'a' as u16,
    b'u' as u16,
    b'r' as u16,
    b'i' as u16,
    b'F' as u16,
    b'r' as u16,
    b'a' as u16,
    b'm' as u16,
    b'e' as u16,
    b'S' as u16,
    b'n' as u16,
    b'a' as u16,
    b'p' as u16,
    b'O' as u16,
    b'v' as u16,
    b'e' as u16,
    b'r' as u16,
    b'l' as u16,
    b'a' as u16,
    b'y' as u16,
    0,
];
const SUBCLASS_ID: usize = 0x7466_736e_6170;
const EVENT_MOUSEENTER: &str = "tauri-frame://snap/mouseenter";
const EVENT_MOUSELEAVE: &str = "tauri-frame://snap/mouseleave";
const EVENT_MOUSEDOWN: &str = "tauri-frame://snap/mousedown";
const EVENT_MOUSEUP: &str = "tauri-frame://snap/mouseup";
const EVENT_CLICK: &str = "tauri-frame://snap/click";

static SNAP_WINDOWS: std::sync::LazyLock<Mutex<HashMap<isize, SnapState>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

struct SnapState {
    overlay: HWND,
    titlebar_height: u32,
    button_width: u32,
    right_index: u32, // number of buttons to the right of the maximize button
    hovering: bool,
    pressing: bool,
    last_x: i32,
    last_y: i32,
    emit: Box<dyn Fn(&'static str) + Send>,
    emit_move: Box<dyn Fn(i32, i32) + Send>,
}

unsafe impl Send for SnapState {}

pub fn install<R: Runtime>(window: &WebviewWindow<R>, titlebar_height: u32, button_width: u32, right_index: u32) -> Result<()> {
    let hwnd = window_hwnd(window)? as isize;
    let webview = window.clone();

    window.run_on_main_thread(move || unsafe {
        let target = webview.clone();
        install_hwnd(
            hwnd,
            titlebar_height,
            button_width,
            right_index,
            Box::new(move |event| {
                let _ = target.emit(event, ());
            }),
            Box::new(move |x, y| {
                let _ = webview.emit("tauri-frame://snap/mousemove", (x, y));
            }),
        );
    })?;

    Ok(())
}

pub fn install_window<R: Runtime>(window: &Window<R>, titlebar_height: u32, button_width: u32, right_index: u32) -> Result<()> {
    let hwnd = window_hwnd(window)? as isize;
    let emit_window = window.clone();
    let main_window = window.clone();

    main_window.run_on_main_thread(move || unsafe {
        let target = emit_window.clone();
        let move_target = emit_window.clone();
        install_hwnd(
            hwnd,
            titlebar_height,
            button_width,
            right_index,
            Box::new(move |event| {
                let _ = target.emit(event, ());
            }),
            Box::new(move |x, y| {
                let _ = move_target.emit("tauri-frame://snap/mousemove", (x, y));
            }),
        );
    })?;

    Ok(())
}


unsafe fn install_hwnd(
    hwnd: isize,
    titlebar_height: u32,
    button_width: u32,
    right_index: u32,
    emit: Box<dyn Fn(&'static str) + Send>,
    emit_move: Box<dyn Fn(i32, i32) + Send>,
) {
    register_class();

    let overlay = CreateWindowExW(
        0,
        SNAP_CLASS.as_ptr(),
        SNAP_CLASS.as_ptr(),
        WS_CHILD | WS_VISIBLE | WS_CLIPSIBLINGS | WS_OVERLAPPED,
        0,
        0,
        0,
        0,
        hwnd as HWND,
        std::ptr::null_mut(),
        module_instance(),
        std::ptr::null_mut(),
    );

    if overlay.is_null() {
        return;
    }

    let mut states = SNAP_WINDOWS.lock().expect("snap state poisoned");
    if let Some(old) = states.remove(&hwnd) {
        RemoveWindowSubclass(hwnd as HWND, Some(parent_subclass_proc), SUBCLASS_ID);
        DestroyWindow(old.overlay as _);
    }

    states.insert(
        hwnd,
        SnapState {
            overlay,
            titlebar_height,
            button_width,
            right_index,
            hovering: false,
            pressing: false,
            last_x: 0,
            last_y: 0,
            emit,
            emit_move,
        },
    );
    drop(states);

    SetWindowSubclass(hwnd as HWND, Some(parent_subclass_proc), SUBCLASS_ID, 0);
    update_overlay_position(hwnd as HWND);
}

pub fn uninstall<R: Runtime>(window: &WebviewWindow<R>) -> Result<()> {
    let hwnd = window_hwnd(window)? as isize;
    window.run_on_main_thread(move || unsafe {
        remove(hwnd as HWND);
    })?;
    Ok(())
}


fn window_hwnd(window: &impl HasWindowHandle) -> Result<HWND> {
    let handle = window.window_handle().map_err(|e| eyre::eyre!("{e}"))?;
    match handle.as_raw() {
        RawWindowHandle::Win32(handle) => {
            let hwnd = handle.hwnd.get();
            Ok(hwnd as HWND)
        }
        _ => Err(eyre::eyre!("native snap overlay requires Win32 window handle").into()),
    }
}

unsafe fn register_class() {
    let class = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(overlay_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: module_instance(),
        hIcon: std::ptr::null_mut(),
        hCursor: std::ptr::null_mut(),
        hbrBackground: GetStockObject(NULL_BRUSH) as HBRUSH,
        lpszMenuName: std::ptr::null(),
        lpszClassName: SNAP_CLASS.as_ptr(),
        hIconSm: std::ptr::null_mut(),
    };
    RegisterClassExW(&class);
}

unsafe fn module_instance() -> HINSTANCE {
    GetModuleHandleW(std::ptr::null())
}

unsafe fn update_overlay_position(hwnd: HWND) {
    let states = SNAP_WINDOWS.lock().expect("snap state poisoned");
    let Some(state) = states.get(&(hwnd as isize)) else {
        return;
    };

    let mut rect = std::mem::zeroed();
    if GetClientRect(hwnd, &mut rect) == 0 {
        return;
    }

    let dpi = GetDpiForWindow(hwnd);
    let scale = dpi as u64;
    let button_width = scaled(state.button_width, scale).max(1);
    let titlebar_height = scaled(state.titlebar_height, scale).max(1);
    let x = rect.right - (button_width * (state.right_index as i32 + 1));

    SetWindowPos(
        state.overlay,
        HWND_TOP,
        x,
        0,
        button_width,
        titlebar_height,
        SWP_ASYNCWINDOWPOS | SWP_SHOWWINDOW,
    );
}

fn scaled(value: u32, dpi: u64) -> i32 {
    ((value as u64 * dpi + 48) / 96) as i32
}

unsafe fn remove(hwnd: HWND) {
    RemoveWindowSubclass(hwnd as _, Some(parent_subclass_proc), SUBCLASS_ID);
    if let Some(state) = SNAP_WINDOWS.lock().expect("snap state poisoned").remove(&(hwnd as isize)) {
        DestroyWindow(state.overlay as _);
    }
}

unsafe fn emit(hwnd: HWND, event: &'static str) {
    if let Some(state) = SNAP_WINDOWS.lock().expect("snap state poisoned").get(&(hwnd as isize)) {
        (state.emit)(event);
    }
}

unsafe extern "system" fn parent_subclass_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _subclass_id: usize,
    _ref_data: usize,
) -> LRESULT {
    match msg {
        WM_SIZE | WM_DPICHANGED => update_overlay_position(hwnd),
        WM_CLOSE => remove(hwnd),
        _ => {}
    }

    DefSubclassProc(hwnd, msg, wparam, lparam)
}

unsafe extern "system" fn overlay_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            let createstruct = lparam as *const CREATESTRUCTW;
            if !createstruct.is_null() {
                return 0;
            }
        }
        WM_NCHITTEST => return HTMAXBUTTON as LRESULT,
        WM_NCMOUSEMOVE => {
            if let Some(parent) = parent_for_overlay(hwnd) {
                let mut point = POINT {
                    x: (lparam as i16) as i32,
                    y: ((lparam >> 16) as i16) as i32,
                };
                ScreenToClient(parent, &mut point);
                let mut states = SNAP_WINDOWS.lock().expect("snap state poisoned");
                if let Some(state) = states.get_mut(&(parent as isize)) {
                    if state.last_x != point.x || state.last_y != point.y {
                        state.last_x = point.x;
                        state.last_y = point.y;
                        (state.emit_move)(point.x, point.y);
                    }

                    if !state.hovering {
                        state.hovering = true;
                        drop(states);
                        emit(parent, EVENT_MOUSEENTER);
                    }
                }

                let mut track = TRACKMOUSEEVENT {
                    cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                    dwFlags: TME_LEAVE | TME_NONCLIENT,
                    hwndTrack: hwnd,
                    dwHoverTime: 0,
                };
                TrackMouseEvent(&mut track);
            }
            return 0;
        }
        WM_NCMOUSELEAVE => {
            if let Some(parent) = parent_for_overlay(hwnd) {
                let mut states = SNAP_WINDOWS.lock().expect("snap state poisoned");
                if let Some(state) = states.get_mut(&(parent as isize)) {
                    state.hovering = false;
                    state.pressing = false;
                }
                drop(states);
                emit(parent, EVENT_MOUSELEAVE);
            }
            return 0;
        }
        WM_NCLBUTTONDOWN => {
            if let Some(parent) = parent_for_overlay(hwnd) {
                let mut states = SNAP_WINDOWS.lock().expect("snap state poisoned");
                if let Some(state) = states.get_mut(&(parent as isize)) {
                    state.pressing = true;
                }
                drop(states);
                emit(parent, EVENT_MOUSEDOWN);
            }
            return 0;
        }
        WM_NCLBUTTONUP => {
            if let Some(parent) = parent_for_overlay(hwnd) {
                let mut states = SNAP_WINDOWS.lock().expect("snap state poisoned");
                let click = states
                    .get_mut(&(parent as isize))
                    .map(|state| {
                        let click = state.pressing;
                        state.pressing = false;
                        click
                    })
                    .unwrap_or(false);
                drop(states);
                emit(parent, EVENT_MOUSEUP);
                if click {
                    emit(parent, EVENT_CLICK);
                }
            }
            return 0;
        }
        _ => {}
    }

    DefWindowProcW(hwnd, msg, wparam, lparam)
}

unsafe fn parent_for_overlay(overlay: HWND) -> Option<HWND> {
    let states = SNAP_WINDOWS.lock().expect("snap state poisoned");
    states
        .iter()
        .find_map(|(parent, state)| (state.overlay == overlay).then_some(*parent as HWND))
}
