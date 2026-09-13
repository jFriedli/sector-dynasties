import { act } from "react-dom/test-utils";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it } from "vitest";
import { StateInspector } from "../src/StateInspector";
import type { SimStateSnapshot, StateSummary } from "../src/simTypes";

// Render smoke test for the state inspector dev panel (issue #93), in the
// same style as DebugOverlay.test.tsx: no React Testing Library dependency
// yet (see docs/TESTING.md), so this drives the real search/filter inputs
// with react-dom/test-utils. The bulk of behavior is covered by the pure
// logic tests in state-inspector.test.ts; this only proves the component
// wires that logic to its inputs and renders the result.

const SUMMARY: StateSummary = {
  tick: 42,
  year: 1,
  seed: 2026,
  system_count: 1,
  city_count: 1,
  total_population: 1000,
  dynasty_name: "House Meridian",
  dynasty_wealth: 500,
  dynasty_members: [{ id: 1, name: "Founder Meridian", age_years: 34, alive: true, role: "Head" }],
  rng_domains: [{ domain: "economy", fingerprint: "deadbeefcafef00d" }],
  pending_events: [],
  businesses: [],
  trade_routes: [],
  careers: [],
};

const SNAPSHOT: SimStateSnapshot = {
  seed: 2026,
  sector: {
    seed: 2026,
    name: "Test Sector",
    systems: [
      {
        id: 1,
        name: "Aster",
        planets: [
          {
            id: 2,
            name: "Aster Prime",
            resource_tags: ["MetalRich"],
            countries: [
              {
                id: 3,
                name: "North Compact",
                backstory: "North Compact grew around orbital freight contracts.",
                social_mobility: 0.45,
                union_power: 0.4,
                government: {
                  federalism: 0.5,
                  franchise: 0.5,
                  economic_liberalism: 0.5,
                  press_freedom: 0.5,
                  legislative_strength: 0.5,
                  judicial_independence: 0.5,
                },
                cities: [
                  {
                    id: 4,
                    name: "Meridian",
                    specialization: "Finance",
                    population: { size: 1000, average_wealth: 12.5, unemployment_rate: 0.04 },
                    treasury: 850.25,
                    recent_output_index: 1,
                  },
                ],
              },
            ],
          },
        ],
      },
    ],
  },
};

function setInputValue(input: HTMLInputElement, value: string) {
  const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, "value")!.set!;
  setter.call(input, value);
  input.dispatchEvent(new Event("input", { bubbles: true }));
}

function setSelectValue(select: HTMLSelectElement, value: string) {
  const setter = Object.getOwnPropertyDescriptor(window.HTMLSelectElement.prototype, "value")!.set!;
  setter.call(select, value);
  select.dispatchEvent(new Event("change", { bubbles: true }));
}

describe("state inspector search and filter", () => {
  let container: HTMLDivElement | null = null;
  let root: Root | null = null;

  afterEach(() => {
    act(() => {
      root?.unmount();
    });
    container?.remove();
    container = null;
    root = null;
  });

  it("lists every entity by default and narrows results as the developer types", () => {
    container = document.createElement("div");
    document.body.appendChild(container);
    root = createRoot(container);

    act(() => {
      root!.render(<StateInspector summary={SUMMARY} snapshot={SNAPSHOT} />);
    });

    const results = container.querySelector('[data-testid="state-inspector-results"]')!;
    // 1 system + 1 planet + 1 country + 1 city + 1 dynasty member + 1 rng domain
    expect(results.textContent).toContain("Aster");
    expect(results.textContent).toContain("Meridian");
    expect(results.textContent).toContain("Founder Meridian");
    expect(results.textContent).toContain("economy");

    const search = container.querySelector<HTMLInputElement>(
      '[data-testid="state-inspector-search"]',
    )!;
    act(() => {
      setInputValue(search, "founder");
    });

    const narrowed = container.querySelector('[data-testid="state-inspector-results"]')!;
    expect(narrowed.textContent).toContain("Founder Meridian");
    expect(narrowed.textContent).not.toContain("Aster");
    expect(narrowed.textContent).not.toContain("economy");
  });

  it("filters by entity type independently of the search text", () => {
    container = document.createElement("div");
    document.body.appendChild(container);
    root = createRoot(container);

    act(() => {
      root!.render(<StateInspector summary={SUMMARY} snapshot={SNAPSHOT} />);
    });

    const typeFilter = container.querySelector<HTMLSelectElement>(
      '[data-testid="state-inspector-type-filter"]',
    )!;
    act(() => {
      setSelectValue(typeFilter, "city");
    });

    const results = container.querySelector('[data-testid="state-inspector-results"]')!;
    // Only the one city row should remain; its breadcrumb legitimately
    // mentions "Aster Prime" as ancestry context, so assert on row count
    // and the standalone planet/dynasty-member rows disappearing instead.
    expect(results.querySelectorAll("li")).toHaveLength(1);
    expect(results.textContent).toContain("Meridian");
    expect(results.textContent).not.toContain("Founder Meridian");
  });

  it("shows a no-match message instead of an empty list", () => {
    container = document.createElement("div");
    document.body.appendChild(container);
    root = createRoot(container);

    act(() => {
      root!.render(<StateInspector summary={SUMMARY} snapshot={SNAPSHOT} />);
    });

    const search = container.querySelector<HTMLInputElement>(
      '[data-testid="state-inspector-search"]',
    )!;
    act(() => {
      setInputValue(search, "does-not-exist-anywhere");
    });

    const results = container.querySelector('[data-testid="state-inspector-results"]')!;
    expect(results.textContent).toContain("no matching entities");
  });
});
