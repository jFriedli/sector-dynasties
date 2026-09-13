import type { City, CitySpecialization, PopulationGroup, Sector } from "./simTypes";

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
  countryName: string;
  specialization: CitySpecialization;
  population: PopulationGroup;
  treasury: number;
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
          countryName: country.name,
          specialization: city.specialization,
          population: city.population,
          treasury: city.treasury,
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
