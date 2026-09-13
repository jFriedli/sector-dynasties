// True when running inside the Tauri desktop shell (issue #102's Linux
// packaging spike) rather than a plain browser tab. Tauri's webview
// injects `__TAURI_INTERNALS__` into `window` before any page script
// runs, so checking for it is the standard, dependency-free way to branch
// on "desktop shell vs. browser" without needing a build-time flag. This
// is the only Tauri-specific check anywhere in the frontend; everything
// else that differs between the two targets (see `tauriFsSaveSlotStore.ts`
// vs. `indexedDbSaveSlotStore.ts`) sits behind the same `SaveSlotStore`
// interface (`saveSlots.ts`), so the browser build never depends on Tauri
// being present.
export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}
