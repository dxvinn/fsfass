# Phase 2 Results (stopped after P1 by decision; P2–P7 not run)

## P0 — Determinism: **PASS**
`experiments/p0_determinism/results/p0_report.md`, tests in `tests/tests/p0_determinism.rs`.
- 100/100 runs produced identical final and checkpoint hashes (12x12 room, 3 bodies, 20,000 ticks).
- Different seeds gave different hashes. Snapshot/resume matched the uninterrupted run. 8 parallel threads matched the sequential run.
- Fixed-point maths has pinned golden bit values.

## P1 — Child and fire: **PARTIAL**
Full report: `experiments/p1_fire_learning/results/p1_report.md`. Raw data: `p1_raw.csv`. Brain traces: `results/traces/`.

**Important:** the 60-seed report was produced by a build from *before* the last two fixes (described under Known issues). Those fixes were checked on 3 seeds only. The full suite was not re-run, by decision.

| Criterion (fixed before running) | Measured | Result |
|---|---|---|
| C1 learning: E(FIRE A) ≥ 0.5 | E = +1.00 (naive 100% contact, experienced 0%) | PASS |
| C2 generalisation: E(FIRE B) ≥ 0.5·E(A) | E = +1.00 | PASS |
| C3 heat concept (HOT METAL) | contact 2% vs naive 100%; 147 of 148 reaches aborted on feeling heat | PASS |
| C4 no blanket fear | E(flower) +0.22 < 1.00; rock 0.72 and food 0.90 contact (≥ 0.7×naive) | PASS (marginal) |
| C5 learned eating faster than naive | 120 s vs 6 s to first swallow | **FAIL** |
| C6 determinism with mind | test written (`tests/tests/p1_mind_determinism.rs`) but the 100-life run was stopped before finishing | **UNVERIFIED** |
| C7 anti-cheat audit | the mind depends only on core + interface; no object-kind words in the source | PASS |

Other measurements:
- 60/60 children were burned once, almost all by mouth (52) rather than by hand (8).
- Mind costs about 22.5 µs per cognitive step and about 119 KB per child.

## Known issues (documented, not fixed by decision)
1. **Fear ratchet (fixed after the 60-seed run, not re-measured).**
   - Second-order (TD) conditioning updated only upwards on noisy prediction differences. Fear crept onto food, water and rock, which explains C5. In the worst case (seed 37) the child starved because it avoided food.
   - The fix restricts TD to increases explained by a new heat sensation and lowers bystander credit.
2. **Over-generalised wariness.** Flower, rock and food contact falls 10–28% after one burn.
3. **Mouth-first exploration.** Most first burns are by mouth. Learning transfers between mouth and hand through a shared "contact" action family.
4. **Approach–avoid dithering.** It is reduced by the proximity gradient and by persistence that decays, but brief loops remain. Some probes show hundreds of reach/abort cycles.
5. **Over-drinking and over-eating in some seeds.** Habit and persistence keep ingestion going after needs are met (mean 688 sips in 2 days).
6. **Learning-progress curiosity was removed** (a per-action global signal misbehaved). It needs per-region learning progress.
7. **Memory cost is above the L3 target** (about 119 KB against 32–64 KB). The associative tables are dense.
8. **Concept flicker.** One fire is sometimes recognised as two prototypes (C4/C5) as its brightness flickers.

## Iteration log (honesty record)
These changes were made during P1 development after observing failures. Each is a modelling correction backed by the research, not a rule about specific objects:
- Fixed point changed from Q16 to Q32: healing and sleep pressure were rounding to no change.
- Outcomes are now intake signals (Keramati–Gutkin), not drops in need.
- Acute-nociception onset is used instead of total-pain onset.
- Need-driven attention and exploration added only for ingestion.
- Curiosity discounted by predicted harm.
- Approach–avoidance proximity gradient.
- Withdrawal only from predicted harm.
- Persistence decays with repetition.
- Rescorla–Wagner cue salience and outcome salience (β).
- Thermometer coding of felt warmth and brightness.
- Contact action family.
- Second-order TD conditioning.
- Two noisy rules removed (an untargeted habit update, and a reflexive withdrawal on tonic pain).
