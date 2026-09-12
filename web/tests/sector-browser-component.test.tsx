import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it, vi } from "vitest";
import { SectorBrowser } from "../src/SectorBrowser";
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
              backstory: "Built around orbital freight contracts.",
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
  ],
};

function renderBrowser() {
  document.body.innerHTML = renderToStaticMarkup(
    <SectorBrowser sector={sector} selectedCityId={4} onSelectCity={vi.fn()} />,
  );
}

describe("SectorBrowser", () => {
  it("renders selectable city controls with selection state", () => {
    renderBrowser();

    const cityButtons = [...document.querySelectorAll<HTMLButtonElement>(".city-row-select")];

    expect(cityButtons).toHaveLength(1);
    expect(cityButtons[0].textContent).toContain("Meridian");
    expect(cityButtons[0].getAttribute("aria-pressed")).toBe("true");
    expect(cityButtons[0].tabIndex).toBe(0);
    expect(cityButtons[0].hasAttribute("tabindex")).toBe(false);
  });
});
