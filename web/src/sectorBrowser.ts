import type { City, CitySpecialization, GovernmentProfile, PopulationGroup, Sector } from "./simTypes";

export interface CityBrowserRow {
  id: number;
  name: string;
  systemName: string;
  planetName: string;
  countryName: string;
  specialization: CitySpecialization;
  population: number;
  treasury: number;
}

export interface CityDetail {
  id: number;
  name: string;
  systemName: string;
  planetName: string;
  countryId: number;
  countryName: string;
  specialization: CitySpecialization;
  population: PopulationGroup;
  treasury: number;
  /** The host country's government, for the government/lobbying section of
   * the city detail panel. Government is a country-level concept, but the
   * panel is keyed off a selected city, not a selected country (there's no
   * country selection UI yet), so it's threaded through here. */
  government: GovernmentProfile;
}

export function cityDetailsFromSector(sector: Sector): CityDetail[] {
  return sector.systems.flatMap((system) =>
    system.planets.flatMap((planet) =>
      planet.countries.flatMap((country) =>
        country.cities.map((city) => ({
          id: city.id,
          name: city.name,
          systemName: system.name,
          planetName: planet.name,
          countryId: country.id,
          countryName: country.name,
          specialization: city.specialization,
          population: city.population,
          treasury: city.treasury,
          government: country.government,
        })),
      ),
    ),
  );
}

export function cityRowsFromSector(sector: Sector): CityBrowserRow[] {
  return cityDetailsFromSector(sector).map((city) => ({
    id: city.id,
    name: city.name,
    systemName: city.systemName,
    planetName: city.planetName,
    countryName: city.countryName,
    specialization: city.specialization,
    population: city.population.size,
    treasury: city.treasury,
  }));
}

export function cityDetailById(sector: Sector, cityId: number): CityDetail | null {
  return cityDetailsFromSector(sector).find((city) => city.id === cityId) ?? null;
}

export function totalTreasury(cities: readonly CityBrowserRow[]): number {
  return cities.reduce((sum, city) => sum + city.treasury, 0);
}

export function specializationLabel(specialization: City["specialization"]): string {
  return specialization;
}
