import { beforeEach, describe, expect, it } from "vitest";
import { createTauriFsSaveSlotStore, type TauriInvoke } from "../src/tauriFsSaveSlotStore";
import { createSaveSlot, DEFAULT_SLOT_ID } from "../src/saveSlots";

// Exercises `createTauriFsSaveSlotStore` against a fake `invoke` that
// simulates the Rust commands in `web/src-tauri/src/main.rs`
// (`saves_dir`/`read_text_file`/`write_text_file`/`list_dir`/`remove_file`)
// with an in-memory "filesystem", the same way
// `indexedDbSaveSlotStore.test.ts` exercises its store against
// `fake-indexeddb` instead of a real browser. No real Tauri runtime is
// available in this test environment, so `invoke` is always the injected
// fake, never the real `@tauri-apps/api/core` import.

const SAVES_DIR = "/fake/app-data/saves";

function createFakeInvoke(): TauriInvoke {
  const files = new Map<string, string>();

  return (async <T>(command: string, args?: Record<string, unknown>): Promise<T> => {
    switch (command) {
      case "saves_dir":
        return SAVES_DIR as unknown as T;
      case "write_text_file": {
        const { path, contents } = args as { path: string; contents: string };
        files.set(path, contents);
        return undefined as T;
      }
      case "read_text_file": {
        const { path } = args as { path: string };
        const contents = files.get(path);
        if (contents === undefined) {
          throw new Error(`no such file: ${path}`);
        }
        return contents as unknown as T;
      }
      case "remove_file": {
        const { path } = args as { path: string };
        files.delete(path);
        return undefined as T;
      }
      case "list_dir": {
        const { path } = args as { path: string };
        const prefix = `${path}/`;
        const names = Array.from(files.keys())
          .filter((key) => key.startsWith(prefix) && key.endsWith(".json"))
          .map((key) => key.slice(prefix.length));
        return names as unknown as T;
      }
      default:
        throw new Error(`unexpected command: ${command}`);
    }
  }) as TauriInvoke;
}

describe("createTauriFsSaveSlotStore", () => {
  let store: Awaited<ReturnType<typeof createTauriFsSaveSlotStore>>;

  beforeEach(async () => {
    // A fresh fake filesystem per test, the same reasoning as the fresh
    // `IDBFactory` in `indexedDbSaveSlotStore.test.ts`.
    store = await createTauriFsSaveSlotStore(createFakeInvoke());
  });

  it("returns undefined for a slot that was never saved", async () => {
    await expect(store.get(DEFAULT_SLOT_ID)).resolves.toBeUndefined();
  });

  it("round-trips a saved slot back out unchanged", async () => {
    const slot = createSaveSlot(DEFAULT_SLOT_ID, "Quick save", '{"schema_version":1,"tick":7}');

    await store.put(slot);
    const loaded = await store.get(DEFAULT_SLOT_ID);

    expect(loaded).toEqual(slot);
  });

  it("overwrites a slot with the same id rather than duplicating it", async () => {
    const first = createSaveSlot(DEFAULT_SLOT_ID, "Quick save", '{"tick":1}');
    const second = createSaveSlot(DEFAULT_SLOT_ID, "Quick save", '{"tick":2}');

    await store.put(first);
    await store.put(second);

    const loaded = await store.get(DEFAULT_SLOT_ID);
    expect(loaded?.stateJson).toBe('{"tick":2}');

    const all = await store.list();
    expect(all).toHaveLength(1);
  });

  it("lists every saved slot", async () => {
    await store.put(createSaveSlot("a", "Slot A", "{}"));
    await store.put(createSaveSlot("b", "Slot B", "{}"));

    const all = await store.list();
    expect(all.map((slot) => slot.id).sort()).toEqual(["a", "b"]);
  });

  it("deletes a slot so it no longer loads or lists", async () => {
    await store.put(createSaveSlot(DEFAULT_SLOT_ID, "Quick save", "{}"));
    await store.delete(DEFAULT_SLOT_ID);

    await expect(store.get(DEFAULT_SLOT_ID)).resolves.toBeUndefined();
    await expect(store.list()).resolves.toEqual([]);
  });

  it("sanitizes slot ids before using them as file names", async () => {
    const weirdId = "../etc/passwd";
    await store.put(createSaveSlot(weirdId, "Weird", "{}"));

    const loaded = await store.get(weirdId);
    expect(loaded?.id).toBe(weirdId);
  });
});
