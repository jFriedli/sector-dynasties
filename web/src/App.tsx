import { useCallback, useEffect, useState } from "react";
import init, { SimHandle } from "./wasm/sim_wasm.js";
import { copy } from "./content/copy";
import type { StateSummary } from "./simTypes";

export function App() {
  const [handle, setHandle] = useState<SimHandle | null>(null);
  const [summary, setSummary] = useState<StateSummary | null>(null);

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
    handle.step_days(360);
    setSummary(JSON.parse(handle.summary_json()) as StateSummary);
  }, [handle]);

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
    </main>
  );
}
