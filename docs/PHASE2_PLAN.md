# Phase 2 Plan: Prove That Artificial Life Works

Phase 2 builds no game and no graphics. It runs small headless experiments, each answering one question
with a pass/fail criterion fixed **before** the code runs. Results go in `docs/PHASE2_RESULTS.md`.

The research corpus is `docs/research/00–09`. New research is allowed only for a specific unanswered question,
a conflict between reports, an implementation detail, a license check, or a failed prototype.

---

## 1. The prime directive, enforced by the build

> **The world contains truth. The agent contains beliefs. They are never the same data structure.**

This is enforced at compile time, not by convention:

```
simulation/
  core/        fixed-point maths, counter-based RNG, state hashing, event queue        (no deps)
  interface/   SensoryFrame, Percept features, Interoception, MotorCommand            (core)
  biology/     body truth: energy, hydration, skin temperature, tissue damage,
               nociceptors, sleep pressure, healing                                    (core)
  world/       world truth: objects with physical properties, physics, sensors
               (truth -> SensoryFrame), motor execution                                (core, interface, biology)
  mind/        ArtificialMind: perception, attention, memory, learning, decision        (core, interface ONLY)
    src/memory/  src/learning/  src/decision/
experiments/
  p0_determinism/  p1_fire_learning/   (later: p2 … p7)
tests/          workspace-level integration tests (determinism, anti-cheat audit, learning)
benchmarks/     (after P1, on request)
docs/
```

- The `mind` crate **cannot import** `world` or `biology`. It is given a `SensoryFrame`: feature values,
  interoceptive signals and opaque tracking tokens. It returns a `MotorCommand`. No object kinds, no world
  ids, no temperatures in °C and no "edible" flags ever reach it.
- **Anti-cheat audit test:** `tests/` scans the `mind` source tree. It fails the build if the code contains
  object-kind words (fire, flame, food, water, rock, stone, flower, metal, berry, edible…) or world-truth
  property names. The mind may only talk about sensory features (hue, brightness, flicker, felt warmth,
  sweetness…), its own concepts, actions and interoceptive signals.
- Tracking tokens are salted per environment, so identities do not survive a reset. Memory and learning
  structures store **features and learned concepts**, never tokens.

---

## 2. Dependency map: which research each prototype needs

| Prototype | Required findings | Source |
|---|---|---|
| **P0 Determinism** | world = f(seed, build, command log); fixed-point arithmetic; counter-based RNG keyed by (seed, entity, tick, stream); no iteration over hash maps; deterministic event ordering (tick, priority, entity, seq); multi-rate systems; snapshot → resume equality (SyncTest); thread-count independence | 07 E0, E4, E7; 00 §3.2 #2 |
| **P1 Child & fire** | AMG learning rules (Rescorla–Wagner summed prediction, Pearce–Hall associability, eligibility on source node, negativity bias κ−>κ+, two-timescale fast/slow weights + sleep consolidation, lazy forgetting, crystallisation into beliefs) | 01 §8.1–8.2 |
| | Decision loop: perceive → GWT workspace (K = 3–7 by salience) → options from affordances → habit Q_MF + goal-directed lookahead Q_MB + Pavlovian bias + curiosity + persistence → reliability arbitration → stochastic selection | 01 §8.3; 01 §3 (Daw/Lee arbitration, GPR basal-ganglia hysteresis) |
| | Homeostatic reward = drive reduction (Keramati–Gutkin); empathic coupling later | 03 R3; 03 HRRL entry |
| | Curiosity: count novelty limited to action-dependent outcomes (ICM lesson), learning progress (IAC/ALP) | 01 §4 |
| | Episodic store with surprise-based event segmentation, deterministic importance, ACT-R base-level + cue match + emotion boost retrieval; MFEC-style "what happened last time" value | 02 C.(b); 01 §2 (episodic control) |
| | Beliefs as records with confidence, source and evidence pointers, separate from world truth | 02 C.(a); 01 rule 9 |
| | Appraisal → discrete emotions (fear = prospective negative, distress, surprise, relief/joy, interest) → PAD mood; emotions modulate attention width, risk weight and temperature (PSI modulators) | 03 R1; 01 MicroPsi entry |
| | Personality → learning/risk/curiosity parameters; age curves for learning rate, κ, curiosity, K, planning depth | 03 R2; 01 §8.5 |
| | Body truth: energy, hydration, skin temperature, tissue damage, healing, nociception, sleep pressure | 04 C.3 |
| | World truth: material/object properties (temperature, luminance, flicker, hue, size, mass, hardness, nutrition, water content); heat transfer | 05 §3.1–3.3 |
| | Decision trace format and "why" explanation | 03 R5; 07 E6 |
| **P2 Extinction / context** | Extinction as new context-gated learning, w_ext(cue, context), faster decay of w_ext → renewal, spontaneous recovery; avoidance preserves fear | 01 §8.2 rule 4, §8.4; 01 psipeter/amygdala entry |
| **P3 Social learning** | Observational fear learning (α × trust × 0.5); being told (word as cue, capped weight until confirmed); naming-game lexicon; belief provenance; trust; false beliefs | 01 rules 10–11; 02 A.1, C.(a); 01 EGG/Babel2 entry |
| **P4 Memory / rumor** | Belief record layout, evidence weights, Talk of the Town deterioration (forget/mutate/transfer), leveling/sharpening/assimilation/transference, Maki–Thompson stifling, topic choice, cheap ToM tags | 02 B.13, C.(a) |
| **P5 Development** | One brain with age-curve parameters; critical periods; attachment; object permanence τ_obj; ToM level; trust in parent vs peer; A-not-B and false belief must emerge | 01 §8.5, §5 (Drescher, ERA, BToM); 04 C.3 growth |
| **P6 Invention** | Material property schema, interaction formulas (strike, cut, join, heat, transform); affordance beliefs (behavior, tool, target, context) → effect; curiosity over effect novelty; generalisation by intersecting predicates | 05 §3–4 (with change D2 below) |
| **P7 Generations** | Skills (practice vs knowledge, rust, catch-up); transmission fidelity by mode; Henrich population-size loss; demography (birth, death); cultural trait drift; divergence of an isolated group | 05 §5–6; 06 §1; 04 C.4 |

Reports 08 (LLM runtimes) and 09 (animation) are **out of scope for P0–P7**. No LLM is used in Phase 2.

---

## 3. Decisions changed after re-reading the reports

### D1. Where concepts come from
- **OLD (01 §8.1):** a shared, world-wide concept vocabulary. Perception maps each thing to a concept ID
  (FIRE, WOLF, RED_BERRY) with a feature signature.
- **NEW:** the mind receives only **sensory feature channels** produced by sensor models (hue, saturation,
  brightness, flicker, size, shape, texture, gloss, felt radiant warmth, contact temperature, hardness,
  wetness, taste, interoception). The agent **forms its own object concepts** as prototypes over feature
  units (adaptive-resonance style: match if similarity ≥ vigilance, else create a new prototype).
  Associations are learned on **both** elemental feature units and learned concepts (an elemental +
  configural Rescorla–Wagner model).
- **EVIDENCE:** 01 §9 risk 1 ("the ontology is the hidden script… open: how much concept formation from raw
  features do we need… signature clustering… needs prototyping"); 02 A.1 Talking Heads (each agent grows
  its own perceptual categories); the Phase 2 prime directive; the P1 generalisation test (FIRE B, HOT
  METAL, ORANGE FLOWER) would be trivial or meaningless if the world told the brain "this is FIRE".
- **WHY:** a world-assigned FIRE id is world truth leaking into the mind. It would make "generalisation"
  either automatic (same id) or impossible (new id), and it cannot express a flower that partly resembles
  fire.

### D2. Invention predicates (affects P6, recorded now)
- **OLD (05 §4.1):** affordance predicates are bins over true material properties ("H ≥ 6 & conchoidal").
- **NEW:** predicates are over **perceived** features (luster, texture, ring when struck, heft, edge feel)
  and over **observed effects**. True hardness and toughness stay in the world.
- **EVIDENCE / WHY:** same prime directive. 05's own risk 2 ("predicate binning determines what is
  discoverable") is safer when bins come from perception.

### D3. Homeostatic drive exponent
- **OLD (03, Keramati–Gutkin):** D(H) = (Σ|h* − h|^n)^(1/m) with n > m > 1.
- **NEW:** n = 2, m = 1 (sum of squared deficits) for Phase 2.
- **WHY:** it keeps the property that matters (convex, so the worst need dominates and relief is worth
  more when deprived) and avoids fractional powers in fixed-point arithmetic. Revisit if risk-aversion
  tests need the exact form.

### D4. Timing wheel deferred
- **OLD (00 §11 P0 / 07 E4):** hierarchical timing wheel.
- **NEW:** P0 uses a deterministic binary-heap event queue with a total order (tick, priority, entity,
  seq) plus multi-rate system buckets. The timing wheel is a performance optimisation for 100k+ agents,
  so it waits until benchmarking shows it is needed.

### D5. Prototype list
The synthesis's P2–P7 (gossip, ecology, invention, LOD, religion, LLM harness) are **superseded** by the
Phase 2 brief's P2–P7 (extinction, social learning, rumor, development, invention, generations).
Ecology, LOD, religion and LLM prototypes move to Phase 3.

### D6. Level-of-detail naming
Reports 01 and 02 number tiers T0/LOD0 = most detailed. 07 and this project use **L4 = observed …
L0 = macro**. All Phase 2 prototypes run at L4/L3 (full cognition); LOD degradation is out of scope until
cognition works.

No other decisions in the synthesis are changed.

---

## 4. Engineering rules for every prototype

1. **Determinism.** All simulation state uses `Fx` (64-bit fixed point, 16 fractional bits). exp, ln,
   sigmoid and sqrt are implemented in integer arithmetic. Randomness comes only from
   `rng(seed, entity, tick, stream)`. No `HashMap` iteration. State hashes use our own FNV-style hasher
   over a canonical field order.
2. **No external crates** in the simulation crates for Phase 2: Rust `std` only. So no third-party
   license questions arise. Any future import must have its license verified at the primary repository
   and recorded in `docs/THIRD_PARTY.md` before use.
3. **No LLM** anywhere in P0–P7.
4. **No faked emergence.** Forbidden patterns include `if object.kind == FIRE`, `if burned { avoid = true }`,
   `if hungry { eat(food) }` and `if parent_says_hot { fire_dangerous = true }`. The world defines physics
   (temperature, nutrition, damage, distance) and the body defines innate biology (nociceptors fire when
   skin is too hot, a stomach fill signal, reflex withdrawal from a painful contact, drive reduction is
   rewarding). Everything about *which things* cause *which outcomes* must be learned.
5. **Honest reporting.** Pass criteria are fixed in this document before results exist. Failures are
   reported as FAIL or PARTIAL with the reason.

---

## 5. P0: determinism

**Components:** `core` (Fx, RNG, hasher, event queue, multi-rate scheduler), `biology` (body truth),
`world` (objects, heat physics, sensors, motor execution) and a random-action driver standing in for a mind.

**Tests (automated, `cargo test`):**
1. Same seed + initial state + action script, 100 runs → **100/100 identical state hashes** (final and at
   checkpoints).
2. Different seeds → different hashes (the test is not vacuous).
3. Snapshot at tick N, restore and continue → identical to the uninterrupted run (SyncTest).
4. Runs on 1 thread vs 8 parallel threads → identical hashes.
5. Event-queue ordering is a total order: same-tick events with different insertion orders resolve
   identically.
6. Fixed-point maths functions are bit-stable (golden values).

**PASS:** all 6 tests pass. **FAIL:** any divergence.
After P1, the same determinism suite is re-run **with the mind attached** (100 runs of a full P1 child
simulation, including brain-state hashes).

---

## 6. P1: child and fire

### 6.1 World (truth)
A small room grid (1 cell ≈ 1 m, tick = 1 game second). Objects and their physical properties are known
only to the world:

| Object | Temp °C | Emitted light | Flicker | Hue | Size | Nutrition | Water | Notes |
|---|---|---|---|---|---|---|---|---|
| FIRE A | 700 | high | high | orange | medium | — | — | radiant heat |
| FIRE B (probe) | 520 | medium | high | red-orange | small | — | — | different size, colour, intensity |
| HOT METAL (probe) | 380 | none (below visible glow) | none | grey | small | — | — | felt only by radiant heat at close range |
| ORANGE FLOWER (probe) | 20 | none (reflects) | none | orange | small | — | — | harmless |
| ROCK | 20 | none | none | grey | small | — | — | hard, inedible |
| FOOD (berries) | 20 | none | none | red | small | yes | some | consumable bites |
| WATER (pool) | 15 | none | low shimmer | blue | medium | — | yes | glossy, wet |

Physics: radiant warmth falls with distance; contact heats skin toward object temperature; tissue damage
above about 44 °C skin temperature; ingestion moves mass into the stomach (energy and water absorbed over
time); the body heals over days.

### 6.2 Body (innate biology, not knowledge)
- Interoception: hunger, thirst, pain, fatigue.
- Thermal nociceptors fire when skin temperature exceeds about 43 °C. A **withdrawal reflex** ends the
  contact.
- A stomach fill signal gives fast satiation after ingesting nutritive matter, and oral or gastric water
  sensing gives fast thirst relief. These are generic biological signals: the body does not know what
  "food" is, only that something nutritive entered the stomach.
- Innate drives: avoid pain (phasic pain onset is aversive), homeostasis (drive reduction is rewarding),
  explore novelty (curiosity bonus). None mention any object.

### 6.3 Mind v0 (implemented in `mind`, every element below appears in the brain trace)
- **Sensory encoding:** innate receptive fields turning features into about 60 graded feature units
  (hue bins, brightness, flicker, size, felt warmth levels, contact heat, hardness, wetness, sweetness…).
- **Perception:** object concepts learned as prototypes (vigilance matching); context prototype learned
  from ambient features.
- **Attention bottleneck:** a workspace of K = 3–7 (age and arousal dependent). Candidates are percepts,
  drives and recalled memories, ranked by salience = intensity + novelty + |predicted value| + drive
  relevance + persistence. Only attended items get full learning and generate options.
- **Working memory:** workspace slots that persist across ticks, the current goal and target, and the last
  action and outcome.
- **Associative memory:** Pavlovian weights (unit → outcome) and instrumental weights (action | unit →
  outcome), each fast + slow; extinction weights per (unit, context); Pearce–Hall associability per unit;
  evidence counts.
- **Episodic memory:** a budgeted store (96 episodes + 16 pinned). An episode is created on surprise
  (|δ| > θ) or high importance and holds features, concept, context, action, outcome, δ, valence and
  arousal. Retrieval uses ACT-R base level + cue similarity + emotional boost. Recalled episodes give an
  episodic-control value estimate.
- **Semantic memory / beliefs:** crystallised from stable, strong associations ("units {…} + action →
  outcome", confidence, evidence count, source = own experience, evidence episode ids, t_learned). A
  belief is the agent's statement about the world, and it can be wrong.
- **Emotion:** appraisal of predicted and actual outcomes against needs gives fear, distress, surprise,
  relief, joy and interest. A PAD mood follows. Arousal narrows K, fear multiplies predicted losses, and
  arousal lowers decision temperature.
- **Needs:** homeostatic drive (D3); hunger and thirst urgency; pain.
- **Personality:** neuroticism → κ− (aversive learning gain) and fear gain; openness → curiosity;
  conscientiousness → persistence; extraversion/sensation-seeking → risk tolerance. Values are seeded per
  child.
- **Goals:** emerge from drives × predicted opportunities (relieve hunger via something predicted to
  satiate; explore the most interesting thing; avoid predicted threat).
- **Three decision systems:**
  1. **Reflex / gut:** innate withdrawal on nociception, plus Pavlovian approach/avoid bias from learned
     cue value (no lookahead).
  2. **Habit:** model-free Q(concept, action) learned by TD from homeostatic reward, with a reliability
     estimate.
  3. **Deliberate:** depth-2 lookahead over the learned action→outcome model (approach → touch / mouth)
     with need-dependent utilities, an episodic-control estimate and an epistemic (inspect) value.
  Arbitration weights habit vs deliberate by reliability and fatigue. The gut system adds bias, and
  reflexes override.
- **Learning:** surprise-gated Rescorla–Wagner/TD with eligibility, negativity bias, Pearce–Hall
  attention and age-scaled rates.
- **Consolidation (sleep):** replay of top episodes; fast → slow transfer with tagging; flashbulb flag;
  fast weights relax.
- **Forgetting:** slow weights decay toward the prior lazily (computed when read), slower for rehearsed or
  flashbulb associations.

### 6.4 Protocol
For each of **N = 60 seeds**:
1. **Learning phase:** a naive child (about 4 years old by the age curves) lives 2 game days in a room
   with FIRE A, ROCK, FOOD and WATER. Hunger and thirst build up, and the child sleeps at night. We log
   whether and when it touched fire, mouthed food, drank and mouthed rock.
2. **Reset:** a new room, new object instances at new positions and a new token salt. **The brain is
   preserved** (cloned).
3. **Probe phase:** for each probe object (FIRE A, FIRE B, HOT METAL, ORANGE FLOWER, ROCK, FOOD), a fresh
   clone of the post-learning brain spends 10 game minutes alone with that object, with moderate hunger.
4. **Control group:** the same probes are run on a naive brain with the same seed and personality.
5. **Measures per probe:** touched or mouthed (yes/no), latency to first contact, closest distance, time
   within 1 cell, number of inspect-from-distance actions, and predicted pain plus fear at first sight
   (read from the brain).
6. **Traces:** a full brain trace for one representative child at (a) first fire contact, (b) first sight
   of FIRE A after the reset, (c) first sight of FIRE B, (d) HOT METAL approach, (e) ORANGE FLOWER.

### 6.5 Pass criteria (fixed now)
Let **avoidance effect** E(probe) = contact rate (naive) − contact rate (experienced). Statistics are taken
over the children that were actually burned during learning ("burned group").

| Criterion | PASS if |
|---|---|
| C1 Learning happens | E(FIRE A) ≥ 0.5 in the burned group |
| C2 Not a lookup of one object | E(FIRE B) ≥ 0.5 × E(FIRE A) |
| C3 Heat concept, not just looks | HOT METAL: experienced children have a lower contact rate **or** shorter contact duration than naive ones, by a meaningful margin (≥ 0.2 rate difference or ≥ 30% less contact time) |
| C4 No omniscience / no blanket fear | E(ORANGE FLOWER) < E(FIRE A), and ROCK/FOOD contact rates of experienced children ≥ 0.7 × naive |
| C5 Other learning | Experienced, hungry children mouth FOOD with shorter latency than naive children |
| C6 Determinism with mind | 100/100 identical hashes for a full P1 run |
| C7 Anti-cheat audit | the mind source contains no object-kind words, and the mind crate does not depend on world/biology |

P1 = **PASS** if C1–C7 all pass. **PARTIAL** if C1, C6 and C7 pass but any of C2–C5 fails. **FAIL** if
C1, C6 or C7 fails. The burn rate itself (how many children touch fire) is reported, not judged. A child
that never touches fire is a legitimate outcome.

---

## 7. P2–P7 (not started; await review of P1)

| # | Question | Main pass criterion (draft, finalised before each starts) |
|---|---|---|
| P2 | Is extinction contextual? | Fear at a safe hearth falls by ≥ 50% over repeated safe exposures, while fear in a new context (forest) stays ≥ 50% of the original (renewal) |
| P3 | Can knowledge travel without memory access? | B avoids fire after A's warning signals at a rate above naive; B acquires a false "blue rock" belief from a confident C; beliefs carry the source |
| P4 | Does information spread without teleporting? | Knowledge only via witness/tell chains (audit = 0 violations); distortion measurable; two isolated groups diverge in version |
| P5 | Does development emerge? | Newborn starts with empty lexicon/concepts; parent recognition, imitation and words appear with age; no knowledge tables unlocked by age |
| P6 | Can relations be discovered without recipes? | A useful composite or property relation is discovered and reused in ≥ X% of seeds; audit shows no recipe tables; honest FAIL if not |
| P7 | Does knowledge persist and drift across generations? | Gen-3 retains a Gen-1 discovery at a measured rate; an isolated subgroup diverges |

Benchmarks (1 / 10 / 100 / 1,000 / 10,000 agents; updates/s, CPU, memory per agent, event queue size)
come once individual cognition works. The brain visualizer (panels plus a live association graph) comes
after P1 is accepted.
