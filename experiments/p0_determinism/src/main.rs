//! P0 report: runs the determinism checks and writes results/p0_report.md.

use alife_core::StableHash;
use p0_determinism::{build_world, run, run_seed, TICKS};
use std::fmt::Write as _;
use std::time::Instant;

fn main() {
    let seed = 42;
    let t0 = Instant::now();
    let (reference, ref_cps, world) = run_seed(seed);
    let single = t0.elapsed();

    // 1. 100 identical runs.
    let mut identical = 0;
    for _ in 0..100 {
        let (h, cps, _) = run_seed(seed);
        if h == reference && cps == ref_cps {
            identical += 1;
        }
    }

    // 2. Different seeds differ.
    let others: Vec<u64> = (1..=10).map(|s| run_seed(s).0).collect();
    let distinct = others.iter().all(|&h| h != reference);

    // 3. Snapshot + resume.
    let mut a = build_world(seed);
    run(&mut a, 7_000);
    let mut snapshot = a.clone();
    run(&mut a, TICKS);
    run(&mut snapshot, TICKS);
    let resume_ok = a.state_hash() == snapshot.state_hash() && a.state_hash() == reference;

    // 4. Parallel threads.
    let par: Vec<u64> = std::thread::scope(|s| {
        let hs: Vec<_> = (0..8).map(|_| s.spawn(move || run_seed(seed).0)).collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let threads_ok = par.iter().all(|&h| h == reference);

    let contacts = world.contacts.len();
    let burns = world.contacts.iter().filter(|c| c.label == "FIRE_A").count();
    let mut out = String::new();
    writeln!(out, "# P0 determinism report\n").unwrap();
    writeln!(out, "- Scenario: 12x12 room, 6 objects, 3 bodies, random policy, {TICKS} ticks (1 tick = 1 game second)").unwrap();
    writeln!(out, "- Reference hash (seed {seed}): `{reference:016x}`").unwrap();
    writeln!(out, "- Physical contacts in reference run: {contacts} ({burns} with FIRE_A)").unwrap();
    writeln!(out, "- Wall time per run: {:.1} ms\n", single.as_secs_f64() * 1000.0).unwrap();
    writeln!(out, "| Check | Result |").unwrap();
    writeln!(out, "|---|---|").unwrap();
    writeln!(out, "| 100 runs, same seed: identical final + {} checkpoint hashes | {}/100 |", ref_cps.len(), identical).unwrap();
    writeln!(out, "| 10 other seeds all differ from reference | {} |", distinct).unwrap();
    writeln!(out, "| Snapshot at tick 7000, resume, equals uninterrupted run | {} |", resume_ok).unwrap();
    writeln!(out, "| 8 parallel threads equal sequential | {} |", threads_ok).unwrap();
    let pass = identical == 100 && distinct && resume_ok && threads_ok;
    writeln!(out, "\n**P0: {}**", if pass { "PASS" } else { "FAIL" }).unwrap();
    print!("{out}");
    std::fs::create_dir_all("experiments/p0_determinism/results").ok();
    std::fs::write("experiments/p0_determinism/results/p0_report.md", out).ok();
    if !pass {
        std::process::exit(1);
    }
}
