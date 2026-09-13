#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! Thin native host for the Sector Dynasties web app: issue #102's Linux
//! packaging spike, with issue #103 layering Windows-target packaging on
//! top (see `docs/ARCHITECTURE.md`'s desktop section, and
//! `tauri.windows.conf.json` alongside this crate's `tauri.conf.json`).
//! No Windows-specific Rust code was needed: everything below is already
//! cross-platform through `tauri`'s own APIs.
//!
//! No simulation logic lives here on purpose. `sim-core`'s save/load path
//! (`SimState::to_json`/`from_json`, going through `save::load_and_migrate`,
//! see `crates/sim-core/src/save.rs`) already runs inside the existing WASM
//! bridge loaded by the webview, exactly as it does in a browser tab. These
//! commands are generic filesystem primitives only: a desktop save is a
//! plain JSON file on disk instead of an IndexedDB record, but the bytes
//! written are the same `SimHandle.to_json()` output either way, and
//! loading them back still goes through `SimHandle.fromJson()` in the
//! frontend (see `web/src/tauriFsSaveSlotStore.ts`).
//!
//! What issue #103 actually verified on Windows: `.github/workflows/ci.yml`'s
//! `windows-tauri` job builds this crate and produces an NSIS bundle on a
//! real `windows-latest` GitHub Actions runner. What it did NOT verify: that
//! the installer, the native window, or the WebView2-backed webview actually
//! work when run by a human on real Windows hardware — no CI runner clicks
//! the installer or the app. See issue #152 for that follow-up.

use std::fs;
use std::path::PathBuf;

use tauri::Manager;

#[tauri::command]
fn read_text_file(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|error| error.to_string())
}

#[tauri::command]
fn write_text_file(path: String, contents: String) -> Result<(), String> {
    let target = PathBuf::from(&path);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(&target, contents).map_err(|error| error.to_string())
}

#[tauri::command]
fn remove_file(path: String) -> Result<(), String> {
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

/// Lists the `.json` save-slot files in `path`, by file name only (not full
/// paths): the frontend already knows the directory (see `saves_dir`
/// below) and only needs the names to build per-slot paths itself.
#[tauri::command]
fn list_dir(path: String) -> Result<Vec<String>, String> {
    let dir = PathBuf::from(&path);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let entries = fs::read_dir(&dir).map_err(|error| error.to_string())?;
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| error.to_string())?;
        let entry_path = entry.path();
        let is_json = entry_path
            .extension()
            .map(|extension| extension == "json")
            .unwrap_or(false);
        if !is_json {
            continue;
        }
        if let Some(name) = entry_path.file_name().and_then(|name| name.to_str()) {
            names.push(name.to_string());
        }
    }
    Ok(names)
}

/// The directory this spike stores save-slot files in, creating it on
/// first use. A real integration would likely make this configurable; a
/// single fixed directory is enough for a spike proving the boundary.
#[tauri::command]
fn saves_dir(app: tauri::AppHandle) -> Result<String, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("saves");
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    dir.to_str()
        .map(|value| value.to_string())
        .ok_or_else(|| "save directory path is not valid UTF-8".to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            read_text_file,
            write_text_file,
            remove_file,
            list_dir,
            saves_dir
        ])
        .run(tauri::generate_context!())
        .expect("error while running the Tauri application");
}
