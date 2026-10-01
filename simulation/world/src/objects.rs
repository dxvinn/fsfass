//! Physical property sets for Phase 2 objects (report 05 §3.1, reduced).
//!
//! `label` exists for experiment reports only; it is never sent to a mind.

use alife_core::{fx, Fx};

#[derive(Clone, Debug, PartialEq)]
pub struct ObjProps {
    pub label: &'static str,
    pub temp_c: Fx,
    /// Light emitted, 0..1.
    pub emit: Fx,
    /// Amplitude of emitted-light fluctuation, 0..1.
    pub flicker: Fx,
    pub hue_deg: Fx,
    pub saturation: Fx,
    /// Fraction of ambient light reflected, 0..1.
    pub reflect: Fx,
    pub size: Fx,
    pub roundness: Fx,
    pub roughness: Fx,
    pub gloss: Fx,
    pub hardness: Fx,
    pub wet: Fx,
    /// Thermal contact conductance, 0..1.
    pub conduct: Fx,
    pub kcal_per_portion: Fx,
    pub water_per_portion: Fx,
    pub sugar: Fx,
    pub bitter: Fx,
    pub liquid: bool,
    /// Remaining consumable portions (-1 = not consumable / unlimited for liquids).
    pub portions: i32,
    pub max_portions: i32,
}

impl ObjProps {
    fn base(label: &'static str) -> ObjProps {
        ObjProps {
            label,
            temp_c: fx(20.0),
            emit: Fx::ZERO,
            flicker: Fx::ZERO,
            hue_deg: Fx::ZERO,
            saturation: Fx::ZERO,
            reflect: fx(0.4),
            size: fx(0.3),
            roundness: fx(0.5),
            roughness: fx(0.5),
            gloss: fx(0.1),
            hardness: fx(0.5),
            wet: Fx::ZERO,
            conduct: fx(0.5),
            kcal_per_portion: Fx::ZERO,
            water_per_portion: Fx::ZERO,
            sugar: Fx::ZERO,
            bitter: Fx::ZERO,
            liquid: false,
            portions: -1,
            max_portions: -1,
        }
    }

    /// A campfire: hot, bright, flickering, orange.
    pub fn fire_a() -> ObjProps {
        ObjProps {
            temp_c: fx(700.0),
            emit: fx(0.9),
            flicker: fx(0.8),
            hue_deg: fx(30.0),
            saturation: fx(0.9),
            reflect: fx(0.1),
            size: fx(0.5),
            roundness: fx(0.3),
            roughness: fx(0.7),
            gloss: fx(0.2),
            hardness: fx(0.05),
            conduct: fx(1.0),
            ..Self::base("FIRE_A")
        }
    }

    /// A different fire: smaller, redder, less intense, still flickering.
    pub fn fire_b() -> ObjProps {
        ObjProps {
            temp_c: fx(520.0),
            emit: fx(0.6),
            flicker: fx(0.7),
            hue_deg: fx(14.0),
            saturation: fx(0.85),
            reflect: fx(0.1),
            size: fx(0.3),
            roundness: fx(0.35),
            roughness: fx(0.65),
            gloss: fx(0.2),
            hardness: fx(0.05),
            conduct: fx(1.0),
            ..Self::base("FIRE_B")
        }
    }

    /// Metal heated below visible glow: looks like any grey object, but radiates heat.
    pub fn hot_metal() -> ObjProps {
        ObjProps {
            temp_c: fx(380.0),
            emit: Fx::ZERO,
            flicker: Fx::ZERO,
            hue_deg: fx(220.0),
            saturation: fx(0.05),
            reflect: fx(0.5),
            size: fx(0.25),
            roundness: fx(0.2),
            roughness: fx(0.2),
            gloss: fx(0.6),
            hardness: fx(0.95),
            conduct: fx(1.0),
            ..Self::base("HOT_METAL")
        }
    }

    /// Harmless flower with an orange colour.
    pub fn orange_flower() -> ObjProps {
        ObjProps {
            temp_c: fx(20.0),
            emit: Fx::ZERO,
            flicker: Fx::ZERO,
            hue_deg: fx(32.0),
            saturation: fx(0.85),
            reflect: fx(0.7),
            size: fx(0.2),
            roundness: fx(0.7),
            roughness: fx(0.4),
            gloss: fx(0.2),
            hardness: fx(0.1),
            conduct: fx(0.3),
            bitter: fx(0.4),
            ..Self::base("ORANGE_FLOWER")
        }
    }

    pub fn rock() -> ObjProps {
        ObjProps {
            temp_c: fx(20.0),
            hue_deg: fx(40.0),
            saturation: fx(0.06),
            reflect: fx(0.35),
            size: fx(0.3),
            roundness: fx(0.6),
            roughness: fx(0.8),
            gloss: fx(0.05),
            hardness: fx(0.95),
            conduct: fx(0.6),
            ..Self::base("ROCK")
        }
    }

    /// A blue-tinted rock (harmless; used for misinformation tests later).
    pub fn blue_rock() -> ObjProps {
        ObjProps { hue_deg: fx(215.0), saturation: fx(0.6), ..Self::rock_labelled("BLUE_ROCK") }
    }

    fn rock_labelled(label: &'static str) -> ObjProps {
        ObjProps { label, ..Self::rock() }
    }

    /// A bush of red berries: soft, sweet, nutritious, several portions.
    pub fn berries() -> ObjProps {
        ObjProps {
            temp_c: fx(20.0),
            hue_deg: fx(350.0),
            saturation: fx(0.8),
            reflect: fx(0.5),
            size: fx(0.3),
            roundness: fx(0.9),
            roughness: fx(0.2),
            gloss: fx(0.5),
            hardness: fx(0.15),
            wet: fx(0.2),
            conduct: fx(0.4),
            kcal_per_portion: fx(30.0),
            water_per_portion: fx(10.0),
            sugar: fx(0.8),
            portions: 12,
            max_portions: 12,
            ..Self::base("FOOD")
        }
    }

    /// A shallow pool of water.
    pub fn water_pool() -> ObjProps {
        ObjProps {
            temp_c: fx(15.0),
            flicker: fx(0.08),
            hue_deg: fx(205.0),
            saturation: fx(0.5),
            reflect: fx(0.6),
            size: fx(0.6),
            roundness: fx(0.8),
            roughness: fx(0.0),
            gloss: fx(0.95),
            hardness: Fx::ZERO,
            wet: Fx::ONE,
            conduct: fx(0.8),
            water_per_portion: fx(50.0),
            liquid: true,
            ..Self::base("WATER")
        }
    }
}
