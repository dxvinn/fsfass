//! Sensory encoding: innate receptive fields.
//!
//! Each percept is turned into graded activations of fixed "feature units"
//! (hue bins, brightness levels, flicker, size, shape, surface, felt warmth).
//! These units are the only things a newborn mind can attach meaning to.
//! They describe sensations, never kinds of things.

use alife_core::{fx, Fx};
use alife_interface::{Percept, Taste, Touch};

pub const HUE_BINS: usize = 12;
pub const U_HUE0: usize = 0; // 0..12, centred on 0°, 30°, ... 330°
pub const U_GREY: usize = 12;
pub const U_BRIGHT0: usize = 13; // dark, then thresholds >0.2, >0.45, >0.7 (thermometer code)
pub const U_FLICK0: usize = 17; // still, shimmer, flicker
pub const U_SIZE0: usize = 20; // small, medium, large
pub const U_ROUND: usize = 23;
pub const U_ANGULAR: usize = 24;
pub const U_ROUGH: usize = 25;
pub const U_SMOOTH: usize = 26;
pub const U_GLOSSY: usize = 27;
pub const U_MATTE: usize = 28;
/// Graded felt warmth (linear in intensity).
pub const U_WARM_LIN: usize = 29;
/// Felt warmth thresholds >0.1, >0.3, >0.5, >0.7 (thermometer code: like
/// thermoreceptors with different thresholds, a hotter sensation recruits all
/// the units a milder one does, plus more).
pub const U_WARM0: usize = 30;
pub const N_WARM: usize = 4;
/// Number of sense units available before acting (vision + distal warmth).
pub const N_SENSE: usize = 34;
/// Units used for object categories (vision only; warmth depends on distance).
pub const N_VISUAL: usize = 29;

pub type SenseVec = [Fx; N_SENSE];

/// Consequence sensations felt during contact (not available before acting).
pub const C_CONTACT_HOT: usize = 0;
pub const C_CONTACT_WARM: usize = 1;
pub const C_CONTACT_COOL: usize = 2;
pub const C_FIRM: usize = 3;
pub const C_SOFT: usize = 4;
pub const C_WET: usize = 5;
pub const C_SWEET: usize = 6;
pub const C_BITTER: usize = 7;
pub const C_LIQUID: usize = 8;
pub const C_SWALLOWED: usize = 9;
pub const N_CONSEQ: usize = 10;
pub type ConseqVec = [Fx; N_CONSEQ];

/// Smooth threshold unit: 0 below `t - w/2`, 1 above `t + w/2`.
fn ramp(x: Fx, t: Fx, w: Fx) -> Fx {
    ((x - t) / w + Fx::HALF).clamp01()
}

/// Triangular tuning curve: 1 at `centre`, 0 beyond `width`.
fn tri(x: Fx, centre: Fx, width: Fx) -> Fx {
    (Fx::ONE - (x - centre).abs() / width).clamp01()
}

fn hue_dist(a: Fx, b: Fx) -> Fx {
    let d = (a - b).abs();
    d.min(fx(360.0) - d)
}

/// Encodes what is perceivable about a thing before touching it.
pub fn encode(p: &Percept) -> SenseVec {
    let v = &p.visual;
    let mut u = [Fx::ZERO; N_SENSE];
    let sat = v.saturation.clamp01();
    for i in 0..HUE_BINS {
        let centre = fx(30.0).muli(i as i64);
        u[U_HUE0 + i] = sat * (Fx::ONE - hue_dist(v.hue_deg, centre) / fx(30.0)).clamp01();
    }
    u[U_GREY] = (Fx::ONE - sat * fx(1.5)).clamp01();
    u[U_BRIGHT0] = (Fx::ONE - ramp(v.brightness, fx(0.2), fx(0.2))).clamp01();
    u[U_BRIGHT0 + 1] = ramp(v.brightness, fx(0.2), fx(0.2));
    u[U_BRIGHT0 + 2] = ramp(v.brightness, fx(0.45), fx(0.2));
    u[U_BRIGHT0 + 3] = ramp(v.brightness, fx(0.7), fx(0.2));
    u[U_FLICK0] = tri(v.flicker, Fx::ZERO, fx(0.2));
    u[U_FLICK0 + 1] = tri(v.flicker, fx(0.3), fx(0.25));
    u[U_FLICK0 + 2] = ((v.flicker - fx(0.35)) / fx(0.45)).clamp01();
    u[U_SIZE0] = tri(v.size, fx(0.15), fx(0.15));
    u[U_SIZE0 + 1] = tri(v.size, fx(0.35), fx(0.2));
    u[U_SIZE0 + 2] = ((v.size - fx(0.35)) / fx(0.3)).clamp01();
    u[U_ROUND] = v.roundness;
    u[U_ANGULAR] = Fx::ONE - v.roundness;
    u[U_ROUGH] = v.texture;
    u[U_SMOOTH] = Fx::ONE - v.texture;
    u[U_GLOSSY] = v.gloss;
    u[U_MATTE] = Fx::ONE - v.gloss;
    let w = p.felt_warmth.clamp01();
    if w > fx(0.03) {
        u[U_WARM_LIN] = w;
        for (i, t) in [fx(0.1), fx(0.3), fx(0.5), fx(0.7)].into_iter().enumerate() {
            u[U_WARM0 + i] = ramp(w, t, fx(0.15));
        }
    }
    u
}

/// Encodes contact and taste sensations (consequences of touching / mouthing).
pub fn encode_consequence(touch: Option<&Touch>, taste: Option<&Taste>) -> ConseqVec {
    let mut c = [Fx::ZERO; N_CONSEQ];
    if let Some(t) = touch {
        c[C_CONTACT_HOT] = ((t.thermal - fx(0.3)) / fx(0.5)).clamp01();
        c[C_CONTACT_WARM] = tri(t.thermal, fx(0.15), fx(0.15));
        c[C_CONTACT_COOL] = (-t.thermal / fx(0.4)).clamp01();
        c[C_FIRM] = t.firmness;
        c[C_SOFT] = Fx::ONE - t.firmness;
        c[C_WET] = t.wetness;
    }
    if let Some(t) = taste {
        c[C_SWEET] = t.sweet;
        c[C_BITTER] = t.bitter;
        c[C_LIQUID] = t.liquid;
        c[C_SWALLOWED] = t.swallowed;
    }
    c
}

/// Innate perceptual salience of each sense unit (the alpha_i of
/// Rescorla–Wagner): intense sensations (glare, flicker, felt heat, vivid
/// colour) capture more of the credit for what happens than texture or shape
/// (overshadowing).
pub fn sense_salience(i: usize) -> Fx {
    match i {
        _ if i < U_GREY => fx(0.8),
        U_GREY => fx(0.3),
        _ if (U_BRIGHT0..U_BRIGHT0 + 4).contains(&i) => [fx(0.3), fx(0.4), fx(0.7), fx(1.0)][i - U_BRIGHT0],
        _ if (U_FLICK0..U_FLICK0 + 3).contains(&i) => [fx(0.3), fx(0.6), fx(1.0)][i - U_FLICK0],
        _ if (U_SIZE0..U_SIZE0 + 3).contains(&i) => fx(0.4),
        U_ROUND | U_ANGULAR | U_ROUGH | U_SMOOTH | U_GLOSSY | U_MATTE => fx(0.3),
        U_WARM_LIN => fx(1.0),
        _ if (U_WARM0..U_WARM0 + N_WARM).contains(&i) => [fx(0.7), fx(0.9), fx(1.0), fx(1.0)][i - U_WARM0],
        _ => fx(0.5),
    }
}

pub fn sense_label(i: usize) -> String {
    const HUES: [&str; 12] = [
        "red", "orange", "yellow", "yellow-green", "green", "green", "cyan", "sky-blue", "blue", "violet",
        "magenta", "pink",
    ];
    match i {
        _ if i < U_GREY => format!("hue:{}", HUES[i]),
        U_GREY => "colourless".into(),
        _ if (U_BRIGHT0..U_BRIGHT0 + 4).contains(&i) => {
            ["dark", "lit>0.2", "bright>0.45", "glaring>0.7"][i - U_BRIGHT0].into()
        }
        _ if (U_FLICK0..U_FLICK0 + 3).contains(&i) => ["still", "shimmering", "flickering"][i - U_FLICK0].into(),
        _ if (U_SIZE0..U_SIZE0 + 3).contains(&i) => ["small", "medium", "large"][i - U_SIZE0].into(),
        U_ROUND => "round".into(),
        U_ANGULAR => "jagged".into(),
        U_ROUGH => "rough".into(),
        U_SMOOTH => "smooth".into(),
        U_GLOSSY => "glossy".into(),
        U_MATTE => "matte".into(),
        U_WARM_LIN => "warmth-felt".into(),
        _ if (U_WARM0..U_WARM0 + N_WARM).contains(&i) => {
            ["warmth>0.1", "warmth>0.3", "warmth>0.5", "warmth>0.7"][i - U_WARM0].into()
        }
        _ => format!("unit{i}"),
    }
}

pub fn conseq_label(i: usize) -> &'static str {
    ["contact-hot", "contact-warm", "contact-cool", "firm", "soft", "wet", "sweet", "bitter", "liquid", "swallowed"][i]
}

/// Squared norm (for normalised learning updates).
pub fn norm2(v: &[Fx]) -> Fx {
    v.iter().fold(Fx::ZERO, |a, &x| a + x * x)
}

/// Cosine similarity of two activation vectors.
pub fn cosine(a: &[Fx], b: &[Fx]) -> Fx {
    let dot = a.iter().zip(b).fold(Fx::ZERO, |s, (&x, &y)| s + x * y);
    let na = norm2(a).sqrt();
    let nb = norm2(b).sqrt();
    if na.raw() == 0 || nb.raw() == 0 {
        return Fx::ZERO;
    }
    dot / (na * nb)
}

/// Short human-readable description: the strongest few sense units.
pub fn describe(u: &SenseVec, n: usize) -> Vec<String> {
    let mut idx: Vec<usize> = (0..N_SENSE).filter(|&i| u[i] > fx(0.35) && i != U_WARM_LIN).collect();
    idx.sort_by(|&a, &b| u[b].cmp(&u[a]).then(a.cmp(&b)));
    idx.into_iter().take(n).map(sense_label).collect()
}
