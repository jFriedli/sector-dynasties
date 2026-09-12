import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it, vi } from "vitest";
import { GlobalSearch } from "../src/GlobalSearch";
import type { DynastyMemberSummary, Sector } from "../src/simTypes";

const members: DynastyMemberSummary[] = [
  { id: 1, name: "Meridia Voss", age_years: 52, alive: true, role: "Head" },
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
  ],
};

function renderSearch() {
  document.body.innerHTML = renderToStaticMarkup(
    <GlobalSearch
      sector={sector}
      members={members}
      onSelectCity={vi.fn()}
      onSelectCharacter={vi.fn()}
    />,
  );
}

describe("GlobalSearch", () => {
  it("associates the search input with its visible label", () => {
    renderSearch();

    const input = document.querySelector("input");
    const label = document.querySelector("label");

    expect(input?.id).toBe("global-search-input");
    expect(input?.getAttribute("type")).toBe("search");
    expect(label?.htmlFor).toBe(input?.id);
  });

  it("shows no results list before the player has typed anything", () => {
    renderSearch();

    expect(document.querySelector(".global-search-results")).toBeNull();
    expect(document.querySelector(".global-search-empty")).toBeNull();
  });
});
