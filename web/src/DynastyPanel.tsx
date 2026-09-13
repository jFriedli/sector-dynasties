import { useEffect } from "react";
import { Crown, Skull, User } from "lucide-react";
import { copy } from "./content/copy";
import { dynastyMemberRows } from "./dynastyPanelLogic";
import type { DynastyMemberSummary } from "./simTypes";

interface DynastyPanelProps {
  members: DynastyMemberSummary[];
  /** A member to highlight and scroll into view, e.g. after picking a
   * character from global search (issue #88). There's no separate
   * character sheet panel yet (#83), so this list is the current
   * equivalent to jump to. */
  selectedMemberId?: number | null;
}

/** Lists dynasty members so the player can see their family (issue #31):
 * name, age, role, and whether each member is still alive. */
export function DynastyPanel({ members, selectedMemberId = null }: DynastyPanelProps) {
  const rows = dynastyMemberRows(members);

  useEffect(() => {
    if (selectedMemberId == null) return;
    document
      .getElementById(`dynasty-member-${selectedMemberId}`)
      ?.scrollIntoView?.({ behavior: "smooth", block: "nearest" });
  }, [selectedMemberId]);

  return (
    <section className="dynasty-panel" aria-labelledby="dynasty-panel-heading">
      <h2 id="dynasty-panel-heading">{copy.dynastyPanelTitle}</h2>
      <ul className="dynasty-member-list">
        {rows.map((member) => (
          <li
            id={`dynasty-member-${member.id}`}
            className={[
              "dynasty-member-row",
              member.alive ? "" : "dynasty-member-row--deceased",
              member.id === selectedMemberId ? "dynasty-member-row--selected" : "",
            ]
              .filter(Boolean)
              .join(" ")}
            key={member.id}
          >
            <div className="dynasty-member-identity-group">
              <span className="dynasty-member-icon" aria-hidden="true">
                {!member.alive ? (
                  <Skull size={20} />
                ) : member.role === "Head" ? (
                  <Crown size={20} />
                ) : (
                  <User size={20} />
                )}
              </span>
              <div className="dynasty-member-identity">
                <span className="dynasty-member-name">{member.name}</span>
                <span className="dynasty-member-role">
                  {member.role === "Head" ? copy.dynastyRoleHead : copy.dynastyRoleMember}
                </span>
              </div>
            </div>
            <div className="dynasty-member-status">
              <span>{copy.dynastyAge(member.ageYears)}</span>
              <span>{member.alive ? copy.dynastyStatusAlive : copy.dynastyStatusDeceased}</span>
            </div>
          </li>
        ))}
      </ul>
    </section>
  );
}
