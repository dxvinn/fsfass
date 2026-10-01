//! Deterministic event queue and multi-rate scheduling helpers.
//!
//! Events are ordered by a total key (tick, priority, entity, discriminator).
//! The discriminator is supplied by the producer (normally a hash of the
//! payload), so the processing order never depends on insertion order: events
//! produced by different threads or in a different order resolve identically.

use crate::hash::{StableHash, StateHasher};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventKey {
    pub tick: u64,
    pub priority: u8,
    pub entity: u64,
    pub discriminator: u64,
}

#[derive(Clone, Debug)]
struct Entry<E> {
    key: EventKey,
    payload: E,
}

impl<E> PartialEq for Entry<E> {
    fn eq(&self, o: &Self) -> bool {
        self.key == o.key
    }
}
impl<E> Eq for Entry<E> {}
impl<E> PartialOrd for Entry<E> {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}
impl<E> Ord for Entry<E> {
    fn cmp(&self, o: &Self) -> Ordering {
        // BinaryHeap is a max-heap; invert so the smallest key pops first.
        o.key.cmp(&self.key)
    }
}

#[derive(Clone, Debug)]
pub struct EventQueue<E> {
    heap: BinaryHeap<Entry<E>>,
}

impl<E: Clone> Default for EventQueue<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: Clone> EventQueue<E> {
    pub fn new() -> Self {
        EventQueue { heap: BinaryHeap::new() }
    }

    pub fn push(&mut self, key: EventKey, payload: E) {
        self.heap.push(Entry { key, payload });
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    pub fn peek_tick(&self) -> Option<u64> {
        self.heap.peek().map(|e| e.key.tick)
    }

    /// Removes and returns every event due at or before `tick`, in key order.
    pub fn pop_due(&mut self, tick: u64) -> Vec<(EventKey, E)> {
        let mut out = Vec::new();
        while let Some(top) = self.heap.peek() {
            if top.key.tick > tick {
                break;
            }
            let e = self.heap.pop().expect("peeked");
            out.push((e.key, e.payload));
        }
        out
    }

    /// Entries in canonical order (for hashing and inspection).
    pub fn sorted(&self) -> Vec<(EventKey, E)> {
        let mut v: Vec<(EventKey, E)> =
            self.heap.iter().map(|e| (e.key, e.payload.clone())).collect();
        v.sort_by(|a, b| a.0.cmp(&b.0));
        v
    }
}

impl<E: Clone + StableHash> StableHash for EventQueue<E> {
    fn stable_hash(&self, h: &mut StateHasher) {
        let v = self.sorted();
        h.u64(v.len() as u64);
        for (k, p) in v {
            h.u64(k.tick);
            h.u64(k.priority as u64);
            h.u64(k.entity);
            h.u64(k.discriminator);
            p.stable_hash(h);
        }
    }
}

/// A system that runs every `period` ticks. Entities are spread across the
/// period by a stable hash so the load is flat and each entity is processed
/// exactly once per period.
#[derive(Clone, Copy, Debug)]
pub struct RateSchedule {
    pub period: u64,
}

impl RateSchedule {
    pub const fn every(period: u64) -> Self {
        RateSchedule { period }
    }

    /// True if `entity` is due on `tick`.
    #[inline]
    pub fn due(&self, entity: u64, tick: u64) -> bool {
        if self.period <= 1 {
            return true;
        }
        (tick.wrapping_add(crate::rng::mix64(entity))) % self.period == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl StableHash for u8x {
        fn stable_hash(&self, h: &mut StateHasher) {
            h.u64(self.0 as u64);
        }
    }
    #[derive(Clone, Debug, PartialEq)]
    #[allow(non_camel_case_types)]
    struct u8x(u8);

    #[test]
    fn insertion_order_does_not_matter() {
        let keys: Vec<(EventKey, u8x)> = (0..50u64)
            .map(|i| {
                (
                    EventKey { tick: i % 5, priority: (i % 3) as u8, entity: i % 7, discriminator: i },
                    u8x(i as u8),
                )
            })
            .collect();
        let mut a = EventQueue::new();
        for (k, p) in keys.iter() {
            a.push(*k, p.clone());
        }
        let mut b = EventQueue::new();
        for (k, p) in keys.iter().rev() {
            b.push(*k, p.clone());
        }
        assert_eq!(a.state_hash(), b.state_hash());
        for t in 0..5 {
            let da: Vec<_> = a.pop_due(t).into_iter().map(|x| x.1).collect();
            let db: Vec<_> = b.pop_due(t).into_iter().map(|x| x.1).collect();
            assert_eq!(da, db);
        }
    }

    #[test]
    fn rate_schedule_visits_each_entity_once_per_period() {
        let s = RateSchedule::every(10);
        for e in 0..100u64 {
            let hits = (0..10u64).filter(|&t| s.due(e, t)).count();
            assert_eq!(hits, 1);
        }
    }
}
