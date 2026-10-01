//! C6: a full child life (2 game days of world + mind) is bit-identical across
//! 100 runs, and independent of thread scheduling.

use p1_fire_learning::run_learning;

#[test]
fn child_life_is_deterministic_100_runs() {
    let (_, reference, _) = run_learning(7, false);
    for i in 0..100 {
        let (_, log, _) = run_learning(7, false);
        assert_eq!(log.hash, reference.hash, "run {i} diverged");
    }
}

#[test]
fn child_life_is_deterministic_across_threads() {
    let (_, reference, _) = run_learning(11, false);
    let hashes: Vec<u64> = std::thread::scope(|s| {
        let hs: Vec<_> = (0..8).map(|_| s.spawn(|| run_learning(11, false).1.hash)).collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    assert!(hashes.iter().all(|&h| h == reference.hash));
}
