import { useCallback, useEffect, useState } from "react";
import init, { SimHandle } from "./wasm/sim_wasm.js";
import { copy } from "./content/copy";
import { DebugOverlay, useDebugOverlayVisible } from "./DebugOverlay";
import type { StateSummary } from "./simTypes";

const DAYS_PER_YEAR = 360;
const YEAR_STEP_OPTIONS = [1, 5, 10] as const;

export function App() {
  const [handle, setHandle] = useState<SimHandle | null>(null);
  const [summary, setSummary] = useState<StateSummary | null>(null);
  const [yearsToAdvance, setYearsToAdvance] = useState<number>(5);
  const debugOverlayVisible = useDebugOverlayVisible();

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
    return <p>{copy.loading}</p>;
  }

  return (
    <main>
      <h1>{copy.title}</h1>
      <p>{copy.subtitle}</p>
      <dl>
        <dt>{copy.yearLabel}</dt>
        <dd>{summary.year}</dd>
        <dt>{copy.systemsLabel}</dt>
        <dd>{summary.system_count}</dd>
        <dt>{copy.citiesLabel}</dt>
        <dd>{summary.city_count}</dd>
        <dt>{copy.populationLabel}</dt>
        <dd>{summary.total_population.toLocaleString()}</dd>
        <dt>{summary.dynasty_name}</dt>
        <dd>
          {copy.wealthLabel}: {summary.dynasty_wealth.toFixed(2)}
        </dd>
      </dl>
      <button onClick={advanceOneYear}>{copy.advanceOneYear}</button>
      <label>
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
      <button onClick={advanceYears}>{copy.advanceYearsButton}</button>
      {debugOverlayVisible && <DebugOverlay summary={summary} />}
    </main>
  );
}
