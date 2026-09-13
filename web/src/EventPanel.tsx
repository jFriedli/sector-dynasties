import { copy } from "./content/copy";
import type { PendingEventSummary } from "./simTypes";

interface EventPanelProps {
  event: PendingEventSummary | null;
  onResolve: (eventId: number, choiceKey: string) => void;
}

export function EventPanel({ event, onResolve }: EventPanelProps) {
  if (!event) return null;

  return (
    <section className="event-panel" aria-label={copy.eventPanelLabel}>
      <div className="event-panel-heading">
        <p>{copy.eventPanelLabel}</p>
        <h2>{event.title}</h2>
        <span>{copy.eventForCharacter(event.character_name, event.raised_year)}</span>
      </div>
      <div className="event-choice-list">
        {event.choices.map((choice) => (
          <button
            className="button"
            type="button"
            key={choice.key}
            onClick={() => onResolve(event.id, choice.key)}
          >
            {choice.label}
          </button>
        ))}
      </div>
    </section>
  );
}
