//! Named seed presets for reproducible demo scenarios.
//!
//! Raw u64 seeds are fine for one-off exploration, but reviewers and bug
//! reports benefit from a small, stable vocabulary: "run the sprawling
//! preset" is easier to communicate and remember than "seed 200, 6
//! systems". Keep this table small (3-5 entries): these are for demos and
//! reproduction steps, not a scenario library. See issue #21.

/// A named, reproducible starting scenario: a seed plus the system count to
/// generate it with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeedPreset {
    pub name: &'static str,
    pub seed: u64,
    pub system_count: u32,
    pub description: &'static str,
}

/// The full table of known presets, sampled and picked by hand from
/// `worldgen::generate_sector` output so each name matches its description.
pub const PRESETS: &[SeedPreset] = &[
    SeedPreset {
        name: "tutorial",
        seed: 42,
        system_count: 2,
        description: "Small two-system start with a handful of cities, easy to narrate in a walkthrough.",
    },
    SeedPreset {
        name: "resource-poor",
        seed: 9999,
        system_count: 2,
        description: "Below-average city wealth throughout, good for stress-testing tight budgets.",
    },
    SeedPreset {
        name: "sprawling",
        seed: 200,
        system_count: 6,
        description: "Six systems and dozens of cities, useful for UI and performance testing at scale.",
    },
    SeedPreset {
        name: "single-system",
        seed: 1000,
        system_count: 1,
        description: "One system with a few cities, a minimal sandbox for fast iteration.",
    },
    SeedPreset {
        name: "prosperous",
        seed: 55,
        system_count: 1,
        description: "The highest average city wealth of these presets, useful for late-game economy scenarios.",
    },
];

/// Look up a preset by its exact name. Returns `None` for anything not in
/// `PRESETS`, including a close-but-wrong spelling: callers should surface
/// the full list on a miss rather than guessing.
pub fn resolve(name: &str) -> Option<&'static SeedPreset> {
    PRESETS.iter().find(|preset| preset.name == name)
}

/// The preset names in table order, handy for usage/error messages.
pub fn names() -> Vec<&'static str> {
    PRESETS.iter().map(|preset| preset.name).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::worldgen::generate_sector;

    #[test]
    fn every_preset_resolves_by_name() {
        for preset in PRESETS {
            let resolved = resolve(preset.name).unwrap_or_else(|| {
                panic!("preset {} did not resolve by its own name", preset.name)
            });
            assert_eq!(resolved.seed, preset.seed);
            assert_eq!(resolved.system_count, preset.system_count);
        }
    }

    #[test]
    fn unknown_preset_name_resolves_to_none() {
        assert!(resolve("does-not-exist").is_none());
    }

    #[test]
    fn each_preset_produces_a_non_empty_and_deterministic_sector() {
        for preset in PRESETS {
            let a = generate_sector(preset.seed, preset.system_count);
            let b = generate_sector(preset.seed, preset.system_count);

            assert_eq!(
                serde_json::to_string(&a).unwrap(),
                serde_json::to_string(&b).unwrap(),
                "preset {} is not deterministic",
                preset.name
            );

            assert!(
                !a.systems.is_empty(),
                "preset {} produced no systems",
                preset.name
            );
            let city_count: usize = a
                .systems
                .iter()
                .flat_map(|s| &s.planets)
                .flat_map(|p| &p.countries)
                .map(|c| c.cities.len())
                .sum();
            assert!(city_count > 0, "preset {} produced no cities", preset.name);
        }
    }

    #[test]
    fn table_stays_small_and_names_are_unique() {
        assert!(
            PRESETS.len() >= 3 && PRESETS.len() <= 5,
            "keep the preset table to 3-5 entries"
        );
        let mut seen = std::collections::HashSet::new();
        for preset in PRESETS {
            assert!(
                seen.insert(preset.name),
                "duplicate preset name: {}",
                preset.name
            );
        }
    }

    #[test]
    fn names_lists_every_preset_in_table_order() {
        let listed = names();
        let expected: Vec<&str> = PRESETS.iter().map(|p| p.name).collect();
        assert_eq!(listed, expected);
    }
}
