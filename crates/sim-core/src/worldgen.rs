//! Seeded, reproducible generation of a small starting sector.
//!
//! This is intentionally small: a handful of systems/planets/countries/
//! cities, enough to prove the hierarchy and give the economy something to
//! act on. See issues under the `worldgen` label for growing this into
//! real procedural history.

use crate::history;
use crate::rng::SimRng;
use crate::world::{
    City, CitySpecialization, Country, GovernmentProfile, Planet, PopulationGroup, ResourceTag,
    Sector, StarSystem,
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

const RESOURCE_TAGS: &[ResourceTag] = &[
    ResourceTag::MetalRich,
    ResourceTag::Agricultural,
    ResourceTag::Arid,
];

/// Relative bias a planet's resource tag applies to a given specialization
/// when worldgen picks a city's primary specialization. `1.0` is neutral
/// (no bias); anything higher makes that pairing more likely. Kept as a
/// small explicit table rather than a formula so the intent per tag stays
/// readable; see epic #4 for growing this into a fuller resource system.
fn specialization_bias(tag: ResourceTag, spec: CitySpecialization) -> f64 {
    match (tag, spec) {
        (ResourceTag::MetalRich, CitySpecialization::Mining) => 4.0,
        (ResourceTag::MetalRich, CitySpecialization::Manufacturing) => 2.0,
        (ResourceTag::Agricultural, CitySpecialization::Logistics) => 3.0,
        (ResourceTag::Agricultural, CitySpecialization::Manufacturing) => 1.5,
        (ResourceTag::Arid, CitySpecialization::Mining) => 2.0,
        (ResourceTag::Arid, CitySpecialization::Research) => 2.0,
        _ => 1.0,
    }
}

/// Pick a city's primary specialization, weighted by the planet's resource
/// tags. With no tags this reduces to a uniform pick across
/// `SPECIALIZATIONS`, matching the prior behavior.
fn pick_specialization(rng: &mut SimRng, tags: &[ResourceTag]) -> CitySpecialization {
    let weights: Vec<f64> = SPECIALIZATIONS
        .iter()
        .map(|&spec| {
            tags.iter()
                .map(|&tag| specialization_bias(tag, spec))
                .product::<f64>()
                .max(f64::MIN_POSITIVE)
        })
        .collect();
    SPECIALIZATIONS[rng.weighted_index(&weights)]
}

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
    let resource_tags = generate_resource_tags(rng);
    let country_count = 1 + rng.next_below(2);

    let countries = (0..country_count)
        .map(|_| generate_country(rng, next_id, &resource_tags))
        .collect();

    Planet {
        id,
        name,
        resource_tags,
        countries,
    }
}

/// A planet gets exactly one primary resource tag in the bootstrap slice.
/// The pick-and-bias machinery downstream (`pick_specialization`) already
/// supports a planet carrying several tags; growing the count here later
/// (e.g. rare dual-tag planets) needs no change beyond this function.
fn generate_resource_tags(rng: &mut SimRng) -> Vec<ResourceTag> {
    vec![RESOURCE_TAGS[rng.pick_index(RESOURCE_TAGS.len())]]
}

fn generate_country(rng: &mut SimRng, next_id: &mut u32, planet_tags: &[ResourceTag]) -> Country {
    let id = take_id(next_id);
    let name = name_for(COUNTRY_NAMES, rng.pick_index(COUNTRY_NAMES.len()), id);
    let backstory = history::generate_backstory(rng, &name);
    let government = GovernmentProfile {
        federalism: rng.next_f64(),
        franchise: rng.next_f64(),
        economic_liberalism: rng.next_f64(),
        press_freedom: rng.next_f64(),
    };
    let city_count = 1 + rng.next_below(3);

    let cities = (0..city_count)
        .map(|_| generate_city(rng, next_id, planet_tags))
        .collect();

    Country {
        id,
        name,
        government,
        backstory,
        cities,
    }
}

fn generate_city(rng: &mut SimRng, next_id: &mut u32, planet_tags: &[ResourceTag]) -> City {
    let id = take_id(next_id);
    let name = name_for(CITY_NAMES, rng.pick_index(CITY_NAMES.len()), id);
    let specialization = pick_specialization(rng, planet_tags);
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
        recent_output_index: 1.0,
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

    #[test]
    fn every_planet_has_at_least_one_resource_tag() {
        let sector = generate_sector(2024, 4);
        for system in &sector.systems {
            for planet in &system.planets {
                assert!(
                    !planet.resource_tags.is_empty(),
                    "planet {} has no resource tags",
                    planet.name
                );
            }
        }
    }

    fn mining_share(tags: &[ResourceTag], samples: usize, seed: u64) -> f64 {
        let mut rng = SimRng::from_seed(seed, "test:specialization-bias");
        let mut next_id: u32 = 1;
        let mining_count = (0..samples)
            .filter(|_| {
                generate_city(&mut rng, &mut next_id, tags).specialization
                    == CitySpecialization::Mining
            })
            .count();
        mining_count as f64 / samples as f64
    }

    #[test]
    fn metal_rich_planets_are_biased_toward_mining_specialization() {
        const SAMPLES: usize = 5_000;

        // With 5 specializations and no tags, Mining should land close to a
        // uniform 20% share.
        let baseline = mining_share(&[], SAMPLES, 1);
        assert!(
            (0.15..=0.25).contains(&baseline),
            "expected an ~uniform Mining share with no tags, got {baseline:.3}"
        );

        // A metal-rich tag biases toward Mining (weight 4 vs. 1, alongside
        // a smaller Manufacturing bump), for an expected share around
        // 4/9 ~= 0.44. Statistical, not exact: the assertion leaves a wide
        // margin both above chance and below the expected value.
        let biased = mining_share(&[ResourceTag::MetalRich], SAMPLES, 2);
        assert!(
            biased > 0.35,
            "expected metal-rich planets to favor Mining, got {biased:.3}"
        );
        assert!(
            biased > baseline * 1.5,
            "expected metal-rich Mining share ({biased:.3}) to clear the \
             untagged baseline ({baseline:.3}) by a wide margin"
        );
    }

    #[test]
    fn same_seed_produces_the_same_resource_tags() {
        let a = generate_sector(777, 3);
        let b = generate_sector(777, 3);
        let tags_a: Vec<Vec<ResourceTag>> = a
            .systems
            .iter()
            .flat_map(|s| s.planets.iter().map(|p| p.resource_tags.clone()))
            .collect();
        let tags_b: Vec<Vec<ResourceTag>> = b
            .systems
            .iter()
            .flat_map(|s| s.planets.iter().map(|p| p.resource_tags.clone()))
            .collect();
        assert_eq!(tags_a, tags_b);
    }

    fn country_backstories(sector: &Sector) -> Vec<String> {
        sector
            .systems
            .iter()
            .flat_map(|s| s.planets.iter())
            .flat_map(|p| p.countries.iter())
            .map(|c| c.backstory.clone())
            .collect()
    }

    #[test]
    fn same_seed_produces_the_same_country_backstories() {
        let a = generate_sector(4040, 3);
        let b = generate_sector(4040, 3);
        assert_eq!(country_backstories(&a), country_backstories(&b));
    }

    #[test]
    fn every_country_gets_a_non_empty_backstory_naming_itself() {
        let sector = generate_sector(4141, 4);
        for system in &sector.systems {
            for planet in &system.planets {
                for country in &planet.countries {
                    assert!(
                        !country.backstory.is_empty(),
                        "country {} has an empty backstory",
                        country.name
                    );
                    assert!(
                        country.backstory.starts_with(&country.name),
                        "expected {}'s backstory to start with its own name, got: {}",
                        country.name,
                        country.backstory
                    );
                }
            }
        }
    }
}
