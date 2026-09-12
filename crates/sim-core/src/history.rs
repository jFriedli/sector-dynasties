//! Deterministic founding-era backstory blurbs for generated countries.
//!
//! Full procedural history (old companies, recessions, migration waves,
//! border changes) is out of scope for the bootstrap slice; see epic #78.
//! This module only gives each country a short founding-era blurb, drawn
//! deterministically from a small template pool, so the world reads as
//! though it has history behind it. See `GAME_DESIGN.md`'s "Procedural
//! history" section and issue #40.

use crate::rng::SimRng;

/// Founding-event templates. `{name}` is replaced with the country's name.
const FOUNDING_TEMPLATES: &[&str] = &[
    "{name} was founded by colonists seeking independence from the old planetary charters.",
    "{name} traces its founding to a mining consortium that outgrew its charter and declared statehood.",
    "{name} began as a refugee settlement that grew into a recognized nation.",
    "{name} was chartered by a coalition of trade guilds looking for a tax free port.",
    "{name} formed when a research colony petitioned for full sovereignty.",
];

/// Old-conflict templates.
const CONFLICT_TEMPLATES: &[&str] = &[
    "{name} spent its first decades locked in a border dispute with a neighboring colony.",
    "{name} emerged from a bloody secession war that split an older, larger state in two.",
    "{name} won its independence only after a prolonged blockade by a rival power.",
    "{name} was forged in the aftermath of a failed rebellion that toppled its predecessor government.",
    "{name} still bears the scars of a resource war fought over its home planet's mining rights.",
];

/// Merger templates.
const MERGER_TEMPLATES: &[&str] = &[
    "{name} was formed when several rival settlements agreed to a single unified charter.",
    "{name} came together as a merger of three struggling colonies pooling their resources.",
    "{name} unified an old patchwork of independent city states under one government.",
    "{name} is the product of a peaceful merger between two exhausted wartime rivals.",
    "{name} consolidated a loose alliance of trading posts into a single sovereign state.",
];

/// The pools a backstory can draw from, in a fixed order so picking is a
/// plain two-step index draw (pool, then template) rather than needing a
/// combined weighted table.
const TEMPLATE_POOLS: &[&[&str]] = &[FOUNDING_TEMPLATES, CONFLICT_TEMPLATES, MERGER_TEMPLATES];

/// Generate a short, deterministic founding-era backstory blurb for a
/// country named `country_name`, drawing from `rng`. Two `SimRng`s in the
/// same state (same world seed and domain, same number of prior draws)
/// always produce the same blurb, matching the rest of worldgen's
/// determinism guarantee.
pub fn generate_backstory(rng: &mut SimRng, country_name: &str) -> String {
    let pool = TEMPLATE_POOLS[rng.pick_index(TEMPLATE_POOLS.len())];
    let template = pool[rng.pick_index(pool.len())];
    template.replace("{name}", country_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mirrors the player-facing copy lint (`web/tests/copy-lint.test.ts`,
    /// see `docs/CONTENT_GUIDE.md`): these blurbs are shown to players (see
    /// `SectorBrowser.tsx`) but the template pool lives in `sim-core`,
    /// outside the directory that lint scans, so this test enforces the
    /// same rule directly against the source templates.
    #[test]
    fn templates_pass_the_copy_lint_rule() {
        for pool in TEMPLATE_POOLS {
            for template in *pool {
                assert!(
                    !template.contains('\u{2014}'),
                    "template contains an em dash: {template}"
                );
                assert!(
                    !template.contains(';'),
                    "template contains a semicolon: {template}"
                );
            }
        }
    }

    #[test]
    fn same_rng_state_produces_the_same_backstory() {
        let mut a = SimRng::from_seed(4242, "worldgen");
        let mut b = SimRng::from_seed(4242, "worldgen");
        assert_eq!(
            generate_backstory(&mut a, "Kestrel Union"),
            generate_backstory(&mut b, "Kestrel Union")
        );
    }

    #[test]
    fn different_draws_can_produce_different_backstories() {
        let mut rng = SimRng::from_seed(1, "worldgen");
        let blurbs: std::collections::HashSet<String> = (0..20)
            .map(|_| generate_backstory(&mut rng, "Kestrel Union"))
            .collect();
        assert!(
            blurbs.len() > 1,
            "expected varied output across draws, got {blurbs:?}"
        );
    }

    #[test]
    fn every_pool_is_reachable() {
        // Exercise enough draws that, with 3 pools, missing one would be
        // vanishingly unlikely by chance (statistical, not exhaustive).
        let mut rng = SimRng::from_seed(99, "test:history-coverage");
        let mut seen = std::collections::HashSet::new();
        for _ in 0..500 {
            let blurb = generate_backstory(&mut rng, "Test Nation");
            for (i, pool) in TEMPLATE_POOLS.iter().enumerate() {
                if pool
                    .iter()
                    .any(|t| blurb == t.replace("{name}", "Test Nation"))
                {
                    seen.insert(i);
                }
            }
        }
        assert_eq!(
            seen.len(),
            TEMPLATE_POOLS.len(),
            "not every pool was drawn from"
        );
    }

    #[test]
    fn backstory_substitutes_the_country_name() {
        let mut rng = SimRng::from_seed(7, "worldgen");
        let blurb = generate_backstory(&mut rng, "Free Cities of Oridine");
        assert!(
            blurb.starts_with("Free Cities of Oridine"),
            "expected the country name at the start of the blurb, got: {blurb}"
        );
        assert!(!blurb.contains("{name}"));
    }
}
