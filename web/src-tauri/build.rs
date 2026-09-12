// `icons/` (referenced from tauri.conf.json's `bundle.icon`, embedded into the
// Windows exe/installer by tauri-build below) is a procedurally generated
// placeholder (a solid square with "SD" text via ImageMagick), not real
// branding; see docs/ASSETS.md's placeholder-asset rule and issue #153 for
// swapping in a real icon.
fn main() {
    tauri_build::build()
}
