# tauri-plugin-frame

Custom window frame controls for Tauri v2 on Windows. Supports Windows Snap Layout and custom titlebar styling.

![demo](./wheeee.gif)

## Install

```bash
cargo add tauri-plugin-frame
```

Add to `src-tauri/capabilities/default.json`:
```json
{
  "permissions": [
    "core:window:allow-close",
    "core:window:allow-center",
    "core:window:allow-minimize",
    "core:window:allow-maximize",
    "core:window:allow-set-size",
    "core:window:allow-set-focus",
    "core:window:allow-is-maximized",
    "core:window:allow-start-dragging",
    "core:window:allow-toggle-maximize",
    "frame:default"
  ]
}
```

Set in `tauri.conf.json`:
```json
{
  "app": {
    "withGlobalTauri": true,
    "windows": [
      {
        "decorations": false,
      }
    ]
  }
}
```

## Usage

**Basic (manual per window):**
```rust
use tauri::Manager;
use tauri_plugin_frame::WebviewWindowExt;

tauri::Builder::default()
    .plugin(tauri_plugin_frame::init())
    .setup(|app| {
        app.get_webview_window("main").unwrap().create_overlay_titlebar()?;
        Ok(())
    })
```

**Auto-apply to all windows:**
```rust
use tauri_plugin_frame::FramePluginBuilder;

tauri::Builder::default()
    .plugin(
        FramePluginBuilder::new()
            .titlebar_height(40)  // default: 32
            .auto_titlebar(true)
            .build()
    )
```

## CSS Styling

```css
[data-tauri-frame-tb] { background: rgba(0,0,0,0.1); }
#frame-tb-minimize, #frame-tb-maximize, #frame-tb-close { /* styles */ }
```

## API

| Option | Default | Description |
|--------|---------|-------------|
| `titlebar_height(u32)` | 32 | Titlebar height in pixels |
| `auto_titlebar(bool)` | false | Auto-apply to all windows |

| Method | Description |
|--------|-------------|
| `create_overlay_titlebar()` | Apply titlebar with default height |
| `create_overlay_titlebar_with_height(u32)` | Apply titlebar with custom height |

## License

MIT - Originally forked from [tauri-plugin-decorum](https://github.com/clearlysid/tauri-plugin-decorum)
