import { useCallback, useEffect, useState } from "react";
import init, { SimHandle } from "./wasm/sim_wasm.js";
import { copy } from "./content/copy";
import type { StateSummary } from "./simTypes";

const DAYS_PER_YEAR = 360;
const YEAR_STEP_OPTIONS = [1, 5, 10] as const;

export function App() {
  const [handle, setHandle] = useState<SimHandle | null>(null);
  const [summary, setSummary] = useState<StateSummary | null>(null);
  const [yearsToAdvance, setYearsToAdvance] = useState<number>(5);

  useEffect(() => {
    let cancelled = false;
    init().then(() => {
      if (cancelled) return;
      const h = new SimHandle(BigInt(42));
      setHandle(h);
      setSummary(JSON.parse(h.summary_json()) as StateSummary);
    });
    return () => {
      cancelled = true;
    };
  }, []);

  const advanceOneYear = useCallback(() => {
    if (!handle) return;
    handle.step_days(DAYS_PER_YEAR);
    setSummary(JSON.parse(handle.summary_json()) as StateSummary);
  }, [handle]);

  // A synchronous call is intentional here: the bootstrap slice's data
  // sizes make even a 10-year advance (3600 simulated days) fast enough
  // not to noticeably block the main thread. Revisit with a worker or
  // chunked stepping if `SimState` grows large enough to change that.
  const advanceYears = useCallback(() => {
    if (!handle) return;
    handle.step_days(DAYS_PER_YEAR * yearsToAdvance);
    setSummary(JSON.parse(handle.summary_json()) as StateSummary);
  }, [handle, yearsToAdvance]);

  if (!summary) {
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
      <section className="controls" aria-label={copy.timeControlsLabel}>
        <button className="button" onClick={advanceOneYear}>
          {copy.advanceOneYear}
        </button>
        <label className="control-field">
          {copy.advanceYearsLabel}
          <select
            value={yearsToAdvance}
            onChange={(event) => setYearsToAdvance(Number(event.target.value))}
          >
            {YEAR_STEP_OPTIONS.map((years) => (
              <option key={years} value={years}>
                {years}
              </option>
            ))}
          </select>
        </label>
        <button className="button" onClick={advanceYears}>
          {copy.advanceYearsButton}
        </button>
      </section>
    </main>
  );
}
