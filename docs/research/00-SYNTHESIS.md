# R&D Synthesis — Artificial World / God Simulator

Status: **research phase, no game code.** This document ties together the nine track reports in this
folder into one architecture recommendation, a license policy, and a prototype plan. Every claim here is
backed by a track report; follow the cross-references (e.g. `→ 01 §8.4`) for sources, repo facts and
formulas.

| # | Report | Lines | Candidates |
|---|---|---|---|
| 01 | [Artificial brains, neural pathways, development](01-brains-neural-development.md) | ~1150 | ~33 |
| 02 | [Language emergence, local knowledge, memory](02-language-knowledge-memory.md) | ~1375 | 33 |
| 03 | [Emotion, personality, motivation, decision, LLM hybrids](03-emotion-personality-motivation-decision-llm.md) | ~1230 | 37 |
| 04 | [Body, genetics, reproduction, evolution, animals, ecology, plants, disease](04-biology-genetics-ecology-disease.md) | ~1080 | 43 |
| 05 | [Worldgen, climate, physics, crafting, skills, technology](05-world-physics-crafting-technology.md) | ~1270 | ~50 |
| 06 | [Culture, religion, relationships, groups, politics, economy, crime, war, history](06-culture-religion-society-economy-history.md) | ~1000 | 50 |
| 07 | [Scale, LOD, engine, god interface, existing OSS worlds](07-scale-lod-engine-god-interface.md) | ~950 | ~45 |
| 08 | [In-game AI models and runtimes (LLM, embeddings, TTS, images)](08-in-game-ai-models-runtimes.md) | ~600 | — |
| 09 | [Animation](09-animation.md) | ~700 | ~45 |

**Verification caveat.** The research sandbox blocked the GitHub API, arXiv, Hugging Face and several
academic hosts. Repo facts were checked by fetching GitHub web pages and raw LICENSE files; star counts are
approximate, many last-commit dates are marked *unverified*, and some formulas are paraphrased from papers
that could not be re-opened (marked in each report). Before any code or weights are actually adopted, the
license must be re-checked at the source (see §8).

---

## 1. The single most important conclusion

Nothing on GitHub can be dropped in as "the brain" of 10k–1M creatures. Spiking simulators (Brian2, NEST,
GeNN, Nengo), cognitive architectures (ACT-R, Soar, LIDA, OpenCog) and LLM agents (generative agents,
Concordia, Project Sid) are all orders of magnitude too expensive per agent, and many are GPL/AGPL.

What the research *does* give us is a set of **cheap, well-validated mechanisms** from psychology,
neuroscience, artificial life and game AI that compose into minds that genuinely learn. The plan is to
re-implement those mechanisms (from papers, in our own code) inside one deterministic simulation engine, and
to use LLMs only as a narrator/conversation layer for the handful of agents the god is watching.

The closest historical precedent is **Creatures (1996)**: reward/punishment-modulated plasticity with decaying
traces, instincts as pre-trained experience. Its lesson is that dense neural brains become unreadable; ours
must be **inspectable by design** (→ 01).

---

## 2. Is the proposed linear cognition pipeline correct?

The brief proposed:
`PERCEPTION → ATTENTION → WORKING MEMORY → BELIEFS → EMOTIONS → NEEDS → PERSONALITY → MEMORY RECALL → GOALS →
PLANNING → DECISION → ACTION → CONSEQUENCES → LEARNING → MEMORY UPDATE`

All the ingredients are right; the **shape** is wrong in five ways the research is consistent about:

1. **It is a loop of predictions, not a pipeline.** Perception is driven by *prediction error*: the brain
   predicts what it will sense and learns mostly from surprise (predictive processing; Rescorla–Wagner; TD /
   dopamine RPE). Learning happens *only when something surprising happens*, which is also what makes it cheap
   (→ 01 §8.2, 02 memory design).
2. **Emotion, needs and personality are not stages — they are modulators of every stage.** Personality sets
   parameters (learning rates, risk, attention); needs set the current value of outcomes; emotions arise from
   appraisal of events against goals and beliefs, and then change attention width, planning depth, risk
   perception and commitment (Dörner's PSI modulators, OCC/EMA appraisal, ALMA layering) (→ 03 R1).
3. **Attention is a bottleneck with competition** (global workspace): a few (3–7) items win the workspace by
   salience; only they get full learning and enter planning. Arousal narrows it (→ 01 §8.3).
4. **There are two (or three) decision systems running in parallel**, arbitrated by reliability: a fast
   Pavlovian system (approach/avoid by learned value), a habit system (cached values), and a slow goal-directed
   planner. Children and stressed or tired adults lean on the fast systems; novel, high-stakes situations
   recruit planning (Daw/Niv/Dayan arbitration) (→ 01 §8.3, 03 decision stack).
5. **Memory is updated twice:** immediately (fast weights, episodic encoding on surprise) and offline during
   sleep (consolidation, replay, episodic → semantic abstraction, forgetting). Recall itself rewrites memories
   (reconsolidation → confident-but-wrong witnesses) (→ 01 §8.2 rule 7, 02 Part C).

### Revised architecture (per agent, per decision cycle)

```
                ┌───────────────────────── modulators ─────────────────────────┐
                │ temperament/personality (years) · mood (hours) · emotions     │
                │ (minutes) · drives/needs (homeostatic) · fatigue · age curve  │
                └──────┬───────────────┬──────────────┬───────────────┬─────────┘
                       ▼               ▼              ▼               ▼
 world ─▶ PERCEIVE (predict vs. sense; prediction error) ─▶ WORKSPACE (K slots, salience competition)
                                                              │      ▲ cued recall (episodic/semantic,
                                                              │      │ beliefs w/ confidence+source)
                                                              ▼      │
                         APPRAISE (relevance → goal congruence → blame → coping → norms) ─▶ emotions
                                                              │
                         OPTIONS from affordances + active goals + habits
                                                              │
     ┌──── Pavlovian bias (learned value of cues) ────┐       │
     ├──── Habit values (cached, cheap) ──────────────┤──▶ ARBITRATE by reliability, fatigue, stakes
     ├──── Goal-directed lookahead (depth 0–4) ───────┤       │   + curiosity (learning progress)
     └──── Social terms (others' needs, norms) ───────┘       │   + commitment / persistence
                                                              ▼
                                                  ACT ─▶ outcome ─▶ LEARN on surprise
                                                  (associations, habits, beliefs, relationships,
                                                   skills, lexicon) ─▶ store episode if salient
                                                              │
                                    SLEEP: replay · consolidate · abstract · forget
                                    (rarely) LLM reflection for observed agents only
```

Every arrow above produces fields in a **decision trace** so the god can see *why* (→ §6).

---

## 3. Unified architecture

### 3.1 Layers (all deterministic; LLM is outside the truth boundary)

| Layer | Contents | Primary report |
|---|---|---|
| **World substrate** | Sphere/grid terrain from plate tectonics + erosion; soils & deposits by geology; climate fields (monthly energy balance, winds, moisture); weather sampled on top; material/property object model | 05 |
| **Ecology** | Plant functional types on a grid with light competition, nitrogen cycle, seed dispersal; animal populations (individuals near the camera, populations elsewhere); disease fields | 04 |
| **Organism** | Diploid genome (128 bi-allelic loci, 32 B), daily body model (~10 slow variables: energy balance, growth curve, sleep pressure, injury, immunity, fertility, ageing hazard) | 04 |
| **Mind** | Associative Mind Graph (AMG) + affect layers + motivation + decision stack; species-scaled tiers (insect → human) | 01, 03, 04 C.5 |
| **Knowledge** | Per-agent beliefs with provenance/confidence; bounded episodic/semantic memory; per-culture lexicons and phonology | 02 |
| **Society** | Directed relationship vectors; culture traits; emergent groups, offices, norms, religion, markets, justice, war | 06 |
| **Chronicle** | Event log with *true* layer + in-world (biased) accounts; significance scoring | 06 §8, 07 |
| **God interface** | Time control, inspector, interventions, observed-facts store | 07 |
| **Presentation** | Renderer, animation, LLM narration/dialogue, TTS, generated portraits | 08, 09 |

### 3.2 Cross-cutting principles (agreed across all reports)

1. **One canonical identity per being for life;** higher LOD tiers *attach* state, never replace it (→ 07 E0).
2. **Determinism:** world = f(seed, build, god-command log, external-input log). Fixed-point maths,
   counter-based RNG keyed by (seed, entity, tick, stream). LLM outputs are logged external inputs (→ 07, 08).
3. **Observed facts are canon.** Anything the god has seen is never contradicted by later lazy generation
   ("alibi generation", Sunshine-Hill & Badler) (→ 07 E3).
4. **Learn from surprise only;** compute decay lazily at read time (FSRS/ACT-R style), never per tick (→ 01, 02).
5. **Priors + deltas:** each creature stores only how it differs from its species/culture defaults
   (copy-on-write). This is both the biggest memory saving and *how culture enters the brain* (→ 01 §8.1).
6. **Perceptual ontology only.** The shared concept vocabulary may contain FIRE, WOLF, RED_BERRY, PAIN — never
   DANGER or ENEMY. Value must be learned, or the vocabulary becomes a hidden script (→ 01 §9 risk 1).
7. **One transmission engine** for everything social: beliefs, rumours, skills/techniques, words, norms,
   rituals, prices. Each item has fidelity, content bias, prestige/conformity bias, and a source chain
   (→ 02, 05 §6, 06 §1).
8. **No authored progressions:** no tech tree (property-affordance invention, → 05 §4), no tribe→state ladder
   (groups detected from the social graph, labels computed afterwards and can regress, → 06 §4), no scripted
   gods (agency attribution to salient unexplained events, → 06 §2), no unconditional money (→ 06 §5).
9. **LOD per subsystem, budget-driven:** an importance score and a CPU budget (dependent on game speed) assign
   tiers; distance alone is not the criterion (→ 07 E0/E2).
10. **The LLM proposes; the engine disposes.** LLM output is grammar-constrained to a schema built from the live
    world (only real ids/actions), validated, applied at a fixed tick, logged for replay. The game must be fully
    playable with no LLM (→ 03, 08).

### 3.3 How the mind pieces fit together

Reports 01 and 03 approached cognition from different ends; they compose as follows:

- **AMG (01)** is the learning substrate: concept nodes, associative edges with fast/slow weights, extinction
  memories by context, habit cache, workspace, lexicon, social models. It supplies *learned value* of cues,
  habits, and a world model for lookahead.
- **Affect (03 R1)** sits on top: temperament (Big Five + HEXACO-H) → baseline mood (PAD) → mood → discrete
  emotions from appraisal. ~18 emotion rules including grief, loneliness, envy, jealousy, shame, guilt.
  Contagion touches mood only.
- **Motivation (03):** homeostatic drives with *empathic coupling* (a loved one's deficit enters your own drive),
  plus SDT-style psychological needs (autonomy, competence, relatedness), status, curiosity (learning progress
  from 01), meaning. Goals emerge when drive × opportunity × personality × recalled memory crosses a threshold,
  with commitment hysteresis.
- **Decision (01 §8.3 + 03):** utility scoring over options generated from affordances; option value =
  arbitration(habit, goal-directed lookahead) + Pavlovian bias + curiosity + social/norm terms + persistence.
  HTN/GOAP turns the chosen goal into steps; a behaviour tree executes; MCTS/POMCP only for rare high-stakes
  choices at L4.
- **Memory (02):** episodic slots with emotional tagging and surprise-based event segmentation; retrieval =
  ACT-R base-level + spreading activation + FSRS stability + emotion boost; nightly consolidation into semantic
  facts. 01's per-node eligibility trace and fast→slow transfer are the *associative* side of the same
  consolidation pass.
- **Beliefs (02):** log-odds confidence, stability, source kind, immediate source, hop count, hidden origin
  event; distortion operators on retelling. AMG "crystallised" edges (01 rule 9) are the bridge: once an
  association is strong and stable it becomes a *sayable belief* that the transmission engine can carry.

### 3.4 Species cognition tiers (→ 04 C.5, 01 §8)

| Tier | Example | Mind | Approx. cost/decision |
|---|---|---|---|
| T0 | ant, bee | gradient-following rules or tiny evolved NN; stigmergy via fields; usually aggregated | ~50 ns |
| T1 | fish, sparrow | Boids steering + flee/forage switch | ~200 ns |
| T2 | rabbit, deer | drives + utility over ~8 actions, habituation, simple fear conditioning, 3–5 remembered patches | ~1 µs |
| T3 | wolf | + social drives, rank (Elo winner/loser effect), territory map, cooperative hunt roles, reinforcement per prey type | ~5 µs |
| T4 | ape, elephant | + individual recognition ledger, social learning, tool use, short planning | ~20 µs |
| T5 | human | full AMG + affect + beliefs + language + culture; LLM only when observed | 2–6 µs (L3) → ms (L4) |

Animals can also use NEAT-style evolved small networks for instincts (biosim4/neat-python are MIT) — evolved
per species at epoch boundaries, not per individual at runtime (→ 04, 08).

---

## 4. Answers to the brief's specific asks (one line each, details in reports)

| Brief item | Recommended mechanism | → |
|---|---|---|
| Neural pathways (fire → pain → fear) | Sparse associative graph; surprise-driven (Rescorla–Wagner + Pearce–Hall attention + TD chaining); context-gated extinction gives renewal; sleep consolidation; worked numeric trace from age 3 to 25 | 01 §8.4 |
| Brain development | One brain, age-curve parameters (learning rate, fear/reward asymmetry, workspace K, planning depth, ToM level, trust in parents vs peers); critical periods; developmental effects (A-not-B, false belief, adolescent risk) must *emerge* and be regression-tested | 01 §8.5 |
| Language emergence | Per-culture phoneme inventory (PHOIBLE, CC-BY) + syllable templates + ordered sound-change rules with lexical diffusion; naming game with lateral inhibition on the contact graph → dialects at low-contact barriers; intelligibility limits information flow; writing freezes forms | 02 Part C |
| Knowledge is local | Belief store with provenance; witness/tell/overhear/teach/write/read/lie/misremember/exaggerate/forget/infer; Talk of the Town evidence strengths & deterioration (read from its source); worked Village A → B murder example | 02 |
| Memory | Fixed budgets (e.g. 96/24/4 episodic slots by tier + pinned emotional slots), lazy FSRS/ACT-R decay, overnight consolidation, reconsolidation distortion, settlement lore with ~3-generation oral horizon | 02 |
| Emotions | Appraisal (GAMYGDALA core + EMA variables + CPM ordering) → discrete emotions → PAD mood → temperament; emotions change risk, planning depth, coping actions | 03 R1 |
| Personality | Big Five + HEXACO Honesty–Humility + ~12 facets + ~10 values; trait→parameter formulas; maturity-principle drift; animals get boldness/sociability/aggression/exploration | 03 R2 |
| Needs / motivation | Homeostatic drives + psychological needs; empathic coupling; incentive salience (wanting ≠ liking); goals emerge with commitment threshold | 03 |
| Decision making at scale | Utility + HTN/GOAP + BT; MCTS rare; ~50–80 ms CPU per real second for 100k agents; cohorts at 1M | 03, 07 |
| LLM role | Proposals via constrained decoding; triggers only (conversation, morning plan, nightly reflection, chronicle prose, myth/naming flavour); ~350 calls/game-day cap 1000 at 1×; background history only at 1000× | 03, 08 |
| Body | ~10 daily variables; Hall energy-balance model; Preece–Baines growth spurt; Borbély sleep; Gompertz–Makeham ageing hazard; full physiology engines (Pulse/BioGears) only offline to make lookup tables | 04 C.3 |
| Genetics / evolution | 128 loci × diploid; additive polygenic traits with per-trait heritability; pleiotropy for trade-offs; mutation 2e-4/locus/generation (god slider); NEAT-style genetic distance for speciation; allele-frequency evolution at population tiers | 04 C.2 |
| Reproduction | 8-step pipeline (mate search → pair bond → conception → pregnancy → birth risk → parental investment → weaning → independence) | 04 C.4 |
| Ecology / plants / food chain | Plant functional types, light competition, nitrogen cycle, seed dispersal by animals; Lotka–Volterra emerges from individual/population behaviour; extinction possible | 04 C.6 |
| Disease | Same parameters drive per-person (location contagion pools, pre-sampled transitions) and compartmental SEIR (age-structured, chain-binomial, coupled settlements) modes | 04 C.7 |
| Weather / climate / terrain | 15-step worldgen (plates → Priority-Flood → stream-power erosion → climate spin-up → rivers → soils → deposits); monthly per-tile climate ~10–50 ms/yr for 250k cells; stochastic weather objects | 05 §1–2 |
| Physics | No atoms: ~25 material properties; interaction formulas for strike/cut/join/heat/transform; temperature & fire as cellular fields | 05 §3 |
| Crafting / invention / technology | Agents learn "action + tool properties + target properties → effect"; curiosity rewards *new effects*; discoveries cached in a world-global physics table; worked flint→hafted axe and clay→tempered pottery traces | 05 §4 |
| Skills | Practice vs knowledge tracks (Cataclysm-DDA, read from source); exponential learning curve; rust only on practice; catch-up relearning; teaching efficiency falls with teacher–learner gap | 05 §5 |
| Cultural transmission of tech | Per-step copying fidelity; Henrich population-size gain/loss; cascading loss (Kolodny); fidelity thresholds (Lewis & Laland) | 05 §6 |
| Culture | Shared transmission engine at individual, cohort and population resolution; conformist/prestige bias; Axelrod-style drift | 06 §1 |
| Religion | Agency attribution to surprising high-consequence events → post-hoc rationalisation from personal memory → shared myth (minimally counterintuitive bias) → ritual (superstitious reinforcement) → office/institution → schism; worked lightning-on-the-mountain example over 120 years | 06 §2 |
| Relationships | Directed vector (familiarity, trust, affection, attraction, respect, fear, resentment, jealousy, loyalty…), Dunbar-capped ~150; kinship computed on demand | 06 §3 |
| Groups / politics | Detected from social graph; offices emerge when coordination need × prestige × group size cross thresholds; labels computed post hoc and can regress | 06 §4 |
| Economics | Gift/barter → commodity money (Kiyotaki–Wright conditions) → bazaarBot-style markets → coinage, only when conditions support it | 06 §5 |
| Crime / justice | Crime = violation of the culture's own norms; feud → compensation → arbitration → courts | 06 §6 |
| War | Lanchester/front resolution with demographic, economic, political, religious consequences | 06 §7 |
| Death | Grief, funeral practice, inheritance, succession crises, vacated offices, ancestors that can become gods | 06, 04 C.8 |
| History | True layer + biased accounts; significance = magnitude + surprise + consequences + story-pattern match + firsts + god actions; can rise retroactively | 06 §8 |
| God interface / LOD / performance | Five tiers L4→L0, multi-rate scheduler + timing wheel, budget governor, alibi generation, Rust core + Godot UI | 07 E |

---

## 5. Level of detail and performance (reconciled)

The reports used two numbering schemes; **this project uses the brief's: L4 = observed, L0 = macro.**
(Report 01 numbers LOD0 = focal; read its tables upside-down.)

### 5.1 Tiers

| Tier | Who | Mind | Cap (at 1×) | Memory / agent |
|---|---|---|---|---|
| **L4 Observed** | followed/possessed/inspected + interaction partners | Full AMG, lookahead depth 3–4, full episodic store, ToM inverse planning, LLM dialogue/reflection, full decision traces | 32 (8 at ≥25×) | 0.5–2 MB |
| **L3 Active** | on screen at street zoom, household & kin of L4, camera settlement, event participants | Full learning rules, lookahead 1–2, ~128 episodes, ~64 relationships, small decision ring | ~5,000 | **32–64 KB** (reconciled, see below) |
| **L2 Background** | everyone in the loaded region; all named/important people world-wide | Event-driven daily plans; Pavlovian + habits only; ≤256 edge deltas; top-16 memories; top-32 relationships | ~100,000 | **2–6 KB** |
| **L1 Statistical** | every other living human (and named animals) | Archetype policy + monthly/yearly vectorised pulses; 8–16 "scars" for anyone ever promoted | ~1,000,000 | **128 B core (+256 B scars if ever promoted)** |
| **L0 Macro** | unobserved peoples, animal & plant populations, fields, polities, species evolution | Cohort vectors, logistic/LV populations, SEIR, aggregate wars | unlimited | per settlement/cell |

**Budget reconciliation.** 07 sized L3 at 8–16 KB and L2 at 1–2 KB for *non-brain* state; 01 sized the brain
alone at 30–60 KB (L3) and 3–5 KB (L2). We adopt the sum and keep 07's caps, which gives roughly:
32 × 2 MB + 5k × 48 KB + 100k × 4 KB + 1M × 160 B ≈ 0.06 + 0.24 + 0.4 + 0.16 ≈ **0.9 GB** for a full
1M-human world, less when much of the population is animals (T0–T3 animals need ≤ 64 edges and no lexicon or
ToM). This is a design target, **not a measurement** — validating it is prototype P5.

### 5.2 CPU budget (from 07, design targets)

| Population | CPU per agent per real second at 1× | What runs |
|---|---|---|
| 10k | ~500 µs | everyone can be L3; whole village fully cognitive |
| 100k | ~50 µs | L3 near camera, L2 for the region |
| 1M | ~5 µs | mostly L1 pulses; ~1M at 1000× ≈ 3.2 s CPU per real second (≈64% of 6 cores) on paper |
| "Skip years" | — | ~1 game-year per real second at 1M; L1/L0 only; LLM writes chronicle prose in background |

Mind decision cost (01): ~2–6 µs per decision at L3, < 0.5 µs at L2. Disease and demography switch between
agent-based and compartmental (equation-based) modes by threshold (Hunter et al. hybrid ABM/EBM).

### 5.3 Continuity when the god zooms in (→ 07 E3)

Village simulated at L1 for 200 years → promotion materialises detail from deterministic hashes + the L1
record + the chronicle (kin from genealogy, non-kin ties from a seeded block model conditioned on recorded
marriages/feuds/shared disasters, memories instantiated from chronicle events — including god miracles —
appraised by personality and decayed analytically). A final **canon pass** applies observed facts and rejects
contradictory samples. If the village was L0, households and a 2–3 generation genealogy are synthesised
backwards, constrained by chronicle totals (the famine of year 1203 must show up as a mortality spike).
Round-trip conservation of population, stocks, wealth and allele frequencies is a CI test.

### 5.4 Engine recommendation (→ 07)

- **Simulation core:** Rust, headless, deterministic (fixed-point, counter-based RNG), ECS-style SoA columns,
  multi-rate system table + hierarchical timing wheel + vectorised pulses, region parallelism with
  deterministic message merge. Saves: versioned, compressed column chunks + god-command log.
- **Shipped front-end:** Godot 4 via GDExtension. Bevy for internal dev tooling. A WebGPU/wasm build is
  realistic around 100k agents. Unity DOTS not recommended.
- Permissive ECS/tooling usable as code: flecs, EnTT, Bevy, egui/ImGui, Tracy, Rerun (→ 07).

---

## 6. The god's inspector ("why did she do that?")

- **Always recorded (cheap):** chronicle events; per-agent last-8 decision records (16 B each) at L2+.
- **Only for L4 (and L3 small ring):** full decision trace: workspace contents, recalled memories (with
  why they were retrieved), appraisals and emotions, each option's score decomposition (habit vs. lookahead
  vs. Pavlovian vs. curiosity vs. social vs. persistence), arbitration weights, commitment check, and cheap
  counterfactuals ("would have eaten herself if hunger ≥ 0.95"). Full JSON example of the
  "give daughter food" case → 03.
- **For anyone not observed at the time:** replay from the nearest snapshot at higher tier, *labelled with the
  tier the agent was actually simulated at* — the inspector must never pretend a statistical person
  deliberated.
- Brain views: top contributing AMG edges for the last decision, extinction memories by context,
  crystallised beliefs and their provenance chains, lexicon, social models (→ 01 §9 risk 2).

---

## 7. AI models in the game (→ 08)

| Role | Recommended (late 2026) | License note |
|---|---|---|
| Dialogue / reflection (local) | Qwen3.5 (0.8–9B) or Gemma 4 (E2B–31B); Ministral 3 alternative; Qwen3.6-35B-A3B MoE for high-end/server | Apache-2.0 (Gemma 4 since 2026-04; Gemma 3 is *not*) |
| Runtime | llama.cpp (MIT) + LLamaSharp / llama-cpp-rs / godot-llm; vLLM/SGLang for a batch server; WebLLM for browser | NobodyWho is EUPL |
| Constrained output | GBNF / llguidance / XGrammar, schema rebuilt per request from live world ids | — |
| Cloud tier (optional) | Claude Haiku 4.5 / Sonnet 5.5 / Opus 5.5 with structured outputs, prompt caching, Batch API | paid API |
| Embeddings (LLM agents only) | Model2Vec potion, all-MiniLM-L6, Qwen3-Embedding-0.6B | MIT / Apache |
| TTS / STT | Kokoro-82M (espeak-ng phonemizer is GPL — isolate), Chatterbox; whisper.cpp, Moonshine | avoid XTTS-v2, F5-TTS (non-commercial) |
| Images (portraits/places) | FLUX.2 klein 4B via stable-diffusion.cpp | klein 9B/FLUX dev non-commercial; SD3.5 <$1M rule; SDXL OpenRAIL++; InstantID face models non-commercial |
| Creature "brains" | Not LLMs: AMG + utility; NEAT/ONNX tiny policies for animal instincts | — |

Prior art lessons: inZOI runs a 0.5B model (~1 GB VRAM) for *thoughts*, not dialogue; open-ended NPC chat
feels aimless without game-state grounding; players expect first token < 0.5 s. LLM work gets a **fixed token
budget per real second; the simulation never waits for it** above 5×.

### 7.1 Animation (→ 09)

- **Interface:** the sim emits a compact **Animation Intent Record** per visible agent (body descriptors from
  genome/age/build, locomotion, action + contact time, affect, condition such as injury/limp/pregnancy/elderly,
  paired interactions like carrying a child). Animation reads it and never writes sim truth (only cosmetic
  sound/VFX callbacks).
- **Procedural core:** Spore-style relative goals solved with IK; Overgrowth-style few-pose cycles driven by
  distance travelled; Godot 4.6 ships a full IK family and 4.5 added spring bones. Age and injury are layers
  (bone scale, asymmetric gait phase for a limp, additive posture). Patent watch: US 11129551 (age-related gait
  transformation) — get a legal read before copying that specific approach.
- **Animals:** no commercially clean generative model or quadruped dataset exists, so animals are procedural:
  Froude-number gait selection, phase-offset legs, two-bone/FABRIK IK, Rain World-style point-chain bodies.
- **AI motion:** pretrained weights of MDM, MoMask, T2M-GPT, MotionGPT are **not shippable** (trained on
  non-commercial AMASS/SMPL/LAFAN1/AI4Animation data). NVIDIA Kimodo and ARDY (2026; Apache-2.0 code, NVIDIA Open
  Model License weights, trained on commercially licensed mocap) are the best basis for an *offline* clip
  pipeline pending a legal read. Tencent HY-Motion excludes EU/UK/South Korea.
- **2D option:** DragonBones runtime (MIT), Inochi2D (BSD-2) for emotion-driven portraits, Wan 2.2 (Apache-2.0)
  offline for sprite/VFX; a Dead Cells-style 3D→2D bake pipeline. Spine is commercial; LPC sprites need
  per-layer license filtering.
- **Faces:** emotion → FACS action units → blendshapes or sprite face swaps; Rhubarb (MIT) lip-sync; Audio2Face
  optional (CUDA).
- **Crowd LOD:** observed person (IK + ragdoll + face) → GPU skinning from bone textures → vertex animation
  textures (OpenVAT engine decoders MIT; Babylon built-in) → impostors → dots at map scale.

## 8. License policy

The game may ship closed-source and commercially. Policy: **re-implement algorithms from papers in our own
code; copy code only from permissively licensed projects; keep a provenance log per algorithm.**

| Class | Examples (see each report for the full list) |
|---|---|
| **Code reuse OK** (MIT/BSD/Apache/Zlib/CC0) | llama.cpp, stable-diffusion.cpp, flecs, EnTT, Bevy, egui, Tracy, Rerun, Mesa, NetworkX, Tracery, bazaarBot, Axelrod-Python, krABMaga, Azgaar FMG (MIT), mapgen4, Fastscape, Crafter/Craftax, pyribs, QDax, biosim4, ALIEN, Lenia, neat-python, SharpNEAT (re-check), Covasim, Starsim, LASER, Pulse, BioGears, pyhgf, torchhd, RatInABox, PsyNeuLink, Model2Vec, Kokoro weights, Talk of the Town, DragonBones runtime, Inochi2D, Rhubarb, Wan 2.2, ozz-animation |
| **Algorithms only** (GPL/LGPL/AGPL/CC-BY-SA/APSL) | NetLogo, GAMA, FLAME GPU 2 (AGPL), AgentTorch (AGPL), BindsNET & htm.core (AGPL), NEST, Nengo, Explauto, SLiM, simuPOP, msprime, fwdpy11, iLand, Thrive, Veloren, openage, OpenTTD, Cataclysm-DDA (CC-BY-SA), Lexurgy, SMCDEL, WASABI, Jason, Avida (LGPL), platec (LGPL), Polyworld (APSL), ComfyUI (tool-only), espeak-ng |
| **Reference only / avoid** | FRED (non-commercial), Dwarf Fortress & CK3 (proprietary design references), Ensemble (BSD-4 advertising clause), repos with no license (Felt, Winnow, HRRL, shortBurglary), non-commercial model weights (XTTS-v2, F5-TTS, FLUX dev, NoobAI, InsightFace) and motion data/models (LAFAN1, AMASS, SMPL/SMPL-X, AI4Animation, MotionLCM, and weights trained on them) |
| **Attribution required** | PHOIBLE (CC-BY) |

---

## 9. Best ideas found (shortlist)

1. **Context-gated extinction** — fear is unlearned only in the context where safety was experienced, so it
   returns elsewhere (Bouton). Gives realistic phobias for free (→ 01).
2. **Avoidance preserves fear** — creatures that avoid a danger never test it, so the fear never extinguishes
   (→ 01 §8.4).
3. **Learning-progress curiosity** (Oudeyer IAC/ALP-GMM) produces stage-like development without scripting
   (→ 01).
4. **Talk of the Town's evidence model** — typed evidence with strengths, daily deterioration into forget /
   mutate / transfer, confident tellers more convincing (→ 02, read from source).
5. **Two timestamps per fact** (when it happened / when I learned it) and *supersede, don't delete*
   (Graphiti) (→ 02).
6. **Lazy decay at read time** (FSRS-style) — memory cost scales with use, not with time (→ 02).
7. **Empathic coupling** — a loved one's need enters your own drive; altruism emerges from utility (→ 03).
8. **Coping as mental actions** (EMA) — denial, blame-shifting, reappraisal appear in traces (→ 03).
9. **Property-affordance invention** with a world-global discovered-physics cache (→ 05).
10. **Practice vs knowledge skill tracks** with rust only on practice (Cataclysm-DDA source) (→ 05).
11. **Agency detection + post-hoc rationalisation** as the seed of religion; the god's own interventions become
    grounding events (→ 06).
12. **Retroactive significance** in the chronicle — an event matters more once its consequences unfold (→ 06).
13. **Alibi generation + canon pass** for consistent zoom-in after centuries of statistical simulation (→ 07).
14. **Rtsim-style low-resolution twins** (Veloren) keep off-screen people living (→ 07).
15. **Budget governor lowers detail before speed** (→ 07).

---

## 10. Consolidated risks

| Risk | Mitigation |
|---|---|
| The concept vocabulary quietly becomes a script | Perceptual-only ontology review; Drescher-style synthetic concepts; lint for behavioural concept names |
| Emergence is invisible or unreadable | Inspector and decision traces are first-class features, built before content |
| Runaway dynamics (phobias spreading through villages, ecosystem collapse, tech too fast/slow, trait drift) | Population-level dampers; calibration dashboards; regression suites per system |
| ~40+ mind parameters × age × genetics are hard to tune | Behavioural test suite from learning psychology (acquisition, blocking, latent inhibition, extinction/renewal, devaluation, A-not-B, false belief) as automated fitness for tuning |
| LOD transitions cause visible personality "pops" or contradictions | Top-20 beliefs preserved across tiers; canon pass; round-trip conservation tests |
| Statistical tiers diverge from what detailed agents would do | Calibrate L1/L0 policies by running the same scenario at L3 and fitting |
| LLM injects facts / breaks determinism | Concept-token interface with trust caps; schema-constrained outputs; logged external inputs; full no-LLM mode |
| Chronicle bloat over centuries | Significance-based compaction; vital events 16 B; summarisation tiers |
| Cross-platform determinism | Fixed-point everywhere in the sim; integer/LUT softmax |
| Licence contamination | Provenance log; algorithms-only list enforced in code review |
| Ethics of simulated suffering; heritable intelligence/pigmentation | Design review on how suffering is surfaced; avoid linking pigmentation to cognition or worth in genome design |

---

## 11. Recommended next step: R&D prototypes (still not the game)

Small headless experiments, each answering one question with a measurable pass criterion. Order matters:
P0 and P1 de-risk everything else.

| # | Prototype | Question | Pass criterion |
|---|---|---|---|
| **P0** | Deterministic core: Rust ECS-lite, fixed-point, counter RNG, multi-rate scheduler + timing wheel, snapshot/replay | Can we get bit-identical replays across runs/threads? | Same seed + command log ⇒ identical world hash after 10 game-years on 2 machines |
| **P1** | AMG in a grid world (Minigrid-like) with the learning-psychology test suite | Does fire→pain→avoidance and its known effects emerge without rules? | Acquisition, blocking, latent inhibition, extinction + renewal, avoidance persistence, devaluation, A-not-B all reproduced; ≤ 6 µs/decision |
| **P2** | Belief/gossip propagation across 3 villages with 2 dialects | Does local knowledge stay local and distort plausibly? | Murder known in B only via a carrier; measurable distortion; ~20% never hear it; no omniscience leaks (audit) |
| **P3** | Ecology grid: plants + rabbits (T2) + wolves (T3) + one disease | Do populations self-regulate and can species go extinct? | Stable or oscillating LV dynamics over 500 years for most seeds; overhunting collapse reproducible |
| **P4** | Invention sandbox: material properties + affordance learning | Do tools emerge and spread without recipes? | Hafted axe and pottery discovered in ≥ X% of seeds; knowledge lost in small isolated groups |
| **P5** | LOD round-trip & scale: 1M L1 + 100k L2 + 5k L3 | Do budgets and continuity hold? | Memory ≤ ~1 GB; conservation tests pass; zoom-in materialisation of a 300-person village < 20 ms |
| **P6** | Religion emergence: god strikes lightning repeatedly on one peak | Does a myth/ritual/taboo emerge and vary between runs? | Agency hypotheses → shared myth → ritual appear in most seeds, with different content per seed |
| **P7** | LLM boundary harness: local 4B model with GBNF schema over live world | Can dialogue be grounded and never mutate state illegally? | 0 invalid actions applied over 10k calls; p50 first token < 0.5 s on a mid GPU; identical replays from logged outputs |

After P0–P7, write the game design document and pick the vertical slice (one valley, a few hundred people,
animals, plants, one climate, the god panel).

---

## 12. Still-unverified items to re-check before adoption

From the track reports: MicroPsi2, SHOP3, py_trees, Curvature licenses; SharpNEAT license text; Melodie license;
Sana and Z-Image weight licenses; Voxtral TTS license; Chatterbox VRAM; Gemma 4 12B release date; Meta "Muse
Glimmer" release; OMNI repo license; code availability for Turchin 2013, GeoSim, DomWorld, Lane's simulations,
NetLogo Kiyotaki–Wright; HERMES; Veloren loaded/unloaded switching file (404); Cataclysm-DDA focus formula
location; most exact last-commit dates.
