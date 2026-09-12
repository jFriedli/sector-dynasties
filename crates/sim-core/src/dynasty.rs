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
