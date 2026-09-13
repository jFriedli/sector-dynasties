import { describe, expect, it } from "vitest";
import { BUSINESS_ARCHETYPES, citiesEligibleForArchetype } from "../src/economyViewLogic";
import type { CitySpecialization, Sector } from "../src/simTypes";

function cityFixture(id: number, specialization: CitySpecialization, name: string) {
  return {
    id,
    name,
    specialization,
    population: { size: 1000, average_wealth: 10, unemployment_rate: 0.1 },
    treasury: 0,
    recent_output_index: 1,
  };
}

const SECTOR: Sector = {
  seed: 1,
  name: "Test Sector",
  systems: [
    {
      id: 1,
      name: "Aster",
      planets: [
        {
          id: 2,
          name: "Aster Prime",
          resource_tags: [],
          countries: [
            {
              id: 3,
              name: "North Compact",
              backstory: "",
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
                cityFixture(10, "Mining", "Ferrous Hold"),
                cityFixture(11, "Finance", "Meridian"),
                cityFixture(12, "Mining", "Deep Vein"),
              ],
            },
          ],
        },
      ],
    },
  ],
};

describe("citiesEligibleForArchetype", () => {
  it("only returns cities whose specialization matches the archetype's requirement", () => {
    const eligible = citiesEligibleForArchetype(SECTOR, "Mining");
    expect(eligible.map((city) => city.name).sort()).toEqual(["Deep Vein", "Ferrous Hold"]);
  });

  it("returns an empty list for an unknown archetype rather than throwing", () => {
    expect(citiesEligibleForArchetype(SECTOR, "NotARealArchetype")).toEqual([]);
  });

  it("lists at least one known archetype", () => {
    expect(BUSINESS_ARCHETYPES.length).toBeGreaterThan(0);
  });
});
