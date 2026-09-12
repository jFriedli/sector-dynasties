import { describe, expect, it } from "vitest";
import { createSaveSlot, DEFAULT_SLOT_ID } from "../src/saveSlots";

describe("createSaveSlot", () => {
  it("builds a slot carrying the id, name, and state JSON unchanged", () => {
    const slot = createSaveSlot(DEFAULT_SLOT_ID, "Quick save", '{"schema_version":1}');

    expect(slot.id).toBe(DEFAULT_SLOT_ID);
    expect(slot.name).toBe("Quick save");
    expect(slot.stateJson).toBe('{"schema_version":1}');
  });

  it("stamps the slot with an ISO timestamp from the injected clock", () => {
    const fixedNow = new Date("2026-01-02T03:04:05.000Z");
    const slot = createSaveSlot("a", "A", "{}", () => fixedNow);

    expect(slot.savedAt).toBe("2026-01-02T03:04:05.000Z");
  });

  it("treats the state JSON as opaque, never inspecting or reshaping it", () => {
    const rawJson = '{"schema_version":1,"nested":{"a":[1,2,3]},"unicode":"caf\\u00e9"}';
    const slot = createSaveSlot("a", "A", rawJson);

    expect(slot.stateJson).toBe(rawJson);
  });
});
