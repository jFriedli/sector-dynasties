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

// Mirrors sim_core::views::EventChoiceView.
export interface EventChoiceView {
  key: string;
  label: string;
}

// Mirrors sim_core::views::PendingEventView. A decision awaiting the
// player; see EventsView.tsx.
export interface PendingEventView {
  id: number;
  key: string;
  title: string;
  character_id: number;
  character_name: string;
  raised_year: number;
  choices: EventChoiceView[];
}

// Mirrors sim_core::views::BusinessView.
export interface BusinessView {
  id: number;
  name: string;
  archetype: string;
  host_city_id: number;
  host_city_name: string;
  equity: number;
}

// Mirrors sim_core::views::TradeRouteView.
export interface TradeRouteView {
  id: number;
  city_a_id: number;
  city_a_name: string;
  city_b_id: number;
  city_b_name: string;
  capacity: number;
  distance: number;
  cost: number;
  reliability: number;
}

// Mirrors sim_core::views::CareerView.
export interface CareerView {
  id: number;
  character_id: number;
  character_name: string;
  track: string;
  job_title: string;
  employer_city_id: number;
  employer_city_name: string;
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
  rng_domains: RngDomainSummary[];
  pending_events: PendingEventView[];
  businesses: BusinessView[];
  trade_routes: TradeRouteView[];
  careers: CareerView[];
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

// Mirrors sim_core::world::GovernmentComponent. Only these four dimensions
// are targetable by lobbying today; legislative_strength/
// judicial_independence are shown read-only until a mechanic reads them.
export type GovernmentComponent = "federalism" | "franchise" | "economicLiberalism" | "pressFreedom";
export type PolicyDirection = "increase" | "decrease";

// Mirrors sim_core::world::GovernmentProfile.
export interface GovernmentProfile {
  federalism: number;
  franchise: number;
  economic_liberalism: number;
  press_freedom: number;
  legislative_strength: number;
  judicial_independence: number;
}

export interface Country {
  id: number;
  name: string;
  backstory: string;
  social_mobility: number;
  union_power: number;
  government: GovernmentProfile;
  cities: City[];
}

// Mirrors sim_core::state::LobbyingOutcome, the JSON SimHandle.lobbyPolicy
// resolves with on success.
export interface LobbyingOutcome {
  country_id: number;
  component: "Federalism" | "Franchise" | "EconomicLiberalism" | "PressFreedom";
  direction: "Increase" | "Decrease";
  wealth_spent: number;
  before: number;
  after: number;
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
