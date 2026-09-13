//! Sector Dynasties simulation core.
//!
//! Pure simulation logic: no rendering, no file I/O, no network. This crate
//! must build and run headlessly, and must build to `wasm32-unknown-unknown`
//! unmodified. See docs/ARCHITECTURE.md for the module map and the reasoning
//! behind this boundary.

pub mod birth;
pub mod business;
pub mod career;
pub mod career_ai;
pub mod culture;
pub mod dynasty;
pub mod economy;
pub mod events;
pub mod favor;
pub mod history;
pub mod invariants;
pub mod migration;
pub mod mortality;
pub mod portrait;
pub mod presets;
pub mod rng;
pub mod save;
pub mod state;
pub mod time;
pub mod trade;
pub mod traits;
pub mod world;
pub mod worldgen;

pub use rng::RngDomainSummary;
pub use save::SaveError;
pub use state::{SimState, StateSummary};
