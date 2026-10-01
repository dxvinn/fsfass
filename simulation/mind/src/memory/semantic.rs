//! Semantic memory: explicit beliefs (report 01 rule 9, report 02 C.(a)).
//!
//! A belief is the agent's own statement about the world, crystallised from
//! a strong and stable association (or, later, acquired from others). It is
//! expressed in the agent's private vocabulary: its sense units and its own
//! concept ids. It carries confidence, source and evidence pointers, and it
//! can be wrong. World truth never enters this structure.

use crate::learning::assoc::{Assoc, CueVec, N_ACT, N_CTX, N_CUES, N_OUT};
use alife_core::hash::{StableHash, StateHasher};
use alife_core::{fx, Fx};

pub const MAX_BELIEFS: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// Learned from the agent's own experience.
    OwnExperience,
    /// Learned by watching someone else (P3).
    Observed { who: u32 },
    /// Told by someone (P3).
    Told { who: u32 },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Belief {
    pub id: u32,
    /// The cue this belief is about (a sense unit or one of the agent's concepts).
    pub cue: u16,
    /// The action involved, or None for "this predicts that" (Pavlovian).
    pub action: Option<u8>,
    pub outcome: u8,
    /// Association strength at crystallisation / last update.
    pub strength: Fx,
    /// 0..1, from strength and amount of evidence.
    pub confidence: Fx,
    pub evidence: u16,
    pub source: Source,
    pub evidence_episodes: Vec<u32>,
    pub t_learned: u64,
    pub t_updated: u64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SemanticMemory {
    pub beliefs: Vec<Belief>,
    pub next_id: u32,
}

impl SemanticMemory {
    pub fn new() -> Self {
        SemanticMemory { beliefs: Vec::new(), next_id: 1 }
    }

    fn confidence(strength: Fx, evidence: u16) -> Fx {
        let ev = Fx::ONE - (-(Fx::from_int(evidence as i64) / fx(2.0))).exp();
        (ev * (strength / fx(0.5)).min(Fx::ONE)).clamp01()
    }

    /// Crystallise explicit beliefs from slow (consolidated) associations.
    /// Returns the number of new or updated beliefs.
    pub fn crystallise(&mut self, assoc: &Assoc, now: u64, episodes_for: impl Fn(u16) -> Vec<u32>) -> u32 {
        let mut changed = 0;
        let mut candidates: Vec<(u16, Option<u8>, u8, Fx, u16)> = Vec::new();
        for c in 0..N_CUES {
            for o in 0..N_OUT {
                let w = assoc.pav_slow[c][o];
                let ev = assoc.pav_evidence[c];
                if w > fx(0.15) && (ev >= 3 || (assoc.flashbulb[c] && ev >= 1)) {
                    candidates.push((c as u16, None, o as u8, w, ev));
                }
                for a in 0..N_ACT {
                    let w = assoc.inst_slow[a * N_CUES + c][o];
                    let ev = assoc.evidence[a * N_CUES + c];
                    if w > fx(0.15) && (ev >= 3 || (assoc.flashbulb[c] && ev >= 1)) {
                        candidates.push((c as u16, Some(a as u8), o as u8, w, ev));
                    }
                }
            }
        }
        for (cue, action, outcome, w, ev) in candidates {
            let conf = Self::confidence(w, ev);
            if let Some(b) = self
                .beliefs
                .iter_mut()
                .find(|b| b.cue == cue && b.action == action && b.outcome == outcome && b.source == Source::OwnExperience)
            {
                if b.strength != w || b.evidence != ev {
                    b.strength = w;
                    b.evidence = ev;
                    b.confidence = conf;
                    b.t_updated = now;
                    changed += 1;
                }
                continue;
            }
            if self.beliefs.len() >= MAX_BELIEFS {
                let (wi, weakest) = self
                    .beliefs
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, b)| b.confidence.raw())
                    .map(|(i, b)| (i, b.confidence))
                    .expect("non-empty");
                if weakest >= conf {
                    continue;
                }
                self.beliefs.remove(wi);
            }
            let id = self.next_id;
            self.next_id += 1;
            self.beliefs.push(Belief {
                id,
                cue,
                action,
                outcome,
                strength: w,
                confidence: conf,
                evidence: ev,
                source: Source::OwnExperience,
                evidence_episodes: episodes_for(cue),
                t_learned: now,
                t_updated: now,
            });
            changed += 1;
        }
        changed
    }

    /// Beliefs relevant to a cue pattern, strongest first.
    pub fn relevant(&self, x: &CueVec) -> Vec<&Belief> {
        let mut v: Vec<(&Belief, Fx)> = self
            .beliefs
            .iter()
            .filter(|b| x[b.cue as usize] > fx(0.3))
            .map(|b| (b, b.confidence * x[b.cue as usize]))
            .collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.id.cmp(&b.0.id)));
        v.into_iter().map(|(b, _)| b).collect()
    }
}

// N_CTX is part of the belief contract (beliefs are context-free summaries).
const _: usize = N_CTX;

impl StableHash for Belief {
    fn stable_hash(&self, h: &mut StateHasher) {
        self.id.stable_hash(h);
        self.cue.stable_hash(h);
        self.action.map(|a| a as u64).stable_hash(h);
        self.outcome.stable_hash(h);
        self.strength.stable_hash(h);
        self.confidence.stable_hash(h);
        self.evidence.stable_hash(h);
        self.evidence_episodes.stable_hash(h);
        self.t_learned.stable_hash(h);
    }
}

impl StableHash for SemanticMemory {
    fn stable_hash(&self, h: &mut StateHasher) {
        self.beliefs.stable_hash(h);
        self.next_id.stable_hash(h);
    }
}
