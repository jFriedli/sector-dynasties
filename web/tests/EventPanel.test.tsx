import { act } from "react";
import { createRoot } from "react-dom/client";
import { describe, expect, it } from "vitest";
import { EventPanel } from "../src/EventPanel";
import type { StateSummary } from "../src/simTypes";

describe("EventPanel", () => {
  it("resolves a pending event through the UI and reflects the effect in summary", async () => {
    let nextSummary: StateSummary = {
      tick: 360,
      year: 1,
      seed: 2026,
      system_count: 3,
      city_count: 6,
      total_population: 1000,
      dynasty_name: "House Meridian",
      dynasty_wealth: 500,
      dynasty_members: [{ id: 1, name: "Founder", age_years: 34, alive: true, role: "Head" }],
      pending_events: [
        {
          id: 1,
          key: "family_seed_money",
          title: "A relative offers seed money",
          character_id: 1,
          character_name: "Founder",
          raised_year: 1,
          choices: [
            { key: "accept", label: "Accept the money" },
            { key: "decline", label: "Politely decline" },
          ],
        },
      ],
      rng_domains: [],
    };
    const before = nextSummary;
    const event = before.pending_events[0];

    const host = document.createElement("div");
    document.body.append(host);
    const root = createRoot(host);

    await act(async () => {
      root.render(
        <EventPanel
          event={event}
          onResolve={(eventId, choiceKey) => {
            if (eventId === event.id && choiceKey === "accept") {
              nextSummary = {
                ...nextSummary,
                dynasty_wealth: nextSummary.dynasty_wealth + 1000,
                pending_events: [],
              };
            }
          }}
        />,
      );
    });

    const acceptButton = [...host.querySelectorAll<HTMLButtonElement>("button")].find(
      (button) => button.textContent === "Accept the money",
    );
    expect(acceptButton).toBeDefined();

    await act(async () => {
      acceptButton?.click();
    });

    const after = nextSummary;
    expect(after.dynasty_wealth).toBeGreaterThan(before.dynasty_wealth);
    expect(after.pending_events.some((pending) => pending.id === event.id)).toBe(false);

    root.unmount();
    host.remove();
  });
});
