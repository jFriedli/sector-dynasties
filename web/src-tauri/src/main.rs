// Windows desktop packaging spike (issue #103): wraps the existing static
// web/dist build in a native window. Deliberately has no commands, no IPC,
// and no simulation logic of its own; sim-core stays the only place game
// rules live (see docs/ARCHITECTURE.md). This binary's only job is to open
// a webview pointed at the frontend that already ships to the browser.
//
// What's verified: this crate and tauri.conf.json compile and bundle (NSIS)
// on GitHub's windows-latest CI runner (see .github/workflows/ci.yml's
// windows-tauri job) and compile-and-launch-without-crashing under Xvfb on
// Linux (a sanity check only). What's NOT verified: real Windows runtime
// behavior (the actual window, WebView2, and installer double-click UX).
// See issue #152 for that real-hardware follow-up.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running the Sector Dynasties Tauri shell");
}
