import { copy } from "./content/copy";
import { specializationLabel, type CityDetail } from "./sectorBrowser";

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

interface CityDetailPanelProps {
  city: CityDetail | null;
}

export function CityDetailPanel({ city }: CityDetailPanelProps) {
  if (!city) {
    return (
      <aside className="city-detail-panel city-detail-panel--empty">
        <h2>{copy.cityDetailEmptyTitle}</h2>
        <p>{copy.cityDetailEmptyBody}</p>
      </aside>
    );
  }

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
          <dd>{specializationLabel(city.specialization)}</dd>
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
    </aside>
  );
}
