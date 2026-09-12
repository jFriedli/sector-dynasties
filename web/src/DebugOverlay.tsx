import { useEffect, useState } from "react";
import {
  entityCountsFromState,
  formatDurationMs,
  type LastStepPerformance,
} from "./simPerformance";
import type { SimStateSnapshot, StateSummary } from "./simTypes";

// Developer-facing debug tooling (issue #35), not player-facing content: it
// is intentionally kept out of `src/content/copy.ts` so the copy lint
// (`tests/copy-lint.test.ts`, see docs/CONTENT_GUIDE.md) never scans it.
// Nothing here should ever describe the game world to a player.
const TOGGLE_KEY = "`";

/** Tracks whether the debug overlay should be shown. Hidden by default;
 * toggled with the backtick key so it never gets in a player's way. */
export function useDebugOverlayVisible(): boolean {
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === TOGGLE_KEY) {
        setVisible((prev) => !prev);
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);

  return visible;
}

interface DebugOverlayProps {
  summary: StateSummary;
  snapshot: SimStateSnapshot;
  lastStep: LastStepPerformance | null;
}

/** Always-available debug overlay: current tick/year, seed, and a summary
 * of active RNG domain streams. See issue #35. */
export function DebugOverlay({ summary, snapshot, lastStep }: DebugOverlayProps) {
  const entityCounts = entityCountsFromState(snapshot, summary);

  return (
    <aside
      data-testid="debug-overlay"
      style={{
        position: "fixed",
        bottom: 12,
        right: 12,
        maxWidth: 320,
        padding: "8px 12px",
        background: "rgba(0, 0, 0, 0.82)",
        color: "#8fe08f",
        fontFamily: "monospace",
        fontSize: 12,
        lineHeight: 1.5,
        borderRadius: 4,
        zIndex: 9999,
        pointerEvents: "none",
      }}
    >
      <div>debug (backtick to toggle)</div>
      <div>tick: {summary.tick}</div>
      <div>year: {summary.year}</div>
      <div>seed: {summary.seed}</div>
      <div>
        last step_days:{" "}
        {lastStep ? `${lastStep.days} days in ${formatDurationMs(lastStep.durationMs)}` : "n/a"}
      </div>
      <div>total entities: {entityCounts.total}</div>
      <div>
        entities: systems {entityCounts.systems}, planets {entityCounts.planets}, countries{" "}
        {entityCounts.countries}, cities {entityCounts.cities}, dynasty members{" "}
        {entityCounts.dynastyMembers}
      </div>
      <div>rng domains:</div>
      <ul style={{ margin: "0 0 0 12px", padding: 0 }}>
        {summary.rng_domains.map((domain) => (
          <li key={domain.domain}>
            {domain.domain}: {domain.fingerprint}
          </li>
        ))}
      </ul>
    </aside>
  );
}
