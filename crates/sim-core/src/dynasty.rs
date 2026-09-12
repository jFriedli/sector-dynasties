//! The player's dynasty. The player always controls the current family
//! head; other characters are simulated but not directly controlled.

use serde::{Deserialize, Serialize};

use crate::portrait::PortraitDescriptor;
use crate::traits::Trait;
use crate::world::EntityId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub id: EntityId,
    pub name: String,
    pub age_years: u32,
    pub alive: bool,
    pub wealth: f64,
    /// City where this character currently resides, if any.
    pub home_city: Option<EntityId>,
    /// Appearance descriptor, independent of any rendering technique. See
    /// `crate::portrait`.
    pub portrait: PortraitDescriptor,
    /// Small fixed set of personality traits. Never empty for a character
    /// created via `crate::traits::generate_for_character`, but not
    /// enforced as non-empty by construction here (a bare test fixture may
    /// reasonably leave it empty); duplicates are rejected by `is_valid`.
    pub traits: Vec<Trait>,
}

impl Character {
    pub fn is_valid(&self) -> bool {
        self.wealth.is_finite()
            && (self.alive || self.wealth >= 0.0)
            && has_no_duplicate_traits(&self.traits)
    }
}

fn has_no_duplicate_traits(traits: &[Trait]) -> bool {
    for (i, a) in traits.iter().enumerate() {
        for b in &traits[i + 1..] {
            if a == b {
                return false;
            }
        }
    }
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dynasty {
    pub name: String,
    pub head_character_id: EntityId,
    pub members: Vec<Character>,
}

impl Dynasty {
    pub fn head(&self) -> Option<&Character> {
        self.members.iter().find(|c| c.id == self.head_character_id)
    }

    pub fn total_wealth(&self) -> f64 {
        self.members.iter().map(|c| c.wealth).sum()
    }

    /// A compact, UI/CLI-friendly view of every member, for the dynasty
    /// panel (see issue #31). Kept alongside `StateSummary` so the wasm
    /// bridge never has to reach around it for member detail.
    pub fn member_summaries(&self) -> Vec<DynastyMemberSummary> {
        self.members
            .iter()
            .map(|character| DynastyMemberSummary {
                id: character.id,
                name: character.name.clone(),
                age_years: character.age_years,
                alive: character.alive,
                role: if character.id == self.head_character_id {
                    DynastyRole::Head
                } else {
                    DynastyRole::Member
                },
            })
            .collect()
    }
}

/// A single dynasty member's row in the dynasty panel: enough to render
/// name, age, alive/dead, and role without exposing the full `Character`
/// (portrait descriptor, traits, wealth, and so on aren't needed there).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DynastyMemberSummary {
    pub id: EntityId,
    pub name: String,
    pub age_years: u32,
    pub alive: bool,
    pub role: DynastyRole,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DynastyRole {
    Head,
    Member,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn head_lookup_finds_the_matching_member() {
        let dynasty = Dynasty {
            name: "House Vantar".into(),
            head_character_id: 2,
            members: vec![
                Character {
                    id: 1,
                    name: "Elder".into(),
                    age_years: 70,
                    alive: false,
                    wealth: 0.0,
                    home_city: None,
                    portrait: PortraitDescriptor::generate_for_character(0, 1),
                    traits: crate::traits::generate_for_character(0, 1),
                },
                Character {
                    id: 2,
                    name: "Heir".into(),
                    age_years: 34,
                    alive: true,
                    wealth: 1200.0,
                    home_city: Some(1),
                    portrait: PortraitDescriptor::generate_for_character(0, 2),
                    traits: crate::traits::generate_for_character(0, 2),
                },
            ],
        };
        assert_eq!(dynasty.head().unwrap().name, "Heir");
        assert_eq!(dynasty.total_wealth(), 1200.0);
    }

    #[test]
    fn member_summaries_mark_the_head_and_preserve_alive_state() {
        let dynasty = Dynasty {
            name: "House Vantar".into(),
            head_character_id: 2,
            members: vec![
                Character {
                    id: 1,
                    name: "Elder".into(),
                    age_years: 70,
                    alive: false,
                    wealth: 0.0,
                    home_city: None,
                    portrait: PortraitDescriptor::generate_for_character(0, 1),
                    traits: crate::traits::generate_for_character(0, 1),
                },
                Character {
                    id: 2,
                    name: "Heir".into(),
                    age_years: 34,
                    alive: true,
                    wealth: 1200.0,
                    home_city: Some(1),
                    portrait: PortraitDescriptor::generate_for_character(0, 2),
                    traits: crate::traits::generate_for_character(0, 2),
                },
            ],
        };

        let summaries = dynasty.member_summaries();
        assert_eq!(
            summaries,
            vec![
                DynastyMemberSummary {
                    id: 1,
                    name: "Elder".into(),
                    age_years: 70,
                    alive: false,
                    role: DynastyRole::Member,
                },
                DynastyMemberSummary {
                    id: 2,
                    name: "Heir".into(),
                    age_years: 34,
                    alive: true,
                    role: DynastyRole::Head,
                },
            ]
        );
    }

    #[test]
    fn a_character_with_duplicate_traits_is_invalid() {
        let mut character = Character {
            id: 1,
            name: "Test".into(),
            age_years: 30,
            alive: true,
            wealth: 0.0,
            home_city: None,
            portrait: PortraitDescriptor::generate_for_character(0, 1),
            traits: vec![Trait::Ambitious],
        };
        assert!(character.is_valid());

        character.traits.push(Trait::Ambitious);
        assert!(!character.is_valid());
    }
}
