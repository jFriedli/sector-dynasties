import { describe, expect, it } from "vitest";
import { dynastyMemberRows } from "../src/dynastyPanelLogic";
import type { DynastyMemberSummary } from "../src/simTypes";

const members: DynastyMemberSummary[] = [
  { id: 1, name: "Elder", age_years: 70, alive: false, role: "Member" },
  { id: 2, name: "Heir", age_years: 34, alive: true, role: "Head" },
  { id: 3, name: "Cousin", age_years: 40, alive: true, role: "Member" },
  { id: 4, name: "Uncle", age_years: 60, alive: false, role: "Member" },
];

describe("dynasty panel data shaping", () => {
  it("puts the head first, then living members before dead ones, oldest first within each group", () => {
    expect(dynastyMemberRows(members)).toEqual([
      { id: 2, name: "Heir", ageYears: 34, alive: true, role: "Head" },
      { id: 3, name: "Cousin", ageYears: 40, alive: true, role: "Member" },
      { id: 1, name: "Elder", ageYears: 70, alive: false, role: "Member" },
      { id: 4, name: "Uncle", ageYears: 60, alive: false, role: "Member" },
    ]);
  });

  it("does not mutate the input array", () => {
    const original = [...members];
    dynastyMemberRows(members);
    expect(members).toEqual(original);
  });

  it("returns an empty list for an empty dynasty", () => {
    expect(dynastyMemberRows([])).toEqual([]);
  });
});
