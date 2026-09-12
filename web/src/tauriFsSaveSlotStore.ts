import { invoke } from "@tauri-apps/api/core";
import type { SaveSlot, SaveSlotStore } from "./saveSlots";

// Filesystem-backed `SaveSlotStore` for the Tauri desktop shell (issue
// #102's Linux packaging spike). Each slot is one JSON file on disk. The
// Rust side (`web/src-tauri/src/main.rs`) exposes only generic file
// primitives (read/write/list/remove a path) and knows nothing about
// `SimState`; the exact same `SimHandle.to_json()` / `SimHandle.fromJson()`
// round trip that already goes through `sim-core`'s save/load path (see
// `save::load_and_migrate` in `crates/sim-core/src/save.rs`) still governs
// what a slot's `stateJson` means, both here and in the browser build. This
// is a second `SaveSlotStore` implementation alongside
// `indexedDbSaveSlotStore.ts`, not a replacement for it: `App.tsx` picks
// between the two at startup with `isTauriRuntime()` (`tauriRuntime.ts`),
// so the browser target keeps using IndexedDB unchanged.
//
// `invokeFn` is injectable, the same way `createIndexedDbSaveSlotStore`
// takes an injectable `IDBFactory`, so tests can exercise this module
// against a fake in-memory "filesystem" without a real Tauri runtime.

export type TauriInvoke = <T>(command: string, args?: Record<string, unknown>) => Promise<T>;

function slotFileName(id: string): string {
  // Slot ids are free-form strings (see `saveSlots.ts`); sanitize before
  // using one as a file name so an unexpected id can't escape the saves
  // directory or collide with the `.json` suffix check in `list_dir`.
  const safeId = id.replace(/[^a-zA-Z0-9_-]/g, "_");
  return `${safeId}.json`;
}

export async function createTauriFsSaveSlotStore(
  invokeFn: TauriInvoke = invoke,
): Promise<SaveSlotStore> {
  const dir = await invokeFn<string>("saves_dir");

  function pathFor(id: string): string {
    return `${dir}/${slotFileName(id)}`;
  }

  return {
    async put(slot: SaveSlot): Promise<void> {
      await invokeFn("write_text_file", {
        path: pathFor(slot.id),
        contents: JSON.stringify(slot),
      });
    },

    async get(id: string): Promise<SaveSlot | undefined> {
      try {
        const contents = await invokeFn<string>("read_text_file", { path: pathFor(id) });
        return JSON.parse(contents) as SaveSlot;
      } catch {
        return undefined;
      }
    },

    async list(): Promise<SaveSlot[]> {
      const names = await invokeFn<string[]>("list_dir", { path: dir });
      const slots = await Promise.all(
        names.map(async (name) => {
          const contents = await invokeFn<string>("read_text_file", { path: `${dir}/${name}` });
          return JSON.parse(contents) as SaveSlot;
        }),
      );
      return slots;
    },

    async delete(id: string): Promise<void> {
      await invokeFn("remove_file", { path: pathFor(id) });
    },
  };
}
