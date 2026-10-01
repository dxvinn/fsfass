//! Counter-based random numbers.
//!
//! A random value is a pure function of (world seed, entity, tick, stream, counter).
//! There is no shared mutable generator, so results do not depend on the order in
//! which entities or threads are processed, and any draw can be reproduced later.

use crate::fx::Fx;

/// SplitMix64 finalizer: a fast, well-distributed 64-bit mixing function.
#[inline]
pub fn mix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Combine a seed with three keys into one 64-bit key.
#[inline]
pub fn key4(seed: u64, a: u64, b: u64, c: u64) -> u64 {
    mix64(seed ^ mix64(a ^ mix64(b ^ mix64(c.wrapping_add(0x51_7CC1_B727_220A)))))
}

/// Well-known stream identifiers, so that unrelated systems never share draws.
pub mod stream {
    pub const WORLD_SETUP: u64 = 1;
    pub const PHYSICS: u64 = 2;
    pub const SENSOR_NOISE: u64 = 3;
    pub const MOTOR: u64 = 4;
    pub const DRIVER: u64 = 5;
    pub const DECIDE: u64 = 10;
    pub const LEARN: u64 = 11;
    pub const MEMORY: u64 = 12;
    pub const PERSONALITY: u64 = 13;
    pub const TOKEN_SALT: u64 = 20;
}

/// A short-lived generator for one (seed, entity, tick, stream) context.
#[derive(Clone, Copy, Debug)]
pub struct Rng {
    key: u64,
    ctr: u64,
}

impl Rng {
    #[inline]
    pub fn new(seed: u64, entity: u64, tick: u64, stream: u64) -> Rng {
        Rng { key: key4(seed, entity, tick, stream), ctr: 0 }
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        self.ctr += 1;
        mix64(self.key ^ self.ctr.wrapping_mul(0xD1B5_4A32_D192_ED03))
    }

    /// Uniform in [0, 1) with 16 bits of resolution (exactly representable in Fx).
    #[inline]
    pub fn unit(&mut self) -> Fx {
        Fx((self.next_u64() >> 48) as i64)
    }

    /// Uniform in [lo, hi).
    #[inline]
    pub fn range_fx(&mut self, lo: Fx, hi: Fx) -> Fx {
        lo + (hi - lo) * self.unit()
    }

    /// Uniform integer in [0, n).
    #[inline]
    pub fn below(&mut self, n: u64) -> u64 {
        if n == 0 {
            return 0;
        }
        ((self.next_u64() as u128 * n as u128) >> 64) as u64
    }

    /// Bernoulli trial with probability p.
    #[inline]
    pub fn chance(&mut self, p: Fx) -> bool {
        self.unit() < p
    }

    /// Approximately normal(0, 1): sum of 4 uniforms, rescaled (Irwin-Hall).
    pub fn gaussish(&mut self) -> Fx {
        let s = self.unit() + self.unit() + self.unit() + self.unit();
        // mean 2, variance 4/12 -> sd 0.57735
        (s - Fx::from_int(2)) * Fx::from_f64(1.7320508)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_key_same_sequence() {
        let mut a = Rng::new(7, 3, 100, stream::DECIDE);
        let mut b = Rng::new(7, 3, 100, stream::DECIDE);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_streams_differ() {
        let mut a = Rng::new(7, 3, 100, stream::DECIDE);
        let mut b = Rng::new(7, 3, 100, stream::LEARN);
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn unit_is_in_range_and_roughly_uniform() {
        let mut r = Rng::new(1, 2, 3, 4);
        let mut sum = Fx::ZERO;
        for _ in 0..10_000 {
            let u = r.unit();
            assert!(u >= Fx::ZERO && u < Fx::ONE);
            sum += u;
        }
        let mean = sum.divi(10_000).to_f64();
        assert!((mean - 0.5).abs() < 0.02, "mean {mean}");
    }
}
