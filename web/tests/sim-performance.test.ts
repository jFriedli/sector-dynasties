import { describe, expect, it } from "vitest";
import { entityCountsFromState, formatDurationMs } from "../src/simPerformance";
import type { SimStateSnapshot, StateSummary } from "../src/simTypes";

const summary: StateSummary = {
  tick: 720,
  year: 2,
  seed: 7,
  system_count: 2,
  city_count: 3,
  total_population: 3000,
  dynasty_name: "House Test",
  dynasty_wealth: 10,
  dynasty_members: [
    { id: 1, name: "A", age_years: 40, alive: true, role: "Head" },
    { id: 2, name: "B", age_years: 18, alive: true, role: "Member" },
  ],
  rng_domains: [],
  pending_events: [],
  businesses: [],
  trade_routes: [],
  careers: [],
};

const snapshot: SimStateSnapshot = {
  seed: 7,
  sector: {
    seed: 7,
    name: "Test Sector",
    systems: [
      {
        id: 1,
        name: "A",
        planets: [
          {
            id: 2,
            name: "A Prime",
            resource_tags: ["MetalRich"],
            countries: [
              {
                id: 3,
                name: "Compact",
                backstory: "Compact grew around an early shipyard.",
                social_mobility: 0.5,
                union_power: 0.5,
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
                    name: "One",
                    specialization: "Mining",
                    population: { size: 1000, average_wealth: 1, unemployment_rate: 0.1 },
                    treasury: 1,
                    recent_output_index: 1,
                  },
                  {
                    id: 5,
                    name: "Two",
                    specialization: "Logistics",
                    population: { size: 2000, average_wealth: 2, unemployment_rate: 0.2 },
                    treasury: 2,
                    recent_output_index: 1,
                  },
                ],
              },
            ],
          },
        ],
      },
      {
        id: 6,
        name: "B",
        planets: [
          {
            id: 7,
            name: "B Prime",
            resource_tags: ["Arid"],
            countries: [
              {
                id: 8,
                name: "League",
                backstory: "League grew around a transit compact.",
                social_mobility: 0.7,
                union_power: 0.3,
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
                    id: 9,
                    name: "Three",
                    specialization: "Research",
                    population: { size: 3000, average_wealth: 3, unemployment_rate: 0.3 },
                    treasury: 3,
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

describe("simulation performance helpers", () => {
  it("counts world and dynasty entities for the debug overlay", () => {
    expect(entityCountsFromState(snapshot, summary)).toEqual({
      systems: 2,
      planets: 2,
      countries: 2,
      cities: 3,
      dynastyMembers: 2,
      total: 11,
    });
  });

  it("formats step duration with useful precision", () => {
    expect(formatDurationMs(2.345)).toBe("2.35 ms");
    expect(formatDurationMs(42.34)).toBe("42.3 ms");
    expect(formatDurationMs(142.34)).toBe("142 ms");
    expect(formatDurationMs(Number.NaN)).toBe("n/a");
  });
});
