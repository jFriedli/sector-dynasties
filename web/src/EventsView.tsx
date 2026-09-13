import { useState } from "react";
import { ScrollText } from "lucide-react";
import { copy } from "./content/copy";
import type { PendingEventView } from "./simTypes";

interface EventsViewProps {
  pendingEvents: PendingEventView[];
  onResolve: (pendingId: number, choiceKey: string) => Promise<void> | void;
}

/** The screen where the player actually makes decisions: each pending
 * event (issue #29's framework) becomes a card with its choices as
 * buttons. Resolving one calls back into the wasm bridge and lets the
 * parent refresh state, this component holds no simulation state itself. */
export function EventsView({ pendingEvents, onResolve }: EventsViewProps) {
  const [resolvingId, setResolvingId] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleChoice = async (pendingId: number, choiceKey: string) => {
    setResolvingId(pendingId);
    setError(null);
    try {
      await onResolve(pendingId, choiceKey);
    } catch (thrown: unknown) {
      setError(copy.eventResolveError(String(thrown)));
    } finally {
      setResolvingId(null);
    }
  };

  return (
    <section className="events-view" aria-labelledby="events-view-heading">
      <h2 id="events-view-heading" className="icon-label">
        <ScrollText size={22} aria-hidden="true" />
        {copy.eventsScreenTitle}
      </h2>
      {error && <p className="form-error">{error}</p>}
      {pendingEvents.length === 0 ? (
        <p className="empty-state">{copy.eventsScreenEmpty}</p>
      ) : (
        <ul className="event-card-list">
          {pendingEvents.map((event) => (
            <li className="event-card" key={event.id}>
              <div className="event-card-header">
                <h3>{event.title}</h3>
                <span className="event-card-meta">{copy.eventRaisedYear(event.raised_year)}</span>
              </div>
              <p className="event-card-character">{event.character_name}</p>
              <div className="event-card-choices">
                {event.choices.map((choice) => (
                  <button
                    key={choice.key}
                    type="button"
                    className="button button--event-choice"
                    disabled={resolvingId === event.id}
                    onClick={() => handleChoice(event.id, choice.key)}
                  >
                    {choice.label}
                  </button>
                ))}
              </div>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
