# ARTIFICIAL MIND V0.1

Code: `simulation/mind` (crate `alife-mind`). It depends only on `alife-core` and `alife-interface`. Minds receive `SensoryFrame`s and return `MotorCommand`s, and never see world truth.

Components: innate sensory encoding (36 feature units: hue, brightness and warmth thermometer codes, flicker, size, shape, surface, seen motion, face pattern) → learned concepts (prototype/ART) → attention workspace (K = 2–7) → working memory → associative memory (Pavlovian and instrumental, fast and slow weights, context-gated extinction, Pearce–Hall associability, cue and outcome salience, TD chaining) → episodic memory (96 + 16 pinned, ACT-R retrieval, episodic-control estimates) → semantic beliefs (crystallised, with confidence, source and evidence) → emotions and PAD mood → homeostatic drives → personality and age parameters → three decision systems (reflex/gut, habit, deliberate) with reliability arbitration → sleep consolidation and forgetting.

Status and known issues of V0: see `docs/PHASE2_RESULTS.md`.

## V0.1 changes (made while fixing the god game's known issues)

Each change was driven by a measured failure in the game. None names a kind of thing; the anti-cheat test still passes.

| Change | Failure it fixed |
|---|---|
| **New sense units.** Two innate units: *moving* (seen self-propelled motion) and *face* (face-like pattern). Faces weigh 2.5× in category matching; motion weighs 0.3×. | People shared early concepts with rocks and bushes, so humans tasted each other. |
| **Being touched is a percept property (`touched_me`).** Comfort is a skin sensation and is localised like pain: my own act earns comfort only if it was my hand on that person. Otherwise comfort is credited to whoever touched me, and a bite to whoever bit me. | Comfort from being touched was credited to whatever the person happened to be doing ("touch rock → comfort", "mouth person → comfort"). |
| **The contact family no longer generalises comfort** from hand to mouth. | Hand comfort spread to mouthing people. |
| **Outcome-specific transfer** in the gut system: <br>• **Touch** is drawn by non-ingestive value (comfort, warmth, harm). <br>• **Mouth** is drawn by what mouthing *that thing* has yielded. <br>• **Approach** is drawn by 0.8 × the best of those. | Pavlovian "people predict meals" made starving adults bite each other. Separately, approach's pull exceeded drinking's, so people "approached water" for hours without drinking. |
| **Habituation to a known category.** Novelty uses the concept's own evidence when the thing is clearly recognised. | A passing new feature (walking) made familiar people look novel again. |
| **Persistence scales with the act's worth.** | Pointless acts chosen by noise were clung to. |
| **Rest is the default.** Rest gets a bonus equal to the expected maximum of the decision noise across the option set, fading with hunger and thirst. | With ~30 noisy options, the best random draw beat resting almost always. |
| **Learned affordances.** After 30+ tries that reliably yielded nothing, touching or mouthing that kind of thing is no longer considered. | — |
| **Need-driven search.** When hungry or thirsty with nothing in view confidently predicted to satisfy the need, wandering gains an innate restlessness (gut), and wandering keeps its heading. | Bands starved beside stripped bushes instead of moving on. |
| **Learning by watching.** `SensoryFrame.observations`: another person seen eating, drinking or recoiling from a thing teaches about that thing vicariously, at 0.3× the rate of one's own experience. | Weaned children died of thirst; knowledge did not spread by example. |
| **Bounded associative strength** (≤ 1.5). | Acquisition in one context and extinction in another ratcheted stored weights to 16+. |
| **Observer output** (`last_recognised`), used only by the game's inspector to name private concepts. | — |

Effect on the Phase 2 fire experiment (60 seeds, rerun after the changes):

| Criterion | V0 | V0.1 |
|---|---|---|
| C1 | PASS | PASS |
| C2 | PASS | PASS |
| C3 (hot metal) | E 0.98 | E 1.00 |
| C4 (no blanket fear) | flower E 0.22, rock contact 0.72, food contact 0.90 | flower E 0.05, rock contact 1.00, food contact 1.00 |
| C5 (time to first swallow, experienced vs naive) | 120 s vs 6 s | 6 s vs 6 s |

C5 is still formally **FAIL**, because experienced children are not strictly faster than naive ones. Full table: `experiments/p1_fire_learning/results/p1_report.md`.
