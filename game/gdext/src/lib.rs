//! Godot bridge. The Godot client never mutates simulation state directly: it
//! calls `advance` (time), `god` (powers) and `select` (observation), and reads
//! render data and JSON views.

use genesis_sim::{export, Kind, Sim};
use godot::prelude::*;
use std::time::Instant;

struct GenesisExtension;

#[gdextension]
unsafe impl ExtensionLibrary for GenesisExtension {}

#[derive(GodotClass)]
#[class(base = RefCounted)]
pub struct GenesisWorld {
    sim: Option<Sim>,
    debt: f64,
    actual_speed: f64,
    base: Base<RefCounted>,
}

#[godot_api]
impl IRefCounted for GenesisWorld {
    fn init(base: Base<RefCounted>) -> Self {
        GenesisWorld { sim: None, debt: 0.0, actual_speed: 0.0, base }
    }
}

#[godot_api]
impl GenesisWorld {
    #[func]
    fn new_world(&mut self, seed: i64, width: i64, height: i64) {
        self.sim = Some(Sim::new(seed as u64, width as i32, height as i32));
        self.debt = 0.0;
    }

    #[func]
    fn width(&self) -> i64 {
        self.sim.as_ref().map_or(0, |s| s.w as i64)
    }

    #[func]
    fn height(&self) -> i64 {
        self.sim.as_ref().map_or(0, |s| s.h as i64)
    }

    #[func]
    fn tick(&self) -> i64 {
        self.sim.as_ref().map_or(0, |s| s.tick as i64)
    }

    /// Advance time. `speed` is in game minutes per real second (1, 5, 25, 100, 1000).
    /// Runs as many 1-second ticks as fit in `budget_ms`; unfinished time is dropped
    /// (the sim slows down honestly instead of skipping thought) and reported.
    #[func]
    fn advance(&mut self, real_dt: f64, speed: i64, budget_ms: f64) -> i64 {
        let Some(sim) = self.sim.as_mut() else { return 0 };
        if speed <= 0 {
            self.actual_speed = 0.0;
            self.debt = 0.0;
            return 0;
        }
        sim.set_speed_lod(speed as u32);
        self.debt += real_dt * speed as f64 * 60.0;
        let want = self.debt.floor() as i64;
        let start = Instant::now();
        let mut done = 0i64;
        while done < want {
            sim.step();
            done += 1;
            if done % 16 == 0 && start.elapsed().as_secs_f64() * 1000.0 > budget_ms {
                break;
            }
        }
        self.debt -= want as f64;
        if done < want {
            self.debt = 0.0;
        }
        let inst = if real_dt > 0.0 { done as f64 / 60.0 / real_dt } else { 0.0 };
        self.actual_speed = self.actual_speed * 0.9 + inst * 0.1;
        done
    }

    /// Stylised RGBA terrain texture, `scale` pixels per tile.
    #[func]
    fn terrain_image(&self, scale: i64) -> PackedByteArray {
        let Some(sim) = self.sim.as_ref() else { return PackedByteArray::new() };
        PackedByteArray::from(genesis_sim::render::terrain_rgba(sim, scale.clamp(1, 16) as i32).as_slice())
    }

    /// Tiles: 4 bytes each = biome, elevation/4, grass/4, burnt flag.
    #[func]
    fn terrain_bytes(&self) -> PackedByteArray {
        let Some(sim) = self.sim.as_ref() else { return PackedByteArray::new() };
        let mut v = Vec::with_capacity(sim.tiles.len() * 4);
        for t in &sim.tiles {
            v.push(t.biome as u8);
            v.push((t.elev / 4).clamp(0, 255) as u8);
            v.push((t.grass / 4).clamp(0, 255) as u8);
            v.push(if t.burnt > 0 { 255 } else { 0 });
        }
        PackedByteArray::from(v.as_slice())
    }

    /// Creatures, stride 16: id, kind, x, y, prev_x, prev_y, action_code, age, female,
    /// health, asleep, pregnant, selected, skin, hair, height.
    #[func]
    fn creatures(&self) -> PackedFloat32Array {
        let Some(sim) = self.sim.as_ref() else { return PackedFloat32Array::new() };
        let mut v: Vec<f32> = Vec::with_capacity(sim.creatures.len() * 16);
        for c in sim.creatures.iter().filter(|c| c.alive) {
            let code = action_code(&c.action);
            v.extend_from_slice(&[
                c.id as f32,
                c.kind as u8 as f32,
                c.x as f32,
                c.y as f32,
                c.px as f32,
                c.py as f32,
                code,
                sim.age_years(c).to_f64() as f32,
                c.female as u8 as f32,
                c.health.to_f64() as f32,
                (c.kind == Kind::Human && c.body.asleep) as u8 as f32,
                c.pregnant.is_some() as u8 as f32,
                (sim.selected == Some(c.id)) as u8 as f32,
                c.traits.skin.to_f64() as f32,
                c.traits.hair.to_f64() as f32,
                c.traits.height.to_f64() as f32,
            ]);
        }
        PackedFloat32Array::from(v.as_slice())
    }

    /// Objects, stride 6: id, kind, x, y, amount (portions or fuel), cooked.
    #[func]
    fn objects(&self) -> PackedFloat32Array {
        let Some(sim) = self.sim.as_ref() else { return PackedFloat32Array::new() };
        let mut v: Vec<f32> = Vec::with_capacity(sim.objs.len() * 6);
        for o in sim.objs.iter().filter(|o| o.alive) {
            v.extend_from_slice(&[
                o.id as f32,
                o.kind as u8 as f32,
                o.x as f32,
                o.y as f32,
                o.props.portions.max(0) as f32,
                o.cooked as u8 as f32,
            ]);
        }
        PackedFloat32Array::from(v.as_slice())
    }

    #[func]
    fn status(&self) -> GString {
        self.sim.as_ref().map(|s| export::status_json(s, self.actual_speed)).unwrap_or_default().as_str().into()
    }

    #[func]
    fn inspect(&self, id: i64) -> GString {
        self.sim.as_ref().map(|s| export::inspect_json(s, id as u32)).unwrap_or_default().as_str().into()
    }

    #[func]
    fn history(&self, from: i64) -> GString {
        self.sim.as_ref().map(|s| export::history_json(s, from.max(0) as usize)).unwrap_or_default().as_str().into()
    }

    #[func]
    fn select(&mut self, id: i64) {
        if let Some(s) = self.sim.as_mut() {
            s.selected = if id < 0 { None } else { Some(id as u32) };
        }
    }

    /// Nearest living creature within `radius` tiles of (x, y), or -1.
    #[func]
    fn creature_at(&self, x: f64, y: f64, radius: f64) -> i64 {
        let Some(sim) = self.sim.as_ref() else { return -1 };
        sim.creatures
            .iter()
            .filter(|c| c.alive)
            .map(|c| {
                let dx = c.x as f64 + 0.5 - x;
                let dy = c.y as f64 + 0.5 - y;
                (dx * dx + dy * dy, c.id)
            })
            .filter(|(d, _)| *d <= radius * radius)
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .map(|(_, id)| id as i64)
            .unwrap_or(-1)
    }

    #[func]
    fn tooltip(&self, id: i64) -> GString {
        let Some(sim) = self.sim.as_ref() else { return GString::new() };
        sim.creatures.iter().find(|c| c.id == id as u32).map(|c| export::tooltip(sim, c)).unwrap_or_default().as_str().into()
    }

    #[func]
    fn god(&mut self, power: GString, x: i64, y: i64) -> GString {
        let Some(sim) = self.sim.as_mut() else { return GString::new() };
        GString::from(sim.god(&power.to_string(), x as i32, y as i32).as_str())
    }

    /// Daylight 0..1 (for the day/night tint).
    #[func]
    fn light(&self) -> f64 {
        self.sim.as_ref().map_or(1.0, |s| s.light().to_f64())
    }

    #[func]
    fn weather(&self) -> i64 {
        self.sim.as_ref().map_or(0, |s| s.weather as i64)
    }
}

/// Coarse action category for animation: 0 idle, 1 walk, 2 eat, 3 drink, 4 sleep,
/// 5 flee/withdraw, 6 touch/social, 7 hurt, 8 hunt/attack, 9 watch.
fn action_code(a: &str) -> f32 {
    let a = a.to_lowercase();
    if a.contains("sleep") {
        4.0
    } else if a.contains("drink") {
        3.0
    } else if a.contains("tasting") || a.contains("eating") || a.contains("grazing") || a.contains("mouth") {
        2.0
    } else if a.contains("flee") || a.contains("backing") {
        5.0
    } else if a.contains("pain") {
        7.0
    } else if a.contains("hunt") || a.contains("attack") {
        8.0
    } else if a.contains("touching") || a.contains("reaching") {
        6.0
    } else if a.contains("watching") {
        9.0
    } else if a.contains("approach") || a.contains("wander") || a.contains("roam") || a.contains("looking") || a.contains("going") {
        1.0
    } else {
        0.0
    }
}
