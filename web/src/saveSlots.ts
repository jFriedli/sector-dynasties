// A minimal, storage-agnostic save-slot abstraction (issue #34). A slot is
// a name, a timestamp, and the exact JSON blob produced by
// `SimHandle.to_json()` / consumed by `SimHandle.fromJson()` (see
// `crates/sim-wasm`). This module never inspects `SimState` fields: the
// state JSON is opaque here, so a future save schema change only ever
// touches `sim-core`/`sim-wasm` and the migration path they already own
// (see docs/ARCHITECTURE.md's persistence section), never this file.
//
// Where the bytes actually live is a separate concern owned by whatever
// implements `SaveSlotStore` (IndexedDB today, see
// `indexedDbSaveSlotStore.ts`); this module and its tests never assume a
// particular backing store.

export interface SaveSlot {
  /** Stable identifier for the slot, distinct from its display `name` so
   * renaming a slot never changes its storage key. */
  id: string;
  name: string;
  /** ISO-8601 timestamp of when this slot was last written. */
  savedAt: string;
  /** Exact `SimState::to_json()` output for this slot. Opaque here. */
  stateJson: string;
}

/** Storage-agnostic slot persistence. Implementations own where a slot's
 * bytes actually live; this is the shape the UI and tests depend on. */
export interface SaveSlotStore {
  put(slot: SaveSlot): Promise<void>;
  get(id: string): Promise<SaveSlot | undefined>;
  list(): Promise<SaveSlot[]>;
  delete(id: string): Promise<void>;
}

/** Builds a `SaveSlot` from a slot id, display name, and the raw state
 * JSON to persist. `now` is injectable so tests get a deterministic
 * timestamp instead of depending on wall-clock time. */
export function createSaveSlot(
  id: string,
  name: string,
  stateJson: string,
  now: () => Date = () => new Date(),
): SaveSlot {
  return { id, name, savedAt: now().toISOString(), stateJson };
}

/** The single slot id today's minimal save UI reads and writes. A real
 * save browser with multiple named slots (issue #93) can introduce more
 * ids later without changing this module's shape. */
export const DEFAULT_SLOT_ID = "default";
