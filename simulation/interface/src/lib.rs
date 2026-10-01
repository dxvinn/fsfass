//! The interface between world truth and minds.
//!
//! A mind receives a `SensoryFrame` and returns a `MotorCommand`. Nothing else
//! crosses this boundary. In particular a frame never contains object kinds,
//! world ids, true temperatures, nutrition values or "edible" flags: only the
//! values that sensory organs would report, already scaled to 0..1 sensation
//! strengths, plus small sensory noise.
//!
//! Tracking tokens let a mind direct an action at "that thing over there".
//! They are salted per environment, so they carry no identity across a reset,
//! and minds must not store them in long-term memory.

use alife_core::hash::{StableHash, StateHasher};
use alife_core::Fx;

/// Opaque handle for a currently perceived thing. Valid only within the
/// current environment; meaningless after a reset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Token(pub u64);

/// What the eyes report about a thing (all values are sensation strengths).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Visual {
    /// Hue angle in degrees [0, 360). Meaningful only when `saturation` is high.
    pub hue_deg: Fx,
    /// Colourfulness, 0 = grey.
    pub saturation: Fx,
    /// Perceived brightness (emitted + reflected light), 0..1.
    pub brightness: Fx,
    /// Temporal luminance variation detected by the eye, 0..1.
    pub flicker: Fx,
    /// Apparent size, 0..1.
    pub size: Fx,
    /// Outline roundness, 0 = jagged, 1 = round.
    pub roundness: Fx,
    /// Surface roughness as seen, 0..1.
    pub texture: Fx,
    /// Specular shine / wet look, 0..1.
    pub gloss: Fx,
    /// Self-propelled movement seen in this thing (walking, breathing, turning), 0..1.
    /// Motion detection is innate and works from birth.
    pub motion: Fx,
    /// Face-like pattern (two eyes above a mouth), 0..1. Newborns orient to faces innately.
    pub face: Fx,
}

/// What the skin reports when it touches something.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Touch {
    /// -1 cold .. 0 neutral .. +1 extremely hot (relative to skin).
    pub thermal: Fx,
    /// Felt firmness, 0..1.
    pub firmness: Fx,
    /// Felt wetness, 0..1.
    pub wetness: Fx,
}

/// What the mouth reports when something is put in it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Taste {
    pub sweet: Fx,
    pub bitter: Fx,
    pub liquid: Fx,
    /// Whether anything could be swallowed (soft enough / liquid).
    pub swallowed: Fx,
}

/// One perceived thing in the scene.
#[derive(Clone, Debug, PartialEq)]
pub struct Percept {
    pub token: Token,
    /// Offset in cells from the perceiver.
    pub dx: i32,
    pub dy: i32,
    /// Chebyshev distance in cells.
    pub dist: i32,
    pub visual: Visual,
    /// Radiant warmth felt on the skin from this direction, 0..1.
    pub felt_warmth: Fx,
    /// Present only if the perceiver touched this thing during the last tick.
    pub touch: Option<Touch>,
    /// Present only if the perceiver mouthed this thing during the last tick.
    pub taste: Option<Taste>,
    /// This thing touched the perceiver's skin during the last tick (being touched).
    pub touched_me: bool,
}

/// Another creature seen putting something in its mouth or touching it.
/// Only what is visible: which thing, and whether it chewed and swallowed,
/// drank (lapping, gulping), or jerked back as if hurt.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Observation {
    pub actor: Token,
    pub target: Token,
    /// True for a mouth act, false for a hand act.
    pub mouth: bool,
    pub chewed: bool,
    pub drank: bool,
    pub recoiled: bool,
}

/// Body signals (interoception), each 0..1.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Interoception {
    pub hunger: Fx,
    pub thirst: Fx,
    /// Overall pain (phasic + tonic).
    pub pain: Fx,
    /// Acute nociceptor firing right now (a new hurt), 0..1.
    pub acute_pain: Fx,
    /// Pain localised to the hand / mouth region.
    pub pain_hand: Fx,
    pub pain_mouth: Fx,
    /// Stomach fullness sensation.
    pub fullness: Fx,
    /// Nutrient / water sensed in the gut (fast intake signals), 0..1.
    pub gut_nutrient: Fx,
    pub gut_fluid: Fx,
    pub fatigue: Fx,
    /// Overall thermal comfort: -1 too cold .. +1 too hot.
    pub body_heat: Fx,
    /// Social need (time without company), 0..1.
    pub loneliness: Fx,
    /// Comforting touch / closeness felt right now, 0..1.
    pub social_comfort: Fx,
    /// True while a protective reflex is pulling a limb away.
    pub reflex_active: bool,
}

/// Ambient sensations used by minds to recognise contexts (not places).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Ambient {
    pub light: Fx,
    /// Feel and look of the ground, 0..1.
    pub ground: Fx,
    /// Sense of enclosure (walls/roof nearby), 0..1.
    pub enclosure: Fx,
    /// Background sound level, 0..1.
    pub sound: Fx,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MotorResult {
    Ok,
    /// The action could not be performed (target gone, blocked, too far).
    Failed,
    /// A reflex overrode the commanded action.
    Overridden,
}

/// Everything a mind is told about the current moment.
#[derive(Clone, Debug, PartialEq)]
pub struct SensoryFrame {
    pub tick: u64,
    pub asleep: bool,
    pub percepts: Vec<Percept>,
    /// Others seen acting on things this moment, with what could be seen of the result.
    pub observations: Vec<Observation>,
    pub body: Interoception,
    pub ambient: Ambient,
    pub last_result: MotorResult,
}

/// Motor programs a body can execute. They are generic: none refers to a kind of object.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MotorCommand {
    /// Do nothing in particular.
    Rest,
    /// Take a step in one of 8 directions (0..8).
    Wander { dir: u8 },
    /// Move toward the target until adjacent.
    Approach { target: Token },
    /// Move to about 2 cells from the target and look at it.
    Inspect { target: Token },
    /// Reach out and touch the target with a hand (moves closer first if needed).
    Touch { target: Token },
    /// Put the target (or part of it) in the mouth; swallow if possible.
    Mouth { target: Token },
    /// Move away from the target.
    Withdraw { target: Token },
}

impl MotorCommand {
    pub fn target(&self) -> Option<Token> {
        match *self {
            MotorCommand::Approach { target }
            | MotorCommand::Inspect { target }
            | MotorCommand::Touch { target }
            | MotorCommand::Mouth { target }
            | MotorCommand::Withdraw { target } => Some(target),
            _ => None,
        }
    }

    pub fn code(&self) -> u64 {
        match *self {
            MotorCommand::Rest => 0,
            MotorCommand::Wander { dir } => 1 + dir as u64 * 16,
            MotorCommand::Approach { target } => 2 + (target.0 << 4),
            MotorCommand::Inspect { target } => 3 + (target.0 << 4),
            MotorCommand::Touch { target } => 4 + (target.0 << 4),
            MotorCommand::Mouth { target } => 5 + (target.0 << 4),
            MotorCommand::Withdraw { target } => 6 + (target.0 << 4),
        }
    }
}

impl StableHash for Token {
    fn stable_hash(&self, h: &mut StateHasher) {
        h.u64(self.0);
    }
}

impl StableHash for MotorCommand {
    fn stable_hash(&self, h: &mut StateHasher) {
        h.u64(self.code());
    }
}
