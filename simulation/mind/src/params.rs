//! Personality and age-dependent parameters (reports 01 §8.5, 03 R2).
//!
//! Development is one brain whose parameters follow age curves, never a set
//! of unlocked knowledge tables. Personality scales the same parameters.

use alife_core::hash::{StableHash, StateHasher};
use alife_core::rng::stream;
use alife_core::{fx, Fx, Rng};

/// Big-Five-like factors, each 0..1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Personality {
    pub openness: Fx,
    pub conscientiousness: Fx,
    pub extraversion: Fx,
    pub agreeableness: Fx,
    pub neuroticism: Fx,
}

impl Personality {
    pub fn average() -> Self {
        Personality {
            openness: Fx::HALF,
            conscientiousness: Fx::HALF,
            extraversion: Fx::HALF,
            agreeableness: Fx::HALF,
            neuroticism: Fx::HALF,
        }
    }

    /// Seeded personality: each factor ~ 0.5 ± 0.17, clamped to 0.05..0.95.
    pub fn seeded(seed: u64, who: u64) -> Self {
        let mut r = Rng::new(seed, who, 0, stream::PERSONALITY);
        let mut f = || (Fx::HALF + r.gaussish() * fx(0.17)).clamp(fx(0.05), fx(0.95));
        Personality {
            openness: f(),
            conscientiousness: f(),
            extraversion: f(),
            agreeableness: f(),
            neuroticism: f(),
        }
    }
}

/// Learning and decision parameters derived from age and personality.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MindParams {
    pub age_years: Fx,
    /// Base learning rate.
    pub alpha: Fx,
    /// Gain on aversive (negative) prediction errors.
    pub kappa_neg: Fx,
    /// Gain on appetitive prediction errors.
    pub kappa_pos: Fx,
    /// Extinction learning rate.
    pub alpha_ext: Fx,
    pub curiosity: Fx,
    /// Workspace capacity before arousal effects.
    pub workspace: usize,
    pub planning_depth: u8,
    /// Weight of the Pavlovian (gut) bias.
    pub k_pav: Fx,
    /// Selection noise temperature.
    pub temperature: Fx,
    pub persistence: Fx,
    /// Multiplier on the disutility of pain.
    pub pain_aversion: Fx,
    pub fear_gain: Fx,
    /// Habit learning rate.
    pub eta_habit: Fx,
    /// Days a slow association lasts before fading (before rehearsal bonus).
    pub forget_days: Fx,
}

impl MindParams {
    pub fn new(age_years: Fx, p: &Personality) -> Self {
        // Piecewise age curves (01 §8.5). Phase 2 only tests children, but the
        // curves are defined across the lifespan.
        let a = age_years;
        let (alpha, kneg, kpos, ext, cur, k, depth, kpav) = if a < fx(3.0) {
            (fx(0.6), fx(1.2), fx(1.0), fx(0.20), fx(1.0), 2usize, 0u8, fx(1.0))
        } else if a < fx(12.0) {
            (fx(0.5), fx(1.6), fx(1.0), fx(0.30), fx(0.8), 4, 2, fx(0.8))
        } else if a < fx(20.0) {
            (fx(0.4), fx(1.2), fx(1.4), fx(0.12), fx(0.6), 6, 3, fx(0.9))
        } else if a < fx(56.0) {
            (fx(0.25), fx(1.5), fx(1.0), fx(0.30), fx(0.35), 7, 3, fx(0.5))
        } else {
            (fx(0.18), fx(1.1), fx(1.1), fx(0.20), fx(0.2), 5, 2, fx(0.5))
        };
        MindParams {
            age_years: a,
            alpha,
            kappa_neg: kneg * (fx(0.7) + p.neuroticism * fx(0.6)),
            kappa_pos: kpos,
            alpha_ext: ext,
            curiosity: cur * (fx(0.6) + p.openness * fx(0.8)),
            workspace: k,
            planning_depth: depth,
            k_pav: kpav * (fx(0.8) + p.neuroticism * fx(0.4)),
            temperature: fx(0.06),
            persistence: fx(0.10) * (fx(0.5) + p.conscientiousness),
            pain_aversion: fx(1.3) - p.extraversion * fx(0.5),
            fear_gain: fx(0.7) + p.neuroticism * fx(0.6),
            eta_habit: fx(0.15),
            forget_days: fx(30.0),
        }
    }
}

impl StableHash for Personality {
    fn stable_hash(&self, h: &mut StateHasher) {
        self.openness.stable_hash(h);
        self.conscientiousness.stable_hash(h);
        self.extraversion.stable_hash(h);
        self.agreeableness.stable_hash(h);
        self.neuroticism.stable_hash(h);
    }
}
