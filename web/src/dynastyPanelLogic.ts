import type { DynastyMemberSummary, DynastyRole } from "./simTypes";

export interface DynastyMemberRow {
  id: number;
  name: string;
  ageYears: number;
  alive: boolean;
  role: DynastyRole;
}

/** Orders dynasty members for display: the head first, then living members
 * before dead ones, then oldest to youngest within each group. Keeps this
 * ordering logic out of the component so it can be unit tested on its own,
 * matching the sector browser's split between shaping and rendering. */
export function dynastyMemberRows(members: readonly DynastyMemberSummary[]): DynastyMemberRow[] {
  return members
    .map((member) => ({
      id: member.id,
      name: member.name,
      ageYears: member.age_years,
      alive: member.alive,
      role: member.role,
    }))
    .sort((a, b) => {
      if (a.role !== b.role) return a.role === "Head" ? -1 : 1;
      if (a.alive !== b.alive) return a.alive ? -1 : 1;
      return b.ageYears - a.ageYears;
    });
}
