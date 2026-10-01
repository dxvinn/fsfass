//! Save and load.
//!
//! The simulation is deterministic, so a save does not need to copy every
//! mind: it records the world seed and size plus a journal of everything that
//! came from outside the simulation (god powers, speed / cognition-LOD changes,
//! which human is selected and thinking at full rate). Loading rebuilds the
//! world by replaying that journal tick for tick, then checks a fingerprint of
//! the saved state so a mismatch is reported instead of silently accepted.

use crate::Sim;
use alife_core::hash::StateHasher;

#[derive(Clone, Debug, PartialEq)]
pub enum JournalEntry {
    God { tick: u64, power: String, x: i32, y: i32 },
    Lod { tick: u64, speed: u32 },
    Select { tick: u64, id: Option<u32> },
}

impl JournalEntry {
    pub fn tick(&self) -> u64 {
        match self {
            JournalEntry::God { tick, .. } | JournalEntry::Lod { tick, .. } | JournalEntry::Select { tick, .. } => *tick,
        }
    }
}

const MAGIC: &str = "GENESIS-SAVE 1";

/// Serialise the world as seed + journal + fingerprint (plain text).
pub fn save_text(sim: &Sim) -> String {
    let mut o = String::new();
    o.push_str(MAGIC);
    o.push('\n');
    o.push_str(&format!("seed {}\nsize {} {}\ntick {}\nfingerprint {}\n", sim.seed, sim.w, sim.h, sim.tick, fingerprint(sim)));
    for e in &sim.journal {
        match e {
            JournalEntry::God { tick, power, x, y } => o.push_str(&format!("god {tick} {power} {x} {y}\n")),
            JournalEntry::Lod { tick, speed } => o.push_str(&format!("lod {tick} {speed}\n")),
            JournalEntry::Select { tick, id } => o.push_str(&format!("select {tick} {}\n", id.map_or(-1, |v| v as i64))),
        }
    }
    o
}

/// A hash of the observable state: every creature and object, weather, history.
pub fn fingerprint(sim: &Sim) -> u64 {
    let mut h = StateHasher::new();
    h.u64(sim.tick);
    h.u64(sim.creatures.len() as u64);
    for c in &sim.creatures {
        h.u64(c.id as u64);
        h.u64(c.alive as u64);
        h.i64(c.x as i64);
        h.i64(c.y as i64);
        h.i64(c.health.raw());
        h.i64(c.body.energy.raw());
        if let Some(m) = c.mind.as_ref() {
            h.u64(m.stats.episodes_stored as u64);
            h.u64(m.concepts.concepts.len() as u64);
        }
    }
    h.u64(sim.objs.len() as u64);
    for o in &sim.objs {
        h.u64(o.id as u64);
        h.i64(o.x as i64);
        h.i64(o.y as i64);
        h.i64(o.props.portions as i64);
    }
    h.u64(sim.history.len() as u64);
    h.u64(sim.weather as u64);
    h.finish()
}

/// A load in progress: the world is rebuilt a slice at a time so the game can
/// show progress instead of freezing.
pub struct Replay {
    pub sim: Sim,
    entries: Vec<JournalEntry>,
    next: usize,
    pub target_tick: u64,
    expected: u64,
}

impl Replay {
    pub fn parse(text: &str) -> Result<Replay, String> {
        let mut lines = text.lines();
        if lines.next().map(str::trim) != Some(MAGIC) {
            return Err("not a Genesis save file".into());
        }
        let (mut seed, mut w, mut h, mut tick, mut fp) = (None, None, None, None, None);
        let mut entries = Vec::new();
        for line in lines {
            let f: Vec<&str> = line.split_whitespace().collect();
            let num = |i: usize| -> Result<i64, String> {
                f.get(i).ok_or_else(|| format!("short line: {line}"))?.parse::<i64>().map_err(|e| format!("{e} in: {line}"))
            };
            match f.first().copied() {
                Some("seed") => seed = Some(f.get(1).and_then(|s| s.parse::<u64>().ok()).ok_or("bad seed")?),
                Some("size") => {
                    w = Some(num(1)? as i32);
                    h = Some(num(2)? as i32);
                }
                Some("tick") => tick = Some(num(1)? as u64),
                Some("fingerprint") => fp = Some(f.get(1).and_then(|s| s.parse::<u64>().ok()).ok_or("bad fingerprint")?),
                Some("god") => entries.push(JournalEntry::God {
                    tick: num(1)? as u64,
                    power: f.get(2).ok_or("missing power")?.to_string(),
                    x: num(3)? as i32,
                    y: num(4)? as i32,
                }),
                Some("lod") => entries.push(JournalEntry::Lod { tick: num(1)? as u64, speed: num(2)? as u32 }),
                Some("select") => {
                    let v = num(2)?;
                    entries.push(JournalEntry::Select { tick: num(1)? as u64, id: (v >= 0).then_some(v as u32) })
                }
                Some(_) | None => {}
            }
        }
        let (Some(seed), Some(w), Some(h), Some(target_tick), Some(expected)) = (seed, w, h, tick, fp) else {
            return Err("save file is missing its header".into());
        };
        Ok(Replay { sim: Sim::new(seed, w, h), entries, next: 0, target_tick, expected })
    }

    /// Advance the rebuild by at most `max_ticks`. Returns true when finished.
    pub fn run(&mut self, max_ticks: u64) -> bool {
        let stop = (self.sim.tick + max_ticks).min(self.target_tick);
        loop {
            // Apply every journal entry recorded at the current tick, in order.
            while let Some(e) = self.entries.get(self.next) {
                if e.tick() != self.sim.tick {
                    break;
                }
                match e.clone() {
                    JournalEntry::God { power, x, y, .. } => {
                        self.sim.god(&power, x, y);
                    }
                    JournalEntry::Lod { speed, .. } => self.sim.set_speed_lod(speed),
                    JournalEntry::Select { id, .. } => self.sim.set_selected(id),
                }
                self.next += 1;
            }
            if self.sim.tick >= stop {
                break;
            }
            self.sim.step();
        }
        self.sim.tick >= self.target_tick
    }

    pub fn progress(&self) -> f64 {
        self.sim.tick as f64 / self.target_tick.max(1) as f64
    }

    /// True if the rebuilt world is identical to the one that was saved.
    pub fn verified(&self) -> bool {
        fingerprint(&self.sim) == self.expected
    }
}
