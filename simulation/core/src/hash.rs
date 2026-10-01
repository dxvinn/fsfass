//! Stable state hashing.
//!
//! `std::hash` makes no promise of stability across Rust versions, so the
//! simulation uses its own FNV-1a 64-bit hasher and an explicit field order.

use crate::fx::Fx;

#[derive(Clone, Debug)]
pub struct StateHasher {
    h: u64,
}

impl Default for StateHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl StateHasher {
    pub const fn new() -> Self {
        StateHasher { h: 0xcbf2_9ce4_8422_2325 }
    }

    #[inline]
    pub fn bytes(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.h ^= b as u64;
            self.h = self.h.wrapping_mul(0x0000_0100_0000_01B3);
        }
    }

    #[inline]
    pub fn u64(&mut self, v: u64) {
        self.bytes(&v.to_le_bytes());
    }

    #[inline]
    pub fn i64(&mut self, v: i64) {
        self.bytes(&v.to_le_bytes());
    }

    #[inline]
    pub fn finish(&self) -> u64 {
        self.h
    }
}

/// Types that contribute to the canonical world-state hash.
pub trait StableHash {
    fn stable_hash(&self, h: &mut StateHasher);

    fn state_hash(&self) -> u64 {
        let mut h = StateHasher::new();
        self.stable_hash(&mut h);
        h.finish()
    }
}

macro_rules! impl_int {
    ($($t:ty),*) => {$(
        impl StableHash for $t {
            #[inline]
            fn stable_hash(&self, h: &mut StateHasher) { h.i64(*self as i64); }
        }
    )*};
}
impl_int!(u8, u16, u32, u64, i8, i16, i32, i64, usize);

impl StableHash for bool {
    fn stable_hash(&self, h: &mut StateHasher) {
        h.bytes(&[*self as u8]);
    }
}
impl StableHash for Fx {
    fn stable_hash(&self, h: &mut StateHasher) {
        h.i64(self.0);
    }
}
impl<T: StableHash> StableHash for Vec<T> {
    fn stable_hash(&self, h: &mut StateHasher) {
        h.u64(self.len() as u64);
        for x in self {
            x.stable_hash(h);
        }
    }
}
impl<T: StableHash, const N: usize> StableHash for [T; N] {
    fn stable_hash(&self, h: &mut StateHasher) {
        for x in self {
            x.stable_hash(h);
        }
    }
}
impl<T: StableHash> StableHash for Option<T> {
    fn stable_hash(&self, h: &mut StateHasher) {
        match self {
            None => h.bytes(&[0]),
            Some(x) => {
                h.bytes(&[1]);
                x.stable_hash(h);
            }
        }
    }
}
impl<A: StableHash, B: StableHash> StableHash for (A, B) {
    fn stable_hash(&self, h: &mut StateHasher) {
        self.0.stable_hash(h);
        self.1.stable_hash(h);
    }
}
impl<A: StableHash, B: StableHash, C: StableHash> StableHash for (A, B, C) {
    fn stable_hash(&self, h: &mut StateHasher) {
        self.0.stable_hash(h);
        self.1.stable_hash(h);
        self.2.stable_hash(h);
    }
}
