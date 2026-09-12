//! Development-time invariant checks. These are cheap enough to run every
//! tick in debug/test builds and should never fire in correct code; a
//! violation means a bug, not a game event.

use crate::state::SimState;

#[derive(Debug, PartialEq, Eq)]
pub struct InvariantViolation(pub String);

pub fn check_invariants(state: &SimState) -> Vec<InvariantViolation> {
    let mut violations = Vec::new();

    for system in &state.sector.systems {
        for planet in &system.planets {
            for country in &planet.countries {
                for city in &country.cities {
                    if !city.population.is_valid() {
                        violations.push(InvariantViolation(format!(
                            "city '{}' has an invalid population group: {:?}",
                            city.name, city.population
                        )));
                    }
                    if !city.treasury.is_finite() || city.treasury < 0.0 {
                        violations.push(InvariantViolation(format!(
                            "city '{}' has an invalid treasury: {}",
                            city.name, city.treasury
                        )));
                    }
                    if !city.recent_output_index.is_finite() || city.recent_output_index < 0.0 {
                        violations.push(InvariantViolation(format!(
                            "city '{}' has an invalid recent_output_index: {}",
                            city.name, city.recent_output_index
                        )));
                    }
                }
            }
        }
    }

    for character in &state.dynasty.members {
        if !character.is_valid() {
            violations.push(InvariantViolation(format!(
                "character '{}' (id {}) has invalid state",
                character.name, character.id
            )));
        }
    }

    if !state
        .dynasty
        .members
        .iter()
        .any(|c| c.id == state.dynasty.head_character_id)
    {
        violations.push(InvariantViolation(
            "dynasty head_character_id does not reference an existing member".to_string(),
        ));
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::SimState;

    #[test]
    fn a_freshly_generated_state_has_no_violations() {
        let state = SimState::new(12345);
        assert_eq!(check_invariants(&state), Vec::new());
    }

    #[test]
    fn a_dangling_dynasty_head_is_caught() {
        let mut state = SimState::new(1);
        state.dynasty.head_character_id = 999_999;
        let violations = check_invariants(&state);
        assert!(!violations.is_empty());
    }

    #[test]
    fn a_non_finite_recent_output_index_is_caught() {
        let mut state = SimState::new(2);
        state.sector.systems[0].planets[0].countries[0].cities[0].recent_output_index = f64::NAN;
        let violations = check_invariants(&state);
        assert!(!violations.is_empty());
    }
}
