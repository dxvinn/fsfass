//! Sensors: the only translation from world truth to sensation.
//!
//! Outputs are sensation strengths with small seeded noise. Nothing that
//! identifies an object's kind, id or true physical values is passed on.

use crate::World;
use alife_core::rng::stream;
use alife_core::{fx, Fx, Rng};
use alife_interface::{Interoception, Percept, SensoryFrame, Visual};

/// Builds the sensory frame for agent index `ai` after the last world step.
pub fn sense(world: &World, ai: usize) -> SensoryFrame {
    let a = &world.agents[ai];
    let mut rng = Rng::new(world.seed, a.id as u64, world.tick, stream::SENSOR_NOISE);
    let mut percepts = Vec::new();
    for o in world.objects.iter().filter(|o| o.present) {
        let dx = o.x - a.x;
        let dy = o.y - a.y;
        let dist = World::dist(a.x, a.y, o.x, o.y);
        if dist > 14 {
            continue;
        }
        let p = &o.props;
        let noise = |r: &mut Rng, s: Fx| r.gaussish() * s;
        // Flickering emitters vary in brightness from moment to moment.
        let osc = rng.unit() - Fx::HALF;
        let brightness = (p.emit * (Fx::ONE + p.flicker * osc) + p.reflect * world.ambient.light * fx(0.6)
            + noise(&mut rng, fx(0.02)))
        .clamp01();
        let visual = Visual {
            hue_deg: wrap_deg(p.hue_deg + noise(&mut rng, fx(4.0))),
            saturation: (p.saturation + noise(&mut rng, fx(0.03))).clamp01(),
            brightness,
            flicker: (p.flicker * (fx(0.85) + rng.unit() * fx(0.3))).clamp01(),
            size: (p.size + noise(&mut rng, fx(0.02))).clamp01(),
            roundness: (p.roundness + noise(&mut rng, fx(0.03))).clamp01(),
            texture: (p.roughness + noise(&mut rng, fx(0.03))).clamp01(),
            gloss: (p.gloss + noise(&mut rng, fx(0.03))).clamp01(),
            motion: Fx::ZERO,
            face: Fx::ZERO,
        };
        // Warmth felt from this direction; much stronger on a hand/face held close to it.
        let mut warmth = world.radiant_from(o, a.x, a.y);
        if a.reaching.map(|(id, _)| id) == Some(o.id) {
            warmth = warmth.max(world.radiant_close(o));
        }
        let felt_warmth = (warmth + noise(&mut rng, fx(0.01))).clamp01();
        let touch = a.touched.and_then(|(id, t)| (id == o.id).then_some(t));
        let taste = a.tasted.and_then(|(id, t)| (id == o.id).then_some(t));
        percepts.push(Percept {
            token: world.token_for(a.id, o.id),
            dx,
            dy,
            dist,
            visual,
            felt_warmth,
            touch,
            taste,
            touched_me: false,
        });
    }
    // Present percepts in a canonical order that carries no identity: by token.
    percepts.sort_by_key(|p| p.token);
    let s = a.body.signals;
    SensoryFrame {
        tick: world.tick,
        asleep: a.body.asleep,
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
            reflex_active: s.reflex_hand || s.reflex_mouth,
            loneliness: Fx::ZERO,
            social_comfort: Fx::ZERO,
        },
        ambient: world.ambient,
        last_result: a.last_result,
    }
}

fn wrap_deg(d: Fx) -> Fx {
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
