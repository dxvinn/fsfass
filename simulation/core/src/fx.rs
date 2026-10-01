//! Deterministic fixed-point number: 64-bit signed integer with 32 fractional bits
//! (range about +/-5e8, resolution 2.3e-10; 16 bits proved too coarse for slow
//! per-second processes such as healing and sleep pressure).
//!
//! Every quantity that is part of simulation state uses `Fx`, so results are
//! bit-identical on every platform, thread count and compiler that implements
//! two's-complement integer arithmetic. Transcendental functions (exp, ln,
//! sigmoid, sqrt) are implemented with integer arithmetic only.
//!
//! `from_f64` is allowed for configuration constants (IEEE-754 conversion of a
//! literal is deterministic); floats must never be used in a running simulation.

use std::fmt;
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub, SubAssign};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Fx(pub i64);

const FRAC: u32 = 32;
/// Internal precision for transcendental helpers (Q40 inside i128).
const P: u32 = 40;
const P_ONE: i128 = 1 << P;

impl Fx {
    pub const ZERO: Fx = Fx(0);
    pub const ONE: Fx = Fx(1 << FRAC);
    pub const HALF: Fx = Fx(1 << (FRAC - 1));
    pub const NEG_ONE: Fx = Fx(-(1 << FRAC));
    pub const MAX: Fx = Fx(i64::MAX / 4);
    pub const MIN: Fx = Fx(i64::MIN / 4);
    pub const EPS: Fx = Fx(1);

    #[inline]
    pub const fn from_int(i: i64) -> Fx {
        Fx(i << FRAC)
    }

    /// Converts a ratio num/den exactly (rounded toward zero).
    #[inline]
    pub const fn ratio(num: i64, den: i64) -> Fx {
        Fx((((num as i128) << FRAC) / den as i128) as i64)
    }

    /// For configuration constants and test fixtures only.
    #[inline]
    pub fn from_f64(f: f64) -> Fx {
        Fx((f * (1u64 << FRAC) as f64).round() as i64)
    }

    /// For reports and display only. Never feed the result back into the simulation.
    #[inline]
    pub fn to_f64(self) -> f64 {
        self.0 as f64 / (1u64 << FRAC) as f64
    }

    #[inline]
    pub const fn raw(self) -> i64 {
        self.0
    }

    #[inline]
    pub fn floor_int(self) -> i64 {
        self.0 >> FRAC
    }

    #[inline]
    pub fn abs(self) -> Fx {
        Fx(self.0.abs())
    }

    #[inline]
    pub fn min(self, o: Fx) -> Fx {
        if self.0 <= o.0 {
            self
        } else {
            o
        }
    }

    #[inline]
    pub fn max(self, o: Fx) -> Fx {
        if self.0 >= o.0 {
            self
        } else {
            o
        }
    }

    #[inline]
    pub fn clamp(self, lo: Fx, hi: Fx) -> Fx {
        self.max(lo).min(hi)
    }

    #[inline]
    pub fn clamp01(self) -> Fx {
        self.clamp(Fx::ZERO, Fx::ONE)
    }

    #[inline]
    pub fn signum(self) -> Fx {
        if self.0 > 0 {
            Fx::ONE
        } else if self.0 < 0 {
            Fx::NEG_ONE
        } else {
            Fx::ZERO
        }
    }

    #[inline]
    pub fn is_positive(self) -> bool {
        self.0 > 0
    }

    #[inline]
    pub fn is_negative(self) -> bool {
        self.0 < 0
    }

    /// Multiply by an integer.
    #[inline]
    pub fn muli(self, k: i64) -> Fx {
        Fx(self.0.saturating_mul(k))
    }

    /// Divide by an integer.
    #[inline]
    pub fn divi(self, k: i64) -> Fx {
        Fx(self.0 / k)
    }

    /// Linear interpolation a + (b - a) * t.
    #[inline]
    pub fn lerp(a: Fx, b: Fx, t: Fx) -> Fx {
        a + (b - a) * t
    }

    /// e^x. Saturates for large x; returns 0 for very negative x.
    pub fn exp(self) -> Fx {
        if self.0 <= -(40i64 << FRAC) {
            return Fx::ZERO;
        }
        if self.0 >= (20i64 << FRAC) {
            return Fx::MAX;
        }
        // y = x * log2(e) in Q40
        const LOG2E_Q40: i128 = 1_586_259_972_792; // 1.4426950408889634 * 2^40
        let y: i128 = (((self.0 as i128) << (P - FRAC)) * LOG2E_Q40) >> P;
        let k = y >> P; // floor
        let f = y - (k << P); // in [0, 2^40)
        let p = exp2_frac(f); // 2^f in Q40, [1,2)
        // result = p * 2^k, converted from Q40 to Q32
        let shift = k as i64 + FRAC as i64 - P as i64;
        let v = if shift >= 0 { p << shift } else { p >> (-shift) };
        Fx(v.min(Fx::MAX.0 as i128) as i64)
    }

    /// Natural logarithm. Returns a large negative number for x <= 0.
    pub fn ln(self) -> Fx {
        if self.0 <= 0 {
            return Fx::from_int(-40);
        }
        // x = m * 2^e with m in [1,2)
        let msb = 63 - self.0.leading_zeros() as i64;
        let e = msb - FRAC as i64;
        let raw = self.0 as i128;
        let m: i128 = if msb >= P as i64 { raw >> (msb - P as i64) } else { raw << (P as i64 - msb) };
        // ln(m) = 2 * atanh(z), z = (m-1)/(m+1) in [0, 1/3]
        let z = ((m - P_ONE) << P) / (m + P_ONE);
        let z2 = (z * z) >> P;
        let mut term = z;
        let mut sum: i128 = 0;
        let mut k: i128 = 1;
        for _ in 0..12 {
            sum += term / k;
            term = (term * z2) >> P;
            k += 2;
        }
        const LN2_Q40: i128 = 762_123_384_786; // 0.6931471805599453 * 2^40
        let total = 2 * sum + (e as i128) * LN2_Q40; // Q40
        Fx((total >> (P - FRAC)) as i64)
    }

    /// Logistic sigmoid 1 / (1 + e^-x).
    pub fn sigmoid(self) -> Fx {
        if self.0 >= 0 {
            let e = (-self).exp();
            Fx::ONE / (Fx::ONE + e)
        } else {
            let e = self.exp();
            e / (Fx::ONE + e)
        }
    }

    /// Square root (x >= 0; negative input returns 0).
    pub fn sqrt(self) -> Fx {
        if self.0 <= 0 {
            return Fx::ZERO;
        }
        let v = (self.0 as u128) << FRAC;
        Fx(isqrt_u128(v) as i64)
    }

    /// x^k for small non-negative integer k.
    pub fn powi(self, k: u32) -> Fx {
        let mut r = Fx::ONE;
        for _ in 0..k {
            r = r * self;
        }
        r
    }

    /// Exponential decay factor e^(-dt / tau). tau must be positive.
    pub fn decay(dt: Fx, tau: Fx) -> Fx {
        if tau.0 <= 0 {
            return Fx::ZERO;
        }
        (-(dt / tau)).exp()
    }
}

/// 2^f for f in [0,1) given in Q40; result in Q40 within [1, 2).
fn exp2_frac(f: i128) -> i128 {
    const LN2_Q40: i128 = 762_123_384_786;
    let x: i128 = (f * LN2_Q40) >> P; // in [0, ln2)
    let mut term: i128 = P_ONE;
    let mut sum: i128 = P_ONE;
    for n in 1..=16i128 {
        term = ((term * x) >> P) / n;
        sum += term;
        if term == 0 {
            break;
        }
    }
    sum
}

fn isqrt_u128(n: u128) -> u128 {
    if n == 0 {
        return 0;
    }
    let mut x: u128 = 1u128 << ((128 - n.leading_zeros()) / 2 + 1);
    loop {
        let y = (x + n / x) / 2;
        if y >= x {
            return x;
        }
        x = y;
    }
}

impl Add for Fx {
    type Output = Fx;
    #[inline]
    fn add(self, o: Fx) -> Fx {
        Fx(self.0.saturating_add(o.0))
    }
}
impl Sub for Fx {
    type Output = Fx;
    #[inline]
    fn sub(self, o: Fx) -> Fx {
        Fx(self.0.saturating_sub(o.0))
    }
}
impl Neg for Fx {
    type Output = Fx;
    #[inline]
    fn neg(self) -> Fx {
        Fx(-self.0)
    }
}
impl Mul for Fx {
    type Output = Fx;
    #[inline]
    fn mul(self, o: Fx) -> Fx {
        let v = (self.0 as i128 * o.0 as i128) >> FRAC;
        Fx(v.clamp(i64::MIN as i128 / 4, i64::MAX as i128 / 4) as i64)
    }
}
impl Div for Fx {
    type Output = Fx;
    #[inline]
    fn div(self, o: Fx) -> Fx {
        if o.0 == 0 {
            return if self.0 >= 0 { Fx::MAX } else { Fx::MIN };
        }
        let v = ((self.0 as i128) << FRAC) / o.0 as i128;
        Fx(v.clamp(i64::MIN as i128 / 4, i64::MAX as i128 / 4) as i64)
    }
}
impl AddAssign for Fx {
    #[inline]
    fn add_assign(&mut self, o: Fx) {
        *self = *self + o;
    }
}
impl SubAssign for Fx {
    #[inline]
    fn sub_assign(&mut self, o: Fx) {
        *self = *self - o;
    }
}
impl MulAssign for Fx {
    #[inline]
    fn mul_assign(&mut self, o: Fx) {
        *self = *self * o;
    }
}

impl fmt::Debug for Fx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4}", self.to_f64())
    }
}
impl fmt::Display for Fx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.3}", self.to_f64())
    }
}

/// Shorthand for configuration constants: `fx(0.25)`.
#[inline]
pub fn fx(f: f64) -> Fx {
    Fx::from_f64(f)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Fx, b: f64, tol: f64) -> bool {
        (a.to_f64() - b).abs() <= tol
    }

    #[test]
    fn exp_ln_accuracy() {
        for &x in &[-10.0, -3.5, -1.0, -0.25, 0.0, 0.3, 1.0, 2.5, 6.0] {
            let e = fx(x).exp();
            assert!(close(e, f64::exp(x), 1e-3 * f64::exp(x).max(1.0)), "exp {x} -> {e:?}");
        }
        for &x in &[0.01, 0.5, 1.0, 2.0, 10.0, 1000.0] {
            let l = fx(x).ln();
            assert!(close(l, f64::ln(x), 2e-3), "ln {x} -> {l:?}");
        }
    }

    #[test]
    fn sigmoid_and_sqrt() {
        assert!(close(fx(0.0).sigmoid(), 0.5, 1e-4));
        assert!(close(fx(2.0).sigmoid(), 0.880797, 1e-3));
        assert!(close(fx(-2.0).sigmoid(), 0.119203, 1e-3));
        assert!(close(fx(2.0).sqrt(), 1.414213, 1e-4));
        assert!(close(fx(0.25).sqrt(), 0.5, 1e-4));
    }

    #[test]
    fn slow_decay_is_representable() {
        // One second of a 2.5-day time constant must not round to exactly 1.
        assert!(Fx::decay(Fx::ONE, fx(216_000.0)) < Fx::ONE);
    }

    #[test]
    fn golden_bits_are_stable() {
        // Bit-exact golden values: if these change, saved worlds and replays break.
        assert_eq!(fx(1.5).exp().raw(), 19_248_707_987);
        assert_eq!(fx(3.0).ln().raw(), 4_718_503_850);
        assert_eq!(fx(0.7).sigmoid().raw(), 2_869_844_629);
        assert_eq!(fx(7.0).sqrt().raw(), 11_363_415_354);
    }
}


