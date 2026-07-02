//! Random number generation utilities for the client.
//!
//! The client draws randomness through the [`FeltRng`] trait. [`FeltRngAdapter`] lifts any
//! [`RngCore`] into a [`FeltRng`], and [`DefaultFeltRng`] is the client's default RNG, backed by
//! a `ChaCha20` generator.

use miden_protocol::crypto::rand::FeltRng;
use miden_protocol::{Felt, Word};
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;

// FELT RNG ADAPTER
// ================================================================================================

/// Adapts any [`RngCore`] into a [`FeltRng`].
///
/// Field elements are produced by rejection sampling over `[0, Felt::ORDER)`, yielding a uniform
/// distribution over the field. A single draw is rejected with probability about `2^-32`.
#[derive(Debug, Clone)]
pub struct FeltRngAdapter<R>(R);

impl<R: RngCore> FeltRngAdapter<R> {
    /// Wraps the given [`RngCore`] so it can be used as a [`FeltRng`].
    pub fn new(rng: R) -> Self {
        Self(rng)
    }
}

impl<R: RngCore> RngCore for FeltRngAdapter<R> {
    fn next_u32(&mut self) -> u32 {
        self.0.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.0.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.0.fill_bytes(dest);
    }
}

impl<R: RngCore> FeltRng for FeltRngAdapter<R> {
    fn draw_element(&mut self) -> Felt {
        // Rejection sampling keeps the draw uniform over the field by discarding the few `u64`
        // values that fall outside the canonical range `[0, Felt::ORDER)`. The retained value is
        // already canonical, so `new_unchecked` avoids a redundant reduction.
        loop {
            let value = self.0.next_u64();
            if value < Felt::ORDER {
                return Felt::new_unchecked(value);
            }
        }
    }

    fn draw_word(&mut self) -> Word {
        [
            self.draw_element(),
            self.draw_element(),
            self.draw_element(),
            self.draw_element(),
        ]
        .into()
    }
}

// DEFAULT FELT RNG
// ================================================================================================

/// The client's default [`FeltRng`], backed by a `ChaCha20` generator.
///
/// `ChaCha20` is value-stable across releases, so an instance built from a fixed seed reproduces
/// the same sequence of elements. This makes it suitable for deterministic testing.
pub type DefaultFeltRng = FeltRngAdapter<ChaCha20Rng>;

impl DefaultFeltRng {
    /// Creates a [`DefaultFeltRng`] from a 32-byte seed.
    pub fn from_seed(seed: [u8; 32]) -> Self {
        FeltRngAdapter(ChaCha20Rng::from_seed(seed))
    }
}
