//! Emotions and mood (report 03 R1, reduced to what P1 needs).
//!
//! Emotions are appraisals of predicted and actual outcomes relative to
//! needs: fear (prospective harm), distress (present pain), surprise (large
//! prediction error), relief (feared harm did not come), joy (needs relieved),
//! interest (curiosity). They decay with their own time constants, push a PAD
//! mood, and modulate cognition (arousal narrows attention and lowers
//! selection noise; fear raises the weight of predicted harm).

use alife_core::hash::{StableHash, StateHasher};
use alife_core::{fx, Fx};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Affect {
    pub fear: Fx,
    pub distress: Fx,
    pub surprise: Fx,
    pub relief: Fx,
    pub joy: Fx,
    pub interest: Fx,
    /// Mood: pleasure, arousal, dominance in -1..1.
    pub pleasure: Fx,
    pub arousal: Fx,
    pub dominance: Fx,
}

impl Affect {
    /// One second of decay and mood dynamics.
    pub fn tick(&mut self) {
        let d = |v: &mut Fx, tau: f64| *v = *v * Fx::decay(Fx::ONE, fx(tau));
        d(&mut self.fear, 20.0);
        d(&mut self.distress, 30.0);
        d(&mut self.surprise, 4.0);
        d(&mut self.relief, 15.0);
        d(&mut self.joy, 30.0);
        d(&mut self.interest, 15.0);
        let p_target = (self.joy + self.relief + self.interest * fx(0.3) - self.distress - self.fear).clamp(Fx::NEG_ONE, Fx::ONE);
        let a_target = (self.fear + self.surprise + self.distress + self.interest * fx(0.5)).clamp01();
        let d_target = (fx(0.3) - self.fear - self.distress * fx(0.5)).clamp(Fx::NEG_ONE, Fx::ONE);
        self.pleasure += (p_target - self.pleasure) * fx(0.03);
        self.arousal += (a_target - self.arousal) * fx(0.15);
        self.dominance += (d_target - self.dominance) * fx(0.03);
    }

    pub fn raise(v: &mut Fx, to: Fx) {
        *v = (*v).max(to.clamp01());
    }

    pub fn list(&self) -> Vec<(&'static str, Fx)> {
        let mut v = vec![
            ("fear", self.fear),
            ("distress", self.distress),
            ("surprise", self.surprise),
            ("relief", self.relief),
            ("joy", self.joy),
            ("interest", self.interest),
        ];
        v.retain(|(_, x)| *x > fx(0.03));
        v.sort_by(|a, b| b.1.cmp(&a.1));
        v
    }
}

impl StableHash for Affect {
    fn stable_hash(&self, h: &mut StateHasher) {
        for v in [
            self.fear, self.distress, self.surprise, self.relief, self.joy, self.interest, self.pleasure, self.arousal,
            self.dominance,
        ] {
            v.stable_hash(h);
        }
    }
}
