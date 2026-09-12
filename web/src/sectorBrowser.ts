import type { City, CitySpecialization, Sector } from "./simTypes";

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

export function cityRowsFromSector(sector: Sector): CityBrowserRow[] {
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
          population: city.population.size,
          treasury: city.treasury,
        })),
      ),
    ),
  );
}

export function totalTreasury(cities: readonly CityBrowserRow[]): number {
  return cities.reduce((sum, city) => sum + city.treasury, 0);
}

export function specializationLabel(specialization: City["specialization"]): string {
  return specialization;
}
