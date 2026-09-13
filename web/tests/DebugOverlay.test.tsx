import { act } from "react-dom/test-utils";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it } from "vitest";
import { DebugOverlay, useDebugOverlayVisible } from "../src/DebugOverlay";
import type { SimStateSnapshot, StateSummary } from "../src/simTypes";

// No React Testing Library dependency in this project yet (see
// docs/TESTING.md); react-dom/client and react-dom/test-utils are already
// dependencies of the app itself, so this drives the real toggle behavior
// (issue #35's acceptance criterion: hidden by default, toggleable) without
// adding a new dependency for one small test.

const SUMMARY: StateSummary = {
  tick: 42,
  year: 1,
  seed: 2026,
  system_count: 3,
  city_count: 5,
  total_population: 1000,
  dynasty_name: "House Meridian",
  dynasty_wealth: 500,
  dynasty_members: [{ id: 1, name: "Founder", age_years: 34, alive: true, role: "Head" }],
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
                cities: [
                  {
                    id: 4,
                    name: "Meridian",
                    specialization: "Finance",
                    population: {
                      size: 1000,
                      average_wealth: 12.5,
                      unemployment_rate: 0.04,
                    },
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

function Harness({ summary }: { summary: StateSummary }) {
  const visible = useDebugOverlayVisible();
  return visible ? (
    <DebugOverlay
      summary={summary}
      snapshot={SNAPSHOT}
      lastStep={{ days: 360, durationMs: 2.34 }}
    />
  ) : null;
}

function pressBacktick() {
  window.dispatchEvent(new KeyboardEvent("keydown", { key: "`" }));
}

describe("debug overlay toggle", () => {
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

  it("is hidden by default and shows tick, seed, and RNG domains once toggled", () => {
    container = document.createElement("div");
    document.body.appendChild(container);
    root = createRoot(container);

    act(() => {
      root!.render(<Harness summary={SUMMARY} />);
    });
    expect(container.querySelector('[data-testid="debug-overlay"]')).toBeNull();

    act(() => {
      pressBacktick();
    });
    const overlay = container.querySelector('[data-testid="debug-overlay"]');
    expect(overlay).not.toBeNull();
    expect(overlay!.textContent).toContain("tick: 42");
    expect(overlay!.textContent).toContain("seed: 2026");
    expect(overlay!.textContent).toContain("last step_days: 360 days in 2.34 ms");
    expect(overlay!.textContent).toContain("total entities: 5");
    expect(overlay!.textContent).toContain("economy: deadbeefcafef00d");

    act(() => {
      pressBacktick();
    });
    expect(container.querySelector('[data-testid="debug-overlay"]')).toBeNull();
  });
});
