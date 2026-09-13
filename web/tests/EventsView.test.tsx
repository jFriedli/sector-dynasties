import { act } from "react-dom/test-utils";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { EventsView } from "../src/EventsView";
import type { PendingEventView } from "../src/simTypes";

// No React Testing Library dependency in this project yet (see
// docs/TESTING.md), same pattern as DebugOverlay.test.tsx.

const PENDING: PendingEventView[] = [
  {
    id: 5,
    key: "family_seed_money",
    title: "An unexpected gift",
    character_id: 1,
    character_name: "Founder Meridian",
    raised_year: 3,
    choices: [
      { key: "accept", label: "Accept the gift" },
      { key: "decline", label: "Decline it" },
    ],
  },
];

describe("EventsView", () => {
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

  it("shows the empty state when there are no pending events", () => {
    container = document.createElement("div");
    document.body.appendChild(container);
    root = createRoot(container);

    act(() => {
      root!.render(<EventsView pendingEvents={[]} onResolve={vi.fn()} />);
    });

    expect(container.textContent).toContain("No decisions await the dynasty right now.");
  });

  it("renders a pending event's title, character, and choices, and resolves the clicked one", () => {
    container = document.createElement("div");
    document.body.appendChild(container);
    root = createRoot(container);
    const onResolve = vi.fn();

    act(() => {
      root!.render(<EventsView pendingEvents={PENDING} onResolve={onResolve} />);
    });

    expect(container.textContent).toContain("An unexpected gift");
    expect(container.textContent).toContain("Founder Meridian");

    const buttons = Array.from(container.querySelectorAll("button"));
    const acceptButton = buttons.find((button) => button.textContent === "Accept the gift");
    expect(acceptButton).toBeDefined();

    act(() => {
      acceptButton!.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });

    expect(onResolve).toHaveBeenCalledWith(5, "accept");
  });
});
