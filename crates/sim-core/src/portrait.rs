//! The character portrait descriptor schema.
//!
//! `PortraitDescriptor` records a character's appearance as a small set of
//! categorical and numeric traits. It is pure data: no image format, file
//! path, or external service appears anywhere in this module or is ever
//! required to construct or serialize one. This lets a future renderer
//! (procedural/layered art, a pre-generated library, or an externally
//! generated and cached image, see docs/ASSETS.md's "Character portraits"
//! section and issue #87) turn a descriptor into a picture however it
//! likes, without `sim-core` knowing or caring which approach is in use.
//!
//! Descriptors are generated deterministically from the world seed and the
//! character's id via a dedicated `SimRng` stream (domain
//! `"portrait:<id>"`), so the same seed always produces the same dynasty,
//! independent of the order other systems draw from their own streams.

use serde::{Deserialize, Serialize};

use crate::rng::SimRng;
use crate::world::EntityId;

/// Skin tone, as a small fixed palette rather than a continuous value: easy
/// to render consistently across any future art style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkinTone {
    Pale,
    Fair,
    Olive,
    Tan,
    Brown,
    Deep,
}

const SKIN_TONES: [SkinTone; 6] = [
    SkinTone::Pale,
    SkinTone::Fair,
    SkinTone::Olive,
    SkinTone::Tan,
    SkinTone::Brown,
    SkinTone::Deep,
];

/// Hair color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HairColor {
    Black,
    Brown,
    Auburn,
    Blonde,
    Gray,
    White,
}

const HAIR_COLORS: [HairColor; 6] = [
    HairColor::Black,
    HairColor::Brown,
    HairColor::Auburn,
    HairColor::Blonde,
    HairColor::Gray,
    HairColor::White,
];

/// Hair style, independent of color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HairStyle {
    Shaved,
    Short,
    Long,
    Braided,
    Tied,
}

const HAIR_STYLES: [HairStyle; 5] = [
    HairStyle::Shaved,
    HairStyle::Short,
    HairStyle::Long,
    HairStyle::Braided,
    HairStyle::Tied,
];

/// Eye color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EyeColor {
    Brown,
    Hazel,
    Green,
    Blue,
    Gray,
    Amber,
}

const EYE_COLORS: [EyeColor; 6] = [
    EyeColor::Brown,
    EyeColor::Hazel,
    EyeColor::Green,
    EyeColor::Blue,
    EyeColor::Gray,
    EyeColor::Amber,
];

/// Overall build.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Build {
    Slight,
    Average,
    Athletic,
    Heavy,
}

const BUILDS: [Build; 4] = [Build::Slight, Build::Average, Build::Athletic, Build::Heavy];

/// A small, serializable set of appearance traits describing a character's
/// look, independent of any rendering technique.
///
/// Every field here is a plain enum or a bounded integer. There is
/// intentionally no field capable of referencing an image, a file path, or
/// a network resource: a future renderer decides what a `PortraitDescriptor`
/// looks like when drawn, and `sim-core` never needs to know.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortraitDescriptor {
    pub skin_tone: SkinTone,
    pub hair_color: HairColor,
    pub hair_style: HairStyle,
    pub eye_color: EyeColor,
    pub build: Build,
    /// Height in centimeters, kept in a plausible adult human range.
    pub height_cm: u16,
}

const MIN_HEIGHT_CM: u16 = 150;
const MAX_HEIGHT_CM: u16 = 200;

impl PortraitDescriptor {
    /// Draw a new descriptor from an already-derived `SimRng` stream. The
    /// caller decides how that stream was seeded; see `generate_for_character`
    /// for the standard per-character derivation.
    pub fn generate(rng: &mut SimRng) -> Self {
        let height_span = (MAX_HEIGHT_CM - MIN_HEIGHT_CM) as u32 + 1;
        PortraitDescriptor {
            skin_tone: SKIN_TONES[rng.pick_index(SKIN_TONES.len())],
            hair_color: HAIR_COLORS[rng.pick_index(HAIR_COLORS.len())],
            hair_style: HAIR_STYLES[rng.pick_index(HAIR_STYLES.len())],
            eye_color: EYE_COLORS[rng.pick_index(EYE_COLORS.len())],
            build: BUILDS[rng.pick_index(BUILDS.len())],
            height_cm: MIN_HEIGHT_CM + rng.next_below(height_span) as u16,
        }
    }

    /// Deterministically generate the descriptor for `character_id` under
    /// `world_seed`. Each character gets its own independent stream (domain
    /// `"portrait:<id>"`), so generating one character's portrait never
    /// shifts another's, regardless of creation order.
    pub fn generate_for_character(world_seed: u64, character_id: EntityId) -> Self {
        let mut rng = SimRng::from_seed(world_seed, &format!("portrait:{character_id}"));
        Self::generate(&mut rng)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_and_character_id_generate_the_same_descriptor() {
        let a = PortraitDescriptor::generate_for_character(42, 7);
        let b = PortraitDescriptor::generate_for_character(42, 7);
        assert_eq!(a, b);
    }

    #[test]
    fn different_character_ids_diverge_even_with_the_same_seed() {
        let a = PortraitDescriptor::generate_for_character(42, 1);
        let b = PortraitDescriptor::generate_for_character(42, 2);
        assert_ne!(a, b);
    }

    #[test]
    fn generating_one_characters_portrait_does_not_affect_a_sibling() {
        let alone = PortraitDescriptor::generate_for_character(99, 5);

        // Generate a handful of unrelated characters' portraits first, in
        // both orders, to prove the streams are independent.
        let _ = PortraitDescriptor::generate_for_character(99, 1);
        let _ = PortraitDescriptor::generate_for_character(99, 2);
        let _ = PortraitDescriptor::generate_for_character(99, 3);
        let after_siblings = PortraitDescriptor::generate_for_character(99, 5);

        assert_eq!(alone, after_siblings);
    }

    #[test]
    fn height_stays_within_the_plausible_adult_range() {
        for id in 0..500 {
            let descriptor = PortraitDescriptor::generate_for_character(1, id);
            assert!(descriptor.height_cm >= MIN_HEIGHT_CM);
            assert!(descriptor.height_cm <= MAX_HEIGHT_CM);
        }
    }

    #[test]
    fn descriptor_round_trips_through_json_exactly() {
        let original = PortraitDescriptor::generate_for_character(2026, 3);
        let json = serde_json::to_string(&original).expect("descriptor always serializes");
        let restored: PortraitDescriptor =
            serde_json::from_str(&json).expect("descriptor always deserializes");
        assert_eq!(original, restored);
    }

    #[test]
    fn serialized_descriptor_contains_no_image_or_network_hints() {
        // A cheap guard against scope creep: this schema is data only. If a
        // future edit adds a path, URL, or file-like field, this test's
        // author intent is to catch it early (a determined renderer can
        // still smuggle something in, but nothing here should hint at
        // needing one).
        let json = serde_json::to_string(&PortraitDescriptor::generate_for_character(1, 1))
            .expect("descriptor always serializes");
        for needle in [
            "http://", "https://", ".png", ".jpg", ".jpeg", ".webp", "path", "url",
        ] {
            assert!(
                !json.to_lowercase().contains(needle),
                "descriptor JSON unexpectedly contains {needle:?}: {json}"
            );
        }
    }
}
