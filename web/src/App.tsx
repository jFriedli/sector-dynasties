import { useCallback, useEffect, useState } from "react";
import init, { SimHandle } from "./wasm/sim_wasm.js";
import { copy } from "./content/copy";
import { CityDetailPanel } from "./CityDetailPanel";
import { DebugOverlay, useDebugOverlayVisible } from "./DebugOverlay";
import { DynastyPanel } from "./DynastyPanel";
import { SectorBrowser } from "./SectorBrowser";
import { TimeControls } from "./TimeControls";
import { cityDetailById } from "./sectorBrowser";
import type { SimStateSnapshot, StateSummary } from "./simTypes";

const DAYS_PER_YEAR = 360;
const YEAR_STEP_OPTIONS = [1, 5, 10] as const;

export function App() {
  const [handle, setHandle] = useState<SimHandle | null>(null);
  const [summary, setSummary] = useState<StateSummary | null>(null);
  const [snapshot, setSnapshot] = useState<SimStateSnapshot | null>(null);
  const [selectedCityId, setSelectedCityId] = useState<number | null>(null);
  const [yearsToAdvance, setYearsToAdvance] = useState<number>(5);
  const debugOverlayVisible = useDebugOverlayVisible();

  useEffect(() => {
    let cancelled = false;
    init().then(() => {
      if (cancelled) return;
      const h = new SimHandle(BigInt(42));
      setHandle(h);
      setSummary(JSON.parse(h.summary_json()) as StateSummary);
      const nextSnapshot = JSON.parse(h.to_json()) as SimStateSnapshot;
      setSnapshot(nextSnapshot);
      setSelectedCityId(
        nextSnapshot.sector.systems[0]?.planets[0]?.countries[0]?.cities[0]?.id ?? null,
      );
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
      {debugOverlayVisible && <DebugOverlay summary={summary} />}
      <DynastyPanel members={summary.dynasty_members} />
      <div className="world-panel">
        <SectorBrowser
          sector={snapshot.sector}
          selectedCityId={selectedCityId}
          onSelectCity={setSelectedCityId}
        />
        <CityDetailPanel
          city={selectedCityId ? cityDetailById(snapshot.sector, selectedCityId) : null}
        />
      </div>
    </main>
  );
}
