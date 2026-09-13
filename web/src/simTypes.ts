// Mirrors sim_core::rng::RngDomainSummary. Debug-only: a named RNG stream's
// domain tag and a fingerprint of its current state, for the debug overlay
// (see App.tsx / DebugOverlay.tsx and issue #35).
export interface RngDomainSummary {
  domain: string;
  fingerprint: string;
}

// Mirrors sim_core::dynasty::DynastyRole.
export type DynastyRole = "Head" | "Member";

// Mirrors sim_core::dynasty::DynastyMemberSummary. See DynastyPanel.tsx and
// issue #31.
export interface DynastyMemberSummary {
  id: number;
  name: string;
  age_years: number;
  alive: boolean;
  role: DynastyRole;
}

export interface PendingEventChoiceSummary {
  key: string;
  label: string;
}

export interface PendingEventSummary {
  id: number;
  key: string;
  title: string;
  character_id: number;
  character_name: string;
  raised_year: number;
  choices: PendingEventChoiceSummary[];
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
  dynasty_members: DynastyMemberSummary[];
  pending_events: PendingEventSummary[];
  rng_domains: RngDomainSummary[];
}

export type CitySpecialization = "Mining" | "Manufacturing" | "Finance" | "Research" | "Logistics";
export type ResourceTag = "MetalRich" | "Agricultural" | "Arid";

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
  recent_output_index: number;
}

export interface Country {
  id: number;
  name: string;
  backstory: string;
  social_mobility: number;
  union_power: number;
  cities: City[];
}

export interface Planet {
  id: number;
  name: string;
  resource_tags: ResourceTag[];
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
