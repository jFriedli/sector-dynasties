import { Orbit } from "lucide-react";
import { copy } from "./content/copy";
import { ResourceTagIcon, SpecializationIcon } from "./gameIcons";
import { cityRowsFromSector, specializationLabel, totalTreasury } from "./sectorBrowser";
import type { Sector } from "./simTypes";

const numberFormat = new Intl.NumberFormat("en-US");
const creditFormat = new Intl.NumberFormat("en-US", {
  maximumFractionDigits: 2,
  minimumFractionDigits: 2,
});

interface SectorBrowserProps {
  sector: Sector;
  selectedCityId: number | null;
  onSelectCity: (cityId: number) => void;
}

export function SectorBrowser({ sector, selectedCityId, onSelectCity }: SectorBrowserProps) {
  const cityRows = cityRowsFromSector(sector);

  return (
    <section className="sector-browser" aria-labelledby="sector-browser-heading">
      <div className="section-heading">
        <div>
          <h2 id="sector-browser-heading">{copy.sectorBrowserTitle}</h2>
          <p>{sector.name}</p>
        </div>
        <dl className="browser-totals">
          <div>
            <dt>{copy.browserCitiesLabel}</dt>
            <dd>{numberFormat.format(cityRows.length)}</dd>
          </div>
          <div>
            <dt>{copy.browserTreasuryLabel}</dt>
            <dd>{creditFormat.format(totalTreasury(cityRows))}</dd>
          </div>
        </dl>
      </div>

      <div className="system-list">
        {sector.systems.map((system) => (
          <details className="system-panel" key={system.id} open>
            <summary>
              <span className="icon-label">
                <Orbit size={18} aria-hidden="true" />
                {system.name}
              </span>
              <span>{copy.planetsCount(system.planets.length)}</span>
            </summary>
            <div className="planet-list">
              {system.planets.map((planet) => (
                <article className="planet-panel" key={planet.id}>
                  <h3 className="icon-label">
                    {planet.name}
                    <span className="resource-tags">
                      {planet.resource_tags.map((tag) => (
                        <span className="resource-tag-badge" key={tag} title={tag}>
                          <ResourceTagIcon tag={tag} />
                        </span>
                      ))}
                    </span>
                  </h3>
                  <div className="country-list">
                    {planet.countries.map((country) => (
                      <section className="country-panel" key={country.id}>
                        <h4>{country.name}</h4>
                        <p className="country-backstory">{country.backstory}</p>
                        <div className="city-list">
                          {country.cities.map((city) => {
                            const isSelected = selectedCityId === city.id;

                            return (
                              <article
                                className={isSelected ? "city-row city-row--selected" : "city-row"}
                                key={city.id}
                              >
                                <button
                                  className="city-row-select"
                                  type="button"
                                  aria-pressed={isSelected}
                                  onClick={() => onSelectCity(city.id)}
                                >
                                  <span>{city.name}</span>
                                  <span className="icon-label">
                                    <SpecializationIcon specialization={city.specialization} />
                                    {specializationLabel(city.specialization)}
                                  </span>
                                </button>
                                <dl>
                                  <div>
                                    <dt>{copy.cityPopulationLabel}</dt>
                                    <dd>{numberFormat.format(city.population.size)}</dd>
                                  </div>
                                  <div>
                                    <dt>{copy.cityTreasuryLabel}</dt>
                                    <dd>{creditFormat.format(city.treasury)}</dd>
                                  </div>
                                </dl>
                              </article>
                            );
                          })}
                        </div>
                      </section>
                    ))}
                  </div>
                </article>
              ))}
            </div>
          </details>
        ))}
      </div>
    </section>
  );
}
