import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { SaveControls } from "../src/SaveControls";

describe("SaveControls", () => {
  it("disables the load button until a slot has been saved", () => {
    document.body.innerHTML = renderToStaticMarkup(
      <SaveControls
        hasSavedSlot={false}
        status={{ kind: "idle" }}
        onSave={() => undefined}
        onLoad={() => undefined}
      />,
    );

    const buttons = [...document.querySelectorAll<HTMLButtonElement>("button")];
    expect(buttons.map((button) => button.textContent)).toEqual(["Save", "Load"]);
    expect(buttons[1].disabled).toBe(true);
  });

  it("enables the load button once a slot exists", () => {
    document.body.innerHTML = renderToStaticMarkup(
      <SaveControls
        hasSavedSlot={true}
        status={{ kind: "idle" }}
        onSave={() => undefined}
        onLoad={() => undefined}
      />,
    );

    const loadButton = document.querySelectorAll<HTMLButtonElement>("button")[1];
    expect(loadButton.disabled).toBe(false);
  });

  it("shows the saved timestamp after a successful save", () => {
    document.body.innerHTML = renderToStaticMarkup(
      <SaveControls
        hasSavedSlot={true}
        status={{ kind: "saved", savedAt: "2026-01-02T03:04:05.000Z" }}
        onSave={() => undefined}
        onLoad={() => undefined}
      />,
    );

    const status = document.querySelector('[role="status"]');
    expect(status?.textContent).toContain("Saved at");
  });

  it("shows a plain-text error message without an em dash or semicolon", () => {
    document.body.innerHTML = renderToStaticMarkup(
      <SaveControls
        hasSavedSlot={false}
        status={{ kind: "error", message: "database unavailable" }}
        onSave={() => undefined}
        onLoad={() => undefined}
      />,
    );

    const status = document.querySelector('[role="status"]');
    expect(status?.textContent).toBe("Save failed. database unavailable");
  });
});
