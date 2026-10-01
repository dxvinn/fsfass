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

fn tile_color(sim: &Sim, tx: i32, ty: i32) -> [i32; 3] {
    let (w, h) = (sim.w, sim.h);
    let (tx, ty) = (tx.clamp(0, w - 1), ty.clamp(0, h - 1));
    let t = sim.tile(tx, ty);
    let elev = |x: i32, y: i32| sim.tile(x.clamp(0, w - 1), y.clamp(0, h - 1)).elev;
    let grass = t.grass * 256 / t.fertility.max(1);
    let mut c = base(t.biome, grass);
    if !t.is_water() {
        // Hill shading from the elevation gradient (light from the north-west).
        let gx = elev(tx + 1, ty) - elev(tx - 1, ty);
        let gy = elev(tx, ty + 1) - elev(tx, ty - 1);
        let shade = (-(gx + gy) * 3).clamp(-60, 60);
        c = [c[0] + shade, c[1] + shade, c[2] + shade];
        if t.burnt > 0 {
            c = lerp(c, [60, 52, 46], 170);
        }
    } else {
        // Shallow water near shore is lighter.
        let shore = (-1..=1).any(|dy| (-1..=1).any(|dx| {
            let (x2, y2) = (tx + dx, ty + dy);
            x2 >= 0 && y2 >= 0 && x2 < w && y2 < h && !sim.tile(x2, y2).is_water()
        }));
        if shore {
            c = lerp(c, [120, 176, 204], 80);
        }
    }
    c
}

/// Colours are computed per tile and bilinearly blended between tile centres, so
/// coasts, forests and hills read as soft natural shapes instead of a grid.
pub fn terrain_rgba(sim: &Sim, scale: i32) -> Vec<u8> {
    let (w, h) = (sim.w, sim.h);
    let (pw, ph) = (w * scale, h * scale);
    let mut cols = Vec::with_capacity(((w + 1) * (h + 1)) as usize);
    for ty in -1..h {
        for tx in -1..w {
            cols.push(tile_color(sim, tx, ty));
        }
    }
    let at = |tx: i32, ty: i32| cols[((ty + 1) * (w + 1) + tx + 1) as usize];
    let mut out = vec![0u8; (pw * ph * 4) as usize];
    for py in 0..ph {
        // Position relative to tile centres, in 1/256 tile.
        let gy = (py * 256 + 128) / scale - 128;
        for px in 0..pw {
            // Domain warp (smooth value noise) so tile edges become organic shapes.
            let (wx, wy) = warp(sim.seed, px, py, scale);
            let gx = (px * 256 + 128) / scale - 128 + wx;
            let gy = gy + wy;
            let ty = gy.div_euclid(256);
            let fy = sharpen(gy.rem_euclid(256));
            let tx = gx.div_euclid(256);
            let fx = sharpen(gx.rem_euclid(256));
            let (tx, ty) = (tx.clamp(-1, w - 1), ty.clamp(-1, h - 1));
            let top = lerp(at(tx, ty), at((tx + 1).min(w - 1), ty), fx);
            let bot = lerp(at(tx, (ty + 1).min(h - 1)), at((tx + 1).min(w - 1), (ty + 1).min(h - 1)), fx);
            let c = lerp(top, bot, fy);
            // Fine grain so flat areas don't look like a spreadsheet.
            let n = (mix64(((px as u64) << 32) ^ py as u64 ^ sim.seed) >> 61) as i32 - 4;
            let i = ((py * pw + px) * 4) as usize;
            out[i] = (c[0] + n).clamp(0, 255) as u8;
            out[i + 1] = (c[1] + n).clamp(0, 255) as u8;
            out[i + 2] = (c[2] + n).clamp(0, 255) as u8;
            out[i + 3] = 255;
        }
    }
    out
}

/// Steeper blend between neighbouring tiles: crisp but not pixel-hard borders.
fn sharpen(f: i32) -> i32 {
    ((f - 128) * 5 / 2 + 128).clamp(0, 256)
}

fn lattice(seed: u64, x: i32, y: i32, k: u64) -> i32 {
    (mix64(seed ^ k ^ ((x as u32 as u64) << 32) ^ (y as u32 as u64)) >> 56) as i32 - 128
}

/// Two octaves of bilinear value noise, about +-0.45 tile.
fn warp(seed: u64, px: i32, py: i32, scale: i32) -> (i32, i32) {
    let mut wx = 0;
    let mut wy = 0;
    for (period, amp) in [(scale * 3 / 2, 80), (scale / 2 + 1, 35)] {
        let (cx, cy) = (px.div_euclid(period), py.div_euclid(period));
        let (fx, fy) = (px.rem_euclid(period) * 256 / period, py.rem_euclid(period) * 256 / period);
        for (k, out) in [(0x51u64, &mut wx), (0xa7u64, &mut wy)] {
            let a = lattice(seed, cx, cy, k);
            let b = lattice(seed, cx + 1, cy, k);
            let c = lattice(seed, cx, cy + 1, k);
            let d = lattice(seed, cx + 1, cy + 1, k);
            let top = a + (b - a) * fx / 256;
            let bot = c + (d - c) * fx / 256;
            *out += (top + (bot - top) * fy / 256) * amp / 128;
        }
    }
    (wx, wy)
}
