//! P1 harness: child + fire. The harness knows world truth (it builds rooms
//! and measures contacts); the child's mind only ever receives sensory frames.

use alife_biology::Body;
use alife_core::rng::stream;
use alife_core::{fx, Fx, Rng, StableHash};
use alife_interface::MotorCommand;
use alife_mind::params::Personality;
use alife_mind::Mind;
use alife_world::sensors::sense;
use alife_world::{ObjProps, World};

pub const LEARN_TICKS: u64 = 2 * 86_400;
pub const PROBE_TICKS: u64 = 600;
pub const CHILD_AGE: i64 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Probe {
    FireA,
    FireB,
    HotMetal,
    OrangeFlower,
    Rock,
    Food,
}

impl Probe {
    pub const ALL: [Probe; 6] = [Probe::FireA, Probe::FireB, Probe::HotMetal, Probe::OrangeFlower, Probe::Rock, Probe::Food];
    pub fn props(self) -> ObjProps {
        match self {
            Probe::FireA => ObjProps::fire_a(),
            Probe::FireB => ObjProps::fire_b(),
            Probe::HotMetal => ObjProps::hot_metal(),
            Probe::OrangeFlower => ObjProps::orange_flower(),
            Probe::Rock => ObjProps::rock(),
            Probe::Food => ObjProps::berries(),
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Probe::FireA => "FIRE A",
            Probe::FireB => "FIRE B",
            Probe::HotMetal => "HOT METAL",
            Probe::OrangeFlower => "ORANGE FLOWER",
            Probe::Rock => "ROCK",
            Probe::Food => "FOOD",
        }
    }
}

pub fn new_mind(seed: u64) -> Mind {
    Mind::new(1, seed, Fx::from_int(CHILD_AGE), Personality::seeded(seed, 1))
}

/// Learning room: FIRE A, ROCK, FOOD, WATER and the child at seeded positions.
pub fn learning_world(seed: u64) -> World {
    let mut w = World::new(seed, 10, 10, seed.wrapping_mul(31).wrapping_add(7));
    let mut r = Rng::new(seed, 0, 0, stream::WORLD_SETUP);
    let free = |w: &World, r: &mut Rng| loop {
        let x = 1 + r.below(8) as i32;
        let y = 1 + r.below(8) as i32;
        let taken = w.objects.iter().any(|o| (o.x - x).abs() <= 1 && (o.y - y).abs() <= 1)
            || w.agents.iter().any(|a| a.x == x && a.y == y);
        if !taken {
            return (x, y);
        }
    };
    for props in [ObjProps::fire_a(), ObjProps::rock(), ObjProps::berries(), ObjProps::water_pool()] {
        let (x, y) = free(&w, &mut r);
        w.add_object(x, y, props);
    }
    let (x, y) = free(&w, &mut r);
    w.add_agent(x, y, Body::child(CHILD_AGE, fx(0.35), fx(0.35)));
    w
}

#[derive(Clone, Debug, Default)]
pub struct LearnLog {
    pub fire_contacts: u32,
    pub fire_contacts_hand: u32,
    pub fire_contacts_mouth: u32,
    pub fire_pain_events: u32,
    pub first_fire_pain_tick: Option<u64>,
    pub food_ingestions: u32,
    pub water_ingestions: u32,
    pub rock_contacts: u32,
    pub max_hand_damage: Fx,
    pub sleep_ticks: u64,
    pub final_hunger: Fx,
    pub final_thirst: Fx,
    pub hash: u64,
    /// Ticks at which the mind's trace was captured (first fire pain).
    pub burn_trace: Option<String>,
    pub decisions: u64,
    pub episodes: usize,
    pub beliefs: usize,
    pub concepts: usize,
    pub mind_bytes: usize,
    pub step_ns: u64,
}

impl LearnLog {
    pub fn burned(&self) -> bool {
        self.fire_pain_events > 0
    }
}

/// Runs the learning phase. Returns the trained mind, the log and the end tick.
pub fn run_learning(seed: u64, capture_trace: bool) -> (Mind, LearnLog, u64) {
    let mut w = learning_world(seed);
    let mut m = new_mind(seed);
    let fire_id = w.objects.iter().find(|o| o.props.label == "FIRE_A").map(|o| o.id).unwrap();
    let mut log = LearnLog::default();
    let mut prev_pain = Fx::ZERO;
    let mut traces_before: Option<String> = None;
    let t_start = std::time::Instant::now();
    let mut mind_ns: u128 = 0;
    while w.tick < LEARN_TICKS {
        let f = sense(&w, 0);
        if capture_trace {
            m.trace_enabled = true;
        }
        let t0 = std::time::Instant::now();
        let cmd = m.step(&f);
        mind_ns += t0.elapsed().as_nanos();
        let reaching_fire = w.agents[0].reaching.map(|(o, _)| o) == Some(fire_id);
        let target_fire = cmd.target() == Some(w.token_for(w.agents[0].id, fire_id));
        if capture_trace && log.burn_trace.is_none() && (reaching_fire || target_fire) {
            traces_before = m.last_trace.as_ref().map(|t| t.to_text());
        }
        let contacts_before = w.contacts.len();
        w.step(&[cmd]);
        for c in &w.contacts[contacts_before..] {
            match c.label {
                "FIRE_A" => {
                    log.fire_contacts += 1;
                    if c.site == "hand" {
                        log.fire_contacts_hand += 1;
                    } else {
                        log.fire_contacts_mouth += 1;
                    }
                }
                "FOOD" if c.ingested => log.food_ingestions += 1,
                "WATER" if c.ingested => log.water_ingestions += 1,
                "ROCK" => log.rock_contacts += 1,
                _ => {}
            }
        }
        let a = &w.agents[0];
        let pain = a.body.signals.pain;
        let near_fire = a.last_contact == Some(fire_id) || a.reaching.map(|(o, _)| o) == Some(fire_id);
        if pain > prev_pain + fx(0.1) && near_fire {
            log.fire_pain_events += 1;
            if log.first_fire_pain_tick.is_none() {
                log.first_fire_pain_tick = Some(w.tick);
                if capture_trace {
                    // The decision that led to pain, then the next moment (learning + reflex).
                    let f2 = sense(&w, 0);
                    let mut m2 = m.clone();
                    m2.trace_enabled = true;
                    m2.step(&f2);
                    let after = m2.last_trace.as_ref().map(|t| t.to_text()).unwrap_or_default();
                    log.burn_trace = Some(format!(
                        "=== DECISION THAT LED TO THE FIRST BURN ===\n{}\n=== NEXT MOMENT (outcome, learning, reflex) ===\n{}",
                        traces_before.clone().unwrap_or_default(),
                        after
                    ));
                }
            }
        }
        prev_pain = pain;
        log.max_hand_damage = log.max_hand_damage.max(a.body.damage_hand.max(a.body.damage_mouth));
        if a.body.asleep {
            log.sleep_ticks += 1;
        }
    }
    m.trace_enabled = false;
    let _ = t_start;
    log.step_ns = (mind_ns / LEARN_TICKS as u128) as u64;
    let a = &w.agents[0];
    log.final_hunger = a.body.signals.hunger;
    log.final_thirst = a.body.signals.thirst;
    log.hash = w.state_hash() ^ m.state_hash().rotate_left(1);
    log.decisions = m.stats.decisions;
    log.episodes = m.episodic.all().count();
    log.beliefs = m.semantic.beliefs.len();
    log.concepts = m.concepts.concepts.len();
    log.mind_bytes = m.memory_bytes();
    let end = w.tick;
    (m, log, end)
}

#[derive(Clone, Debug, Default)]
pub struct ProbeResult {
    pub contact: bool,
    pub latency: Option<u64>,
    pub min_dist: i32,
    pub ticks_adjacent: u64,
    pub inspects: u32,
    pub withdraws: u32,
    pub reaches: u32,
    pub aborted_reaches: u32,
    pub ingested: bool,
    pub ingest_latency: Option<u64>,
    pub ingestions: u32,
    pub pav_pain_first: Fx,
    pub touch_pain_first: Fx,
    pub fear_first: Fx,
    pub max_fear: Fx,
    pub concept_first: u16,
    pub pain_events: u32,
    pub first_trace: Option<String>,
    pub key_trace: Option<String>,
}

/// One probe: the (cloned) mind alone with one object in a fresh room.
pub fn run_probe(mind: &Mind, probe: Probe, seed: u64, start_tick: u64, capture: bool) -> ProbeResult {
    let mut m = mind.clone();
    m.new_environment();
    let salt = seed ^ 0xBEEF ^ (probe as u64) << 40;
    let mut w = World::new(seed.wrapping_add(1_000_003), 10, 10, salt);
    w.tick = start_tick;
    let oid = w.add_object(7, 5, probe.props());
    w.add_agent(2, 5, Body::child(CHILD_AGE, fx(0.5), fx(0.2)));
    let mut res = ProbeResult { min_dist: 99, ..Default::default() };
    let mut prev_pain = Fx::ZERO;
    let mut was_reaching = false;
    let end = start_tick + PROBE_TICKS;
    let mut first = true;
    while w.tick < end {
        let f = sense(&w, 0);
        if first {
            if let Some(&(_, _, pav, touch, _, cid)) = m.appraise(&f).first() {
                res.pav_pain_first = pav[0];
                res.touch_pain_first = touch[0];
                res.concept_first = cid;
            }
        }
        m.trace_enabled = capture && (first || res.key_trace.is_none());
        let cmd = m.step(&f);
        if first {
            res.fear_first = m.affect.fear;
            if capture {
                res.first_trace = m.last_trace.as_ref().map(|t| t.to_text());
            }
            first = false;
        }
        res.max_fear = res.max_fear.max(m.affect.fear);
        match cmd {
            MotorCommand::Inspect { .. } => res.inspects += 1,
            MotorCommand::Withdraw { .. } => res.withdraws += 1,
            _ => {}
        }
        let a = &w.agents[0];
        let reaching = a.reaching.is_some();
        // Capture the decision made right after a reach began (abort or follow through?).
        if capture && res.key_trace.is_none() && reaching {
            res.key_trace = m.last_trace.as_ref().map(|t| t.to_text());
        }
        let contacts_before = w.contacts.len();
        w.step(&[cmd]);
        let a = &w.agents[0];
        if a.reaching.is_some() && !was_reaching {
            res.reaches += 1;
        }
        let contact_now = w.contacts[contacts_before..].iter().any(|c| c.object == oid);
        if was_reaching && a.reaching.is_none() && !contact_now {
            res.aborted_reaches += 1;
        }
        was_reaching = a.reaching.is_some();
        if contact_now {
            if !res.contact {
                res.latency = Some(w.tick - start_tick);
            }
            res.contact = true;
            let n = w.contacts[contacts_before..].iter().filter(|c| c.object == oid && c.ingested).count() as u32;
            if n > 0 {
                if !res.ingested {
                    res.ingest_latency = Some(w.tick - start_tick);
                }
                res.ingested = true;
                res.ingestions += n;
            }
        }
        let o = &w.objects[0];
        let d = World::dist(a.x, a.y, o.x, o.y);
        res.min_dist = res.min_dist.min(d);
        if d <= 1 {
            res.ticks_adjacent += 1;
        }
        let pain = a.body.signals.pain;
        if pain > prev_pain + fx(0.1) {
            res.pain_events += 1;
        }
        prev_pain = pain;
    }
    res
}
