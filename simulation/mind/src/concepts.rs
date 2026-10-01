//! Learned categories (adaptive-resonance style prototype matching).
//!
//! A newborn mind has no categories. When it sees something, it compares the
//! visual pattern with its stored prototypes. If the best match is similar
//! enough (vigilance), the thing is recognised as that concept and the
//! prototype moves slightly toward it; otherwise a new concept is created.
//! Concept ids are private to this mind ("C3" means nothing to anyone else).

use crate::encode::{cosine, SenseVec, N_VISUAL};
use alife_core::hash::{StableHash, StateHasher};
use alife_core::{fx, Fx};

pub const MAX_CONCEPTS: usize = 32;

#[derive(Clone, Debug, PartialEq)]
pub struct Concept {
    pub id: u16,
    pub proto: [Fx; N_VISUAL],
    /// Number of times recognised.
    pub seen: u32,
    pub last_seen: u64,
    pub created: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConceptStore {
    pub concepts: Vec<Concept>,
    pub vigilance: Fx,
    pub next_id: u16,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Recognition {
    /// Slot index into `concepts`.
    pub slot: usize,
    pub id: u16,
    pub similarity: Fx,
    pub novel: bool,
    /// Id of a concept that was forgotten to make room for this one.
    pub replaced: Option<u16>,
}

impl ConceptStore {
    pub fn new(vigilance: Fx) -> Self {
        ConceptStore { concepts: Vec::new(), vigilance, next_id: 1 }
    }

    fn visual(u: &SenseVec) -> [Fx; N_VISUAL] {
        let mut v = [Fx::ZERO; N_VISUAL];
        v.copy_from_slice(&u[..N_VISUAL]);
        v
    }

    /// Best match without learning.
    pub fn peek(&self, u: &SenseVec) -> Option<(usize, Fx)> {
        let v = Self::visual(u);
        let mut best: Option<(usize, Fx)> = None;
        for (i, c) in self.concepts.iter().enumerate() {
            let s = cosine(&v, &c.proto);
            if best.map_or(true, |(_, b)| s > b) {
                best = Some((i, s));
            }
        }
        best
    }

    /// Recognise (and learn from) a visual pattern.
    pub fn recognise(&mut self, u: &SenseVec, tick: u64) -> Recognition {
        let v = Self::visual(u);
        if let Some((i, s)) = self.peek(u) {
            if s >= self.vigilance {
                let c = &mut self.concepts[i];
                let rate = fx(0.05);
                for k in 0..N_VISUAL {
                    c.proto[k] += (v[k] - c.proto[k]) * rate;
                }
                c.seen += 1;
                c.last_seen = tick;
                return Recognition { slot: i, id: c.id, similarity: s, novel: false, replaced: None };
            }
        }
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        let c = Concept { id, proto: v, seen: 1, last_seen: tick, created: tick };
        let mut replaced = None;
        let slot = if self.concepts.len() < MAX_CONCEPTS {
            self.concepts.push(c);
            self.concepts.len() - 1
        } else {
            // Replace the least used, least recent concept.
            let (slot, _) = self
                .concepts
                .iter()
                .enumerate()
                .min_by_key(|(_, c)| (c.seen as u64).saturating_mul(1000) + c.last_seen / 3600)
                .expect("non-empty");
            replaced = Some(self.concepts[slot].id);
            self.concepts[slot] = c;
            slot
        };
        Recognition { slot, id, similarity: Fx::ONE, novel: true, replaced }
    }
}

impl StableHash for ConceptStore {
    fn stable_hash(&self, h: &mut StateHasher) {
        h.u64(self.concepts.len() as u64);
        for c in &self.concepts {
            c.id.stable_hash(h);
            c.proto.stable_hash(h);
            c.seen.stable_hash(h);
            c.last_seen.stable_hash(h);
        }
        self.next_id.stable_hash(h);
    }
}
