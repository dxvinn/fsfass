//! Episodic memory (report 02 C.(b)): a budgeted store of surprising or
//! important moments, with ACT-R-style retrieval (base-level activation from
//! use and age + cue match + emotional boost) and DF-style pinned slots for
//! emotionally defining episodes.

use crate::encode::{SenseVec, N_SENSE};
use crate::learning::assoc::{Out, N_OUT};
use alife_core::hash::{StableHash, StateHasher};
use alife_core::{fx, Fx};

pub const RECENT_CAP: usize = 96;
pub const PINNED_CAP: usize = 16;
/// Action code for an episode with no action (something just happened).
pub const EP_NO_ACTION: u8 = 7;

#[derive(Clone, Debug, PartialEq)]
pub struct Episode {
    pub id: u32,
    pub tick: u64,
    pub ctx: u8,
    pub concept: u16,
    /// Strongest sense units at the time (unit index, activation).
    pub features: Vec<(u8, Fx)>,
    /// Strongest consequence sensations (contact/taste unit, activation).
    pub conseq: Vec<(u8, Fx)>,
    pub action: u8,
    pub outcome: Out,
    pub surprise: Fx,
    pub valence: Fx,
    pub arousal: Fx,
    pub importance: Fx,
    pub recalls: u16,
    pub last_recall: u64,
    pub pinned: bool,
}

impl Episode {
    pub fn sense_vec(&self) -> SenseVec {
        let mut v = [Fx::ZERO; N_SENSE];
        for &(u, a) in &self.features {
            v[u as usize] = a;
        }
        v
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Recall {
    pub episode: u32,
    pub activation: Fx,
    pub similarity: Fx,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EpisodicMemory {
    pub recent: Vec<Episode>,
    pub pinned: Vec<Episode>,
    pub next_id: u32,
    pub evicted: u32,
}

impl Default for EpisodicMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl EpisodicMemory {
    pub fn new() -> Self {
        EpisodicMemory { recent: Vec::new(), pinned: Vec::new(), next_id: 1, evicted: 0 }
    }

    pub fn all(&self) -> impl Iterator<Item = &Episode> {
        self.pinned.iter().chain(self.recent.iter())
    }

    pub fn get(&self, id: u32) -> Option<&Episode> {
        self.all().find(|e| e.id == id)
    }

    fn get_mut(&mut self, id: u32) -> Option<&mut Episode> {
        self.pinned.iter_mut().chain(self.recent.iter_mut()).find(|e| e.id == id)
    }

    /// ACT-R base-level activation from number of uses and time since last use.
    pub fn base_level(e: &Episode, now: u64) -> Fx {
        let d = Fx::HALF;
        let hours = Fx::from_int((now.saturating_sub(e.last_recall.max(e.tick))) as i64) / fx(3600.0) + fx(0.0167);
        (Fx::from_int(e.recalls as i64 + 1) / (Fx::ONE - d)).ln_fast() - d * hours.ln_fast()
    }

    pub fn store(&mut self, mut e: Episode) -> u32 {
        e.id = self.next_id;
        self.next_id += 1;
        let id = e.id;
        if e.arousal >= fx(0.85) {
            e.pinned = true;
            if self.pinned.len() < PINNED_CAP {
                self.pinned.push(e);
                return id;
            }
            // Replace a weaker pinned memory; otherwise fall through to recent.
            let (wi, w) = self
                .pinned
                .iter()
                .enumerate()
                .min_by_key(|(_, p)| (p.arousal + p.importance).raw())
                .map(|(i, p)| (i, p.arousal + p.importance))
                .expect("non-empty");
            if e.arousal + e.importance > w {
                let now = e.tick;
                let mut old = std::mem::replace(&mut self.pinned[wi], e);
                old.pinned = false;
                self.push_recent(old, now);
                return id;
            }
            e.pinned = false;
            let now = e.tick;
            self.push_recent(e, now);
            return id;
        }
        let now = e.tick;
        self.push_recent(e, now);
        id
    }

    fn push_recent(&mut self, e: Episode, now: u64) {
        if self.recent.len() >= RECENT_CAP {
            let now = now.max(e.tick);
            let (wi, _) = self
                .recent
                .iter()
                .enumerate()
                .min_by_key(|(_, r)| (Self::base_level(r, now) + r.importance * fx(2.0)).raw())
                .expect("non-empty");
            self.recent.remove(wi);
            self.evicted += 1;
        }
        self.recent.push(e);
    }

    /// Retrieve up to `k` episodes matching a cue (and optionally an action).
    pub fn retrieve(&self, cue: &SenseVec, concept: u16, action: Option<u8>, now: u64, k: usize) -> Vec<Recall> {
        let mut out: Vec<Recall> = Vec::new();
        let cn = crate::encode::norm2(cue).sqrt();
        if cn.raw() == 0 {
            return out;
        }
        for e in self.all() {
            // Sparse cosine: episodes store only their strongest features.
            let mut dot = Fx::ZERO;
            let mut en2 = Fx::ZERO;
            for &(u, a) in &e.features {
                dot += cue[u as usize] * a;
                en2 += a * a;
            }
            if dot.raw() <= 0 {
                continue;
            }
            let sim = dot / (cn * en2.sqrt());
            if sim < fx(0.3) {
                continue;
            }
            let mut s = sim * fx(2.5);
            if concept != 0 && e.concept == concept {
                s += fx(0.5);
            }
            if let Some(a) = action {
                if e.action == a {
                    s += fx(0.4);
                }
            }
            let act = Self::base_level(e, now) + s + e.arousal;
            if act > fx(0.5) {
                out.push(Recall { episode: e.id, activation: act, similarity: sim });
            }
        }
        out.sort_by(|a, b| b.activation.cmp(&a.activation).then(a.episode.cmp(&b.episode)));
        out.truncate(k);
        out
    }

    /// Episodic-control estimate: what happened last times I did `action` to similar things.
    pub fn estimate(&self, recalls: &[Recall], action: u8) -> Option<(Out, Fx)> {
        let mut num = [Fx::ZERO; N_OUT];
        let mut den = Fx::ZERO;
        for r in recalls {
            if let Some(e) = self.get(r.episode) {
                if e.action != action {
                    continue;
                }
                let w = r.similarity * r.similarity;
                for o in 0..N_OUT {
                    num[o] += e.outcome[o] * w;
                }
                den += w;
            }
        }
        if den < fx(0.2) {
            return None;
        }
        let mut est = [Fx::ZERO; N_OUT];
        for o in 0..N_OUT {
            est[o] = num[o] / den;
        }
        Some((est, den.min(Fx::ONE)))
    }

    /// Strengthen an episode that was used (rehearsal).
    pub fn rehearse(&mut self, id: u32, now: u64) {
        if let Some(e) = self.get_mut(id) {
            e.recalls = e.recalls.saturating_add(1);
            e.last_recall = now;
        }
    }

    /// Episodes to replay during sleep: most surprising and emotional first.
    pub fn replay_order(&self, since: u64, n: usize) -> Vec<u32> {
        let mut v: Vec<(&Episode, Fx)> = self
            .all()
            .filter(|e| e.tick >= since)
            .map(|e| (e, e.surprise * (e.valence.abs() + fx(0.2)) + e.arousal * fx(0.5)))
            .collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.id.cmp(&b.0.id)));
        v.into_iter().take(n).map(|(e, _)| e.id).collect()
    }
}

impl StableHash for Episode {
    fn stable_hash(&self, h: &mut StateHasher) {
        self.id.stable_hash(h);
        self.tick.stable_hash(h);
        self.ctx.stable_hash(h);
        self.concept.stable_hash(h);
        self.features.stable_hash(h);
        self.conseq.stable_hash(h);
        self.action.stable_hash(h);
        self.outcome.stable_hash(h);
        self.surprise.stable_hash(h);
        self.valence.stable_hash(h);
        self.arousal.stable_hash(h);
        self.recalls.stable_hash(h);
        self.last_recall.stable_hash(h);
        self.pinned.stable_hash(h);
    }
}

impl StableHash for EpisodicMemory {
    fn stable_hash(&self, h: &mut StateHasher) {
        self.recent.stable_hash(h);
        self.pinned.stable_hash(h);
        self.next_id.stable_hash(h);
    }
}
