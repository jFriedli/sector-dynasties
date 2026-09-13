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
            for row in &planet.resource_abundance {
                if !row.is_valid() {
                    violations.push(InvariantViolation(format!(
                        "planet '{}' has an invalid resource_abundance row: {row:?}",
                        planet.name
                    )));
                }
            }
            for country in &planet.countries {
                if !country.government.is_valid() {
                    violations.push(InvariantViolation(format!(
                        "country '{}' has an invalid government profile: {:?}",
                        country.name, country.government
                    )));
                }
                if !country.social_mobility.is_finite()
                    || !(0.0..=1.0).contains(&country.social_mobility)
                {
                    violations.push(InvariantViolation(format!(
                        "country '{}' has an invalid social_mobility: {}",
                        country.name, country.social_mobility
                    )));
                }
                if !country.union_power.is_finite() || !(0.0..=1.0).contains(&country.union_power) {
                    violations.push(InvariantViolation(format!(
                        "country '{}' has an invalid union_power: {}",
                        country.name, country.union_power
                    )));
                }
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

    for business in &state.businesses {
        if !business.is_valid() {
            violations.push(InvariantViolation(format!(
                "business '{}' (id {}) has an invalid equity: {}",
                business.name, business.id, business.equity
            )));
        }
        if !state
            .dynasty
            .members
            .iter()
            .any(|c| c.id == business.owner_character_id)
        {
            violations.push(InvariantViolation(format!(
                "business '{}' (id {}) owner_character_id {} does not reference an existing \
                 dynasty member",
                business.name, business.id, business.owner_character_id
            )));
        }
        if state.sector.find_city(business.host_city_id).is_none() {
            violations.push(InvariantViolation(format!(
                "business '{}' (id {}) host_city_id {} does not reference an existing city",
                business.name, business.id, business.host_city_id
            )));
        }
    }

    for career in &state.careers {
        if !career.is_valid() {
            violations.push(InvariantViolation(format!(
                "career (id {}) has an invalid level: {}",
                career.id, career.level
            )));
        }
        if !state
            .dynasty
            .members
            .iter()
            .any(|c| c.id == career.character_id)
        {
            violations.push(InvariantViolation(format!(
                "career (id {}) character_id {} does not reference an existing dynasty member",
                career.id, career.character_id
            )));
        }
        if state.sector.find_city(career.employer_city_id).is_none() {
            violations.push(InvariantViolation(format!(
                "career (id {}) employer_city_id {} does not reference an existing city",
                career.id, career.employer_city_id
            )));
        }
    }

    for route in &state.trade_routes {
        if !route.is_valid() {
            violations.push(InvariantViolation(format!(
                "trade route (id {}) has an invalid field: {route:?}",
                route.id
            )));
        }
        if state.sector.find_city(route.city_a_id).is_none() {
            violations.push(InvariantViolation(format!(
                "trade route (id {}) city_a_id {} does not reference an existing city",
                route.id, route.city_a_id
            )));
        }
        if state.sector.find_city(route.city_b_id).is_none() {
            violations.push(InvariantViolation(format!(
                "trade route (id {}) city_b_id {} does not reference an existing city",
                route.id, route.city_b_id
            )));
        }
    }

    for favor in &state.favors {
        if !favor.is_valid() {
            violations.push(InvariantViolation(format!(
                "favor from debtor {} to creditor {} has an invalid magnitude: {}",
                favor.debtor_id, favor.creditor_id, favor.magnitude
            )));
        }
        if !state
            .dynasty
            .members
            .iter()
            .any(|c| c.id == favor.creditor_id)
        {
            violations.push(InvariantViolation(format!(
                "favor creditor_id {} does not reference an existing dynasty member",
                favor.creditor_id
            )));
        }
        if !state
            .dynasty
            .members
            .iter()
            .any(|c| c.id == favor.debtor_id)
        {
            violations.push(InvariantViolation(format!(
                "favor debtor_id {} does not reference an existing dynasty member",
                favor.debtor_id
            )));
        }
    }

    for (i, pending) in state.pending_events.iter().enumerate() {
        if crate::events::definition(&pending.key).is_none() {
            violations.push(InvariantViolation(format!(
                "pending event (id {}) key '{}' does not reference a known event definition",
                pending.id, pending.key
            )));
        }
        if !state
            .dynasty
            .members
            .iter()
            .any(|c| c.id == pending.character_id)
        {
            violations.push(InvariantViolation(format!(
                "pending event (id {}) character_id {} does not reference an existing dynasty \
                 member",
                pending.id, pending.character_id
            )));
        }
        if state.pending_events[..i]
            .iter()
            .any(|other| other.key == pending.key && other.character_id == pending.character_id)
        {
            violations.push(InvariantViolation(format!(
                "duplicate pending event for key '{}' and character_id {}",
                pending.key, pending.character_id
            )));
        }
    }

    for resolved in &state.event_log.resolved {
        if crate::events::definition(&resolved.key).is_none() {
            violations.push(InvariantViolation(format!(
                "resolved event log entry key '{}' does not reference a known event definition",
                resolved.key
            )));
        }
        if !state
            .dynasty
            .members
            .iter()
            .any(|c| c.id == resolved.character_id)
        {
            violations.push(InvariantViolation(format!(
                "resolved event log entry character_id {} does not reference an existing dynasty \
                 member",
                resolved.character_id
            )));
        }
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

    #[test]
    fn an_out_of_range_government_component_is_caught() {
        let mut state = SimState::new(2);
        state.sector.systems[0].planets[0].countries[0]
            .government
            .franchise = 1.5;
        let violations = check_invariants(&state);
        assert!(!violations.is_empty());
    }

    #[test]
    fn a_non_finite_government_component_is_caught() {
        let mut state = SimState::new(2);
        state.sector.systems[0].planets[0].countries[0]
            .government
            .press_freedom = f64::NAN;
        let violations = check_invariants(&state);
        assert!(!violations.is_empty());
    }

    #[test]
    fn a_character_born_this_year_is_checked_the_same_as_any_other_member() {
        // Find a seed/year where the founder head actually has a child, so
        // this exercises a real birth rather than an empty no-op.
        let (seed, year) = (0..200u64)
            .find_map(|seed| {
                let mut state = SimState::new(seed);
                for year in 1..40u64 {
                    if crate::birth::maybe_birth_child(state.seed, &mut state.dynasty, year)
                        .is_some()
                    {
                        return Some((seed, year));
                    }
                }
                None
            })
            .expect("expected at least one seed to produce a birth within 40 years");

        let mut state = SimState::new(seed);
        let child_id = crate::birth::maybe_birth_child(state.seed, &mut state.dynasty, year)
            .expect("re-rolling the same seed and year must reproduce the same birth");

        // A freshly born character starts out valid, same as any other
        // member.
        assert_eq!(check_invariants(&state), Vec::new());

        // Corrupting *only* the new character (duplicate traits) is caught
        // by the same generic member check every other character goes
        // through, not a birth-specific carve-out.
        let child = state
            .dynasty
            .members
            .iter_mut()
            .find(|c| c.id == child_id)
            .unwrap();
        if let Some(first_trait) = child.traits.first().copied() {
            child.traits.push(first_trait);
        } else {
            child.traits.push(crate::traits::Trait::Ambitious);
            child.traits.push(crate::traits::Trait::Ambitious);
        }
        let violations = check_invariants(&state);
        assert!(
            violations
                .iter()
                .any(|v| v.0.contains(&format!("id {child_id}"))),
            "expected a violation naming the corrupted child, got {violations:?}"
        );
    }

    #[test]
    fn an_out_of_range_legislative_strength_is_caught() {
        let mut state = SimState::new(3);
        state.sector.systems[0].planets[0].countries[0]
            .government
            .legislative_strength = 1.5;
        let violations = check_invariants(&state);
        assert!(!violations.is_empty());
    }

    #[test]
    fn a_non_finite_judicial_independence_is_caught() {
        let mut state = SimState::new(4);
        state.sector.systems[0].planets[0].countries[0]
            .government
            .judicial_independence = f64::NAN;
        let violations = check_invariants(&state);
        assert!(!violations.is_empty());
    }

    #[test]
    fn an_out_of_range_social_mobility_is_caught() {
        let mut state = SimState::new(3);
        state.sector.systems[0].planets[0].countries[0].social_mobility = 1.5;
        let violations = check_invariants(&state);
        assert!(!violations.is_empty());
    }

    #[test]
    fn a_non_finite_social_mobility_is_caught() {
        let mut state = SimState::new(4);
        state.sector.systems[0].planets[0].countries[0].social_mobility = f64::NAN;
        let violations = check_invariants(&state);
        assert!(!violations.is_empty());
    }

    #[test]
    fn an_out_of_range_union_power_is_caught() {
        let mut state = SimState::new(5);
        state.sector.systems[0].planets[0].countries[0].union_power = 1.5;
        let violations = check_invariants(&state);
        assert!(!violations.is_empty());
    }

    #[test]
    fn a_non_finite_union_power_is_caught() {
        let mut state = SimState::new(6);
        state.sector.systems[0].planets[0].countries[0].union_power = f64::NAN;
        let violations = check_invariants(&state);
        assert!(!violations.is_empty());
    }

    #[test]
    fn an_out_of_range_resource_abundance_is_caught() {
        let mut state = SimState::new(7);
        state.sector.systems[0].planets[0].resource_abundance[0].abundance = 1.5;
        let violations = check_invariants(&state);
        assert!(!violations.is_empty());
    }

    #[test]
    fn a_non_finite_resource_abundance_is_caught() {
        let mut state = SimState::new(8);
        state.sector.systems[0].planets[0].resource_abundance[0].abundance = f64::NAN;
        let violations = check_invariants(&state);
        assert!(!violations.is_empty());
    }

    fn mining_city_id(state: &SimState) -> crate::world::EntityId {
        state
            .sector
            .systems
            .iter()
            .flat_map(|s| &s.planets)
            .flat_map(|p| &p.countries)
            .flat_map(|c| &c.cities)
            .find(|c| c.specialization == crate::world::CitySpecialization::Mining)
            .expect("test seed should generate at least one mining city")
            .id
    }

    #[test]
    fn a_non_finite_business_equity_is_caught() {
        let mut state = SimState::new(5);
        let host_city_id = mining_city_id(&state);
        state
            .found_business(
                "Ferrous Extraction Co.".to_string(),
                crate::business::BusinessArchetype::Mining,
                host_city_id,
            )
            .unwrap();
        state.businesses[0].equity = f64::NAN;
        let violations = check_invariants(&state);
        assert!(!violations.is_empty());
    }

    #[test]
    fn a_business_owned_by_an_unknown_character_is_caught() {
        let mut state = SimState::new(6);
        let host_city_id = mining_city_id(&state);
        state
            .found_business(
                "Ferrous Extraction Co.".to_string(),
                crate::business::BusinessArchetype::Mining,
                host_city_id,
            )
            .unwrap();
        state.businesses[0].owner_character_id = 999_999;
        let violations = check_invariants(&state);
        assert!(!violations.is_empty());
    }

    #[test]
    fn a_business_hosted_in_an_unknown_city_is_caught() {
        let mut state = SimState::new(7);
        let host_city_id = mining_city_id(&state);
        state
            .found_business(
                "Ferrous Extraction Co.".to_string(),
                crate::business::BusinessArchetype::Mining,
                host_city_id,
            )
            .unwrap();
        state.businesses[0].host_city_id = 999_999;
        let violations = check_invariants(&state);
        assert!(!violations.is_empty());
    }

    #[test]
    fn a_freshly_founded_business_has_no_violations() {
        let mut state = SimState::new(8);
        let host_city_id = mining_city_id(&state);
        state
            .found_business(
                "Ferrous Extraction Co.".to_string(),
                crate::business::BusinessArchetype::Mining,
                host_city_id,
            )
            .unwrap();
        assert_eq!(check_invariants(&state), Vec::new());
    }

    #[test]
    fn a_career_with_an_out_of_range_level_is_caught() {
        let mut state = SimState::new(9);
        let home_city = state.dynasty.head().unwrap().home_city.unwrap();
        let head_id = state.dynasty.head_character_id;
        state
            .start_career(crate::career::CareerTrack::Corporate, head_id, home_city)
            .unwrap();
        state.careers[0].level = 200;
        let violations = check_invariants(&state);
        assert!(!violations.is_empty());
    }

    #[test]
    fn a_career_held_by_an_unknown_character_is_caught() {
        let mut state = SimState::new(10);
        let home_city = state.dynasty.head().unwrap().home_city.unwrap();
        let head_id = state.dynasty.head_character_id;
        state
            .start_career(crate::career::CareerTrack::Corporate, head_id, home_city)
            .unwrap();
        state.careers[0].character_id = 999_999;
        let violations = check_invariants(&state);
        assert!(!violations.is_empty());
    }

    #[test]
    fn a_career_based_in_an_unknown_city_is_caught() {
        let mut state = SimState::new(11);
        let home_city = state.dynasty.head().unwrap().home_city.unwrap();
        let head_id = state.dynasty.head_character_id;
        state
            .start_career(crate::career::CareerTrack::Corporate, head_id, home_city)
            .unwrap();
        state.careers[0].employer_city_id = 999_999;
        let violations = check_invariants(&state);
        assert!(!violations.is_empty());
    }

    #[test]
    fn a_freshly_started_career_has_no_violations() {
        let mut state = SimState::new(12);
        let home_city = state.dynasty.head().unwrap().home_city.unwrap();
        let head_id = state.dynasty.head_character_id;
        state
            .start_career(crate::career::CareerTrack::Corporate, head_id, home_city)
            .unwrap();
        assert_eq!(check_invariants(&state), Vec::new());
    }

    /// Adds a second dynasty member (a clone of the head with a new id) so
    /// favor tests have two distinct, real character ids to work with
    /// without depending on the seed-dependent birth mechanic.
    fn add_second_member(state: &mut SimState) -> crate::world::EntityId {
        let head_id = state.dynasty.head_character_id;
        let new_id = head_id + 1;
        let mut member = state.dynasty.head().unwrap().clone();
        member.id = new_id;
        state.dynasty.members.push(member);
        new_id
    }

    #[test]
    fn a_freshly_granted_favor_has_no_violations() {
        let mut state = SimState::new(9);
        let head_id = state.dynasty.head_character_id;
        let debtor_id = add_second_member(&mut state);
        state.grant_favor(head_id, debtor_id, 2.0).unwrap();
        assert_eq!(check_invariants(&state), Vec::new());
    }

    #[test]
    fn a_favor_with_an_unknown_creditor_is_caught() {
        let mut state = SimState::new(10);
        let debtor_id = add_second_member(&mut state);
        state.favors.push(crate::favor::Favor {
            creditor_id: 999_999,
            debtor_id,
            magnitude: 1.0,
        });
        let violations = check_invariants(&state);
        assert!(!violations.is_empty());
    }

    #[test]
    fn a_favor_with_an_unknown_debtor_is_caught() {
        let mut state = SimState::new(11);
        let head_id = state.dynasty.head_character_id;
        state.favors.push(crate::favor::Favor {
            creditor_id: head_id,
            debtor_id: 999_999,
            magnitude: 1.0,
        });
        let violations = check_invariants(&state);
        assert!(!violations.is_empty());
    }

    #[test]
    fn a_freshly_generated_state_has_no_trade_route_violations() {
        let state = SimState::new(13);
        assert!(
            !state.trade_routes.is_empty(),
            "expected some generated trade routes"
        );
        assert_eq!(check_invariants(&state), Vec::new());
    }

    #[test]
    fn a_trade_route_with_a_dangling_endpoint_is_caught() {
        let mut state = SimState::new(14);
        assert!(
            !state.trade_routes.is_empty(),
            "expected some generated trade routes"
        );
        state.trade_routes[0].city_b_id = 999_999;
        let violations = check_invariants(&state);
        assert!(
            violations
                .iter()
                .any(|v| v.0.contains("does not reference an existing city")),
            "expected a violation naming the dangling trade route endpoint, got {violations:?}"
        );
    }

    #[test]
    fn a_trade_route_with_an_out_of_range_reliability_is_caught() {
        let mut state = SimState::new(15);
        assert!(
            !state.trade_routes.is_empty(),
            "expected some generated trade routes"
        );
        state.trade_routes[0].reliability = 1.5;
        let violations = check_invariants(&state);
        assert!(!violations.is_empty());
    }

    #[test]
    fn a_favor_with_a_non_finite_magnitude_is_caught() {
        let mut state = SimState::new(12);
        let head_id = state.dynasty.head_character_id;
        let debtor_id = add_second_member(&mut state);
        state.favors.push(crate::favor::Favor {
            creditor_id: head_id,
            debtor_id,
            magnitude: f64::NAN,
        });
        let violations = check_invariants(&state);
        assert!(!violations.is_empty());
    }

    #[test]
    fn a_freshly_scanned_state_has_no_pending_event_violations() {
        let mut state = SimState::new(13);
        crate::events::scan_for_eligible_events(&mut state);
        assert!(
            !state.pending_events.is_empty(),
            "expected the founder to be offered at least one event"
        );
        assert_eq!(check_invariants(&state), Vec::new());
    }

    #[test]
    fn a_pending_event_with_an_unknown_key_is_caught() {
        let mut state = SimState::new(14);
        let head_id = state.dynasty.head_character_id;
        state.pending_events.push(crate::events::PendingEvent {
            id: 1,
            key: "not_a_real_event".to_string(),
            character_id: head_id,
            raised_year: 0,
        });
        let violations = check_invariants(&state);
        assert!(
            violations
                .iter()
                .any(|v| v.0.contains("does not reference a known event definition")),
            "expected a violation naming the unknown event key, got {violations:?}"
        );
    }

    #[test]
    fn a_pending_event_with_an_unknown_character_is_caught() {
        let mut state = SimState::new(15);
        state.pending_events.push(crate::events::PendingEvent {
            id: 1,
            key: "family_seed_money".to_string(),
            character_id: 999_999,
            raised_year: 0,
        });
        let violations = check_invariants(&state);
        assert!(
            violations
                .iter()
                .any(|v| v.0.contains("does not reference an existing dynasty")),
            "expected a violation naming the dangling character, got {violations:?}"
        );
    }

    #[test]
    fn a_duplicate_pending_event_for_the_same_character_is_caught() {
        let mut state = SimState::new(16);
        let head_id = state.dynasty.head_character_id;
        state.pending_events.push(crate::events::PendingEvent {
            id: 1,
            key: "family_seed_money".to_string(),
            character_id: head_id,
            raised_year: 0,
        });
        state.pending_events.push(crate::events::PendingEvent {
            id: 2,
            key: "family_seed_money".to_string(),
            character_id: head_id,
            raised_year: 0,
        });
        let violations = check_invariants(&state);
        assert!(
            violations
                .iter()
                .any(|v| v.0.contains("duplicate pending event")),
            "expected a violation naming the duplicate, got {violations:?}"
        );
    }

    #[test]
    fn a_resolved_event_log_entry_with_an_unknown_character_is_caught() {
        let mut state = SimState::new(17);
        state.event_log.resolved.push(crate::events::ResolvedEvent {
            key: "family_seed_money".to_string(),
            character_id: 999_999,
            choice_key: "accept".to_string(),
            year: 1,
        });
        let violations = check_invariants(&state);
        assert!(
            violations
                .iter()
                .any(|v| v.0.contains("resolved event log entry")),
            "expected a violation naming the dangling resolved-event character, got {violations:?}"
        );
    }
}
