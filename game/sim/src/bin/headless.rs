//! Headless run: simulate N game hours and print population, history and timing.
use genesis_sim::{export, Kind, Sim};
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let hours: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(24);
    let speed: u32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
    let seed: u64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(42);
    let t0 = Instant::now();
    let mut sim = Sim::new(seed, 200, 140);
    println!("world generated in {:?}; {} objects, {} creatures", t0.elapsed(), sim.objs.len(), sim.creatures.len());
    sim.set_speed_lod(speed);
    let t1 = Instant::now();
    let secs: u64 = std::env::var("SECS").ok().and_then(|s| s.parse().ok()).unwrap_or(3600);
    let stats = std::env::var_os("STATS").is_some();
    let mut why_printed = 0;
    let mut mouth_human_adult = 0u64;
    let mut who_mouths: std::collections::BTreeMap<String, u64> = Default::default();
    let (mut mouth_human, mut mouth_obj, mut touch_wolf, mut touch_human, mut samples) = (0u64, 0u64, 0u64, 0u64, 0u64);
    for h in 0..hours {
        for t in 0..secs {
            sim.step();
            if let Ok(who) = std::env::var("WHY_MOUTH") {
                let id = sim.creatures.iter().find(|c| c.name == who).map(|c| c.id);
                if id.is_some() && sim.selected != id {
                    sim.set_selected(id);
                }
                if let Some(c) = sim.creatures.iter().find(|c| c.name == who) {
                    let a = &c.action;
                    let humans: Vec<&str> = sim.creatures.iter().filter(|o| o.kind == Kind::Human).map(|o| o.name.as_str()).collect();
                    if (a.starts_with("tasting ") || a.starts_with("bringing ")) && humans.iter().any(|n| a.contains(n)) && why_printed < 3 && t % 97 == 0 && h >= std::env::var("WHY_AFTER").ok().and_then(|v| v.parse().ok()).unwrap_or(0u64) {
                        why_printed += 1;
                        if let Some(tr) = c.mind.as_ref().and_then(|m| m.last_trace.as_ref()) {
                            println!("==== WHY {} {}\n{}", a, export::clock(&sim), tr.to_text());
                        }
                    }
                }
            }
            if stats && t % 10 == 0 {
                let humans: Vec<String> = sim.creatures.iter().filter(|c| c.alive && c.kind == Kind::Human).map(|c| c.name.clone()).collect();
                for c in sim.creatures.iter().filter(|c| c.alive && c.kind == Kind::Human) {
                    samples += 1;
                    let a = &c.action;
                    let obj = a.trim_start_matches("tasting ").trim_start_matches("bringing ").trim_end_matches(" to mouth");
                    let mouthing = a.starts_with("tasting ") || a.starts_with("bringing ");
                    let touching = a.starts_with("touching ") || a.starts_with("reaching for ");
                    let who = a.trim_start_matches("touching ").trim_start_matches("reaching for ");
                    if mouthing && humans.iter().any(|n| n == obj) {
                        mouth_human += 1;
                        if sim.age_years(c).to_f64() >= 12.0 {
                            mouth_human_adult += 1;
                            *who_mouths.entry(c.name.clone()).or_insert(0u64) += 1;
                        }
                    } else if mouthing {
                        mouth_obj += 1;
                    }
                    if touching && who.starts_with("Wolf") {
                        touch_wolf += 1;
                    }
                    if touching && humans.iter().any(|n| n == who) {
                        touch_human += 1;
                    }
                }
            }
        }
        if let Ok(who) = std::env::var("DUMP") {
            if let Some(c) = sim.creatures.iter().find(|c| c.name == who) {
                let id = c.id;
                sim.selected = Some(id);
                sim.step();
                let c = sim.creatures.iter().find(|c| c.id == id).unwrap();
                if let Some(t) = c.mind.as_ref().and_then(|m| m.last_trace.as_ref()) {
                    println!("==== TRACE {} at {}\n{}", who, export::clock(&sim), t.to_text());
                }
                sim.selected = None;
            }
        }
        if std::env::var_os("PAIRS").is_some() {
            for c in sim.creatures.iter().filter(|c| c.alive && c.kind == Kind::Human && c.female) {
                if let Some(p) = c.partner.and_then(|p| sim.creatures.iter().find(|o| o.id == p)) {
                    println!("pair {} ({:.0}) - {} dist {} hunger {:.2} preg {}", c.name, sim.age_years(c).to_f64(), p.name, (c.x - p.x).abs().max((c.y - p.y).abs()), c.body.signals.hunger.to_f64(), c.pregnant.is_some());
                }
            }
        }
        if h % 6 == 5 {
            let humans: Vec<String> = sim
                .creatures
                .iter()
                .filter(|c| c.alive && c.kind == Kind::Human)
                .map(|c| format!("{}({:.0}) h{:.2} t{:.2} {}", c.name, sim.age_years(c).to_f64(), c.body.signals.hunger.to_f64(), c.body.signals.thirst.to_f64(), c.action))
                .collect();
            println!("[{}] {}", export::clock(&sim), humans.join(" | "));
        }
    }
    if std::env::var_os("KNOW").is_some() {
        for c in sim.creatures.iter().filter(|c| c.alive && c.kind == Kind::Human) {
            let j = export::inspect_json(&sim, c.id);
            if let Some(k) = j.find("\"knowledge\":[") {
                let end = j[k..].find("],").map(|e| k + e).unwrap_or(j.len());
                println!("KNOW {}: {}", c.name, &j[k..end]);
            }
        }
    }
    if stats {
        println!("STATS adult mouthers {:?}", who_mouths);
        let bites = sim.history.iter().filter(|e| e.text.starts_with("A wolf")).count();
        println!(
            "STATS samples {samples} mouth_human {mouth_human} ({:.2}%) adults {mouth_human_adult} mouth_obj {mouth_obj} touch_human {touch_human} ({:.2}%) touch_wolf {touch_wolf} ({:.3}%) wolf_bite_events {bites}",
            mouth_human as f64 * 100.0 / samples.max(1) as f64,
            touch_human as f64 * 100.0 / samples.max(1) as f64,
            touch_wolf as f64 * 100.0 / samples.max(1) as f64
        );
    }
    let el = t1.elapsed();
    println!(
        "simulated {} h in {:.2?} => {:.0} game-seconds per real second; mind steps {}",
        hours,
        el,
        (hours * 3600) as f64 / el.as_secs_f64(),
        sim.mind_steps
    );
    println!("{}", export::status_json(&sim, 0.0));
    println!("profile: sense {:.2}s, mind {:.2}s, per mind step {:.1} us + sense {:.1} us", sim.prof_sense_ns as f64 / 1e9, sim.prof_mind_ns as f64 / 1e9, sim.prof_mind_ns as f64 / 1e3 / sim.mind_steps.max(1) as f64, sim.prof_sense_ns as f64 / 1e3 / sim.mind_steps.max(1) as f64);
    for e in &sim.history {
        println!("  {}  {}", export::clock(&Sim { tick: e.tick, ..clone_shell(&sim) }), e.text);
    }
}

fn clone_shell(s: &Sim) -> Sim {
    Sim {
        seed: s.seed,
        tick: s.tick,
        w: 1,
        h: 1,
        tiles: Vec::new(),
        objs: Vec::new(),
        creatures: Vec::new(),
        history: Vec::new(),
        firsts: Vec::new(),
        weather: s.weather,
        weather_until: 0,
        god_temp: s.god_temp,
        rain_until: 0,
        next_id: 0,
        selected: None,
        human_interval: 1,
        animal_interval: 1,
        mind_steps: 0,
        births: 0,
        deaths: 0,
        prof_sense_ns: 0,
        prof_mind_ns: 0,
        water_dist: Vec::new(),
        journal: Vec::new(),
        prof_rest_ns: 0,
    }
}
