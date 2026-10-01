//! Stylised terrain texture, `scale` pixels per tile.
//!
//! The picture is painted from smooth fields, not from tile colours:
//! - coasts come from a signed distance to the shore (interpolated and roughened
//!   with noise), which gives a shallow-water band, a foam line and wet sand;
//! - land biomes are chosen per pixel by noisy "votes" of the nearby tiles, so
//!   borders are crisp but organic instead of blurred or blocky;
//! - rivers are smoothed centre lines through the river tiles, drawn by distance;
//! - grass vigour, forest floor and canopy clumps, hill shading, rock strata,
//!   snow caps and burn scars are layered on top.
//!
//! Alpha is a water mask for the client's shimmer shader: land is 255, water is
//! 204..=254 where lower means deeper (`254 - depth * 50`, depth 0..1).
//! Rendering never touches simulation state; everything is deterministic for a
//! given world.

use crate::terrain::Biome;
use crate::Sim;
use alife_core::rng::mix64;

const N_LAND: usize = 6; // Sand, Grass, Forest, Hills, Mountain, Snow
const SAND: usize = 0;
const GRASS: usize = 1;
const FOREST: usize = 2;
const HILLS: usize = 3;
const MOUNTAIN: usize = 4;
const SNOW: usize = 5;

fn land_index(b: Biome) -> Option<usize> {
    match b {
        Biome::Sand => Some(SAND),
        Biome::Grass => Some(GRASS),
        Biome::Forest => Some(FOREST),
        Biome::Hills => Some(HILLS),
        Biome::Mountain => Some(MOUNTAIN),
        Biome::Snow => Some(SNOW),
        _ => None,
    }
}

/// Per-tile fields, bilinearly interpolated per pixel.
#[derive(Clone, Copy, Default)]
struct Cell {
    votes: [f32; N_LAND],
    /// Signed distance to the sea/lake shore in tiles (+ water, - land).
    sd: f32,
    shade: f32,
    grass: f32,
    burnt: f32,
    elev: f32,
}

impl Cell {
    #[inline]
    fn lerp(a: &Cell, b: &Cell, t: f32) -> Cell {
        let l = |x: f32, y: f32| x + (y - x) * t;
        let mut votes = [0.0; N_LAND];
        for i in 0..N_LAND {
            votes[i] = l(a.votes[i], b.votes[i]);
        }
        Cell {
            votes,
            sd: l(a.sd, b.sd),
            shade: l(a.shade, b.shade),
            grass: l(a.grass, b.grass),
            burnt: l(a.burnt, b.burnt),
            elev: l(a.elev, b.elev),
        }
    }
}

/// Smooth value noise on a lattice, in tile units, with `N` independent channels
/// sharing one lattice (one lookup gives all of them).
struct Noise<const N: usize> {
    cols: i32,
    rows: i32,
    inv: f32,
    v: Vec<[f32; N]>,
}

impl<const N: usize> Noise<N> {
    fn new(seed: u64, key: u64, w: i32, h: i32, period: f32) -> Self {
        let cols = (w as f32 / period).ceil() as i32 + 5;
        let rows = (h as f32 / period).ceil() as i32 + 5;
        let mut v = Vec::with_capacity((cols * rows) as usize);
        for y in 0..rows {
            for x in 0..cols {
                let mut c = [0.0f32; N];
                for (k, o) in c.iter_mut().enumerate() {
                    let key = key.wrapping_mul(0x9E37_79B9).wrapping_add(k as u64 * 0x1000_0001);
                    let r = mix64(seed ^ key ^ ((x as u64) << 32) ^ y as u64);
                    *o = (r >> 40) as f32 / (1u64 << 23) as f32 - 1.0;
                }
                v.push(c);
            }
        }
        Noise { cols, rows, inv: 1.0 / period, v }
    }

    /// Values in about -1..1.
    #[inline(always)]
    fn at(&self, x: f32, y: f32) -> [f32; N] {
        let fx = (x * self.inv + 2.0).clamp(0.0, (self.cols - 2) as f32 - 0.001);
        let fy = (y * self.inv + 2.0).clamp(0.0, (self.rows - 2) as f32 - 0.001);
        let (ix, iy) = (fx as i32, fy as i32);
        let (tx, ty) = (fx - ix as f32, fy - iy as f32);
        let tx = tx * tx * (3.0 - 2.0 * tx);
        let ty = ty * ty * (3.0 - 2.0 * ty);
        let i = (iy * self.cols + ix) as usize;
        let c = self.cols as usize;
        let (a, b, cc, d) = (&self.v[i], &self.v[i + 1], &self.v[i + c], &self.v[i + c + 1]);
        let mut out = [0.0f32; N];
        for k in 0..N {
            let top = a[k] + (b[k] - a[k]) * tx;
            let bot = cc[k] + (d[k] - cc[k]) * tx;
            out[k] = top + (bot - top) * ty;
        }
        out
    }

    #[inline(always)]
    fn at1(&self, x: f32, y: f32) -> f32 {
        self.at(x, y)[0]
    }
}

type Rgb = [f32; 3];

#[inline]
fn mix(a: Rgb, b: Rgb, t: f32) -> Rgb {
    let t = t.clamp(0.0, 1.0);
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

#[inline]
fn add(a: Rgb, v: f32) -> Rgb {
    [a[0] + v, a[1] + v, a[2] + v]
}

#[inline]
fn mul(a: Rgb, v: f32) -> Rgb {
    [a[0] * v, a[1] * v, a[2] * v]
}

#[inline]
fn smooth(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

struct Seg {
    a: (f32, f32),
    b: (f32, f32),
}

struct Ctx {
    w: i32,
    h: i32,
    cells: Vec<Cell>, // (w + 2) x (h + 2), padded by one tile
    segs: Vec<Seg>,
    seg_start: Vec<u32>, // per tile, CSR into seg_idx
    seg_idx: Vec<u32>,
    /// Warp x, warp y, vote 1, vote 2 (period about 2 tiles).
    coarse: Noise<4>,
    /// Fine warp x, warp y, vote 3 (period about 0.6 tile).
    detail: Noise<3>,
    big: Noise<1>,
    mid: Noise<1>,
    fine: Noise<1>,
    rock: Noise<1>,
    canopy: Noise<1>,
    seed: u64,
}

impl Ctx {
    #[inline(always)]
    fn cell(&self, x: f32, y: f32) -> Cell {
        let gx = (x - 0.5).clamp(-1.0, self.w as f32 - 0.001);
        let gy = (y - 0.5).clamp(-1.0, self.h as f32 - 0.001);
        // Truncation is floor here: shift by one so the value is never negative.
        let (ix, iy) = ((gx + 1.0) as i32 - 1, (gy + 1.0) as i32 - 1);
        let (fx, fy) = (gx - ix as f32, gy - iy as f32);
        let pw = (self.w + 2) as usize;
        let i = ((iy + 1) as usize) * pw + (ix + 1) as usize;
        let top = Cell::lerp(&self.cells[i], &self.cells[i + 1], fx);
        let bot = Cell::lerp(&self.cells[i + pw], &self.cells[i + pw + 1], fx);
        Cell::lerp(&top, &bot, fy)
    }

    /// Distance from (x, y) to the nearest river centre line, in tiles.
    #[inline]
    fn river_dist(&self, x: f32, y: f32) -> f32 {
        if x < 0.0 || y < 0.0 {
            return f32::MAX;
        }
        let (tx, ty) = (x as i32, y as i32);
        if tx >= self.w || ty >= self.h {
            return f32::MAX;
        }
        let t = (ty * self.w + tx) as usize;
        let (s, e) = (self.seg_start[t] as usize, self.seg_start[t + 1] as usize);
        let mut best = f32::MAX;
        for &k in &self.seg_idx[s..e] {
            let sg = &self.segs[k as usize];
            let (dx, dy) = (sg.b.0 - sg.a.0, sg.b.1 - sg.a.1);
            let (px, py) = (x - sg.a.0, y - sg.a.1);
            let l2 = dx * dx + dy * dy;
            let u = if l2 > 0.0 { ((px * dx + py * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
            let (ex, ey) = (px - dx * u, py - dy * u);
            best = best.min(ex * ex + ey * ey);
        }
        best.sqrt()
    }
}

fn build(sim: &Sim) -> Ctx {
    let (w, h) = (sim.w, sim.h);
    let idx = |x: i32, y: i32| (y.clamp(0, h - 1) * w + x.clamp(0, w - 1)) as usize;
    let n = (w * h) as usize;
    let tiles = &sim.tiles;
    let sea = |t: &crate::terrain::Tile| matches!(t.biome, Biome::Water | Biome::DeepWater);

    // Chamfer distance (tile centres) to the other side of the shore.
    let inf = 1.0e6f32;
    let mut d_land = vec![inf; n]; // for water tiles: distance to land
    let mut d_water = vec![inf; n]; // for land tiles: distance to water
    for i in 0..n {
        if sea(&tiles[i]) {
            d_water[i] = 0.0;
        } else {
            d_land[i] = 0.0;
        }
    }
    for d in [&mut d_land, &mut d_water] {
        let s2 = std::f32::consts::SQRT_2;
        for y in 0..h {
            for x in 0..w {
                let i = (y * w + x) as usize;
                let mut v = d[i];
                if x > 0 { v = v.min(d[i - 1] + 1.0); }
                if y > 0 {
                    v = v.min(d[i - w as usize] + 1.0);
                    if x > 0 { v = v.min(d[i - w as usize - 1] + s2); }
                    if x < w - 1 { v = v.min(d[i - w as usize + 1] + s2); }
                }
                d[i] = v;
            }
        }
        for y in (0..h).rev() {
            for x in (0..w).rev() {
                let i = (y * w + x) as usize;
                let mut v = d[i];
                if x < w - 1 { v = v.min(d[i + 1] + 1.0); }
                if y < h - 1 {
                    v = v.min(d[i + w as usize] + 1.0);
                    if x < w - 1 { v = v.min(d[i + w as usize + 1] + s2); }
                    if x > 0 { v = v.min(d[i + w as usize - 1] + s2); }
                }
                d[i] = v;
            }
        }
    }

    // Smoothed elevation for shading.
    let mut es = vec![0.0f32; n];
    for y in 0..h {
        for x in 0..w {
            let mut s = 0.0;
            let mut k = 0.0;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let wgt = [1.0, 2.0, 1.0][(dx + 1) as usize] * [1.0, 2.0, 1.0][(dy + 1) as usize];
                    s += tiles[idx(x + dx, y + dy)].elev as f32 * wgt;
                    k += wgt;
                }
            }
            es[(y * w + x) as usize] = s / k;
        }
    }

    // One-hot land votes, then half own / half blurred so single tiles survive.
    let mut own = vec![[0.0f32; N_LAND]; n];
    for i in 0..n {
        if let Some(b) = land_index(tiles[i].biome) {
            own[i][b] = 1.0;
        }
    }
    let pw = (w + 2) as usize;
    let mut cells = vec![Cell::default(); pw * (h + 2) as usize];
    for py in 0..h + 2 {
        for px in 0..w + 2 {
            let (x, y) = (px - 1, py - 1);
            let i = idx(x, y);
            let t = &tiles[i];
            let mut votes = [0.0f32; N_LAND];
            let mut k = 0.0;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let wgt = [1.0, 2.0, 1.0][(dx + 1) as usize] * [1.0, 2.0, 1.0][(dy + 1) as usize];
                    let o = &own[idx(x + dx, y + dy)];
                    for b in 0..N_LAND {
                        votes[b] += o[b] * wgt;
                    }
                    k += wgt;
                }
            }
            for b in 0..N_LAND {
                votes[b] = votes[b] / k * 0.5 + own[i][b] * 0.5;
            }
            let sd = if sea(t) { (d_land[i] - 0.5).min(12.0) } else { -(d_water[i] - 0.5).min(12.0) };
            let (cx, cy) = (x.clamp(0, w - 1), y.clamp(0, h - 1));
            let e = |x: i32, y: i32| es[idx(x, y)];
            let gx = e(cx + 1, cy) - e(cx - 1, cy);
            let gy = e(cx, cy + 1) - e(cx, cy - 1);
            let shade = (-(gx * 0.8 + gy) * 1.6).clamp(-70.0, 70.0);
            let grass = if t.fertility > 0 { (t.grass as f32 / t.fertility as f32).clamp(0.0, 1.0) } else { 1.0 };
            cells[py as usize * pw + px as usize] = Cell {
                votes,
                sd,
                shade,
                grass,
                burnt: if t.burnt > 0 { 1.0 } else { 0.0 },
                elev: es[i],
            };
        }
    }

    // Rivers: chains through river tiles, smoothed, as line segments.
    let river = |x: i32, y: i32| x >= 0 && y >= 0 && x < w && y < h && tiles[(y * w + x) as usize].biome == Biome::River;
    let dirs = [(1, 0), (0, 1), (-1, 0), (0, -1)];
    let degree = |x: i32, y: i32| dirs.iter().filter(|&&(dx, dy)| river(x + dx, y + dy)).count();
    let mut used = std::collections::HashSet::new();
    let key = |a: (i32, i32), b: (i32, i32)| if a < b { (a, b) } else { (b, a) };
    let mut chains: Vec<Vec<(f32, f32)>> = Vec::new();
    let centre = |p: (i32, i32)| (p.0 as f32 + 0.5, p.1 as f32 + 0.5);
    let mut starts: Vec<(i32, i32)> = Vec::new();
    for y in 0..h {
        for x in 0..w {
            if river(x, y) && degree(x, y) != 2 {
                starts.push((x, y));
            }
        }
    }
    for y in 0..h {
        for x in 0..w {
            if river(x, y) && degree(x, y) == 2 {
                starts.push((x, y)); // loops and leftovers
            }
        }
    }
    for &s in &starts {
        for &(dx, dy) in &dirs {
            let nb = (s.0 + dx, s.1 + dy);
            if !river(nb.0, nb.1) || used.contains(&key(s, nb)) {
                continue;
            }
            used.insert(key(s, nb));
            let mut chain = vec![centre(s), centre(nb)];
            let (mut prev, mut cur) = (s, nb);
            while degree(cur.0, cur.1) == 2 && cur != s {
                let next = dirs
                    .iter()
                    .map(|&(dx, dy)| (cur.0 + dx, cur.1 + dy))
                    .find(|&p| p != prev && river(p.0, p.1) && !used.contains(&key(cur, p)));
                let Some(next) = next else { break };
                used.insert(key(cur, next));
                chain.push(centre(next));
                prev = cur;
                cur = next;
            }
            chains.push(chain);
        }
    }
    // River mouths and lone river tiles.
    for y in 0..h {
        for x in 0..w {
            if !river(x, y) {
                continue;
            }
            let mut joined = false;
            for &(dx, dy) in &dirs {
                let (nx, ny) = (x + dx, y + dy);
                if nx >= 0 && ny >= 0 && nx < w && ny < h && sea(&tiles[(ny * w + nx) as usize]) {
                    chains.push(vec![centre((x, y)), centre((nx, ny))]);
                    joined = true;
                    break;
                }
            }
            if !joined && degree(x, y) == 0 {
                chains.push(vec![centre((x, y))]);
            }
        }
    }
    let mut segs = Vec::new();
    for mut c in chains {
        // Chaikin corner cutting, endpoints kept.
        for _ in 0..3 {
            if c.len() < 3 {
                break;
            }
            let mut o = vec![c[0]];
            for k in 0..c.len() - 1 {
                let (a, b) = (c[k], c[k + 1]);
                let q = (a.0 * 0.75 + b.0 * 0.25, a.1 * 0.75 + b.1 * 0.25);
                let r = (a.0 * 0.25 + b.0 * 0.75, a.1 * 0.25 + b.1 * 0.75);
                if k > 0 {
                    o.push(q);
                }
                if k + 2 < c.len() {
                    o.push(r);
                }
            }
            o.push(*c.last().unwrap());
            c = o;
        }
        if c.len() == 1 {
            segs.push(Seg { a: c[0], b: c[0] });
        }
        for k in 0..c.len().saturating_sub(1) {
            segs.push(Seg { a: c[k], b: c[k + 1] });
        }
    }
    // Bucket segments by tile (with a margin for width and warp).
    let mut buckets: Vec<Vec<u32>> = vec![Vec::new(); n];
    for (k, s) in segs.iter().enumerate() {
        let m = 1.0;
        let x0 = (s.a.0.min(s.b.0) - m).floor().max(0.0) as i32;
        let x1 = (s.a.0.max(s.b.0) + m).floor().min((w - 1) as f32) as i32;
        let y0 = (s.a.1.min(s.b.1) - m).floor().max(0.0) as i32;
        let y1 = (s.a.1.max(s.b.1) + m).floor().min((h - 1) as f32) as i32;
        for y in y0..=y1 {
            for x in x0..=x1 {
                buckets[(y * w + x) as usize].push(k as u32);
            }
        }
    }
    let mut seg_start = Vec::with_capacity(n + 1);
    let mut seg_idx = Vec::new();
    for b in &buckets {
        seg_start.push(seg_idx.len() as u32);
        seg_idx.extend_from_slice(b);
    }
    seg_start.push(seg_idx.len() as u32);

    let s = sim.seed;
    Ctx {
        w,
        h,
        cells,
        segs,
        seg_start,
        seg_idx,
        coarse: Noise::new(s, 1, w, h, 2.0),
        detail: Noise::new(s, 3, w, h, 0.6),
        big: Noise::new(s, 8, w, h, 7.0),
        mid: Noise::new(s, 9, w, h, 1.3),
        fine: Noise::new(s, 10, w, h, 0.28),
        rock: Noise::new(s, 11, w, h, 0.5),
        canopy: Noise::new(s, 12, w, h, 0.42),
        seed: s,
    }
}

// Palette.
const SAND_C: Rgb = [224.0, 206.0, 152.0];
const WET_SAND: Rgb = [170.0, 152.0, 108.0];
const GRASS_LUSH: Rgb = [100.0, 156.0, 68.0];
const GRASS_DRY: Rgb = [178.0, 166.0, 98.0];
const GRASS_WARM: Rgb = [136.0, 166.0, 70.0];
const GRASS_COOL: Rgb = [78.0, 138.0, 76.0];
const FOREST_C: Rgb = [52.0, 94.0, 50.0];
const FOREST_DRY: Rgb = [102.0, 104.0, 60.0];
const CANOPY_DARK: Rgb = [34.0, 70.0, 40.0];
const CANOPY_LIT: Rgb = [70.0, 120.0, 60.0];
const HILLS_C: Rgb = [140.0, 152.0, 84.0];
const HILLS_DRY: Rgb = [176.0, 162.0, 108.0];
const ROCK: Rgb = [142.0, 132.0, 116.0];
const ROCK_DARK: Rgb = [98.0, 92.0, 84.0];
const SNOW_C: Rgb = [240.0, 244.0, 250.0];
const SNOW_SHADOW: Rgb = [176.0, 192.0, 222.0];
const CHAR: Rgb = [62.0, 54.0, 48.0];
const ASH: Rgb = [128.0, 122.0, 114.0];
const SCORCH: Rgb = [112.0, 90.0, 62.0];
const SHALLOW: Rgb = [104.0, 192.0, 190.0];
const MIDWATER: Rgb = [50.0, 122.0, 172.0];
const DEEP: Rgb = [30.0, 72.0, 128.0];
const ABYSS: Rgb = [20.0, 50.0, 100.0];
const RIVER_C: Rgb = [70.0, 146.0, 186.0];
const FOAM: Rgb = [232.0, 244.0, 240.0];
const MUD: Rgb = [104.0, 98.0, 70.0];

/// Per-biome noise mixing weights (cos, sin of b * 2.4 and sin of b * 4.08).
const VOTE_K: [[f32; 3]; N_LAND] = [
    [1.0, 0.0, 0.0],
    [-0.737, 0.675, -0.809],
    [0.087, -0.996, 0.951],
    [0.608, 0.794, -0.309],
    [-0.984, -0.176, -0.588],
    [0.843, -0.537, 0.951],
];

/// Per-pixel inputs shared by the colour functions.
struct Px<'a> {
    ctx: &'a Ctx,
    c: Cell,
    x0: f32,
    y0: f32,
    mid: f32,
    fine: f32,
    big: f32,
    hash: u64,
    grain: f32,
}

/// Half-width of the anti-aliased band at hard edges, in tiles.
const AA: f32 = 0.035;

fn water_color(q: &Px, depth: f32) -> (Rgb, f32) {
    let (c, mid, fine) = (&q.c, q.mid, q.fine);
    // Depth: distance from shore near the coast, sea-floor elevation further out.
    let off = smooth(0.2, 2.2, depth);
    let floor = c.elev + mid * 12.0;
    let mut col = mix(SHALLOW, MIDWATER, smooth(0.1, 2.6, depth));
    col = mix(col, DEEP, smooth(372.0, 270.0, floor) * off);
    col = mix(col, ABYSS, smooth(270.0, 140.0, floor) * off);
    col = add(col, q.big * 6.0 + q.grain * 0.4);
    // Foam line hugging the shore, and a fainter swash line further out.
    let foam = 1.0 - smooth(0.05, 0.2 + fine * 0.06, depth);
    let swash = (1.0 - ((depth - 0.42 - mid * 0.08).abs() / 0.05).min(1.0)) * 0.35 * (0.5 + 0.5 * fine);
    col = mix(col, FOAM, foam * 0.85 + swash);
    let d01 = (depth / 3.0).max(smooth(380.0, 200.0, floor) * off).clamp(0.0, 1.0);
    (col, 254.0 - d01 * 50.0)
}

fn land_color(q: &Px, best: usize, edge: f32) -> Rgb {
    let (ctx, c, x0, y0, mid, fine, big) = (q.ctx, &q.c, q.x0, q.y0, q.mid, q.fine, q.big);
    let shade = c.shade;
    let lush = c.grass;
    match best {
        SAND => {
            let ripple = ((x0 * 3.1 + y0 * 1.3 + mid * 2.0) * 3.0).sin() * 3.0;
            add(mix(SAND_C, [210.0, 188.0, 140.0], 0.5 + 0.5 * big), ripple + fine * 5.0 + shade * 0.35)
        }
        GRASS => {
            let mut g = mix(GRASS_DRY, GRASS_LUSH, lush * 1.1 - 0.05);
            let hue = big * 0.8 + mid * 0.35;
            g = if hue > 0.0 { mix(g, GRASS_WARM, hue * 0.6) } else { mix(g, GRASS_COOL, -hue * 0.6) };
            g = add(g, fine * 7.0 + shade * 0.45);
            // Tufts.
            let t = (q.hash & 0xff) as u32;
            if t < 12 {
                g = add(g, -9.0);
            } else if t < 20 {
                g = add(g, 7.0);
            }
            mix(g, add(g, -16.0), 0.5 - edge * 0.5)
        }
        FOREST => {
            let mut f = mix(FOREST_DRY, FOREST_C, lush * 1.2 - 0.1);
            f = add(f, big * 5.0 + fine * 5.0);
            // Soft canopy mottling: darker hollows, lit crowns on the north-west.
            let cn = ctx.canopy.at1(x0, y0) + mid * 0.3;
            let cl = ctx.canopy.at1(x0 - 0.08, y0 - 0.08) + mid * 0.3;
            let crown = smooth(-0.35, 0.45, cn);
            let rim = ((cn - cl) * 5.0).clamp(-1.0, 1.0);
            let cc = mix(CANOPY_DARK, CANOPY_LIT, 0.4 + rim * 0.22 + lush * 0.15);
            f = mix(f, mix(cc, FOREST_DRY, (1.0 - lush) * 0.4), crown * 0.75);
            f = add(f, shade * 0.4);
            // Darker margin where the forest meets open ground.
            mix(add(f, -14.0), f, edge)
        }
        HILLS => {
            let hl = mix(HILLS_DRY, HILLS_C, lush);
            let r = ctx.rock.at1(x0, y0);
            let mut hcol = add(hl, big * 6.0 + fine * 6.0 + shade * 0.9);
            if r > 0.62 {
                hcol = mix(hcol, add(ROCK, shade * 0.9 + 10.0), smooth(0.62, 0.7, r) * 0.7);
            }
            hcol
        }
        MOUNTAIN | SNOW => {
            // Rock with strata and an embossed bump.
            let r0 = ctx.rock.at1(x0, y0) + fine * 0.5;
            let r1 = ctx.rock.at1(x0 - 0.05, y0 - 0.05) + ctx.fine.at1(x0 - 0.05, y0 - 0.05) * 0.5;
            let bump = ((r0 - r1) * 120.0).clamp(-40.0, 40.0);
            let strata = ((c.elev * 0.09 + mid * 2.5).sin() * 0.5 + 0.5) * 0.5;
            let mut m = mix(ROCK, ROCK_DARK, strata + big * 0.2);
            m = add(m, shade * 1.3 + bump);
            let snow_line = 838.0 - mid * 18.0 - fine * 10.0;
            let snowy = if best == SNOW { 1.0 } else { smooth(snow_line - 4.0, snow_line + 4.0, c.elev + shade * 0.3) };
            if snowy > 0.0 {
                let lit = ((shade * 1.2 + bump * 0.6 + 40.0) / 80.0).clamp(0.0, 1.0);
                let s = mix(SNOW_SHADOW, SNOW_C, lit);
                m = mix(m, s, snowy);
            }
            m
        }
        _ => GRASS_LUSH,
    }
}

fn pixel(ctx: &Ctx, px: i32, py: i32, scale: i32) -> ([f32; 3], u8) {
    let inv = 1.0 / scale as f32;
    let x0 = (px as f32 + 0.5) * inv;
    let y0 = (py as f32 + 0.5) * inv;
    let [wx, wy, n1, n2] = ctx.coarse.at(x0, y0);
    let [dx, dy, n3] = ctx.detail.at(x0, y0);
    let x = x0 + wx * 0.34 + dx * 0.12;
    let y = y0 + wy * 0.34 + dy * 0.12;
    let hash = mix64(ctx.seed ^ ((px as u64) << 32) ^ py as u64 ^ 0x7E44);
    let q = Px {
        ctx,
        c: ctx.cell(x, y),
        x0,
        y0,
        mid: ctx.mid.at1(x0, y0),
        fine: ctx.fine.at1(x0, y0),
        big: ctx.big.at1(x0, y0),
        hash,
        grain: ((hash >> 59) as f32 - 16.0) * 0.17, // about -2.7..2.5
    };
    let (c, mid, fine) = (&q.c, q.mid, q.fine);

    // --- Sea and lakes (the shore line itself is anti-aliased below).
    let sd = c.sd + mid * 0.22 + fine * 0.08;
    if sd > AA {
        let (col, a) = water_color(&q, sd);
        return (col, a.round() as u8);
    }

    // --- Land biome by noisy votes.
    let mut best = SAND;
    let mut runner = SAND;
    let mut bs = -1.0e9f32;
    let mut second = -1.0e9f32;
    let mut total = 0.0;
    for b in 0..N_LAND {
        let v = c.votes[b];
        total += v;
        if v <= 0.001 {
            continue;
        }
        let k = VOTE_K[b];
        let s = v + (n1 * k[0] + n2 * k[1]) * 0.16 + n3 * k[2] * 0.07;
        if s > bs {
            second = bs;
            runner = best;
            bs = s;
            best = b;
        } else if s > second {
            second = s;
            runner = b;
        }
    }
    if total < 0.01 {
        best = GRASS;
        second = -1.0e9;
    }
    let margin = bs - second;
    let edge = smooth(0.0, 0.12, margin); // 0 right at a biome border
    let mut col = land_color(&q, best, edge);
    // Anti-alias the border between two biomes.
    const VAA: f32 = 0.02;
    if margin < VAA && runner != best {
        let other = land_color(&q, runner, edge);
        col = mix(other, col, 0.5 + 0.5 * margin / VAA);
    }

    // Burn scars.
    if c.burnt > 0.01 {
        let b = c.burnt + mid * 0.25 + fine * 0.15;
        if b > 0.5 {
            // Charred ground with grey ash drifts and brown scorched patches,
            // keeping a little of the land underneath.
            let speck = if hash & 0x3f < 5 { 26.0 } else { 0.0 };
            let ash = smooth(0.1, 0.75, ctx.canopy.at1(x0, y0) + fine * 0.4);
            let mut ch = mix(CHAR, SCORCH, smooth(0.62, 0.5, b) + (mid * 0.5 + 0.1).max(0.0) * 0.6);
            ch = mix(ch, ASH, ash * 0.55);
            col = add(mix(ch, col, 0.18), speck + c.shade * 0.3);
        } else if b > 0.35 {
            col = mix(col, SCORCH, smooth(0.35, 0.5, b) * 0.8);
        }
    }

    // Wet sand / mud right at the shore, then the anti-aliased shore line.
    let wet = smooth(-0.22, -0.02, sd);
    if wet > 0.0 {
        let tgt = if best == SAND { WET_SAND } else { MUD };
        col = mix(col, tgt, wet * 0.8);
    }
    let mut alpha = 255.0f32;
    if sd > -AA {
        let (wc, wa) = water_color(&q, sd.max(0.0));
        let t = smooth(-AA, AA, sd);
        col = mix(col, wc, t);
        alpha = 255.0 + (wa - 255.0) * t;
    }

    // Rivers.
    let rd = ctx.river_dist(x + n1 * 0.16, y + n2 * 0.16);
    if rd < 0.75 {
        let hw = 0.36 + mid * 0.05;
        // Muddy, darker bank.
        let bank = 1.0 - smooth(hw, hw + 0.2, rd);
        col = mix(col, mul(MUD, 0.95), bank * 0.6);
        if rd < hw + AA {
            let t = (rd / hw).min(1.0);
            let mut rc = mix(RIVER_C, SHALLOW, smooth(0.5, 1.0, t));
            rc = add(rc, fine * 5.0 + q.grain * 0.5);
            rc = mix(rc, FOAM, smooth(0.8, 1.0, t) * 0.25);
            let k = 1.0 - smooth(hw - AA, hw + AA, rd);
            col = mix(col, rc, k);
            alpha = alpha.min(255.0 - 21.0 * k);
        }
    }

    (add(col, q.grain), alpha.round() as u8)
}

/// RGBA, width `sim.w * scale`, height `sim.h * scale`.
pub fn terrain_rgba(sim: &Sim, scale: i32) -> Vec<u8> {
    let scale = scale.max(1);
    let (pw, ph) = (sim.w * scale, sim.h * scale);
    let mut out = vec![0u8; (pw * ph * 4) as usize];
    if pw == 0 || ph == 0 {
        return out;
    }
    let ctx = build(sim);
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1).clamp(1, 16);
    let rows_per = ((ph as usize + threads * 4 - 1) / (threads * 4)).max(1);
    let chunks: Vec<(usize, &mut [u8])> = out.chunks_mut(rows_per * pw as usize * 4).enumerate().collect();
    let next = std::sync::Mutex::new(chunks.into_iter());
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| loop {
                let job = next.lock().unwrap().next();
                let Some((ci, buf)) = job else { break };
                let y0 = (ci * rows_per) as i32;
                for (r, row) in buf.chunks_mut(pw as usize * 4).enumerate() {
                    let py = y0 + r as i32;
                    for px in 0..pw {
                        let (c, a) = pixel(&ctx, px, py, scale);
                        let i = px as usize * 4;
                        row[i] = c[0].clamp(0.0, 255.0) as u8;
                        row[i + 1] = c[1].clamp(0.0, 255.0) as u8;
                        row[i + 2] = c[2].clamp(0.0, 255.0) as u8;
                        row[i + 3] = a;
                    }
                }
            });
        }
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Also a debugging hook: GENESIS_TERRAIN_SEED picks the world and
    /// GENESIS_TERRAIN_DUMP=<path> writes the raw RGBA (3200 x 2240) there.
    #[test]
    fn terrain_texture_size_and_determinism() {
        let seed: u64 = std::env::var("GENESIS_TERRAIN_SEED").ok().and_then(|s| s.parse().ok()).unwrap_or(7);
        let sim = Sim::new(seed, 200, 140);
        let t0 = std::time::Instant::now();
        let a = terrain_rgba(&sim, 16);
        eprintln!("terrain_rgba(16): {} ms", t0.elapsed().as_millis());
        assert_eq!(a.len(), 200 * 16 * 140 * 16 * 4);
        let b = terrain_rgba(&sim, 16);
        assert!(a == b);
        if let Ok(path) = std::env::var("GENESIS_TERRAIN_DUMP") {
            std::fs::write(path, &a).unwrap();
        }
        let small = terrain_rgba(&sim, 1);
        assert_eq!(small.len(), 200 * 140 * 4);
        // Water is marked in alpha, land is opaque.
        assert!(a.chunks(4).any(|p| p[3] == 255));
        assert!(a.chunks(4).any(|p| p[3] < 255));
    }
}
