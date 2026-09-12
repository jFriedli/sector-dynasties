//! Minimal weekly economic settlement.
//!
//! This intentionally does not model individual goods, production chains,
//! or trade routes yet; it exists to prove the tick/settlement-frequency
//! architecture (see docs/ARCHITECTURE.md) and give the vertical slice
//! something to show. Real production chains are backlog items under the
//! `economy` and `trade` labels.

use crate::rng::SimRng;
use crate::world::{CitySpecialization, Sector};

/// Base weekly output per capita, before specialization and noise. Chosen
/// to make wealth visibly move over a handful of simulated years without
/// tuning for balance, since balance is out of scope for the bootstrap
/// slice.
const BASE_OUTPUT_PER_CAPITA: f64 = 0.02;

fn specialization_multiplier(spec: CitySpecialization) -> f64 {
    match spec {
        CitySpecialization::Finance => 1.4,
        CitySpecialization::Research => 1.25,
        CitySpecialization::Manufacturing => 1.1,
        CitySpecialization::Logistics => 1.05,
        CitySpecialization::Mining => 1.0,
    }
}

/// Run one weekly settlement over every city in the sector, using an
/// economy-domain RNG stream so this never perturbs worldgen, character, or
/// any other stream's sequence.
pub fn settle_week(sector: &mut Sector, rng: &mut SimRng) {
    for system in &mut sector.systems {
        for planet in &mut system.planets {
            for country in &mut planet.countries {
                for city in &mut country.cities {
                    let multiplier = specialization_multiplier(city.specialization);
                    let noise = rng.range_f64(0.85, 1.15);
                    let output =
                        city.population.size as f64 * BASE_OUTPUT_PER_CAPITA * multiplier * noise;

                    city.treasury += output;

                    let employment_rate = 1.0 - city.population.unemployment_rate;
                    let wage_share =
                        output * 0.4 * employment_rate / city.population.size.max(1) as f64;
                    city.population.average_wealth =
                        (city.population.average_wealth + wage_share).max(0.0);

                    // Unemployment drifts slightly with a soft floor/ceiling
                    // so it never silently walks out of a valid range.
                    let drift = rng.range_f64(-0.01, 0.01);
                    city.population.unemployment_rate =
                        (city.population.unemployment_rate + drift).clamp(0.01, 0.35);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::worldgen::generate_sector;

    #[test]
    fn settlement_keeps_population_groups_valid() {
        let mut sector = generate_sector(2024, 3);
        let mut rng = SimRng::from_seed(2024, "economy");
        for _ in 0..52 {
            settle_week(&mut sector, &mut rng);
        }
        for system in &sector.systems {
            for planet in &system.planets {
                for country in &planet.countries {
                    for city in &country.cities {
                        assert!(
                            city.population.is_valid(),
                            "invalid population in {}",
                            city.name
                        );
                        assert!(city.treasury.is_finite() && city.treasury >= 0.0);
                    }
                }
            }
        }
    }

    #[test]
    fn same_seed_settlement_is_deterministic() {
        let run = || {
            let mut sector = generate_sector(77, 2);
            let mut rng = SimRng::from_seed(77, "economy");
            for _ in 0..10 {
                settle_week(&mut sector, &mut rng);
            }
            serde_json::to_string(&sector).unwrap()
        };
        assert_eq!(run(), run());
    }
}
