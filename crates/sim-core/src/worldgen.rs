//! Seeded, reproducible generation of a small starting sector.
//!
//! This is intentionally small: a handful of systems/planets/countries/
//! cities, enough to prove the hierarchy and give the economy something to
//! act on. See issues under the `worldgen` label for growing this into
//! real procedural history.

use crate::rng::SimRng;
use crate::world::{
    City, CitySpecialization, Country, GovernmentProfile, Planet, PopulationGroup, Sector,
    StarSystem,
};

const SYSTEM_NAMES: &[&str] = &["Kestrel", "Vantar", "Oridine", "Halcyon"];
const PLANET_NAMES: &[&str] = &["Prime", "Secundus", "Tertia"];
const COUNTRY_NAMES: &[&str] = &[
    "Meridian Compact",
    "Free Cities of Oridine",
    "Kestrel Union",
];
const CITY_NAMES: &[&str] = &[
    "New Halcyon",
    "Port Adren",
    "Ferrous Hold",
    "Lowmarket",
    "Stillwater",
];

const SPECIALIZATIONS: &[CitySpecialization] = &[
    CitySpecialization::Mining,
    CitySpecialization::Manufacturing,
    CitySpecialization::Finance,
    CitySpecialization::Research,
    CitySpecialization::Logistics,
];

/// Generate a sector deterministically from `seed`. Calling this twice with
/// the same seed always produces an identical `Sector`.
pub fn generate_sector(seed: u64, system_count: u32) -> Sector {
    let mut rng = SimRng::from_seed(seed, "worldgen");
    let mut next_id: u32 = 1;

    let systems = (0..system_count)
        .map(|i| generate_system(&mut rng, &mut next_id, i))
        .collect();

    Sector {
        seed,
        name: "The Meridian Sector".to_string(),
        systems,
    }
}

fn generate_system(rng: &mut SimRng, next_id: &mut u32, index: u32) -> StarSystem {
    let id = take_id(next_id);
    let name = name_for(SYSTEM_NAMES, index as usize, id);
    let planet_count = 1 + rng.next_below(2); // 1-2 planets per system in the bootstrap slice

    let planets = (0..planet_count)
        .map(|_| generate_planet(rng, next_id))
        .collect();

    StarSystem { id, name, planets }
}

fn generate_planet(rng: &mut SimRng, next_id: &mut u32) -> Planet {
    let id = take_id(next_id);
    let name = name_for(PLANET_NAMES, rng.pick_index(PLANET_NAMES.len()), id);
    let country_count = 1 + rng.next_below(2);

    let countries = (0..country_count)
        .map(|_| generate_country(rng, next_id))
        .collect();

    Planet {
        id,
        name,
        countries,
    }
}

fn generate_country(rng: &mut SimRng, next_id: &mut u32) -> Country {
    let id = take_id(next_id);
    let name = name_for(COUNTRY_NAMES, rng.pick_index(COUNTRY_NAMES.len()), id);
    let government = GovernmentProfile {
        federalism: rng.next_f64(),
        franchise: rng.next_f64(),
        economic_liberalism: rng.next_f64(),
        press_freedom: rng.next_f64(),
    };
    let city_count = 1 + rng.next_below(3);

    let cities = (0..city_count)
        .map(|_| generate_city(rng, next_id))
        .collect();

    Country {
        id,
        name,
        government,
        cities,
    }
}

fn generate_city(rng: &mut SimRng, next_id: &mut u32) -> City {
    let id = take_id(next_id);
    let name = name_for(CITY_NAMES, rng.pick_index(CITY_NAMES.len()), id);
    let specialization = SPECIALIZATIONS[rng.pick_index(SPECIALIZATIONS.len())];
    let population = PopulationGroup {
        size: 50_000 + rng.next_below(2_000_000) as u64,
        average_wealth: rng.range_f64(500.0, 5_000.0),
        unemployment_rate: rng.range_f64(0.02, 0.15),
    };

    City {
        id,
        name,
        specialization,
        population,
        treasury: 0.0,
    }
}

fn take_id(next_id: &mut u32) -> u32 {
    let id = *next_id;
    *next_id += 1;
    id
}

fn name_for(pool: &[&str], index: usize, id: u32) -> String {
    let base = pool[index % pool.len()];
    if id as usize > pool.len() {
        format!("{base} {id}")
    } else {
        base.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_produces_an_identical_sector() {
        let a = generate_sector(1234, 3);
        let b = generate_sector(1234, 3);
        let json_a = serde_json::to_string(&a).unwrap();
        let json_b = serde_json::to_string(&b).unwrap();
        assert_eq!(json_a, json_b);
    }

    #[test]
    fn different_seeds_usually_diverge() {
        let a = generate_sector(1, 3);
        let b = generate_sector(2, 3);
        assert_ne!(
            serde_json::to_string(&a).unwrap(),
            serde_json::to_string(&b).unwrap()
        );
    }

    #[test]
    fn requested_system_count_is_respected() {
        let sector = generate_sector(99, 4);
        assert_eq!(sector.systems.len(), 4);
    }

    #[test]
    fn generated_population_groups_are_valid() {
        let sector = generate_sector(55, 3);
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
