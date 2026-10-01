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

use crate::learning::assoc::{Out, N_OUT, O_HYDRATE, O_NOURISH, O_PAIN, O_SOCIAL, O_WARM};
use crate::params::MindParams;
use alife_core::{fx, Fx};
use alife_interface::Interoception;

fn cold(b: &Interoception) -> Fx {
    (-b.body_heat).clamp01()
}

pub fn drive(b: &Interoception) -> Fx {
    let c = cold(b);
    b.hunger * b.hunger + b.thirst * b.thirst + b.pain * b.pain * fx(1.5) + b.loneliness * b.loneliness * fx(0.5) + c * c
}

/// Homeostatic reward for the last step.
pub fn reward(prev: &Interoception, now: &Interoception, pain_onset: Fx, p: &MindParams) -> Fx {
    (drive(prev) - drive(now)) * fx(4.0) - pain_onset * p.pain_aversion
}

/// Need-dependent utility of each outcome (per unit of predicted outcome).
pub fn utilities(b: &Interoception, fear: Fx, p: &MindParams) -> Out {
    let mut u = [Fx::ZERO; N_OUT];
    u[O_SOCIAL] = b.loneliness * fx(1.5) + fx(0.02);
    u[O_WARM] = cold(b) * fx(2.0) - b.body_heat.max(Fx::ZERO);
    u[O_PAIN] = -(p.pain_aversion * (Fx::ONE + fear * p.fear_gain));
    u[O_NOURISH] = b.hunger * fx(2.0) + fx(0.05) - b.gut_nutrient * fx(0.5);
    u[O_HYDRATE] = b.thirst * fx(2.0) + fx(0.05) - b.gut_fluid * fx(0.5);
    u
}

/// Innate valence used by the Pavlovian (gut) system.
pub fn innate_valence(b: &Interoception) -> Out {
    [Fx::NEG_ONE, b.hunger, b.thirst, b.loneliness, cold(b)]
}

/// Outcomes experienced this step, from changes in body signals:
/// pain onset, and intake sensed in the gut.
pub fn outcomes(prev: &Interoception, now: &Interoception) -> Out {
    let mut o = [Fx::ZERO; N_OUT];
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
    o[O_SOCIAL] = ((now.social_comfort - prev.social_comfort) * fx(3.0)).clamp01();
    if prev.body_heat < Fx::ZERO {
        o[O_WARM] = ((now.body_heat - prev.body_heat) * fx(4.0)).clamp01();
    }
    for v in o.iter_mut().skip(3) {
        if *v < fx(0.03) {
            *v = Fx::ZERO;
        }
    }
    o
}
