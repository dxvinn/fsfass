//! Associative memory and its learning rules (report 01 §8.2).
//!
//! Two kinds of association, each with a fast and a slow weight:
//! * Pavlovian: cue -> outcome ("this kind of sight predicts pain"),
//! * instrumental: (action | cue) -> outcome ("touching this predicts pain").
//!
//! Cues are sense units plus learned concept units. Outcomes are body-signal
//! changes the mind experiences: pain onset, hunger relief, thirst relief.
//!
//! Rules: normalised Rescorla–Wagner with a summed prediction (cue
//! competition), Pearce–Hall associability (surprise raises attention),
//! negativity bias, context-gated extinction (omission writes to a separate
//! per-context store instead of erasing), two-timescale consolidation during
//! sleep, and slow forgetting that rehearsal and flashbulb tagging resist.

use crate::encode::N_SENSE;
use crate::concepts::MAX_CONCEPTS;
use crate::params::MindParams;
use alife_core::hash::{StableHash, StateHasher};
use alife_core::{fx, Fx};

pub const N_CUES: usize = N_SENSE + MAX_CONCEPTS;
pub const N_OUT: usize = 3;
pub const O_PAIN: usize = 0;
pub const O_NOURISH: usize = 1;
pub const O_HYDRATE: usize = 2;
pub const N_ACT: usize = 6;
pub const A_APPROACH: usize = 0;
pub const A_INSPECT: usize = 1;
pub const A_TOUCH: usize = 2;
pub const A_MOUTH: usize = 3;
pub const A_WITHDRAW: usize = 4;
/// Action family "make contact": learns from both touching and mouthing, so
/// what one contact act taught generalises to the other (hand vs face).
pub const A_CONTACT: usize = 5;

/// Outcome salience (the beta of Rescorla–Wagner): pain is maximally salient.
pub fn outcome_salience(o: usize) -> Fx {
    [fx(1.0), fx(0.6), fx(0.6)][o]
}
pub const N_CTX: usize = 8;

pub type CueVec = [Fx; N_CUES];
pub type Out = [Fx; N_OUT];

/// Salience of a cue: sense units have innate salience; learned concepts
/// (configural units) are fully salient.
pub fn cue_salience(c: usize) -> Fx {
    if c < N_SENSE {
        crate::encode::sense_salience(c)
    } else {
        Fx::ONE
    }
}

/// Salience-weighted squared norm: the normaliser of the update rule, so a
/// single trial moves the summed prediction by exactly rate * delta, with
/// credit shared in proportion to salience * activation^2.
fn snorm2(x: &CueVec) -> Fx {
    let mut s = Fx::ZERO;
    for c in 0..N_CUES {
        if x[c].raw() != 0 {
            s += cue_salience(c) * x[c] * x[c];
        }
    }
    s.max(fx(0.05))
}

pub fn outcome_label(o: usize) -> &'static str {
    ["pain", "nutrient-intake", "fluid-intake"][o]
}

pub fn action_label(a: usize) -> &'static str {
    ["approach", "inspect", "touch", "mouth", "withdraw", "contact"][a]
}

#[derive(Clone, Debug, PartialEq)]
pub struct Assoc {
    pub pav_fast: Vec<Out>,
    pub pav_slow: Vec<Out>,
    pub inst_fast: Vec<Out>,
    pub inst_slow: Vec<Out>,
    /// Pavlovian extinction memories, per (cue, context).
    pub pav_ext: Vec<Out>,
    /// Instrumental extinction memories, per (action, cue, context).
    pub inst_ext: Vec<Out>,
    /// Pearce–Hall associability per cue (novel cues start at 1).
    pub alpha: Vec<Fx>,
    /// How many times each (action, cue) has been tried (evidence).
    pub evidence: Vec<u16>,
    pub pav_evidence: Vec<u16>,
    /// Consolidation tag per cue (high = surprising, intense learning; resists forgetting).
    pub tag: Vec<Fx>,
    pub flashbulb: Vec<bool>,
    pub rehearsals: Vec<u16>,
}

/// One weight change, for traces and the association-graph view.
#[derive(Clone, Debug, PartialEq)]
pub struct WeightChange {
    pub kind: &'static str,
    pub action: Option<usize>,
    pub cue: usize,
    pub outcome: usize,
    pub before: Fx,
    pub after: Fx,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LearnReport {
    pub predicted: Out,
    pub actual: Out,
    pub delta: Out,
    pub rate: Fx,
    pub changes: Vec<WeightChange>,
}

#[inline]
fn ai(a: usize, c: usize) -> usize {
    a * N_CUES + c
}
#[inline]
fn ci(c: usize, ctx: usize) -> usize {
    c * N_CTX + ctx
}
#[inline]
fn aci(a: usize, c: usize, ctx: usize) -> usize {
    (a * N_CUES + c) * N_CTX + ctx
}

fn zero_out() -> Out {
    [Fx::ZERO; N_OUT]
}

impl Default for Assoc {
    fn default() -> Self {
        Self::new()
    }
}

impl Assoc {
    pub fn new() -> Self {
        Assoc {
            pav_fast: vec![zero_out(); N_CUES],
            pav_slow: vec![zero_out(); N_CUES],
            inst_fast: vec![zero_out(); N_ACT * N_CUES],
            inst_slow: vec![zero_out(); N_ACT * N_CUES],
            pav_ext: vec![zero_out(); N_CUES * N_CTX],
            inst_ext: vec![zero_out(); N_ACT * N_CUES * N_CTX],
            alpha: vec![Fx::ONE; N_CUES],
            evidence: vec![0; N_ACT * N_CUES],
            pav_evidence: vec![0; N_CUES],
            tag: vec![Fx::ZERO; N_CUES],
            flashbulb: vec![false; N_CUES],
            rehearsals: vec![0; N_CUES],
        }
    }

    /// Forget everything about one cue (used when a concept slot is recycled).
    pub fn clear_cue(&mut self, c: usize) {
        self.pav_fast[c] = zero_out();
        self.pav_slow[c] = zero_out();
        self.alpha[c] = Fx::ONE;
        self.pav_evidence[c] = 0;
        self.tag[c] = Fx::ZERO;
        self.flashbulb[c] = false;
        self.rehearsals[c] = 0;
        for ctx in 0..N_CTX {
            self.pav_ext[ci(c, ctx)] = zero_out();
        }
        for a in 0..N_ACT {
            self.inst_fast[ai(a, c)] = zero_out();
            self.inst_slow[ai(a, c)] = zero_out();
            self.evidence[ai(a, c)] = 0;
            for ctx in 0..N_CTX {
                self.inst_ext[aci(a, c, ctx)] = zero_out();
            }
        }
    }

    /// Net Pavlovian weight: acquisition minus context-specific extinction.
    pub fn pav_net(&self, c: usize, o: usize, ctx: usize) -> Fx {
        let w = self.pav_fast[c][o] + self.pav_slow[c][o];
        if w.is_positive() {
            (w - self.pav_ext[ci(c, ctx)][o]).max(Fx::ZERO)
        } else {
            w
        }
    }

    pub fn inst_net(&self, a: usize, c: usize, o: usize, ctx: usize) -> Fx {
        let w = self.inst_fast[ai(a, c)][o] + self.inst_slow[ai(a, c)][o];
        if w.is_positive() {
            (w - self.inst_ext[aci(a, c, ctx)][o]).max(Fx::ZERO)
        } else {
            w
        }
    }

    pub fn predict_pav(&self, x: &CueVec, ctx: usize) -> Out {
        let mut p = zero_out();
        for c in 0..N_CUES {
            if x[c].raw() == 0 {
                continue;
            }
            for (o, po) in p.iter_mut().enumerate() {
                *po += x[c] * self.pav_net(c, o, ctx);
            }
        }
        p
    }

    pub fn predict_inst(&self, a: usize, x: &CueVec, ctx: usize) -> Out {
        let mut p = zero_out();
        for c in 0..N_CUES {
            if x[c].raw() == 0 {
                continue;
            }
            for (o, po) in p.iter_mut().enumerate() {
                *po += x[c] * self.inst_net(a, c, o, ctx);
            }
        }
        p
    }

    /// Evidence for (action, cue pattern): activation-weighted trial count.
    pub fn evidence_for(&self, a: usize, x: &CueVec) -> Fx {
        let mut num = Fx::ZERO;
        let mut den = Fx::ZERO;
        for c in 0..N_CUES {
            if x[c] > fx(0.2) {
                num += x[c] * Fx::from_int(self.evidence[ai(a, c)] as i64);
                den += x[c];
            }
        }
        if den.raw() == 0 {
            Fx::ZERO
        } else {
            num / den
        }
    }

    /// Record that a non-contact action (inspect, approach, withdraw) was
    /// carried out on something: repeated looking yields less new information.
    pub fn note_attempt(&mut self, a: usize, x: &CueVec) {
        for c in 0..N_CUES {
            if x[c] > fx(0.2) {
                let e = &mut self.evidence[ai(a, c)];
                *e = e.saturating_add(1);
            }
        }
    }

    /// Mean associability of the active cues.
    fn mean_alpha(&self, x: &CueVec) -> Fx {
        let mut num = Fx::ZERO;
        let mut den = Fx::ZERO;
        for c in 0..N_CUES {
            if x[c].is_positive() {
                num += x[c] * self.alpha[c];
                den += x[c];
            }
        }
        if den.raw() == 0 {
            Fx::ONE
        } else {
            num / den
        }
    }

    /// Learning rate: age base rate x cue associability (Pearce–Hall, with a
    /// floor so salient cues stay learnable) x valence gain (negativity bias)
    /// x outcome salience (beta) x attention.
    fn rate(&self, p: &MindParams, x: &CueVec, delta: Fx, attention: Fx, o: usize) -> Fx {
        let kappa = if delta.is_positive() { p.kappa_pos } else { p.kappa_neg };
        let assoc = fx(0.5) + fx(0.5) * self.mean_alpha(x);
        (p.alpha * assoc * kappa * outcome_salience(o) * attention).clamp(Fx::ZERO, fx(0.9))
    }

    /// Instrumental learning after `action` on a target with cue vector `x`.
    /// `aversive[o]` marks outcomes whose occurrence is aversive (gets kappa_neg).
    pub fn learn_instrumental(
        &mut self,
        p: &MindParams,
        action: usize,
        x: &CueVec,
        actual: &Out,
        ctx: usize,
        attention: Fx,
        aversive: &[bool; N_OUT],
    ) -> LearnReport {
        let predicted = self.predict_inst(action, x, ctx);
        let n2 = snorm2(x);
        let mut rep = LearnReport { predicted, actual: *actual, ..Default::default() };
        for o in 0..N_OUT {
            let delta = actual[o] - predicted[o];
            rep.delta[o] = delta;
            if delta.abs() < fx(0.02) {
                continue;
            }
            let signed = if aversive[o] { delta } else { -delta };
            let rate = self.rate(p, x, -signed, attention, o);
            if delta.is_positive() {
                rep.rate = rep.rate.max(rate);
                for c in 0..N_CUES {
                    if x[c].raw() == 0 {
                        continue;
                    }
                    let before = self.inst_net(action, c, o, ctx);
                    let dw = rate * delta * cue_salience(c) * x[c] / n2;
                    self.inst_fast[ai(action, c)][o] += dw;
                    // Re-acquisition also weakens the extinction memory for this context.
                    let e = &mut self.inst_ext[aci(action, c, ctx)][o];
                    *e = (*e - dw).max(Fx::ZERO);
                    self.push_change(&mut rep, "instrumental", Some(action), c, o, before, x[c]);
                }
            } else {
                // Omission: learn a context-bound "not here / not now" memory instead of erasing.
                let rate = (p.alpha_ext * self.mean_alpha(x) * attention).clamp(Fx::ZERO, fx(0.9));
                rep.rate = rep.rate.max(rate);
                for c in 0..N_CUES {
                    if x[c].raw() == 0 {
                        continue;
                    }
                    let w = self.inst_fast[ai(action, c)][o] + self.inst_slow[ai(action, c)][o];
                    if !w.is_positive() {
                        continue;
                    }
                    let before = self.inst_net(action, c, o, ctx);
                    let de = rate * (-delta) * cue_salience(c) * x[c] / n2;
                    let e = &mut self.inst_ext[aci(action, c, ctx)][o];
                    *e = (*e + de).min(w);
                    self.push_change(&mut rep, "extinction", Some(action), c, o, before, x[c]);
                }
            }
        }
        self.update_alpha(x, &rep.delta);
        for c in 0..N_CUES {
            if x[c] > fx(0.2) {
                let e = &mut self.evidence[ai(action, c)];
                *e = e.saturating_add(1);
            }
        }
        self.tag_cues(x, &rep.delta, actual);
        rep
    }

    /// Pavlovian learning. `cues` = (cue vector, eligibility) for each thing in mind.
    pub fn learn_pavlovian(
        &mut self,
        p: &MindParams,
        cues: &[(CueVec, Fx)],
        actual: &Out,
        ctx: usize,
        aversive: &[bool; N_OUT],
    ) -> LearnReport {
        // Summed prediction over everything present (cue competition).
        let mut predicted = zero_out();
        for (x, e) in cues {
            let px = self.predict_pav(x, ctx);
            for o in 0..N_OUT {
                predicted[o] += px[o] * *e;
            }
        }
        let mut rep = LearnReport { predicted, actual: *actual, ..Default::default() };
        for o in 0..N_OUT {
            let delta = actual[o] - predicted[o];
            rep.delta[o] = delta;
            if delta.abs() < fx(0.02) || !delta.is_positive() {
                continue; // omissions are handled by exposure trials
            }
            for (x, e) in cues {
                if e.raw() == 0 {
                    continue;
                }
                let n2 = snorm2(x);
                let rate = self.rate(p, x, if aversive[o] { -delta } else { delta }, *e, o);
                rep.rate = rep.rate.max(rate);
                for c in 0..N_CUES {
                    if x[c].raw() == 0 {
                        continue;
                    }
                    let before = self.pav_net(c, o, ctx);
                    let dw = rate * delta * cue_salience(c) * x[c] / n2;
                    self.pav_fast[c][o] += dw;
                    let ex = &mut self.pav_ext[ci(c, ctx)][o];
                    *ex = (*ex - dw).max(Fx::ZERO);
                    self.push_change(&mut rep, "pavlovian", None, c, o, before, x[c]);
                }
            }
        }
        for (x, _) in cues {
            self.update_alpha(x, &rep.delta);
            for c in 0..N_CUES {
                if x[c] > fx(0.2) {
                    self.pav_evidence[c] = self.pav_evidence[c].saturating_add(1);
                }
            }
            self.tag_cues(x, &rep.delta, actual);
        }
        rep
    }

    /// Temporal-difference chaining (second-order conditioning, 01 rule 3):
    /// cues `x` were followed by a situation that itself predicts outcome `o`
    /// with strength `next_value`; move x's prediction toward gamma * next_value.
    /// This is how the look of a thing comes to predict pain through the heat it
    /// radiates, without the pain ever happening.
    pub fn learn_td(&mut self, p: &MindParams, x: &CueVec, ctx: usize, o: usize, next_value: Fx) -> Fx {
        let gamma = fx(0.8);
        let pred = self.predict_pav(x, ctx)[o];
        let delta = gamma * next_value - pred;
        if delta < fx(0.02) {
            return Fx::ZERO;
        }
        let n2 = snorm2(x);
        let rate = (p.alpha * fx(0.5) * outcome_salience(o)).clamp(Fx::ZERO, fx(0.9));
        for c in 0..N_CUES {
            if x[c].raw() == 0 {
                continue;
            }
            self.pav_fast[c][o] += rate * delta * cue_salience(c) * x[c] / n2;
        }
        delta
    }

    /// A stretch of safe exposure: the cue was close for `weight` of a trial
    /// and the predicted outcome did not occur. Writes extinction for this context.
    pub fn exposure_without_outcome(&mut self, p: &MindParams, x: &CueVec, ctx: usize, o: usize, weight: Fx) -> Fx {
        let pred = self.predict_pav(x, ctx)[o];
        if pred < fx(0.03) {
            return Fx::ZERO;
        }
        let n2 = snorm2(x);
        let rate = (p.alpha_ext * self.mean_alpha(x) * weight).clamp(Fx::ZERO, fx(0.9));
        for c in 0..N_CUES {
            if x[c].raw() == 0 {
                continue;
            }
            let w = self.pav_fast[c][o] + self.pav_slow[c][o];
            if !w.is_positive() {
                continue;
            }
            let de = rate * pred * cue_salience(c) * x[c] / n2;
            let e = &mut self.pav_ext[ci(c, ctx)][o];
            *e = (*e + de).min(w);
        }
        // Unsurprising exposure lowers associability slightly (latent inhibition).
        for c in 0..N_CUES {
            if x[c] > fx(0.2) {
                self.alpha[c] = (self.alpha[c] - fx(0.002) * weight).max(fx(0.1));
            }
        }
        -pred
    }

    fn update_alpha(&mut self, x: &CueVec, delta: &Out) {
        let surprise = delta.iter().fold(Fx::ZERO, |m, d| m.max(d.abs())).min(Fx::ONE);
        let g = fx(0.3);
        for c in 0..N_CUES {
            if x[c] > fx(0.2) {
                self.alpha[c] = (g * surprise + (Fx::ONE - g) * self.alpha[c]).clamp(fx(0.1), Fx::ONE);
            }
        }
    }

    fn tag_cues(&mut self, x: &CueVec, delta: &Out, actual: &Out) {
        let mut tag = Fx::ZERO;
        for o in 0..N_OUT {
            tag = tag.max(delta[o].abs() * actual[o]);
        }
        let tag = tag.min(Fx::ONE);
        for c in 0..N_CUES {
            if x[c] > fx(0.2) {
                self.tag[c] = self.tag[c].max(tag * x[c]);
            }
        }
    }

    fn push_change(&self, rep: &mut LearnReport, kind: &'static str, action: Option<usize>, c: usize, o: usize, before: Fx, x: Fx) {
        if x < fx(0.25) {
            return;
        }
        let after = match action {
            Some(a) => self.inst_net(a, c, o, 0),
            None => self.pav_net(c, o, 0),
        };
        rep.changes.push(WeightChange { kind, action, cue: c, outcome: o, before, after });
    }

    /// Sleep consolidation (01 rule 7–8): move fast weights into slow storage,
    /// more for tagged (surprising, intense) cues; relax fast weights; apply
    /// forgetting to slow weights and faster decay to extinction memories.
    /// `days` = time since the last consolidation.
    pub fn consolidate(&mut self, p: &MindParams, replayed: &[usize], days: Fx) -> ConsolidationReport {
        let mut rep = ConsolidationReport::default();
        for c in 0..N_CUES {
            let replay_bonus = if replayed.contains(&c) { fx(0.2) } else { Fx::ZERO };
            let rho = (fx(0.3) + fx(0.6) * self.tag[c] + replay_bonus).min(fx(0.95));
            if self.tag[c] > fx(0.7) {
                if !self.flashbulb[c] {
                    rep.flashbulbs += 1;
                }
                self.flashbulb[c] = true;
            }
            if replayed.contains(&c) {
                self.rehearsals[c] = self.rehearsals[c].saturating_add(1);
            }
            // Forgetting time constant grows with rehearsal; flashbulb memories last ~10x longer.
            let reh = Fx::ONE + Fx::from_int(1 + self.rehearsals[c] as i64).ln();
            let mut tau = p.forget_days * reh;
            if self.flashbulb[c] {
                tau = tau.muli(10);
            }
            let keep = Fx::decay(days, tau);
            let ext_keep = Fx::decay(days, fx(10.0));
            let mut moved = Fx::ZERO;
            for o in 0..N_OUT {
                let f = self.pav_fast[c][o];
                self.pav_slow[c][o] = self.pav_slow[c][o] * keep + f * rho;
                self.pav_fast[c][o] = f * fx(0.25);
                moved += (f * rho).abs();
                for a in 0..N_ACT {
                    let f = self.inst_fast[ai(a, c)][o];
                    self.inst_slow[ai(a, c)][o] = self.inst_slow[ai(a, c)][o] * keep + f * rho;
                    self.inst_fast[ai(a, c)][o] = f * fx(0.25);
                    moved += (f * rho).abs();
                    for ctx in 0..N_CTX {
                        let e = &mut self.inst_ext[aci(a, c, ctx)][o];
                        *e = *e * ext_keep;
                    }
                }
                for ctx in 0..N_CTX {
                    let e = &mut self.pav_ext[ci(c, ctx)][o];
                    *e = *e * ext_keep;
                }
            }
            if moved > fx(0.01) {
                rep.cues_consolidated += 1;
            }
            self.tag[c] = Fx::ZERO;
        }
        rep
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConsolidationReport {
    pub cues_consolidated: u32,
    pub flashbulbs: u32,
    pub episodes_replayed: u32,
    pub beliefs_crystallised: u32,
}

pub fn norm2(x: &CueVec) -> Fx {
    x.iter().fold(Fx::ZERO, |a, &v| a + v * v)
}

impl StableHash for Assoc {
    fn stable_hash(&self, h: &mut StateHasher) {
        for v in [&self.pav_fast, &self.pav_slow, &self.inst_fast, &self.inst_slow, &self.pav_ext, &self.inst_ext] {
            for o in v.iter() {
                o.stable_hash(h);
            }
        }
        self.alpha.stable_hash(h);
        for e in &self.evidence {
            e.stable_hash(h);
        }
        for e in &self.pav_evidence {
            e.stable_hash(h);
        }
        self.tag.stable_hash(h);
        for f in &self.flashbulb {
            f.stable_hash(h);
        }
        for r in &self.rehearsals {
            r.stable_hash(h);
        }
    }
}
