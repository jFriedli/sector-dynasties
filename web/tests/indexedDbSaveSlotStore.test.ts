import { IDBFactory } from "fake-indexeddb";
import { beforeEach, describe, expect, it } from "vitest";
import { createIndexedDbSaveSlotStore } from "../src/indexedDbSaveSlotStore";
import { createSaveSlot, DEFAULT_SLOT_ID } from "../src/saveSlots";

// Exercises the real IndexedDB code path (open/upgrade/transaction/close)
// against `fake-indexeddb`'s in-memory `IDBFactory`, injected the same way
// the browser's global `indexedDB` is at runtime. jsdom itself does not
// implement IndexedDB, hence the fake.

describe("createIndexedDbSaveSlotStore", () => {
  let store: ReturnType<typeof createIndexedDbSaveSlotStore>;

  beforeEach(() => {
    // A fresh factory per test so a stale database from a previous test
    // never leaks state across tests.
    store = createIndexedDbSaveSlotStore(new IDBFactory());
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
});
