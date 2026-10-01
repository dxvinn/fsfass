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
    for h in 0..hours {
        for _ in 0..secs {
            sim.step();
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
        prof_rest_ns: 0,
    }
}
