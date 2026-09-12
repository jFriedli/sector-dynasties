import { useMemo, useState } from "react";
import {
  ENTITY_TYPES,
  entityRowsFromState,
  entityTypeLabel,
  filterEntityRows,
} from "./stateInspector";
import type { EntityType } from "./stateInspector";
import type { SimStateSnapshot, StateSummary } from "./simTypes";

// Developer-facing state inspector dev panel (issue #93), extending the
// debug overlay (issue #35) rather than being a wholly separate surface:
// it shares the same visibility gating in App.tsx (the backtick toggle) so
// it never gets in a player's way and is never rendered outside of debug
// tooling. Dev-only, so intentionally not routed through `src/content`:
// see the copy lint's scope in docs/CONTENT_GUIDE.md and DebugOverlay.tsx's
// header comment for the same reasoning.

const ALL_TYPES_VALUE = "all";

interface StateInspectorProps {
  summary: StateSummary;
  snapshot: SimStateSnapshot;
}

/** Searchable tree/table view over the current SimState: every system,
 * planet, country, city, dynasty member, and RNG domain, filterable by
 * entity type and name. See stateInspector.ts for the data shaping. */
export function StateInspector({ summary, snapshot }: StateInspectorProps) {
  const [query, setQuery] = useState("");
  const [typeFilter, setTypeFilter] = useState<EntityType | null>(null);

  const rows = useMemo(() => entityRowsFromState(snapshot, summary), [snapshot, summary]);
  const filtered = useMemo(
    () => filterEntityRows(rows, query, typeFilter),
    [rows, query, typeFilter],
  );

  return (
    <aside
      data-testid="state-inspector"
      style={{
        position: "fixed",
        bottom: 12,
        left: 12,
        width: 360,
        maxHeight: "60vh",
        display: "flex",
        flexDirection: "column",
        padding: "8px 12px",
        background: "rgba(0, 0, 0, 0.82)",
        color: "#8fe08f",
        fontFamily: "monospace",
        fontSize: 12,
        lineHeight: 1.5,
        borderRadius: 4,
        zIndex: 9999,
      }}
    >
      <div>
        state inspector ({filtered.length}/{rows.length})
      </div>
      <div style={{ display: "flex", gap: 6, margin: "6px 0" }}>
        <input
          data-testid="state-inspector-search"
          type="text"
          placeholder="search by name"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          style={{
            flex: 1,
            background: "#0b1a0b",
            color: "#8fe08f",
            border: "1px solid #2f4f2f",
            fontFamily: "monospace",
            fontSize: 12,
          }}
        />
        <select
          data-testid="state-inspector-type-filter"
          value={typeFilter ?? ALL_TYPES_VALUE}
          onChange={(event) =>
            setTypeFilter(
              event.target.value === ALL_TYPES_VALUE ? null : (event.target.value as EntityType),
            )
          }
          style={{
            background: "#0b1a0b",
            color: "#8fe08f",
            border: "1px solid #2f4f2f",
            fontFamily: "monospace",
            fontSize: 12,
          }}
        >
          <option value={ALL_TYPES_VALUE}>all types</option>
          {ENTITY_TYPES.map((type) => (
            <option key={type} value={type}>
              {entityTypeLabel(type)}
            </option>
          ))}
        </select>
      </div>
      <ul
        data-testid="state-inspector-results"
        style={{ margin: 0, padding: 0, listStyle: "none", overflowY: "auto" }}
      >
        {filtered.length === 0 ? (
          <li>no matching entities</li>
        ) : (
          filtered.map((row) => (
            <li key={row.key}>
              [{entityTypeLabel(row.type)}] {row.path ? `${row.path} > ` : ""}
              {row.name} ({row.detail})
            </li>
          ))
        )}
      </ul>
    </aside>
  );
}
