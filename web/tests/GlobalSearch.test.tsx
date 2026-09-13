import { act } from "react-dom/test-utils";
import { createRoot, type Root } from "react-dom/client";
import { renderToStaticMarkup } from "react-dom/server";
import { afterEach, describe, expect, it, vi } from "vitest";
import { GlobalSearch } from "../src/GlobalSearch";
import type { BusinessView, DynastyMemberSummary, Sector } from "../src/simTypes";

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
  ],
};

const businesses: BusinessView[] = [
  {
    id: 9,
    name: "Meridian Freight Trust",
    archetype: "Logistics",
    host_city_id: 4,
    host_city_name: "Meridian",
    equity: 1000,
  },
];

function setInputValue(input: HTMLInputElement, value: string) {
  const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, "value")!.set!;
  setter.call(input, value);
  input.dispatchEvent(new Event("input", { bubbles: true }));
}

function renderSearch() {
  document.body.innerHTML = renderToStaticMarkup(
    <GlobalSearch
      sector={sector}
      members={members}
      businesses={businesses}
      onSelectCity={vi.fn()}
      onSelectCharacter={vi.fn()}
      onSelectBusiness={vi.fn()}
    />,
  );
}

describe("GlobalSearch", () => {
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

  it("selects a matching business result", () => {
    container = document.createElement("div");
    document.body.appendChild(container);
    const onSelectBusiness = vi.fn();
    root = createRoot(container);

    act(() => {
      root!.render(
        <GlobalSearch
          sector={sector}
          members={members}
          businesses={businesses}
          onSelectCity={vi.fn()}
          onSelectCharacter={vi.fn()}
          onSelectBusiness={onSelectBusiness}
        />,
      );
    });

    const input = container.querySelector("input") as HTMLInputElement;
    act(() => {
      setInputValue(input, "freight");
    });

    const result = container.querySelector(".global-search-result") as HTMLButtonElement;
    expect(result.textContent).toContain("Meridian Freight Trust");
    expect(result.textContent).toContain("Business");

    act(() => {
      result.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });

    expect(onSelectBusiness).toHaveBeenCalledWith(9);
    expect(input.value).toBe("");
  });
});
