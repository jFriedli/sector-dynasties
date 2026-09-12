//! Sector Dynasties simulation core.
//!
//! Pure simulation logic: no rendering, no file I/O, no network. This crate
//! must build and run headlessly, and must build to `wasm32-unknown-unknown`
//! unmodified. See docs/ARCHITECTURE.md for the module map and the reasoning
//! behind this boundary.

pub mod culture;
pub mod dynasty;
pub mod economy;
pub mod invariants;
pub mod rng;
pub mod save;
pub mod state;
pub mod time;
pub mod world;
pub mod worldgen;

pub use save::SaveError;
pub use state::{SimState, StateSummary};
