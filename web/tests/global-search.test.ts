import { describe, expect, it } from "vitest";
import { searchGlobally } from "../src/globalSearch";
import type { DynastyMemberSummary, Sector } from "../src/simTypes";

const members: DynastyMemberSummary[] = [
  { id: 1, name: "Meridia Voss", age_years: 52, alive: true, role: "Head" },
  { id: 2, name: "Corvin Voss", age_years: 24, alive: true, role: "Member" },
  { id: 3, name: "Meridian Ash", age_years: 80, alive: false, role: "Member" },
];

const sector: Sector = {
  seed: 42,
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
              backstory: "Built around orbital freight contracts.",
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
                  name: "Meridian",
                  specialization: "Finance",
                  population: { size: 1200, average_wealth: 12.5, unemployment_rate: 0.04 },
                  treasury: 850.25,
                  recent_output_index: 1,
                },
                {
                  id: 5,
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
};

describe("global search matching", () => {
  it("returns no results for a blank or whitespace-only query", () => {
    expect(searchGlobally(sector, members, "")).toEqual([]);
    expect(searchGlobally(sector, members, "   ")).toEqual([]);
  });

  it("matches characters by a case-insensitive substring of their name", () => {
    const results = searchGlobally(sector, members, "voss");

    expect(results).toEqual([
      { kind: "character", id: 1, name: "Meridia Voss", role: "Head", alive: true },
      { kind: "character", id: 2, name: "Corvin Voss", role: "Member", alive: true },
    ]);
  });

  it("matches cities by a case-insensitive substring of their name", () => {
    const results = searchGlobally(sector, members, "ember");

    expect(results).toEqual([
      {
        kind: "city",
        id: 5,
        name: "Port Ember",
        planetName: "Aster Prime",
        countryName: "North Compact",
      },
    ]);
  });

  it("matches both characters and cities in one query", () => {
    const results = searchGlobally(sector, members, "meridi");

    expect(results.map((result) => `${result.kind}:${result.name}`)).toEqual([
      "character:Meridia Voss",
      "character:Meridian Ash",
      "city:Meridian",
    ]);
  });

  it("ranks an exact name match ahead of a mere prefix match", () => {
    const results = searchGlobally(sector, members, "meridian");

    expect(results.map((result) => result.name)).toEqual(["Meridian", "Meridian Ash"]);
  });

  it("returns an empty list when nothing matches", () => {
    expect(searchGlobally(sector, members, "nonexistent")).toEqual([]);
  });

  it("does not mutate the members it searches", () => {
    const original = [...members];
    searchGlobally(sector, members, "voss");
    expect(members).toEqual(original);
  });
});
