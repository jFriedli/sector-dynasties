//! The Sector -> StarSystem -> Planet -> Country -> City hierarchy.
//!
//! This is deliberately shallow for the bootstrap slice: enough structure
//! to prove the hierarchy and let population/economy systems attach to a
//! `City`, without modeling orbits, borders, or geography in any depth.
//! See docs/ARCHITECTURE.md for the intended growth path.

use serde::{Deserialize, Serialize};

pub type EntityId = u32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sector {
    pub seed: u64,
    pub name: String,
    pub systems: Vec<StarSystem>,
}

impl Sector {
    /// Look up a city anywhere in the hierarchy by id. Returns `None` for a
    /// stale or unknown id rather than panicking, since callers that hold
    /// only an `EntityId` reference (e.g. `crate::business`) must treat a
    /// dangling reference as a recoverable condition, not a bug that
    /// crashes the tick.
    pub fn find_city(&self, id: EntityId) -> Option<&City> {
        self.systems
            .iter()
            .flat_map(|s| &s.planets)
            .flat_map(|p| &p.countries)
            .flat_map(|c| &c.cities)
            .find(|city| city.id == id)
    }

    /// Look up the country that hosts the city with the given id. Returns
    /// `None` for a stale or unknown city id, for the same reason as
    /// [`Sector::find_city`]: callers holding only an `EntityId` (e.g.
    /// `crate::business` reading a host country's `union_power`) must
    /// treat a dangling reference as recoverable, not a crash.
    pub fn find_country_for_city(&self, city_id: EntityId) -> Option<&Country> {
        self.systems
            .iter()
            .flat_map(|s| &s.planets)
            .flat_map(|p| &p.countries)
            .find(|country| country.cities.iter().any(|city| city.id == city_id))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarSystem {
    pub id: EntityId,
    pub name: String,
    pub planets: Vec<Planet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Planet {
    pub id: EntityId,
    pub name: String,
    /// Small set of resource tags biasing which `CitySpecialization`
    /// worldgen picks for this planet's cities. Intentionally a short,
    /// fixed list rather than a full resource system; see epic #4.
    pub resource_tags: Vec<ResourceTag>,
    /// This planet's resource-abundance table: exactly one entry per
    /// `ResourceKind::ALL`, giving how abundant that resource is here (see
    /// `ResourceAbundance`). This is the intended hook point for epic #4's
    /// production chains: a future production system reads this table to
    /// decide how much of a resource a planet's industry can draw on,
    /// rather than modeling extraction here.
    ///
    /// Complements, rather than replaces, `resource_tags` above: tags bias
    /// which city specialization worldgen picks, while this table gives
    /// numeric per-resource quantities for trade/production to consume. The
    /// two are generated in a related way (worldgen biases a planet's
    /// abundance rolls toward resources that match its tags, see
    /// `worldgen::abundance_bias`) but are not required to agree exactly.
    /// Defaults to an empty table for saves predating this field; such a
    /// save simply has no abundance data until it's regenerated.
    #[serde(default)]
    pub resource_abundance: Vec<ResourceAbundance>,
    pub countries: Vec<Country>,
}

/// A small fixed set of planet-level resource characteristics. Kept
/// deliberately short: this proves the worldgen hook (tags bias city
/// specialization odds) rather than modeling a full resource economy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceTag {
    MetalRich,
    Agricultural,
    Arid,
}

/// The fixed set of tradeable resources tracked per planet. Kept
/// deliberately short (three resources) to prove the abundance-table shape
/// rather than model a full resource economy; see epic #4 for growing this
/// into real production chains that consume `Planet::resource_abundance`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceKind {
    Ore,
    Hydrocarbons,
    RareElements,
}

impl ResourceKind {
    /// Every `ResourceKind` variant, in the fixed order worldgen generates
    /// `Planet::resource_abundance` rows and invariants check them. Adding a
    /// new resource is a matter of extending this array; every planet's
    /// table then picks up the new row on the next `generate_sector` call.
    pub const ALL: [ResourceKind; 3] = [
        ResourceKind::Ore,
        ResourceKind::Hydrocarbons,
        ResourceKind::RareElements,
    ];
}

/// One row of a planet's resource-abundance table: how much of `kind` this
/// planet has, as a fraction in `[0.0, 1.0]` (0 = none, 1 = exceptionally
/// rich). See `Planet::resource_abundance`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ResourceAbundance {
    pub kind: ResourceKind,
    pub abundance: f64,
}

impl ResourceAbundance {
    pub fn is_valid(&self) -> bool {
        self.abundance.is_finite() && (0.0..=1.0).contains(&self.abundance)
    }
}

/// A country's government is described by its components rather than a
/// single enum, per the architecture goal of building government out of
/// institutions and rules. The bootstrap slice only models a couple of
/// dimensions; more can be added without breaking existing data since each
/// field is independent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernmentProfile {
    /// 0.0 = fully centralized authority, 1.0 = fully distributed/federal.
    pub federalism: f64,
    /// 0.0 = no popular franchise, 1.0 = universal suffrage.
    pub franchise: f64,
    /// 0.0 = command economy, 1.0 = laissez-faire.
    pub economic_liberalism: f64,
    /// 0.0 = state/press fully controlled, 1.0 = fully free press.
    pub press_freedom: f64,
    /// How much real, independent power the legislature holds over the
    /// executive. `0.0` means the legislature (if one exists at all) is a
    /// rubber stamp with no ability to check the executive; `1.0` means a
    /// legislature that can meaningfully block, amend, or remove executive
    /// action. Independent of `federalism`: a centralized state can still
    /// have a strong national legislature, and a federal one can still
    /// have a weak one.
    #[serde(default = "default_legislative_strength")]
    pub legislative_strength: f64,
    /// How independent the judiciary is from the ruling power. `0.0` means
    /// courts are an arm of the executive/party and rule as directed;
    /// `1.0` means courts are fully independent and their rulings bind the
    /// state itself, including the executive.
    #[serde(default = "default_judicial_independence")]
    pub judicial_independence: f64,
}

/// Default for `GovernmentProfile::legislative_strength` on saves predating
/// this field: the midpoint, neither rubber-stamp nor fully empowered, so
/// an old save doesn't suddenly snap to either extreme.
fn default_legislative_strength() -> f64 {
    0.5
}

/// Default for `GovernmentProfile::judicial_independence` on saves
/// predating this field: the midpoint, neither captured nor fully
/// independent, so an old save doesn't suddenly snap to either extreme.
fn default_judicial_independence() -> f64 {
    0.5
}

impl GovernmentProfile {
    /// A short human-readable label derived from the component values.
    /// This is a placeholder classifier for the bootstrap slice, not a
    /// final taxonomy.
    ///
    /// `franchise` and `press_freedom` don't feed this label, same as
    /// before this dimension was added: they describe who gets a say and
    /// how freely they can speak, not the structural shape a name like
    /// "Federal Market Republic" is trying to capture. `legislative_strength`
    /// and `judicial_independence` do feed it, because a government with
    /// neither a legislature that can check the executive nor a judiciary
    /// independent of it has no real institutional constraint on power,
    /// regardless of how it distributes authority or runs its economy, and
    /// calling that a "Republic" would be misleading.
    pub fn display_name(&self) -> String {
        let structure = if self.federalism > 0.5 {
            "Federal"
        } else {
            "Centralized"
        };
        let economy = if self.economic_liberalism > 0.5 {
            "Market"
        } else {
            "Planned"
        };
        let regime = if self.legislative_strength < 0.35 && self.judicial_independence < 0.35 {
            "Autocracy"
        } else {
            "Republic"
        };
        format!("{structure} {economy} {regime}")
    }

    /// Every component is a finite fraction in `[0.0, 1.0]`. See
    /// `invariants::check_invariants`, which calls this for every country
    /// in the sector.
    pub fn is_valid(&self) -> bool {
        [
            self.federalism,
            self.franchise,
            self.economic_liberalism,
            self.press_freedom,
            self.legislative_strength,
            self.judicial_independence,
        ]
        .iter()
        .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Country {
    pub id: EntityId,
    pub name: String,
    pub government: GovernmentProfile,
    /// A short, deterministic founding-era backstory blurb (a founding
    /// event, an old conflict, a merger), generated by `crate::history`.
    /// Intentionally shallow: a real procedural history simulation is
    /// tracked under issue #78.
    pub backstory: String,
    /// How readily wealth gained by output flows into the broad population
    /// rather than staying concentrated. `0.0` means rigid: gains mostly
    /// stay concentrated at the top, so `PopulationGroup::average_wealth`
    /// (still just an average, not a full distribution) grows slowly.
    /// `1.0` means highly fluid: gains spread quickly into wages. See
    /// `economy::wage_share_fraction`, the calculation this currently
    /// drives; a fuller distribution model (percentiles, Gini-style
    /// measures) is future `population`-label work, not this bootstrap
    /// parameter.
    #[serde(default = "default_social_mobility")]
    pub social_mobility: f64,
    /// How much organized labor can raise a business's labor costs in this
    /// country. `0.0` means no organized labor (labor costs sit at a
    /// business archetype's baseline); `1.0` means fully organized, strong
    /// union power (labor costs rise toward their archetype-specific
    /// ceiling). See `crate::business::labor_cost_fraction`, the
    /// calculation this drives, for how it turns into a measurable
    /// difference in a business's net income.
    #[serde(default = "default_union_power")]
    pub union_power: f64,
    pub cities: Vec<City>,
}

/// Default for `Country::social_mobility` on saves predating this field:
/// the midpoint, neither rigid nor fluid, so an old save doesn't suddenly
/// snap to either extreme.
fn default_social_mobility() -> f64 {
    0.5
}

/// Default for `Country::union_power` on saves predating this field: the
/// midpoint, neither unorganized nor fully organized, so an old save
/// doesn't suddenly snap to either extreme.
fn default_union_power() -> f64 {
    0.5
}

/// Broad economic specialization. A city may lean into one or more of
/// these; the bootstrap slice keeps it to a single primary specialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CitySpecialization {
    Mining,
    Manufacturing,
    Finance,
    Research,
    Logistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct City {
    pub id: EntityId,
    pub name: String,
    pub specialization: CitySpecialization,
    pub population: PopulationGroup,
    /// Accumulated economic output, in abstract credits. Not tied to any
    /// per-unit cargo simulation; see docs/ARCHITECTURE.md Economy section.
    pub treasury: f64,
    /// Exponentially-weighted rolling measure of recent weekly output
    /// relative to the city's noise-free baseline (population,
    /// specialization multiplier, no random variance). `1.0` means output
    /// has been tracking baseline; sustained values below (above) `1.0`
    /// mean a depressed (booming) stretch, and `economy::settle_week` uses
    /// this to drive unemployment. Defaults to `1.0` for saves predating
    /// this field. See `crates/sim-core/src/economy.rs`.
    #[serde(default = "default_recent_output_index")]
    pub recent_output_index: f64,
}

/// Default for `City::recent_output_index` on saves from before this field
/// existed: "output has been at baseline," the same value newly generated
/// cities start with.
fn default_recent_output_index() -> f64 {
    1.0
}

/// Most inhabitants are represented statistically, never as individual
/// characters. See docs/GAME_DESIGN.md.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationGroup {
    pub size: u64,
    pub average_wealth: f64,
    pub unemployment_rate: f64,
}

impl PopulationGroup {
    pub fn is_valid(&self) -> bool {
        self.average_wealth.is_finite()
            && self.average_wealth >= 0.0
            && (0.0..=1.0).contains(&self.unemployment_rate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_profile() -> GovernmentProfile {
        GovernmentProfile {
            federalism: 0.5,
            franchise: 0.5,
            economic_liberalism: 0.5,
            press_freedom: 0.5,
            legislative_strength: 0.5,
            judicial_independence: 0.5,
        }
    }

    #[test]
    fn a_midpoint_government_profile_is_valid() {
        assert!(valid_profile().is_valid());
    }

    #[test]
    fn an_out_of_range_legislative_strength_is_invalid() {
        let mut profile = valid_profile();
        profile.legislative_strength = 1.5;
        assert!(!profile.is_valid());
    }

    #[test]
    fn a_non_finite_judicial_independence_is_invalid() {
        let mut profile = valid_profile();
        profile.judicial_independence = f64::NAN;
        assert!(!profile.is_valid());
    }

    #[test]
    fn weak_legislature_and_captured_courts_is_named_an_autocracy() {
        let mut profile = valid_profile();
        profile.federalism = 0.9;
        profile.economic_liberalism = 0.9;
        profile.legislative_strength = 0.1;
        profile.judicial_independence = 0.1;
        assert_eq!(profile.display_name(), "Federal Market Autocracy");
    }

    #[test]
    fn a_strong_legislature_alone_is_still_named_a_republic() {
        let mut profile = valid_profile();
        profile.federalism = 0.9;
        profile.economic_liberalism = 0.9;
        profile.legislative_strength = 0.9;
        profile.judicial_independence = 0.1;
        assert_eq!(profile.display_name(), "Federal Market Republic");
    }

    #[test]
    fn an_independent_judiciary_alone_is_still_named_a_republic() {
        let mut profile = valid_profile();
        profile.federalism = 0.9;
        profile.economic_liberalism = 0.9;
        profile.legislative_strength = 0.1;
        profile.judicial_independence = 0.9;
        assert_eq!(profile.display_name(), "Federal Market Republic");
    }
}
