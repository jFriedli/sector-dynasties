//! Businesses: entities the player's dynasty (or, eventually, an NPC
//! dynasty) can found and run for profit, distinct from a city's own
//! government treasury (`crate::economy`).
//!
//! This is deliberately the first, narrow slice of a much larger area (see
//! epic #6 and `GAME_DESIGN.md`'s economy section): one archetype, no
//! shares, no employees, no facilities, no mergers or bankruptcy. Per
//! `GAME_DESIGN.md`, different business archetypes should carry genuinely
//! different pressures rather than being a generic production building
//! with a different name, so [`BusinessArchetype`] carries its own required
//! host city specialization, revenue rate, labor sensitivity, and running
//! cost rather than sharing one formula; adding a second archetype
//! (logistics, finance, ...) means adding a match arm to each, not
//! touching the settlement loop.
//!
//! Net income is split into a labor cost (a fraction of revenue that grows
//! with the host country's `crate::world::Country::union_power`, see
//! [`labor_cost_fraction`]) and a fixed non-labor running cost (equipment,
//! permits, safety compliance) that has to be paid whether or not the
//! week's output was any good. This is the concrete hook proving union
//! power actually raises labor costs for a real settled business, rather
//! than a throwaway comparison function living alongside it.
//!
//! `sim-core` has no concept of a dynasty id yet (`SimState` only ever
//! holds one dynasty; see `crate::dynasty`), so a business's "owner
//! reference to the dynasty" is expressed as `owner_character_id`, the
//! dynasty member who owns it (in practice the dynasty head at founding
//! time). Net income flows into that character's `wealth`, which is what
//! `Dynasty::total_wealth` already sums, so this reference is the concrete
//! link to "the owning dynasty's wealth" until dynasties get their own id.

use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::dynasty::Dynasty;
use crate::world::{City, CitySpecialization, EntityId, Sector};

/// Revenue per resident of the host city, before labor and running costs
/// are deducted. Scaled well below `economy::BASE_OUTPUT_PER_CAPITA`: a
/// mining company only captures a slice of a city's output, not the whole
/// city's economy.
const MINING_REVENUE_PER_CAPITA: f64 = 0.006;

/// Mining's labor cost as a fraction of revenue at `union_power == 0.0`
/// (no organized labor). See [`labor_cost_fraction`] for how union power
/// scales this upward. Labor-heavy, matching a mining company's real cost
/// structure: wages already make up a large share of revenue even before
/// unionizing.
const MINING_BASE_LABOR_COST_FRACTION: f64 = 0.35;

/// How much larger an archetype's baseline labor cost fraction can grow,
/// proportionally, under fully organized labor (`union_power == 1.0`).
/// Scaling by the archetype's own baseline (rather than by its headroom to
/// `1.0`) means a labor-heavy archetype, which already spends more of its
/// revenue on wages, sees the larger *absolute* swing from unionizing, not
/// the smaller one. Chosen so the swing is clearly visible without ever
/// being able to push labor cost past 100% of revenue.
const UNION_POWER_LABOR_COST_BONUS: f64 = 0.4;

/// Weekly non-labor running cost for a mining company: equipment upkeep,
/// extraction permits, and safety compliance, on top of whatever labor
/// costs union power drives. This fixed floor is the "genuinely different
/// pressure" a mining company carries versus, say, a lower fixed-cost
/// archetype that trades a smaller floor for more volatile revenue:
/// chosen large enough that a small or badly-performing host city can push
/// a mining company into a losing week, not just a smaller profit.
const MINING_WEEKLY_RUNNING_COST: f64 = 45.0;

/// An archetype's labor cost as a fraction of revenue at the given
/// `union_power`. `union_power` is expected to already be within
/// `0.0..=1.0` (see `invariants.rs`), but the result is clamped defensively
/// here too so a corrupt value can't push the labor cost fraction outside a
/// sane range.
fn labor_cost_fraction(base_labor_cost_fraction: f64, union_power: f64) -> f64 {
    let union_power = union_power.clamp(0.0, 1.0);
    (base_labor_cost_fraction * (1.0 + union_power * UNION_POWER_LABOR_COST_BONUS)).clamp(0.0, 1.0)
}

/// A business archetype. Only one exists today; see the module docs for why
/// each new archetype gets its own match arm rather than a shared formula.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BusinessArchetype {
    Mining,
}

/// Failure parsing a [`BusinessArchetype`] from a string (the wasm bridge's
/// only caller, since UI callers pass a plain string across the boundary).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseBusinessArchetypeError;

impl FromStr for BusinessArchetype {
    type Err = ParseBusinessArchetypeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "Mining" | "mining" => Ok(Self::Mining),
            _ => Err(ParseBusinessArchetypeError),
        }
    }
}

impl BusinessArchetype {
    /// The city specialization a business of this archetype must be
    /// founded in. A mining company needs a city actually organized around
    /// extraction to have anything to sell.
    pub fn required_city_specialization(&self) -> CitySpecialization {
        match self {
            BusinessArchetype::Mining => CitySpecialization::Mining,
        }
    }

    fn revenue_per_capita(&self) -> f64 {
        match self {
            BusinessArchetype::Mining => MINING_REVENUE_PER_CAPITA,
        }
    }

    fn base_labor_cost_fraction(&self) -> f64 {
        match self {
            BusinessArchetype::Mining => MINING_BASE_LABOR_COST_FRACTION,
        }
    }

    fn weekly_running_cost(&self) -> f64 {
        match self {
            BusinessArchetype::Mining => MINING_WEEKLY_RUNNING_COST,
        }
    }

    /// This week's net income (revenue minus labor cost minus the fixed
    /// running cost) for a business of this archetype hosted in `city`,
    /// whose host country has the given `union_power`. Revenue scales with
    /// the host city's population and tracks `City::recent_output_index`,
    /// the same rolling measure `economy::settle_week` uses, so a
    /// business's fortunes rise and fall with its host city's actual
    /// performance rather than an independent dice roll; this keeps the
    /// settlement pure and deterministic given the current sector state,
    /// with no business-specific RNG stream needed.
    fn weekly_net_income(&self, city: &City, union_power: f64) -> f64 {
        let revenue =
            city.population.size as f64 * self.revenue_per_capita() * city.recent_output_index;
        let labor_cost =
            revenue * labor_cost_fraction(self.base_labor_cost_fraction(), union_power);
        revenue - labor_cost - self.weekly_running_cost()
    }
}

/// A founded business: an owner, a host city, and a running book value.
/// See the module docs for why ownership is expressed via
/// `owner_character_id` rather than a dynasty id.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Business {
    pub id: EntityId,
    pub name: String,
    pub archetype: BusinessArchetype,
    /// The dynasty member who owns this business; net weekly income flows
    /// into this character's `wealth`.
    pub owner_character_id: EntityId,
    /// The city this business operates out of. Looked up through
    /// `Sector::find_city` each settlement rather than holding a direct
    /// reference, per `docs/ARCHITECTURE.md`'s "reference by id" rule.
    pub host_city_id: EntityId,
    /// Retained book value: the running total of this business's own net
    /// income since founding, kept separately from what has already
    /// flowed out to the owner's `wealth`. Can go negative during a
    /// sustained loss (bankruptcy handling is out of scope for this slice,
    /// see epic #6); must always stay finite.
    pub equity: f64,
}

impl Business {
    pub fn is_valid(&self) -> bool {
        self.equity.is_finite()
    }
}

/// Why founding a business was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoundBusinessError {
    /// The host city's specialization doesn't match what this archetype
    /// requires.
    WrongCitySpecialization {
        archetype: BusinessArchetype,
        required: CitySpecialization,
        found: CitySpecialization,
    },
    /// `host_city_id` doesn't resolve to any city in the sector.
    UnknownHostCity(EntityId),
}

/// Found a new business of `archetype`, owned by `owner_character_id`, in
/// `host_city`. Fails if the host city's specialization doesn't match what
/// this archetype requires (see [`BusinessArchetype::required_city_specialization`]).
pub fn found_business(
    id: EntityId,
    name: String,
    archetype: BusinessArchetype,
    owner_character_id: EntityId,
    host_city: &City,
) -> Result<Business, FoundBusinessError> {
    let required = archetype.required_city_specialization();
    if host_city.specialization != required {
        return Err(FoundBusinessError::WrongCitySpecialization {
            archetype,
            required,
            found: host_city.specialization,
        });
    }

    Ok(Business {
        id,
        name,
        archetype,
        owner_character_id,
        host_city_id: host_city.id,
        equity: 0.0,
    })
}

/// Like [`found_business`], but takes a `host_city_id` and looks it up in
/// `sector` rather than requiring the caller to already hold a `&City`.
/// This is what `SimState::found_business` uses, since a caller working
/// from an id (e.g. the UI, or a save-loaded id) shouldn't need to walk the
/// sector hierarchy itself.
pub fn found_business_by_city_id(
    sector: &Sector,
    id: EntityId,
    name: String,
    archetype: BusinessArchetype,
    owner_character_id: EntityId,
    host_city_id: EntityId,
) -> Result<Business, FoundBusinessError> {
    let host_city = sector
        .find_city(host_city_id)
        .ok_or(FoundBusinessError::UnknownHostCity(host_city_id))?;
    found_business(id, name, archetype, owner_character_id, host_city)
}

/// Run one weekly settlement over every business: compute net income from
/// its host city and host country's union power, fold it into the
/// business's own `equity`, and pay it out to the owning character's
/// `wealth`. A business whose host city no longer resolves (a stale
/// reference, which invariants.rs also flags) is skipped rather than
/// panicking; a resolvable city with no resolvable host country (should
/// not happen outside a corrupt save) settles with `union_power` treated
/// as `0.0` rather than panicking.
pub fn settle_week(businesses: &mut [Business], sector: &Sector, dynasty: &mut Dynasty) {
    for business in businesses.iter_mut() {
        let Some(city) = sector.find_city(business.host_city_id) else {
            continue;
        };
        let union_power = sector
            .find_country_for_city(business.host_city_id)
            .map(|country| country.union_power)
            .unwrap_or(0.0);
        let net_income = business.archetype.weekly_net_income(city, union_power);
        business.equity += net_income;

        if let Some(owner) = dynasty
            .members
            .iter_mut()
            .find(|character| character.id == business.owner_character_id)
        {
            owner.wealth += net_income;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dynasty::Character;
    use crate::portrait::PortraitDescriptor;
    use crate::world::{Country, GovernmentProfile, Planet, PopulationGroup, StarSystem};
    use crate::worldgen::generate_sector;

    fn city_with(specialization: CitySpecialization, size: u64) -> City {
        City {
            id: 1,
            name: "Test City".to_string(),
            specialization,
            population: PopulationGroup {
                size,
                average_wealth: 100.0,
                unemployment_rate: 0.1,
            },
            treasury: 0.0,
            recent_output_index: 1.0,
        }
    }

    fn owner() -> Character {
        Character {
            id: 7,
            name: "Owner".to_string(),
            age_years: 40,
            alive: true,
            wealth: 1_000.0,
            home_city: Some(1),
            portrait: PortraitDescriptor::generate_for_character(0, 7),
            traits: Vec::new(),
        }
    }

    /// A minimal one-city sector hierarchy so `Sector::find_city` and
    /// `Sector::find_country_for_city` resolve `city`, with the given
    /// `union_power` on its host country.
    fn sector_with(city: City, union_power: f64) -> Sector {
        Sector {
            seed: 1,
            name: "Test Sector".to_string(),
            systems: vec![StarSystem {
                id: 1,
                name: "Test System".to_string(),
                planets: vec![Planet {
                    id: 1,
                    name: "Test Planet".to_string(),
                    resource_tags: Vec::new(),
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
                        backstory: String::new(),
                        social_mobility: 0.5,
                        union_power,
                        cities: vec![city],
                    }],
                }],
            }],
        }
    }

    #[test]
    fn founding_by_an_unknown_city_id_is_refused() {
        let sector = Sector {
            seed: 1,
            name: "Empty Sector".to_string(),
            systems: Vec::new(),
        };
        let err = found_business_by_city_id(
            &sector,
            1,
            "Ghost Mine".to_string(),
            BusinessArchetype::Mining,
            7,
            999,
        )
        .expect_err("an empty sector has no city 999 to found in");
        assert_eq!(err, FoundBusinessError::UnknownHostCity(999));
    }

    #[test]
    fn founding_in_a_mining_city_succeeds() {
        let city = city_with(CitySpecialization::Mining, 10_000);
        let business = found_business(
            1,
            "Ferrous Extraction Co.".to_string(),
            BusinessArchetype::Mining,
            7,
            &city,
        )
        .expect("mining company should found in a mining city");
        assert_eq!(business.host_city_id, city.id);
        assert_eq!(business.owner_character_id, 7);
        assert_eq!(business.equity, 0.0);
    }

    #[test]
    fn founding_in_a_non_mining_city_is_refused() {
        let city = city_with(CitySpecialization::Finance, 10_000);
        let err = found_business(
            1,
            "Ferrous Extraction Co.".to_string(),
            BusinessArchetype::Mining,
            7,
            &city,
        )
        .expect_err("mining company should refuse a non-mining host city");
        assert_eq!(
            err,
            FoundBusinessError::WrongCitySpecialization {
                archetype: BusinessArchetype::Mining,
                required: CitySpecialization::Mining,
                found: CitySpecialization::Finance,
            }
        );
    }

    #[test]
    fn a_populous_thriving_city_produces_positive_net_income_even_fully_unionized() {
        let city = city_with(CitySpecialization::Mining, 50_000);
        let net_income = BusinessArchetype::Mining.weekly_net_income(&city, 1.0);
        assert!(net_income > 0.0);
    }

    #[test]
    fn a_small_city_produces_a_loss_after_costs() {
        // A tiny host city's revenue can't cover the fixed running cost:
        // this is the "genuinely different pressure" a mining company
        // carries versus a lower-fixed-cost archetype.
        let city = city_with(CitySpecialization::Mining, 100);
        let net_income = BusinessArchetype::Mining.weekly_net_income(&city, 0.0);
        assert!(net_income < 0.0);
    }

    #[test]
    fn higher_union_power_lowers_net_income_holding_everything_else_constant() {
        let city = city_with(CitySpecialization::Mining, 50_000);
        let low_union = BusinessArchetype::Mining.weekly_net_income(&city, 0.0);
        let high_union = BusinessArchetype::Mining.weekly_net_income(&city, 1.0);

        assert!(
            high_union < low_union,
            "full union power ({high_union}) should not out-earn no organized \
             labor ({low_union}) holding the host city constant"
        );
    }

    #[test]
    fn labor_cost_fraction_clamps_a_corrupt_union_power_value() {
        let low = labor_cost_fraction(MINING_BASE_LABOR_COST_FRACTION, -5.0);
        let high = labor_cost_fraction(MINING_BASE_LABOR_COST_FRACTION, 5.0);
        assert_eq!(
            low,
            labor_cost_fraction(MINING_BASE_LABOR_COST_FRACTION, 0.0)
        );
        assert_eq!(
            high,
            labor_cost_fraction(MINING_BASE_LABOR_COST_FRACTION, 1.0)
        );
        assert!(high <= 1.0);
    }

    #[test]
    fn settlement_pays_net_income_into_the_owners_wealth_and_the_business_equity() {
        let city = city_with(CitySpecialization::Mining, 50_000);
        let sector = sector_with(city.clone(), 0.5);

        let mut dynasty = Dynasty {
            name: "House Test".to_string(),
            head_character_id: 7,
            members: vec![owner()],
        };
        let business = found_business(
            1,
            "Ferrous Extraction Co.".to_string(),
            BusinessArchetype::Mining,
            7,
            &city,
        )
        .unwrap();
        let expected_net_income = BusinessArchetype::Mining.weekly_net_income(&city, 0.5);
        let wealth_before = dynasty.members[0].wealth;

        let mut businesses = vec![business.clone()];
        settle_week(&mut businesses, &sector, &mut dynasty);
        let business = businesses.into_iter().next().unwrap();

        assert_eq!(business.equity, expected_net_income);
        assert_eq!(
            dynasty.members[0].wealth,
            wealth_before + expected_net_income
        );
    }

    #[test]
    fn settlement_skips_a_business_whose_host_city_no_longer_resolves() {
        let city = city_with(CitySpecialization::Mining, 50_000);
        let sector = Sector {
            seed: 1,
            name: "Empty Sector".to_string(),
            systems: Vec::new(),
        };
        let mut dynasty = Dynasty {
            name: "House Test".to_string(),
            head_character_id: 7,
            members: vec![owner()],
        };
        let business = found_business(
            1,
            "Ghost Mine".to_string(),
            BusinessArchetype::Mining,
            7,
            &city,
        )
        .unwrap();
        let mut businesses = vec![business];

        settle_week(&mut businesses, &sector, &mut dynasty);

        assert_eq!(businesses[0].equity, 0.0, "no host city, no income");
        assert_eq!(dynasty.members[0].wealth, 1_000.0, "owner wealth untouched");
    }

    #[test]
    fn same_seed_weekly_settlement_is_deterministic() {
        // No RNG of its own (see `weekly_net_income`'s docs), so this
        // mainly pins that settlement is a pure function of sector state:
        // two identical setups settle to identical results.
        let run = || {
            let sector = generate_sector(99, 2);
            let mining_city = sector
                .systems
                .iter()
                .flat_map(|s| &s.planets)
                .flat_map(|p| &p.countries)
                .flat_map(|c| &c.cities)
                .find(|c| c.specialization == CitySpecialization::Mining)
                .expect("seed 99 should generate at least one mining city")
                .clone();

            let mut dynasty = Dynasty {
                name: "House Test".to_string(),
                head_character_id: 7,
                members: vec![owner()],
            };
            let business = found_business(
                1,
                "Ferrous Extraction Co.".to_string(),
                BusinessArchetype::Mining,
                7,
                &mining_city,
            )
            .unwrap();
            let mut businesses = vec![business];
            settle_week(&mut businesses, &sector, &mut dynasty);
            (businesses[0].equity, dynasty.members[0].wealth)
        };

        assert_eq!(run(), run());
    }
}
