//! PROJECT GENESIS — deterministic world simulation.
//!
//! Truth lives here (terrain, climate, objects, bodies, genomes). Humans think
//! with ArtificialMind V0 (`alife-mind`), which only ever receives sensory
//! frames built by `sense_human`. Animals use cheaper modular cognition.
//! Nothing here scripts civilisation: no houses, farms, tech trees or gods.

pub mod export;
pub mod genetics;
pub mod names;
pub mod render;
pub mod terrain;

use alife_biology::{Body, Site};
use alife_core::rng::{key4, stream};
use alife_core::{fx, Fx, Rng};
use alife_interface::{Ambient, Interoception, MotorCommand, MotorResult, Percept, SensoryFrame, Taste, Token, Touch, Visual};
use alife_mind::params::{MindParams, Personality};
use alife_mind::Mind;
use alife_world::ObjProps;
use genetics::{Genome, Traits};
use terrain::{Biome, Tile};

pub const TICKS_PER_DAY: u64 = 86_400;
/// Life is compressed for playability: one year of age passes every 2 game days.
pub const DAYS_PER_YEAR: u64 = 2;
pub const TICKS_PER_YEAR: u64 = TICKS_PER_DAY * DAYS_PER_YEAR;
pub const VISION: i32 = 9;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Kind {
    Human = 0,
    Grazer = 1,
    Predator = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ObjKind {
    BerryBush = 0,
    Tree = 1,
    Stone = 2,
    Flint = 3,
    Fire = 4,
    Carcass = 5,
    Flake = 6,
    Remains = 7,
}

#[derive(Clone, Debug)]
pub struct Obj {
    pub id: u32,
    pub kind: ObjKind,
    pub x: i32,
    pub y: i32,
    pub props: ObjProps,
    pub alive: bool,
    /// Kind-specific timer: fire fuel, carcass decay, bush regrowth.
    pub timer: i64,
    /// Ticks spent next to a fire (cooking).
    pub heat_ticks: u32,
    pub cooked: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Relation {
    pub other: u32,
    pub familiarity: Fx,
    pub affection: Fx,
    pub trust: Fx,
    pub attraction: Fx,
    pub fear: Fx,
    pub resentment: Fx,
    pub last_seen: u64,
    pub last_touch: u64,
}

/// Cheap animal cognition (report 04 §C.5 tiers T2/T3): drives, innate
/// responses, a learned fear of humans, remembered water.
#[derive(Clone, Debug, Default)]
pub struct AnimalBrain {
    pub energy: Fx,
    pub water: Fx,
    pub fatigue: Fx,
    pub fear_of_humans: Fx,
    pub last_water: Option<(i32, i32)>,
    pub target: Option<u32>,
    pub state: &'static str,
    pub flee_from: Option<(i32, i32)>,
    /// Predators back off after biting a human until this tick.
    pub retreat_until: u64,
}

#[derive(Clone, Debug)]
pub struct Creature {
    pub id: u32,
    pub kind: Kind,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub px: i32,
    pub py: i32,
    pub alive: bool,
    pub female: bool,
    pub born_tick: i64,
    pub genome: Genome,
    pub traits: Traits,
    pub health: Fx,
    pub body: Body,
    pub mind: Option<Box<Mind>>,
    pub animal: AnimalBrain,
    pub action: String,
    pub last_cmd: MotorCommand,
    pub reaching: Option<(Target, Site)>,
    pub tokens: Vec<(Token, Target)>,
    pub touched: Option<(Target, Touch)>,
    pub tasted: Option<(Target, Taste)>,
    pub last_contact: Option<Target>,
    pub last_result: MotorResult,
    pub loneliness: Fx,
    pub social_comfort: Fx,
    pub mother: Option<u32>,
    pub father: Option<u32>,
    pub partner: Option<u32>,
    pub children: Vec<u32>,
    pub pregnant: Option<(u32, u64)>,
    pub relations: Vec<Relation>,
    pub blessed_until: u64,
    pub cursed_until: u64,
    pub cause_of_death: Option<String>,
    pub died_tick: u64,
    pub think_interval: u32,
    pub skills: Vec<(String, Fx)>,
    pub last_mind_age: i64,
    pub kills: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Obj(u32),
    Water(i32, i32),
    Creature(u32),
}

#[derive(Clone, Debug)]
pub struct HistoryEvent {
    pub tick: u64,
    pub kind: String,
    pub text: String,
    pub x: i32,
    pub y: i32,
    pub important: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Weather {
    Clear = 0,
    Cloudy = 1,
    Rain = 2,
    Storm = 3,
}

pub struct Sim {
    pub seed: u64,
    pub tick: u64,
    pub w: i32,
    pub h: i32,
    pub tiles: Vec<Tile>,
    pub objs: Vec<Obj>,
    pub creatures: Vec<Creature>,
    pub history: Vec<HistoryEvent>,
    pub firsts: Vec<String>,
    pub weather: Weather,
    pub weather_until: u64,
    pub god_temp: Fx,
    pub rain_until: u64,
    pub next_id: u32,
    pub selected: Option<u32>,
    /// Cognitive level of detail: humans (other than the selected one) think
    /// every `human_interval` ticks, animals every `animal_interval`.
    pub human_interval: u32,
    pub animal_interval: u32,
    pub mind_steps: u64,
    pub births: u32,
    pub deaths: u32,
    pub prof_sense_ns: u64,
    pub prof_mind_ns: u64,
    pub prof_rest_ns: u64,
    /// Steps to the nearest drinkable water over walkable land (animal navigation);
    /// empty means "recompute".
    pub water_dist: Vec<u16>,
}

fn props_for(kind: ObjKind) -> ObjProps {
    match kind {
        // A whole bush laden with berries: each portion is a good handful (60 kcal).
        ObjKind::BerryBush => ObjProps { portions: 20, max_portions: 20, kcal_per_portion: fx(60.0), water_per_portion: fx(15.0), ..ObjProps::berries() },
        ObjKind::Fire => ObjProps::fire_a(),
        ObjKind::Stone => ObjProps::rock(),
        ObjKind::Flint => ObjProps {
            label: "FLINT",
            hue_deg: fx(30.0),
            saturation: fx(0.12),
            reflect: fx(0.3),
            gloss: fx(0.55),
            roughness: fx(0.35),
            roundness: fx(0.4),
            size: fx(0.2),
            ..ObjProps::rock()
        },
        ObjKind::Flake => ObjProps {
            label: "FLAKE",
            hue_deg: fx(30.0),
            saturation: fx(0.12),
            reflect: fx(0.35),
            gloss: fx(0.8),
            roughness: fx(0.1),
            roundness: fx(0.05),
            size: fx(0.08),
            hardness: fx(0.9),
            ..ObjProps::rock()
        },
        ObjKind::Tree => ObjProps {
            label: "TREE",
            hue_deg: fx(110.0),
            saturation: fx(0.6),
            reflect: fx(0.35),
            size: fx(0.9),
            roundness: fx(0.6),
            roughness: fx(0.8),
            gloss: fx(0.1),
            hardness: fx(0.7),
            bitter: fx(0.6),
            conduct: fx(0.3),
            ..ObjProps::rock()
        },
        ObjKind::Carcass => ObjProps {
            label: "CARCASS",
            hue_deg: fx(5.0),
            saturation: fx(0.55),
            reflect: fx(0.35),
            size: fx(0.4),
            roundness: fx(0.5),
            roughness: fx(0.5),
            gloss: fx(0.4),
            hardness: fx(0.3),
            wet: fx(0.4),
            kcal_per_portion: fx(40.0),
            water_per_portion: fx(5.0),
            sugar: fx(0.1),
            bitter: fx(0.2),
            portions: 14,
            max_portions: 14,
            ..ObjProps::rock()
        },
        ObjKind::Remains => ObjProps {
            label: "REMAINS",
            hue_deg: fx(30.0),
            saturation: fx(0.15),
            reflect: fx(0.4),
            size: fx(0.4),
            hardness: fx(0.4),
            ..ObjProps::rock()
        },
    }
}

fn creature_visual(c: &Creature) -> Visual {
    match c.kind {
        Kind::Human => Visual {
            hue_deg: fx(25.0),
            saturation: fx(0.35),
            brightness: fx(0.55) - c.traits.skin * fx(0.25),
            flicker: Fx::ZERO,
            size: fx(0.45) * c.traits.height,
            roundness: fx(0.55),
            texture: fx(0.3),
            gloss: fx(0.2),
        },
        Kind::Grazer => Visual {
            hue_deg: fx(32.0),
            saturation: fx(0.5),
            brightness: fx(0.45),
            flicker: Fx::ZERO,
            size: fx(0.5),
            roundness: fx(0.6),
            texture: fx(0.6),
            gloss: fx(0.15),
        },
        Kind::Predator => Visual {
            hue_deg: fx(210.0),
            saturation: fx(0.08),
            brightness: fx(0.4),
            flicker: Fx::ZERO,
            size: fx(0.45),
            roundness: fx(0.3),
            texture: fx(0.85),
            gloss: fx(0.1),
        },
    }
}

fn water_visual() -> Visual {
    Visual {
        hue_deg: fx(205.0),
        saturation: fx(0.5),
        brightness: fx(0.5),
        flicker: fx(0.08),
        size: fx(0.6),
        roundness: fx(0.8),
        texture: Fx::ZERO,
        gloss: fx(0.95),
    }
}

const DIRS: [(i32, i32); 8] = [(1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1), (1, -1)];

pub fn cheb(ax: i32, ay: i32, bx: i32, by: i32) -> i32 {
    (ax - bx).abs().max((ay - by).abs())
}

impl Sim {
    pub fn new(seed: u64, w: i32, h: i32) -> Sim {
        let tiles = terrain::generate(seed, w, h);
        let mut s = Sim {
            seed,
            tick: 6 * 3600,
            w,
            h,
            tiles,
            objs: Vec::new(),
            creatures: Vec::new(),
            history: Vec::new(),
            firsts: Vec::new(),
            weather: Weather::Clear,
            weather_until: 0,
            god_temp: Fx::ZERO,
            rain_until: 0,
            next_id: 1,
            selected: None,
            human_interval: 1,
            animal_interval: 1,
            mind_steps: 0,
            births: 0,
            deaths: 0,
            prof_sense_ns: 0,
            prof_mind_ns: 0,
            water_dist: Vec::new(),
            prof_rest_ns: 0,
        };
        s.populate();
        s.record("world", "The world was created.".into(), w / 2, h / 2, true);
        s
    }

    fn rng(&self, entity: u64, stream_id: u64) -> Rng {
        Rng::new(self.seed, entity, self.tick, stream_id)
    }

    pub fn tile(&self, x: i32, y: i32) -> &Tile {
        &self.tiles[(y * self.w + x) as usize]
    }

    fn tile_mut(&mut self, x: i32, y: i32) -> &mut Tile {
        let w = self.w;
        &mut self.tiles[(y * w + x) as usize]
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.w && y < self.h
    }

    fn obj_at(&self, x: i32, y: i32) -> bool {
        self.objs.iter().any(|o| o.alive && o.x == x && o.y == y && o.kind != ObjKind::Flake)
    }

    pub fn walkable(&self, x: i32, y: i32) -> bool {
        self.in_bounds(x, y) && self.tile(x, y).walkable() && !self.obj_at(x, y)
    }

    fn new_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn record(&mut self, kind: &str, text: String, x: i32, y: i32, important: bool) {
        // The chronicle notes a repeated event once per game hour, not every second.
        let recent = self.history.iter().rev().take_while(|e| self.tick.saturating_sub(e.tick) < 3600).any(|e| e.text == text);
        if recent {
            return;
        }
        self.history.push(HistoryEvent { tick: self.tick, kind: kind.into(), text, x, y, important });
    }

    /// Record a "first ever" event once.
    fn first(&mut self, key: &str, text: String, x: i32, y: i32) {
        if !self.firsts.iter().any(|k| k == key) {
            self.firsts.push(key.to_string());
            self.record("first", text, x, y, true);
        }
    }

    // ------------------------------------------------------------------ setup

    fn random_land(&self, r: &mut Rng, near: Option<(i32, i32)>, radius: i32) -> Option<(i32, i32)> {
        for _ in 0..400 {
            let (x, y) = match near {
                Some((cx, cy)) => (
                    cx + r.below((2 * radius + 1) as u64) as i32 - radius,
                    cy + r.below((2 * radius + 1) as u64) as i32 - radius,
                ),
                None => (r.below(self.w as u64) as i32, r.below(self.h as u64) as i32),
            };
            if self.walkable(x, y) {
                return Some((x, y));
            }
        }
        None
    }

    pub fn add_obj(&mut self, kind: ObjKind, x: i32, y: i32) -> u32 {
        let id = self.new_id();
        let timer = match kind {
            ObjKind::Fire => 3 * 3600,
            ObjKind::Carcass => 2 * TICKS_PER_DAY as i64,
            ObjKind::Remains => 4 * TICKS_PER_DAY as i64,
            _ => 0,
        };
        self.objs.push(Obj { id, kind, x, y, props: props_for(kind), alive: true, timer, heat_ticks: 0, cooked: false });
        id
    }

    fn populate(&mut self) {
        let mut r = Rng::new(self.seed, 0, 0, stream::WORLD_SETUP);
        // Plants and stones scattered by biome.
        for y in 0..self.h {
            for x in 0..self.w {
                let t = self.tile(x, y).clone();
                let roll = r.below(1000);
                match t.biome {
                    Biome::Forest if roll < 140 => {
                        self.add_obj(ObjKind::Tree, x, y);
                    }
                    Biome::Forest | Biome::Grass if roll < 158 && roll >= 150 => {
                        self.add_obj(ObjKind::BerryBush, x, y);
                    }
                    Biome::Grass if roll < 12 => {
                        self.add_obj(ObjKind::Tree, x, y);
                    }
                    Biome::Hills if roll < 30 => {
                        self.add_obj(if roll < 10 { ObjKind::Flint } else { ObjKind::Stone }, x, y);
                    }
                    Biome::Grass | Biome::Sand if roll >= 990 => {
                        self.add_obj(ObjKind::Stone, x, y);
                    }
                    _ => {}
                }
            }
        }
        // Two separate bands of humans, so groups can develop apart.
        let centres: Vec<(i32, i32)> = (0..2)
            .filter_map(|i| {
                let mut rr = Rng::new(self.seed, 77 + i, 0, stream::WORLD_SETUP);
                self.random_band_site(&mut rr)
            })
            .collect();
        for (bi, c) in centres.iter().enumerate() {
            for k in 0..5 {
                let age = [24, 30, 19, 8, 52][k] + r.below(6) as i64;
                let female = (k + bi) % 2 == 0;
                if let Some((x, y)) = self.random_land(&mut r, Some(*c), 3) {
                    self.spawn_human(x, y, age, female, None, None);
                }
            }
        }
        // Six grazer herds of about six animals on open grassland.
        for _herd in 0..6 {
            for _try in 0..200 {
                if let Some((x, y)) = self.random_land(&mut r, None, 0) {
                    if self.tile(x, y).biome == Biome::Grass {
                        for k in 0..6 {
                            let age = 1 + r.below(6) as i64;
                            if let Some((gx, gy)) = self.random_land(&mut r, Some((x, y)), 3) {
                                self.spawn_animal(Kind::Grazer, gx, gy, age, k % 2 == 0);
                            }
                        }
                        break;
                    }
                }
            }
        }
        // Two wolf packs of three, each in forest or hills.
        for _pack in 0..2 {
            for _try in 0..200 {
                if let Some((x, y)) = self.random_land(&mut r, None, 0) {
                    if matches!(self.tile(x, y).biome, Biome::Forest | Biome::Hills) {
                        for k in 0..3 {
                            let age = 2 + r.below(5) as i64;
                            if let Some((wx, wy)) = self.random_land(&mut r, Some((x, y)), 2) {
                                self.spawn_animal(Kind::Predator, wx, wy, age, k % 2 == 0);
                            }
                        }
                        break;
                    }
                }
            }
        }
    }

    /// A grassland spot near water with bushes around.
    fn random_band_site(&self, r: &mut Rng) -> Option<(i32, i32)> {
        let mut best: Option<((i32, i32), i32)> = None;
        for _ in 0..300 {
            let x = 8 + r.below((self.w - 16) as u64) as i32;
            let y = 8 + r.below((self.h - 16) as u64) as i32;
            if self.tile(x, y).biome != Biome::Grass || !self.walkable(x, y) {
                continue;
            }
            let mut water = 0;
            let mut bushes = 0;
            for dy in -8..=8 {
                for dx in -8..=8 {
                    if self.in_bounds(x + dx, y + dy) && self.tile(x + dx, y + dy).is_water() {
                        water += 1;
                    }
                }
            }
            for o in &self.objs {
                if o.kind == ObjKind::BerryBush && cheb(o.x, o.y, x, y) <= 10 {
                    bushes += 1;
                }
            }
            if water == 0 {
                continue;
            }
            let far = best.map_or(true, |_| true);
            let score = bushes * 10 + water.min(20);
            if far && best.map_or(true, |(_, s)| score > s) {
                if let Some(((bx, by), _)) = best {
                    if cheb(bx, by, x, y) < 30 && score <= best.unwrap().1 {
                        continue;
                    }
                }
                best = Some(((x, y), score));
            }
        }
        best.map(|(p, _)| p)
    }

    pub fn spawn_human(&mut self, x: i32, y: i32, age_years: i64, female: bool, mother: Option<u32>, father: Option<u32>) -> u32 {
        let id = self.new_id();
        let mut r = self.rng(id as u64, stream::PERSONALITY);
        let genome = match (
            mother.and_then(|m| self.creatures.iter().find(|c| c.id == m)).map(|c| c.genome),
            father.and_then(|m| self.creatures.iter().find(|c| c.id == m)).map(|c| c.genome),
        ) {
            (Some(m), Some(f)) => Genome::child(&m, &f, &mut r),
            _ => Genome::random(&mut r),
        };
        let traits = genome.traits(&mut r);
        let personality = Personality {
            openness: traits.openness,
            conscientiousness: traits.conscientiousness,
            extraversion: traits.extraversion,
            agreeableness: traits.agreeableness,
            neuroticism: traits.neuroticism,
        };
        let mind = Mind::new(id as u64, self.seed, Fx::from_int(age_years.max(0)), personality);
        let name = names::name(self.seed, id as u64, female);
        let born = self.tick as i64 - age_years.max(0) * TICKS_PER_YEAR as i64;
        let mut body = Body::child(4, fx(0.2), fx(0.15));
        body.age_s = self.tick as i64 - born;
        let c = Creature {
            id,
            kind: Kind::Human,
            name: name.clone(),
            x,
            y,
            px: x,
            py: y,
            alive: true,
            female,
            born_tick: born,
            genome,
            traits,
            health: Fx::ONE,
            body,
            mind: Some(Box::new(mind)),
            animal: AnimalBrain::default(),
            action: "waking".into(),
            last_cmd: MotorCommand::Rest,
            reaching: None,
            tokens: Vec::new(),
            touched: None,
            tasted: None,
            last_contact: None,
            last_result: MotorResult::Ok,
            loneliness: fx(0.2),
            social_comfort: Fx::ZERO,
            mother,
            father,
            partner: None,
            children: Vec::new(),
            pregnant: None,
            relations: Vec::new(),
            blessed_until: 0,
            cursed_until: 0,
            cause_of_death: None,
            died_tick: 0,
            think_interval: 1,
            skills: Vec::new(),
            last_mind_age: age_years,
            kills: 0,
        };
        self.creatures.push(c);
        // Kin know each other from birth.
        for p in [mother, father].into_iter().flatten() {
            self.bond_kin(id, p);
        }
        id
    }

    fn bond_kin(&mut self, a: u32, b: u32) {
        for (x, y) in [(a, b), (b, a)] {
            if let Some(c) = self.creatures.iter_mut().find(|c| c.id == x) {
                let r = rel_mut(&mut c.relations, y);
                r.familiarity = fx(0.8);
                r.affection = fx(0.6);
                r.trust = fx(0.7);
            }
        }
    }

    pub fn spawn_animal(&mut self, kind: Kind, x: i32, y: i32, age_years: i64, female: bool) -> u32 {
        let id = self.new_id();
        let mut r = self.rng(id as u64, stream::PERSONALITY);
        let genome = Genome::random(&mut r);
        let traits = genome.traits(&mut r);
        let born = self.tick as i64 - age_years.max(0) * TICKS_PER_YEAR as i64;
        let name = if kind == Kind::Grazer { "Grazer" } else { "Wolf" };
        let c = Creature {
            id,
            kind,
            name: format!("{name} #{id}"),
            x,
            y,
            px: x,
            py: y,
            alive: true,
            female,
            born_tick: born,
            genome,
            traits,
            health: Fx::ONE,
            body: Body::child(4, Fx::ZERO, Fx::ZERO),
            mind: None,
            animal: AnimalBrain { energy: fx(0.7), water: fx(0.7), fatigue: fx(0.2), state: "roaming", ..Default::default() },
            action: "roaming".into(),
            last_cmd: MotorCommand::Rest,
            reaching: None,
            tokens: Vec::new(),
            touched: None,
            tasted: None,
            last_contact: None,
            last_result: MotorResult::Ok,
            loneliness: Fx::ZERO,
            social_comfort: Fx::ZERO,
            mother: None,
            father: None,
            partner: None,
            children: Vec::new(),
            pregnant: None,
            relations: Vec::new(),
            blessed_until: 0,
            cursed_until: 0,
            cause_of_death: None,
            died_tick: 0,
            think_interval: 1,
            skills: Vec::new(),
            last_mind_age: 0,
            kills: 0,
        };
        self.creatures.push(c);
        id
    }

    pub fn age_years(&self, c: &Creature) -> Fx {
        Fx::ratio(self.tick as i64 - c.born_tick, TICKS_PER_YEAR as i64)
    }

    // ------------------------------------------------------------------ climate

    pub fn time_of_day(&self) -> u64 {
        self.tick % TICKS_PER_DAY
    }

    /// Daylight 0..1.
    pub fn light(&self) -> Fx {
        let t = self.time_of_day() as i64;
        let h = Fx::ratio(t, 3600);
        let v = if h < fx(5.0) || h > fx(21.0) {
            fx(0.08)
        } else if h < fx(7.0) {
            fx(0.08) + (h - fx(5.0)) * fx(0.42)
        } else if h > fx(19.0) {
            fx(0.92) - (h - fx(19.0)) * fx(0.42)
        } else {
            fx(0.92)
        };
        let cloud = match self.weather {
            Weather::Clear => Fx::ONE,
            Weather::Cloudy => fx(0.8),
            Weather::Rain => fx(0.65),
            Weather::Storm => fx(0.5),
        };
        v * cloud
    }

    /// Season 0..3 (spring, summer, autumn, winter); one year = DAYS_PER_YEAR days.
    pub fn season(&self) -> u64 {
        (self.tick % TICKS_PER_YEAR) * 4 / TICKS_PER_YEAR
    }

    pub fn year(&self) -> u64 {
        self.tick / TICKS_PER_YEAR
    }

    /// Air temperature at a tile (°C).
    pub fn temperature(&self, x: i32, y: i32) -> Fx {
        let season = [fx(15.0), fx(23.0), fx(14.0), fx(4.0)][self.season() as usize];
        let h = Fx::ratio(self.time_of_day() as i64, 3600);
        let diurnal = if h > fx(6.0) && h < fx(20.0) { fx(4.0) } else { fx(-4.0) };
        let elev = Fx::from_int(self.tile(x, y).elev.max(400) as i64 - 400) / fx(60.0);
        let weather = match self.weather {
            Weather::Rain => fx(-3.0),
            Weather::Storm => fx(-5.0),
            _ => Fx::ZERO,
        };
        season + diurnal - elev + weather + self.god_temp
    }

    fn update_weather(&mut self) {
        if self.tick < self.weather_until {
            return;
        }
        let mut r = self.rng(0, stream::PHYSICS ^ 0xA1);
        let roll = r.below(100);
        let forced_rain = self.tick < self.rain_until;
        self.weather = if forced_rain {
            Weather::Rain
        } else if roll < 55 {
            Weather::Clear
        } else if roll < 78 {
            Weather::Cloudy
        } else if roll < 95 {
            Weather::Rain
        } else {
            Weather::Storm
        };
        self.weather_until = self.tick + 3600 + r.below(4 * 3600);
        if self.weather == Weather::Storm {
            self.record("weather", "A storm swept over the land.".into(), self.w / 2, self.h / 2, false);
        }
    }

    // ------------------------------------------------------------------ main loop

    /// Advance one game second.
    pub fn step(&mut self) {
        self.tick += 1;
        if self.tick % 60 == 0 {
            self.update_weather();
        }
        if self.tick % 600 == 0 {
            self.plants_and_fire_slow();
        }
        self.fires_fast();
        let n = self.creatures.len();
        for i in 0..n {
            if !self.creatures[i].alive {
                continue;
            }
            self.creatures[i].px = self.creatures[i].x;
            self.creatures[i].py = self.creatures[i].y;
            match self.creatures[i].kind {
                Kind::Human => self.human_tick(i),
                _ => self.animal_tick(i),
            }
        }
        if self.tick % 30 == 0 {
            self.social_tick();
        }
        if self.tick % 3600 == 0 {
            self.life_tick();
        }
        if self.tick % 600 == 0 {
            // Dead animals are dropped after a day; dead humans stay for family history.
            let t = self.tick;
            self.creatures.retain(|c| c.alive || c.kind == Kind::Human || t - c.died_tick < TICKS_PER_DAY);
        }
    }

    // ------------------------------------------------------------------ plants & fire

    fn plants_and_fire_slow(&mut self) {
        let raining = matches!(self.weather, Weather::Rain | Weather::Storm);
        let season = self.season();
        let growth = [6, 8, 4, 1][season as usize] + if raining { 3 } else { 0 };
        for t in self.tiles.iter_mut() {
            if t.burnt > 0 {
                t.burnt = t.burnt.saturating_sub(600);
                continue;
            }
            if t.fertility > 0 {
                let cap = t.fertility;
                t.grass = (t.grass + growth * t.fertility / 300).min(cap);
            }
        }
        let mut r = self.rng(0, stream::PHYSICS ^ 0xB2);
        let mut lightning: Option<(i32, i32)> = None;
        if self.weather == Weather::Storm && r.below(6) == 0 {
            let x = r.below(self.w as u64) as i32;
            let y = r.below(self.h as u64) as i32;
            lightning = Some((x, y));
        }
        let mut seeds: Vec<(i32, i32)> = Vec::new();
        for o in self.objs.iter_mut().filter(|o| o.alive) {
            match o.kind {
                ObjKind::BerryBush => {
                    // Fruiting: fastest in late summer/autumn, slow in winter.
                    o.timer += 600;
                    let every = [2400, 1800, 1500, 7200][season as usize];
                    if o.timer > every && o.props.portions < o.props.max_portions {
                        o.props.portions += 1;
                        o.timer = 0;
                    }
                    if o.props.portions == o.props.max_portions && season != 3 && r.below(400) == 0 {
                        seeds.push((o.x, o.y));
                    }
                }
                ObjKind::Carcass | ObjKind::Remains => {
                    o.timer -= 600;
                    if o.timer <= 0 {
                        o.alive = false;
                    }
                }
                _ => {}
            }
        }
        if let Some((x, y)) = lightning {
            self.strike_lightning(x, y, false);
        }
        // Plants spread: a laden bush drops seeds that may take root on fertile ground
        // nearby, unless the spot is already crowded.
        let bushes = self.objs.iter().filter(|o| o.alive && o.kind == ObjKind::BerryBush).count();
        for (sx, sy) in seeds {
            if bushes > 600 {
                break;
            }
            let nx = sx + r.below(9) as i32 - 4;
            let ny = sy + r.below(9) as i32 - 4;
            if !self.in_bounds(nx, ny) || !matches!(self.tile(nx, ny).biome, Biome::Grass | Biome::Forest) {
                continue;
            }
            let crowd = self.objs.iter().filter(|o| o.alive && o.kind == ObjKind::BerryBush && cheb(o.x, o.y, nx, ny) <= 3).count();
            if crowd < 3 {
                let id = self.add_obj(ObjKind::BerryBush, nx, ny);
                if let Some(o) = self.objs.iter_mut().find(|o| o.id == id) {
                    o.props.portions = 0;
                }
            }
        }
        self.objs.retain(|o| o.alive || o.kind == ObjKind::BerryBush);
        // Berry bushes re-sprout when emptied.
        for o in self.objs.iter_mut() {
            if o.kind == ObjKind::BerryBush && o.props.portions == 0 {
                o.props.portions = 0;
            }
        }
    }

    fn fires_fast(&mut self) {
        if self.tick % 10 != 0 {
            return;
        }
        let raining = matches!(self.weather, Weather::Rain | Weather::Storm);
        let mut spread: Vec<(i32, i32)> = Vec::new();
        let mut r = self.rng(0, stream::PHYSICS ^ 0xF1);
        let fires: Vec<(usize, i32, i32)> =
            self.objs.iter().enumerate().filter(|(_, o)| o.alive && o.kind == ObjKind::Fire).map(|(i, o)| (i, o.x, o.y)).collect();
        for (i, x, y) in &fires {
            let o = &mut self.objs[*i];
            o.timer -= if raining { 60 } else { 10 };
            if o.timer <= 0 {
                o.alive = false;
                continue;
            }
            if !raining && r.below(1000) < 4 {
                let (dx, dy) = DIRS[r.below(8) as usize];
                spread.push((x + dx, y + dy));
            }
        }
        for (x, y) in spread {
            if let Some(tree) = self.objs.iter_mut().find(|o| o.alive && o.kind == ObjKind::Tree && o.x == x && o.y == y) {
                tree.alive = false;
                self.add_obj(ObjKind::Fire, x, y);
                if self.in_bounds(x, y) {
                    self.tile_mut(x, y).burnt = 6 * 3600;
                    self.tile_mut(x, y).grass = 0;
                }
            }
        }
        // Cooking: carcasses next to fire for a while become cooked.
        let fire_pos: Vec<(i32, i32)> = self.objs.iter().filter(|o| o.alive && o.kind == ObjKind::Fire).map(|o| (o.x, o.y)).collect();
        let mut cooked_now: Vec<(i32, i32)> = Vec::new();
        for o in self.objs.iter_mut().filter(|o| o.alive && o.kind == ObjKind::Carcass && !o.cooked) {
            if fire_pos.iter().any(|&(fx_, fy)| cheb(fx_, fy, o.x, o.y) <= 1) {
                o.heat_ticks += 10;
                if o.heat_ticks > 600 {
                    o.cooked = true;
                    o.props.sugar = fx(0.5);
                    o.props.bitter = Fx::ZERO;
                    o.props.kcal_per_portion = fx(60.0);
                    o.props.hue_deg = fx(25.0);
                    o.props.saturation = fx(0.45);
                    cooked_now.push((o.x, o.y));
                }
            }
        }
        for (x, y) in cooked_now {
            self.first("cooked", "Fire cooked meat for the first time (no one planned it).".into(), x, y);
        }
    }

    pub fn strike_lightning(&mut self, x: i32, y: i32, divine: bool) {
        if !self.in_bounds(x, y) {
            return;
        }
        let mut hurt = Vec::new();
        for c in self.creatures.iter_mut().filter(|c| c.alive) {
            let d = cheb(c.x, c.y, x, y);
            if d <= 1 {
                if c.kind == Kind::Human {
                    c.body.wound(if d == 0 { fx(0.9) } else { fx(0.4) });
                } else {
                    c.health -= if d == 0 { fx(0.9) } else { fx(0.4) };
                }
                hurt.push(c.name.clone());
            }
        }
        let mut ignited = false;
        if let Some(tree) = self.objs.iter_mut().find(|o| o.alive && o.kind == ObjKind::Tree && cheb(o.x, o.y, x, y) <= 1) {
            tree.alive = false;
            let (tx, ty) = (tree.x, tree.y);
            self.add_obj(ObjKind::Fire, tx, ty);
            ignited = true;
        } else if self.walkable(x, y) && !self.tile(x, y).is_water() && self.tile(x, y).grass > 200 {
            self.add_obj(ObjKind::Fire, x, y);
            ignited = true;
        }
        let who = if divine { "God's lightning" } else { "Lightning" };
        let mut text = format!("{who} struck the ground");
        if ignited {
            text.push_str(" and started a fire");
        }
        if !hurt.is_empty() {
            text.push_str(&format!("; hit {}", hurt.join(", ")));
        }
        text.push('.');
        self.record("lightning", text, x, y, divine || !hurt.is_empty());
        if ignited {
            self.first("fire", "Fire appeared in the world for the first time.".into(), x, y);
        }
    }

    // ------------------------------------------------------------------ humans

    fn token(&self, agent: u32, t: Target) -> Token {
        let k = match t {
            Target::Obj(id) => id as u64,
            Target::Creature(id) => (1u64 << 40) | id as u64,
            Target::Water(x, y) => (2u64 << 40) | ((x as u64) << 20) | y as u64,
        };
        Token(key4(self.seed ^ 0x70C3, k, agent as u64, stream::TOKEN_SALT) >> 1)
    }

    fn target_pos(&self, t: Target) -> Option<(i32, i32)> {
        match t {
            Target::Obj(id) => self.objs.iter().find(|o| o.id == id && o.alive).map(|o| (o.x, o.y)),
            Target::Creature(id) => self.creatures.iter().find(|c| c.id == id && c.alive).map(|c| (c.x, c.y)),
            Target::Water(x, y) => Some((x, y)),
        }
    }

    /// Radiant warmth felt at (x, y) from object o (same physics as the P1 room world).
    fn radiant(o: &Obj, x: i32, y: i32) -> Fx {
        let excess = o.props.temp_c - fx(30.0);
        if !excess.is_positive() {
            return Fx::ZERO;
        }
        let d2 = Fx::from_int(((o.x - x) * (o.x - x) + (o.y - y) * (o.y - y)) as i64);
        let strength = excess / fx(700.0) * o.props.size * fx(2.5);
        (strength / (d2 + fx(0.5))).clamp01()
    }

    fn radiant_close(o: &Obj) -> Fx {
        let excess = o.props.temp_c - fx(30.0);
        if !excess.is_positive() {
            return Fx::ZERO;
        }
        (excess / fx(700.0) * o.props.size * fx(2.5) / fx(0.55)).clamp01()
    }

    /// Builds the sensory frame for human i: only what its senses can report.
    pub fn sense_human(&mut self, i: usize) -> SensoryFrame {
        let c = &self.creatures[i];
        let (cx, cy, id) = (c.x, c.y, c.id);
        let mut rng = Rng::new(self.seed, id as u64, self.tick, stream::SENSOR_NOISE);
        let light = self.light();
        let mut cands: Vec<(i32, Target)> = Vec::new();
        for o in self.objs.iter().filter(|o| o.alive) {
            let d = cheb(o.x, o.y, cx, cy);
            if d <= VISION {
                cands.push((d, Target::Obj(o.id)));
            }
        }
        for o in self.creatures.iter().filter(|o| o.alive && o.id != id) {
            let d = cheb(o.x, o.y, cx, cy);
            if d <= VISION && !(o.kind == Kind::Human && self.is_carried(o)) {
                cands.push((d, Target::Creature(o.id)));
            }
        }
        // Nearest water tiles (at most two, in different directions).
        let mut waters: Vec<(i32, i32, i32)> = Vec::new();
        for dy in -VISION..=VISION {
            for dx in -VISION..=VISION {
                let (x, y) = (cx + dx, cy + dy);
                if self.in_bounds(x, y) && self.tile(x, y).is_water() {
                    waters.push((cheb(x, y, cx, cy), x, y));
                }
            }
        }
        waters.sort();
        let mut chosen_w: Vec<(i32, i32, i32)> = Vec::new();
        for wv in waters {
            if chosen_w.iter().all(|c| cheb(c.1, c.2, wv.1, wv.2) > 6) {
                chosen_w.push(wv);
            }
            if chosen_w.len() >= 2 {
                break;
            }
        }
        // Perception budget per category: a few nearest creatures and objects,
        // plus open water, which is visible from afar.
        cands.sort_by_key(|(d, t)| (*d, format!("{t:?}")));
        let mut picked: Vec<(i32, Target)> = Vec::new();
        let (mut n_obj, mut n_cre) = (0, 0);
        for (d, t) in cands {
            match t {
                Target::Obj(_) if n_obj < 8 => {
                    n_obj += 1;
                    picked.push((d, t));
                }
                Target::Creature(_) if n_cre < 4 => {
                    n_cre += 1;
                    picked.push((d, t));
                }
                _ => {}
            }
        }
        for (d, x, y) in chosen_w {
            picked.push((d, Target::Water(x, y)));
        }
        let cands = picked;
        let c = &self.creatures[i];
        let mut percepts = Vec::new();
        let mut tokens = Vec::new();
        let reach_t = c.reaching.map(|(t, _)| t);
        for (d, t) in cands {
            let (visual, warmth, (tx, ty)) = match t {
                Target::Obj(oid) => {
                    let o = self.objs.iter().find(|o| o.id == oid).expect("alive");
                    let p = &o.props;
                    let osc = rng.unit() - Fx::HALF;
                    // A bush shows its berries only while it has them; stripped, it is green leaves.
                    let (hue, sat) = if o.kind == ObjKind::BerryBush {
                        let full = Fx::ratio(p.portions.max(0) as i64, p.max_portions.max(1) as i64);
                        let berry = (full * fx(3.0)).min(Fx::ONE);
                        (fx(110.0) + (p.hue_deg - fx(110.0)) * berry, fx(0.45) + (p.saturation - fx(0.45)) * berry)
                    } else {
                        (p.hue_deg, p.saturation)
                    };
                    let v = Visual {
                        hue_deg: hue,
                        saturation: sat,
                        brightness: (p.emit * (Fx::ONE + p.flicker * osc) + p.reflect * light * fx(0.6)).clamp01(),
                        flicker: (p.flicker * (fx(0.85) + rng.unit() * fx(0.3))).clamp01(),
                        size: p.size,
                        roundness: p.roundness,
                        texture: p.roughness,
                        gloss: p.gloss,
                    };
                    let mut w = Self::radiant(o, cx, cy);
                    if reach_t == Some(t) {
                        w = w.max(Self::radiant_close(o));
                    }
                    (v, w, (o.x, o.y))
                }
                Target::Creature(cid) => {
                    let o = self.creatures.iter().find(|o| o.id == cid).expect("alive");
                    let mut v = creature_visual(o);
                    v.brightness = (v.brightness * (fx(0.4) + light * fx(0.7))).clamp01();
                    (v, Fx::ZERO, (o.x, o.y))
                }
                Target::Water(x, y) => {
                    let mut v = water_visual();
                    v.brightness = (v.brightness * (fx(0.4) + light * fx(0.7))).clamp01();
                    (v, Fx::ZERO, (x, y))
                }
            };
            let noise = |r: &mut Rng, s: f64| r.gaussish() * fx(s);
            let visual = Visual {
                hue_deg: wrap(visual.hue_deg + noise(&mut rng, 4.0)),
                saturation: (visual.saturation + noise(&mut rng, 0.03)).clamp01(),
                brightness: (visual.brightness + noise(&mut rng, 0.02)).clamp01(),
                size: (visual.size + noise(&mut rng, 0.02)).clamp01(),
                roundness: (visual.roundness + noise(&mut rng, 0.03)).clamp01(),
                texture: (visual.texture + noise(&mut rng, 0.03)).clamp01(),
                gloss: (visual.gloss + noise(&mut rng, 0.03)).clamp01(),
                flicker: visual.flicker,
            };
            let token = self.token(id, t);
            tokens.push((token, t));
            percepts.push(Percept {
                token,
                dx: tx - cx,
                dy: ty - cy,
                dist: d,
                visual,
                felt_warmth: (warmth + noise(&mut rng, 0.01)).clamp01(),
                touch: c.touched.and_then(|(tt, f)| (tt == t).then_some(f)),
                taste: c.tasted.and_then(|(tt, f)| (tt == t).then_some(f)),
            });
        }
        percepts.sort_by_key(|p| p.token);
        let s = c.body.signals;
        let ground = match self.tile(cx, cy).biome {
            Biome::Forest => fx(0.7),
            Biome::Sand => fx(0.2),
            Biome::Hills => fx(0.55),
            _ => fx(0.4),
        };
        let frame = SensoryFrame {
            tick: self.tick,
            asleep: c.body.asleep,
            percepts,
            body: Interoception {
                hunger: s.hunger,
                thirst: s.thirst,
                pain: s.pain,
                acute_pain: s.acute_pain,
                pain_hand: s.pain_hand,
                pain_mouth: s.pain_mouth,
                fullness: s.fullness,
                gut_nutrient: s.gut_nutrient,
                gut_fluid: s.gut_fluid,
                fatigue: s.fatigue,
                body_heat: s.body_heat,
                loneliness: c.loneliness,
                social_comfort: c.social_comfort,
                reflex_active: s.reflex_hand || s.reflex_mouth,
            },
            ambient: Ambient {
                light: (light * fx(5.0)).floor_int().max(0).min(5).pipe(|v| Fx::ratio(v, 5)),
                ground,
                enclosure: if self.tile(cx, cy).biome == Biome::Forest { fx(0.6) } else { fx(0.1) },
                sound: if matches!(self.weather, Weather::Rain | Weather::Storm) { fx(0.6) } else { fx(0.2) },
            },
            last_result: c.last_result,
        };
        self.creatures[i].tokens = tokens;
        frame
    }

    fn is_carried(&self, c: &Creature) -> bool {
        c.kind == Kind::Human
            && self.age_years(c) < fx(2.0)
            && c.mother.and_then(|m| self.creatures.iter().find(|x| x.id == m && x.alive)).is_some()
    }

    fn step_toward(&mut self, i: usize, tx: i32, ty: i32, toward: bool) -> bool {
        let (ax, ay) = (self.creatures[i].x, self.creatures[i].y);
        let cur = cheb(ax, ay, tx, ty);
        let mut best: Option<(i64, usize)> = None;
        for (di, (dx, dy)) in DIRS.iter().enumerate() {
            let (nx, ny) = (ax + dx, ay + dy);
            if !self.walkable(nx, ny) {
                continue;
            }
            let d = cheb(nx, ny, tx, ty);
            let e2 = ((nx - tx) * (nx - tx) + (ny - ty) * (ny - ty)) as i64;
            let score = if toward {
                if d > cur || (d == cur && e2 >= ((ax - tx).pow(2) + (ay - ty).pow(2)) as i64) {
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
            self.creatures[i].x += DIRS[di].0;
            self.creatures[i].y += DIRS[di].1;
            true
        } else {
            false
        }
    }

    fn human_tick(&mut self, i: usize) {
        // Carried infants move with their mother and are nursed.
        let carried = self.is_carried(&self.creatures[i]);
        if carried {
            let m = self.creatures[i].mother.unwrap();
            if let Some(mi) = self.creatures.iter().position(|c| c.id == m) {
                let (mx, my) = (self.creatures[mi].x, self.creatures[mi].y);
                self.creatures[i].x = mx;
                self.creatures[i].y = my;
                if self.tick % 1800 == 0 && self.creatures[i].body.signals.hunger > fx(0.3) {
                    self.creatures[i].body.ingest(fx(45.0), fx(40.0));
                    self.creatures[mi].body.energy -= fx(45.0);
                }
            }
        }
        // Environment acts on the body.
        let (x, y) = (self.creatures[i].x, self.creatures[i].y);
        let ambient = self.temperature(x, y);
        let mut rad = Fx::ZERO;
        for o in self.objs.iter().filter(|o| o.alive && o.kind == ObjKind::Fire) {
            if cheb(o.x, o.y, x, y) <= 4 {
                rad += Self::radiant(o, x, y);
            }
        }
        // What the skin effectively feels: air temperature, plus metabolic heat (more
        // when active), plus shelter under forest canopy, plus body heat shared with
        // humans lying or standing right next to you. Physics only: whether to stay
        // near others or under trees is the mind's own choice.
        let metabolic = if self.creatures[i].body.asleep { fx(3.0) } else { fx(6.0) };
        let canopy = if self.tile(x, y).biome == terrain::Biome::Forest { fx(3.0) } else { Fx::ZERO };
        let huddle = self
            .creatures
            .iter()
            .filter(|o| o.alive && o.kind == Kind::Human && o.id != self.creatures[i].id && cheb(o.x, o.y, x, y) <= 1)
            .take(3)
            .count() as i64;
        let felt_air = ambient + metabolic + canopy + fx(1.5) * Fx::from_int(huddle);
        {
            let c = &mut self.creatures[i];
            c.body.ambient_c = felt_air;
            c.body.add_radiant(rad);
            c.social_comfort = c.social_comfort * fx(0.9);
            if let Some((Target::Obj(oid), site)) = c.reaching {
                if let Some(o) = self.objs.iter().find(|o| o.id == oid && o.alive) {
                    c.body.add_radiant_site(site, Self::radiant_close(o));
                }
            }
        }
        // Thinking (cognitive LOD: unselected humans think every `human_interval` ticks).
        let selected = self.selected == Some(self.creatures[i].id);
        let interval = if selected { 1 } else { self.human_interval.max(1) };
        self.creatures[i].think_interval = interval;
        let think = (self.tick + self.creatures[i].id as u64) % interval as u64 == 0;
        let cmd = if think {
            let t0 = std::time::Instant::now();
            let frame = self.sense_human(i);
            let t1 = std::time::Instant::now();
            let mut mind = self.creatures[i].mind.take().expect("human has a mind");
            mind.trace_enabled = selected;
            let cmd = mind.step(&frame);
            self.prof_sense_ns += (t1 - t0).as_nanos() as u64;
            self.prof_mind_ns += t1.elapsed().as_nanos() as u64;
            self.creatures[i].mind = Some(mind);
            self.mind_steps += 1;
            cmd
        } else {
            self.creatures[i].last_cmd
        };
        self.creatures[i].touched = None;
        self.creatures[i].tasted = None;
        let res = if carried { MotorResult::Overridden } else { self.execute_human(i, cmd) };
        self.creatures[i].last_result = res;
        self.creatures[i].last_cmd = cmd;
        self.creatures[i].body.step();
        self.creatures[i].action = self.describe_action(i, cmd, res);
    }

    fn describe_action(&self, i: usize, cmd: MotorCommand, res: MotorResult) -> String {
        let c = &self.creatures[i];
        if c.body.asleep {
            return "sleeping".into();
        }
        if self.is_carried(c) {
            return "carried by mother".into();
        }
        if res == MotorResult::Overridden {
            return "recoiling in pain".into();
        }
        let tgt = |t: Token| -> String {
            match c.tokens.iter().find(|(tk, _)| *tk == t).map(|x| x.1) {
                Some(Target::Obj(id)) => self.objs.iter().find(|o| o.id == id).map(|o| obj_name(o)).unwrap_or("something".into()),
                Some(Target::Creature(id)) => self.creatures.iter().find(|o| o.id == id).map(|o| o.name.clone()).unwrap_or("someone".into()),
                Some(Target::Water(..)) => "water".into(),
                None => "something".into(),
            }
        };
        match cmd {
            MotorCommand::Rest => "resting".into(),
            MotorCommand::Wander { .. } => "wandering".into(),
            MotorCommand::Approach { target } => format!("approaching {}", tgt(target)),
            MotorCommand::Inspect { target } => format!("watching {}", tgt(target)),
            MotorCommand::Touch { target } => {
                if c.reaching.is_some() { format!("reaching for {}", tgt(target)) } else { format!("touching {}", tgt(target)) }
            }
            MotorCommand::Mouth { target } => {
                let n = tgt(target);
                if n == "water" { "drinking".into() } else if c.reaching.is_some() { format!("bringing {n} to mouth") } else { format!("tasting {n}") }
            }
            MotorCommand::Withdraw { target } => format!("backing away from {}", tgt(target)),
        }
    }

    fn attachment_figure(&self, i: usize) -> Option<(i32, i32)> {
        let c = &self.creatures[i];
        let young = self.age_years(c) < fx(12.0);
        let who = if young { c.mother.or(c.partner) } else { c.partner };
        who.and_then(|id| self.creatures.iter().find(|o| o.id == id && o.alive)).map(|o| (o.x, o.y))
    }

    fn execute_human(&mut self, i: usize, cmd: MotorCommand) -> MotorResult {
        if self.creatures[i].body.asleep {
            self.creatures[i].reaching = None;
            return MotorResult::Overridden;
        }
        let sig = self.creatures[i].body.signals;
        if sig.reflex_hand || sig.reflex_mouth {
            self.creatures[i].reaching = None;
            if let Some((tx, ty)) = self.creatures[i].last_contact.and_then(|t| self.target_pos(t)) {
                self.step_toward(i, tx, ty, false);
            }
            return MotorResult::Overridden;
        }
        let prev_reach = self.creatures[i].reaching.take();
        match cmd {
            MotorCommand::Rest => MotorResult::Ok,
            MotorCommand::Wander { dir } => {
                // Innate attachment prior (species-level, like the grazers' fear of wolves):
                // undirected wandering drifts back toward a distant partner, or a young
                // child's mother. Targeted actions are untouched; they are the mind's.
                if let Some((ax, ay)) = self.attachment_figure(i) {
                    let (x, y) = (self.creatures[i].x, self.creatures[i].y);
                    let mut r = self.rng(self.creatures[i].id as u64, stream::PHYSICS ^ 0xA77A);
                    if cheb(ax, ay, x, y) > 5 && r.chance(fx(0.75)) && self.step_toward(i, ax, ay, true) {
                        return MotorResult::Ok;
                    }
                }
                let (dx, dy) = DIRS[(dir % 8) as usize];
                let (nx, ny) = (self.creatures[i].x + dx, self.creatures[i].y + dy);
                if self.walkable(nx, ny) {
                    self.creatures[i].x = nx;
                    self.creatures[i].y = ny;
                    MotorResult::Ok
                } else {
                    MotorResult::Failed
                }
            }
            _ => {
                let token = cmd.target().expect("targeted");
                let Some(t) = self.creatures[i].tokens.iter().find(|(tk, _)| *tk == token).map(|x| x.1) else {
                    return MotorResult::Failed;
                };
                let Some((tx, ty)) = self.target_pos(t) else {
                    return MotorResult::Failed;
                };
                let d = cheb(self.creatures[i].x, self.creatures[i].y, tx, ty);
                match cmd {
                    MotorCommand::Approach { .. } => {
                        if d > 1 && !self.step_toward(i, tx, ty, true) {
                            return MotorResult::Failed;
                        }
                        MotorResult::Ok
                    }
                    MotorCommand::Inspect { .. } => {
                        if d > 2 {
                            self.step_toward(i, tx, ty, true);
                        } else if d < 2 {
                            self.step_toward(i, tx, ty, false);
                        }
                        MotorResult::Ok
                    }
                    MotorCommand::Withdraw { .. } => {
                        if self.step_toward(i, tx, ty, false) { MotorResult::Ok } else { MotorResult::Failed }
                    }
                    MotorCommand::Touch { .. } | MotorCommand::Mouth { .. } => {
                        let site = if matches!(cmd, MotorCommand::Touch { .. }) { Site::Hand } else { Site::Mouth };
                        if d > 1 {
                            return if self.step_toward(i, tx, ty, true) { MotorResult::Ok } else { MotorResult::Failed };
                        }
                        if prev_reach == Some((t, site)) {
                            self.contact(i, t, site);
                        } else {
                            self.creatures[i].reaching = Some((t, site));
                            self.creatures[i].last_contact = Some(t);
                        }
                        MotorResult::Ok
                    }
                    _ => MotorResult::Ok,
                }
            }
        }
    }

    /// Physical contact between human i and a target (hand or mouth).
    fn contact(&mut self, i: usize, t: Target, site: Site) {
        self.creatures[i].last_contact = Some(t);
        let hid = self.creatures[i].id;
        match t {
            Target::Water(_, _) => {
                let body = &mut self.creatures[i].body;
                body.contact(site, fx(15.0), fx(0.8));
                if site == Site::Mouth {
                    body.ingest(Fx::ZERO, fx(50.0));
                    self.creatures[i].tasted = Some((t, Taste { sweet: Fx::ZERO, bitter: Fx::ZERO, liquid: Fx::ONE, swallowed: Fx::ONE }));
                } else {
                    self.creatures[i].touched = Some((t, Touch { thermal: fx(-0.45), firmness: Fx::ZERO, wetness: Fx::ONE }));
                }
            }
            Target::Obj(oid) => {
                let Some(oi) = self.objs.iter().position(|o| o.id == oid && o.alive) else { return };
                let p = self.objs[oi].props.clone();
                let kind = self.objs[oi].kind;
                let (ox, oy) = (self.objs[oi].x, self.objs[oi].y);
                self.creatures[i].body.contact(site, p.temp_c, p.conduct);
                if kind == ObjKind::Fire {
                    let name = self.creatures[i].name.clone();
                    self.first(&format!("burn-{}", hid), format!("{name} was burned by fire."), ox, oy);
                }
                if site == Site::Hand {
                    self.creatures[i].touched = Some((t, Touch {
                        thermal: ((p.temp_c - fx(33.0)) / fx(40.0)).clamp(Fx::NEG_ONE, Fx::ONE),
                        firmness: p.hardness,
                        wetness: p.wet,
                    }));
                    if kind == ObjKind::Flake {
                        // Sharp edge: a small cut.
                        self.creatures[i].body.cut_hand(fx(0.06));
                    }
                    if kind == ObjKind::Flint {
                        self.knap(i, oi);
                    }
                } else {
                    let swallowable = p.liquid || (p.hardness < fx(0.5) && p.portions != 0 && p.size < fx(0.5));
                    if swallowable {
                        self.creatures[i].body.ingest(p.kcal_per_portion, p.water_per_portion);
                        let o = &mut self.objs[oi];
                        if o.props.portions > 0 {
                            o.props.portions -= 1;
                            if o.props.portions == 0 && o.kind != ObjKind::BerryBush {
                                o.alive = false;
                            }
                        }
                        let name = self.creatures[i].name.clone();
                        match kind {
                            ObjKind::BerryBush => self.first("eat-berries", format!("{name} discovered that berries can be eaten."), ox, oy),
                            ObjKind::Carcass => {
                                if self.objs[oi].cooked {
                                    self.first("eat-cooked", format!("{name} ate cooked meat."), ox, oy);
                                } else {
                                    self.first("eat-meat", format!("{name} ate meat from a carcass."), ox, oy);
                                }
                            }
                            _ => {}
                        }
                    }
                    self.creatures[i].tasted = Some((t, Taste {
                        sweet: if swallowable { p.sugar } else { p.sugar * fx(0.3) },
                        bitter: p.bitter,
                        liquid: if p.liquid { Fx::ONE } else { p.wet * fx(0.5) },
                        swallowed: if swallowable { Fx::ONE } else { Fx::ZERO },
                    }));
                }
            }
            Target::Creature(cid) => {
                let Some(ci) = self.creatures.iter().position(|c| c.id == cid && c.alive) else { return };
                let kind = self.creatures[ci].kind;
                self.creatures[i].touched = Some((t, Touch { thermal: fx(0.1), firmness: fx(0.3), wetness: Fx::ZERO }));
                match kind {
                    Kind::Human => self.social_contact(i, ci),
                    Kind::Grazer => {
                        let hp = (self.creatures[i].x, self.creatures[i].y);
                        let a = &mut self.creatures[ci].animal;
                        a.fear_of_humans = (a.fear_of_humans + fx(0.2)).min(Fx::ONE);
                        a.flee_from = Some(hp);
                    }
                    Kind::Predator => {
                        // Touching a wolf provokes a bite.
                        self.creatures[i].body.wound(fx(0.2));
                        let name = self.creatures[i].name.clone();
                        let (wx, wy) = (self.creatures[ci].x, self.creatures[ci].y);
                        self.record("attack", format!("A wolf bit {name} who reached out to it."), wx, wy, false);
                    }
                }
            }
        }
    }

    /// Flint struck while another stone is at hand fractures into a sharp flake.
    fn knap(&mut self, i: usize, oi: usize) {
        let (ox, oy) = (self.objs[oi].x, self.objs[oi].y);
        let has_hammer = self.objs.iter().any(|o| {
            o.alive && o.id != self.objs[oi].id && matches!(o.kind, ObjKind::Stone | ObjKind::Flint) && cheb(o.x, o.y, ox, oy) <= 2
        });
        let mut r = self.rng(self.creatures[i].id as u64, stream::PHYSICS ^ 0x4A);
        let chance = fx(0.05) + self.creatures[i].traits.dexterity * fx(0.1) + if has_hammer { fx(0.2) } else { Fx::ZERO };
        if r.chance(chance) {
            for (dx, dy) in DIRS {
                let (nx, ny) = (ox + dx, oy + dy);
                if self.walkable(nx, ny) {
                    self.add_obj(ObjKind::Flake, nx, ny);
                    let name = self.creatures[i].name.clone();
                    self.first("flake", format!("{name} struck flint and it split into a sharp-edged flake."), nx, ny);
                    let c = &mut self.creatures[i];
                    match c.skills.iter_mut().find(|s| s.0 == "stone working") {
                        Some(s) => s.1 = (s.1 + fx(0.05)).min(Fx::ONE),
                        None => c.skills.push(("stone working".into(), fx(0.05))),
                    }
                    break;
                }
            }
        }
    }

    /// Two humans in touch: comfort, relationship change and sharing of what they know.
    fn social_contact(&mut self, i: usize, j: usize) {
        let (ai, bj) = (self.creatures[i].id, self.creatures[j].id);
        self.creatures[i].social_comfort = Fx::ONE;
        self.creatures[j].social_comfort = Fx::ONE;
        // Relationship change and telling at most once per 10 minutes per pair.
        let last = rel(&self.creatures[i].relations, bj).map(|r| r.last_touch).unwrap_or(0);
        if last != 0 && self.tick < last + 600 {
            return;
        }
        rel_mut(&mut self.creatures[i].relations, bj).last_touch = self.tick;
        for (a, b) in [(i, bj), (j, ai)] {
            let r = rel_mut(&mut self.creatures[a].relations, b);
            r.affection = (r.affection + fx(0.03)).min(Fx::ONE);
            r.trust = (r.trust + fx(0.02)).min(Fx::ONE);
            r.familiarity = (r.familiarity + fx(0.03)).min(Fx::ONE);
            r.last_seen = self.tick;
        }
        // Telling: the toucher shares their strongest lesson; the listener weighs it by trust.
        let told = self.creatures[i].mind.as_ref().and_then(|m| m.tellable_belief());
        if let Some((cue, outcome, strength)) = told {
            let trust = fx(0.3) + rel(&self.creatures[j].relations, ai).map(|r| r.trust).unwrap_or(Fx::ZERO) * fx(0.7);
            let tick = self.tick;
            let accepted = self.creatures[j].mind.as_mut().map(|m| m.receive_told(cue, outcome, strength, trust, ai, tick)).unwrap_or(false);
            if accepted {
                let what = self.creatures[i].mind.as_ref().map(|m| m.cue_name(cue as usize)).unwrap_or_default();
                let out = ["pain", "food", "drink", "comfort", "warmth"][outcome as usize % 5];
                let (na, nb) = (self.creatures[i].name.clone(), self.creatures[j].name.clone());
                let (x, y) = (self.creatures[i].x, self.creatures[i].y);
                self.first("told", format!("{na} passed knowledge to {nb} for the first time: things that are '{what}' mean {out}."), x, y);
                self.record("knowledge", format!("{na} told {nb}: '{what}' means {out}."), x, y, false);
            }
        }
    }

    // ------------------------------------------------------------------ social & life

    fn social_tick(&mut self) {
        let humans: Vec<(usize, u32, i32, i32)> = self
            .creatures
            .iter()
            .enumerate()
            .filter(|(_, c)| c.alive && c.kind == Kind::Human)
            .map(|(i, c)| (i, c.id, c.x, c.y))
            .collect();
        for &(i, id, x, y) in &humans {
            let mut company = false;
            for &(_, oid, ox, oy) in &humans {
                if oid == id {
                    continue;
                }
                let d = cheb(x, y, ox, oy);
                if d <= 4 {
                    company = true;
                    let tick = self.tick;
                    let r = rel_mut(&mut self.creatures[i].relations, oid);
                    r.familiarity = (r.familiarity + fx(0.004)).min(Fx::ONE);
                    r.last_seen = tick;
                    if r.familiarity > fx(0.3) {
                        r.affection = (r.affection + fx(0.0003)).min(Fx::ONE);
                    }
                }
            }
            let c = &mut self.creatures[i];
            if company {
                c.loneliness = (c.loneliness - fx(0.01)).max(Fx::ZERO);
            } else {
                c.loneliness = (c.loneliness + fx(0.004)).min(Fx::ONE);
            }
            // Attraction (adults): grows with familiarity and affection, shaped by temperament.
            let age = self.age_years(&self.creatures[i]);
            if age >= fx(15.0) {
                let female = self.creatures[i].female;
                let partners: Vec<(u32, bool, Fx)> = self
                    .creatures
                    .iter()
                    .filter(|o| o.alive && o.kind == Kind::Human && o.id != id)
                    .map(|o| (o.id, o.female, self.age_years(o)))
                    .collect();
                for (oid, of, oage) in partners {
                    if of == female || oage < fx(15.0) || (oage - age).abs() > fx(18.0) || self.related(id, oid) {
                        continue;
                    }
                    let c = &mut self.creatures[i];
                    let r = rel_mut(&mut c.relations, oid);
                    if r.familiarity > fx(0.2) {
                        r.attraction = (r.attraction + r.affection * fx(0.0004)).min(Fx::ONE);
                    }
                }
            }
        }
        self.pair_bonds();
    }

    fn related(&self, a: u32, b: u32) -> bool {
        let get = |id: u32| self.creatures.iter().find(|c| c.id == id);
        let (Some(x), Some(y)) = (get(a), get(b)) else { return false };
        x.mother == Some(b) || x.father == Some(b) || y.mother == Some(a) || y.father == Some(a)
            || (x.mother.is_some() && x.mother == y.mother)
            || (x.father.is_some() && x.father == y.father)
    }

    fn pair_bonds(&mut self) {
        let n = self.creatures.len();
        for i in 0..n {
            let c = &self.creatures[i];
            if !c.alive || c.kind != Kind::Human || c.partner.is_some() {
                continue;
            }
            let cand = c.relations.iter().filter(|r| r.attraction > fx(0.35) && r.affection > fx(0.4)).max_by_key(|r| r.attraction.raw()).copied();
            let Some(r) = cand else { continue };
            let Some(j) = self.creatures.iter().position(|o| o.id == r.other && o.alive && o.partner.is_none()) else { continue };
            let mutual = rel(&self.creatures[j].relations, self.creatures[i].id).map_or(false, |r2| r2.attraction > fx(0.3) && r2.affection > fx(0.35));
            if mutual {
                let (a, b) = (self.creatures[i].id, self.creatures[j].id);
                self.creatures[i].partner = Some(b);
                self.creatures[j].partner = Some(a);
                let (na, nb) = (self.creatures[i].name.clone(), self.creatures[j].name.clone());
                let (x, y) = (self.creatures[i].x, self.creatures[i].y);
                self.record("bond", format!("{na} and {nb} became a couple."), x, y, true);
                self.first("couple", format!("The first couple formed: {na} and {nb}."), x, y);
            }
        }
    }

    /// Hourly: health, ageing, conception, births, deaths.
    fn life_tick(&mut self) {
        let n = self.creatures.len();
        let mut births: Vec<(usize, u32)> = Vec::new();
        let mut deaths: Vec<(usize, String)> = Vec::new();
        for i in 0..n {
            if !self.creatures[i].alive {
                continue;
            }
            let age = self.age_years(&self.creatures[i]);
            let mut r = self.rng(self.creatures[i].id as u64, stream::PHYSICS ^ 0x11FE);
            if self.creatures[i].kind == Kind::Human {
                let c = &mut self.creatures[i];
                let s = c.body.signals;
                let mut dh = fx(0.004);
                let mut cause = None;
                if s.hunger >= fx(0.98) {
                    dh -= fx(0.012);
                    cause = Some("starvation");
                }
                if s.thirst >= fx(0.98) {
                    dh -= fx(0.03);
                    cause = Some("thirst");
                }
                if c.body.thermal < fx(-0.8) {
                    dh -= fx(0.008);
                    cause = Some("cold");
                }
                if c.body.damage_body > fx(0.7) {
                    dh -= fx(0.015);
                    cause = Some("wounds");
                }
                if self.tick < c.cursed_until {
                    dh -= fx(0.01);
                    cause = Some("a divine curse");
                }
                if self.tick < c.blessed_until {
                    dh += fx(0.02);
                }
                c.health = (c.health + dh).min(Fx::ONE);
                if c.health <= Fx::ZERO || c.body.damage_body >= Fx::ONE {
                    deaths.push((i, cause.unwrap_or("injuries").into()));
                    continue;
                }
                // Old age: Gompertz hazard (per year 0.0006·e^(0.09·age)), scaled by vitality.
                let hz_year = fx(0.0006) * (age * fx(0.09)).exp() * (fx(1.4) - c.traits.vitality * fx(0.8));
                let p_hour = hz_year / Fx::from_int((DAYS_PER_YEAR * 24) as i64);
                if r.chance(p_hour) {
                    deaths.push((i, "old age".into()));
                    continue;
                }
                // Mind development: refresh age-dependent parameters each year.
                let years = age.floor_int();
                if years != c.last_mind_age {
                    c.last_mind_age = years;
                    if let Some(m) = c.mind.as_mut() {
                        m.params = MindParams::new(age, &m.personality);
                    }
                }
                // Pregnancy.
                if let Some((father, due)) = c.pregnant {
                    if self.tick >= due {
                        births.push((i, father));
                    }
                } else if c.female && age >= fx(16.0) && age <= fx(44.0) {
                    if let Some(p) = c.partner {
                        let close = self.creatures.iter().any(|o| o.id == p && o.alive && cheb(o.x, o.y, self.creatures[i].x, self.creatures[i].y) <= 3);
                        let c = &mut self.creatures[i];
                        let fed = c.body.signals.hunger < fx(0.7);
                        if close && fed && r.chance(fx(0.04) * (fx(0.5) + c.traits.fertility)) {
                            c.pregnant = Some((p, self.tick + TICKS_PER_YEAR * 3 / 4));
                            let name = c.name.clone();
                            let (x, y) = (c.x, c.y);
                            self.record("pregnancy", format!("{name} is expecting a child."), x, y, false);
                        }
                    }
                }
            } else {
                // Animals: ageing and reproduction.
                let max_age = if self.creatures[i].kind == Kind::Grazer { fx(14.0) } else { fx(16.0) };
                let c = &mut self.creatures[i];
                if c.health <= Fx::ZERO {
                    deaths.push((i, c.cause_of_death.clone().unwrap_or("injuries".into())));
                    continue;
                }
                if age > max_age && r.chance(fx(0.05)) {
                    deaths.push((i, "old age".into()));
                    continue;
                }
                if c.female && age >= fx(2.0) && c.animal.energy > fx(0.6) && c.pregnant.is_none() {
                    let kind = c.kind;
                    let (x, y) = (c.x, c.y);
                    let pop = self.creatures.iter().filter(|o| o.alive && o.kind == kind).count();
                    let cap = if kind == Kind::Grazer { 80 } else { 14 };
                    let mate = self.creatures.iter().any(|o| o.alive && o.kind == kind && !o.female && cheb(o.x, o.y, x, y) <= 20);
                    if mate && pop < cap && r.chance(if kind == Kind::Grazer { fx(0.05) } else { fx(0.012) }) {
                        self.creatures[i].pregnant = Some((0, self.tick + TICKS_PER_YEAR / 2));
                    }
                }
                if let Some((_, due)) = self.creatures[i].pregnant {
                    if self.tick >= due {
                        births.push((i, 0));
                    }
                }
            }
        }
        for (i, cause) in deaths {
            self.die(i, &cause);
        }
        for (mi, father) in births {
            self.birth(mi, father);
        }
    }

    fn birth(&mut self, mi: usize, father: u32) {
        self.creatures[mi].pregnant = None;
        let (x, y, kind) = (self.creatures[mi].x, self.creatures[mi].y, self.creatures[mi].kind);
        let mother = self.creatures[mi].id;
        let mut r = self.rng(mother as u64, stream::PHYSICS ^ 0xB1);
        let female = r.below(2) == 0;
        if kind == Kind::Human {
            let id = self.spawn_human(x, y, 0, female, Some(mother), Some(father));
            if let Some(mi2) = self.creatures.iter().position(|c| c.id == mother) {
                self.creatures[mi2].children.push(id);
            }
            if let Some(fi) = self.creatures.iter().position(|c| c.id == father) {
                self.creatures[fi].children.push(id);
            }
            self.births += 1;
            let name = self.creatures.iter().find(|c| c.id == id).map(|c| c.name.clone()).unwrap_or_default();
            let mname = self.creatures[mi].name.clone();
            let fname = self.creatures.iter().find(|c| c.id == father).map(|c| c.name.clone()).unwrap_or("unknown".into());
            self.record("birth", format!("{name} was born to {mname} and {fname}."), x, y, true);
            self.first("birth", format!("The first child of this world was born: {name}."), x, y);
            if self.creatures.iter().filter(|c| c.kind == Kind::Human && c.mother == Some(mother) && c.father == Some(father)).count() == 1 {
                self.first("family", format!("The first family: {mname}, {fname} and their child {name}."), x, y);
            }
        } else {
            if let Some((sx, sy)) = self.random_land(&mut r, Some((x, y)), 1) {
                self.spawn_animal(kind, sx, sy, 0, female);
            }
        }
    }

    fn die(&mut self, i: usize, cause: &str) {
        let c = &mut self.creatures[i];
        if !c.alive {
            return;
        }
        c.alive = false;
        c.cause_of_death = Some(cause.into());
        c.died_tick = self.tick;
        let (x, y, kind, id) = (c.x, c.y, c.kind, c.id);
        let name = c.name.clone();
        let age = self.age_years(&self.creatures[i]).floor_int();
        if std::env::var_os("GENESIS_DEBUG_DEATHS").is_some() {
            eprintln!("death t={} kind={:?} cause={cause} age={age}", self.tick, kind);
        }
        if kind == Kind::Human {
            self.deaths += 1;
            self.record("death", format!("{name} died of {cause} at age {age}."), x, y, true);
            self.first("death", format!("{name} was the first person to die ({cause})."), x, y);
            self.add_obj(ObjKind::Remains, x, y);
            // Grief in those who knew them.
            for o in self.creatures.iter_mut().filter(|o| o.alive && o.kind == Kind::Human) {
                if let Some(r) = rel(&o.relations, id) {
                    if r.affection > fx(0.2) {
                        if let Some(m) = o.mind.as_mut() {
                            m.affect.distress = (m.affect.distress + r.affection).min(Fx::ONE);
                        }
                        o.loneliness = (o.loneliness + r.affection * fx(0.5)).min(Fx::ONE);
                    }
                }
                if o.partner == Some(id) {
                    o.partner = None;
                }
            }
        } else {
            self.add_obj(ObjKind::Carcass, x, y);
        }
    }

    // ------------------------------------------------------------------ animals

    fn animal_tick(&mut self, i: usize) {
        let interval = self.animal_interval.max(1) as u64;
        let id = self.creatures[i].id as u64;
        {
            // Predators gorge and then fast for a day or two; grazers eat steadily.
            let pred = self.creatures[i].kind == Kind::Predator;
            let a = &mut self.creatures[i].animal;
            a.energy -= if pred { fx(0.000005) } else { fx(0.000012) };
            a.water -= fx(0.000016);
            a.fatigue += fx(0.000008);
        }
        if a_dying(&self.creatures[i].animal) {
            let c = &mut self.creatures[i];
            c.health -= fx(0.00002);
            if c.health <= Fx::ZERO {
                c.cause_of_death = Some(if c.animal.water <= Fx::ZERO { "thirst".into() } else { "starvation".into() });
            }
        }
        if (self.tick + id) % interval != 0 {
            return;
        }
        let kind = self.creatures[i].kind;
        let (x, y) = (self.creatures[i].x, self.creatures[i].y);
        let mut r = self.rng(id, stream::DECIDE);
        // Fire: innate avoidance (species prior).
        if let Some(f) = self.objs.iter().find(|o| o.alive && o.kind == ObjKind::Fire && cheb(o.x, o.y, x, y) <= 4) {
            let (fx_, fy) = (f.x, f.y);
            self.step_toward(i, fx_, fy, false);
            self.creatures[i].action = "fleeing fire".into();
            return;
        }
        match kind {
            Kind::Grazer => {
                // Threats: wolves (innate) and humans (learned fear).
                let fear_h = self.creatures[i].animal.fear_of_humans;
                let threat = self
                    .creatures
                    .iter()
                    .filter(|o| o.alive && o.id != id as u32)
                    .filter(|o| (o.kind == Kind::Predator && cheb(o.x, o.y, x, y) <= 6) || (o.kind == Kind::Human && fear_h > fx(0.2) && cheb(o.x, o.y, x, y) <= 4))
                    .map(|o| (o.x, o.y))
                    .next();
                if let Some((tx, ty)) = threat.or(self.creatures[i].animal.flee_from.take()) {
                    self.step_toward(i, tx, ty, false);
                    self.creatures[i].action = "fleeing".into();
                    return;
                }
                self.forage(i, &mut r, true);
            }
            Kind::Predator => {
                if self.tick < self.creatures[i].animal.retreat_until {
                    if let Some((tx, ty)) = self.creatures[i].animal.flee_from {
                        self.step_toward(i, tx, ty, false);
                        self.creatures[i].action = "backing off".into();
                        return;
                    }
                }
                // Thirst comes before the hunt.
                if self.creatures[i].animal.water < fx(0.3) {
                    self.forage(i, &mut r, false);
                    return;
                }
                let hunger = Fx::ONE - self.creatures[i].animal.energy;
                // Gorge on any carcass nearby until full; hunt only when truly hungry.
                if hunger > fx(0.1) {
                    if let Some(oi) = self.objs.iter().position(|o| o.alive && o.kind == ObjKind::Carcass && cheb(o.x, o.y, x, y) <= 8) {
                        let (ox, oy) = (self.objs[oi].x, self.objs[oi].y);
                        if cheb(ox, oy, x, y) <= 1 {
                            self.objs[oi].props.portions -= 1;
                            if self.objs[oi].props.portions <= 0 {
                                self.objs[oi].alive = false;
                            }
                            self.creatures[i].animal.energy = (self.creatures[i].animal.energy + fx(0.12)).min(Fx::ONE);
                            self.creatures[i].action = "eating".into();
                        } else {
                            self.step_toward(i, ox, oy, true);
                            self.creatures[i].action = "going to a carcass".into();
                        }
                        return;
                    }
                }
                if hunger > fx(0.55) {
                    // Hunt: grazers, or humans when very hungry.
                    let prey = self
                        .creatures
                        .iter()
                        .enumerate()
                        .filter(|(_, o)| {
                            o.alive
                                && (o.kind == Kind::Grazer
                                    || (o.kind == Kind::Human && hunger > fx(0.85) && self.humans_near(o.x, o.y, 2) < 3))
                        })
                        .map(|(j, o)| (cheb(o.x, o.y, x, y), j))
                        .filter(|(d, _)| *d <= 12)
                        .min();
                    if let Some((d, j)) = prey {
                        let (px, py) = (self.creatures[j].x, self.creatures[j].y);
                        if d <= 1 {
                            self.attack(i, j);
                        } else {
                            self.step_toward(i, px, py, true);
                            self.creatures[i].action = "hunting".into();
                        }
                        return;
                    }
                }
                self.forage(i, &mut r, false);
            }
            Kind::Human => {}
        }
    }

    fn humans_near(&self, x: i32, y: i32, r: i32) -> usize {
        self.creatures.iter().filter(|o| o.alive && o.kind == Kind::Human && cheb(o.x, o.y, x, y) <= r).count()
    }

    fn attack(&mut self, i: usize, j: usize) {
        let mut r = self.rng(self.creatures[i].id as u64, stream::PHYSICS ^ 0xA7);
        self.creatures[i].action = "attacking".into();
        if !r.chance(fx(0.45)) {
            return;
        }
        let (tx, ty) = (self.creatures[j].x, self.creatures[j].y);
        if self.creatures[j].kind == Kind::Human {
            // Wolves bite and back off rather than fight a standing human to the death.
            self.creatures[i].animal.flee_from = Some((tx, ty));
            self.creatures[i].animal.retreat_until = self.tick + 1800;
            self.creatures[i].animal.energy = (self.creatures[i].animal.energy + fx(0.05)).min(Fx::ONE);
            self.creatures[j].body.wound(fx(0.16));
            let name = self.creatures[j].name.clone();
            self.record("attack", format!("A wolf attacked {name}."), tx, ty, false);
            self.first("wolf-attack", format!("A wolf attacked a human for the first time: {name}."), tx, ty);
            if self.creatures[j].body.damage_body >= fx(0.95) {
                self.die(j, "a wolf attack");
                self.creatures[i].kills += 1;
            }
        } else {
            self.creatures[j].health -= fx(0.35);
            self.creatures[j].animal.flee_from = Some((self.creatures[i].x, self.creatures[i].y));
            if self.creatures[j].health <= Fx::ZERO {
                self.creatures[j].cause_of_death = Some("a wolf".into());
                self.die(j, "a wolf");
                self.creatures[i].kills += 1;
            }
        }
    }

    /// Shared animal routine: drink, eat (graze), rest, roam.
    fn forage(&mut self, i: usize, r: &mut Rng, grazer: bool) {
        let (x, y) = (self.creatures[i].x, self.creatures[i].y);
        let a = self.creatures[i].animal.clone();
        if a.water < fx(0.45) {
            let near = (-1..=1).flat_map(|dy| (-1..=1).map(move |dx| (x + dx, y + dy))).find(|&(wx, wy)| self.in_bounds(wx, wy) && self.tile(wx, wy).is_water());
            if near.is_some() {
                self.creatures[i].animal.water = Fx::ONE;
                self.creatures[i].animal.last_water = Some((x, y));
                self.creatures[i].action = "drinking".into();
                return;
            }
            if !self.step_to_water(i) {
                self.wander(i, r);
            }
            self.creatures[i].action = "going to water".into();
            return;
        }
        if grazer && a.energy < fx(0.8) {
            let g = self.tile(x, y).grass;
            if g > 60 {
                self.tile_mut(x, y).grass -= 25;
                self.creatures[i].animal.energy = (a.energy + fx(0.004)).min(Fx::ONE);
                self.creatures[i].action = "grazing".into();
                return;
            }
            let mut best = (g, x, y);
            for (dx, dy) in DIRS {
                let (nx, ny) = (x + dx * 2, y + dy * 2);
                if self.walkable(nx, ny) && self.tile(nx, ny).grass > best.0 {
                    best = (self.tile(nx, ny).grass, nx, ny);
                }
            }
            if best.1 != x || best.2 != y {
                self.step_toward(i, best.1, best.2, true);
                self.creatures[i].action = "looking for grass".into();
                return;
            }
        }
        let h = self.time_of_day() / 3600;
        if a.fatigue > fx(0.5) || (h < 5 && r.below(3) != 0) {
            self.creatures[i].animal.fatigue = (a.fatigue - fx(0.0005)).max(Fx::ZERO);
            self.creatures[i].action = "resting".into();
            return;
        }
        if r.below(4) == 0 {
            self.wander(i, r);
            self.creatures[i].action = "roaming".into();
        }
    }

    fn wander(&mut self, i: usize, r: &mut Rng) {
        let (dx, dy) = DIRS[r.below(8) as usize];
        let (nx, ny) = (self.creatures[i].x + dx, self.creatures[i].y + dy);
        if self.walkable(nx, ny) {
            self.creatures[i].x = nx;
            self.creatures[i].y = ny;
        }
    }

    /// Breadth-first distance field from every water tile across walkable land.
    fn ensure_water_dist(&mut self) {
        if !self.water_dist.is_empty() {
            return;
        }
        let (w, h) = (self.w, self.h);
        let mut d = vec![u16::MAX; (w * h) as usize];
        let mut q = std::collections::VecDeque::new();
        for y in 0..h {
            for x in 0..w {
                if self.tile(x, y).is_water() {
                    d[(y * w + x) as usize] = 0;
                    q.push_back((x, y));
                }
            }
        }
        while let Some((x, y)) = q.pop_front() {
            let nd = d[(y * w + x) as usize] + 1;
            for (dx, dy) in DIRS {
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx >= w || ny >= h {
                    continue;
                }
                let k = (ny * w + nx) as usize;
                if d[k] > nd && self.tile(nx, ny).walkable() {
                    d[k] = nd;
                    q.push_back((nx, ny));
                }
            }
        }
        self.water_dist = d;
    }

    /// One step down the water distance field. False if no path exists.
    fn step_to_water(&mut self, i: usize) -> bool {
        self.ensure_water_dist();
        let (x, y) = (self.creatures[i].x, self.creatures[i].y);
        let w = self.w;
        let here = self.water_dist[(y * w + x) as usize];
        let mut best: Option<(u16, i32, i32)> = None;
        for (dx, dy) in DIRS {
            let (nx, ny) = (x + dx, y + dy);
            if !self.in_bounds(nx, ny) {
                continue;
            }
            let d = self.water_dist[(ny * w + nx) as usize];
            if d < here && (d == 0 || self.walkable(nx, ny)) && best.map_or(true, |b| d < b.0) {
                best = Some((d, nx, ny));
            }
        }
        match best {
            Some((0, _, _)) => true, // water is adjacent: drink next turn
            Some((_, nx, ny)) => {
                self.creatures[i].x = nx;
                self.creatures[i].y = ny;
                true
            }
            None => false,
        }
    }

    // ------------------------------------------------------------------ god powers

    /// Applies a god power at a tile. Returns a short description of what happened.
    pub fn god(&mut self, power: &str, x: i32, y: i32) -> String {
        if !self.in_bounds(x, y) {
            return "outside the world".into();
        }
        let mut r = self.rng(0xD0D0, stream::PHYSICS ^ x as u64 ^ ((y as u64) << 20));
        let near = |s: &Sim, maxd: i32| -> Option<usize> {
            s.creatures
                .iter()
                .enumerate()
                .filter(|(_, c)| c.alive && cheb(c.x, c.y, x, y) <= maxd)
                .min_by_key(|(_, c)| cheb(c.x, c.y, x, y))
                .map(|(i, _)| i)
        };
        let msg = match power {
            "create_human" => match self.random_land(&mut r, Some((x, y)), 2) {
                Some((sx, sy)) => {
                    let female = r.below(2) == 0;
                    let age = 16 + r.below(14) as i64;
                    let id = self.spawn_human(sx, sy, age, female, None, None);
                    let name = self.creatures.iter().find(|c| c.id == id).unwrap().name.clone();
                    self.record("god", format!("God created a human: {name}."), sx, sy, true);
                    format!("Created {name}")
                }
                None => "no land here".into(),
            },
            "create_grazer" | "create_predator" | "create_animal" => match self.random_land(&mut r, Some((x, y)), 2) {
                Some((sx, sy)) => {
                    let kind = if power == "create_predator" { Kind::Predator } else { Kind::Grazer };
                    self.spawn_animal(kind, sx, sy, 2, r.below(2) == 0);
                    format!("Created a {}", if kind == Kind::Predator { "wolf" } else { "grazer" })
                }
                None => "no land here".into(),
            },
            "create_plant" => {
                let mut n = 0;
                for _ in 0..3 {
                    if let Some((sx, sy)) = self.random_land(&mut r, Some((x, y)), 3) {
                        self.add_obj(if n == 2 { ObjKind::Tree } else { ObjKind::BerryBush }, sx, sy);
                        n += 1;
                    }
                }
                format!("Planted {n} plants")
            }
            "add_food" => {
                let mut n = 0;
                for _ in 0..4 {
                    if let Some((sx, sy)) = self.random_land(&mut r, Some((x, y)), 3) {
                        self.add_obj(ObjKind::BerryBush, sx, sy);
                        n += 1;
                    }
                }
                self.record("god", "God made berry bushes grow.".into(), x, y, false);
                format!("{n} berry bushes appeared")
            }
            "add_water" => {
                for dy in -2..=2 {
                    for dx in -2..=2 {
                        if dx * dx + dy * dy <= 5 && self.in_bounds(x + dx, y + dy) {
                            let t = self.tile_mut(x + dx, y + dy);
                            t.biome = Biome::Water;
                        }
                    }
                }
                self.objs.retain(|o| !(cheb(o.x, o.y, x, y) <= 2 && (o.x - x).pow(2) + (o.y - y).pow(2) <= 5) || o.kind == ObjKind::Fire);
                self.water_dist.clear();
                self.record("god", "God made a pond.".into(), x, y, false);
                "A pond appeared".into()
            }
            "kill" => match near(self, 2) {
                Some(i) => {
                    let n = self.creatures[i].name.clone();
                    self.die(i, "the hand of God");
                    format!("{n} was struck down")
                }
                None => "nobody here".into(),
            },
            "heal" => match near(self, 2) {
                Some(i) => {
                    let c = &mut self.creatures[i];
                    c.health = Fx::ONE;
                    c.body.damage_body = Fx::ZERO;
                    c.body.damage_hand = Fx::ZERO;
                    c.body.damage_mouth = Fx::ZERO;
                    c.cursed_until = 0;
                    let n = c.name.clone();
                    self.record("god", format!("God healed {n}."), x, y, false);
                    format!("Healed {n}")
                }
                None => "nobody here".into(),
            },
            "bless" => match near(self, 2) {
                Some(i) => {
                    let t = self.tick;
                    let c = &mut self.creatures[i];
                    c.blessed_until = t + TICKS_PER_DAY * 2;
                    c.health = Fx::ONE;
                    c.body.energy = alife_biology::k::energy_setpoint();
                    c.body.water = alife_biology::k::water_setpoint();
                    c.animal.energy = Fx::ONE;
                    c.animal.water = Fx::ONE;
                    let n = c.name.clone();
                    self.record("god", format!("God blessed {n}."), x, y, true);
                    format!("Blessed {n}")
                }
                None => "nobody here".into(),
            },
            "curse" => match near(self, 2) {
                Some(i) => {
                    let t = self.tick;
                    let c = &mut self.creatures[i];
                    c.cursed_until = t + TICKS_PER_DAY * 2;
                    c.body.wound(fx(0.2));
                    let n = c.name.clone();
                    self.record("god", format!("God cursed {n}."), x, y, true);
                    format!("Cursed {n}")
                }
                None => "nobody here".into(),
            },
            "lightning" => {
                self.strike_lightning(x, y, true);
                "Lightning!".into()
            }
            "fire" => {
                if self.walkable(x, y) || self.obj_at(x, y) {
                    self.objs.retain(|o| !(o.x == x && o.y == y && o.kind == ObjKind::Tree));
                    self.add_obj(ObjKind::Fire, x, y);
                    self.record("god", "God set a fire.".into(), x, y, false);
                    self.first("fire", "Fire appeared in the world for the first time.".into(), x, y);
                    "Fire!".into()
                } else {
                    "cannot burn here".into()
                }
            }
            "rain" => {
                self.rain_until = self.tick + 4 * 3600;
                self.weather = Weather::Rain;
                self.weather_until = self.tick + 4 * 3600;
                for o in self.objs.iter_mut().filter(|o| o.kind == ObjKind::Fire) {
                    o.timer = o.timer.min(300);
                }
                self.record("god", "God sent rain.".into(), x, y, false);
                "Rain falls".into()
            }
            "warmer" => {
                self.god_temp += fx(4.0);
                format!("World warmer ({:+.0}°C)", self.god_temp.to_f64())
            }
            "colder" => {
                self.god_temp -= fx(4.0);
                format!("World colder ({:+.0}°C)", self.god_temp.to_f64())
            }
            _ => "unknown power".into(),
        };
        msg
    }

    // ------------------------------------------------------------------ time & LOD

    /// Chooses cognitive detail for a game speed (multiples of 1 game minute per real second).
    pub fn set_speed_lod(&mut self, speed: u32) {
        let (h, a) = match speed {
            0..=5 => (1, 1),
            6..=25 => (2, 2),
            26..=100 => (6, 4),
            _ => (20, 10),
        };
        self.human_interval = h;
        self.animal_interval = a;
    }

    pub fn fidelity_label(&self) -> String {
        match self.human_interval {
            1 => "Full cognition: every human thinks every second".into(),
            n => format!("Reduced cognition: humans think every {n} s (selected human: every second)"),
        }
    }
}

fn a_dying(a: &AnimalBrain) -> bool {
    a.energy <= Fx::ZERO || a.water <= Fx::ZERO
}

fn wrap(d: Fx) -> Fx {
    let full = fx(360.0);
    let mut v = d;
    while v < Fx::ZERO {
        v += full;
    }
    while v >= full {
        v -= full;
    }
    v
}

trait Pipe: Sized {
    fn pipe<R>(self, f: impl FnOnce(Self) -> R) -> R {
        f(self)
    }
}
impl<T> Pipe for T {}

pub fn obj_name(o: &Obj) -> String {
    match o.kind {
        ObjKind::BerryBush => "berry bush",
        ObjKind::Tree => "tree",
        ObjKind::Stone => "stone",
        ObjKind::Flint => "flint",
        ObjKind::Fire => "fire",
        ObjKind::Carcass => if o.cooked { "cooked meat" } else { "carcass" },
        ObjKind::Flake => "sharp flake",
        ObjKind::Remains => "remains",
    }
    .into()
}

pub fn rel(v: &[Relation], other: u32) -> Option<&Relation> {
    v.iter().find(|r| r.other == other)
}

pub fn rel_mut(v: &mut Vec<Relation>, other: u32) -> &mut Relation {
    if let Some(i) = v.iter().position(|r| r.other == other) {
        return &mut v[i];
    }
    v.push(Relation { other, ..Default::default() });
    v.last_mut().unwrap()
}
