//! Read-only UI projections over [`SimState`]: join a raw entity (a
//! business, a trade route, a pending event) with the names and titles a
//! screen needs to display it, so the frontend never re-implements a
//! `Sector`/`Dynasty` lookup or mirrors event catalog content itself.
//!
//! This is the "explicit query" boundary the product spec's "Querying
//! simulation state" guidance asks for: the UI reads through these views,
//! attached to [`crate::state::StateSummary`], rather than walking or
//! duplicating `SimState`'s internal structures.

use serde::{Deserialize, Serialize};

use crate::events;
use crate::state::SimState;
use crate::world::EntityId;

fn city_name(state: &SimState, city_id: EntityId) -> String {
    state
        .sector
        .find_city(city_id)
        .map(|city| city.name.clone())
        .unwrap_or_else(|| "Unknown city".to_string())
}

fn character_name(state: &SimState, character_id: EntityId) -> String {
    state
        .dynasty
        .members
        .iter()
        .find(|character| character.id == character_id)
        .map(|character| character.name.clone())
        .unwrap_or_else(|| "Unknown".to_string())
}

/// One choice on a [`PendingEventView`], joined from its
/// [`crate::events::EventDefinition`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventChoiceView {
    pub key: String,
    pub label: String,
}

/// A pending event, joined with its definition's title/choices and the
/// character's name, ready to render as a decision card.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PendingEventView {
    pub id: EntityId,
    pub key: String,
    pub title: String,
    pub character_id: EntityId,
    pub character_name: String,
    pub raised_year: u64,
    pub choices: Vec<EventChoiceView>,
}

/// Every currently pending event, ready to display. A pending event whose
/// key no longer resolves to a catalog entry (a stale save from a build
/// whose catalog has since changed) is silently omitted rather than shown
/// broken; see [`events::definition`].
pub fn pending_event_views(state: &SimState) -> Vec<PendingEventView> {
    state
        .pending_events
        .iter()
        .filter_map(|pending| {
            let definition = events::definition(&pending.key)?;
            Some(PendingEventView {
                id: pending.id,
                key: pending.key.clone(),
                title: definition.title.to_string(),
                character_id: pending.character_id,
                character_name: character_name(state, pending.character_id),
                raised_year: pending.raised_year,
                choices: definition
                    .choices
                    .iter()
                    .map(|choice| EventChoiceView {
                        key: choice.key.to_string(),
                        label: choice.label.to_string(),
                    })
                    .collect(),
            })
        })
        .collect()
}

/// A founded business, joined with its host city's name.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BusinessView {
    pub id: EntityId,
    pub name: String,
    pub archetype: String,
    pub host_city_id: EntityId,
    pub host_city_name: String,
    pub equity: f64,
}

pub fn business_views(state: &SimState) -> Vec<BusinessView> {
    state
        .businesses
        .iter()
        .map(|business| BusinessView {
            id: business.id,
            name: business.name.clone(),
            archetype: format!("{:?}", business.archetype),
            host_city_id: business.host_city_id,
            host_city_name: city_name(state, business.host_city_id),
            equity: business.equity,
        })
        .collect()
}

/// A trade route, joined with both endpoint cities' names.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TradeRouteView {
    pub id: EntityId,
    pub city_a_id: EntityId,
    pub city_a_name: String,
    pub city_b_id: EntityId,
    pub city_b_name: String,
    pub capacity: f64,
    pub distance: f64,
    pub cost: f64,
    pub reliability: f64,
}

pub fn trade_route_views(state: &SimState) -> Vec<TradeRouteView> {
    state
        .trade_routes
        .iter()
        .map(|route| TradeRouteView {
            id: route.id,
            city_a_id: route.city_a_id,
            city_a_name: city_name(state, route.city_a_id),
            city_b_id: route.city_b_id,
            city_b_name: city_name(state, route.city_b_id),
            capacity: route.capacity,
            distance: route.distance,
            cost: route.cost,
            reliability: route.reliability,
        })
        .collect()
}

/// A character's job, joined with their name and employer city's name.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CareerView {
    pub id: EntityId,
    pub character_id: EntityId,
    pub character_name: String,
    pub track: String,
    pub job_title: String,
    pub employer_city_id: EntityId,
    pub employer_city_name: String,
}

pub fn career_views(state: &SimState) -> Vec<CareerView> {
    state
        .careers
        .iter()
        .map(|career| CareerView {
            id: career.id,
            character_id: career.character_id,
            character_name: character_name(state, career.character_id),
            track: format!("{:?}", career.track),
            job_title: career.job_title().to_string(),
            employer_city_id: career.employer_city_id,
            employer_city_name: city_name(state, career.employer_city_id),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::business::BusinessArchetype;
    use crate::career::CareerTrack;
    use crate::world::CitySpecialization;

    fn mining_city_id(state: &SimState) -> EntityId {
        state
            .sector
            .systems
            .iter()
            .flat_map(|s| &s.planets)
            .flat_map(|p| &p.countries)
            .flat_map(|c| &c.cities)
            .find(|city| city.specialization == CitySpecialization::Mining)
            .expect("seeded sector should contain a mining city")
            .id
    }

    #[test]
    fn business_view_resolves_the_host_citys_real_name() {
        let mut state = SimState::new(1);
        let city_id = mining_city_id(&state);
        let city_name_expected = city_name(&state, city_id);
        state
            .found_business(
                "Ferrous Extraction Co.".to_string(),
                BusinessArchetype::Mining,
                city_id,
            )
            .unwrap();

        let views = business_views(&state);
        assert_eq!(views.len(), 1);
        assert_eq!(views[0].host_city_name, city_name_expected);
        assert_eq!(views[0].name, "Ferrous Extraction Co.");
    }

    #[test]
    fn career_view_resolves_character_and_employer_names() {
        let mut state = SimState::new(1);
        let head_id = state.dynasty.head_character_id;
        let head_name = character_name(&state, head_id);
        let city_id = state.dynasty.members[0]
            .home_city
            .expect("founder has a home city");
        let city_name_expected = city_name(&state, city_id);
        state
            .start_career(CareerTrack::Corporate, head_id, city_id)
            .unwrap();

        let views = career_views(&state);
        assert_eq!(views.len(), 1);
        assert_eq!(views[0].character_name, head_name);
        assert_eq!(views[0].employer_city_name, city_name_expected);
        assert_eq!(views[0].job_title, "Analyst");
    }

    #[test]
    fn trade_route_view_resolves_both_endpoint_names() {
        let state = SimState::new(1);
        let views = trade_route_views(&state);
        for view in &views {
            assert_ne!(view.city_a_name, "Unknown city");
            assert_ne!(view.city_b_name, "Unknown city");
        }
    }

    #[test]
    fn pending_event_view_omits_a_stale_unknown_key_rather_than_panicking() {
        use crate::events::PendingEvent;
        let mut state = SimState::new(1);
        let head_id = state.dynasty.head_character_id;
        state.pending_events.push(PendingEvent {
            id: 999,
            key: "no-such-event".to_string(),
            character_id: head_id,
            raised_year: state.clock.year(),
        });

        assert!(pending_event_views(&state).is_empty());
    }
}
