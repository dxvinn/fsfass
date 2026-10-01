//! Procedural terrain: value-noise elevation and moisture, biomes, rivers and lakes.

use alife_core::rng::mix64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Biome {
    DeepWater = 0,
    Water = 1,
    Sand = 2,
    Grass = 3,
    Forest = 4,
    Hills = 5,
    Mountain = 6,
    Snow = 7,
    River = 8,
}

#[derive(Clone, Debug)]
pub struct Tile {
    pub elev: i32,     // 0..1000
    pub moisture: i32, // 0..1000
    pub biome: Biome,
    /// Grass/forage biomass 0..1000.
    pub grass: i32,
    /// Fertility 0..1000 (soil quality).
    pub fertility: i32,
    /// Ticks until a burnt tile recovers (0 = not burnt).
    pub burnt: u32,
}

impl Tile {
    pub fn is_water(&self) -> bool {
        matches!(self.biome, Biome::DeepWater | Biome::Water | Biome::River)
    }
    pub fn walkable(&self) -> bool {
        !matches!(self.biome, Biome::DeepWater | Biome::Water | Biome::Mountain | Biome::River)
    }
}

fn lattice(seed: u64, x: i64, y: i64) -> i64 {
    (mix64(seed ^ mix64((x as u64).wrapping_mul(0x9E37_79B9) ^ mix64(y as u64))) >> 54) as i64 // 0..1023
}

/// Smooth value noise in 0..1023 at fixed-point coordinates (scale = cells per lattice step).
fn value_noise(seed: u64, x: i64, y: i64, scale: i64) -> i64 {
    let (gx, gy) = (x.div_euclid(scale), y.div_euclid(scale));
    let (fx, fy) = (x.rem_euclid(scale) * 1024 / scale, y.rem_euclid(scale) * 1024 / scale);
    // smoothstep weights
    let s = |t: i64| t * t * (3 * 1024 - 2 * t) / (1024 * 1024);
    let (sx, sy) = (s(fx), s(fy));
    let a = lattice(seed, gx, gy);
    let b = lattice(seed, gx + 1, gy);
    let c = lattice(seed, gx, gy + 1);
    let d = lattice(seed, gx + 1, gy + 1);
    let top = a + (b - a) * sx / 1024;
    let bot = c + (d - c) * sx / 1024;
    top + (bot - top) * sy / 1024
}

fn fbm(seed: u64, x: i64, y: i64, base: i64, octaves: u32) -> i64 {
    let mut total = 0;
    let mut amp = 512;
    let mut norm = 0;
    let mut scale = base;
    for o in 0..octaves {
        total += value_noise(seed.wrapping_add(o as u64 * 7919), x, y, scale.max(2)) * amp;
        norm += amp;
        amp /= 2;
        scale /= 2;
    }
    total / norm
}

pub fn generate(seed: u64, w: i32, h: i32) -> Vec<Tile> {
    let mut tiles = Vec::with_capacity((w * h) as usize);
    for y in 0..h {
        for x in 0..w {
            let (xi, yi) = (x as i64, y as i64);
            let mut e = fbm(seed ^ 0xE1E7, xi, yi, 48, 5); // 0..1023
            // Gentle island falloff so edges tend toward water.
            let dx = (2 * x - w).abs() as i64 * 1000 / w as i64;
            let dy = (2 * y - h).abs() as i64 * 1000 / h as i64;
            let edge = dx.max(dy);
            if edge > 700 {
                e -= (edge - 700) * 2;
            }
            let m = fbm(seed ^ 0x3015, xi, yi, 40, 4);
            let e = e.clamp(0, 1000) as i32;
            let m = m.clamp(0, 1000) as i32;
            let biome = if e < 300 {
                Biome::DeepWater
            } else if e < 380 {
                Biome::Water
            } else if e < 410 {
                Biome::Sand
            } else if e < 640 {
                if m > 560 { Biome::Forest } else { Biome::Grass }
            } else if e < 760 {
                if m > 640 { Biome::Forest } else { Biome::Hills }
            } else if e < 860 {
                Biome::Mountain
            } else {
                Biome::Snow
            };
            let fertility = match biome {
                Biome::Grass => 500 + m / 2,
                Biome::Forest => 600 + m / 3,
                Biome::Hills => 300 + m / 4,
                Biome::Sand => 100,
                _ => 0,
            }
            .min(1000);
            tiles.push(Tile { elev: e, moisture: m, biome, grass: fertility * 3 / 4, fertility, burnt: 0 });
        }
    }
    carve_rivers(seed, w, h, &mut tiles);
    tiles
}

/// Rivers: from several high springs, walk downhill to water; small lakes form in pits.
fn carve_rivers(seed: u64, w: i32, h: i32, tiles: &mut [Tile]) {
    let idx = |x: i32, y: i32| (y * w + x) as usize;
    let mut springs: Vec<(i32, i32)> = Vec::new();
    for i in 0..400u64 {
        let r = mix64(seed ^ 0x51_F1 ^ i);
        let x = (r % w as u64) as i32;
        let y = ((r >> 20) % h as u64) as i32;
        let t = &tiles[idx(x, y)];
        if t.elev > 680 && t.elev < 840 && springs.iter().all(|&(sx, sy)| (sx - x).abs() + (sy - y).abs() > 25) {
            springs.push((x, y));
        }
        if springs.len() >= 7 {
            break;
        }
    }
    for (sx, sy) in springs {
        let (mut x, mut y) = (sx, sy);
        for _ in 0..600 {
            let t = &tiles[idx(x, y)];
            if matches!(t.biome, Biome::Water | Biome::DeepWater) {
                break;
            }
            tiles[idx(x, y)].biome = Biome::River;
            // Lower the riverbed a bit so it keeps flowing.
            tiles[idx(x, y)].elev -= 15;
            let mut best = (i32::MAX, x, y);
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx >= w || ny >= h {
                    continue;
                }
                let n = &tiles[idx(nx, ny)];
                if n.biome == Biome::River {
                    continue;
                }
                if n.elev < best.0 {
                    best = (n.elev, nx, ny);
                }
            }
            if best.0 == i32::MAX {
                break;
            }
            if best.0 >= tiles[idx(x, y)].elev {
                // A pit: make a small lake here.
                for dy in -2..=2 {
                    for dx in -2..=2 {
                        let (lx, ly) = (x + dx, y + dy);
                        if lx >= 0 && ly >= 0 && lx < w && ly < h && dx * dx + dy * dy <= 5 {
                            tiles[idx(lx, ly)].biome = Biome::Water;
                        }
                    }
                }
                break;
            }
            x = best.1;
            y = best.2;
        }
    }
    // Moisten land next to water.
    let copy: Vec<bool> = tiles.iter().map(|t| t.is_water()).collect();
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            if copy[idx(x, y)] {
                continue;
            }
            let near = (-1..=1).any(|dy| (-1..=1).any(|dx| copy[idx(x + dx, y + dy)]));
            if near {
                let t = &mut tiles[idx(x, y)];
                t.moisture = (t.moisture + 200).min(1000);
                if t.biome == Biome::Grass {
                    t.fertility = (t.fertility + 150).min(1000);
                }
            }
        }
    }
}
