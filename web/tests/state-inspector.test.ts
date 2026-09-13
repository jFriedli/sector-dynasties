import { describe, expect, it } from "vitest";
import {
  ENTITY_TYPES,
  entityRowsFromState,
  entityTypeLabel,
  filterEntityRows,
} from "../src/stateInspector";
import type { SimStateSnapshot, StateSummary } from "../src/simTypes";

const SUMMARY: StateSummary = {
  tick: 42,
  year: 1,
  seed: 2026,
  system_count: 2,
  city_count: 2,
  total_population: 4600,
  dynasty_name: "House Meridian",
  dynasty_wealth: 500,
  dynasty_members: [
    { id: 1, name: "Founder Meridian", age_years: 54, alive: true, role: "Head" },
    { id: 2, name: "Junior Meridian", age_years: 20, alive: false, role: "Member" },
  ],
  rng_domains: [
    { domain: "economy", fingerprint: "deadbeefcafef00d" },
    { domain: "worldgen", fingerprint: "0badf00d12345678" },
  ],
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
                    population: { size: 1200, average_wealth: 12.5, unemployment_rate: 0.04 },
                    treasury: 850.25,
                    recent_output_index: 1,
                  },
                ],
              },
            ],
          },
        ],
      },
      {
        id: 5,
        name: "Boreal",
        planets: [
          {
            id: 6,
            name: "Boreal Station",
            resource_tags: ["Arid"],
            countries: [
              {
                id: 7,
                name: "Dock League",
                backstory: "Dock League formed from several unified settlements.",
                social_mobility: 0.6,
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
                    id: 8,
                    name: "Port Ember",
                    specialization: "Logistics",
                    population: { size: 3400, average_wealth: 7.1, unemployment_rate: 0.08 },
                    treasury: 420.5,
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

describe("entityRowsFromState", () => {
  it("flattens the full hierarchy plus dynasty members and RNG domains", () => {
    const rows = entityRowsFromState(SNAPSHOT, SUMMARY);

    // 2 systems + 2 planets + 2 countries + 2 cities + 2 dynasty members + 2 rng domains
    expect(rows).toHaveLength(12);
    expect(rows.map((r) => r.type)).toEqual([
      "system",
      "planet",
      "country",
      "city",
      "system",
      "planet",
      "country",
      "city",
      "dynasty_member",
      "dynasty_member",
      "rng_domain",
      "rng_domain",
    ]);
  });

  it("gives each row a hierarchy breadcrumb through its path", () => {
    const rows = entityRowsFromState(SNAPSHOT, SUMMARY);
    const city = rows.find((r) => r.type === "city" && r.name === "Meridian");
    expect(city?.path).toBe("Aster > Aster Prime > North Compact");
    const country = rows.find((r) => r.type === "country" && r.name === "North Compact");
    expect(country?.path).toBe("Aster > Aster Prime");
    const system = rows.find((r) => r.type === "system" && r.name === "Aster");
    expect(system?.path).toBe("");
  });

  it("gives dynasty member rows a role/age/alive detail string", () => {
    const rows = entityRowsFromState(SNAPSHOT, SUMMARY);
    const founder = rows.find((r) => r.type === "dynasty_member" && r.name === "Founder Meridian");
    expect(founder?.detail).toBe("Head, age 54");
    const junior = rows.find((r) => r.type === "dynasty_member" && r.name === "Junior Meridian");
    expect(junior?.detail).toBe("Member, age 20 (deceased)");
  });

  it("keys rows uniquely across entity types even when raw ids collide", () => {
    // System id 1 and dynasty member id 1 must not collide in `key`.
    const rows = entityRowsFromState(SNAPSHOT, SUMMARY);
    const keys = rows.map((r) => r.key);
    expect(new Set(keys).size).toBe(keys.length);
  });
});

describe("filterEntityRows", () => {
  const rows = entityRowsFromState(SNAPSHOT, SUMMARY);

  it("matches by case-insensitive name substring", () => {
    const filtered = filterEntityRows(rows, "merid", null);
    expect(filtered.map((r) => r.name).sort()).toEqual([
      "Founder Meridian",
      "Junior Meridian",
      "Meridian",
    ]);
  });

  it("matches by entity type alone when the query is empty", () => {
    const filtered = filterEntityRows(rows, "", "city");
    expect(filtered).toHaveLength(2);
    expect(filtered.every((r) => r.type === "city")).toBe(true);
  });

  it("combines a name query and a type filter", () => {
    const filtered = filterEntityRows(rows, "port", "city");
    expect(filtered).toHaveLength(1);
    expect(filtered[0].name).toBe("Port Ember");
  });

  it("returns everything when the query and type filter are both empty", () => {
    expect(filterEntityRows(rows, "", null)).toHaveLength(rows.length);
  });

  it("returns nothing for a query that matches no entity", () => {
    expect(filterEntityRows(rows, "does-not-exist", null)).toEqual([]);
  });

  it("trims surrounding whitespace from the query", () => {
    const filtered = filterEntityRows(rows, "  boreal  ", null);
    expect(filtered.map((r) => r.name)).toEqual(["Boreal", "Boreal Station"]);
  });
});

describe("entityTypeLabel", () => {
  it("has a distinct human-readable label for every entity type", () => {
    const labels = ENTITY_TYPES.map(entityTypeLabel);
    expect(new Set(labels).size).toBe(ENTITY_TYPES.length);
    expect(labels).toContain("Dynasty member");
    expect(labels).toContain("RNG domain");
  });
});
