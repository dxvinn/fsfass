//! Needs and homeostatic reward (report 03 R3; Keramati–Gutkin, decision D3).
//!
//! Drive D = hunger² + thirst² + 1.5·pain² (convex: the worst need dominates).
//! Reward is drive reduction, minus an innate aversion to pain onset.
//!
//! Outcomes that are *learned* are what the body senses happening (pain onset,
//! nutrient intake, fluid intake), not changes in need. Their *value* depends
//! on the current need (Keramati–Gutkin): the same intake is worth a lot when
//! deprived, nothing when sated, and is unpleasant when overfull. So a child
//! who first mouths something liquid while not thirsty still learns that it
//! yields fluid, and uses that knowledge later when thirsty.

use crate::learning::assoc::{Out, O_HYDRATE, O_NOURISH, O_PAIN};
use crate::params::MindParams;
use alife_core::{fx, Fx};
use alife_interface::Interoception;

pub fn drive(b: &Interoception) -> Fx {
    b.hunger * b.hunger + b.thirst * b.thirst + b.pain * b.pain * fx(1.5)
}

/// Homeostatic reward for the last step.
pub fn reward(prev: &Interoception, now: &Interoception, pain_onset: Fx, p: &MindParams) -> Fx {
    (drive(prev) - drive(now)) * fx(4.0) - pain_onset * p.pain_aversion
}

/// Need-dependent utility of each outcome (per unit of predicted outcome).
pub fn utilities(b: &Interoception, fear: Fx, p: &MindParams) -> Out {
    let mut u = [Fx::ZERO; 3];
    u[O_PAIN] = -(p.pain_aversion * (Fx::ONE + fear * p.fear_gain));
    u[O_NOURISH] = b.hunger * fx(2.0) + fx(0.05) - b.gut_nutrient * fx(0.5);
    u[O_HYDRATE] = b.thirst * fx(2.0) + fx(0.05) - b.gut_fluid * fx(0.5);
    u
}

/// Innate valence used by the Pavlovian (gut) system.
pub fn innate_valence(b: &Interoception) -> Out {
    [Fx::NEG_ONE, b.hunger, b.thirst]
}

/// Outcomes experienced this step, from changes in body signals:
/// pain onset, and intake sensed in the gut.
pub fn outcomes(prev: &Interoception, now: &Interoception) -> Out {
    let mut o = [Fx::ZERO; 3];
    // A new hurt = onset of acute nociceptor firing (not the slow inflammatory ache).
    let onset = now.acute_pain - prev.acute_pain;
    if onset > fx(0.02) {
        o[O_PAIN] = onset.clamp01();
    }
    o[O_NOURISH] = ((now.gut_nutrient - prev.gut_nutrient) * fx(3.0)).clamp01();
    o[O_HYDRATE] = ((now.gut_fluid - prev.gut_fluid) * fx(5.0)).clamp01();
    if o[O_NOURISH] < fx(0.03) {
        o[O_NOURISH] = Fx::ZERO;
    }
    if o[O_HYDRATE] < fx(0.03) {
        o[O_HYDRATE] = Fx::ZERO;
    }
    o
}
