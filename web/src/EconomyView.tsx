import { type FormEvent, useEffect, useState } from "react";
import { Briefcase, Building2, Coins, Route } from "lucide-react";
import { copy } from "./content/copy";
import { BUSINESS_ARCHETYPES, citiesEligibleForArchetype } from "./economyViewLogic";
import type { BusinessView, CareerView, Sector, TradeRouteView } from "./simTypes";

const creditFormat = new Intl.NumberFormat("en-US", {
  maximumFractionDigits: 2,
  minimumFractionDigits: 2,
});
const percentFormat = new Intl.NumberFormat("en-US", {
  maximumFractionDigits: 0,
  style: "percent",
});

interface EconomyViewProps {
  sector: Sector;
  businesses: BusinessView[];
  tradeRoutes: TradeRouteView[];
  careers: CareerView[];
  selectedBusinessId?: number | null;
  onFoundBusiness: (name: string, archetype: string, hostCityId: number) => Promise<void> | void;
}

export function EconomyView({
  sector,
  businesses,
  tradeRoutes,
  careers,
  selectedBusinessId = null,
  onFoundBusiness,
}: EconomyViewProps) {
  const [formOpen, setFormOpen] = useState(false);
  const [name, setName] = useState("");
  const [archetype, setArchetype] = useState(BUSINESS_ARCHETYPES[0]?.value ?? "");
  const [hostCityId, setHostCityId] = useState<number | "">("");
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  const eligibleCities = citiesEligibleForArchetype(sector, archetype);

  useEffect(() => {
    if (selectedBusinessId == null) return;
    document
      .getElementById(`business-${selectedBusinessId}`)
      ?.scrollIntoView?.({ behavior: "smooth", block: "nearest" });
  }, [selectedBusinessId]);

  const handleSubmit = async (event: FormEvent) => {
    event.preventDefault();
    if (hostCityId === "" || name.trim().length === 0) return;
    setSubmitting(true);
    setError(null);
    try {
      await onFoundBusiness(name.trim(), archetype, hostCityId);
      setName("");
      setHostCityId("");
      setFormOpen(false);
    } catch (thrown: unknown) {
      setError(copy.foundBusinessError(String(thrown)));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <section className="economy-view" aria-labelledby="economy-view-heading">
      <h2 id="economy-view-heading" className="icon-label">
        <Coins size={22} aria-hidden="true" />
        {copy.economyScreenTitle}
      </h2>

      <div className="economy-section">
        <div className="section-heading">
          <h3 className="icon-label">
            <Building2 size={18} aria-hidden="true" />
            {copy.businessesHeading}
          </h3>
          <button type="button" className="button" onClick={() => setFormOpen((open) => !open)}>
            {copy.foundBusinessButton}
          </button>
        </div>

        {formOpen && (
          <form className="found-business-form" onSubmit={handleSubmit}>
            <h4>{copy.foundBusinessFormTitle}</h4>
            {error && <p className="form-error">{error}</p>}
            <label>
              {copy.foundBusinessNameLabel}
              <input
                type="text"
                value={name}
                onChange={(event) => setName(event.target.value)}
                required
              />
            </label>
            <label>
              {copy.foundBusinessArchetypeLabel}
              <select
                value={archetype}
                onChange={(event) => {
                  setArchetype(event.target.value);
                  setHostCityId("");
                }}
              >
                {BUSINESS_ARCHETYPES.map((entry) => (
                  <option key={entry.value} value={entry.value}>
                    {entry.value}
                  </option>
                ))}
              </select>
            </label>
            <label>
              {copy.foundBusinessCityLabel}
              {eligibleCities.length === 0 ? (
                <p className="empty-state">{copy.foundBusinessNoEligibleCities}</p>
              ) : (
                <select
                  value={hostCityId}
                  onChange={(event) => setHostCityId(Number(event.target.value))}
                  required
                >
                  <option value="" disabled>
                    {copy.foundBusinessCityLabel}
                  </option>
                  {eligibleCities.map((city) => (
                    <option key={city.id} value={city.id}>
                      {city.name}
                    </option>
                  ))}
                </select>
              )}
            </label>
            <div className="found-business-form-actions">
              <button
                type="submit"
                className="button"
                disabled={submitting || eligibleCities.length === 0}
              >
                {copy.foundBusinessSubmit}
              </button>
              <button
                type="button"
                className="button button--secondary"
                onClick={() => setFormOpen(false)}
              >
                {copy.foundBusinessCancel}
              </button>
            </div>
          </form>
        )}

        {businesses.length === 0 ? (
          <p className="empty-state">{copy.businessesEmpty}</p>
        ) : (
          <ul className="business-list">
            {businesses.map((business) => (
              <li
                id={`business-${business.id}`}
                className={[
                  "business-row",
                  business.id === selectedBusinessId ? "business-row--selected" : "",
                ]
                  .filter(Boolean)
                  .join(" ")}
                key={business.id}
              >
                <div>
                  <span className="business-name">{business.name}</span>
                  <span className="business-archetype">{business.archetype}</span>
                </div>
                <div>
                  <span>{business.host_city_name}</span>
                  <dl>
                    <dt>{copy.businessEquityLabel}</dt>
                    <dd>{creditFormat.format(business.equity)}</dd>
                  </dl>
                </div>
              </li>
            ))}
          </ul>
        )}
      </div>

      <div className="economy-section">
        <h3 className="icon-label">
          <Route size={18} aria-hidden="true" />
          {copy.tradeRoutesHeading}
        </h3>
        {tradeRoutes.length === 0 ? (
          <p className="empty-state">{copy.tradeRoutesEmpty}</p>
        ) : (
          <ul className="trade-route-list">
            {tradeRoutes.map((route) => (
              <li className="trade-route-row" key={route.id}>
                <span className="trade-route-endpoints">
                  {route.city_a_name} <span aria-hidden="true">↔</span> {route.city_b_name}
                </span>
                <dl>
                  <div>
                    <dt>{copy.tradeRouteCapacityLabel}</dt>
                    <dd>{creditFormat.format(route.capacity)}</dd>
                  </div>
                  <div>
                    <dt>{copy.tradeRouteCostLabel}</dt>
                    <dd>{creditFormat.format(route.cost)}</dd>
                  </div>
                  <div>
                    <dt>{copy.tradeRouteReliabilityLabel}</dt>
                    <dd>{percentFormat.format(route.reliability)}</dd>
                  </div>
                </dl>
              </li>
            ))}
          </ul>
        )}
      </div>

      <div className="economy-section">
        <h3 className="icon-label">
          <Briefcase size={18} aria-hidden="true" />
          {copy.careersHeading}
        </h3>
        {careers.length === 0 ? (
          <p className="empty-state">{copy.careersEmpty}</p>
        ) : (
          <ul className="career-list">
            {careers.map((career) => (
              <li className="career-row" key={career.id}>
                <span>{career.character_name}</span>
                <span className="icon-label">
                  <Briefcase size={14} aria-hidden="true" />
                  {career.job_title}
                </span>
                <span>{career.employer_city_name}</span>
              </li>
            ))}
          </ul>
        )}
      </div>
    </section>
  );
}
