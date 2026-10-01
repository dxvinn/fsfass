//! Genetics (report 04 §C.2, reduced): a diploid genome of 2 x 128 biallelic
//! loci (32 bytes). Traits are additive over fixed locus sets; children get a
//! recombined gamete from each parent plus rare mutations.

use alife_core::{fx, Fx, Rng};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Genome {
    pub a: [u64; 2],
    pub b: [u64; 2],
}

/// Inherited, visible traits (computed from the genome once at birth).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Traits {
    pub height: Fx,      // relative 0.8..1.2
    pub build: Fx,       // 0..1 slender..stocky
    pub skin: Fx,        // 0..1 light..dark
    pub hair: Fx,        // 0..1 blond..black
    pub eye: Fx,         // 0..1 blue..brown
    pub metabolism: Fx,  // 0.85..1.15
    pub vitality: Fx,    // 0..1 (longevity, disease resistance)
    pub fertility: Fx,   // 0..1
    pub openness: Fx,
    pub conscientiousness: Fx,
    pub extraversion: Fx,
    pub agreeableness: Fx,
    pub neuroticism: Fx,
    pub dexterity: Fx,   // 0..1 (skill aptitude)
}

impl Genome {
    pub fn random(r: &mut Rng) -> Genome {
        Genome { a: [r.next_u64(), r.next_u64()], b: [r.next_u64(), r.next_u64()] }
    }

    fn allele(&self, locus: usize) -> u32 {
        let (w, bit) = (locus / 64, locus % 64);
        (((self.a[w] >> bit) & 1) + ((self.b[w] >> bit) & 1)) as u32
    }

    /// Mean allele dosage over a locus range, 0..1.
    fn score(&self, start: usize, n: usize) -> Fx {
        let s: u32 = (start..start + n).map(|l| self.allele(l % 128)).sum();
        Fx::ratio(s as i64, (2 * n) as i64)
    }

    fn gamete(&self, r: &mut Rng) -> [u64; 2] {
        // One crossover per 64-locus chromosome arm.
        let mut g = [0u64; 2];
        for w in 0..2 {
            let cut = r.below(64);
            let mask = if cut == 0 { 0 } else { u64::MAX >> (64 - cut) };
            let (p, q) = if r.below(2) == 0 { (self.a[w], self.b[w]) } else { (self.b[w], self.a[w]) };
            g[w] = (p & mask) | (q & !mask);
            // Mutation: ~2e-4 per locus.
            for _ in 0..64 {
                if r.below(5000) == 0 {
                    g[w] ^= 1 << r.below(64);
                }
            }
        }
        g
    }

    pub fn child(mother: &Genome, father: &Genome, r: &mut Rng) -> Genome {
        Genome { a: mother.gamete(r), b: father.gamete(r) }
    }

    pub fn traits(&self, r: &mut Rng) -> Traits {
        let env = |r: &mut Rng, s: f64| r.gaussish() * fx(s);
        let t = |g: &Genome, s: usize, n: usize| g.score(s, n);
        Traits {
            height: (fx(0.8) + t(self, 0, 12) * fx(0.4) + env(r, 0.02)).clamp(fx(0.75), fx(1.25)),
            build: (t(self, 10, 10) + env(r, 0.05)).clamp01(),
            skin: t(self, 24, 10),
            hair: t(self, 34, 8),
            eye: t(self, 42, 6),
            metabolism: fx(0.85) + t(self, 48, 10) * fx(0.3),
            vitality: (t(self, 58, 12) + env(r, 0.05)).clamp01(),
            fertility: (t(self, 70, 8) + env(r, 0.05)).clamp01(),
            openness: (t(self, 78, 8) + env(r, 0.1)).clamp(fx(0.05), fx(0.95)),
            conscientiousness: (t(self, 86, 8) + env(r, 0.1)).clamp(fx(0.05), fx(0.95)),
            extraversion: (t(self, 94, 8) + env(r, 0.1)).clamp(fx(0.05), fx(0.95)),
            agreeableness: (t(self, 102, 8) + env(r, 0.1)).clamp(fx(0.05), fx(0.95)),
            neuroticism: (t(self, 110, 8) + env(r, 0.1)).clamp(fx(0.05), fx(0.95)),
            dexterity: (t(self, 118, 10) + env(r, 0.05)).clamp01(),
        }
    }
}
