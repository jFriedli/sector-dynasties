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
}

impl GovernmentProfile {
    /// A short human-readable label derived from the component values.
    /// This is a placeholder classifier for the bootstrap slice, not a
    /// final taxonomy.
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
        format!("{structure} {economy} Republic")
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
