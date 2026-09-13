//! The minimal event framework: a data-driven trigger/condition check, a set
//! of player choices, and effects, without hardcoding scenarios into
//! `sim-core`'s control flow. Issue #29, part of epic #9. See
//! `GAME_DESIGN.md`'s "Events tell stories that arise from state" section
//! and `docs/CONTENT_GUIDE.md` for the design intent this implements.
//!
//! ## Shape
//!
//! An [`EventDefinition`] is a static catalog entry: a `condition` (reads
//! `SimState` and a candidate character id, returns whether the event is
//! eligible for them right now) and a fixed list of [`EventChoiceDefinition`]
//! entries, each with an `apply` effect. Definitions live in [`ALL_EVENTS`],
//! the same "small fixed list of plain Rust data" shape as
//! `crate::traits::ALL_TRAITS`, deliberately not a scripting language or an
//! external file format (that's backlog work for a real content-pack schema,
//! see `docs/CONTENT_GUIDE.md`'s "Where content will live" section, not this
//! issue).
//!
//! [`scan_for_eligible_events`] is the yearly system (see
//! `state::SimState::step_one_day`): it checks every living character
//! against every definition and, when a condition matches, appends a
//! [`PendingEvent`] rather than applying any effect immediately. A pending
//! event sits in `SimState` (so it survives a save/load) until
//! [`resolve_pending_event`] is called with the player's chosen
//! [`EventChoiceDefinition::key`], at which point exactly one effect runs and
//! the event is recorded in [`EventLog`] and removed from the pending list.
//! This is the "surfaced but not applied until a choice is made" acceptance
//! criterion.
//!
//! [`EventLog`] doubles as the "event flags" store: whether a character has
//! ever resolved a given event, and when, is read back by
//! [`scan_for_eligible_events`] itself (a non-repeatable event never recurs
//! for the same character once resolved; a repeatable one respects
//! `cooldown_years`) and is available to any future event's `condition` too,
//! so a later event can react to how an earlier one was resolved. That's the
//! hook for "some consequences surface years later" from `GAME_DESIGN.md`,
//! without this issue needing to build a separate delayed-effect scheduler.
//!
//! ## Adding an event
//!
//! Write a `condition` and one `apply` function per choice (plain functions
//! reading/mutating `SimState`, the same pattern every other system in this
//! crate uses), add an [`EventDefinition`] entry to [`ALL_EVENTS`], and
//! that's the whole surface. No other module needs to change; a real content
//! pack (per `docs/CONTENT_GUIDE.md`) is expected to add many such entries
//! without ever touching `state.rs` or the scan/resolve logic below.

use serde::{Deserialize, Serialize};

use crate::rng::SimRng;
use crate::state::SimState;
use crate::world::EntityId;

/// A predicate over the whole simulation state and one candidate character:
/// is this event eligible for `character_id` right now? Kept as a plain
/// function pointer (not a trait object or a scripting hook) so every event
/// definition is ordinary, auditable Rust code; see the module doc comment.
pub type EventCondition = fn(&SimState, EntityId) -> bool;

/// One choice's consequence: mutate `SimState` on behalf of `character_id`.
/// Runs exactly once, when [`resolve_pending_event`] is called with the
/// matching [`EventChoiceDefinition::key`], never as a side effect of
/// scanning.
pub type EventEffect = fn(&mut SimState, EntityId);

/// One option a player can pick when an event is presented.
pub struct EventChoiceDefinition {
    /// Stable identifier for this choice within its event, e.g. `"accept"`.
    /// Passed back into [`resolve_pending_event`] to select it.
    pub key: &'static str,
    /// Player-facing label. Must never contain an em dash or a semicolon
    /// per `docs/CONTENT_GUIDE.md`; the copy lint only scans
    /// `web/src/content/`, so this crate's own event labels are not
    /// currently linted automatically, but the rule still applies.
    pub label: &'static str,
    pub apply: EventEffect,
}

/// A catalog entry: an event's identity, eligibility rule, and choices.
pub struct EventDefinition {
    /// Stable identifier, unique across [`ALL_EVENTS`]. Persisted on
    /// [`PendingEvent`] and [`ResolvedEvent`] as a plain `String` (rather
    /// than an index into this slice) so a save file stays meaningful even
    /// if the catalog's order changes between versions.
    pub key: &'static str,
    /// Player-facing title. See the em dash/semicolon note on
    /// [`EventChoiceDefinition::label`].
    pub title: &'static str,
    pub condition: EventCondition,
    /// Never empty; see the `every_definition_has_at_least_one_choice` test.
    pub choices: &'static [EventChoiceDefinition],
    /// If `false`, once a character has ever resolved this event (any
    /// choice), it never becomes eligible for them again, regardless of
    /// `cooldown_years` (which is ignored in that case; a non-repeatable
    /// event's cooldown is "forever").
    pub repeatable: bool,
    /// For a repeatable event only: minimum number of in-world years after
    /// a character's most recent resolution before it can become eligible
    /// for them again. Ignored when `repeatable` is `false`.
    pub cooldown_years: u64,
}

/// Look up a catalog entry by its stable key. `None` means either a bug (a
/// [`PendingEvent`]/[`ResolvedEvent`] was constructed with a key that was
/// never in [`ALL_EVENTS`]) or a save from a build whose catalog has since
/// dropped that key; callers treat both the same way, as a graceful miss
/// rather than a panic.
pub fn definition(key: &str) -> Option<&'static EventDefinition> {
    ALL_EVENTS.iter().find(|definition| definition.key == key)
}

/// An event that has been raised for a specific character but not yet
/// resolved. Surfaced to the player (a UI/CLI layer reads
/// `SimState::pending_events`); no effect has run yet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PendingEvent {
    /// Unique among pending events for the lifetime of a save, monotonic
    /// per [`scan_for_eligible_events`] call, the same `max() + 1` pattern
    /// `SimState::found_business`/`start_career` use for their own ids.
    pub id: EntityId,
    /// Matches an [`EventDefinition::key`]; see [`definition`].
    pub key: String,
    pub character_id: EntityId,
    /// In-world year this event became eligible, for display and for
    /// debugging a stuck/stale pending event.
    pub raised_year: u64,
}

/// One resolved event, kept forever in [`EventLog::resolved`]: who, which
/// event, which choice, and when. This is both the narrative log and the
/// "has this happened before" flag store [`scan_for_eligible_events`] reads
/// back (see the module doc comment).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResolvedEvent {
    pub key: String,
    pub character_id: EntityId,
    pub choice_key: String,
    pub year: u64,
}

/// Every event ever resolved. Additive, append-only, and empty on a save
/// from before the event framework existed (see its `#[serde(default)]`
/// field on `SimState`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventLog {
    pub resolved: Vec<ResolvedEvent>,
}

impl EventLog {
    /// Whether `character_id` has ever resolved the event named `key`, any
    /// choice, any year.
    pub fn has_ever_resolved(&self, key: &str, character_id: EntityId) -> bool {
        self.resolved
            .iter()
            .any(|entry| entry.key == key && entry.character_id == character_id)
    }

    /// The most recent year `character_id` resolved the event named `key`,
    /// if ever.
    pub fn last_resolved_year(&self, key: &str, character_id: EntityId) -> Option<u64> {
        self.resolved
            .iter()
            .filter(|entry| entry.key == key && entry.character_id == character_id)
            .map(|entry| entry.year)
            .max()
    }
}

/// Whether `definition` should be skipped for `character_id` at `year`
/// purely because of its own history with this event, independent of its
/// `condition`. A non-repeatable event is skipped forever once resolved; a
/// repeatable one is skipped until `cooldown_years` have passed since its
/// last resolution.
fn blocked_by_history(
    log: &EventLog,
    definition: &EventDefinition,
    character_id: EntityId,
    year: u64,
) -> bool {
    if !definition.repeatable {
        return log.has_ever_resolved(definition.key, character_id);
    }
    match log.last_resolved_year(definition.key, character_id) {
        Some(last_year) => year.saturating_sub(last_year) < definition.cooldown_years,
        None => false,
    }
}

/// The yearly system: for every living dynasty member and every catalog
/// entry, raise a [`PendingEvent`] if it isn't already pending for that
/// character, isn't blocked by [`blocked_by_history`], and its `condition`
/// matches. Applies no effects; see the module doc comment.
///
/// Iterates [`ALL_EVENTS`] in its fixed declaration order and
/// `state.dynasty.members` in their existing, already-deterministic vector
/// order, so two same-seed runs raise events in the same order at the same
/// tick, matching this crate's determinism guarantee (see `crate::rng`).
pub fn scan_for_eligible_events(state: &mut SimState) {
    let year = state.clock.year();
    let mut next_id = state
        .pending_events
        .iter()
        .map(|event| event.id)
        .max()
        .unwrap_or(0)
        + 1;

    let candidate_ids: Vec<EntityId> = state
        .dynasty
        .members
        .iter()
        .filter(|character| character.alive)
        .map(|character| character.id)
        .collect();

    for def in ALL_EVENTS {
        for &character_id in &candidate_ids {
            let already_pending = state
                .pending_events
                .iter()
                .any(|pending| pending.key == def.key && pending.character_id == character_id);
            if already_pending {
                continue;
            }
            if blocked_by_history(&state.event_log, def, character_id, year) {
                continue;
            }
            if (def.condition)(state, character_id) {
                state.pending_events.push(PendingEvent {
                    id: next_id,
                    key: def.key.to_string(),
                    character_id,
                    raised_year: year,
                });
                next_id += 1;
            }
        }
    }
}

/// Why [`resolve_pending_event`] refused to apply a choice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveEventError {
    /// No pending event with this id exists (already resolved, never
    /// existed, or a stale id from a different save).
    UnknownPendingEvent(EntityId),
    /// The pending event's own `key` doesn't resolve to any entry in
    /// [`ALL_EVENTS`] (a stale save from a build whose catalog has since
    /// dropped that key).
    UnknownEventDefinition(String),
    /// `choice_key` doesn't match any of the event's
    /// [`EventChoiceDefinition::key`]s.
    UnknownChoice(String),
}

/// Apply the player's chosen effect for the pending event named `pending_id`
/// and `choice_key`, then move it from `state.pending_events` into
/// `state.event_log.resolved`. Fails without mutating `state` if the
/// pending event, its definition, or the choice can't be resolved.
pub fn resolve_pending_event(
    state: &mut SimState,
    pending_id: EntityId,
    choice_key: &str,
) -> Result<(), ResolveEventError> {
    let index = state
        .pending_events
        .iter()
        .position(|pending| pending.id == pending_id)
        .ok_or(ResolveEventError::UnknownPendingEvent(pending_id))?;

    let def = definition(&state.pending_events[index].key).ok_or_else(|| {
        ResolveEventError::UnknownEventDefinition(state.pending_events[index].key.clone())
    })?;
    let choice = def
        .choices
        .iter()
        .find(|choice| choice.key == choice_key)
        .ok_or_else(|| ResolveEventError::UnknownChoice(choice_key.to_string()))?;

    let pending = state.pending_events[index].clone();
    (choice.apply)(state, pending.character_id);

    let year = state.clock.year();
    state.pending_events.remove(index);
    state.event_log.resolved.push(ResolvedEvent {
        key: pending.key,
        character_id: pending.character_id,
        choice_key: choice_key.to_string(),
        year,
    });
    Ok(())
}

// --- Example events -------------------------------------------------------
//
// Two small, self-contained events proving the framework works end to end:
// one reads age/career context with a deterministic, non-random effect
// (family_seed_money), the other reads wealth/career context and rolls a
// deterministic per-instance outcome (risky_investment_tip). Real content
// (per `docs/CONTENT_GUIDE.md`) is explicitly out of scope for this issue.

const FAMILY_SEED_MONEY_ADULT_AGE: u32 = 18;
const FAMILY_SEED_MONEY_AMOUNT: f64 = 1_000.0;

fn family_seed_money_condition(state: &SimState, character_id: EntityId) -> bool {
    let Some(character) = state.dynasty.members.iter().find(|c| c.id == character_id) else {
        return false;
    };
    character.alive
        && character.age_years >= FAMILY_SEED_MONEY_ADULT_AGE
        && !state
            .careers
            .iter()
            .any(|career| career.character_id == character_id)
}

fn family_seed_money_accept(state: &mut SimState, character_id: EntityId) {
    if let Some(character) = state
        .dynasty
        .members
        .iter_mut()
        .find(|c| c.id == character_id)
    {
        character.wealth += FAMILY_SEED_MONEY_AMOUNT;
    }
}

fn family_seed_money_decline(_state: &mut SimState, _character_id: EntityId) {
    // Declining changes nothing but is still recorded in the event log (see
    // `resolve_pending_event`), which is what keeps this non-repeatable
    // event from being offered to the same character again.
}

const RISKY_INVESTMENT_MIN_WEALTH: f64 = 500.0;
const RISKY_INVESTMENT_COOLDOWN_YEARS: u64 = 3;
const RISKY_INVESTMENT_SUCCESS_CHANCE: f64 = 0.55;
const RISKY_INVESTMENT_GAIN_MULTIPLIER: f64 = 1.5;
const RISKY_INVESTMENT_LOSS_MULTIPLIER: f64 = 0.6;

fn risky_investment_tip_condition(state: &SimState, character_id: EntityId) -> bool {
    let Some(character) = state.dynasty.members.iter().find(|c| c.id == character_id) else {
        return false;
    };
    character.alive
        && character.wealth >= RISKY_INVESTMENT_MIN_WEALTH
        && state
            .careers
            .iter()
            .any(|career| career.character_id == character_id)
}

/// Deterministic roll for this exact (character, year) pair, from an
/// ephemeral stream derived from the world seed rather than a field
/// persisted on `SimState`. Matches `crate::birth::maybe_birth_child`'s
/// pattern: a one-shot draw at resolution time reproduces exactly on replay
/// without needing a new persisted RNG domain for an event that most
/// characters, most years, never even trigger.
fn risky_investment_tip_invest(state: &mut SimState, character_id: EntityId) {
    let mut rng = SimRng::from_seed(
        state.seed,
        &format!(
            "events:risky_investment_tip:{character_id}:{}",
            state.clock.year()
        ),
    );
    let succeeded = rng.chance(RISKY_INVESTMENT_SUCCESS_CHANCE);
    if let Some(character) = state
        .dynasty
        .members
        .iter_mut()
        .find(|c| c.id == character_id)
    {
        character.wealth *= if succeeded {
            RISKY_INVESTMENT_GAIN_MULTIPLIER
        } else {
            RISKY_INVESTMENT_LOSS_MULTIPLIER
        };
    }
}

fn risky_investment_tip_decline(_state: &mut SimState, _character_id: EntityId) {}

/// Every event definition in the game. Order is fixed and matters for
/// determinism (see [`scan_for_eligible_events`]); append new entries rather
/// than reordering existing ones.
pub const ALL_EVENTS: &[EventDefinition] = &[
    EventDefinition {
        key: "family_seed_money",
        title: "A relative offers seed money",
        condition: family_seed_money_condition,
        choices: &[
            EventChoiceDefinition {
                key: "accept",
                label: "Accept the money",
                apply: family_seed_money_accept,
            },
            EventChoiceDefinition {
                key: "decline",
                label: "Politely decline",
                apply: family_seed_money_decline,
            },
        ],
        repeatable: false,
        cooldown_years: 0,
    },
    EventDefinition {
        key: "risky_investment_tip",
        title: "A colleague shares an investment tip",
        condition: risky_investment_tip_condition,
        choices: &[
            EventChoiceDefinition {
                key: "invest",
                label: "Invest",
                apply: risky_investment_tip_invest,
            },
            EventChoiceDefinition {
                key: "decline",
                label: "Decline",
                apply: risky_investment_tip_decline,
            },
        ],
        repeatable: true,
        cooldown_years: RISKY_INVESTMENT_COOLDOWN_YEARS,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::career::CareerTrack;
    use crate::invariants::check_invariants;

    // --- Catalog sanity -----------------------------------------------

    #[test]
    fn every_event_key_is_unique() {
        let mut keys: Vec<&str> = ALL_EVENTS.iter().map(|d| d.key).collect();
        let original_len = keys.len();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(
            keys.len(),
            original_len,
            "duplicate event key in ALL_EVENTS"
        );
    }

    #[test]
    fn every_definition_has_at_least_one_choice_with_a_unique_key() {
        for def in ALL_EVENTS {
            assert!(!def.choices.is_empty(), "event {} has no choices", def.key);
            let mut choice_keys: Vec<&str> = def.choices.iter().map(|c| c.key).collect();
            let original_len = choice_keys.len();
            choice_keys.sort_unstable();
            choice_keys.dedup();
            assert_eq!(
                choice_keys.len(),
                original_len,
                "event {} has duplicate choice keys",
                def.key
            );
        }
    }

    #[test]
    fn definition_looks_up_known_and_unknown_keys() {
        assert!(definition("family_seed_money").is_some());
        assert!(definition("risky_investment_tip").is_some());
        assert!(definition("does_not_exist").is_none());
    }

    // --- EventLog --------------------------------------------------------

    #[test]
    fn event_log_reports_history_correctly() {
        let mut log = EventLog::default();
        assert!(!log.has_ever_resolved("family_seed_money", 1));
        assert_eq!(log.last_resolved_year("family_seed_money", 1), None);

        log.resolved.push(ResolvedEvent {
            key: "family_seed_money".to_string(),
            character_id: 1,
            choice_key: "accept".to_string(),
            year: 5,
        });
        assert!(log.has_ever_resolved("family_seed_money", 1));
        assert!(!log.has_ever_resolved("family_seed_money", 2));
        assert_eq!(log.last_resolved_year("family_seed_money", 1), Some(5));
    }

    // --- family_seed_money -------------------------------------------

    #[test]
    fn family_seed_money_is_eligible_for_an_unemployed_adult() {
        let state = SimState::new(2026);
        let head_id = state.dynasty.head_character_id;
        // The founder starts unemployed and always at least 28, so the
        // condition should hold immediately.
        assert!(family_seed_money_condition(&state, head_id));
    }

    #[test]
    fn family_seed_money_is_not_eligible_once_employed() {
        let mut state = SimState::new(2026);
        let home_city = state.dynasty.head().unwrap().home_city.unwrap();
        let head_id = state.dynasty.head_character_id;
        state
            .start_career(CareerTrack::Corporate, head_id, home_city)
            .unwrap();
        assert!(!family_seed_money_condition(&state, head_id));
    }

    #[test]
    fn family_seed_money_is_not_eligible_for_a_dead_character() {
        let mut state = SimState::new(2026);
        let head_id = state.dynasty.head_character_id;
        state
            .dynasty
            .members
            .iter_mut()
            .find(|c| c.id == head_id)
            .unwrap()
            .alive = false;
        assert!(!family_seed_money_condition(&state, head_id));
    }

    #[test]
    fn family_seed_money_accept_adds_exactly_the_seed_amount() {
        let mut state = SimState::new(2026);
        let head_id = state.dynasty.head_character_id;
        let before = state.dynasty.head().unwrap().wealth;
        family_seed_money_accept(&mut state, head_id);
        assert_eq!(
            state.dynasty.head().unwrap().wealth,
            before + FAMILY_SEED_MONEY_AMOUNT
        );
    }

    #[test]
    fn family_seed_money_decline_changes_nothing() {
        let mut state = SimState::new(2026);
        let head_id = state.dynasty.head_character_id;
        let before = state.to_json();
        family_seed_money_decline(&mut state, head_id);
        assert_eq!(state.to_json(), before);
    }

    // --- risky_investment_tip -----------------------------------------

    #[test]
    fn risky_investment_tip_requires_wealth_and_a_career() {
        let mut state = SimState::new(2026);
        let head_id = state.dynasty.head_character_id;
        // Unemployed, regardless of wealth: not eligible.
        assert!(!risky_investment_tip_condition(&state, head_id));

        let home_city = state.dynasty.head().unwrap().home_city.unwrap();
        state
            .start_career(CareerTrack::Corporate, head_id, home_city)
            .unwrap();
        state
            .dynasty
            .members
            .iter_mut()
            .find(|c| c.id == head_id)
            .unwrap()
            .wealth = RISKY_INVESTMENT_MIN_WEALTH - 1.0;
        assert!(
            !risky_investment_tip_condition(&state, head_id),
            "not enough wealth"
        );

        state
            .dynasty
            .members
            .iter_mut()
            .find(|c| c.id == head_id)
            .unwrap()
            .wealth = RISKY_INVESTMENT_MIN_WEALTH;
        assert!(risky_investment_tip_condition(&state, head_id));
    }

    #[test]
    fn risky_investment_tip_invest_moves_wealth_by_one_of_the_two_multipliers() {
        let mut state = SimState::new(2026);
        let head_id = state.dynasty.head_character_id;
        let before = state.dynasty.head().unwrap().wealth;
        risky_investment_tip_invest(&mut state, head_id);
        let after = state.dynasty.head().unwrap().wealth;
        let gain = before * RISKY_INVESTMENT_GAIN_MULTIPLIER;
        let loss = before * RISKY_INVESTMENT_LOSS_MULTIPLIER;
        assert!(
            (after - gain).abs() < 1e-9 || (after - loss).abs() < 1e-9,
            "expected wealth to move by the gain or loss multiplier, got {before} -> {after}"
        );
    }

    #[test]
    fn risky_investment_tip_invest_is_deterministic_for_the_same_seed_and_year() {
        fn invest_outcome(seed: u64) -> f64 {
            let mut state = SimState::new(seed);
            let head_id = state.dynasty.head_character_id;
            state
                .dynasty
                .members
                .iter_mut()
                .find(|c| c.id == head_id)
                .unwrap()
                .wealth = 1000.0;
            risky_investment_tip_invest(&mut state, head_id);
            state.dynasty.head().unwrap().wealth
        }
        assert_eq!(invest_outcome(2026), invest_outcome(2026));
    }

    // --- scan_for_eligible_events --------------------------------------

    #[test]
    fn scanning_raises_a_pending_event_when_the_condition_matches() {
        let mut state = SimState::new(2026);
        let head_id = state.dynasty.head_character_id;
        assert!(state.pending_events.is_empty());

        scan_for_eligible_events(&mut state);

        assert!(state
            .pending_events
            .iter()
            .any(|p| p.key == "family_seed_money" && p.character_id == head_id));
    }

    #[test]
    fn scanning_twice_does_not_duplicate_an_already_pending_event() {
        let mut state = SimState::new(2026);
        scan_for_eligible_events(&mut state);
        let count_after_first_scan = state.pending_events.len();
        scan_for_eligible_events(&mut state);
        assert_eq!(state.pending_events.len(), count_after_first_scan);
    }

    #[test]
    fn a_non_repeatable_event_never_recurs_after_being_resolved() {
        let mut state = SimState::new(2026);
        let head_id = state.dynasty.head_character_id;
        scan_for_eligible_events(&mut state);
        let pending_id = state
            .pending_events
            .iter()
            .find(|p| p.key == "family_seed_money" && p.character_id == head_id)
            .unwrap()
            .id;

        state.resolve_event(pending_id, "decline").unwrap();
        // The condition (unemployed adult) still holds, but history should
        // now block it.
        assert!(family_seed_money_condition(&state, head_id));

        scan_for_eligible_events(&mut state);
        assert!(!state
            .pending_events
            .iter()
            .any(|p| p.key == "family_seed_money" && p.character_id == head_id));
    }

    #[test]
    fn a_repeatable_event_respects_its_cooldown_then_recurs() {
        let mut state = SimState::new(2026);
        let home_city = state.dynasty.head().unwrap().home_city.unwrap();
        let head_id = state.dynasty.head_character_id;
        state
            .start_career(CareerTrack::Corporate, head_id, home_city)
            .unwrap();
        state
            .dynasty
            .members
            .iter_mut()
            .find(|c| c.id == head_id)
            .unwrap()
            .wealth = 10_000.0;

        scan_for_eligible_events(&mut state);
        let pending_id = state
            .pending_events
            .iter()
            .find(|p| p.key == "risky_investment_tip")
            .unwrap()
            .id;
        state.resolve_event(pending_id, "decline").unwrap();

        // Immediately after resolving: still within the cooldown window.
        scan_for_eligible_events(&mut state);
        assert!(!state
            .pending_events
            .iter()
            .any(|p| p.key == "risky_investment_tip"));

        // Advance past the cooldown and rescan.
        state.step_days(
            crate::time::DAYS_PER_YEAR as u32 * (RISKY_INVESTMENT_COOLDOWN_YEARS as u32 + 1),
        );
        scan_for_eligible_events(&mut state);
        assert!(state
            .pending_events
            .iter()
            .any(|p| p.key == "risky_investment_tip"));
    }

    // --- resolve_pending_event -------------------------------------------

    #[test]
    fn resolving_an_unknown_pending_id_is_refused_and_leaves_state_untouched() {
        let mut state = SimState::new(2026);
        let before = state.to_json();
        let err = state.resolve_event(999_999, "accept").unwrap_err();
        assert_eq!(err, ResolveEventError::UnknownPendingEvent(999_999));
        assert_eq!(state.to_json(), before);
    }

    #[test]
    fn resolving_an_unknown_choice_key_is_refused_and_leaves_state_untouched() {
        let mut state = SimState::new(2026);
        scan_for_eligible_events(&mut state);
        let pending_id = state.pending_events[0].id;
        let before = state.to_json();

        let err = state
            .resolve_event(pending_id, "not_a_real_choice")
            .unwrap_err();

        assert_eq!(
            err,
            ResolveEventError::UnknownChoice("not_a_real_choice".to_string())
        );
        assert_eq!(state.to_json(), before);
    }

    #[test]
    fn resolving_removes_the_pending_event_and_appends_to_the_log() {
        let mut state = SimState::new(2026);
        let head_id = state.dynasty.head_character_id;
        scan_for_eligible_events(&mut state);
        let pending_id = state
            .pending_events
            .iter()
            .find(|p| p.character_id == head_id && p.key == "family_seed_money")
            .unwrap()
            .id;
        let wealth_before = state.dynasty.head().unwrap().wealth;

        state.resolve_event(pending_id, "accept").unwrap();

        assert!(!state.pending_events.iter().any(|p| p.id == pending_id));
        assert!(state
            .event_log
            .has_ever_resolved("family_seed_money", head_id));
        assert_eq!(
            state.dynasty.head().unwrap().wealth,
            wealth_before + FAMILY_SEED_MONEY_AMOUNT
        );
        assert!(check_invariants(&state).is_empty());
    }

    // --- End-to-end determinism ------------------------------------------

    #[test]
    fn same_seed_runs_raise_the_same_pending_events_at_the_same_tick() {
        fn run(seed: u64) -> Vec<PendingEvent> {
            let mut state = SimState::new(seed);
            state.step_days(365 * 2);
            state.pending_events.clone()
        }
        assert_eq!(run(2026), run(2026));
        assert!(
            !run(2026).is_empty(),
            "expected at least one event to be raised over two years"
        );
    }

    #[test]
    fn applying_the_same_choice_on_two_same_seed_states_is_deterministic_and_valid() {
        fn run(seed: u64) -> String {
            let mut state = SimState::new(seed);
            state.step_days(365 * 2);
            let pending_id = state
                .pending_events
                .first()
                .expect("expected at least one pending event after two years")
                .id;
            state
                .resolve_event(pending_id, "accept")
                .or_else(|_| state.resolve_event(pending_id, "invest"))
                .expect("the first pending event should offer accept or invest");
            state.step_days(365);
            state.to_json()
        }

        let a = run(2026);
        let b = run(2026);
        assert_eq!(a, b);
    }
}
