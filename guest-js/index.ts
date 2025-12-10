import { invoke } from "@tauri-apps/api/core";

export async function showSnapOverlay(): Promise<void> {
  await invoke("plugin:frame|show_snap_overlay");
}
