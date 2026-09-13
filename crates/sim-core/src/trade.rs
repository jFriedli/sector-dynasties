//! Trade routes: generic capacity/distance/cost/reliability links between
//! two cities.
//!
//! This is the foundation multiple transport modes (#58), tariffs on
//! cross-country routes (#59), and a logistics business archetype that
//! operates routes (#60) build on; see epic #5 and `GAME_DESIGN.md`'s
//! Economy section ("production chains create real dependencies... prices
//! react to supply and demand"). Deliberately generic and narrow: no
//! transport mode, tariff, or goods-flow modeling here, just the route
//! shape and its baseline economics, per issue #57's scope.
//!
//! `sim-core` has no spatial coordinates for cities yet (see `crate::world`),
//! so [`TradeRoute::distance`] is a deliberately abstract, tier-based
//! quantity, in the same spirit as `world::ResourceTag`: a proxy good enough
//! to drive gameplay economics, not a geometry simulation. Two cities in the
//! same country get a short distance, two cities on the same planet but in
//! different countries get a medium one, and two cities on different
//! planets in the same star system get a long one. [`generate_trade_routes`]
//! never links cities across different star systems: interstellar trade is
//! out of scope for this slice.
//!
//! Route generation deliberately keeps the graph sparse rather than
//! connecting every possible city pair: every city trades directly with
//! every other city in its own country, and each country/planet/system tier
//! above that is bridged through a single most-populous "hub" city, the
//! same hub-and-spoke shape real regional trade tends to take. This still
//! guarantees every city has at least one route (a city is always in a
//! country with at least one city; see the tests below), without an O(n^2)
//! blowup across a whole sector.

use serde::{Deserialize, Serialize};

use crate::rng::SimRng;
use crate::world::{City, CitySpecialization, Country, EntityId, Sector};

/// Half-width of the reliability noise draw applied on top of a tier's base
/// reliability, so two routes of the same tier don't land on an identical
/// reliability value.
const RELIABILITY_NOISE_HALF_RANGE: f64 = 0.05;

/// How much a single `Logistics`-specialized endpoint adds to a route's
/// reliability, on top of its tier's base value. Applied once per matching
/// endpoint, so a route between two `Logistics` cities gets twice the bonus.
const LOGISTICS_RELIABILITY_BONUS: f64 = 0.08;

/// How much a single `Logistics`-specialized endpoint multiplies a route's
/// capacity by, on top of `1.0`. Applied once per matching endpoint, in the
/// same spirit as [`LOGISTICS_RELIABILITY_BONUS`].
const LOGISTICS_CAPACITY_BONUS: f64 = 0.25;

/// Baseline weekly capacity (in abstract goods units) per unit of average
/// endpoint population. Tuned so a mid-size bootstrap-slice city pairing
/// lands in the low hundreds, comfortably above [`MIN_CAPACITY`].
const CAPACITY_PER_CAPITA: f64 = 0.00005;

/// A floor on generated capacity so even a pairing of very small cities
/// still has a route worth modeling.
const MIN_CAPACITY: f64 = 50.0;

/// Weekly cost per unit of distance at perfect reliability. Scaled up by
/// [`UNRELIABILITY_COST_MULTIPLIER`] as reliability drops below `1.0`.
const COST_PER_DISTANCE_UNIT: f64 = 0.05;

/// How strongly a shortfall in reliability drives up a route's cost: at
/// `reliability == 0.0` cost is `1.0 + UNRELIABILITY_COST_MULTIPLIER` times
/// the perfectly-reliable baseline, tapering to exactly the baseline at
/// `reliability == 1.0`.
const UNRELIABILITY_COST_MULTIPLIER: f64 = 1.5;

/// One trade route: a bidirectional link between two cities carrying
/// capacity, distance, cost, and reliability. See the module docs for how
/// these fields are generated and why `distance` is abstract rather than
/// geometric.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct TradeRoute {
    pub id: EntityId,
    pub city_a_id: EntityId,
    pub city_b_id: EntityId,
    /// Units of goods this route can move per week, before any transport
    /// mode (#58) or event modifies it.
    pub capacity: f64,
    /// Abstract distance; see the module docs.
    pub distance: f64,
    /// Weekly cost of running this route at capacity, before tariffs (#59)
    /// or a transport mode's own cost multiplier (#58).
    pub cost: f64,
    /// Fraction of shipments that arrive as planned, in `0.0..=1.0`.
    pub reliability: f64,
}

impl TradeRoute {
    /// A route is valid when its endpoints are two distinct cities and every
    /// numeric field is finite and within its documented range. This does
    /// *not* check that `city_a_id`/`city_b_id` resolve to real cities in
    /// any particular sector; that dangling-reference check belongs to
    /// `invariants::check_invariants`, which has a `Sector` to check against.
    pub fn is_valid(&self) -> bool {
        self.city_a_id != self.city_b_id
            && self.capacity.is_finite()
            && self.capacity >= 0.0
            && self.distance.is_finite()
            && self.distance > 0.0
            && self.cost.is_finite()
            && self.cost >= 0.0
            && self.reliability.is_finite()
            && (0.0..=1.0).contains(&self.reliability)
    }
}

/// How far apart two cities are in the sector hierarchy, which drives a
/// generated route's distance and baseline reliability tier. See the module
/// docs for why this stands in for real geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RouteTier {
    /// Two cities in the same country.
    Local,
    /// Two cities on the same planet, in different countries.
    Regional,
    /// Two cities in the same star system, on different planets.
    Interplanetary,
}

impl RouteTier {
    fn distance_range(&self) -> (f64, f64) {
        match self {
            RouteTier::Local => (40.0, 180.0),
            RouteTier::Regional => (300.0, 900.0),
            RouteTier::Interplanetary => (1_500.0, 6_000.0),
        }
    }

    fn base_reliability(&self) -> f64 {
        match self {
            RouteTier::Local => 0.92,
            RouteTier::Regional => 0.80,
            RouteTier::Interplanetary => 0.65,
        }
    }
}

/// Build one generated route between `city_a` and `city_b` at the given
/// `tier`, drawing from `rng` and assigning (and advancing) `next_id`.
fn make_route(
    rng: &mut SimRng,
    next_id: &mut EntityId,
    city_a: &City,
    city_b: &City,
    tier: RouteTier,
) -> TradeRoute {
    let id = *next_id;
    *next_id += 1;

    let (min_distance, max_distance) = tier.distance_range();
    let distance = rng.range_f64(min_distance, max_distance);

    let logistics_endpoints = [city_a, city_b]
        .iter()
        .filter(|city| city.specialization == CitySpecialization::Logistics)
        .count() as f64;

    let reliability_noise =
        rng.range_f64(-RELIABILITY_NOISE_HALF_RANGE, RELIABILITY_NOISE_HALF_RANGE);
    let reliability = (tier.base_reliability()
        + logistics_endpoints * LOGISTICS_RELIABILITY_BONUS
        + reliability_noise)
        .clamp(0.05, 0.99);

    let average_population = (city_a.population.size + city_b.population.size) as f64 / 2.0;
    let capacity_multiplier = 1.0 + logistics_endpoints * LOGISTICS_CAPACITY_BONUS;
    let capacity =
        (average_population * CAPACITY_PER_CAPITA * capacity_multiplier).max(MIN_CAPACITY);

    let cost = distance
        * COST_PER_DISTANCE_UNIT
        * (1.0 + (1.0 - reliability) * UNRELIABILITY_COST_MULTIPLIER);

    TradeRoute {
        id,
        city_a_id: city_a.id,
        city_b_id: city_b.id,
        capacity,
        distance,
        cost,
        reliability,
    }
}

/// The most populous city in `cities`, `None` for an empty slice. Used to
/// pick the "hub" city that represents a country (or, one level up, a
/// planet) when bridging up to the next tier.
fn most_populous_city(cities: &[City]) -> Option<&City> {
    cities.iter().max_by_key(|city| city.population.size)
}

/// Connect every pair of cities within `country` with a `Local`
/// route.
fn add_intra_country_routes(
    rng: &mut SimRng,
    next_id: &mut EntityId,
    country: &Country,
    routes: &mut Vec<TradeRoute>,
) {
    for i in 0..country.cities.len() {
        for j in (i + 1)..country.cities.len() {
            routes.push(make_route(
                rng,
                next_id,
                &country.cities[i],
                &country.cities[j],
                RouteTier::Local,
            ));
        }
    }
}

/// Generate a sector's trade routes deterministically: calling this twice
/// with the same `rng` state and `sector` content always produces the same
/// routes. Callers should draw `rng` from a stream seeded independently from
/// world generation's own domain (see `SimState::new_with_system_count`),
/// since trade route generation happens once, at sector creation, and never
/// needs to be replayed as part of resuming a save.
pub fn generate_trade_routes(rng: &mut SimRng, sector: &Sector) -> Vec<TradeRoute> {
    let mut routes = Vec::new();
    let mut next_id: EntityId = 1;

    for system in &sector.systems {
        let mut system_hub: Option<&City> = None;

        for planet in &system.planets {
            let mut planet_hub: Option<&City> = None;

            for country in &planet.countries {
                add_intra_country_routes(rng, &mut next_id, country, &mut routes);

                if let Some(country_hub) = most_populous_city(&country.cities) {
                    match planet_hub {
                        None => planet_hub = Some(country_hub),
                        Some(existing_hub) => routes.push(make_route(
                            rng,
                            &mut next_id,
                            existing_hub,
                            country_hub,
                            RouteTier::Regional,
                        )),
                    }
                }
            }

            if let Some(this_planet_hub) = planet_hub {
                match system_hub {
                    None => system_hub = Some(this_planet_hub),
                    Some(existing_hub) => routes.push(make_route(
                        rng,
                        &mut next_id,
                        existing_hub,
                        this_planet_hub,
                        RouteTier::Interplanetary,
                    )),
                }
            }
        }
    }

    routes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::worldgen::generate_sector;

    fn all_city_ids(sector: &Sector) -> Vec<EntityId> {
        sector
            .systems
            .iter()
            .flat_map(|s| &s.planets)
            .flat_map(|p| &p.countries)
            .flat_map(|c| &c.cities)
            .map(|city| city.id)
            .collect()
    }

    #[test]
    fn generated_routes_are_all_valid() {
        let sector = generate_sector(2024, 3);
        let mut rng = SimRng::from_seed(2024, "test:trade");
        let routes = generate_trade_routes(&mut rng, &sector);

        assert!(!routes.is_empty(), "expected at least a few routes");
        for route in &routes {
            assert!(route.is_valid(), "invalid route: {route:?}");
        }
    }

    #[test]
    fn every_route_endpoint_references_a_real_city() {
        let sector = generate_sector(55, 3);
        let mut rng = SimRng::from_seed(55, "test:trade");
        let routes = generate_trade_routes(&mut rng, &sector);
        let city_ids = all_city_ids(&sector);

        for route in &routes {
            assert!(
                city_ids.contains(&route.city_a_id),
                "route {} references unknown city_a_id {}",
                route.id,
                route.city_a_id
            );
            assert!(
                city_ids.contains(&route.city_b_id),
                "route {} references unknown city_b_id {}",
                route.id,
                route.city_b_id
            );
        }
    }

    #[test]
    fn every_city_appears_in_at_least_one_route() {
        // Every country generated by worldgen has at least one city, and
        // every country with more than one city gets intra-country routes;
        // a lone city in a country still gets bridged up as its country's
        // hub, so no city should ever end up with zero routes as long as
        // there's more than one city in the sector overall.
        let sector = generate_sector(77, 3);
        let mut rng = SimRng::from_seed(77, "test:trade");
        let routes = generate_trade_routes(&mut rng, &sector);

        let mut endpoints = std::collections::HashSet::new();
        for route in &routes {
            endpoints.insert(route.city_a_id);
            endpoints.insert(route.city_b_id);
        }

        for city_id in all_city_ids(&sector) {
            assert!(
                endpoints.contains(&city_id),
                "city {city_id} has no trade route at all"
            );
        }
    }

    #[test]
    fn route_ids_are_unique() {
        let sector = generate_sector(88, 3);
        let mut rng = SimRng::from_seed(88, "test:trade");
        let routes = generate_trade_routes(&mut rng, &sector);

        let ids: std::collections::HashSet<EntityId> = routes.iter().map(|r| r.id).collect();
        assert_eq!(
            ids.len(),
            routes.len(),
            "expected every route id to be unique"
        );
    }

    #[test]
    fn same_seed_generation_is_deterministic() {
        let run = || {
            let sector = generate_sector(99, 3);
            let mut rng = SimRng::from_seed(99, "test:trade");
            let routes = generate_trade_routes(&mut rng, &sector);
            serde_json::to_string(&routes).unwrap()
        };
        assert_eq!(run(), run());
    }

    #[test]
    fn different_seeds_usually_diverge() {
        let routes_for = |seed: u64| {
            let sector = generate_sector(seed, 3);
            let mut rng = SimRng::from_seed(seed, "test:trade");
            serde_json::to_string(&generate_trade_routes(&mut rng, &sector)).unwrap()
        };
        assert_ne!(routes_for(1), routes_for(2));
    }

    #[test]
    fn intra_country_routes_are_cheaper_and_more_reliable_than_intra_system_routes() {
        // A same-country pairing (short distance, high base reliability)
        // should come out clearly cheaper and more reliable than a
        // same-system, different-planet pairing (long distance, low base
        // reliability), holding population and specialization constant.
        let mut rng = SimRng::from_seed(1, "test:trade-tiers");
        let city_a = crate::world::City {
            id: 1,
            name: "A".to_string(),
            specialization: CitySpecialization::Mining,
            population: crate::world::PopulationGroup {
                size: 100_000,
                average_wealth: 100.0,
                unemployment_rate: 0.1,
            },
            treasury: 0.0,
            recent_output_index: 1.0,
        };
        let mut city_b = city_a.clone();
        city_b.id = 2;

        let mut next_id = 1;
        let local = make_route(&mut rng, &mut next_id, &city_a, &city_b, RouteTier::Local);
        let distant = make_route(
            &mut rng,
            &mut next_id,
            &city_a,
            &city_b,
            RouteTier::Interplanetary,
        );

        assert!(
            local.distance < distant.distance,
            "local {} should be shorter than distant {}",
            local.distance,
            distant.distance
        );
        assert!(
            local.reliability > distant.reliability,
            "local {} should be more reliable than distant {}",
            local.reliability,
            distant.reliability
        );
        assert!(
            local.cost < distant.cost,
            "local {} should be cheaper than distant {}",
            local.cost,
            distant.cost
        );
    }

    #[test]
    fn a_logistics_endpoint_raises_reliability_and_capacity_holding_the_tier_constant() {
        let base_city = |id: EntityId, spec: CitySpecialization| crate::world::City {
            id,
            name: "City".to_string(),
            specialization: spec,
            // Large enough that the resulting capacity clears `MIN_CAPACITY`
            // even before the logistics multiplier, so the multiplier's
            // effect is actually visible rather than swallowed by the floor.
            population: crate::world::PopulationGroup {
                size: 2_000_000,
                average_wealth: 100.0,
                unemployment_rate: 0.1,
            },
            treasury: 0.0,
            recent_output_index: 1.0,
        };

        let mining_a = base_city(1, CitySpecialization::Mining);
        let mining_b = base_city(2, CitySpecialization::Mining);
        let logistics_b = base_city(2, CitySpecialization::Logistics);

        let mut rng_plain = SimRng::from_seed(42, "test:trade-logistics");
        let mut rng_logistics = SimRng::from_seed(42, "test:trade-logistics");
        let mut next_id_plain = 1;
        let mut next_id_logistics = 1;

        let plain = make_route(
            &mut rng_plain,
            &mut next_id_plain,
            &mining_a,
            &mining_b,
            RouteTier::Local,
        );
        let with_logistics = make_route(
            &mut rng_logistics,
            &mut next_id_logistics,
            &mining_a,
            &logistics_b,
            RouteTier::Local,
        );

        assert!(with_logistics.reliability > plain.reliability);
        assert!(with_logistics.capacity > plain.capacity);
    }

    #[test]
    fn a_single_city_country_still_gets_bridged_into_a_route_when_another_country_exists() {
        // Two countries, one city each, on the same planet: no intra-country
        // routes are possible, but the hub-bridging step should still
        // connect the two lone cities to each other.
        let city = |id: EntityId| crate::world::City {
            id,
            name: format!("City {id}"),
            specialization: CitySpecialization::Mining,
            population: crate::world::PopulationGroup {
                size: 10_000,
                average_wealth: 100.0,
                unemployment_rate: 0.1,
            },
            treasury: 0.0,
            recent_output_index: 1.0,
        };
        let country = |id: EntityId, city_id: EntityId| crate::world::Country {
            id,
            name: format!("Country {id}"),
            government: crate::world::GovernmentProfile {
                federalism: 0.5,
                franchise: 0.5,
                economic_liberalism: 0.5,
                press_freedom: 0.5,
                legislative_strength: 0.5,
                judicial_independence: 0.5,
            },
            backstory: String::new(),
            social_mobility: 0.5,
            union_power: 0.5,
            cities: vec![city(city_id)],
        };
        let sector = Sector {
            seed: 1,
            name: "Test Sector".to_string(),
            systems: vec![crate::world::StarSystem {
                id: 1,
                name: "Test System".to_string(),
                planets: vec![crate::world::Planet {
                    id: 1,
                    name: "Test Planet".to_string(),
                    resource_tags: Vec::new(),
                    resource_abundance: Vec::new(),
                    countries: vec![country(1, 1), country(2, 2)],
                }],
            }],
        };

        let mut rng = SimRng::from_seed(1, "test:trade-single-city");
        let routes = generate_trade_routes(&mut rng, &sector);

        assert_eq!(routes.len(), 1, "expected exactly one bridging route");
        let route = routes[0];
        assert!(
            (route.city_a_id == 1 && route.city_b_id == 2)
                || (route.city_a_id == 2 && route.city_b_id == 1)
        );
    }
}
