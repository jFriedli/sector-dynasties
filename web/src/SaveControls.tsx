import { copy } from "./content/copy";

// Minimal save/load UI (issue #34): one slot, a save button, a load
// button, and a status line. A real save browser with multiple named
// slots is issue #93, deliberately not built here.

export type SaveLoadStatus =
  | { kind: "idle" }
  | { kind: "saved"; savedAt: string }
  | { kind: "loaded"; savedAt: string }
  | { kind: "error"; message: string };

interface SaveControlsProps {
  hasSavedSlot: boolean;
  status: SaveLoadStatus;
  onSave: () => void;
  onLoad: () => void;
}

export function SaveControls({ hasSavedSlot, status, onSave, onLoad }: SaveControlsProps) {
  return (
    <section className="controls" aria-label={copy.saveControlsLabel}>
      <button className="button" type="button" onClick={onSave}>
        {copy.saveButton}
      </button>
      <button className="button" type="button" onClick={onLoad} disabled={!hasSavedSlot}>
        {copy.loadButton}
      </button>
      <p className="save-status" role="status">
        {statusMessage(status)}
      </p>
    </section>
  );
}

function statusMessage(status: SaveLoadStatus): string {
  switch (status.kind) {
    case "idle":
      return copy.saveStatusIdle;
    case "saved":
      return copy.saveStatusSaved(formatTimestamp(status.savedAt));
    case "loaded":
      return copy.saveStatusLoaded(formatTimestamp(status.savedAt));
    case "error":
      return copy.saveStatusError(status.message);
  }
}

function formatTimestamp(savedAt: string): string {
  const date = new Date(savedAt);
  return Number.isNaN(date.getTime()) ? savedAt : date.toLocaleString();
}
