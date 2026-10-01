//! Attention bottleneck and working memory (global workspace, report 01 §8.3).
//!
//! Percepts, needs and recalled memories compete for K workspace slots by
//! salience. Only workspace items are thought about (options, learning at
//! full rate). Working memory keeps the current goal, target and last action
//! across steps; it holds tokens only short-term and is never consolidated.

use alife_core::hash::{StableHash, StateHasher};
use alife_core::Fx;
use alife_interface::{MotorCommand, Token};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemKind {
    Percept(Token),
    Need(u8),
    Memory(u32),
}

#[derive(Clone, Debug, PartialEq)]
pub struct WsItem {
    pub kind: ItemKind,
    pub salience: Fx,
}

pub fn need_label(n: u8) -> &'static str {
    ["hunger", "thirst", "pain", "fatigue"][n as usize]
}

#[derive(Clone, Debug, PartialEq)]
pub enum Goal {
    None,
    Explore(Token),
    Relieve(u8, Token),
    Avoid(Token),
    Rest,
    Wander,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorkingMemory {
    pub slots: Vec<WsItem>,
    pub capacity: usize,
    pub goal: Goal,
    pub last_command: Option<MotorCommand>,
}

impl WorkingMemory {
    pub fn new(capacity: usize) -> Self {
        WorkingMemory { slots: Vec::new(), capacity, goal: Goal::None, last_command: None }
    }

    pub fn holds(&self, t: Token) -> bool {
        self.slots.iter().any(|s| s.kind == ItemKind::Percept(t))
    }

    /// Keep the top-K candidates by salience (deterministic tie-break).
    pub fn admit(&mut self, mut candidates: Vec<WsItem>, k: usize) {
        candidates.sort_by(|a, b| b.salience.cmp(&a.salience).then(key(&a.kind).cmp(&key(&b.kind))));
        candidates.truncate(k.max(1));
        self.slots = candidates;
    }
}

fn key(k: &ItemKind) -> (u8, u64) {
    match *k {
        ItemKind::Percept(t) => (0, t.0),
        ItemKind::Need(n) => (1, n as u64),
        ItemKind::Memory(m) => (2, m as u64),
    }
}

impl StableHash for WorkingMemory {
    fn stable_hash(&self, h: &mut StateHasher) {
        h.u64(self.slots.len() as u64);
        for s in &self.slots {
            let (a, b) = key(&s.kind);
            h.u64(a as u64);
            h.u64(b);
            s.salience.stable_hash(h);
        }
        self.last_command.stable_hash(h);
    }
}
