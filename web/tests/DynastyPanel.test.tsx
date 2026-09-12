import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { DynastyPanel } from "../src/DynastyPanel";
import type { DynastyMemberSummary } from "../src/simTypes";

const members: DynastyMemberSummary[] = [
  { id: 1, name: "Heir", age_years: 34, alive: true, role: "Head" },
  { id: 2, name: "Cousin", age_years: 40, alive: true, role: "Member" },
];

describe("DynastyPanel", () => {
  it("marks the member picked from search as selected", () => {
    document.body.innerHTML = renderToStaticMarkup(
      <DynastyPanel members={members} selectedMemberId={2} />,
    );

    const selectedRow = document.getElementById("dynasty-member-2");
    const otherRow = document.getElementById("dynasty-member-1");

    expect(selectedRow?.className).toContain("dynasty-member-row--selected");
    expect(otherRow?.className).not.toContain("dynasty-member-row--selected");
  });

  it("selects no row when nothing was picked from search", () => {
    document.body.innerHTML = renderToStaticMarkup(<DynastyPanel members={members} />);

    expect(document.querySelector(".dynasty-member-row--selected")).toBeNull();
  });
});
