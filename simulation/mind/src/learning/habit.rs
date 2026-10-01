//! Habit system: model-free cached action values (report 01 rule 6).
//!
//! Q(concept, action) is learned from the reward actually experienced
//! (homeostatic drive reduction minus pain). Habits are specific: they attach
//! to a learned concept, not to raw features, so they generalise poorly. Their
//! reliability is 1 - running mean of |reward prediction error|.

use alife_core::hash::{StableHash, StateHasher};
use alife_core::{fx, Fx};

/// Action codes used by the habit table (targeted 0..5, then wander, rest).
pub const H_WANDER: u8 = 5;
pub const H_REST: u8 = 6;
pub const MAX_HABITS: usize = 256;

#[derive(Clone, Debug, PartialEq)]
pub struct HabitEntry {
    /// 0 = no target (untargeted actions).
    pub concept: u16,
    pub action: u8,
    pub q: Fx,
    pub n: u16,
    pub last: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Habits {
    pub entries: Vec<HabitEntry>,
    /// Running mean of |reward prediction error|.
    pub abs_rpe: Fx,
}

impl Default for Habits {
    fn default() -> Self {
        Self::new()
    }
}

impl Habits {
    pub fn new() -> Self {
        Habits { entries: Vec::new(), abs_rpe: fx(0.5) }
    }

    pub fn q(&self, concept: u16, action: u8) -> (Fx, u16) {
        self.entries
            .iter()
            .find(|e| e.concept == concept && e.action == action)
            .map(|e| (e.q, e.n))
            .unwrap_or((Fx::ZERO, 0))
    }

    pub fn reliability(&self) -> Fx {
        (Fx::ONE - self.abs_rpe).clamp01()
    }

    /// TD(0) update with no bootstrapping (one-step contextual bandit).
    pub fn update(&mut self, concept: u16, action: u8, reward: Fx, eta: Fx, tick: u64) -> Fx {
        let idx = match self.entries.iter().position(|e| e.concept == concept && e.action == action) {
            Some(i) => i,
            None => {
                if self.entries.len() >= MAX_HABITS {
                    // Drop the least-practised, oldest habit.
                    let (i, _) = self
                        .entries
                        .iter()
                        .enumerate()
                        .min_by_key(|(_, e)| (e.n as u64) << 32 | (e.last / 60))
                        .expect("non-empty");
                    self.entries.remove(i);
                }
                self.entries.push(HabitEntry { concept, action, q: Fx::ZERO, n: 0, last: tick });
                self.entries.len() - 1
            }
        };
        let e = &mut self.entries[idx];
        let rpe = reward - e.q;
        // Learning rate falls with practice: habits stabilise.
        let rate = eta * fx(3.0) / (fx(3.0) + Fx::from_int(e.n as i64).sqrt());
        e.q += rpe * rate;
        e.n = e.n.saturating_add(1);
        e.last = tick;
        self.abs_rpe = self.abs_rpe * fx(0.95) + rpe.abs().min(Fx::ONE) * fx(0.05);
        rpe
    }

    /// Forget a concept (its slot was recycled).
    pub fn clear_concept(&mut self, concept: u16) {
        self.entries.retain(|e| e.concept != concept);
    }
}

impl StableHash for Habits {
    fn stable_hash(&self, h: &mut StateHasher) {
        h.u64(self.entries.len() as u64);
        for e in &self.entries {
            e.concept.stable_hash(h);
            e.action.stable_hash(h);
            e.q.stable_hash(h);
            e.n.stable_hash(h);
            e.last.stable_hash(h);
        }
        self.abs_rpe.stable_hash(h);
    }
}
