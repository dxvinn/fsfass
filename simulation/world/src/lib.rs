//! World truth for Phase 2 prototypes.
//!
//! Objects carry physical properties (temperature, emitted light, flicker,
//! colour, size, hardness, nutrition, water...). Physics turns actions into
//! physical consequences on bodies. Sensors turn truth into a `SensoryFrame`
//! containing only sensation strengths. Minds never see this crate.

pub mod objects;
pub mod sensors;

use alife_biology::{Body, Site};
use alife_core::hash::{StableHash, StateHasher};
use alife_core::rng::{key4, stream};
use alife_core::{fx, EventKey, EventQueue, Fx};
use alife_interface::{Ambient, MotorCommand, MotorResult, Taste, Token, Touch};
pub use objects::ObjProps;

#[derive(Clone, Debug, PartialEq)]
pub struct Object {
    pub id: u32,
    pub x: i32,
    pub y: i32,
    pub props: ObjProps,
    pub present: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AgentBody {
    pub id: u32,
    pub x: i32,
    pub y: i32,
    pub body: Body,
    pub last_result: MotorResult,
    /// Object touched / mouthed during the last tick, with the sensation produced.
    pub touched: Option<(u32, Touch)>,
    pub tasted: Option<(u32, Taste)>,
    /// Object last contacted (for the withdrawal reflex direction).
    pub last_contact: Option<u32>,
    /// Hand or face currently held close to an object, about to make contact.
    pub reaching: Option<(u32, Reach)>,
}

/// Contact is a two-step motor act: first the hand/face nears the surface
/// (feeling its radiant heat), then it makes contact on the next step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reach {
    Hand,
    Mouth,
}

/// Truth-side log of physical interactions, for experiment metrics only.
#[derive(Clone, Debug, PartialEq)]
pub struct ContactRecord {
    pub tick: u64,
    pub agent: u32,
    pub object: u32,
    pub label: &'static str,
    pub site: &'static str,
    pub ingested: bool,
    pub skin_c_after: Fx,
}

#[derive(Clone, Debug, PartialEq)]
pub enum WorldEvent {
    /// A consumed object grows back (e.g. berries).
    Regrow { object: u32 },
}

impl StableHash for WorldEvent {
    fn stable_hash(&self, h: &mut StateHasher) {
        match self {
            WorldEvent::Regrow { object } => {
                h.u64(1);
                h.u64(*object as u64);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct World {
    pub seed: u64,
    pub tick: u64,
    pub width: i32,
    pub height: i32,
    pub objects: Vec<Object>,
    pub agents: Vec<AgentBody>,
    /// Salt for tracking tokens; change it to make old tokens meaningless.
    pub token_salt: u64,
    pub ambient: Ambient,
    pub ambient_temp_c: Fx,
    pub events: EventQueue<WorldEvent>,
    pub contacts: Vec<ContactRecord>,
    pub regrow_ticks: u64,
}

impl World {
    pub fn new(seed: u64, width: i32, height: i32, token_salt: u64) -> World {
        World {
            seed,
            tick: 0,
            width,
            height,
            objects: Vec::new(),
            agents: Vec::new(),
            token_salt,
            ambient: Ambient { light: fx(0.7), ground: fx(0.4), enclosure: fx(0.8), sound: fx(0.2) },
            ambient_temp_c: fx(20.0),
            events: EventQueue::new(),
            contacts: Vec::new(),
            regrow_ticks: 3600,
        }
    }

    pub fn add_object(&mut self, x: i32, y: i32, props: ObjProps) -> u32 {
        let id = self.objects.len() as u32 + 1;
        self.objects.push(Object { id, x, y, props, present: true });
        id
    }

    pub fn add_agent(&mut self, x: i32, y: i32, body: Body) -> usize {
        let id = 1000 + self.agents.len() as u32;
        self.agents.push(AgentBody {
            id,
            x,
            y,
            body,
            last_result: MotorResult::Ok,
            touched: None,
            tasted: None,
            last_contact: None,
            reaching: None,
        });
        self.agents.len() - 1
    }

    /// Opaque token under which `agent` perceives `object` in this environment.
    pub fn token_for(&self, agent: u32, object: u32) -> Token {
        Token(key4(self.token_salt, object as u64, agent as u64, stream::TOKEN_SALT) >> 1)
    }

    fn object_by_token(&self, agent: u32, t: Token) -> Option<usize> {
        self.objects
            .iter()
            .position(|o| o.present && self.token_for(agent, o.id) == t)
    }

    fn occupied(&self, x: i32, y: i32, ignore_agent: usize) -> bool {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return true;
        }
        self.objects.iter().any(|o| o.present && o.x == x && o.y == y)
            || self
                .agents
                .iter()
                .enumerate()
                .any(|(i, a)| i != ignore_agent && a.x == x && a.y == y)
    }

    /// Chebyshev distance.
    pub fn dist(ax: i32, ay: i32, bx: i32, by: i32) -> i32 {
        (ax - bx).abs().max((ay - by).abs())
    }

    const DIRS: [(i32, i32); 8] = [(1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1), (1, -1)];

    /// One step that minimises (or maximises) distance to (tx, ty); deterministic tie-breaks.
    fn step_relative(&mut self, ai: usize, tx: i32, ty: i32, toward: bool) -> bool {
        let (ax, ay) = (self.agents[ai].x, self.agents[ai].y);
        let cur = Self::dist(ax, ay, tx, ty);
        let mut best: Option<(i64, usize)> = None;
        for (di, (dx, dy)) in Self::DIRS.iter().enumerate() {
            let (nx, ny) = (ax + dx, ay + dy);
            if self.occupied(nx, ny, ai) {
                continue;
            }
            let d = Self::dist(nx, ny, tx, ty);
            let e2 = ((nx - tx) * (nx - tx) + (ny - ty) * (ny - ty)) as i64;
            let score = if toward {
                if d >= cur && !(d == cur && e2 < ((ax - tx) * (ax - tx) + (ay - ty) * (ay - ty)) as i64) {
                    continue;
                }
                d as i64 * 10_000 + e2
            } else {
                if d < cur {
                    continue;
                }
                -(d as i64 * 10_000 + e2)
            };
            if best.map_or(true, |(s, _)| score < s) {
                best = Some((score, di));
            }
        }
        if let Some((_, di)) = best {
            let (dx, dy) = Self::DIRS[di];
            self.agents[ai].x += dx;
            self.agents[ai].y += dy;
            true
        } else {
            false
        }
    }

    /// Advance the world by one second. `commands[i]` is the command of agent i.
    pub fn step(&mut self, commands: &[MotorCommand]) {
        self.tick += 1;
        for (_, ev) in self.events.pop_due(self.tick) {
            match ev {
                WorldEvent::Regrow { object } => {
                    if let Some(o) = self.objects.iter_mut().find(|o| o.id == object) {
                        o.present = true;
                        o.props.portions = o.props.max_portions;
                    }
                }
            }
        }
        for ai in 0..self.agents.len() {
            self.agents[ai].touched = None;
            self.agents[ai].tasted = None;
            let cmd = commands.get(ai).copied().unwrap_or(MotorCommand::Rest);
            let result = self.execute(ai, cmd);
            self.agents[ai].last_result = result;
        }
        self.radiant_heat();
        for a in self.agents.iter_mut() {
            a.body.step();
        }
    }

    fn execute(&mut self, ai: usize, cmd: MotorCommand) -> MotorResult {
        if self.agents[ai].body.asleep {
            return MotorResult::Overridden;
        }
        // Innate withdrawal reflex: a limb or the head is pulled away from a painful contact.
        let sig = self.agents[ai].body.signals;
        if sig.reflex_hand || sig.reflex_mouth {
            self.agents[ai].reaching = None;
            if let Some(oid) = self.agents[ai].last_contact {
                if let Some(o) = self.objects.iter().find(|o| o.id == oid) {
                    let (tx, ty) = (o.x, o.y);
                    self.step_relative(ai, tx, ty, false);
                }
            }
            return MotorResult::Overridden;
        }
        let agent_id = self.agents[ai].id;
        let prev_reach = self.agents[ai].reaching.take();
        match cmd {
            MotorCommand::Rest => MotorResult::Ok,
            MotorCommand::Wander { dir } => {
                let (dx, dy) = Self::DIRS[(dir % 8) as usize];
                let (nx, ny) = (self.agents[ai].x + dx, self.agents[ai].y + dy);
                if self.occupied(nx, ny, ai) {
                    MotorResult::Failed
                } else {
                    self.agents[ai].x = nx;
                    self.agents[ai].y = ny;
                    MotorResult::Ok
                }
            }
            MotorCommand::Approach { target }
            | MotorCommand::Inspect { target }
            | MotorCommand::Touch { target }
            | MotorCommand::Mouth { target }
            | MotorCommand::Withdraw { target } => {
                let Some(oi) = self.object_by_token(agent_id, target) else {
                    return MotorResult::Failed;
                };
                let (ox, oy) = (self.objects[oi].x, self.objects[oi].y);
                let d = Self::dist(self.agents[ai].x, self.agents[ai].y, ox, oy);
                match cmd {
                    MotorCommand::Approach { .. } => {
                        if d > 1 && !self.step_relative(ai, ox, oy, true) {
                            return MotorResult::Failed;
                        }
                        MotorResult::Ok
                    }
                    MotorCommand::Inspect { .. } => {
                        if d > 2 {
                            self.step_relative(ai, ox, oy, true);
                        } else if d < 2 {
                            self.step_relative(ai, ox, oy, false);
                        }
                        MotorResult::Ok
                    }
                    MotorCommand::Withdraw { .. } => {
                        if self.step_relative(ai, ox, oy, false) {
                            MotorResult::Ok
                        } else {
                            MotorResult::Failed
                        }
                    }
                    MotorCommand::Touch { .. } => {
                        if d > 1 {
                            if !self.step_relative(ai, ox, oy, true) {
                                return MotorResult::Failed;
                            }
                            return MotorResult::Ok;
                        }
                        let oid = self.objects[oi].id;
                        if prev_reach == Some((oid, Reach::Hand)) {
                            self.touch(ai, oi);
                        } else {
                            self.agents[ai].reaching = Some((oid, Reach::Hand));
                            self.agents[ai].last_contact = Some(oid);
                        }
                        MotorResult::Ok
                    }
                    MotorCommand::Mouth { .. } => {
                        if d > 1 {
                            if !self.step_relative(ai, ox, oy, true) {
                                return MotorResult::Failed;
                            }
                            return MotorResult::Ok;
                        }
                        let oid = self.objects[oi].id;
                        if prev_reach == Some((oid, Reach::Mouth)) {
                            self.mouth(ai, oi);
                        } else {
                            self.agents[ai].reaching = Some((oid, Reach::Mouth));
                            self.agents[ai].last_contact = Some(oid);
                        }
                        MotorResult::Ok
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    fn touch(&mut self, ai: usize, oi: usize) {
        let p = self.objects[oi].props.clone();
        let oid = self.objects[oi].id;
        let body = &mut self.agents[ai].body;
        body.contact(Site::Hand, p.temp_c, p.conduct);
        let feel = Touch {
            thermal: ((p.temp_c - fx(33.0)) / fx(40.0)).clamp(Fx::NEG_ONE, Fx::ONE),
            firmness: p.hardness,
            wetness: p.wet,
        };
        self.agents[ai].touched = Some((oid, feel));
        self.agents[ai].last_contact = Some(oid);
        self.log_contact(ai, oi, "hand", false);
    }

    fn mouth(&mut self, ai: usize, oi: usize) {
        let p = self.objects[oi].props.clone();
        let oid = self.objects[oi].id;
        let swallowable = p.liquid || (p.hardness < fx(0.5) && p.portions != 0 && p.size < fx(0.5));
        {
            let body = &mut self.agents[ai].body;
            body.contact(Site::Mouth, p.temp_c, p.conduct);
            if swallowable {
                body.ingest(p.kcal_per_portion, p.water_per_portion);
            }
        }
        if swallowable && !p.liquid && p.portions > 0 {
            let o = &mut self.objects[oi];
            o.props.portions -= 1;
            if o.props.portions == 0 {
                o.present = false;
                let key = EventKey {
                    tick: self.tick + self.regrow_ticks,
                    priority: 0,
                    entity: oid as u64,
                    discriminator: 1,
                };
                self.events.push(key, WorldEvent::Regrow { object: oid });
            }
        }
        let taste = Taste {
            sweet: if swallowable { p.sugar } else { p.sugar * fx(0.3) },
            bitter: p.bitter,
            liquid: if p.liquid { Fx::ONE } else { p.wet * fx(0.5) },
            swallowed: if swallowable { Fx::ONE } else { Fx::ZERO },
        };
        self.agents[ai].tasted = Some((oid, taste));
        self.agents[ai].last_contact = Some(oid);
        self.log_contact(ai, oi, "mouth", swallowable);
    }

    fn log_contact(&mut self, ai: usize, oi: usize, site: &'static str, ingested: bool) {
        let a = &self.agents[ai];
        let skin = if site == "hand" { a.body.skin_hand } else { a.body.skin_mouth };
        self.contacts.push(ContactRecord {
            tick: self.tick,
            agent: a.id,
            object: self.objects[oi].id,
            label: self.objects[oi].props.label,
            site,
            ingested,
            skin_c_after: skin,
        });
    }

    /// Radiant warmth from object `o` at Chebyshev/Euclid distance from (x, y), 0..1 scale.
    pub fn radiant_from(&self, o: &Object, x: i32, y: i32) -> Fx {
        let excess = o.props.temp_c - fx(30.0);
        if !excess.is_positive() {
            return Fx::ZERO;
        }
        let d2 = Fx::from_int(((o.x - x) * (o.x - x) + (o.y - y) * (o.y - y)) as i64);
        let strength = excess / fx(700.0) * o.props.size * fx(2.5);
        (strength / (d2 + fx(0.5))).clamp01()
    }

    /// Radiant warmth on a hand or face held a few centimetres from the object's surface.
    pub fn radiant_close(&self, o: &Object) -> Fx {
        let excess = o.props.temp_c - fx(30.0);
        if !excess.is_positive() {
            return Fx::ZERO;
        }
        let strength = excess / fx(700.0) * o.props.size * fx(2.5);
        (strength / fx(0.55)).clamp01()
    }

    fn radiant_heat(&mut self) {
        for ai in 0..self.agents.len() {
            let (x, y) = (self.agents[ai].x, self.agents[ai].y);
            let mut total = Fx::ZERO;
            for o in self.objects.iter().filter(|o| o.present) {
                total += self.radiant_from(o, x, y);
            }
            self.agents[ai].body.add_radiant(total);
            if let Some((oid, reach)) = self.agents[ai].reaching {
                if let Some(o) = self.objects.iter().find(|o| o.id == oid && o.present) {
                    let close = self.radiant_close(o);
                    let site = match reach {
                        Reach::Hand => Site::Hand,
                        Reach::Mouth => Site::Mouth,
                    };
                    self.agents[ai].body.add_radiant_site(site, close);
                }
            }
        }
    }
}

impl StableHash for World {
    fn stable_hash(&self, h: &mut StateHasher) {
        self.seed.stable_hash(h);
        self.tick.stable_hash(h);
        self.token_salt.stable_hash(h);
        h.u64(self.objects.len() as u64);
        for o in &self.objects {
            o.id.stable_hash(h);
            o.x.stable_hash(h);
            o.y.stable_hash(h);
            o.present.stable_hash(h);
            o.props.portions.stable_hash(h);
            o.props.temp_c.stable_hash(h);
        }
        h.u64(self.agents.len() as u64);
        for a in &self.agents {
            a.id.stable_hash(h);
            a.x.stable_hash(h);
            a.y.stable_hash(h);
            a.body.stable_hash(h);
            (a.last_result as u8 as u64).stable_hash(h);
            a.reaching.map(|(o, r)| (o, r as u8)).stable_hash(h);
        }
        self.events.stable_hash(h);
        (self.contacts.len() as u64).stable_hash(h);
    }
}
