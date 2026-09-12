import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { TimeControls } from "../src/TimeControls";

function renderControls() {
  document.body.innerHTML = renderToStaticMarkup(
    <TimeControls
      yearsToAdvance={5}
      yearStepOptions={[1, 5, 10]}
      onYearsToAdvanceChange={() => undefined}
      onAdvanceOneYear={() => undefined}
      onAdvanceYears={() => undefined}
    />,
  );
}

describe("TimeControls", () => {
  it("uses native controls in reading order without overriding tab order", () => {
    renderControls();

    const controls = [...document.querySelectorAll<HTMLElement>("button, select")];

    expect(controls.map((control) => control.tagName)).toEqual(["BUTTON", "SELECT", "BUTTON"]);
    expect(controls.every((control) => !control.hasAttribute("tabindex"))).toBe(true);
    expect(controls.every((control) => control.tabIndex === 0)).toBe(true);
  });

  it("associates the year selector with its visible label", () => {
    renderControls();

    const select = document.querySelector("select");
    const label = document.querySelector("label");

    expect(select?.id).toBe("years-to-advance");
    expect(label?.htmlFor).toBe(select?.id);
    expect(label?.firstChild?.textContent).toBe("Years to advance");
  });
});
