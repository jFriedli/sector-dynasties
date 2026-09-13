import { useCallback, useEffect, useState } from "react";
import init, { SimHandle } from "./wasm/sim_wasm.js";
import { copy } from "./content/copy";
import { CityDetailPanel } from "./CityDetailPanel";
import { DebugOverlay, useDebugOverlayVisible } from "./DebugOverlay";
import { DynastyPanel } from "./DynastyPanel";
import { EconomyView } from "./EconomyView";
import { EventsView } from "./EventsView";
import { GlobalSearch } from "./GlobalSearch";
import { createIndexedDbSaveSlotStore } from "./indexedDbSaveSlotStore";
import { Nav, type Screen } from "./Nav";
import { SaveControls, type SaveLoadStatus } from "./SaveControls";
import { createSaveSlot, DEFAULT_SLOT_ID, type SaveSlotStore } from "./saveSlots";
import { SectorBrowser } from "./SectorBrowser";
import { StateInspector } from "./StateInspector";
import { createTauriFsSaveSlotStore } from "./tauriFsSaveSlotStore";
import { isTauriRuntime } from "./tauriRuntime";
import { TimeControls } from "./TimeControls";
import { cityDetailById } from "./sectorBrowser";
import type { LastStepPerformance } from "./simPerformance";
import type { SimStateSnapshot, StateSummary } from "./simTypes";

const DAYS_PER_YEAR = 360;
const YEAR_STEP_OPTIONS = [1, 5, 10] as const;

export function App() {
  const [handle, setHandle] = useState<SimHandle | null>(null);
  const [summary, setSummary] = useState<StateSummary | null>(null);
  const [snapshot, setSnapshot] = useState<SimStateSnapshot | null>(null);
  const [screen, setScreen] = useState<Screen>("galaxy");
  const [selectedCityId, setSelectedCityId] = useState<number | null>(null);
  const [selectedMemberId, setSelectedMemberId] = useState<number | null>(null);
  const [lastStep, setLastStep] = useState<LastStepPerformance | null>(null);
  const [yearsToAdvance, setYearsToAdvance] = useState<number>(5);
  const debugOverlayVisible = useDebugOverlayVisible();
  const [saveStore, setSaveStore] = useState<SaveSlotStore>(() => createIndexedDbSaveSlotStore());
  const [hasSavedSlot, setHasSavedSlot] = useState(false);
  const [saveStatus, setSaveStatus] = useState<SaveLoadStatus>({ kind: "idle" });

  // Swap in the filesystem-backed store when running inside the Tauri
  // desktop shell (issue #102's Linux packaging spike). The browser
  // target never runs this branch, so it keeps using IndexedDB exactly as
  // before; see `tauriFsSaveSlotStore.ts` for why both implement the same
  // `SaveSlotStore` interface instead of App.tsx branching on save/load
  // calls directly.
  useEffect(() => {
    if (!isTauriRuntime()) return;
    let cancelled = false;
    createTauriFsSaveSlotStore().then((store) => {
      if (!cancelled) setSaveStore(store);
    });
    return () => {
      cancelled = true;
    };
  }, []);

  // Re-read both projections from a handle after any mutating call
  // (stepping time, resolving an event, founding a business, loading a
  // save). Centralized here so every action below stays a one-liner and
  // none of them can forget the snapshot half of the refresh.
  const refreshFrom = useCallback((h: SimHandle) => {
    setSummary(JSON.parse(h.summary_json()) as StateSummary);
    const nextSnapshot = JSON.parse(h.to_json()) as SimStateSnapshot;
    setSnapshot(nextSnapshot);
    return nextSnapshot;
  }, []);

  useEffect(() => {
    let cancelled = false;
    init().then(() => {
      if (cancelled) return;
      const h = new SimHandle(BigInt(42));
      setHandle(h);
      const nextSnapshot = refreshFrom(h);
      setSelectedCityId(
        nextSnapshot.sector.systems[0]?.planets[0]?.countries[0]?.cities[0]?.id ?? null,
      );
    });
    return () => {
      cancelled = true;
    };
  }, [refreshFrom]);

  useEffect(() => {
    let cancelled = false;
    saveStore.get(DEFAULT_SLOT_ID).then((slot) => {
      if (!cancelled && slot) setHasSavedSlot(true);
    });
    return () => {
      cancelled = true;
    };
  }, [saveStore]);

  const advanceOneYear = useCallback(() => {
    if (!handle) return;
    const startedAt = performance.now();
    handle.step_days(DAYS_PER_YEAR);
    setLastStep({ days: DAYS_PER_YEAR, durationMs: performance.now() - startedAt });
    refreshFrom(handle);
  }, [handle, refreshFrom]);

  // A synchronous call is intentional here: the bootstrap slice's data
  // sizes make even a 10-year advance (3600 simulated days) fast enough
  // not to noticeably block the main thread. Revisit with a worker or
  // chunked stepping if `SimState` grows large enough to change that.
  const advanceYears = useCallback(() => {
    if (!handle) return;
    const days = DAYS_PER_YEAR * yearsToAdvance;
    const startedAt = performance.now();
    handle.step_days(days);
    setLastStep({ days, durationMs: performance.now() - startedAt });
    refreshFrom(handle);
  }, [handle, yearsToAdvance, refreshFrom]);

  const onResolveEvent = useCallback(
    (pendingId: number, choiceKey: string) => {
      if (!handle) return;
      handle.resolveEvent(pendingId, choiceKey);
      refreshFrom(handle);
    },
    [handle, refreshFrom],
  );

  const onFoundBusiness = useCallback(
    (name: string, archetype: string, hostCityId: number) => {
      if (!handle) return;
      handle.foundBusiness(name, archetype, hostCityId);
      refreshFrom(handle);
    },
    [handle, refreshFrom],
  );

  const onSave = useCallback(() => {
    if (!handle) return;
    const slot = createSaveSlot(DEFAULT_SLOT_ID, copy.defaultSaveSlotName, handle.to_json());
    saveStore
      .put(slot)
      .then(() => {
        setHasSavedSlot(true);
        setSaveStatus({ kind: "saved", savedAt: slot.savedAt });
      })
      .catch((error: unknown) => {
        setSaveStatus({ kind: "error", message: String(error) });
      });
  }, [handle, saveStore]);

  const onLoad = useCallback(() => {
    saveStore
      .get(DEFAULT_SLOT_ID)
      .then((slot) => {
        if (!slot) return;
        // Go through the same `fromJson`/`to_json` path a fresh load
        // would use so a loaded slot continues the same simulation future
        // as never having saved, see state.rs's round-trip test.
        const nextHandle = SimHandle.fromJson(slot.stateJson);
        handle?.free();
        setHandle(nextHandle);
        const nextSnapshot = refreshFrom(nextHandle);
        setSelectedCityId(
          nextSnapshot.sector.systems[0]?.planets[0]?.countries[0]?.cities[0]?.id ?? null,
        );
        setSaveStatus({ kind: "loaded", savedAt: slot.savedAt });
      })
      .catch((error: unknown) => {
        setSaveStatus({ kind: "error", message: String(error) });
      });
  }, [handle, saveStore, refreshFrom]);

  const onSelectCityFromSearch = useCallback((cityId: number) => {
    setSelectedCityId(cityId);
    setScreen("galaxy");
  }, []);

  const onSelectCharacterFromSearch = useCallback((characterId: number) => {
    setSelectedMemberId(characterId);
    setScreen("dynasty");
  }, []);

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
      <div className="control-strip">
        <TimeControls
          yearsToAdvance={yearsToAdvance}
          yearStepOptions={YEAR_STEP_OPTIONS}
          onYearsToAdvanceChange={setYearsToAdvance}
          onAdvanceOneYear={advanceOneYear}
          onAdvanceYears={advanceYears}
        />
        <SaveControls
          hasSavedSlot={hasSavedSlot}
          status={saveStatus}
          onSave={onSave}
          onLoad={onLoad}
        />
        <GlobalSearch
          sector={snapshot.sector}
          members={summary.dynasty_members}
          onSelectCity={onSelectCityFromSearch}
          onSelectCharacter={onSelectCharacterFromSearch}
        />
      </div>
      {debugOverlayVisible && (
        <>
          <DebugOverlay summary={summary} snapshot={snapshot} lastStep={lastStep} />
          <StateInspector summary={summary} snapshot={snapshot} />
        </>
      )}
      <Nav
        screen={screen}
        onSelectScreen={setScreen}
        pendingEventCount={summary.pending_events.length}
      />
      {screen === "galaxy" && (
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
      )}
      {screen === "dynasty" && (
        <DynastyPanel members={summary.dynasty_members} selectedMemberId={selectedMemberId} />
      )}
      {screen === "economy" && (
        <EconomyView
          sector={snapshot.sector}
          businesses={summary.businesses}
          tradeRoutes={summary.trade_routes}
          careers={summary.careers}
          onFoundBusiness={onFoundBusiness}
        />
      )}
      {screen === "events" && (
        <EventsView pendingEvents={summary.pending_events} onResolve={onResolveEvent} />
      )}
    </main>
  );
}
