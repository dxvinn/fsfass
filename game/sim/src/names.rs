//! Procedural personal names (deterministic per seed and person).

use alife_core::rng::key4;

const ONSET: [&str; 18] = ["k", "t", "m", "n", "r", "s", "l", "v", "d", "b", "h", "y", "z", "th", "sh", "g", "p", "f"];
const VOWEL: [&str; 8] = ["a", "e", "i", "o", "u", "ai", "ea", "o"];
const CODA_F: [&str; 6] = ["a", "i", "e", "ah", "ia", "el"];
const CODA_M: [&str; 6] = ["n", "k", "r", "o", "an", "us"];

pub fn name(seed: u64, who: u64, female: bool) -> String {
    let mut h = key4(seed, who, 0x4E41, 0x4D45);
    let mut next = |n: usize| {
        let v = (h % n as u64) as usize;
        h = h.rotate_left(13) ^ 0x9E37_79B9_7F4A_7C15;
        v
    };
    let syll = 1 + next(2);
    let mut s = String::new();
    for _ in 0..syll {
        s.push_str(ONSET[next(ONSET.len())]);
        s.push_str(VOWEL[next(VOWEL.len())]);
    }
    s.push_str(if female { CODA_F[next(CODA_F.len())] } else { CODA_M[next(CODA_M.len())] });
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => s,
    }
}
