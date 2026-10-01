//! Stylised terrain texture: per-pixel colour with hill shading, water depth,
//! shoreline foam, grass vigour and burn scars. `scale` pixels per tile.

use crate::terrain::Biome;
use crate::Sim;
use alife_core::rng::mix64;

fn lerp(a: [i32; 3], b: [i32; 3], t: i32) -> [i32; 3] {
    [a[0] + (b[0] - a[0]) * t / 256, a[1] + (b[1] - a[1]) * t / 256, a[2] + (b[2] - a[2]) * t / 256]
}

fn base(b: Biome, grass: i32) -> [i32; 3] {
    match b {
        Biome::DeepWater => [28, 70, 120],
        Biome::Water => [52, 112, 168],
        Biome::River => [62, 128, 182],
        Biome::Sand => [214, 196, 140],
        Biome::Grass => lerp([170, 160, 92], [98, 160, 72], grass.clamp(0, 256)),
        Biome::Forest => lerp([120, 120, 70], [58, 118, 58], grass.clamp(0, 256)),
        Biome::Hills => [140, 138, 100],
        Biome::Mountain => [128, 120, 112],
        Biome::Snow => [236, 240, 244],
    }
}

pub fn terrain_rgba(sim: &Sim, scale: i32) -> Vec<u8> {
    let (w, h) = (sim.w, sim.h);
    let (pw, ph) = (w * scale, h * scale);
    let mut out = vec![0u8; (pw * ph * 4) as usize];
    let elev = |x: i32, y: i32| sim.tile(x.clamp(0, w - 1), y.clamp(0, h - 1)).elev;
    for py in 0..ph {
        for px in 0..pw {
            let (tx, ty) = (px / scale, py / scale);
            let t = sim.tile(tx, ty);
            // Sub-tile position for smooth blending with the right/bottom neighbours.
            let fx_ = (px % scale) * 256 / scale;
            let fy_ = (py % scale) * 256 / scale;
            let grass = t.grass * 256 / t.fertility.max(1);
            let mut c = base(t.biome, grass);
            if !t.is_water() {
                let nx = sim.tile((tx + 1).min(w - 1), ty);
                let ny = sim.tile(tx, (ty + 1).min(h - 1));
                if !nx.is_water() && nx.biome != t.biome {
                    c = lerp(c, base(nx.biome, nx.grass * 256 / nx.fertility.max(1)), (fx_ - 128).max(0) / 2);
                }
                if !ny.is_water() && ny.biome != t.biome {
                    c = lerp(c, base(ny.biome, ny.grass * 256 / ny.fertility.max(1)), (fy_ - 128).max(0) / 2);
                }
                // Hill shading from the elevation gradient (light from the north-west).
                let gx = elev(tx + 1, ty) - elev(tx - 1, ty);
                let gy = elev(tx, ty + 1) - elev(tx, ty - 1);
                let shade = (-(gx + gy) * 3).clamp(-60, 60);
                c = [c[0] + shade, c[1] + shade, c[2] + shade];
                if t.burnt > 0 {
                    c = lerp(c, [60, 52, 46], 170);
                }
            } else {
                // Depth tint and shoreline foam.
                let shore = (-1..=1).any(|dy| (-1..=1).any(|dx| {
                    let (x2, y2) = (tx + dx, ty + dy);
                    x2 >= 0 && y2 >= 0 && x2 < w && y2 < h && !sim.tile(x2, y2).is_water()
                }));
                if shore {
                    c = lerp(c, [120, 176, 204], 70);
                }
            }
            // Fine grain noise so flat areas don't look like a spreadsheet.
            let n = (mix64(((px as u64) << 32) ^ py as u64 ^ sim.seed) >> 60) as i32 - 8;
            let i = ((py * pw + px) * 4) as usize;
            out[i] = (c[0] + n).clamp(0, 255) as u8;
            out[i + 1] = (c[1] + n).clamp(0, 255) as u8;
            out[i + 2] = (c[2] + n).clamp(0, 255) as u8;
            out[i + 3] = 255;
        }
    }
    out
}
