// Mirrors sim_core::rng::RngDomainSummary. Debug-only: a named RNG stream's
// domain tag and a fingerprint of its current state, for the debug overlay
// (see App.tsx / DebugOverlay.tsx and issue #35).
export interface RngDomainSummary {
  domain: string;
  fingerprint: string;
}

// Mirrors sim_core::state::StateSummary. Kept as a hand-written type
// rather than a generated one for the bootstrap slice; see the backlog
// item under `tooling` for generating this from the Rust definition so the
// two can never drift silently.
export interface StateSummary {
  tick: number;
  year: number;
  seed: number;
  system_count: number;
  city_count: number;
  total_population: number;
  dynasty_name: string;
  dynasty_wealth: number;
  rng_domains: RngDomainSummary[];
}

export type CitySpecialization = "Mining" | "Manufacturing" | "Finance" | "Research" | "Logistics";

export interface PopulationGroup {
  size: number;
  average_wealth: number;
  unemployment_rate: number;
}

export interface City {
  id: number;
  name: string;
  specialization: CitySpecialization;
  population: PopulationGroup;
  treasury: number;
}

export interface Country {
  id: number;
  name: string;
  cities: City[];
}

export interface Planet {
  id: number;
  name: string;
  countries: Country[];
}

export interface StarSystem {
  id: number;
  name: string;
  planets: Planet[];
}

export interface Sector {
  seed: number;
  name: string;
  systems: StarSystem[];
}

export interface SimStateSnapshot {
  seed: number;
  sector: Sector;
}
