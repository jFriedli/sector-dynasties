import { describe, expect, it } from "vitest";
import { cityRowsFromSector, totalTreasury } from "../src/sectorBrowser";
import type { Sector } from "../src/simTypes";

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
          countries: [
            {
              id: 3,
              name: "North Compact",
              cities: [
                {
                  id: 4,
                  name: "Meridian",
                  specialization: "Finance",
                  population: {
                    size: 1200,
                    average_wealth: 12.5,
                    unemployment_rate: 0.04,
                  },
                  treasury: 850.25,
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
          countries: [
            {
              id: 7,
              name: "Dock League",
              cities: [
                {
                  id: 8,
                  name: "Port Ember",
                  specialization: "Logistics",
                  population: {
                    size: 3400,
                    average_wealth: 7.1,
                    unemployment_rate: 0.08,
                  },
                  treasury: 420.5,
                },
              ],
            },
          ],
        },
      ],
    },
  ],
};

describe("sector browser data shaping", () => {
  it("flattens city rows with their full hierarchy context", () => {
    expect(cityRowsFromSector(sector)).toEqual([
      {
        id: 4,
        name: "Meridian",
        systemName: "Aster",
        planetName: "Aster Prime",
        countryName: "North Compact",
        specialization: "Finance",
        population: 1200,
        treasury: 850.25,
      },
      {
        id: 8,
        name: "Port Ember",
        systemName: "Boreal",
        planetName: "Boreal Station",
        countryName: "Dock League",
        specialization: "Logistics",
        population: 3400,
        treasury: 420.5,
      },
    ]);
  });

  it("aggregates displayed city treasury totals", () => {
    expect(totalTreasury(cityRowsFromSector(sector))).toBe(1270.75);
  });
});
