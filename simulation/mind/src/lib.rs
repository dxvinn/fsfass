//! ArtificialMind v0.
//!
//! The mind receives a `SensoryFrame` and returns a `MotorCommand`. It cannot
//! see the world: this crate depends only on `alife-core` and
//! `alife-interface`. Everything it knows about which things cause which
//! outcomes is learned from its own experience.
//!
//! One cognitive cycle (once per game second while awake):
//! 1. recognise context and percepts (sense units -> learned concepts)
//! 2. evaluate what just happened: outcomes from body-signal changes,
//!    prediction errors, learning (instrumental, Pavlovian, habit), episodes
//! 3. attention: percepts, needs and memories compete for K workspace slots
//! 4. recall episodes for attended things
//! 5. generate options from attended things; score them with three systems:
//!    reflex/gut (innate reflex + Pavlovian bias), habit (cached values),
//!    deliberate (learned outcome model + episodic estimate + lookahead);
//!    add curiosity and persistence; arbitrate by reliability; select
//! 6. update emotions; remember what was done for the next learning step
//!
//! Sleep runs consolidation: replay, fast->slow transfer, forgetting, beliefs.

pub mod affect;
pub mod attention;
pub mod concepts;
pub mod drives;
pub mod encode;
pub mod learning;
pub mod memory;
pub mod params;
pub mod trace;

use affect::Affect;
use alife_core::hash::{StableHash, StateHasher};
use alife_core::rng::stream;
use alife_core::{fx, Fx, Rng};
use alife_interface::{Ambient, Interoception, MotorCommand, SensoryFrame, Token};
use attention::{need_label, Goal, ItemKind, WorkingMemory, WsItem};
use concepts::{ConceptStore, Recognition};
use encode::{describe, encode, encode_consequence, ConseqVec, SenseVec, N_CONSEQ, N_SENSE};
use learning::assoc::{
    action_label, outcome_label, Assoc, CueVec, Out, A_APPROACH, A_CONTACT, A_INSPECT, A_MOUTH, A_TOUCH, A_WITHDRAW, N_ACT,
    N_CTX, N_CUES, N_OUT, O_HYDRATE, O_NOURISH, O_PAIN,
};
use learning::habit::{Habits, H_REST, H_WANDER};
use memory::episodic::{Episode, EpisodicMemory, Recall};
use memory::semantic::SemanticMemory;
use params::{MindParams, Personality};
use trace::{AssocLine, MemoryLine, OptionLine, PerceptLine, TraceRecord};

/// Cognitive capabilities, configured per species (animals later get fewer).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Capabilities {
    pub episodic_memory: bool,
    pub deliberate_planning: bool,
    pub semantic_beliefs: bool,
    pub concept_learning: bool,
    pub language: bool,
    pub theory_of_mind: bool,
    pub social_learning: bool,
    pub tool_use: bool,
    pub self_recognition: bool,
}

impl Capabilities {
    pub fn human() -> Self {
        Capabilities {
            episodic_memory: true,
            deliberate_planning: true,
            semantic_beliefs: true,
            concept_learning: true,
            language: false, // not implemented until P3/P5
            theory_of_mind: false,
            social_learning: false,
            tool_use: false,
            self_recognition: false,
        }
    }
}

/// Learned context prototypes (from ambient sensations, not places).
#[derive(Clone, Debug, PartialEq)]
pub struct ContextStore {
    pub protos: Vec<[Fx; 4]>,
}

impl ContextStore {
    fn vec(a: &Ambient) -> [Fx; 4] {
        [a.light, a.ground, a.enclosure, a.sound]
    }
    pub fn recognise(&mut self, a: &Ambient) -> usize {
        let v = Self::vec(a);
        let dist = |p: &[Fx; 4]| p.iter().zip(v.iter()).fold(Fx::ZERO, |m, (x, y)| m.max((*x - *y).abs()));
        if let Some((i, d)) = self.protos.iter().enumerate().map(|(i, p)| (i, dist(p))).min_by_key(|(_, d)| d.raw()) {
            if d < fx(0.12) || self.protos.len() >= N_CTX {
                return i;
            }
        }
        self.protos.push(v);
        self.protos.len() - 1
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Pending {
    cmd: MotorCommand,
    act: Option<usize>,
    target: Option<Token>,
    cue: CueVec,
    sense: SenseVec,
    concept: u16,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MindStats {
    pub decisions: u64,
    pub instrumental_trials: u64,
    pub pavlovian_events: u64,
    pub episodes_stored: u64,
    pub consolidations: u64,
}

/// Per-percept state for the current cycle.
#[derive(Clone, Debug)]
struct PState {
    token: Token,
    dist: i32,
    sense: SenseVec,
    cue: CueVec,
    rec: Recognition,
    conseq: ConseqVec,
    contact: bool,
    pav: Out,
    pav_value: Fx,
    salience: Fx,
    intensity: Fx,
}

#[derive(Clone, Debug)]
struct Opt {
    cmd: MotorCommand,
    act: Option<usize>,
    pi: Option<usize>,
    pred: Out,
    deliberate: Fx,
    habit: Fx,
    gut: Fx,
    curiosity: Fx,
    persistence: Fx,
    noise: Fx,
    total: Fx,
    episodic: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Mind {
    pub id: u64,
    pub seed: u64,
    pub personality: Personality,
    pub params: MindParams,
    pub caps: Capabilities,
    pub concepts: ConceptStore,
    pub contexts: ContextStore,
    pub assoc: Assoc,
    pub habits: Habits,
    pub episodic: EpisodicMemory,
    pub semantic: SemanticMemory,
    pub wm: WorkingMemory,
    pub affect: Affect,
    prev_body: Interoception,
    pending: Option<Pending>,
    /// Running mean of |outcome prediction error| of the learned model.
    model_error: Fx,
    lp_fast: [Fx; N_ACT],
    lp_slow: [Fx; N_ACT],
    was_asleep: bool,
    last_consolidation: u64,
    /// Multiplier on curiosity from unmet needs (1 = none).
    explore_drive: Fx,
    /// Consecutive repetitions of the same command.
    repeats: u16,
    pub trace_enabled: bool,
    pub last_trace: Option<TraceRecord>,
    pub stats: MindStats,
    pub last_learning: Vec<String>,
}

const AVERSIVE: [bool; N_OUT] = [true, false, false];

impl Mind {
    pub fn new(id: u64, seed: u64, age_years: Fx, personality: Personality) -> Mind {
        let params = MindParams::new(age_years, &personality);
        Mind {
            id,
            seed,
            personality,
            params,
            caps: Capabilities::human(),
            concepts: ConceptStore::new(fx(0.9)),
            contexts: ContextStore { protos: Vec::new() },
            assoc: Assoc::new(),
            habits: Habits::new(),
            episodic: EpisodicMemory::new(),
            semantic: SemanticMemory::new(),
            wm: WorkingMemory::new(params.workspace),
            affect: Affect::default(),
            prev_body: Interoception::default(),
            pending: None,
            model_error: fx(0.5),
            lp_fast: [Fx::ZERO; N_ACT],
            lp_slow: [Fx::ZERO; N_ACT],
            was_asleep: false,
            last_consolidation: 0,
            explore_drive: Fx::ONE,
            repeats: 0,
            trace_enabled: false,
            last_trace: None,
            stats: MindStats::default(),
            last_learning: Vec::new(),
        }
    }

    /// Call when the mind is moved into a new environment: forget transient
    /// short-term state (tokens are meaningless now). Long-term memory stays.
    pub fn new_environment(&mut self) {
        self.pending = None;
        self.wm.slots.clear();
        self.wm.goal = Goal::None;
        self.wm.last_command = None;
    }

    fn concept_slot(&self, id: u16) -> Option<usize> {
        self.concepts.concepts.iter().position(|c| c.id == id)
    }

    fn cue_vec(sense: &SenseVec, rec: &Recognition) -> CueVec {
        let mut x = [Fx::ZERO; N_CUES];
        x[..N_SENSE].copy_from_slice(sense);
        x[N_SENSE + rec.slot] = rec.similarity;
        x
    }

    fn cue_label(&self, c: usize) -> String {
        if c < N_SENSE {
            encode::sense_label(c)
        } else {
            let slot = c - N_SENSE;
            match self.concepts.concepts.get(slot) {
                Some(k) => format!("concept C{}", k.id),
                None => format!("concept slot {slot}"),
            }
        }
    }

    fn token_label(t: Token) -> String {
        format!("thing#{:04x}", t.0 & 0xffff)
    }

    fn cmd_label(&self, cmd: &MotorCommand, ps: &[PState]) -> String {
        let tgt = |t: Token| {
            ps.iter()
                .find(|p| p.token == t)
                .map(|p| format!("C{}@{}m", p.rec.id, p.dist))
                .unwrap_or_else(|| Self::token_label(t))
        };
        match *cmd {
            MotorCommand::Rest => "rest".into(),
            MotorCommand::Wander { .. } => "wander".into(),
            MotorCommand::Approach { target } => format!("approach {}", tgt(target)),
            MotorCommand::Inspect { target } => format!("inspect {} from 2m", tgt(target)),
            MotorCommand::Touch { target } => format!("touch {}", tgt(target)),
            MotorCommand::Mouth { target } => format!("mouth {}", tgt(target)),
            MotorCommand::Withdraw { target } => format!("withdraw from {}", tgt(target)),
        }
    }

    /// Outcome prediction for an action, generalising from the "contact"
    /// family when this specific action has little evidence on these cues.
    fn predict_action(&self, a: usize, x: &CueVec, ctx: usize) -> Out {
        let own = self.assoc.predict_inst(a, x, ctx);
        if a != A_TOUCH && a != A_MOUTH {
            return own;
        }
        let ev = self.assoc.evidence_for(a, x);
        let w_family = fx(2.0) / (fx(2.0) + ev);
        let fam = self.assoc.predict_inst(A_CONTACT, x, ctx);
        let mut out = own;
        for o in 0..N_OUT {
            out[o] = own[o] * (Fx::ONE - w_family) + fam[o] * w_family;
        }
        out
    }

    /// One cognitive cycle.
    pub fn step(&mut self, f: &SensoryFrame) -> MotorCommand {
        let tick = f.tick;
        self.affect.tick();
        self.last_learning.clear();
        if f.asleep {
            if !self.was_asleep {
                self.sleep(tick);
                self.was_asleep = true;
            }
            self.prev_body = f.body;
            self.pending = None;
            if self.trace_enabled {
                self.last_trace = Some(TraceRecord {
                    tick,
                    asleep: true,
                    learning: self.last_learning.clone(),
                    ..Default::default()
                });
            }
            return MotorCommand::Rest;
        }
        self.was_asleep = false;
        let ctx = self.contexts.recognise(&f.ambient);

        // 1. Perception.
        let mut ps: Vec<PState> = Vec::with_capacity(f.percepts.len());
        for p in &f.percepts {
            let sense = encode(p);
            let rec = self.concepts.recognise(&sense, tick);
            if let Some(old) = rec.replaced {
                self.assoc.clear_cue(N_SENSE + rec.slot);
                self.habits.clear_concept(old);
            }
            let cue = Self::cue_vec(&sense, &rec);
            let conseq = encode_consequence(p.touch.as_ref(), p.taste.as_ref());
            let intensity = p.visual.brightness * fx(0.5)
                + p.visual.flicker * fx(0.4)
                + p.visual.saturation * fx(0.15)
                + p.visual.size * fx(0.2)
                + p.felt_warmth * fx(0.8);
            ps.push(PState {
                token: p.token,
                dist: p.dist,
                sense,
                cue,
                rec,
                conseq,
                contact: p.touch.is_some() || p.taste.is_some(),
                pav: [Fx::ZERO; N_OUT],
                pav_value: Fx::ZERO,
                salience: Fx::ZERO,
                intensity,
            });
        }
        let valence = drives::innate_valence(&f.body);
        for q in ps.iter_mut() {
            q.pav = self.assoc.predict_pav(&q.cue, ctx);
            q.pav_value = (0..N_OUT).fold(Fx::ZERO, |s, o| s + q.pav[o] * valence[o]);
        }

        // 2. Outcome evaluation and learning.
        let out = drives::outcomes(&self.prev_body, &f.body);
        let reward = drives::reward(&self.prev_body, &f.body, out[O_PAIN], &self.params);
        self.learn(f, &ps, &out, reward, ctx, tick);
        Affect::raise(&mut self.affect.distress, f.body.pain);
        if reward > fx(0.02) {
            Affect::raise(&mut self.affect.joy, reward * fx(2.0));
        }

        // 3. Attention.
        let hunger = f.body.hunger;
        let thirst = f.body.thirst;
        let pending_target = self.pending.as_ref().and_then(|p| p.target);
        for q in ps.iter_mut() {
            let seen = self.concepts.concepts.get(q.rec.slot).map(|c| c.seen).unwrap_or(1);
            let novelty = Fx::ONE / (Fx::ONE + Fx::from_int(seen as i64) / fx(20.0)).sqrt();
            let pm = self.assoc.predict_inst(A_MOUTH, &q.cue, ctx);
            let relevance = pm[O_NOURISH] * hunger + pm[O_HYDRATE] * thirst;
            // Need-driven search: while a need is unmet, things whose intake
            // outcome is still unknown attract attention.
            let unknown = Fx::ONE / (Fx::ONE + self.assoc.evidence_for(A_MOUTH, &q.cue));
            let search = hunger.max(thirst) * unknown * (Fx::ONE - q.pav[O_PAIN].muli(3)).clamp01();
            let mut s = q.intensity * fx(0.6) + novelty * fx(0.5) + q.pav_value.abs() * fx(1.2) + relevance + search;
            if self.wm.holds(q.token) {
                s += fx(0.3);
            }
            if pending_target == Some(q.token) {
                s += fx(0.4);
            }
            s += fx(0.3) / Fx::from_int(1 + q.dist as i64);
            q.salience = s;
        }
        let mut cands: Vec<WsItem> =
            ps.iter().map(|q| WsItem { kind: ItemKind::Percept(q.token), salience: q.salience }).collect();
        cands.push(WsItem { kind: ItemKind::Need(0), salience: hunger });
        cands.push(WsItem { kind: ItemKind::Need(1), salience: thirst });
        if f.body.pain > fx(0.05) {
            cands.push(WsItem { kind: ItemKind::Need(2), salience: f.body.pain * fx(1.5) });
        }
        let mut k = self.params.workspace;
        if self.affect.arousal > fx(0.6) && k > 2 {
            k -= 1;
        }

        // 4. Recall for the most salient things.
        let mut by_sal: Vec<usize> = (0..ps.len()).collect();
        by_sal.sort_by(|&a, &b| ps[b].salience.cmp(&ps[a].salience).then(ps[a].token.cmp(&ps[b].token)));
        let mut recalls: Vec<(usize, Vec<Recall>)> = Vec::new();
        if self.caps.episodic_memory {
            let mut best: Option<Recall> = None;
            for &i in by_sal.iter().take(3) {
                let r = self.episodic.retrieve(&ps[i].sense, ps[i].rec.id, None, tick, 3);
                if let Some(top) = r.first() {
                    if best.as_ref().map_or(true, |b| top.activation > b.activation) {
                        best = Some(top.clone());
                    }
                }
                recalls.push((i, r));
            }
            // The single most active memory competes for the workspace ("being reminded").
            if let Some(b) = best {
                cands.push(WsItem { kind: ItemKind::Memory(b.episode), salience: b.activation / fx(6.0) });
            }
        }
        self.wm.capacity = k;
        self.wm.admit(cands, k);
        let attended: Vec<usize> =
            (0..ps.len()).filter(|&i| self.wm.holds(ps[i].token)).collect();

        // Safe exposure to things predicted to hurt (Pavlovian extinction, context-bound).
        if out[O_PAIN].raw() == 0 && f.body.pain < fx(0.2) {
            for &i in &attended {
                if ps[i].dist <= 2 {
                    self.assoc.exposure_without_outcome(&self.params, &ps[i].cue, ctx, O_PAIN, fx(1.0 / 60.0));
                }
            }
        }

        // 5. Options and the three decision systems.
        let util = drives::utilities(&f.body, self.affect.fear, &self.params);
        // Need-driven exploration (PSI): a strong need with no known way to
        // relieve it makes trying unfamiliar things more attractive.
        let need = f.body.hunger.max(f.body.thirst);
        let mut known_relief = Fx::ZERO;
        for &i in &attended {
            let pm = self.assoc.predict_inst(A_MOUTH, &ps[i].cue, ctx);
            known_relief = known_relief.max(pm[O_NOURISH] * f.body.hunger + pm[O_HYDRATE] * f.body.thirst);
        }
        self.explore_drive = Fx::ONE + (need - known_relief.muli(2)).max(Fx::ZERO).muli(3);
        let mut rng = Rng::new(self.seed, self.id, tick, stream::DECIDE);
        let temp = self.params.temperature * (Fx::ONE - self.affect.arousal * fx(0.5));
        let last_cmd = self.wm.last_command;
        let mut opts: Vec<Opt> = Vec::new();
        for &i in &attended {
            let q = &ps[i];
            let t = q.token;
            let mut acts = vec![(MotorCommand::Touch { target: t }, A_TOUCH), (MotorCommand::Mouth { target: t }, A_MOUTH)];
            if q.dist > 1 {
                acts.push((MotorCommand::Approach { target: t }, A_APPROACH));
            }
            acts.push((MotorCommand::Inspect { target: t }, A_INSPECT));
            // Withdrawal is an option only from things that are expected to hurt.
            if q.dist <= 3 && (q.pav[O_PAIN] > fx(0.1) || (q.contact && f.body.acute_pain > fx(0.1))) {
                acts.push((MotorCommand::Withdraw { target: t }, A_WITHDRAW));
            }
            let rec = recalls.iter().find(|(pi, _)| *pi == i).map(|(_, r)| r.as_slice()).unwrap_or(&[]);
            for (cmd, a) in acts {
                let o = self.score_option(cmd, a, i, &ps, rec, &util, ctx, f, temp, &mut rng, last_cmd);
                opts.push(o);
            }
        }
        // Untargeted options.
        let wander_dir = rng.below(8) as u8;
        let bored = Fx::ONE - attended.iter().map(|&i| ps[i].salience).fold(Fx::ZERO, |m, s| m.max(s)).min(Fx::ONE);
        for cmd in [MotorCommand::Wander { dir: wander_dir }, MotorCommand::Rest] {
            let h = if matches!(cmd, MotorCommand::Rest) { H_REST } else { H_WANDER };
            let (hq, hn) = self.habits.q(0, h);
            let deliberate = if matches!(cmd, MotorCommand::Rest) {
                f.body.fatigue * fx(0.15) + f.body.pain * fx(0.2)
            } else {
                fx(0.04) * bored * self.params.curiosity
            };
            let persistence = if last_cmd.map(|c| std::mem::discriminant(&c)) == Some(std::mem::discriminant(&cmd)) {
                self.params.persistence * fx(0.5)
            } else {
                Fx::ZERO
            };
            let noise = rng.gaussish() * temp;
            let habit = if hn > 0 { hq } else { Fx::ZERO };
            opts.push(Opt {
                cmd,
                act: None,
                pi: None,
                pred: [Fx::ZERO; N_OUT],
                deliberate,
                habit,
                gut: Fx::ZERO,
                curiosity: Fx::ZERO,
                persistence,
                noise,
                total: Fx::ZERO,
                episodic: None,
            });
        }

        // Arbitration between habit and deliberate control (reliability-based).
        let r_mb = (Fx::ONE - self.model_error).clamp01();
        let r_mf = self.habits.reliability();
        let w_mb = if self.caps.deliberate_planning {
            ((r_mb - r_mf) * fx(4.0) + fx(1.0) - f.body.fatigue * fx(1.5)).sigmoid()
        } else {
            Fx::ZERO
        };
        for o in opts.iter_mut() {
            o.deliberate = o.deliberate * w_mb;
            o.habit = o.habit * (Fx::ONE - w_mb);
            o.total = o.deliberate + o.habit + o.gut + o.curiosity + o.persistence + o.noise;
        }
        opts.sort_by(|a, b| b.total.cmp(&a.total).then(a.cmd.code().cmp(&b.cmd.code())));
        let mut chosen = opts[0].clone();

        // Reflex: an innate withdrawal reflex overrides everything (the body enforces it too).
        let mut reflex_text = String::from("none");
        if f.body.reflex_active {
            if let Some(t) = pending_target.or_else(|| self.pending.as_ref().and_then(|p| p.target)) {
                chosen = Opt { cmd: MotorCommand::Withdraw { target: t }, ..chosen };
                reflex_text = "WITHDRAW (innate protective reflex: skin nociceptors firing)".into();
            }
        }

        // 6. Emotions from what is now expected.
        let mut fear = Fx::ZERO;
        let mut fear_src: Option<usize> = None;
        for &i in &attended {
            let q = &ps[i];
            let prox = match q.dist {
                0 | 1 => Fx::ONE,
                2 => fx(0.6),
                3 => fx(0.3),
                _ => fx(0.1),
            };
            let pt = self.predict_action(A_TOUCH, &q.cue, ctx)[O_PAIN];
            let threat = q.pav[O_PAIN].max(pt * fx(0.5)) * prox * self.params.fear_gain;
            if threat > fear {
                fear = threat;
                fear_src = Some(i);
            }
        }
        if self.affect.fear > fx(0.2) && fear < self.affect.fear * fx(0.5) && out[O_PAIN].raw() == 0 {
            Affect::raise(&mut self.affect.relief, self.affect.fear - fear);
        }
        Affect::raise(&mut self.affect.fear, fear);
        Affect::raise(&mut self.affect.interest, chosen.curiosity * fx(2.0));

        // Goal and explanation.
        let (goal, reason) = self.explain(&chosen, &ps, f, &reflex_text);
        self.wm.goal = goal.clone();

        if self.trace_enabled {
            let tr = self.build_trace(
                f, tick, ctx, &ps, &attended, &recalls, &opts, &chosen, &reflex_text, w_mb, r_mb, r_mf, &goal, &reason,
                fear_src,
            );
            self.last_trace = Some(tr);
        }

        // Remember what was done, for learning on the next cycle.
        let pi = chosen.cmd.target().and_then(|t| ps.iter().position(|p| p.token == t));
        if let (Some(i), Some(a)) = (pi, chosen.act) {
            if a == A_INSPECT && ps[i].dist == 2 {
                self.assoc.note_attempt(A_INSPECT, &ps[i].cue);
            }
        }
        self.pending = Some(Pending {
            cmd: chosen.cmd,
            act: chosen.act.or_else(|| {
                if matches!(chosen.cmd, MotorCommand::Withdraw { .. }) {
                    Some(A_WITHDRAW)
                } else {
                    None
                }
            }),
            target: chosen.cmd.target(),
            cue: pi.map(|i| ps[i].cue).unwrap_or([Fx::ZERO; N_CUES]),
            sense: pi.map(|i| ps[i].sense).unwrap_or([Fx::ZERO; N_SENSE]),
            concept: pi.map(|i| ps[i].rec.id).unwrap_or(0),
        });
        if self.wm.last_command == Some(chosen.cmd) {
            self.repeats = self.repeats.saturating_add(1);
        } else {
            self.repeats = 0;
        }
        self.wm.last_command = Some(chosen.cmd);
        self.prev_body = f.body;
        self.stats.decisions += 1;
        chosen.cmd
    }

    #[allow(clippy::too_many_arguments)]
    fn score_option(
        &mut self,
        cmd: MotorCommand,
        a: usize,
        i: usize,
        ps: &[PState],
        recalls: &[Recall],
        util: &Out,
        ctx: usize,
        f: &SensoryFrame,
        temp: Fx,
        rng: &mut Rng,
        last_cmd: Option<MotorCommand>,
    ) -> Opt {
        let q = &ps[i];
        let value = |pred: &Out| (0..N_OUT).fold(Fx::ZERO, |s, o| s + pred[o] * util[o]);
        let mut pred = self.predict_action(a, &q.cue, ctx);
        let mut episodic = None;
        if let Some((est, w)) = self.episodic.estimate(recalls, a as u8) {
            let mix = w * fx(0.4);
            for o in 0..N_OUT {
                pred[o] = pred[o] * (Fx::ONE - mix) + est[o] * mix;
            }
            episodic = Some(format!(
                "last times: pain {:.2}, nutrient {:.2}, fluid {:.2}",
                est[O_PAIN].to_f64(),
                est[O_NOURISH].to_f64(),
                est[O_HYDRATE].to_f64()
            ));
        }
        let effort = fx(0.01);
        let mut deliberate = value(&pred) - effort;
        // Lookahead: approaching enables touching/mouthing next.
        if a == A_APPROACH && self.params.planning_depth >= 2 {
            let vt = value(&self.predict_action(A_TOUCH, &q.cue, ctx));
            let vm = value(&self.predict_action(A_MOUTH, &q.cue, ctx));
            deliberate += fx(0.8) * vt.max(vm).max(Fx::ZERO);
        }
        // Withdrawing is worth the harm one expects from staying close.
        if a == A_WITHDRAW {
            let prox = if q.dist <= 1 { Fx::ONE } else { fx(0.5) };
            deliberate = q.pav[O_PAIN] * (-util[O_PAIN]) * prox * fx(0.5) - effort;
        }
        // Epistemic value of inspecting: worth it when the outcome of touching is uncertain but could be bad.
        let ev_touch = self.assoc.evidence_for(A_TOUCH, &q.cue);
        let uncertainty = Fx::ONE / (Fx::ONE + ev_touch);
        if a == A_INSPECT {
            // Looking again at the same thing tells less and less (epistemic value habituates).
            let looked = Fx::ONE / (Fx::ONE + self.assoc.evidence_for(A_INSPECT, &q.cue) / fx(10.0)).sqrt();
            let risk = self.predict_action(A_TOUCH, &q.cue, ctx)[O_PAIN].max(q.pav[O_PAIN]);
            deliberate += risk * uncertainty * looked * fx(0.5);
        }
        // Habit (model-free cached value for this concept and action).
        let (hq, hn) = self.habits.q(q.rec.id, a as u8);
        let habit = if hn > 0 { hq } else { Fx::ZERO };
        // Gut: Pavlovian approach/avoid bias from learned cue value.
        // Approach-avoidance gradient: the pull of a learned threat grows as it gets closer.
        let k = self.params.k_pav;
        let prox = match q.dist {
            0 | 1 => Fx::ONE,
            2 => fx(0.6),
            3 => fx(0.3),
            _ => fx(0.1),
        };
        let gut = match a {
            A_APPROACH | A_TOUCH | A_MOUTH => k * q.pav_value,
            A_INSPECT => k * q.pav_value * fx(0.3),
            A_WITHDRAW => {
                // Pulling away from the thing that is hurting right now (acute pain on contact).
                let reflexive = if q.contact && f.body.acute_pain > fx(0.2) { fx(0.3) } else { Fx::ZERO };
                k * q.pav[O_PAIN] * prox + reflexive
            }
            _ => Fx::ZERO,
        };
        // Curiosity: only about outcomes my actions could change (touch, mouth, inspect).
        let c = self.params.curiosity;
        let novelty = |act: usize| Fx::ONE / (Fx::ONE + self.assoc.evidence_for(act, &q.cue)).sqrt();
        // Curiosity is discounted by expected harm: creatures do not explore what they fear.
        let harm = self.predict_action(a, &q.cue, ctx)[O_PAIN].max(q.pav[O_PAIN]);
        let safe = (Fx::ONE - harm.muli(3)).clamp01();
        let curiosity = match a {
            A_TOUCH => c * novelty(a) * fx(0.25) * safe,
            // Unmet needs make *ingesting* untried things more attractive (not touching dangerous ones).
            A_MOUTH => c * novelty(a) * fx(0.25) * self.explore_drive * safe,
            A_INSPECT => c * novelty(a) * fx(0.08),
            A_APPROACH => c * fx(0.8) * (novelty(A_TOUCH).max(novelty(A_MOUTH) * self.explore_drive) * fx(0.25)) * safe,
            _ => Fx::ZERO,
        };
        // Persistence: continue what I was doing (and finish a started reach).
        // Persistence prevents dithering but wears off with repetition (boredom).
        let mut persistence = Fx::ZERO;
        if last_cmd == Some(cmd) && f.last_result != alife_interface::MotorResult::Failed {
            let fatigue_of_repeating = (Fx::ONE - Fx::from_int(self.repeats as i64) / fx(30.0)).clamp01();
            persistence = self.params.persistence * fatigue_of_repeating;
            // Finishing a reach that has just started (contact is a two-step act).
            if matches!(cmd, MotorCommand::Touch { .. } | MotorCommand::Mouth { .. }) && q.dist <= 1 && self.repeats == 0 {
                persistence = self.params.persistence;
            }
        }
        let noise = rng.gaussish() * temp;
        Opt {
            cmd,
            act: Some(a),
            pi: Some(i),
            pred,
            deliberate,
            habit,
            gut,
            curiosity,
            persistence,
            noise,
            total: Fx::ZERO,
            episodic,
        }
    }

    /// Learning from the outcome of the previous action.
    fn learn(&mut self, f: &SensoryFrame, ps: &[PState], out: &Out, reward: Fx, ctx: usize, tick: u64) {
        let any_out = out.iter().any(|v| v.is_positive());
        let pending = self.pending.take();
        let mut max_delta = Fx::ZERO;
        if let Some(pd) = &pending {
            let tq = pd.target.and_then(|t| ps.iter().find(|q| q.token == t));
            let contact = tq.map(|q| q.contact).unwrap_or(false);
            if let Some(a) = pd.act {
                let trial = match a {
                    A_TOUCH | A_MOUTH => contact || any_out || f.body.reflex_active,
                    _ => any_out,
                };
                if trial {
                    if a == A_TOUCH || a == A_MOUTH {
                        self.assoc.learn_instrumental(&self.params, A_CONTACT, &pd.cue, out, ctx, Fx::ONE, &AVERSIVE);
                    }
                    let rep = self.assoc.learn_instrumental(&self.params, a, &pd.cue, out, ctx, Fx::ONE, &AVERSIVE);
                    self.stats.instrumental_trials += 1;
                    let err = rep.delta.iter().fold(Fx::ZERO, |m, d| m.max(d.abs())).min(Fx::ONE);
                    max_delta = max_delta.max(err);
                    self.model_error = self.model_error * fx(0.9) + err * fx(0.1);
                    self.lp_fast[a] = self.lp_fast[a] * fx(0.7) + err * fx(0.3);
                    self.lp_slow[a] = self.lp_slow[a] * fx(0.95) + err * fx(0.05);
                    self.habits.update(pd.concept, a as u8, reward, self.params.eta_habit, tick);
                    let desc = describe(&pd.sense, 4).join(", ");
                    self.last_learning.push(format!(
                        "{} [{}]: expected pain {:.2} nutrient {:.2} fluid {:.2}; got {:.2} / {:.2} / {:.2}; surprise {:.2}; learning rate {:.2}",
                        action_label(a),
                        desc,
                        rep.predicted[O_PAIN].to_f64(),
                        rep.predicted[O_NOURISH].to_f64(),
                        rep.predicted[O_HYDRATE].to_f64(),
                        out[O_PAIN].to_f64(),
                        out[O_NOURISH].to_f64(),
                        out[O_HYDRATE].to_f64(),
                        err.to_f64(),
                        rep.rate.to_f64()
                    ));
                    if self.caps.episodic_memory {
                        let novel_pair = self.assoc.evidence_for(a, &pd.cue) < fx(2.0);
                        if err > fx(0.3) || out[O_PAIN] > fx(0.2) || (contact && novel_pair) {
                            let conseq = tq.map(|q| q.conseq).unwrap_or([Fx::ZERO; N_CONSEQ]);
                            self.store_episode(tick, ctx, pd, a as u8, out, err, conseq);
                        }
                    }
                }
            }
        }
        // Second-order conditioning: approaching or reaching for something
        // produced a *new heat sensation* that predicts pain (01 rule 3). Only
        // the part of the prediction carried by the new sensation counts, so
        // sensory noise cannot ratchet fear upward.
        if let Some(pd) = &pending {
            let acting_on_it = matches!(pd.act, Some(A_APPROACH) | Some(A_TOUCH) | Some(A_MOUTH));
            if let (true, Some(tq)) = (acting_on_it, pd.target.and_then(|t| ps.iter().find(|q| q.token == t))) {
                if out[O_PAIN].raw() == 0 {
                    let warm = encode::U_WARM_LIN..encode::U_WARM0 + encode::N_WARM;
                    let heat_part = |x: &CueVec| {
                        let mut h = [Fx::ZERO; N_CUES];
                        for u in warm.clone() {
                            h[u] = x[u];
                        }
                        self.assoc.predict_pav(&h, ctx)[O_PAIN]
                    };
                    let now = self.assoc.predict_pav(&tq.cue, ctx)[O_PAIN];
                    if heat_part(&tq.cue) > heat_part(&pd.cue) + fx(0.05) {
                        // Chain only from what was perceivable *before* acting (vision).
                        let mut seen = pd.cue;
                        for u in encode::U_WARM_LIN..encode::U_WARM0 + encode::N_WARM {
                            seen[u] = Fx::ZERO;
                        }
                        let d = self.assoc.learn_td(&self.params, &seen, ctx, O_PAIN, now);
                        if d.is_positive() {
                            self.last_learning.push(format!(
                                "second-order: [{}] now predicts pain via what followed it (target {:.2}, step {:.2})",
                                describe(&pd.sense, 4).join(", "),
                                (now * fx(0.8)).to_f64(),
                                d.to_f64()
                            ));
                        }
                    }
                }
            }
        }
        if any_out {
            // Pavlovian: whatever was in mind when it happened gets some of the credit.
            let mut cues: Vec<(CueVec, Fx)> = Vec::new();
            if let Some(pd) = &pending {
                if pd.target.is_some() {
                    cues.push((pd.cue, Fx::ONE));
                }
            }
            // Bystanders get little credit when I know what I just acted on (causal attribution).
            let acted = pending.as_ref().and_then(|p| p.target).is_some();
            let bystander = if acted { fx(0.05) } else { fx(0.2) };
            for q in ps {
                if self.wm.holds(q.token) && Some(q.token) != pending.as_ref().and_then(|p| p.target) {
                    cues.push((q.cue, bystander));
                }
            }
            if !cues.is_empty() {
                let rep = self.assoc.learn_pavlovian(&self.params, &cues, out, ctx, &AVERSIVE);
                self.stats.pavlovian_events += 1;
                let err = rep.delta.iter().fold(Fx::ZERO, |m, d| m.max(d.abs())).min(Fx::ONE);
                max_delta = max_delta.max(err);
                self.last_learning.push(format!(
                    "pavlovian: things in mind predicted pain {:.2}, got {:.2}; {} cue weights updated",
                    rep.predicted[O_PAIN].to_f64(),
                    out[O_PAIN].to_f64(),
                    rep.changes.len()
                ));
            }
        }
        Affect::raise(&mut self.affect.surprise, max_delta);
    }

    #[allow(clippy::too_many_arguments)]
    fn store_episode(&mut self, tick: u64, ctx: usize, pd: &Pending, action: u8, out: &Out, surprise: Fx, conseq: ConseqVec) {
        let mut feats: Vec<(u8, Fx)> =
            (0..N_SENSE).filter(|&u| pd.sense[u] > fx(0.25)).map(|u| (u as u8, pd.sense[u])).collect();
        feats.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        feats.truncate(10);
        let mut cs: Vec<(u8, Fx)> = (0..N_CONSEQ).filter(|&u| conseq[u] > fx(0.25)).map(|u| (u as u8, conseq[u])).collect();
        cs.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        cs.truncate(4);
        let valence = (out[O_NOURISH] + out[O_HYDRATE] - out[O_PAIN]).clamp(Fx::NEG_ONE, Fx::ONE);
        let arousal = self.affect.fear.max(self.affect.distress).max(surprise).max(out[O_PAIN]);
        let importance = (fx(0.3) * arousal + fx(0.15) * valence.abs() + fx(0.2) * surprise + fx(0.1)).clamp01();
        self.episodic.store(Episode {
            id: 0,
            tick,
            ctx: ctx as u8,
            concept: pd.concept,
            features: feats,
            conseq: cs,
            action,
            outcome: *out,
            surprise,
            valence,
            arousal,
            importance,
            recalls: 0,
            last_recall: tick,
            pinned: false,
        });
        self.stats.episodes_stored += 1;
    }

    /// Sleep: replay, consolidation, forgetting, belief crystallisation.
    fn sleep(&mut self, tick: u64) {
        let replay = self.episodic.replay_order(self.last_consolidation, 12);
        let mut cues: Vec<usize> = Vec::new();
        for id in &replay {
            if let Some(e) = self.episodic.get(*id).cloned() {
                for &(u, a) in &e.features {
                    if a > fx(0.3) && !cues.contains(&(u as usize)) {
                        cues.push(u as usize);
                    }
                }
                if let Some(slot) = self.concept_slot(e.concept) {
                    let c = N_SENSE + slot;
                    if !cues.contains(&c) {
                        cues.push(c);
                    }
                }
            }
            self.episodic.rehearse(*id, tick);
        }
        let days = Fx::from_int(tick.saturating_sub(self.last_consolidation) as i64) / fx(86_400.0);
        let mut rep = self.assoc.consolidate(&self.params, &cues, days.max(fx(0.1)));
        rep.episodes_replayed = replay.len() as u32;
        if self.caps.semantic_beliefs {
            let ep = &self.episodic;
            let concepts = &self.concepts;
            rep.beliefs_crystallised = self.semantic.crystallise(&self.assoc, tick, |cue| {
                let cue = cue as usize;
                ep.all()
                    .filter(|e| {
                        if cue < N_SENSE {
                            e.features.iter().any(|&(u, a)| u as usize == cue && a > fx(0.3))
                        } else {
                            concepts.concepts.get(cue - N_SENSE).map(|c| c.id) == Some(e.concept)
                        }
                    })
                    .map(|e| e.id)
                    .take(4)
                    .collect()
            });
        }
        self.last_consolidation = tick;
        self.stats.consolidations += 1;
        self.last_learning.push(format!(
            "SLEEP CONSOLIDATION: replayed {} episodes, consolidated {} cues, {} new flashbulb memories, {} beliefs crystallised/updated",
            rep.episodes_replayed, rep.cues_consolidated, rep.flashbulbs, rep.beliefs_crystallised
        ));
    }

    fn explain(&self, chosen: &Opt, ps: &[PState], f: &SensoryFrame, reflex: &str) -> (Goal, String) {
        if reflex != "none" {
            let t = chosen.cmd.target().expect("withdraw has a target");
            return (Goal::Avoid(t), "pain reflex pulled away from what was touched".into());
        }
        let label = self.cmd_label(&chosen.cmd, ps);
        let terms = [
            ("expected outcome (deliberate)", chosen.deliberate),
            ("habit", chosen.habit),
            ("gut feeling", chosen.gut),
            ("curiosity", chosen.curiosity),
            ("continuing what I was doing", chosen.persistence),
        ];
        let (dom, dv) = terms.iter().copied().max_by_key(|(_, v)| v.abs().raw()).expect("non-empty");
        let target = chosen.cmd.target();
        let goal = match (chosen.cmd, target) {
            (MotorCommand::Rest, _) => Goal::Rest,
            (MotorCommand::Wander { .. }, _) => Goal::Wander,
            (MotorCommand::Withdraw { target }, _) => Goal::Avoid(target),
            (_, Some(t)) => {
                if chosen.pred[O_NOURISH] > fx(0.1) && f.body.hunger > fx(0.2) {
                    Goal::Relieve(0, t)
                } else if chosen.pred[O_HYDRATE] > fx(0.1) && f.body.thirst > fx(0.2) {
                    Goal::Relieve(1, t)
                } else if chosen.gut < fx(-0.05) || chosen.pred[O_PAIN] > fx(0.2) {
                    Goal::Avoid(t)
                } else {
                    Goal::Explore(t)
                }
            }
            _ => Goal::None,
        };
        let mut why = format!("{label}: strongest reason = {dom} ({:+.2})", dv.to_f64());
        if chosen.pred[O_PAIN] > fx(0.1) {
            why.push_str(&format!("; expects pain {:.2}", chosen.pred[O_PAIN].to_f64()));
        }
        if let Some(e) = &chosen.episodic {
            why.push_str(&format!("; memory: {e}"));
        }
        (goal, why)
    }

    #[allow(clippy::too_many_arguments)]
    fn build_trace(
        &self,
        f: &SensoryFrame,
        tick: u64,
        ctx: usize,
        ps: &[PState],
        attended: &[usize],
        recalls: &[(usize, Vec<Recall>)],
        opts: &[Opt],
        chosen: &Opt,
        reflex: &str,
        w_mb: Fx,
        r_mb: Fx,
        r_mf: Fx,
        goal: &Goal,
        reason: &str,
        fear_src: Option<usize>,
    ) -> TraceRecord {
        let mut tr = TraceRecord { tick, context: ctx as u8, ..Default::default() };
        let mut order: Vec<usize> = (0..ps.len()).collect();
        order.sort_by(|&a, &b| ps[a].dist.cmp(&ps[b].dist).then(ps[a].token.cmp(&ps[b].token)));
        for &i in &order {
            let q = &ps[i];
            let contact: Vec<String> = (0..N_CONSEQ)
                .filter(|&u| q.conseq[u] > fx(0.3))
                .map(|u| encode::conseq_label(u).to_string())
                .collect();
            tr.perception.push(PerceptLine {
                label: Self::token_label(q.token),
                dist: q.dist,
                features: describe(&q.sense, 6),
                warmth: q.sense[encode::U_WARM_LIN],
                concept: q.rec.id,
                similarity: q.rec.similarity,
                novel_concept: q.rec.novel,
                contact,
                gut_pain: q.pav[O_PAIN],
                gut_value: q.pav_value,
            });
        }
        for s in &self.wm.slots {
            let label = match s.kind {
                ItemKind::Percept(t) => {
                    let q = ps.iter().find(|p| p.token == t).expect("attended percept exists");
                    format!("{} (C{}): {}", Self::token_label(t), q.rec.id, describe(&q.sense, 3).join(", "))
                }
                ItemKind::Need(n) => format!("need: {}", need_label(n)),
                ItemKind::Memory(m) => format!("memory: episode #{m}"),
            };
            tr.attention.push((label, s.salience));
        }
        let goal_text = match goal {
            Goal::None => "none".to_string(),
            Goal::Explore(t) => format!("find out about {}", Self::token_label(*t)),
            Goal::Relieve(n, t) => format!("relieve {} using {}", need_label(*n), Self::token_label(*t)),
            Goal::Avoid(t) => format!("avoid harm from {}", Self::token_label(*t)),
            Goal::Rest => "rest".into(),
            Goal::Wander => "wander / look around".into(),
        };
        tr.working_memory.push(format!("goal: {goal_text}"));
        if let Some(c) = self.wm.last_command {
            tr.working_memory.push(format!("previous action: {}", self.cmd_label(&c, ps)));
        }
        tr.working_memory.push(format!("workspace capacity: {} items", self.wm.capacity));
        for (_, rs) in recalls {
            for r in rs.iter().take(2) {
                if tr.memories.iter().any(|m| m.episode == r.episode) {
                    continue;
                }
                if let Some(e) = self.episodic.get(r.episode) {
                    tr.memories.push(MemoryLine {
                        episode: e.id,
                        summary: self.episode_summary(e),
                        activation: r.activation,
                        age_s: tick.saturating_sub(e.tick),
                        pinned: e.pinned,
                    });
                }
            }
        }
        // Associations for the chosen target (or the most feared thing).
        let focus = chosen.pi.or(fear_src).or_else(|| attended.first().copied());
        if let Some(i) = focus {
            let x = &ps[i].cue;
            let mut lines: Vec<AssocLine> = Vec::new();
            for c in 0..N_CUES {
                if x[c] < fx(0.3) {
                    continue;
                }
                for o in 0..N_OUT {
                    let wp = self.assoc.pav_net(c, o, ctx);
                    if wp.abs() > fx(0.03) {
                        let raw = self.assoc.pav_fast[c][o] + self.assoc.pav_slow[c][o];
                        lines.push(AssocLine {
                            cue: self.cue_label(c),
                            action: None,
                            outcome: outcome_label(o),
                            weight: wp,
                            extinction: (raw - wp).max(Fx::ZERO),
                            evidence: self.assoc.pav_evidence[c],
                            activation: x[c],
                        });
                    }
                    for a in [A_TOUCH, A_MOUTH] {
                        let w = self.assoc.inst_net(a, c, o, ctx);
                        if w.abs() > fx(0.03) {
                            let raw = self.assoc.inst_fast[a * N_CUES + c][o] + self.assoc.inst_slow[a * N_CUES + c][o];
                            lines.push(AssocLine {
                                cue: self.cue_label(c),
                                action: Some(action_label(a)),
                                outcome: outcome_label(o),
                                weight: w,
                                extinction: (raw - w).max(Fx::ZERO),
                                evidence: self.assoc.evidence[a * N_CUES + c],
                                activation: x[c],
                            });
                        }
                    }
                }
            }
            lines.sort_by(|a, b| (b.weight.abs() * b.activation).cmp(&(a.weight.abs() * a.activation)).then(a.cue.cmp(&b.cue)));
            lines.truncate(14);
            tr.associations = lines;
            for b in self.semantic.relevant(x).into_iter().take(4) {
                tr.beliefs.push(format!(
                    "\"{}{} -> {}\"  confidence {:.2}, {} trials, source: {:?}, evidence episodes {:?}",
                    b.action.map(|a| format!("{} + ", action_label(a as usize))).unwrap_or_default(),
                    self.cue_label(b.cue as usize),
                    outcome_label(b.outcome as usize),
                    b.confidence.to_f64(),
                    b.evidence,
                    b.source,
                    b.evidence_episodes
                ));
            }
        }
        tr.emotions = self.affect.list();
        tr.mood = (self.affect.pleasure, self.affect.arousal, self.affect.dominance);
        tr.needs = vec![
            ("hunger", f.body.hunger),
            ("thirst", f.body.thirst),
            ("pain", f.body.pain),
            ("fatigue", f.body.fatigue),
        ];
        for o in opts {
            tr.options.push(OptionLine {
                label: self.cmd_label(&o.cmd, ps),
                total: o.total,
                deliberate: o.deliberate,
                habit: o.habit,
                gut: o.gut,
                curiosity: o.curiosity,
                persistence: o.persistence,
                noise: o.noise,
                pred_pain: o.pred[O_PAIN],
                pred_nourish: o.pred[O_NOURISH],
                pred_hydrate: o.pred[O_HYDRATE],
                episodic: o.episodic.clone(),
            });
        }
        let best_by = |key: &dyn Fn(&Opt) -> Fx| -> String {
            opts.iter()
                .max_by(|a, b| key(a).cmp(&key(b)).then(b.cmd.code().cmp(&a.cmd.code())))
                .filter(|o| key(o).abs() > fx(0.02))
                .map(|o| format!("{} ({:+.2})", self.cmd_label(&o.cmd, ps), key(o).to_f64()))
                .unwrap_or_else(|| "no preference".into())
        };
        tr.reflex = if reflex != "none" { reflex.to_string() } else { best_by(&|o: &Opt| o.gut) };
        tr.habit = if self.habits.entries.is_empty() { "no habits yet".into() } else { best_by(&|o: &Opt| o.habit) };
        tr.deliberate = best_by(&|o: &Opt| o.deliberate + o.curiosity);
        tr.w_deliberate = w_mb;
        tr.reliability_deliberate = r_mb;
        tr.reliability_habit = r_mf;
        tr.decision = self.cmd_label(&chosen.cmd, ps);
        tr.goal = goal_text;
        tr.reason = reason.to_string();
        tr.learning = self.last_learning.clone();
        tr
    }

    pub fn episode_summary(&self, e: &Episode) -> String {
        let act = if (e.action as usize) < N_ACT { action_label(e.action as usize) } else { "noticed" };
        let feats: Vec<String> = e.features.iter().take(5).map(|&(u, _)| encode::sense_label(u as usize)).collect();
        let mut outs: Vec<String> = Vec::new();
        for o in 0..N_OUT {
            if e.outcome[o] > fx(0.05) {
                outs.push(format!("{} {:.2}", outcome_label(o), e.outcome[o].to_f64()));
            }
        }
        if outs.is_empty() {
            outs.push("nothing happened".into());
        }
        let cons: Vec<&str> = e.conseq.iter().map(|&(u, _)| encode::conseq_label(u as usize)).collect();
        format!(
            "{} C{} [{}]{} -> {}",
            act,
            e.concept,
            feats.join(", "),
            if cons.is_empty() { String::new() } else { format!(" felt {}", cons.join("+")) },
            outs.join(", ")
        )
    }

    /// Diagnostic read-out: what this mind predicts for an action on a cue
    /// pattern, in a given context. Used by experiments; never by the world.
    pub fn predict(&self, a: usize, cue: &CueVec, ctx: usize) -> Out {
        self.assoc.predict_inst(a, cue, ctx)
    }

    /// Diagnostic, non-mutating appraisal of a frame: for each percept, what the
    /// gut (Pavlovian) and the outcome model predict for touching and mouthing.
    /// Used by experiments to read the mind; it does not change any state.
    pub fn appraise(&self, f: &SensoryFrame) -> Vec<(Token, i32, Out, Out, Out, u16)> {
        let v = [f.ambient.light, f.ambient.ground, f.ambient.enclosure, f.ambient.sound];
        let ctx = self
            .contexts
            .protos
            .iter()
            .enumerate()
            .map(|(i, p)| (i, p.iter().zip(v.iter()).fold(Fx::ZERO, |m, (x, y)| m.max((*x - *y).abs()))))
            .min_by_key(|(_, d)| d.raw())
            .map(|(i, _)| i)
            .unwrap_or(0);
        f.percepts
            .iter()
            .map(|p| {
                let sense = encode(p);
                let (slot, sim, id) = match self.concepts.peek(&sense) {
                    Some((slot, s)) if s >= self.concepts.vigilance => (Some(slot), s, self.concepts.concepts[slot].id),
                    _ => (None, Fx::ZERO, 0),
                };
                let mut x = [Fx::ZERO; N_CUES];
                x[..N_SENSE].copy_from_slice(&sense);
                if let Some(slot) = slot {
                    x[N_SENSE + slot] = sim;
                }
                (
                    p.token,
                    p.dist,
                    self.assoc.predict_pav(&x, ctx),
                    self.predict_action(A_TOUCH, &x, ctx),
                    self.predict_action(A_MOUTH, &x, ctx),
                    id,
                )
            })
            .collect()
    }

    /// Approximate memory footprint of this mind, in bytes.
    pub fn memory_bytes(&self) -> usize {
        let out = std::mem::size_of::<Out>();
        let a = &self.assoc;
        let assoc = (a.pav_fast.len() + a.pav_slow.len() + a.inst_fast.len() + a.inst_slow.len() + a.pav_ext.len() + a.inst_ext.len()) * out
            + a.alpha.len() * 8
            + (a.evidence.len() + a.pav_evidence.len() + a.rehearsals.len()) * 2
            + a.tag.len() * 8
            + a.flashbulb.len();
        let concepts = self.concepts.concepts.len() * std::mem::size_of::<concepts::Concept>();
        let episodes = self.episodic.all().map(|e| std::mem::size_of::<Episode>() + (e.features.len() + e.conseq.len()) * 16).sum::<usize>();
        let beliefs = self.semantic.beliefs.len() * std::mem::size_of::<memory::semantic::Belief>();
        let habits = self.habits.entries.len() * std::mem::size_of::<learning::habit::HabitEntry>();
        std::mem::size_of::<Mind>() + assoc + concepts + episodes + beliefs + habits
    }
}

impl StableHash for Mind {
    fn stable_hash(&self, h: &mut StateHasher) {
        self.id.stable_hash(h);
        self.personality.stable_hash(h);
        self.concepts.stable_hash(h);
        h.u64(self.contexts.protos.len() as u64);
        for p in &self.contexts.protos {
            p.stable_hash(h);
        }
        self.assoc.stable_hash(h);
        self.habits.stable_hash(h);
        self.episodic.stable_hash(h);
        self.semantic.stable_hash(h);
        self.wm.stable_hash(h);
        self.affect.stable_hash(h);
        self.model_error.stable_hash(h);
        self.lp_fast.stable_hash(h);
        self.lp_slow.stable_hash(h);
        self.was_asleep.stable_hash(h);
        self.last_consolidation.stable_hash(h);
    }
}

// Unused-import guard for constants referenced only in some builds.
const _: (usize, usize) = (A_INSPECT, H_REST as usize);
