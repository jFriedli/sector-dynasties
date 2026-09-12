//! Simulation time. One tick equals one in-world day. Systems that run at
//! lower frequency (weekly economy settlement, monthly population updates,
//! yearly demographics) check the clock rather than being ticked separately,
//! so the world is never fully recomputed every tick.

use serde::{Deserialize, Serialize};

pub const DAYS_PER_WEEK: u64 = 7;
pub const DAYS_PER_MONTH: u64 = 30;
pub const DAYS_PER_YEAR: u64 = 360;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimClock {
    /// Total elapsed days since the simulation started.
    pub tick: u64,
}

impl SimClock {
    pub fn new() -> Self {
        SimClock { tick: 0 }
    }

    pub fn advance_one_day(&mut self) {
        self.tick += 1;
    }

    pub fn is_week_boundary(&self) -> bool {
        self.tick.is_multiple_of(DAYS_PER_WEEK)
    }

    pub fn is_month_boundary(&self) -> bool {
        self.tick.is_multiple_of(DAYS_PER_MONTH)
    }

    pub fn is_year_boundary(&self) -> bool {
        self.tick.is_multiple_of(DAYS_PER_YEAR)
    }

    pub fn year(&self) -> u64 {
        self.tick / DAYS_PER_YEAR
    }

    pub fn day_of_year(&self) -> u64 {
        self.tick % DAYS_PER_YEAR
    }
}

impl Default for SimClock {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundaries_trigger_at_expected_ticks() {
        let mut clock = SimClock::new();
        assert!(clock.is_week_boundary());
        assert!(clock.is_month_boundary());
        assert!(clock.is_year_boundary());

        for _ in 0..6 {
            clock.advance_one_day();
        }
        assert!(!clock.is_week_boundary());

        for _ in 0..1 {
            clock.advance_one_day();
        }
        assert!(clock.is_week_boundary());
    }

    #[test]
    fn year_and_day_of_year_derive_from_tick() {
        let mut clock = SimClock::new();
        for _ in 0..(DAYS_PER_YEAR + 5) {
            clock.advance_one_day();
        }
        assert_eq!(clock.year(), 1);
        assert_eq!(clock.day_of_year(), 5);
    }
}
