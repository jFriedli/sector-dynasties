import { type FormEvent, useState } from "react";
import { Landmark } from "lucide-react";
import { copy } from "./content/copy";
import { SpecializationIcon } from "./gameIcons";
import { specializationLabel, type CityDetail } from "./sectorBrowserLogic";
import type { GovernmentComponent, LobbyingOutcome, PolicyDirection } from "./simTypes";

const numberFormat = new Intl.NumberFormat("en-US");
const percentFormat = new Intl.NumberFormat("en-US", {
  maximumFractionDigits: 1,
  minimumFractionDigits: 1,
  style: "percent",
});
const creditFormat = new Intl.NumberFormat("en-US", {
  maximumFractionDigits: 2,
  minimumFractionDigits: 2,
});

const LOBBYABLE_COMPONENTS: { value: GovernmentComponent; label: string }[] = [
  { value: "federalism", label: copy.governmentFederalismLabel },
  { value: "franchise", label: copy.governmentFranchiseLabel },
  { value: "economicLiberalism", label: copy.governmentEconomicLiberalismLabel },
  { value: "pressFreedom", label: copy.governmentPressFreedomLabel },
];

interface CityDetailPanelProps {
  city: CityDetail | null;
  /** Country-level lobbying cost, for the form's label. Read from
   * sim-core's LOBBYING_WEALTH_COST via App.tsx rather than duplicated
   * here, so the two can never drift. */
  lobbyingCost: number;
  onLobby: (
    countryId: number,
    component: GovernmentComponent,
    direction: PolicyDirection,
  ) => Promise<LobbyingOutcome> | LobbyingOutcome;
}

export function CityDetailPanel({ city, lobbyingCost, onLobby }: CityDetailPanelProps) {
  const [component, setComponent] = useState<GovernmentComponent>("federalism");
  const [direction, setDirection] = useState<PolicyDirection>("increase");
  const [status, setStatus] = useState<{ kind: "outcome" | "error"; message: string } | null>(null);
  const [submitting, setSubmitting] = useState(false);

  if (!city) {
    return (
      <aside className="city-detail-panel city-detail-panel--empty">
        <h2>{copy.cityDetailEmptyTitle}</h2>
        <p>{copy.cityDetailEmptyBody}</p>
      </aside>
    );
  }

  const handleLobby = async (event: FormEvent) => {
    event.preventDefault();
    setSubmitting(true);
    try {
      const outcome = await onLobby(city.countryId, component, direction);
      setStatus({
        kind: "outcome",
        message: copy.lobbyOutcome(
          outcome.component,
          percentFormat.format(outcome.before),
          percentFormat.format(outcome.after),
        ),
      });
    } catch (thrown: unknown) {
      setStatus({ kind: "error", message: copy.lobbyError(String(thrown)) });
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <aside className="city-detail-panel" aria-labelledby="city-detail-heading">
      <p className="eyebrow">{copy.cityDetailEyebrow}</p>
      <h2 id="city-detail-heading">{city.name}</h2>
      <p className="city-location">
        {city.systemName} / {city.planetName} / {city.countryName}
      </p>
      <dl className="detail-stats">
        <div>
          <dt>{copy.citySpecializationLabel}</dt>
          <dd className="icon-label">
            <SpecializationIcon specialization={city.specialization} />
            {specializationLabel(city.specialization)}
          </dd>
        </div>
        <div>
          <dt>{copy.cityPopulationLabel}</dt>
          <dd>{numberFormat.format(city.population.size)}</dd>
        </div>
        <div>
          <dt>{copy.cityTreasuryLabel}</dt>
          <dd>{creditFormat.format(city.treasury)}</dd>
        </div>
        <div>
          <dt>{copy.cityAverageWealthLabel}</dt>
          <dd>{creditFormat.format(city.population.average_wealth)}</dd>
        </div>
        <div>
          <dt>{copy.cityUnemploymentLabel}</dt>
          <dd>{percentFormat.format(city.population.unemployment_rate)}</dd>
        </div>
      </dl>

      <h3 className="icon-label government-heading">
        <Landmark size={18} aria-hidden="true" />
        {copy.governmentHeading}
      </h3>
      <dl className="detail-stats government-stats">
        <div>
          <dt>{copy.governmentFederalismLabel}</dt>
          <dd>{percentFormat.format(city.government.federalism)}</dd>
        </div>
        <div>
          <dt>{copy.governmentFranchiseLabel}</dt>
          <dd>{percentFormat.format(city.government.franchise)}</dd>
        </div>
        <div>
          <dt>{copy.governmentEconomicLiberalismLabel}</dt>
          <dd>{percentFormat.format(city.government.economic_liberalism)}</dd>
        </div>
        <div>
          <dt>{copy.governmentPressFreedomLabel}</dt>
          <dd>{percentFormat.format(city.government.press_freedom)}</dd>
        </div>
        <div>
          <dt>{copy.governmentLegislativeStrengthLabel}</dt>
          <dd>{percentFormat.format(city.government.legislative_strength)}</dd>
        </div>
        <div>
          <dt>{copy.governmentJudicialIndependenceLabel}</dt>
          <dd>{percentFormat.format(city.government.judicial_independence)}</dd>
        </div>
      </dl>

      <form className="lobby-form" onSubmit={handleLobby}>
        <p className="lobby-form-title">{copy.lobbyFormTitle(creditFormat.format(lobbyingCost))}</p>
        {status && (
          <p className={`form-${status.kind === "error" ? "error" : "status"}`}>{status.message}</p>
        )}
        <label>
          {copy.lobbyComponentLabel}
          <select
            value={component}
            onChange={(event) => setComponent(event.target.value as GovernmentComponent)}
          >
            {LOBBYABLE_COMPONENTS.map((entry) => (
              <option key={entry.value} value={entry.value}>
                {entry.label}
              </option>
            ))}
          </select>
        </label>
        <label>
          {copy.lobbyDirectionLabel}
          <select
            value={direction}
            onChange={(event) => setDirection(event.target.value as PolicyDirection)}
          >
            <option value="increase">{copy.lobbyDirectionIncrease}</option>
            <option value="decrease">{copy.lobbyDirectionDecrease}</option>
          </select>
        </label>
        <button type="submit" className="button" disabled={submitting}>
          {copy.lobbySubmit}
        </button>
      </form>
    </aside>
  );
}
