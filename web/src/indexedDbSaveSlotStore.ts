import type { SaveSlot, SaveSlotStore } from "./saveSlots";

// IndexedDB-backed `SaveSlotStore`. Deliberately not `localStorage`:
// campaign saves are not small, and `localStorage` is synchronous and
// size-limited (see docs/ARCHITECTURE.md's persistence section and issue
// #34). This is the only place in the frontend that talks to IndexedDB
// directly; everything else goes through the `SaveSlotStore` interface.
//
// The model here must not assume browser storage is authoritative
// long-term: a slot is just a cache of a `SimState::to_json()` blob, and
// export/import to a real file (issue #108) or a future server-backed
// store can replace or supplement this without changing `SaveSlotStore`.

const DB_NAME = "sector-dynasties-saves";
const DB_VERSION = 1;
const STORE_NAME = "save-slots";

function openDatabase(factory: IDBFactory): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = factory.open(DB_NAME, DB_VERSION);
    request.onupgradeneeded = () => {
      const db = request.result;
      if (!db.objectStoreNames.contains(STORE_NAME)) {
        db.createObjectStore(STORE_NAME, { keyPath: "id" });
      }
    };
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error ?? new Error("failed to open save database"));
  });
}

function awaitRequest<T>(request: IDBRequest<T>): Promise<T> {
  return new Promise((resolve, reject) => {
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error ?? new Error("IndexedDB request failed"));
  });
}

function awaitTransaction(transaction: IDBTransaction): Promise<void> {
  return new Promise((resolve, reject) => {
    transaction.oncomplete = () => resolve();
    transaction.onerror = () =>
      reject(transaction.error ?? new Error("IndexedDB transaction failed"));
    transaction.onabort = () =>
      reject(transaction.error ?? new Error("IndexedDB transaction aborted"));
  });
}

/** Creates a `SaveSlotStore` backed by IndexedDB. `factory` defaults to the
 * browser's global `indexedDB` and is injectable so tests can pass a fake
 * implementation (see `tests/indexedDbSaveSlotStore.test.ts`) instead of a
 * real browser database. */
export function createIndexedDbSaveSlotStore(
  factory: IDBFactory = globalThis.indexedDB,
): SaveSlotStore {
  async function writeOnly(mode: "readwrite", run: (store: IDBObjectStore) => void) {
    const db = await openDatabase(factory);
    try {
      const transaction = db.transaction(STORE_NAME, mode);
      run(transaction.objectStore(STORE_NAME));
      await awaitTransaction(transaction);
    } finally {
      db.close();
    }
  }

  async function readOnly<T>(run: (store: IDBObjectStore) => IDBRequest<T>): Promise<T> {
    const db = await openDatabase(factory);
    try {
      const transaction = db.transaction(STORE_NAME, "readonly");
      return await awaitRequest(run(transaction.objectStore(STORE_NAME)));
    } finally {
      db.close();
    }
  }

  return {
    async put(slot: SaveSlot): Promise<void> {
      await writeOnly("readwrite", (store) => {
        store.put(slot);
      });
    },
    async get(id: string): Promise<SaveSlot | undefined> {
      const result = await readOnly<SaveSlot | undefined>((store) => store.get(id));
      return result;
    },
    async list(): Promise<SaveSlot[]> {
      const result = await readOnly<SaveSlot[]>((store) => store.getAll());
      return result;
    },
    async delete(id: string): Promise<void> {
      await writeOnly("readwrite", (store) => {
        store.delete(id);
      });
    },
  };
}
