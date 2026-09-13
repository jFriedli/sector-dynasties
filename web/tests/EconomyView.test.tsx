import { act } from "react-dom/test-utils";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { EconomyView } from "../src/EconomyView";
import type { BusinessView, CareerView, Sector, TradeRouteView } from "../src/simTypes";

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
                {
                  id: 10,
                  name: "Ferrous Hold",
                  specialization: "Mining",
                  population: { size: 5000, average_wealth: 10, unemployment_rate: 0.1 },
                  treasury: 0,
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

const BUSINESSES: BusinessView[] = [
  {
    id: 1,
    name: "Ferrous Extraction Co.",
    archetype: "Mining",
    host_city_id: 10,
    host_city_name: "Ferrous Hold",
    equity: 42.5,
  },
];

const TRADE_ROUTES: TradeRouteView[] = [
  {
    id: 1,
    city_a_id: 10,
    city_a_name: "Ferrous Hold",
    city_b_id: 11,
    city_b_name: "Meridian",
    capacity: 100,
    distance: 3,
    cost: 20,
    reliability: 0.9,
  },
];

const CAREERS: CareerView[] = [
  {
    id: 1,
    character_id: 1,
    character_name: "Founder Meridian",
    track: "Corporate",
    job_title: "Analyst",
    employer_city_id: 10,
    employer_city_name: "Ferrous Hold",
  },
];

// Same helpers as StateInspector.test.tsx: React's controlled inputs
// override the native value setter, so a plain `.value = ...` assignment
// is silently ignored without going through the prototype's real setter.
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

type FoundBusinessFn = (
  name: string,
  archetype: string,
  hostCityId: number,
) => void | Promise<void>;

function renderInto(
  container: HTMLDivElement,
  onFoundBusiness: FoundBusinessFn,
  selectedBusinessId: number | null = null,
) {
  const root = createRoot(container);
  act(() => {
    root.render(
      <EconomyView
        sector={SECTOR}
        businesses={BUSINESSES}
        tradeRoutes={TRADE_ROUTES}
        careers={CAREERS}
        selectedBusinessId={selectedBusinessId}
        onFoundBusiness={onFoundBusiness}
      />,
    );
  });
  return root;
}

describe("EconomyView", () => {
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

  it("lists existing businesses, trade routes, and careers", () => {
    container = document.createElement("div");
    document.body.appendChild(container);
    root = renderInto(container, vi.fn());

    expect(container.textContent).toContain("Ferrous Extraction Co.");
    expect(container.textContent).toContain("Ferrous Hold");
    expect(container.textContent).toContain("Meridian");
    expect(container.textContent).toContain("Founder Meridian");
    expect(container.textContent).toContain("Analyst");
  });

  it("highlights the selected business row", () => {
    container = document.createElement("div");
    document.body.appendChild(container);
    root = renderInto(container, vi.fn(), 1);

    const selectedRow = container.querySelector("#business-1");
    expect(selectedRow?.className).toContain("business-row--selected");
  });

  it("founds a business with the form's chosen name and city", async () => {
    container = document.createElement("div");
    document.body.appendChild(container);
    const onFoundBusiness = vi.fn().mockResolvedValue(undefined);
    root = renderInto(container, onFoundBusiness);

    const openButton = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent === "Found a business",
    )!;
    act(() => {
      openButton.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });

    const nameInput = container.querySelector("input[type=text]") as HTMLInputElement;
    act(() => {
      setInputValue(nameInput, "Deep Vein Consolidated");
    });

    const citySelect = container.querySelectorAll("select")[1] as HTMLSelectElement;
    act(() => {
      setSelectValue(citySelect, "10");
    });

    const form = container.querySelector("form")!;
    await act(async () => {
      form.dispatchEvent(new Event("submit", { bubbles: true, cancelable: true }));
    });

    expect(onFoundBusiness).toHaveBeenCalledWith("Deep Vein Consolidated", "Mining", 10);
  });
});
