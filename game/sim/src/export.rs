//! JSON exports for the Godot client: status, inspector, history, brain graph.
//! (Hand-written JSON; the simulation crates use no external dependencies.)

use crate::{cheb, obj_name, rel, Creature, Kind, Sim, Target, TICKS_PER_DAY};
use alife_core::Fx;
use alife_mind::learning::assoc::{action_label, N_CUES, N_OUT, A_MOUTH, A_TOUCH};
use alife_mind::memory::semantic::Source;
use std::fmt::Write as _;

pub fn q(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    o.push('"');
    for ch in s.chars() {
        match ch {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            c if (c as u32) < 0x20 => {}
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

fn f(x: Fx) -> String {
    format!("{:.3}", x.to_f64())
}

pub fn clock(sim: &Sim) -> String {
    let t = sim.time_of_day();
    let day = sim.tick / TICKS_PER_DAY + 1;
    format!("Day {} {:02}:{:02}", day, t / 3600, (t / 60) % 60)
}

pub fn status_json(sim: &Sim, actual_speed: f64) -> String {
    let humans = sim.creatures.iter().filter(|c| c.alive && c.kind == Kind::Human).count();
    let grazers = sim.creatures.iter().filter(|c| c.alive && c.kind == Kind::Grazer).count();
    let wolves = sim.creatures.iter().filter(|c| c.alive && c.kind == Kind::Predator).count();
    let season = ["Spring", "Summer", "Autumn", "Winter"][sim.season() as usize];
    let weather = ["Clear", "Cloudy", "Rain", "Storm"][sim.weather as usize];
    let (cx, cy) = (sim.w / 2, sim.h / 2);
    format!(
        "{{\"tick\":{},\"clock\":{},\"year\":{},\"season\":{},\"weather\":{},\"temp\":{:.1},\"light\":{:.2},\"humans\":{},\"grazers\":{},\"wolves\":{},\"births\":{},\"deaths\":{},\"actual_speed\":{:.1},\"fidelity\":{},\"history_len\":{},\"god_temp\":{:.0}}}",
        sim.tick,
        q(&clock(sim)),
        sim.year() + 1,
        q(season),
        q(weather),
        sim.temperature(cx, cy).to_f64(),
        sim.light().to_f64(),
        humans,
        grazers,
        wolves,
        sim.births,
        sim.deaths,
        actual_speed,
        q(&sim.fidelity_label()),
        sim.history.len(),
        sim.god_temp.to_f64()
    )
}

pub fn history_json(sim: &Sim, from: usize) -> String {
    let mut o = String::from("[");
    for (i, e) in sim.history.iter().enumerate().skip(from) {
        if i > from {
            o.push(',');
        }
        let day = e.tick / TICKS_PER_DAY + 1;
        let t = e.tick % TICKS_PER_DAY;
        let _ = write!(
            o,
            "{{\"i\":{},\"when\":{},\"year\":{},\"kind\":{},\"text\":{},\"x\":{},\"y\":{},\"important\":{}}}",
            i,
            q(&format!("Day {} {:02}:{:02}", day, t / 3600, (t / 60) % 60)),
            e.tick / crate::TICKS_PER_YEAR + 1,
            q(&e.kind),
            q(&e.text),
            e.x,
            e.y,
            e.important
        );
    }
    o.push(']');
    o
}

fn name_of(sim: &Sim, id: u32) -> String {
    sim.creatures.iter().find(|c| c.id == id).map(|c| c.name.clone()).unwrap_or_else(|| format!("#{id}"))
}

fn person_ref(sim: &Sim, id: u32) -> String {
    match sim.creatures.iter().find(|c| c.id == id) {
        Some(c) => format!(
            "{{\"id\":{},\"name\":{},\"alive\":{},\"age\":{:.0},\"female\":{}}}",
            c.id,
            q(&c.name),
            c.alive,
            sim.age_years(c).to_f64(),
            c.female
        ),
        None => format!("{{\"id\":{id},\"name\":\"?\",\"alive\":false,\"age\":0,\"female\":false}}"),
    }
}

fn life_stage(age: f64, kind: Kind) -> &'static str {
    if kind != Kind::Human {
        return if age < 1.5 { "young" } else { "adult" };
    }
    match age {
        a if a < 2.0 => "infant",
        a if a < 12.0 => "child",
        a if a < 18.0 => "adolescent",
        a if a < 55.0 => "adult",
        _ => "elder",
    }
}

pub fn inspect_json(sim: &Sim, id: u32) -> String {
    let Some(c) = sim.creatures.iter().find(|c| c.id == id) else { return "{}".into() };
    let age = sim.age_years(c).to_f64();
    let mut o = String::from("{");
    let kind = ["Human", "Grazer", "Wolf"][c.kind as usize];
    let _ = write!(
        o,
        "\"id\":{},\"name\":{},\"kind\":{},\"alive\":{},\"age\":{:.1},\"stage\":{},\"sex\":{},\"health\":{},\"action\":{},\"x\":{},\"y\":{},\"cause_of_death\":{},\"think_interval\":{},",
        c.id,
        q(&c.name),
        q(kind),
        c.alive,
        age,
        q(life_stage(age, c.kind)),
        q(if c.female { "female" } else { "male" }),
        f(c.health),
        q(&c.action),
        c.x,
        c.y,
        q(c.cause_of_death.as_deref().unwrap_or("")),
        c.think_interval
    );
    // Body.
    if c.kind == Kind::Human {
        let s = c.body.signals;
        let _ = write!(
            o,
            "\"body\":{{\"hunger\":{},\"thirst\":{},\"energy\":{},\"pain\":{},\"health\":{},\"warmth\":{},\"loneliness\":{},\"asleep\":{},\"wounds\":{},\"burns\":{},\"pregnant\":{}}},",
            f(s.hunger),
            f(s.thirst),
            f(Fx::ONE - s.fatigue),
            f(s.pain),
            f(c.health),
            f(c.body.thermal),
            f(c.loneliness),
            c.body.asleep,
            f(c.body.damage_body),
            f(c.body.damage_hand.max(c.body.damage_mouth)),
            c.pregnant.is_some()
        );
    } else {
        let a = &c.animal;
        let _ = write!(
            o,
            "\"body\":{{\"hunger\":{},\"thirst\":{},\"energy\":{},\"pain\":0,\"health\":{},\"fear_of_humans\":{},\"pregnant\":{}}},",
            f(Fx::ONE - a.energy),
            f(Fx::ONE - a.water),
            f(Fx::ONE - a.fatigue),
            f(c.health),
            f(a.fear_of_humans),
            c.pregnant.is_some()
        );
    }
    // Genetics.
    let t = &c.traits;
    let _ = write!(
        o,
        "\"genetics\":{{\"height\":{},\"build\":{},\"skin\":{},\"hair\":{},\"eye\":{},\"metabolism\":{},\"vitality\":{},\"fertility\":{},\"dexterity\":{},\"openness\":{},\"conscientiousness\":{},\"extraversion\":{},\"agreeableness\":{},\"neuroticism\":{},\"genome\":{}}},",
        f(t.height),
        f(t.build),
        f(t.skin),
        f(t.hair),
        f(t.eye),
        f(t.metabolism),
        f(t.vitality),
        f(t.fertility),
        f(t.dexterity),
        f(t.openness),
        f(t.conscientiousness),
        f(t.extraversion),
        f(t.agreeableness),
        f(t.neuroticism),
        q(&format!("{:016x}{:016x}/{:016x}{:016x}", c.genome.a[0], c.genome.a[1], c.genome.b[0], c.genome.b[1]))
    );
    // Family.
    let siblings: Vec<u32> = sim
        .creatures
        .iter()
        .filter(|s| s.id != c.id && s.kind == Kind::Human && ((c.mother.is_some() && s.mother == c.mother) || (c.father.is_some() && s.father == c.father)))
        .map(|s| s.id)
        .collect();
    let grand: Vec<u32> = [c.mother, c.father]
        .into_iter()
        .flatten()
        .filter_map(|p| sim.creatures.iter().find(|x| x.id == p))
        .flat_map(|p| [p.mother, p.father])
        .flatten()
        .collect();
    let refs = |v: &[u32]| v.iter().map(|&i| person_ref(sim, i)).collect::<Vec<_>>().join(",");
    let _ = write!(
        o,
        "\"family\":{{\"mother\":{},\"father\":{},\"partner\":{},\"children\":[{}],\"siblings\":[{}],\"grandparents\":[{}]}},",
        c.mother.map(|m| person_ref(sim, m)).unwrap_or("null".into()),
        c.father.map(|m| person_ref(sim, m)).unwrap_or("null".into()),
        c.partner.map(|m| person_ref(sim, m)).unwrap_or("null".into()),
        refs(&c.children),
        refs(&siblings),
        refs(&grand)
    );
    // Relationships.
    let mut rels = c.relations.clone();
    rels.sort_by(|a, b| (b.familiarity + b.affection).cmp(&(a.familiarity + a.affection)));
    o.push_str("\"relationships\":[");
    for (k, r) in rels.iter().take(16).enumerate() {
        if k > 0 {
            o.push(',');
        }
        let _ = write!(
            o,
            "{{\"id\":{},\"name\":{},\"familiarity\":{},\"affection\":{},\"trust\":{},\"attraction\":{},\"fear\":{},\"resentment\":{}}}",
            r.other,
            q(&name_of(sim, r.other)),
            f(r.familiarity),
            f(r.affection),
            f(r.trust),
            f(r.attraction),
            f(r.fear),
            f(r.resentment)
        );
    }
    o.push_str("],");
    // Skills.
    o.push_str("\"skills\":[");
    o.push_str(&c.skills.iter().map(|(n, v)| format!("{{\"name\":{},\"level\":{}}}", q(n), f(*v))).collect::<Vec<_>>().join(","));
    o.push_str("],");
    // Mind.
    if let Some(m) = c.mind.as_ref() {
        // Episodic memories (pinned first, then most recent).
        let mut eps: Vec<&alife_mind::memory::episodic::Episode> = m.episodic.all().collect();
        eps.sort_by(|a, b| b.pinned.cmp(&a.pinned).then(b.tick.cmp(&a.tick)));
        o.push_str("\"memories\":[");
        for (k, e) in eps.iter().take(24).enumerate() {
            if k > 0 {
                o.push(',');
            }
            let ago = sim.tick.saturating_sub(e.tick);
            let _ = write!(
                o,
                "{{\"id\":{},\"text\":{},\"pinned\":{},\"ago\":{},\"valence\":{},\"arousal\":{},\"recalls\":{}}}",
                e.id,
                q(&m.episode_summary(e)),
                e.pinned,
                q(&ago_text(ago)),
                f(e.valence),
                f(e.arousal),
                e.recalls
            );
        }
        o.push_str("],");
        // Knowledge: what this mind has learned predicts outcomes (strongest associations).
        let mut know: Vec<(String, String, Fx, u16)> = Vec::new();
        for cue in 0..N_CUES {
            for out in 0..N_OUT {
                let w = m.assoc.pav_fast[cue][out] + m.assoc.pav_slow[cue][out];
                if w > Fx::from_f64(0.08) {
                    know.push((readable_cue(sim, c, &m.cue_name(cue)), format!("mean {}", readable_outcome(out)), w, m.assoc.pav_evidence[cue]));
                }
                for a in [A_TOUCH, A_MOUTH] {
                    let w = m.assoc.inst_fast[a * N_CUES + cue][out] + m.assoc.inst_slow[a * N_CUES + cue][out];
                    if w > Fx::from_f64(0.08) {
                        know.push((readable_cue(sim, c, &m.cue_name(cue)), format!("{} them → {}", if a == A_TOUCH { "touch" } else { "put in mouth" }, readable_outcome(out)), w, m.assoc.evidence[a * N_CUES + cue]));
                    }
                }
            }
        }
        know.sort_by(|a, b| b.2.cmp(&a.2));
        o.push_str("\"knowledge\":[");
        o.push_str(
            &know
                .iter()
                .take(20)
                .map(|(c, w, s, e)| format!("{{\"cue\":{},\"what\":{},\"strength\":{},\"evidence\":{}}}", q(c), q(w), f(*s), e))
                .collect::<Vec<_>>()
                .join(","),
        );
        o.push_str("],");
        // Explicit beliefs.
        o.push_str("\"beliefs\":[");
        let mut bs = m.semantic.beliefs.clone();
        bs.sort_by(|a, b| b.confidence.cmp(&a.confidence));
        for (k, b) in bs.iter().take(16).enumerate() {
            if k > 0 {
                o.push(',');
            }
            let src = match b.source {
                Source::OwnExperience => "own experience".to_string(),
                Source::Observed { who } => format!("watched {}", name_of(sim, who)),
                Source::Told { who } => format!("told by {}", name_of(sim, who)),
            };
            let cue = readable_cue(sim, c, &m.cue_name(b.cue as usize));
            let out = readable_outcome(b.outcome as usize);
            let text = match b.action.map(|a| a as usize) {
                Some(A_TOUCH) => format!("Touching {cue} brings {out}"),
                Some(A_MOUTH) => format!("Putting {cue} in the mouth brings {out}"),
                Some(a) => format!("{} {cue} brings {out}", action_label(a)),
                None => format!("{} mean {out}", capitalize(&cue)),
            };
            let _ = write!(o, "{{\"belief\":{},\"confidence\":{},\"source\":{},\"evidence\":{}}}", q(&text), f(b.confidence), q(&src), b.evidence);
        }
        o.push_str("],");
        let _ = write!(o, "\"concepts\":{},\"episodes_stored\":{},\"observed\":{},", m.concepts.concepts.len(), m.stats.episodes_stored, m.stats.observed);
        // Live decision trace (only recorded for the selected human).
        match &m.last_trace {
            Some(tr) if !tr.asleep => {
                let _ = write!(o, "\"trace\":{},\"trace_text\":{},", observer_names(sim, c, &tr.to_json()), q(&observer_names(sim, c, &tr.to_text())));
                let _ = write!(o, "\"graph\":{},", observer_names(sim, c, &brain_graph(tr)));
            }
            Some(_) => o.push_str("\"trace\":null,\"trace_text\":\"Asleep. (Sleep consolidates memories.)\",\"graph\":null,"),
            None => o.push_str("\"trace\":null,\"trace_text\":\"Select this person and let time run to watch them think.\",\"graph\":null,"),
        }
    }
    // Nearby (for context).
    let near: Vec<String> = sim
        .objs
        .iter()
        .filter(|ob| ob.alive && cheb(ob.x, ob.y, c.x, c.y) <= 3)
        .take(6)
        .map(|ob| q(&obj_name(ob)))
        .collect();
    let _ = write!(o, "\"nearby\":[{}]", near.join(","));
    o.push('}');
    o
}

/// Observer view only: replace the mind's private, meaningless tokens ("thing#51d1")
/// with what the player can see is really there ("berry bush"). The mind never
/// receives these names; this is applied to the exported text after the fact.
fn observer_names(sim: &Sim, c: &Creature, text: &str) -> String {
    let mut out = concept_names(sim, c, text);
    if !out.contains("thing#") {
        return out;
    }
    for (tk, t) in &c.tokens {
        let key = format!("thing#{:04x}", tk.0 & 0xffff);
        if !out.contains(&key) {
            continue;
        }
        let name = match *t {
            Target::Obj(id) => sim.objs.iter().find(|o| o.id == id).map(obj_name).unwrap_or_else(|| "something".into()),
            Target::Water(..) => "water".into(),
            Target::Creature(id) => name_of(sim, id),
        };
        out = out.replace(&key, &name);
    }
    out
}

/// "concept C12" -> "C12 (berry bush)": what this private category has really
/// been recognised on so far (observer view).
fn concept_names(sim: &Sim, c: &Creature, text: &str) -> String {
    let owned = short_concepts(sim, c, &text.replace("nutrient-intake", "food").replace("fluid-intake", "drink"));
    let text: &str = owned.as_str();
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(p) = rest.find("concept C") {
        out.push_str(&rest[..p]);
        let after = &rest[p + 9..];
        let digits: String = after.chars().take_while(|ch| ch.is_ascii_digit()).collect();
        if digits.is_empty() {
            out.push_str("concept C");
            rest = after;
            continue;
        }
        let id: u16 = digits.parse().unwrap_or(0);
        match sim.concept_meaning(c, id) {
            Some(m) => out.push_str(&format!("C{id} ({m})")),
            None => out.push_str(&format!("C{id}")),
        }
        rest = &after[digits.len()..];
    }
    out.push_str(rest);
    out
}

/// Trace shorthand "C12@3m" / "C12 [..." -> "C12 (berry bush)@3m".
fn short_concepts(sim: &Sim, c: &Creature, text: &str) -> String {
    let b = text.as_bytes();
    let mut out = String::with_capacity(text.len() + 32);
    let mut i = 0;
    while i < b.len() {
        let start_ok = i == 0 || !(b[i - 1] as char).is_ascii_alphanumeric();
        if text.is_char_boundary(i) && b[i] == b'C' && start_ok && i + 1 < b.len() && b[i + 1].is_ascii_digit() && !text[..i].ends_with("concept ") {
            let mut j = i + 1;
            while j < b.len() && b[j].is_ascii_digit() {
                j += 1;
            }
            let next_ok = j < b.len() && (b[j] == b'@' || (b[j] == b' ' && j + 1 < b.len() && b[j + 1] == b'['));
            if next_ok {
                let id: u16 = text[i + 1..j].parse().unwrap_or(0);
                out.push_str(&text[i..j]);
                if let Some(m) = sim.concept_meaning(c, id) {
                    out.push_str(&format!(" ({m})"));
                }
                i = j;
                continue;
            }
        }
        // Copy one whole UTF-8 character.
        let ch = text[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// Plain-language name for a cue: a sensation ("red things", "faces", "hot
/// things") or a learned category ("C3 (berry bush)").
pub fn readable_cue(sim: &Sim, c: &Creature, raw: &str) -> String {
    if raw.starts_with("concept C") {
        return concept_names(sim, c, raw);
    }
    if let Some(h) = raw.strip_prefix("hue:") {
        return format!("{h} things");
    }
    match raw {
        "colourless" => "grey things".into(),
        "dark" => "dark things".into(),
        "lit>0.2" => "visible (lit) things".into(),
        "bright>0.45" => "bright things".into(),
        "glaring>0.7" => "glaring, glowing things".into(),
        "warmth-felt" | "warmth>0.1" => "slightly warm things".into(),
        "warmth>0.3" => "warm things".into(),
        "warmth>0.5" => "hot things".into(),
        "warmth>0.7" => "very hot things".into(),
        "face" => "faces".into(),
        "moving" => "moving things".into(),
        "jagged" => "jagged things".into(),
        other => format!("{other} things"),
    }
}

fn readable_outcome(o: usize) -> &'static str {
    ["pain", "food", "drink", "comfort", "warmth"][o % 5]
}

fn ago_text(t: u64) -> String {
    if t < 120 {
        format!("{t}s ago")
    } else if t < 7200 {
        format!("{}min ago", t / 60)
    } else if t < 2 * TICKS_PER_DAY {
        format!("{}h ago", t / 3600)
    } else {
        format!("{}d ago", t / TICKS_PER_DAY)
    }
}

/// A small node/edge graph of the active pathway behind the current decision:
/// perception -> learned associations -> outcomes/memory -> emotion -> goal -> action.
pub fn brain_graph(tr: &alife_mind::trace::TraceRecord) -> String {
    let mut nodes: Vec<(String, String, &str)> = Vec::new(); // (id, label, layer)
    let mut edges: Vec<(String, String, f64)> = Vec::new();
    let add = |nodes: &mut Vec<(String, String, &'static str)>, id: String, label: String, layer: &'static str| {
        if !nodes.iter().any(|n| n.0 == id) {
            nodes.push((id, label, layer));
        }
    };
    // Perception: attended things.
    for (k, (label, sal)) in tr.attention.iter().enumerate().take(4) {
        let id = format!("a{k}");
        add(&mut nodes, id.clone(), label.chars().take(42).collect(), "perception");
        let _ = sal;
    }
    // Associations of the focus.
    for (k, a) in tr.associations.iter().take(5).enumerate() {
        let cid = format!("c{k}");
        add(&mut nodes, cid.clone(), a.cue.clone(), "cue");
        if !nodes.is_empty() {
            edges.push(("a0".into(), cid.clone(), a.activation.to_f64()));
        }
        let oid = format!("o_{}", a.outcome);
        add(&mut nodes, oid.clone(), format!("expect {}", a.outcome), "outcome");
        edges.push((cid, oid, a.weight.to_f64()));
    }
    for (k, m) in tr.memories.iter().take(2).enumerate() {
        let id = format!("m{k}");
        add(&mut nodes, id.clone(), format!("memory: {}", m.summary.chars().take(48).collect::<String>()), "memory");
        edges.push(("a0".into(), id.clone(), (m.activation.to_f64() / 6.0).min(1.0)));
        let target = nodes.iter().find(|n| n.2 == "outcome").map(|n| n.0.clone());
        if let Some(t) = target {
            edges.push((id, t, 0.5));
        }
    }
    for (k, (e, v)) in tr.emotions.iter().take(3).enumerate() {
        let id = format!("e{k}");
        add(&mut nodes, id.clone(), format!("{e} {:.2}", v.to_f64()), "emotion");
        let outs: Vec<String> = nodes.iter().filter(|n| n.2 == "outcome").map(|n| n.0.clone()).collect();
        for oid in outs {
            edges.push((oid, id.clone(), v.to_f64()));
        }
        edges.push((id, "goal".into(), v.to_f64()));
    }
    add(&mut nodes, "goal".into(), format!("goal: {}", tr.goal), "goal");
    add(&mut nodes, "act".into(), tr.decision.to_uppercase(), "action");
    edges.push(("goal".into(), "act".into(), 1.0));
    if tr.emotions.is_empty() {
        let outs: Vec<String> = nodes.iter().filter(|n| n.2 == "outcome").map(|n| n.0.clone()).collect();
        for oid in outs {
            edges.push((oid, "goal".into(), 0.5));
        }
    }
    let mut o = String::from("{\"nodes\":[");
    o.push_str(&nodes.iter().map(|(id, l, layer)| format!("{{\"id\":{},\"label\":{},\"layer\":{}}}", q(id), q(l), q(layer))).collect::<Vec<_>>().join(","));
    o.push_str("],\"edges\":[");
    o.push_str(&edges.iter().map(|(a, b, w)| format!("{{\"from\":{},\"to\":{},\"w\":{:.3}}}", q(a), q(b), w)).collect::<Vec<_>>().join(","));
    o.push_str("]}");
    o
}

/// Short tooltip text for a creature under the cursor.
pub fn tooltip(sim: &Sim, c: &Creature) -> String {
    format!("{} ({}, {:.0}) — {}", c.name, ["human", "grazer", "wolf"][c.kind as usize], sim.age_years(c).to_f64(), c.action)
}

/// Names of things a human currently has tokens for (debug helper).
pub fn _known_targets(c: &Creature) -> usize {
    c.tokens.iter().filter(|(_, t)| matches!(t, Target::Obj(_) | Target::Water(..) | Target::Creature(_))).count()
}

pub fn _rel_name(sim: &Sim, c: &Creature, other: u32) -> Option<String> {
    rel(&c.relations, other).map(|_| name_of(sim, other))
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}
