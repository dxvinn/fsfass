//! Brain trace: a record of one decision, showing why it was made.
//!
//! Traces are built only when tracing is enabled (record-on-observe), so
//! unobserved minds pay nothing for them.

use alife_core::Fx;
use std::fmt::Write as _;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PerceptLine {
    pub label: String,
    pub dist: i32,
    pub features: Vec<String>,
    pub warmth: Fx,
    pub concept: u16,
    pub similarity: Fx,
    pub novel_concept: bool,
    pub contact: Vec<String>,
    /// Gut (Pavlovian) prediction of pain, and overall gut value.
    pub gut_pain: Fx,
    pub gut_value: Fx,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MemoryLine {
    pub episode: u32,
    pub summary: String,
    pub activation: Fx,
    pub age_s: u64,
    pub pinned: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AssocLine {
    pub cue: String,
    pub action: Option<&'static str>,
    pub outcome: &'static str,
    pub weight: Fx,
    pub extinction: Fx,
    pub evidence: u16,
    pub activation: Fx,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct OptionLine {
    pub label: String,
    pub total: Fx,
    pub deliberate: Fx,
    pub habit: Fx,
    pub gut: Fx,
    pub curiosity: Fx,
    pub persistence: Fx,
    pub noise: Fx,
    pub pred_pain: Fx,
    pub pred_nourish: Fx,
    pub pred_hydrate: Fx,
    pub episodic: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TraceRecord {
    pub tick: u64,
    pub asleep: bool,
    pub context: u8,
    pub perception: Vec<PerceptLine>,
    pub attention: Vec<(String, Fx)>,
    pub working_memory: Vec<String>,
    pub memories: Vec<MemoryLine>,
    pub associations: Vec<AssocLine>,
    pub beliefs: Vec<String>,
    pub emotions: Vec<(&'static str, Fx)>,
    pub mood: (Fx, Fx, Fx),
    pub needs: Vec<(&'static str, Fx)>,
    pub options: Vec<OptionLine>,
    pub reflex: String,
    pub habit: String,
    pub deliberate: String,
    pub w_deliberate: Fx,
    pub reliability_deliberate: Fx,
    pub reliability_habit: Fx,
    pub decision: String,
    pub goal: String,
    pub reason: String,
    pub learning: Vec<String>,
}

pub fn clock(tick: u64) -> String {
    let day = tick / 86_400;
    let s = tick % 86_400;
    format!("day {} {:02}:{:02}:{:02}", day + 1, s / 3600, (s / 60) % 60, s % 60)
}

fn f(x: Fx) -> String {
    format!("{:.2}", x.to_f64())
}

fn signed(x: Fx) -> String {
    format!("{:+.2}", x.to_f64())
}

impl TraceRecord {
    /// Human-readable trace in the Phase 2 format.
    pub fn to_text(&self) -> String {
        let mut o = String::new();
        let _ = writeln!(o, "TIME {}   (tick {}, context X{})", clock(self.tick), self.tick, self.context);
        if self.asleep {
            let _ = writeln!(o, "ASLEEP");
            for l in &self.learning {
                let _ = writeln!(o, "  {l}");
            }
            return o;
        }
        let _ = writeln!(o, "\nPERCEPTION");
        for p in &self.perception {
            let _ = writeln!(
                o,
                "  {} at {}m: {}{}  -> concept C{} (match {}){}   gut: pain {} value {}",
                p.label,
                p.dist,
                p.features.join(", "),
                if p.warmth.raw() > 0 { format!(", felt warmth {}", f(p.warmth)) } else { String::new() },
                p.concept,
                f(p.similarity),
                if p.novel_concept { " NEW" } else { "" },
                f(p.gut_pain),
                signed(p.gut_value)
            );
            if !p.contact.is_empty() {
                let _ = writeln!(o, "      contact sensations: {}", p.contact.join(", "));
            }
        }
        let _ = writeln!(o, "\nNEEDS");
        let _ = writeln!(
            o,
            "  {}",
            self.needs.iter().map(|(n, v)| format!("{n} {}", f(*v))).collect::<Vec<_>>().join("   ")
        );
        let _ = writeln!(o, "\nATTENTION (workspace)");
        for (i, (l, s)) in self.attention.iter().enumerate() {
            let _ = writeln!(o, "  {}. {}   salience {}", i + 1, l, f(*s));
        }
        let _ = writeln!(o, "\nWORKING MEMORY");
        for l in &self.working_memory {
            let _ = writeln!(o, "  {l}");
        }
        let _ = writeln!(o, "\nMEMORIES ACTIVATED");
        if self.memories.is_empty() {
            let _ = writeln!(o, "  (none)");
        }
        for m in &self.memories {
            let _ = writeln!(
                o,
                "  episode #{}{} ({} ago): {}   activation {}",
                m.episode,
                if m.pinned { " [pinned]" } else { "" },
                age(m.age_s),
                m.summary,
                f(m.activation)
            );
        }
        let _ = writeln!(o, "\nASSOCIATIONS (target)");
        if self.associations.is_empty() {
            let _ = writeln!(o, "  (none learned)");
        }
        for a in &self.associations {
            let _ = writeln!(
                o,
                "  {}{} -> {}   weight {}{}   evidence {}   (cue active {})",
                a.action.map(|x| format!("{x} + ")).unwrap_or_default(),
                a.cue,
                a.outcome,
                f(a.weight),
                if a.extinction.raw() > 0 { format!(" (minus extinction here {})", f(a.extinction)) } else { String::new() },
                a.evidence,
                f(a.activation)
            );
        }
        let _ = writeln!(o, "\nBELIEFS (explicit)");
        if self.beliefs.is_empty() {
            let _ = writeln!(o, "  (none relevant)");
        }
        for b in &self.beliefs {
            let _ = writeln!(o, "  {b}");
        }
        let _ = writeln!(o, "\nEMOTION");
        let _ = writeln!(
            o,
            "  {}   mood P{} A{} D{}",
            if self.emotions.is_empty() {
                "calm".to_string()
            } else {
                self.emotions.iter().map(|(n, v)| format!("{n} {}", f(*v))).collect::<Vec<_>>().join(", ")
            },
            signed(self.mood.0),
            signed(self.mood.1),
            signed(self.mood.2)
        );
        let _ = writeln!(o, "\nPREDICTIONS (expected outcome if I...)");
        for p in self.options.iter().take(8) {
            let _ = writeln!(
                o,
                "  {:<22} pain {}  nutrient {}  fluid {}{}",
                p.label,
                f(p.pred_pain),
                f(p.pred_nourish),
                f(p.pred_hydrate),
                p.episodic.as_ref().map(|e| format!("   [{e}]")).unwrap_or_default()
            );
        }
        let _ = writeln!(o, "\nDECISION SYSTEMS");
        let _ = writeln!(o, "  REFLEX/GUT:  {}", self.reflex);
        let _ = writeln!(o, "  HABIT:       {}", self.habit);
        let _ = writeln!(o, "  DELIBERATE:  {}", self.deliberate);
        let _ = writeln!(
            o,
            "  arbitration: deliberate weight {} (reliability deliberate {} vs habit {})",
            f(self.w_deliberate),
            f(self.reliability_deliberate),
            f(self.reliability_habit)
        );
        let _ = writeln!(o, "\nOPTION SCORES  total = deliberate + habit + gut + curiosity + persistence + noise");
        for p in self.options.iter().take(8) {
            let _ = writeln!(
                o,
                "  {:<22} {:>6} = {} {} {} {} {} {}",
                p.label,
                signed(p.total),
                signed(p.deliberate),
                signed(p.habit),
                signed(p.gut),
                signed(p.curiosity),
                signed(p.persistence),
                signed(p.noise)
            );
        }
        let _ = writeln!(o, "\nFINAL ACTION: {}", self.decision);
        let _ = writeln!(o, "GOAL: {}", self.goal);
        let _ = writeln!(o, "WHY: {}", self.reason);
        if !self.learning.is_empty() {
            let _ = writeln!(o, "\nLEARNING THIS MOMENT");
            for l in &self.learning {
                let _ = writeln!(o, "  {l}");
            }
        }
        o
    }

    /// Compact JSON (hand-written; no external crates).
    pub fn to_json(&self) -> String {
        let q = |s: &str| format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""));
        let mut o = String::from("{");
        let _ = write!(o, "\"tick\":{},\"asleep\":{},\"context\":{},", self.tick, self.asleep, self.context);
        o.push_str("\"perception\":[");
        for (i, p) in self.perception.iter().enumerate() {
            if i > 0 {
                o.push(',');
            }
            let _ = write!(
                o,
                "{{\"label\":{},\"dist\":{},\"features\":[{}],\"warmth\":{:.3},\"concept\":{},\"similarity\":{:.3}}}",
                q(&p.label),
                p.dist,
                p.features.iter().map(|x| q(x)).collect::<Vec<_>>().join(","),
                p.warmth.to_f64(),
                p.concept,
                p.similarity.to_f64()
            );
        }
        o.push_str("],\"attention\":[");
        o.push_str(
            &self
                .attention
                .iter()
                .map(|(l, s)| format!("{{\"item\":{},\"salience\":{:.3}}}", q(l), s.to_f64()))
                .collect::<Vec<_>>()
                .join(","),
        );
        o.push_str("],\"memories\":[");
        o.push_str(
            &self
                .memories
                .iter()
                .map(|m| format!("{{\"episode\":{},\"summary\":{},\"activation\":{:.3}}}", m.episode, q(&m.summary), m.activation.to_f64()))
                .collect::<Vec<_>>()
                .join(","),
        );
        o.push_str("],\"associations\":[");
        o.push_str(
            &self
                .associations
                .iter()
                .map(|a| {
                    format!(
                        "{{\"cue\":{},\"action\":{},\"outcome\":{},\"weight\":{:.3},\"evidence\":{}}}",
                        q(&a.cue),
                        a.action.map(q).unwrap_or_else(|| "null".into()),
                        q(a.outcome),
                        a.weight.to_f64(),
                        a.evidence
                    )
                })
                .collect::<Vec<_>>()
                .join(","),
        );
        o.push_str("],\"emotions\":{");
        o.push_str(
            &self.emotions.iter().map(|(n, v)| format!("{}:{:.3}", q(n), v.to_f64())).collect::<Vec<_>>().join(","),
        );
        o.push_str("},\"needs\":{");
        o.push_str(&self.needs.iter().map(|(n, v)| format!("{}:{:.3}", q(n), v.to_f64())).collect::<Vec<_>>().join(","));
        o.push_str("},\"options\":[");
        o.push_str(
            &self
                .options
                .iter()
                .map(|p| {
                    format!(
                        "{{\"option\":{},\"total\":{:.3},\"deliberate\":{:.3},\"habit\":{:.3},\"gut\":{:.3},\"curiosity\":{:.3},\"pain\":{:.3},\"nourish\":{:.3},\"hydrate\":{:.3}}}",
                        q(&p.label),
                        p.total.to_f64(),
                        p.deliberate.to_f64(),
                        p.habit.to_f64(),
                        p.gut.to_f64(),
                        p.curiosity.to_f64(),
                        p.pred_pain.to_f64(),
                        p.pred_nourish.to_f64(),
                        p.pred_hydrate.to_f64()
                    )
                })
                .collect::<Vec<_>>()
                .join(","),
        );
        let _ = write!(
            o,
            "],\"systems\":{{\"reflex\":{},\"habit\":{},\"deliberate\":{},\"w_deliberate\":{:.3}}},\"decision\":{},\"goal\":{},\"reason\":{},\"learning\":[{}]}}",
            q(&self.reflex),
            q(&self.habit),
            q(&self.deliberate),
            self.w_deliberate.to_f64(),
            q(&self.decision),
            q(&self.goal),
            q(&self.reason),
            self.learning.iter().map(|l| q(l)).collect::<Vec<_>>().join(",")
        );
        o
    }
}

fn age(s: u64) -> String {
    if s < 120 {
        format!("{s}s")
    } else if s < 7200 {
        format!("{}min", s / 60)
    } else if s < 172_800 {
        format!("{}h", s / 3600)
    } else {
        format!("{}d", s / 86_400)
    }
}
