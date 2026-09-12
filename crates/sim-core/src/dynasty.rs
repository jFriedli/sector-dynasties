//! The player's dynasty. The player always controls the current family
//! head; other characters are simulated but not directly controlled.

use serde::{Deserialize, Serialize};

use crate::portrait::PortraitDescriptor;
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
}

impl Character {
    pub fn is_valid(&self) -> bool {
        self.wealth.is_finite() && (self.alive || self.wealth >= 0.0)
    }
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
                },
                Character {
                    id: 2,
                    name: "Heir".into(),
                    age_years: 34,
                    alive: true,
                    wealth: 1200.0,
                    home_city: Some(1),
                    portrait: PortraitDescriptor::generate_for_character(0, 2),
                },
            ],
        };
        assert_eq!(dynasty.head().unwrap().name, "Heir");
        assert_eq!(dynasty.total_wealth(), 1200.0);
    }
}
