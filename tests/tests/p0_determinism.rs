//! P0: the simulation must be a pure function of seed + initial state + actions.

use alife_core::StableHash;
use p0_determinism::{build_world, run, run_seed, TICKS};

#[test]
fn hundred_runs_produce_identical_hashes() {
    let (reference, ref_cps, _) = run_seed(42);
    assert_eq!(ref_cps.len() as u64, TICKS / 1000);
    for i in 0..100 {
        let (h, cps, _) = run_seed(42);
        assert_eq!(h, reference, "run {i} diverged (final)");
        assert_eq!(cps, ref_cps, "run {i} diverged (checkpoints)");
    }
}

#[test]
fn different_seeds_give_different_worlds() {
    let a = run_seed(1).0;
    let b = run_seed(2).0;
    let c = run_seed(3).0;
    assert!(a != b && b != c && a != c);
}

#[test]
fn snapshot_and_resume_equals_uninterrupted() {
    let (reference, _, _) = run_seed(7);
    let mut w = build_world(7);
    run(&mut w, 7_000);
    let mut snap = w.clone();
    drop(w);
    run(&mut snap, TICKS);
    assert_eq!(snap.state_hash(), reference);
}

#[test]
fn parallel_threads_match_sequential() {
    let reference = run_seed(99).0;
    let hashes: Vec<u64> = std::thread::scope(|s| {
        let hs: Vec<_> = (0..8).map(|_| s.spawn(|| run_seed(99).0)).collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    assert!(hashes.iter().all(|&h| h == reference));
}

#[test]
fn random_policy_actually_exercises_physics() {
    // Guard against a vacuous test: bodies must move, touch, eat and get burned.
    let (_, _, w) = run_seed(42);
    assert!(w.contacts.len() > 50, "contacts {}", w.contacts.len());
    assert!(w.contacts.iter().any(|c| c.ingested));
    assert!(w.agents.iter().any(|a| a.body.damage_hand.raw() > 0 || a.body.damage_mouth.raw() > 0));
}
