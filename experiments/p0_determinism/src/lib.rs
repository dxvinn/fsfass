//! P0 scenario: a small room with physical objects and bodies driven by a
//! seeded random policy (no mind yet). Used by the P0 binary and by tests.

use alife_biology::Body;
use alife_core::rng::stream;
use alife_core::{fx, Rng, StableHash};
use alife_interface::MotorCommand;
use alife_world::sensors::sense;
use alife_world::{ObjProps, World};

pub const TICKS: u64 = 20_000;
pub const CHECKPOINT_EVERY: u64 = 1_000;

/// Builds the P0 world for a seed: object positions and starting needs vary with the seed.
pub fn build_world(seed: u64) -> World {
    let mut w = World::new(seed, 12, 12, seed ^ 0xA5A5);
    let mut r = Rng::new(seed, 0, 0, stream::WORLD_SETUP);
    let place = |w: &mut World, props: ObjProps, r: &mut Rng| loop {
        let x = r.below(12) as i32;
        let y = r.below(12) as i32;
        if !w.objects.iter().any(|o| o.x == x && o.y == y) {
            w.add_object(x, y, props);
            break;
        }
    };
    place(&mut w, ObjProps::fire_a(), &mut r);
    place(&mut w, ObjProps::rock(), &mut r);
    place(&mut w, ObjProps::berries(), &mut r);
    place(&mut w, ObjProps::berries(), &mut r);
    place(&mut w, ObjProps::water_pool(), &mut r);
    place(&mut w, ObjProps::orange_flower(), &mut r);
    for i in 0..3 {
        let hunger = r.range_fx(fx(0.1), fx(0.6));
        let thirst = r.range_fx(fx(0.1), fx(0.6));
        loop {
            let x = r.below(12) as i32;
            let y = r.below(12) as i32;
            let free = !w.objects.iter().any(|o| o.x == x && o.y == y)
                && !w.agents.iter().any(|a| a.x == x && a.y == y);
            if free {
                w.add_agent(x, y, Body::child(3 + i, hunger, thirst));
                break;
            }
        }
    }
    w
}

/// Seeded random policy that only uses what the sensory frame provides.
pub fn random_command(world: &World, ai: usize) -> MotorCommand {
    let frame = sense(world, ai);
    let mut r = Rng::new(world.seed, world.agents[ai].id as u64, world.tick, stream::DRIVER);
    if frame.percepts.is_empty() {
        return MotorCommand::Wander { dir: r.below(8) as u8 };
    }
    // Usually follow through on a reach that was started (contact takes two steps).
    let a = &world.agents[ai];
    if let Some((oid, reach)) = a.reaching {
        if r.chance(fx(0.7)) {
            let target = world.token_for(a.id, oid);
            return match reach {
                alife_world::Reach::Hand => MotorCommand::Touch { target },
                alife_world::Reach::Mouth => MotorCommand::Mouth { target },
            };
        }
    }
    let t = frame.percepts[r.below(frame.percepts.len() as u64) as usize].token;
    match r.below(8) {
        0 => MotorCommand::Rest,
        1 | 2 => MotorCommand::Wander { dir: r.below(8) as u8 },
        3 => MotorCommand::Approach { target: t },
        4 => MotorCommand::Inspect { target: t },
        5 => MotorCommand::Touch { target: t },
        6 => MotorCommand::Mouth { target: t },
        _ => MotorCommand::Withdraw { target: t },
    }
}

/// Advances `world` to `until` ticks, returning state hashes at each checkpoint.
pub fn run(world: &mut World, until: u64) -> Vec<u64> {
    let mut hashes = Vec::new();
    while world.tick < until {
        let cmds: Vec<MotorCommand> = (0..world.agents.len()).map(|i| random_command(world, i)).collect();
        world.step(&cmds);
        if world.tick % CHECKPOINT_EVERY == 0 {
            hashes.push(world.state_hash());
        }
    }
    hashes
}

/// Full run from scratch.
pub fn run_seed(seed: u64) -> (u64, Vec<u64>, World) {
    let mut w = build_world(seed);
    let cps = run(&mut w, TICKS);
    (w.state_hash(), cps, w)
}
