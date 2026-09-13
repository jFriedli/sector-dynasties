//! Monthly migration between cities.
//!
//! A small, deterministic population-pressure model: cities whose
//! attractiveness (average wealth and employment) sits below the sector's
//! population-weighted average lose a small population share each month,
//! and cities above the average gain it. This is intentionally simple;
//! balancing the formula is later work (see issue #41's notes). No RNG is
//! involved, so this never perturbs any RNG stream's sequence and adds
//! nothing new to reproduce from a seed beyond the existing wealth and
//! unemployment state the formula already reads.

use crate::world::Sector;

/// Upper bound on how much of a city's population can leave in a single
/// month from migration pressure alone. Kept small so migration reads as a
/// slow multi-year drift rather than a monthly population swing.
const MAX_MONTHLY_OUTFLOW_SHARE: f64 = 0.01;

/// How strongly a city's relative shortfall below the sector's average
/// attractiveness converts into outflow share, before the
/// `MAX_MONTHLY_OUTFLOW_SHARE` cap applies.
const MIGRATION_PRESSURE_SENSITIVITY: f64 = 0.05;

/// A city's migration-relevant attractiveness: higher average wealth and
/// lower unemployment both make a city more attractive to migrants. A
/// simple product rather than a weighted sum, since balancing this formula
/// is explicitly out of scope for the bootstrap version of this system.
fn attractiveness(average_wealth: f64, unemployment_rate: f64) -> f64 {
    average_wealth * (1.0 - unemployment_rate).max(0.0)
}

/// Run one monthly migration pass over every city in the sector.
///
/// A deterministic pressure formula moves a small population share from
/// below-average-attractiveness cities into above-average ones. Total
/// population across the sector is exactly conserved: every person who
/// leaves a city in this pass arrives in another city in the same pass, so
/// calling this never changes the sector-wide population sum.
pub fn run_monthly_migration(sector: &mut Sector) {
    // Pass 1: snapshot each city's (size, attractiveness) in a fixed,
    // deterministic order (systems, then planets, then countries, then
    // cities) that pass 2 below re-walks identically, so per-city amounts
    // can be computed from a stable read-only view before anything moves.
    let snapshot: Vec<(u64, f64)> = sector
        .systems
        .iter()
        .flat_map(|s| &s.planets)
        .flat_map(|p| &p.countries)
        .flat_map(|c| &c.cities)
        .map(|city| {
            (
                city.population.size,
                attractiveness(
                    city.population.average_wealth,
                    city.population.unemployment_rate,
                ),
            )
        })
        .collect();

    if snapshot.is_empty() {
        return;
    }

    let total_population: u64 = snapshot.iter().map(|(size, _)| *size).sum();
    if total_population == 0 {
        return;
    }

    let weighted_attractiveness: f64 = snapshot
        .iter()
        .map(|(size, score)| *size as f64 * score)
        .sum();
    let mean_attractiveness = weighted_attractiveness / total_population as f64;
    // A non-positive or non-finite mean (every city has zero wealth, or
    // corrupt data slipped past invariants) leaves "relative shortfall"
    // undefined; skip the pass rather than divide by zero or NaN.
    if !mean_attractiveness.is_finite() || mean_attractiveness <= 0.0 {
        return;
    }

    // Outflow: cities below the mean lose a share proportional to how far
    // below it they are, capped so migration stays a slow drift. Each
    // amount is floored to whole people and capped at the city's own
    // population, so no city can be driven negative.
    let outflow: Vec<u64> = snapshot
        .iter()
        .map(|(size, score)| {
            let relative_shortfall = (mean_attractiveness - score) / mean_attractiveness;
            if relative_shortfall <= 0.0 {
                return 0;
            }
            let share = (relative_shortfall * MIGRATION_PRESSURE_SENSITIVITY)
                .min(MAX_MONTHLY_OUTFLOW_SHARE);
            let amount = (*size as f64 * share).floor() as u64;
            amount.min(*size)
        })
        .collect();
    let total_outflow: u64 = outflow.iter().sum();

    if total_outflow == 0 {
        return;
    }

    // Inflow: place the exact number of people who left, proportional to
    // each city's positive pressure above the mean, with any leftover from
    // integer flooring going to the single most attractive city
    // (deterministic: the first city attaining the maximum pressure in
    // iteration order) so the total placed always equals `total_outflow`
    // and the sector-wide sum is exactly conserved.
    let positive_pressure: Vec<f64> = snapshot
        .iter()
        .map(|(_, score)| (score - mean_attractiveness).max(0.0))
        .collect();
    let total_positive_pressure: f64 = positive_pressure.iter().sum();

    let mut inflow = vec![0u64; snapshot.len()];
    if total_positive_pressure > 0.0 {
        let mut placed: u64 = 0;
        for (i, pressure) in positive_pressure.iter().enumerate() {
            if *pressure <= 0.0 {
                continue;
            }
            let share = total_outflow as f64 * pressure / total_positive_pressure;
            let amount = share.floor() as u64;
            inflow[i] = amount;
            placed += amount;
        }
        let remainder = total_outflow - placed;
        if remainder > 0 {
            if let Some(best) = (0..positive_pressure.len())
                .filter(|&i| positive_pressure[i] > 0.0)
                .max_by(|&a, &b| positive_pressure[a].total_cmp(&positive_pressure[b]))
            {
                inflow[best] += remainder;
            }
        }
    }

    // Pass 2: apply in the same deterministic order as the snapshot.
    let mut index = 0;
    for system in &mut sector.systems {
        for planet in &mut system.planets {
            for country in &mut planet.countries {
                for city in &mut country.cities {
                    city.population.size = city
                        .population
                        .size
                        .saturating_sub(outflow[index])
                        .saturating_add(inflow[index]);
                    index += 1;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{
        City, CitySpecialization, Country, GovernmentProfile, Planet, PopulationGroup, ResourceTag,
        StarSystem,
    };
    use crate::worldgen::generate_sector;

    fn city(id: u32, name: &str, size: u64, average_wealth: f64, unemployment_rate: f64) -> City {
        City {
            id,
            name: name.to_string(),
            specialization: CitySpecialization::Mining,
            population: PopulationGroup {
                size,
                average_wealth,
                unemployment_rate,
            },
            treasury: 0.0,
            recent_output_index: 1.0,
        }
    }

    fn sector_with_two_cities(rich: City, poor: City) -> Sector {
        Sector {
            seed: 1,
            name: "Test Sector".to_string(),
            systems: vec![StarSystem {
                id: 1,
                name: "Test System".to_string(),
                planets: vec![Planet {
                    id: 1,
                    name: "Test Planet".to_string(),
                    resource_tags: vec![ResourceTag::Agricultural],
                    resource_abundance: Vec::new(),
                    countries: vec![Country {
                        id: 1,
                        name: "Test Country".to_string(),
                        government: GovernmentProfile {
                            federalism: 0.5,
                            franchise: 0.5,
                            economic_liberalism: 0.5,
                            press_freedom: 0.5,
                            legislative_strength: 0.5,
                            judicial_independence: 0.5,
                        },
                        backstory: "Test backstory.".to_string(),
                        social_mobility: 0.5,
                        union_power: 0.5,
                        cities: vec![rich, poor],
                    }],
                }],
            }],
        }
    }

    fn total_population(sector: &Sector) -> u64 {
        sector
            .systems
            .iter()
            .flat_map(|s| &s.planets)
            .flat_map(|p| &p.countries)
            .flat_map(|c| &c.cities)
            .map(|city| city.population.size)
            .sum()
    }

    #[test]
    fn migration_moves_people_from_the_poorer_city_to_the_richer_one() {
        let rich = city(1, "Richford", 10_000, 5_000.0, 0.02);
        let poor = city(2, "Poorton", 10_000, 100.0, 0.30);
        let mut sector = sector_with_two_cities(rich, poor);

        run_monthly_migration(&mut sector);

        let sizes: Vec<u64> = sector
            .systems
            .iter()
            .flat_map(|s| &s.planets)
            .flat_map(|p| &p.countries)
            .flat_map(|c| &c.cities)
            .map(|city| city.population.size)
            .collect();

        assert!(sizes[0] > 10_000, "richer city should have gained people");
        assert!(sizes[1] < 10_000, "poorer city should have lost people");
    }

    #[test]
    fn migration_conserves_total_population_across_many_months() {
        let mut sector = generate_sector(2024, 3);
        let before = total_population(&sector);
        for _ in 0..120 {
            run_monthly_migration(&mut sector);
        }
        let after = total_population(&sector);
        assert_eq!(
            before, after,
            "migration alone must never change the sector-wide total"
        );
    }

    #[test]
    fn migration_keeps_every_population_group_valid() {
        let mut sector = generate_sector(4242, 3);
        for _ in 0..120 {
            run_monthly_migration(&mut sector);
            for system in &sector.systems {
                for planet in &system.planets {
                    for country in &planet.countries {
                        for city in &country.cities {
                            assert!(
                                city.population.is_valid(),
                                "invalid population in {}",
                                city.name
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn migration_is_a_pure_function_of_sector_state_and_so_is_deterministic() {
        let run = || {
            let mut sector = generate_sector(77, 2);
            for _ in 0..24 {
                run_monthly_migration(&mut sector);
            }
            serde_json::to_string(&sector).unwrap()
        };
        assert_eq!(run(), run());
    }

    #[test]
    fn an_even_sector_has_no_migration_pressure() {
        // Every city identical: no one is above or below the mean, so
        // nothing should move.
        let a = city(1, "Alpha", 5_000, 1_000.0, 0.1);
        let b = city(2, "Beta", 5_000, 1_000.0, 0.1);
        let mut sector = sector_with_two_cities(a, b);

        run_monthly_migration(&mut sector);

        let sizes: Vec<u64> = sector
            .systems
            .iter()
            .flat_map(|s| &s.planets)
            .flat_map(|p| &p.countries)
            .flat_map(|c| &c.cities)
            .map(|city| city.population.size)
            .collect();
        assert_eq!(sizes, vec![5_000, 5_000]);
    }

    #[test]
    fn a_sector_with_no_wealth_anywhere_has_no_migration_pressure() {
        // Mean attractiveness is zero, so the pass must bail out rather
        // than divide by zero.
        let a = city(1, "Alpha", 5_000, 0.0, 0.1);
        let b = city(2, "Beta", 5_000, 0.0, 0.1);
        let mut sector = sector_with_two_cities(a, b);

        run_monthly_migration(&mut sector);

        let sizes: Vec<u64> = sector
            .systems
            .iter()
            .flat_map(|s| &s.planets)
            .flat_map(|p| &p.countries)
            .flat_map(|c| &c.cities)
            .map(|city| city.population.size)
            .collect();
        assert_eq!(sizes, vec![5_000, 5_000]);
    }
}
