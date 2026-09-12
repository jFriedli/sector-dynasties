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

/// How quickly `City::recent_output_index` folds in each week's output
/// ratio. Smaller means a longer memory (one noisy week barely moves it),
/// so only a *stretch* of depressed or booming output, not a single bad
/// week, meaningfully drives unemployment below.
const OUTPUT_INDEX_SMOOTHING: f64 = 0.12;

/// How much a sustained shortfall (or surplus) in `recent_output_index`
/// moves unemployment per week, on top of the existing residual noise.
/// Tuned so a genuinely depressed multi-week stretch clearly outpaces
/// ordinary noise-driven drift.
const UNEMPLOYMENT_OUTPUT_SENSITIVITY: f64 = 0.05;

fn specialization_multiplier(spec: CitySpecialization) -> f64 {
    match spec {
        CitySpecialization::Finance => 1.4,
        CitySpecialization::Research => 1.25,
        CitySpecialization::Manufacturing => 1.1,
        CitySpecialization::Logistics => 1.05,
        CitySpecialization::Mining => 1.0,
    }
}

/// Fold this week's output ratio (actual output / noise-free baseline
/// output) into the rolling output index using an exponential moving
/// average, so the index tracks a stretch of performance rather than any
/// single week's noise draw.
fn update_output_index(previous_index: f64, output_ratio: f64) -> f64 {
    previous_index * (1.0 - OUTPUT_INDEX_SMOOTHING) + output_ratio * OUTPUT_INDEX_SMOOTHING
}

/// This week's unemployment drift: the existing residual noise plus a
/// response to the rolling output index. An index below `1.0` (a
/// depressed stretch) pushes unemployment up; above `1.0` (a boom) pulls
/// it down.
fn unemployment_drift(recent_output_index: f64, noise: f64) -> f64 {
    let output_driven = (1.0 - recent_output_index) * UNEMPLOYMENT_OUTPUT_SENSITIVITY;
    output_driven + noise
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
                    let baseline_output =
                        city.population.size as f64 * BASE_OUTPUT_PER_CAPITA * multiplier;
                    let noise = rng.range_f64(0.85, 1.15);
                    let output = baseline_output * noise;

                    city.treasury += output;

                    let employment_rate = 1.0 - city.population.unemployment_rate;
                    let wage_share =
                        output * 0.4 * employment_rate / city.population.size.max(1) as f64;
                    city.population.average_wealth =
                        (city.population.average_wealth + wage_share).max(0.0);

                    // Roll this week's output ratio into the city's recent
                    // output index, then let a sustained shortfall or
                    // surplus (not just this week's noise draw) drive
                    // unemployment, with a soft floor/ceiling so it never
                    // silently walks out of a valid range.
                    let output_ratio = if baseline_output > 0.0 {
                        output / baseline_output
                    } else {
                        1.0
                    };
                    city.recent_output_index =
                        update_output_index(city.recent_output_index, output_ratio);

                    let residual_noise = rng.range_f64(-0.01, 0.01);
                    let drift = unemployment_drift(city.recent_output_index, residual_noise);
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

    #[test]
    fn output_index_tracks_a_sustained_ratio_rather_than_snapping_to_it() {
        // One week of a bad ratio should barely move the index (it's a
        // rolling measure, not this week's noise); a stretch of the same
        // ratio should move it much closer.
        let after_one_week = update_output_index(1.0, 0.5);
        assert!(
            (0.9..1.0).contains(&after_one_week),
            "one bad week moved the index too far: {after_one_week}"
        );

        let mut index = 1.0;
        for _ in 0..60 {
            index = update_output_index(index, 0.5);
        }
        assert!(
            (index - 0.5).abs() < 0.01,
            "a long sustained ratio should converge near it, got {index}"
        );
    }

    #[test]
    fn unemployment_drift_pushes_up_for_a_shortfall_and_down_for_a_surplus() {
        assert!(unemployment_drift(0.5, 0.0) > 0.0);
        assert!(unemployment_drift(1.5, 0.0) < 0.0);
        assert_eq!(unemployment_drift(1.0, 0.0), 0.0);
    }

    #[test]
    fn sustained_output_shortfall_raises_unemployment_more_than_baseline_noise_would() {
        // Two independent weekly walks from the same starting unemployment
        // rate and output index, with noise held at its midpoint (0.0) in
        // both so the only difference is the output signal: one city's
        // output stays at half its baseline for a stretch (a "specialization
        // shock"), the other tracks baseline exactly, so it never gets a
        // noise-only nudge either.
        let mut depressed_rate = 0.05_f64;
        let mut depressed_index = 1.0_f64;
        let mut baseline_rate = 0.05_f64;
        let mut baseline_index = 1.0_f64;

        for _ in 0..12 {
            depressed_index = update_output_index(depressed_index, 0.5);
            depressed_rate =
                (depressed_rate + unemployment_drift(depressed_index, 0.0)).clamp(0.01, 0.35);

            baseline_index = update_output_index(baseline_index, 1.0);
            baseline_rate =
                (baseline_rate + unemployment_drift(baseline_index, 0.0)).clamp(0.01, 0.35);
        }

        assert_eq!(baseline_rate, 0.05, "an unshocked city should not drift");
        assert!(
            depressed_rate > baseline_rate + 0.03,
            "a sustained shortfall ({depressed_rate}) should measurably exceed baseline \
             drift ({baseline_rate})"
        );
    }

    #[test]
    fn settlement_updates_the_recent_output_index_and_keeps_it_finite() {
        let mut sector = generate_sector(4242, 2);
        let mut rng = SimRng::from_seed(4242, "economy");
        for _ in 0..26 {
            settle_week(&mut sector, &mut rng);
        }
        for system in &sector.systems {
            for planet in &system.planets {
                for country in &planet.countries {
                    for city in &country.cities {
                        assert!(city.recent_output_index.is_finite());
                        assert!(city.recent_output_index > 0.0);
                        // Noise is bounded to [0.85, 1.15], so a rolling
                        // average of ratios in that range can never stray
                        // outside it either.
                        assert!((0.85..=1.15).contains(&city.recent_output_index));
                    }
                }
            }
        }
    }
}
