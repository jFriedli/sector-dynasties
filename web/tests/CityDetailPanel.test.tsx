import { act } from "react-dom/test-utils";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { CityDetailPanel } from "../src/CityDetailPanel";
import type { CityDetail } from "../src/sectorBrowserLogic";
import type { GovernmentComponent, LobbyingOutcome, PolicyDirection } from "../src/simTypes";

const CITY: CityDetail = {
  id: 4,
  name: "Meridian",
  systemName: "Aster",
  planetName: "Aster Prime",
  countryId: 3,
  countryName: "North Compact",
  specialization: "Finance",
  population: { size: 1200, average_wealth: 12.5, unemployment_rate: 0.04 },
  treasury: 850.25,
  government: {
    federalism: 0.6,
    franchise: 0.7,
    economic_liberalism: 0.4,
    press_freedom: 0.5,
    legislative_strength: 0.3,
    judicial_independence: 0.35,
  },
};

function setSelectValue(select: HTMLSelectElement, value: string) {
  const setter = Object.getOwnPropertyDescriptor(window.HTMLSelectElement.prototype, "value")!.set!;
  setter.call(select, value);
  select.dispatchEvent(new Event("change", { bubbles: true }));
}

describe("CityDetailPanel", () => {
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

  it("shows the empty state when no city is selected", () => {
    container = document.createElement("div");
    document.body.appendChild(container);
    root = createRoot(container);

    act(() => {
      root!.render(<CityDetailPanel city={null} lobbyingCost={750} onLobby={vi.fn()} />);
    });

    expect(container.textContent).toContain("No city selected");
  });

  it("shows the host country's government stats and the lobbying cost", () => {
    container = document.createElement("div");
    document.body.appendChild(container);
    root = createRoot(container);

    act(() => {
      root!.render(<CityDetailPanel city={CITY} lobbyingCost={750} onLobby={vi.fn()} />);
    });

    expect(container.textContent).toContain("Government");
    expect(container.textContent).toContain("60.0%");
    expect(container.textContent).toContain("750.00");
  });

  it("lobbies for the selected component and direction, then shows the outcome", async () => {
    container = document.createElement("div");
    document.body.appendChild(container);
    const outcome: LobbyingOutcome = {
      country_id: 3,
      component: "Franchise",
      direction: "Increase",
      wealth_spent: 750,
      before: 0.7,
      after: 0.75,
    };
    const onLobby = vi
      .fn<
        (
          countryId: number,
          component: GovernmentComponent,
          direction: PolicyDirection,
        ) => Promise<LobbyingOutcome>
      >()
      .mockResolvedValue(outcome);
    root = createRoot(container);

    act(() => {
      root!.render(<CityDetailPanel city={CITY} lobbyingCost={750} onLobby={onLobby} />);
    });

    const selects = container.querySelectorAll("select");
    act(() => {
      setSelectValue(selects[0] as HTMLSelectElement, "franchise");
      setSelectValue(selects[1] as HTMLSelectElement, "increase");
    });

    const form = container.querySelector("form")!;
    await act(async () => {
      form.dispatchEvent(new Event("submit", { bubbles: true, cancelable: true }));
    });

    expect(onLobby).toHaveBeenCalledWith(3, "franchise", "increase");
    expect(container.textContent).toContain("Franchise moved from 70.0% to 75.0%.");
  });

  it("shows an error message when lobbying fails", async () => {
    container = document.createElement("div");
    document.body.appendChild(container);
    const onLobby = vi.fn().mockRejectedValue(new Error("InsufficientWealth"));
    root = createRoot(container);

    act(() => {
      root!.render(<CityDetailPanel city={CITY} lobbyingCost={750} onLobby={onLobby} />);
    });

    const form = container.querySelector("form")!;
    await act(async () => {
      form.dispatchEvent(new Event("submit", { bubbles: true, cancelable: true }));
    });

    expect(container.textContent).toContain("Lobbying failed.");
  });
});
