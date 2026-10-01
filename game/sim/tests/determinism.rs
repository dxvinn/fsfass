//! The game world is deterministic: the same seed and the same god actions give
//! the same history, population and positions, tick for tick.
use genesis_sim::{export, Sim};

fn fingerprint(sim: &Sim) -> String {
    let mut s = export::status_json(sim, 0.0);
    s.push_str(&export::history_json(sim, 0));
    for c in &sim.creatures {
        s.push_str(&format!("|{}:{},{},{},{}", c.id, c.x, c.y, c.alive, c.health.raw()));
    }
    s
}

fn run(seed: u64) -> String {
    let mut sim = Sim::new(seed, 120, 90);
    sim.set_speed_lod(100);
    for t in 0..5400u32 {
        if t == 600 {
            sim.god("lightning", 40, 40);
            sim.god("create_human", 60, 45);
        }
        sim.step();
    }
    fingerprint(&sim)
}

#[test]
fn same_seed_same_world() {
    assert_eq!(run(5), run(5));
}

#[test]
fn different_seed_different_world() {
    assert_ne!(run(5), run(6));
}

#[test]
fn save_and_load_rebuilds_the_identical_world() {
    use genesis_sim::save::{fingerprint, save_text, Replay};
    let mut sim = Sim::new(9, 120, 90);
    for t in 0..7200u32 {
        match t {
            300 => sim.set_speed_lod(25),
            900 => {
                sim.god("fire", 50, 40);
                sim.god("create_grazer", 30, 30);
            }
            1500 => {
                let id = sim.creatures.iter().find(|c| c.kind == genesis_sim::Kind::Human).map(|c| c.id);
                sim.set_selected(id);
            }
            3000 => sim.set_speed_lod(1000),
            5000 => sim.set_selected(None),
            _ => {}
        }
        sim.step();
    }
    let text = save_text(&sim);
    let mut r = Replay::parse(&text).expect("parses");
    while !r.run(500) {}
    assert!(r.verified(), "rebuilt world differs");
    assert_eq!(fingerprint(&r.sim), fingerprint(&sim));
    assert_eq!(save_text(&r.sim), text, "re-saving the loaded world gives the same file");
}
