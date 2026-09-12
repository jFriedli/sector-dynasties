// Mirrors sim_core::state::StateSummary. Kept as a hand-written type
// rather than a generated one for the bootstrap slice; see the backlog
// item under `tooling` for generating this from the Rust definition so the
// two can never drift silently.
export interface StateSummary {
  tick: number;
  year: number;
  system_count: number;
  city_count: number;
  total_population: number;
  dynasty_name: string;
  dynasty_wealth: number;
}
