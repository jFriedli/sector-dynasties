import { copy } from "./content/copy";
import { dynastyMemberRows } from "./dynastyPanel";
import type { DynastyMemberSummary } from "./simTypes";

interface DynastyPanelProps {
  members: DynastyMemberSummary[];
}

/** Lists dynasty members so the player can see their family (issue #31):
 * name, age, role, and whether each member is still alive. */
export function DynastyPanel({ members }: DynastyPanelProps) {
  const rows = dynastyMemberRows(members);

  return (
    <section className="dynasty-panel" aria-labelledby="dynasty-panel-heading">
      <h2 id="dynasty-panel-heading">{copy.dynastyPanelTitle}</h2>
      <ul className="dynasty-member-list">
        {rows.map((member) => (
          <li
            className={
              member.alive
                ? "dynasty-member-row"
                : "dynasty-member-row dynasty-member-row--deceased"
            }
            key={member.id}
          >
            <div className="dynasty-member-identity">
              <span className="dynasty-member-name">{member.name}</span>
              <span className="dynasty-member-role">
                {member.role === "Head" ? copy.dynastyRoleHead : copy.dynastyRoleMember}
              </span>
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
