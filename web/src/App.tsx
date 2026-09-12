import { useCallback, useEffect, useState } from "react";
import init, { SimHandle } from "./wasm/sim_wasm.js";
import { copy } from "./content/copy";
import { SectorBrowser } from "./SectorBrowser";
import { TimeControls } from "./TimeControls";
import type { SimStateSnapshot, StateSummary } from "./simTypes";

const DAYS_PER_YEAR = 360;
const YEAR_STEP_OPTIONS = [1, 5, 10] as const;

export function App() {
  const [handle, setHandle] = useState<SimHandle | null>(null);
  const [summary, setSummary] = useState<StateSummary | null>(null);
  const [snapshot, setSnapshot] = useState<SimStateSnapshot | null>(null);
  const [yearsToAdvance, setYearsToAdvance] = useState<number>(5);

  useEffect(() => {
    let cancelled = false;
    init().then(() => {
      if (cancelled) return;
      const h = new SimHandle(BigInt(42));
      setHandle(h);
      setSummary(JSON.parse(h.summary_json()) as StateSummary);
      setSnapshot(JSON.parse(h.to_json()) as SimStateSnapshot);
    });
    return () => {
      cancelled = true;
    };
  }, []);

  const advanceOneYear = useCallback(() => {
    if (!handle) return;
    handle.step_days(DAYS_PER_YEAR);
    setSummary(JSON.parse(handle.summary_json()) as StateSummary);
    setSnapshot(JSON.parse(handle.to_json()) as SimStateSnapshot);
  }, [handle]);

  // A synchronous call is intentional here: the bootstrap slice's data
  // sizes make even a 10-year advance (3600 simulated days) fast enough
  // not to noticeably block the main thread. Revisit with a worker or
  // chunked stepping if `SimState` grows large enough to change that.
  const advanceYears = useCallback(() => {
    if (!handle) return;
    handle.step_days(DAYS_PER_YEAR * yearsToAdvance);
    setSummary(JSON.parse(handle.summary_json()) as StateSummary);
    setSnapshot(JSON.parse(handle.to_json()) as SimStateSnapshot);
  }, [handle, yearsToAdvance]);

  if (!summary || !snapshot) {
    return <p className="loading">{copy.loading}</p>;
  }

  return (
    <main className="app-shell">
      <header className="app-header">
        <h1>{copy.title}</h1>
        <p>{copy.subtitle}</p>
      </header>
      <dl className="summary-grid">
        <div className="summary-item">
          <dt>{copy.yearLabel}</dt>
          <dd>{summary.year}</dd>
        </div>
        <div className="summary-item summary-item--systems">
          <dt>{copy.systemsLabel}</dt>
          <dd>{summary.system_count}</dd>
        </div>
        <div className="summary-item summary-item--cities">
          <dt>{copy.citiesLabel}</dt>
          <dd>{summary.city_count}</dd>
        </div>
        <div className="summary-item summary-item--population">
          <dt>{copy.populationLabel}</dt>
          <dd>{summary.total_population.toLocaleString()}</dd>
        </div>
        <div className="summary-item summary-item--dynasty">
          <dt>{summary.dynasty_name}</dt>
          <dd>
            {copy.wealthLabel}: {summary.dynasty_wealth.toFixed(2)}
          </dd>
        </div>
      </dl>
      <TimeControls
        yearsToAdvance={yearsToAdvance}
        yearStepOptions={YEAR_STEP_OPTIONS}
        onYearsToAdvanceChange={setYearsToAdvance}
        onAdvanceOneYear={advanceOneYear}
        onAdvanceYears={advanceYears}
      />
      <SectorBrowser sector={snapshot.sector} />
    </main>
  );
}
