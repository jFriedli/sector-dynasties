import type { SimStateSnapshot, StateSummary } from "./simTypes";

export interface EntityCounts {
  systems: number;
  planets: number;
  countries: number;
  cities: number;
  dynastyMembers: number;
  total: number;
}

export interface LastStepPerformance {
  days: number;
  durationMs: number;
}

export function entityCountsFromState(
  snapshot: SimStateSnapshot,
  summary: StateSummary,
): EntityCounts {
  const systems = snapshot.sector.systems.length;
  const planets = snapshot.sector.systems.reduce((sum, system) => sum + system.planets.length, 0);
  const countries = snapshot.sector.systems.reduce(
    (sum, system) =>
      sum + system.planets.reduce((planetSum, planet) => planetSum + planet.countries.length, 0),
    0,
  );
  const cities = snapshot.sector.systems.reduce(
    (sum, system) =>
      sum +
      system.planets.reduce(
        (planetSum, planet) =>
          planetSum +
          planet.countries.reduce((countrySum, country) => countrySum + country.cities.length, 0),
        0,
      ),
    0,
  );
  const dynastyMembers = summary.dynasty_members.length;

  return {
    systems,
    planets,
    countries,
    cities,
    dynastyMembers,
    total: systems + planets + countries + cities + dynastyMembers,
  };
}

export function formatDurationMs(durationMs: number): string {
  if (!Number.isFinite(durationMs) || durationMs < 0) return "n/a";
  if (durationMs < 10) return `${durationMs.toFixed(2)} ms`;
  if (durationMs < 100) return `${durationMs.toFixed(1)} ms`;
  return `${Math.round(durationMs)} ms`;
}
