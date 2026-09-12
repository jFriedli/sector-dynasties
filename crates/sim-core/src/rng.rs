//! Deterministic, dependency-free random number generation.
//!
//! The simulation must be reproducible from a seed. Rather than one global
//! RNG (where an unrelated system consuming an extra random number would
//! shift every later draw), each simulation domain gets its own stream
//! derived from the world seed plus a domain tag. Splitmix64 is used as the
//! stream splitter and xoshiro256** as the generator; both are small,
//! well-known, and easy to audit without pulling in a dependency.

use serde::{Deserialize, Serialize};

/// Splits a single u64 seed into an arbitrary number of well-distributed
/// u64s, used to derive independent stream seeds from one world seed.
fn splitmix64_next(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

/// A named, independent deterministic random stream.
///
/// Two `SimRng`s constructed from the same world seed and the same
/// `domain` tag always produce the same sequence, regardless of what any
/// other domain's stream has consumed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimRng {
    state: [u64; 4],
}

impl SimRng {
    /// Derive a stream for `domain` from a world seed. `domain` should be a
    /// short stable string such as `"worldgen"`, `"economy"`, or
    /// `"events:character:42"` for a per-character stream.
    pub fn from_seed(world_seed: u64, domain: &str) -> Self {
        let mut mix = world_seed ^ fnv1a64(domain.as_bytes());
        let mut state = [0u64; 4];
        for slot in &mut state {
            *slot = splitmix64_next(&mut mix);
        }
        // xoshiro256** requires a non-zero state.
        if state.iter().all(|s| *s == 0) {
            state[0] = 1;
        }
        SimRng { state }
    }

    fn next_u64(&mut self) -> u64 {
        let s = &mut self.state;
        let result = s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = s[1] << 17;
        s[2] ^= s[0];
        s[3] ^= s[1];
        s[1] ^= s[2];
        s[0] ^= s[3];
        s[2] ^= t;
        s[3] = s[3].rotate_left(45);
        result
    }

    /// Uniform integer in `[0, bound)`. `bound` must be > 0.
    pub fn next_below(&mut self, bound: u32) -> u32 {
        assert!(bound > 0, "SimRng::next_below requires bound > 0");
        (self.next_u64() % bound as u64) as u32
    }

    /// Uniform float in `[0.0, 1.0)`.
    pub fn next_f64(&mut self) -> f64 {
        // Use the top 53 bits for a uniform double, matching common
        // practice for converting a 64-bit stream into f64.
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    /// Uniform float in `[min, max)`.
    pub fn range_f64(&mut self, min: f64, max: f64) -> f64 {
        min + self.next_f64() * (max - min)
    }

    /// True with probability `p` (`p` clamped to `[0, 1]`).
    pub fn chance(&mut self, p: f64) -> bool {
        self.next_f64() < p.clamp(0.0, 1.0)
    }

    /// Pick an index into a slice of length `len` (panics if `len == 0`).
    pub fn pick_index(&mut self, len: usize) -> usize {
        self.next_below(len as u32) as usize
    }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in bytes {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_and_domain_reproduce_the_same_sequence() {
        let mut a = SimRng::from_seed(42, "economy");
        let mut b = SimRng::from_seed(42, "economy");
        let seq_a: Vec<u64> = (0..50).map(|_| a.next_u64()).collect();
        let seq_b: Vec<u64> = (0..50).map(|_| b.next_u64()).collect();
        assert_eq!(seq_a, seq_b);
    }

    #[test]
    fn different_domains_diverge_even_with_the_same_seed() {
        let mut a = SimRng::from_seed(42, "economy");
        let mut b = SimRng::from_seed(42, "worldgen");
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn consuming_one_stream_does_not_affect_a_sibling_stream() {
        let world_seed = 7;
        let mut worldgen_untouched = SimRng::from_seed(world_seed, "worldgen");

        let mut economy = SimRng::from_seed(world_seed, "economy");
        for _ in 0..1000 {
            economy.next_u64();
        }
        let mut worldgen_after = SimRng::from_seed(world_seed, "worldgen");

        assert_eq!(worldgen_untouched.next_u64(), worldgen_after.next_u64());
    }

    #[test]
    fn next_below_stays_in_bounds() {
        let mut rng = SimRng::from_seed(1, "test");
        for _ in 0..1000 {
            assert!(rng.next_below(6) < 6);
        }
    }

    // --- Property-style sanity checks -------------------------------------
    //
    // These use hand-rolled large-sample checks rather than a
    // proptest/quickcheck-style crate. Deterministic simulation RNGs like
    // this one have a fixed, well-understood output space (a bounded f64,
    // an integer below a bound), so generating and checking a large batch of
    // samples from a few fixed seeds is simpler and just as effective at
    // catching a broken shift/mask as shrink-based property testing, without
    // adding a new workspace dependency for one test module.

    #[test]
    fn next_f64_stays_in_unit_interval_and_is_reasonably_uniform() {
        let mut rng = SimRng::from_seed(99, "property:next_f64");
        const SAMPLES: usize = 200_000;
        const BUCKETS: usize = 10;

        let mut counts = [0u32; BUCKETS];
        for _ in 0..SAMPLES {
            let x = rng.next_f64();
            assert!(
                (0.0..1.0).contains(&x),
                "next_f64 produced {x}, outside [0, 1)"
            );
            let bucket = ((x * BUCKETS as f64) as usize).min(BUCKETS - 1);
            counts[bucket] += 1;
        }

        // Chi-square-style uniformity check, not a strict statistical
        // proof: with a truly uniform generator this statistic follows a
        // chi-square distribution with BUCKETS - 1 = 9 degrees of freedom,
        // averaging 9 and exceeding ~27.9 only about one time in a
        // thousand by chance. A generous threshold well above that keeps
        // a genuinely uniform generator from ever tripping this while
        // still catching a badly broken shift/mask (e.g. one that leaves
        // output skewed into half the range or clustered in a few
        // buckets).
        let expected = SAMPLES as f64 / BUCKETS as f64;
        let chi_square: f64 = counts
            .iter()
            .map(|&c| {
                let diff = c as f64 - expected;
                diff * diff / expected
            })
            .sum();
        assert!(
            chi_square < 50.0,
            "next_f64 output is not reasonably uniform across buckets \
             (chi-square statistic {chi_square:.2}, bucket counts {counts:?})"
        );
    }

    #[test]
    fn next_below_never_escapes_its_bound() {
        let mut rng = SimRng::from_seed(123, "property:next_below");
        const SAMPLES: usize = 50_000;

        for &bound in &[1u32, 2, 3, 7, 16, 100, 10_000] {
            let mut seen_nonzero = false;
            for _ in 0..SAMPLES {
                let v = rng.next_below(bound);
                assert!(
                    v < bound,
                    "next_below({bound}) produced {v}, outside [0, {bound})"
                );
                seen_nonzero |= v != 0;
            }
            // Sanity check the sample is not degenerate (e.g. a generator
            // that always returns 0 would still pass the bound check above).
            if bound > 1 {
                assert!(
                    seen_nonzero,
                    "next_below({bound}) returned 0 for all {SAMPLES} samples"
                );
            }
        }
    }
}
