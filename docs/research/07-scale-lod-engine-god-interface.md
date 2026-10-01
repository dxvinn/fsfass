# 07 — Scale, Simulation LOD, Performance Architecture & the God Interface

Research track 36 (god interface & observability), 37 (multiscale / level-of-detail simulation), 38 (performance architecture), plus a verified survey of open-source god games, colony sims, artificial-life worlds and simulation infrastructure.

Status: research only, no game code. Compiled 2026-10-01.

Verification method: every repository entry below was fetched from its GitHub/GitLab page (or raw LICENSE file) during this session. Star counts are approximate as displayed at fetch time. Where a fact could not be confirmed it is marked **(unverified)**. Several domains (veloren.net, gamedeveloper.com, arxiv.org abstract pages, devtrackers.gg) were blocked by the network proxy, so some literature facts come from search-result abstracts only and are labelled accordingly.

License flags used throughout:
- **[PERMISSIVE]** MIT / Apache-2.0 / BSD / Zlib / MPL-2.0 (file-level copyleft only) — code may be reused in a closed-source commercial game with attribution.
- **[COPYLEFT]** GPL / AGPL / CC BY-SA / APSL — study algorithms and designs only; do not copy code into a closed-source product.
- **[CLOSED]** not open source; included only as a design reference, not as a candidate.

---

## Part A — Key findings in one page

1. **Every successful large world sim splits "simulated in detail" from "simulated abstractly" and treats the boundary as a first-class system.** Veloren keeps a dual of every NPC inside `rtsim`, a deliberately relational, non-ECS, low-resolution world model that keeps running when no player is near; Cataclysm-DDA only fully simulates a 5x5 overmap-tile "reality bubble" and moves NPCs and hordes on the overmap outside it; Dwarf Fortress spent its 2014 "World Activation" release porting world-gen-only processes (armies, plots) into play mode using the *same data structures in all modes*; RimWorld keeps "world pawns" and garbage-collects unimportant ones after a year off-map. The lesson: design the abstract tier first, and make the detailed tier a *refinement* of it, never the reverse.
2. **Pulse scheduling beats uniform ticking.** CK3 runs only a few crucial systems daily and most systems monthly or yearly, with event hooks ("on_actions") firing on pulses or on world events. For 100k–1M agents this multi-rate approach is necessary.
3. **Lazy, consistent "fill-in-the-details" is a solved research problem with a name: alibi generation** (Sunshine-Hill & Badler 2010). Agents start with minimal state; any detail needed later is sampled *conditioned on everything previously observed*. Combined with counter-based deterministic RNG keyed by (world seed, entity id, field), this lets us materialise a villager's past on demand without storing it.
4. **Continuous LOD exists.** Brom, Šerý & Poch (IVA 2007) and Wißner, Kistler & André (2010) describe gradual, multi-level AI LOD rather than a binary on/off switch; Sunshine-Hill's "LOD Trader" (2013) treats LOD selection as a budget optimisation problem. We should do the same: a per-frame budget solver that picks each agent's tier.
5. **Hybrid agent/equation models are a mature epidemiology technique** (e.g. Hunter et al., JASSS 23(4) 14, 2020): run individuals while numbers are small or locally important, switch to compartmental ODEs when large, using aggregate counts as initial conditions. This is exactly our L1↔L0 boundary for disease, demography and ecology.
6. **Hard numbers:** AC Unity rendered ~10,000 crowd NPCs with only ~40 "real" AIs and ~120 high-resolution models, swapping via pooling (GDC 2015, Cournoyer). FLAME GPU 2 runs Sugarscape with up to 16M agents on a GPU. A WebGPU boids demo reaches ~16M 2D boids at 60 fps on an M1 Max, using a uniform grid (brute force caps near 50k even on GPU). 0 A.D. uses 250 ms lockstep turns and checksums state every 20 turns.
7. **Determinism is the enabler for the god interface.** If the sim is deterministic given (seed, god-command log), then "why did she do that?" can be answered *after the fact* by re-simulating from the last snapshot with tracing turned on. We don't have to record traces for 1M agents; we record traces only for observed ones, and can reconstruct any other agent's trace by replay.
8. **Engine choice:** a headless, deterministic Rust simulation core (custom SoA tables or `bevy_ecs` used as a library) with a separate presentation layer (Bevy or Godot 4 via GDExtension, plus egui/Dear ImGui debug tooling) is the best fit. Unity DOTS is capable but closed and ties the sim to Unity; pure Godot/GDScript can't reach 100k agents.

---

## Part B — Simulation LOD: literature and production techniques

### B1. Academic and GDC sources (verified via search unless noted)

| Source | Core idea | Use for us |
|---|---|---|
| Chenney, "Simulation Level-of-Detail", GDC 2001 (pages.cs.wisc.edu/~schenney/research/culling/chenney-gdc2001.pdf) | **Proxy simulations**: an in-view simulator for visible objects, a cheap proxy simulator for out-of-view objects that makes sure they *re-enter view at the right place and time*. | The constraint is *observational consistency*, not identical computation. Proxies for travel, combat outcomes and production. |
| Brockington, "Level-of-detail AI for a large role-playing game", *AI Game Programming Wisdom* (2002) | LOD AI for a large RPG; agents far from the player run reduced AI. Found by search; full text not fetched. | Distance/relevance-based tiers. |
| Brom, Šerý, Poch, "Simulation Level of Detail for Virtual Humans", IVA 2007, LNCS 4722 | **Gradual** LOD: several degrees of detail, applied to a storytelling world of tens of complex virtual humans; the world's places/objects are themselves LOD-structured (a hierarchy of areas). | Our LOD hierarchy should follow the *spatial* hierarchy (world → region → settlement → building → room), not just agent distance. |
| Wißner, Kistler, André, "Level of Detail AI for Virtual Characters in Games and Simulation", MIG 2010 | LOD classification applied to multiple behaviour aspects (movement, animation, decision making) separately. | LOD per *subsystem*, not per agent: an agent can have L4 social cognition and L2 locomotion. |
| Sunshine-Hill & Badler, "Perceptually Realistic Behavior through Alibi Generation", AIIDE 2010; Sunshine-Hill PhD thesis "Perceptually Driven Simulation" (UPenn, 2011) | Agents are created with minimal information. Information needed later is generated **on demand, consistent with previous observations**, by probabilistic modelling of observable state. | The exact mechanism for "zoom into a statistically simulated village". |
| Sunshine-Hill, "Managing Simulation Level-of-Detail with the LOD Trader", MIG 2013 (ACM DL 10.1145/2522628.2541250) | LOD selection is an optimisation: trade detail between entities under a fixed budget, weighting by perceptual importance. | Our per-frame budget solver. |
| Cournoyer, "Massive Crowd on Assassin's Creed Unity: AI Recycling", GDC 2015 | ~40 full AIs and ~120 high-res models drive scenes of ~10,000 crowd NPCs; a pooling system swaps low-res to high-res NPCs without the player noticing. | Pool and recycle "full brains". Full cognition is a scarce resource leased to agents, not a property of an agent. |
| Morvan, "Multi-level agent-based modeling — A literature survey", arXiv:1205.0561 (2012–13) | Taxonomy of multi-level ABM: levels as agents, aggregation/disaggregation, inter-level influence. | Vocabulary and pitfalls for L0–L2 couplings. |
| "On time and consistency in multi-level agent-based simulations", arXiv:1703.02399 (title verified by search; abstract page blocked) | Time models and consistency between levels that run at different temporal granularities. | Read before implementing level coupling. |
| Hunter, Mac Namee, Kelleher, "A Hybrid Agent-Based and Equation Based Model for the Spread of Infectious Diseases", JASSS 23(4) 14, 2020 | Disease component switches between ABM and equation-based once a percentage of agents are infected. | L1↔L0 switching rule for plague, famine and fertility. |
| Osborne & Dickinson, "grouped hierarchical LOD" | **(unverified)**: search did not confirm the citation. | Don't cite until found. |

### B2. Production systems (concept references)

- **Veloren rtsim** (open source, GPL-3.0; details in Part D). A whole-world, low-resolution simulation with a dual for every NPC. It throttles tick rates for distant or slow processes ("tick rates may vary over time and space") and spreads expensive work across many ticks. It uses a relational-database-like data model rather than ECS because "data access patterns are often discontinuous and unpredictable". It programs defensively: "the unhappy path *does* matter". State is persisted with MessagePack and has a version number that is bumped only for a hard purge.
- **Cataclysm-DDA reality bubble**. The loaded area is the player's overmap tile plus two tiles in every direction (5x5 overmap tiles). Outside it things mostly don't exist, with exceptions: horde movement, and NPCs travelling on the "overmapbuffer" to a destination (PR #35124 "Dynamic NPCs: find a base and travel there"). Effect-on-Condition scripts can run for NPCs outside the bubble.
- **Dwarf Fortress**. World generation simulates centuries of history at civ/site/historical-figure granularity, and fortress mode simulates a single embark in full detail. The 2014 "World Activation" release moved armies, invasions and villain plots into play mode, with "detailed data structures that are the same in all modes". Lesson: a single canonical data model for history, read by both tiers.
- **Crusader Kings III** (closed). Updates are split into daily, monthly and yearly; most systems are monthly. Scripted content fires through `on_actions` hooks: world events (birth, death) and pulses (yearly, every five years). Dev Diary 187 (2025) is a later performance diary (title verified; the page was blocked).
- **RimWorld world pawns** (closed). Off-map pawns persist. They are garbage-collected when they are "unimportant" (not faction leaders, kidnapped, caravan members, quest-reserved) and have been off-map for more than 1 year (5 for faction pawns). Pawns with relations or log references are kept, and the community found this creates unbounded growth (~2,000 world pawns, 200+ ticking each frame, in long saves). Lesson: reference-counted retention needs **compaction**: summarise the references, then free the pawn.
- **Shadow of Mordor Nemesis** (closed). Off-screen hierarchy of named orcs whose power struggles resolve as abstract events between encounters. Lesson: off-screen life can be a small set of *event templates resolved by dice*, as long as the outcome is recorded and narrated when observed.
- **Mount & Blade campaign map** (closed). Parties are points with strength numbers; battles auto-resolve off-screen and are only played tactically when the player joins. This is the classic two-tier "strategic proxy / tactical instance" split.
- **Hitman: Absolution crowds** (Fauerby, GDC 2012 "Crowds in Hitman: Absolution": **unverified title**, not fetched). Large crowds via cheap shared behaviour with individual promotion near the player.

### B3. Techniques catalogue (our synthesis)

1. **State summarisation on demotion**: keep sufficient statistics (counts, means, top-K salient items, last-event timestamps), drop the rest, and log what was dropped as a "summarised at tick T" marker.
2. **Lazy materialisation on promotion**: `value = f(seed, entity_id, field_id, conditioning_facts)`. Use counter-based RNG (SplitMix64/Philox/PCG keyed by a hash), so the result is order-independent and reproducible.
3. **Alibi constraints**: an "observed facts" store (what the god or another L4 agent saw) is immutable canon; the generator must sample conditioned on it, typically by rejection sampling or constrained construction.
4. **Analytic catch-up**: when an agent was dormant for Δt, advance needs, decay, ageing and skill rusting in closed form (exponential decay, logistic growth) rather than stepping through each tick.
5. **Event-template resolution**: at low LOD, an interaction ("trade", "fight", "courtship") is one dice roll with a precomputed outcome distribution, written into the chronicle.
6. **Compartmental mode**: population cells hold cohort vectors (age band × sex × role × health state) updated by difference equations; individuals are sampled out of a cohort when promoted.
7. **Hysteresis**: promote at importance > P, demote at < D with D < P and a minimum dwell time, to prevent thrashing.
8. **Conservation invariants**: food, money, population and genetic pool must be conserved across tier changes. Unit-test every promotion/demotion pair for this.

---

## Part C — Performance architecture research

### C1. Time models

- **Fixed tick** (RimWorld-style 60 ticks/s at 1x; OpenTTD-style day = fixed tick count). Simple, deterministic, and wasteful for sparse events.
- **Pulse / multi-rate**: each system has a period (every tick, every 10 ticks, daily, monthly). Stagger entities across the period (`(entity_id + tick) mod period == 0`) to flatten load. CK3-style.
- **Discrete event simulation (DES)**: a priority queue of (time, seq, event). SimPy (Python, MIT) is a process-based DES with generators as processes; DESMO-J is the classic Java DES library (**unverified** in this session). The openage event system is an `EventLoop` with an `EventQueue` sorted by invoke time. Handlers implement `invoke`, `predict_invoke_time` and `setup_event`, events are rescheduled when a dependency entity `changes(..)`, and events auto-cancel when targets die. That design is very close to what we need for births, pregnancies, harvests, disease incubation and aging.
- **Curves (openage)**: state stored as keyframed time→value curves (discrete, linear, segmented). Values at any time are interpolated, which gives free time travel and network efficiency at the cost of memory and interpolation time. Use them for slow-changing L1/L0 quantities (population, food stock, prices); positions of moving L1 parties can be "linear segment curves" so they need **no ticks at all** while travelling.
- **Command-queue determinism (Widelands)**: game logic is a function of the RNG seed, the starting conditions and the sequence of time-ordered commands in `Cmd_Queue`; commands run when game time ≥ their due time. This is the model for god commands too.
- **Lockstep with checksums (0 A.D.)**: 250 ms turns, state checksum every 20 turns (~5 s). Use the same periodic state hashing for our replay/desync self-checks.
- **Optimistic Time Warp** (Jefferson, 1985; classic PDES, not re-verified here): processes run ahead and roll back on straggler messages. Interesting for distributed or multi-region parallelism. **Not recommended** for v1: rollback of cognitive state is expensive. Conservative region-parallelism (regions sync at a lookahead boundary, e.g. 1 game hour, since nothing crosses a region faster than that) is simpler.
- **Timing wheels** (Varghese & Lauck 1987, classic; not re-verified): O(1) insert/expire for millions of timers. Use a hierarchical wheel (minute / hour / day / year buckets) for scheduled wake-ups and a binary heap only for the near-term bucket.

### C2. Data layout and ECS

- **Archetype ECS** (flecs, Bevy, Unity Entities): entities with the same component set are stored together in SoA chunks. Iteration is fast, but adding or removing components moves the entity, which is costly for frequent tier changes.
- **Sparse-set ECS** (EnTT, bitECS): O(1) add/remove, slightly slower joins. Good for tags like `Observed` and `Tier3` that flip often.
- **Relational tables (Veloren rtsim)**: plain `Vec`/slotmap tables keyed by id. Best for the abstract tiers, where access is graph-shaped (kinship, factions, sites).
- **Recommendation**: hybrid. Use dense SoA arrays for hot per-agent physical state of all individual-tier agents (L1–L4), slotmap tables for sparse cognitive state (allocated only at L3/L4), and cohort tables for L0. An ECS library is optional; the important part is that hot loops touch ≤64 bytes per agent.

Hot-path arithmetic: a 48-byte hot record × 1M agents = 48 MB. At a realistic ~20 GB/s effective memory bandwidth per socket, one streaming pass costs ~2.4 ms. So "touch every agent every tick" is affordable at 1M only a few times per frame, and anything heavier must be staggered or tiered.

### C3. Spatial partitioning

- **Uniform grid with counting sort** (rebuilt each tick by key → prefix sum → scatter): O(N), cache-friendly and GPU-friendly. This is the technique behind the 16M-boid WebGPU demo and the classic "fast fixed-radius nearest neighbour" approach. Use it for everything with a bounded interaction radius (perception, collision, local social encounters).
- **Spatial hash** (unbounded worlds): the same idea with hashed cell keys.
- **Quadtree / BVH**: for variable-size static objects (buildings, resources) and ray/region queries. Rebuild rarely.
- **Hierarchical region graph** (settlement → district → building) for "who is in the tavern" queries; this also doubles as the LOD hierarchy.
- **Flow fields / hierarchical pathfinding (HPA*)** with path caching. Pathfinding is the #1 CPU hog in DF-likes, so share paths between agents, cache them per (region pair) and invalidate on terrain change.

### C4. Parallelism

- **Job systems**: Rust `rayon` (work-stealing), Bevy's task pool, flecs' lockless scheduler, Unity's Burst/Jobs. Systems declare read/write sets and the scheduler runs non-conflicting systems concurrently.
- **Determinism under parallelism**: use counter-based per-entity RNG (never a shared RNG stream), double-buffered writes (read state at t, write t+1), deterministic reductions (sort by entity id before applying cross-entity effects), and no iteration over randomly seeded hash maps.
- **SIMD**: auto-vectorisation of SoA f32/i32 loops; explicit `std::simd`/`wide` only for need decay, field diffusion and steering.
- **GPU**: FLAME GPU 2 (AGPL; ideas only), Taichi (Apache-2.0), WebGPU/wgpu compute. Good for fields (temperature, pheromone, water, vegetation growth, disease pressure), steering and L1 movement. Bad for branching cognition. Keep cognition on CPU.

### C5. Determinism, replays, saves

- **Determinism contract**: same build + same seed + same god-command log ⇒ bit-identical world. Floats are acceptable on a single platform/binary if FMA contraction is controlled and there are no `fast-math` libm differences. For cross-platform replays, use fixed-point (i32 Q16.16) for positions and needs, or a soft-float libm.
- **Saves for multi-century worlds**: columnar, chunked, versioned; each component table is serialised separately and compressed with zstd. Use a **base snapshot + periodic delta snapshots** (e.g. full every game-year, delta every game-day), plus the god-command log. The history chronicle is append-only and stored in yearly segment files, with old segments optionally summarised.
- **Schema evolution**: explicit version numbers per table with migration functions. Veloren's approach (bump the version only for hard purges and tolerate missing fields with defaults) is pragmatic.

### C6. Language and engine survey (summary; candidates in Part D)

- **Rust**: memory safety without GC, excellent parallelism (rayon), and determinism is achievable. Mature ecosystem: bevy_ecs, wgpu, egui, serde, tracy bindings. Compiles to wasm. Slower compile times; Bevy API churn every ~3 months.
- **C++**: EnTT/flecs, Tracy, ImGui; maximum control; the mature language for the job; slower iteration and memory-safety risk.
- **C#**: Unity DOTS (closed, Burst gives near-C++ speed) or Godot C# / Arch ECS (used by Thrive). GC pauses need care; good tooling.
- **Zig**: attractive for data-oriented work but a small ecosystem; not recommended as the primary language.
- **Browser**: wasm + SharedArrayBuffer threads need COOP/COEP headers. wasm32 has a 4 GB address limit; Memory64 is shipping in recent Chromium/Firefox, but Safari support is **unverified**. Realistic browser target: ≤100k individual agents plus aggregate tiers.

---

## Part D — Verified open-source survey

### D1. Artificial worlds, colony sims and god-game-adjacent games

### Veloren (rtsim)
- URL: https://github.com/veloren/veloren (primary development on GitLab: https://gitlab.com/veloren/veloren)
- License: GPL-3.0 [COPYLEFT]
- Language: Rust
- Activity/maintenance: very active; ~7.6k stars on the GitHub mirror, ~18.5k commits.
- Architecture (concrete): the game server uses an ECS (historically `specs`) for loaded entities. `rtsim/` is a separate crate. `RtState` holds resources (data, time), **rules** (pluggable systems) and **event handlers** binding rules to events. `tick()` advances time and emits an `OnTick` event to all rules. Default rules cover NPC AI, resource replenishment, migration and cleanup. The persisted `Data` has `nature`, `actors`, `sites`, `factions`, `reports`, `architect`, `quests`, `tick`, `time_of_day` and `should_purge`, serialised with MessagePack (`rmp_serde`), version 11 at fetch time. NPCs have a Big-Five `Personality` (5 × u8), a `Role` (Civilised(profession) / Wild / Monster / Vehicle) and 11 professions. Unloaded NPCs move in `simulate_npcs` with an approximated speed (`max_speed_approx × speed_factor × dt`), clamped to their destination. They mount vehicles and airships, change home sites (updating site population) and stand still during gather/hunt/talk activities.
- Concept: each NPC has a dual in rtsim. On leaving player view it is subsumed into rtsim rather than despawned, and re-materialised when loaded.
- Computational cost / performance characteristics: designed for tens of thousands of NPCs in real time. Tick rates vary by distance and process; expensive work is spread over many ticks.
- What we can learn: the dual-representation pattern; a rule/event plugin architecture for the abstract tier; a relational data model for the abstract tier; "reports" (rumour/knowledge objects about events that NPCs can hold); defensive handling of dangling ids.
- What we could integrate: design only (GPL).
- What we should NOT integrate: code; its voxel-specific worldgen.
- Scalability: tens of thousands of NPCs world-wide on a server; per-NPC state is small.

### Cataclysm: Dark Days Ahead
- URL: https://github.com/CleverRaven/Cataclysm-DDA
- License: CC BY-SA 3.0 for the game, with third-party component licenses [COPYLEFT]
- Language: C++ (JSON data)
- Activity/maintenance: extremely active; ~13.3k stars, ~130k commits.
- Architecture (concrete): the reality bubble is 5x5 overmap tiles around the player (120 map squares a side) and is fully simulated. Outside it: overmapbuffer, horde movement, and NPCs that travel on the overmap to a "source of safety" (PR #35124). NPCs can spawn anywhere on the overmap (PR #35146). Effect-on-Condition scripts run for NPCs outside the bubble. A huge JSON data-driven content system.
- Concept: survival roguelike with a persistent, procedurally generated world.
- Computational cost / performance characteristics: turn-based, so there is no frame budget pressure. Bubble size bounds cost; known bugs exist around NPCs vanishing on bubble switches (issues #52061, #58325).
- What we can learn: the cost of a hard binary bubble (NPCs disappearing, follower desync). A multi-tier gradient is safer. The JSON data-driven content model is excellent for moddable professions and items.
- What we could integrate: ideas only (CC BY-SA).
- What we should NOT integrate: the binary bubble pattern; content (share-alike).
- Scalability: a single-player bubble; the overmap NPC count is modest.

### Dwarf Fortress (closed) + DFHack
- URL: https://github.com/DFHack/dfhack
- License: DFHack is Zlib [PERMISSIVE]; Dwarf Fortress itself is closed [CLOSED]
- Language: C++ (library), Lua (scripts)
- Activity/maintenance: active; ~2k stars, ~20.8k commits.
- Architecture (concrete): DFHack is a memory-access library with structure definitions for DF's internal data (units, historical figures, entities, sites) plus plugins and scripts. Reading its `df-structures` definitions is the closest thing to a public schema of DF's world model (historical figures, events, entity populations).
- Concept: modding/inspection layer over a closed simulation.
- Computational cost / performance characteristics: n/a (tooling). DF itself is famously CPU-bound by pathfinding, temperature and unit updates in fortress mode.
- What we can learn: how a mature sim separates *historical figures* (persistent, few fields) from *units* (active bodies), and how world-gen events are recorded as typed history events. The fact that players built DFHack and Dwarf Therapist proves demand for deep inspection, so the god inspector is a core feature, not a debug tool.
- What we could integrate: Zlib code is usable, but irrelevant to our engine. Schema ideas.
- What we should NOT integrate: anything from DF itself.
- Scalability: n/a.

### Dwarf Therapist
- URL: https://github.com/Dwarf-Therapist/Dwarf-Therapist
- License: MIT [PERMISSIVE]
- Language: C++ / Qt5
- Activity/maintenance: maintained; ~720 stars.
- Architecture (concrete): attaches to the running DF process, reads memory via layout files, and presents dwarves in sortable grids (skills, attributes, personality facets, needs, thoughts), with custom role ratings computed from weighted attributes.
- Concept: external "brain inspector" for a closed sim.
- Computational cost / performance characteristics: polling memory reads; cheap.
- What we can learn: the **grid view of hundreds of agents × dozens of traits** with heatmap colouring and custom formula columns is the right inspector UX for population-scale inspection, complementing the single-agent "brain view".
- What we could integrate: UX patterns; the MIT code is usable but Qt-specific.
- What we should NOT integrate: process-memory-reading approach.
- Scalability: hundreds of rows; we'd need virtualised tables for 100k.

### KeeperRL
- URL: https://github.com/miki151/keeperrl
- License: GPL-2.0 (with some Zlib parts) [COPYLEFT]
- Language: C++
- Activity/maintenance: maintained, commercial game on Steam with OSS code; ~1k stars, ~7.1k commits.
- Architecture (concrete): a dungeon-keeper colony sim with a world map of sites and per-site models. Detailed per-model scheduling internals were not verified in this session **(unverified)**.
- Concept: you are the evil keeper, managing minions, with adventurer-like retirement into a persistent world.
- Computational cost / performance characteristics: modest; turn-based with real-time display.
- What we can learn: how a commercial indie game ships with GPL source while selling assets separately; a world-map + site-model split.
- What we could integrate: ideas only.
- What we should NOT integrate: code.
- Scalability: hundreds of creatures per site.

### Goblin Camp (legacy C++) and goblin-camp (Rust re-imagining)
- URL: https://github.com/TheCatPlusPlus/goblin-camp-legacy (archived 2023-07-27) and https://github.com/acaradonna/goblin-camp
- License: legacy: not stated on the page **(unverified)**; Rust version: MIT/Apache-2.0 [PERMISSIVE]
- Language: C++17 (legacy) / Rust (new)
- Activity/maintenance: legacy archived (1 star); the Rust version is a small, recently active project (~2 stars, 155 commits).
- Architecture (concrete): the Rust version uses Bevy ECS, a deterministic fixed-step sim with seeded RNG, a hierarchical job system (mine/haul/build), A* with LRU path cache, and crates `gc_core` / `gc_cli` / `gc_tui`, with save/load.
- Concept: DF-inspired goblin colony management.
- Computational cost / performance characteristics: small scale.
- What we can learn: the Rust version is a compact reference for "headless deterministic core + multiple front-ends (CLI/TUI)", which is the architecture we recommend.
- What we could integrate: permissive Rust code could be read freely; it is too small to matter.
- What we should NOT integrate: legacy code (license unclear).
- Scalability: low.

### Talk of the Town
- URL: https://github.com/james-owen-ryan/talktown
- License: MIT [PERMISSIVE]
- Language: Python
- Activity/maintenance: research artefact, dormant (~105 stars, 18 commits).
- Architecture (concrete): simulates a town's founding and ~century of history at coarse granularity (business openings, marriages, births, deaths, moves), then switches to fine-grained day/night simulation. Characters hold **mental models** of other people and places; knowledge propagates through observation and conversation with transmission errors (memory decay, misremembering, lies).
- Concept: an AI-driven town where the player investigates using characters' imperfect knowledge.
- Computational cost / performance characteristics: Python, small towns; the coarse history phase skips most days.
- What we can learn: the **two-phase history (coarse) → play (fine) design**, and belief objects with provenance (who told whom, confidence, decay). This is directly applicable to our L1→L3 memory materialisation and to the belief/knowledge inspector.
- What we could integrate: algorithms and data model (MIT; code too, but Python).
- What we should NOT integrate: its performance profile.
- Scalability: hundreds of characters.

### Thrive
- URL: https://github.com/Revolutionary-Games/Thrive
- License: GPL-3.0-or-later for code; assets separate; uses Arch ECS (Apache-2.0), JoltPhysics (MIT) [COPYLEFT]
- Language: C# on Godot, with native C++ modules
- Activity/maintenance: active (~3.7k stars, ~10.5k commits, v0.8.x releases).
- Architecture (concrete): the Microbe stage runs real-time physics/ECS for nearby cells. **Auto-evo** is an off-screen, run-between-sessions evolutionary simulation: for each species it generates random mutations (five per species in the documented version), evaluates them with a population simulation over map "patches", picks the best (or none) and simulates migration. A "miche" (micro-niche) tree scores species against niches, with recent work caching occupant-pressure scores (PR #7267).
- Concept: Spore-like, scientifically grounded evolution game.
- Computational cost / performance characteristics: auto-evo runs as a batch job with progress bars, not per frame.
- What we can learn: **macro evolution as a discrete batch step at L0**, run while the player is in an editor or between epochs. For a god game, a "decade jump" can run species-level auto-evo, with individual genetics only for promoted agents.
- What we could integrate: algorithm ideas (GPL code).
- What we should NOT integrate: code/assets.
- Scalability: species × patches, cheap.

### Biosim4
- URL: https://github.com/davidrmiller/biosim4
- License: MIT [PERMISSIVE]
- Language: C++ (OpenMP)
- Activity/maintenance: maintenance-only; ~3.4k stars.
- Architecture (concrete): a 2D grid storing 16-bit indices of creatures. `Peeps` holds the population vector; each `Indiv` has a genome encoding a small neural net (sensor → internal → action neurons), location and parameters. The inner per-individual loop is thread-safe and parallelised with OpenMP; actions that mutate the world (moves, deaths) are queued and applied at end of step. Generational selection by survival criteria.
- Concept: evolving neural-net creatures under selection pressure.
- Computational cost / performance characteristics: thousands of agents × hundreds of steps per generation; fast because brains are tiny and deferred-mutation queues avoid locks.
- What we can learn: the **queue-world-mutations-then-apply** pattern (deterministic parallelism); compact genome → brain wiring; per-generation logging of genetic diversity.
- What we could integrate: MIT, so the genome-to-net encoding idea and even code could be adapted for animals/L1 instinct brains.
- What we should NOT integrate: the generational (non-overlapping) life cycle; we need continuous births/deaths.
- Scalability: ~10^3–10^4 agents per CPU comfortably.

### ALIEN
- URL: https://github.com/chrxh/alien
- License: BSD-3-Clause [PERMISSIVE]
- Language: CUDA C++
- Activity/maintenance: active (~5.5k stars, ~9k commits on develop, nightly builds).
- Architecture (concrete): a GPU particle engine where organisms are networks of bonded particles with genomes and neural networks. It needs CUDA compute capability 7.5+ (RTX 20+), with AMD via HIP/SCALE.
- Concept: physics-based artificial life lab with an editor.
- Computational cost / performance characteristics: "millions of particles" in real time on one GPU.
- What we can learn: GPU-resident world state with CPU only for UI; how to make an artificial-life lab inspectable (particle/cell editors, statistics windows).
- What we could integrate: BSD, so code is usable, but CUDA-only ties us to NVIDIA. Prefer concepts.
- What we should NOT integrate: a CUDA dependency for a consumer game.
- Scalability: millions of particles, GPU-bound.

### Lenia
- URL: https://github.com/Chakazul/Lenia
- License: MIT [PERMISSIVE]
- Language: Python (also MATLAB/JS implementations)
- Activity/maintenance: research reference, low activity (~3.9k stars).
- Architecture (concrete): continuous cellular automaton, where each step convolves the state with a ring kernel (FFT), applies a growth function and clips.
- Concept: continuous-space/time/state life forms.
- Computational cost / performance characteristics: O(N log N) per step via FFT; GPU-friendly.
- What we can learn: **field-based L0 ecology**. Vegetation, microbial mats and "spirit/mana" fields can be Lenia-like PDE/CA fields on the GPU, cheap at any speed.
- What we could integrate: MIT, so the algorithm and code are usable.
- What we should NOT integrate: n/a.
- Scalability: megapixel grids at interactive rates on GPU.

### Polyworld
- URL: https://github.com/polyworld/polyworld
- License: Apple Public Source License 2.0 (file-level copyleft on externally deployed modifications) [COPYLEFT-ish]
- Language: C++ (not displayed on page; historically C++/Qt/OpenGL)
- Activity/maintenance: low (~212 stars).
- Architecture (concrete): agents with vision (rendered colour retina), neural nets with Hebbian learning, energy, and mating, in a 3D world.
- Concept: classic artificial-life approach to AI (Yaeger).
- Computational cost / performance characteristics: vision-by-rendering is expensive per agent.
- What we can learn: **perception by rendering is too expensive at scale**. Use symbolic perception (query spatial grid, produce percept list) except for the observed agent, where we may *visualise* the percept list as a view cone.
- What we could integrate: ideas only.
- What we should NOT integrate: rendered-vision perception.
- Scalability: hundreds of agents.

### The Life Engine
- URL: https://github.com/MaxRobinsonTheGreat/LifeEngine
- License: GPL-3.0 [COPYLEFT]
- Language: JavaScript
- Activity/maintenance: low-moderate (~570 stars).
- Architecture (concrete): a grid CA where organisms are multi-cell bodies (mouth, producer, mover, killer, armor, eye). Reproduction by cloning with mutation (change/add/remove cell); simple eye-driven brains.
- Concept: browser evolution sandbox.
- Computational cost / performance characteristics: vanilla JS, modest grids.
- What we can learn: a clear "god sandbox" UX (paint food/walls, drop organisms, pause/speed), which is a model for minimal god tools.
- What we could integrate: ideas only.
- What we should NOT integrate: code.
- Scalability: low.

### Particle Life
- URL: https://github.com/hunar4321/particle-life
- License: MIT [PERMISSIVE]
- Language: C++ (+ JS, Python ports)
- Activity/maintenance: low (~3.4k stars).
- Architecture (concrete): typed particles with an attraction/repulsion matrix; O(N²) naive in the reference version, ported widely.
- Concept: emergent life-like self-organisation.
- Computational cost / performance characteristics: thousands of particles naive; scaling needs a grid.
- What we can learn: tiny rule sets give rich emergent visuals. Use for ambient micro-life at L0 visual LOD.
- What we could integrate: MIT, so algorithm/code.
- What we should NOT integrate: O(N²) loops.
- Scalability: low without spatial partitioning.

### openage
- URL: https://github.com/SFTtech/openage
- License: GPL-3.0-or-later [COPYLEFT]
- Language: C++20 core, Python, Qt6, nyan
- Activity/maintenance: active (~14.5k stars); gameplay currently rebuilt and "basically non-functional" by its own README.
- Architecture (concrete): **event-driven simulation**. `EventLoop` + `EventQueue` sorted by invoke time; handlers with `invoke`, `predict_invoke_time` and `setup_event`; dependency-driven rescheduling via `changes(..)`; auto-cancel on target expiry. **Curves**: keyframed time→value containers (discrete/continuous/segmented) so values at any time are interpolated; this gives easy desync recovery, network efficiency and time travel at a memory cost. The nyan language handles moddable game data.
- Concept: free engine clone of the Age of Empires Genie engine.
- Computational cost / performance characteristics: no per-tick work for entities in steady state; cost is proportional to events.
- What we can learn: the best open reference for **event-driven, curve-based state**, ideal for L1 travelling parties, stockpiles and slow variables, and for a scrubbable timeline in the god UI.
- What we could integrate: design only (GPL).
- What we should NOT integrate: code.
- Scalability: thousands of units (RTS scale).

### 0 A.D.
- URL: https://github.com/0ad/0ad (archived GitHub mirror; development moved to Gitea in Aug 2024)
- License: GPL-2.0 code, with MIT/LGPL parts; art CC BY-SA [COPYLEFT]
- Language: C++ engine, JavaScript components (SpiderMonkey)
- Activity/maintenance: active upstream (Gitea); GitHub mirror archived 2024-09-09.
- Architecture (concrete): "simulation2" component system with deterministic lockstep, 250 ms turns and a state checksum every 20 turns. Script components run in a sandboxed JS VM.
- Concept: historical RTS.
- Computational cost / performance characteristics: well-known pathfinding and AI script costs at high unit counts.
- What we can learn: periodic state hashing for OOS detection, which we reuse for **replay verification**; the cost of scripting VMs in hot loops (keep scripting at event level only).
- What we could integrate: ideas.
- What we should NOT integrate: code.
- Scalability: hundreds to low thousands of units.

### Freeciv
- URL: https://github.com/freeciv/freeciv
- License: GPL-2.0 [COPYLEFT]
- Language: C
- Activity/maintenance: active (~1.6k stars, ~32.7k commits).
- Architecture (concrete): authoritative server and thin clients; turn-based; ruleset files define content; AI players run server-side.
- Concept: Civilization-like empire builder.
- Computational cost / performance characteristics: turn-based, so cost is per turn.
- What we can learn: a client/server split even in single-player (the server is the sim and the client is a viewer). This maps to our "sim core process + UI" split. Rulesets as data.
- What we could integrate: ideas.
- What we should NOT integrate: code.
- Scalability: civ-scale (dozens of players, thousands of units/cities).

### Widelands
- URL: https://github.com/widelands/widelands
- License: GPL-2.0+ (assets various CC) [COPYLEFT]
- Language: C++, Lua
- Activity/maintenance: active (~3.1k stars). Notably has an explicit policy against AI-generated contributions.
- Architecture (concrete): game logic is determined by RNG seed + starting conditions + time-ordered `Cmd_Queue` (commands execute when game time ≥ due time). Lua scripts are enqueued as commands (`Cmd_LuaScript`, `Cmd_LuaCoroutine`). A long-standing issue requests multithreading (#2056).
- Concept: Settlers II-like economy RTS with carriers and roads.
- Computational cost / performance characteristics: single-threaded logic; many economy agents (carriers).
- What we can learn: the **command-queue-as-event-scheduler** model, which unifies player/god commands, scripts and scheduled actions into one deterministic queue.
- What we could integrate: design only.
- What we should NOT integrate: code.
- Scalability: thousands of workers; single-thread bound.

### Simutrans
- URL: https://github.com/aburch/simutrans (mirror of SVN)
- License: Artistic License 1.0 [PERMISSIVE-ish; review with counsel]
- Language: C++
- Activity/maintenance: active (~450 stars on mirror).
- Architecture (concrete): a transport simulation with passengers/mail/goods routed between cities and industries; tick-based world.
- Concept: transport tycoon.
- Computational cost / performance characteristics: passenger generation and routing dominate on large maps.
- What we can learn: **aggregate demand generation**: passengers are generated statistically from city size, not as persistent individuals. This is the L0/L1 trade pattern for our economy.
- What we could integrate: ideas.
- What we should NOT integrate: code without legal review.
- Scalability: large maps, aggregate demand.

### OpenTTD
- URL: https://github.com/OpenTTD/OpenTTD
- License: GPL-2.0 [COPYLEFT]
- Language: C++
- Activity/maintenance: very active (~8.3k stars, ~33k commits).
- Architecture (concrete): deterministic tick-based sim (fixed ticks per day) with lockstep multiplayer; desync detection; NewGRF/Game Script/AI scripting via Squirrel in sandboxed VMs with opcode budgets.
- Concept: Transport Tycoon Deluxe reimplementation.
- Computational cost / performance characteristics: vehicles tick every frame; AI scripts are throttled by opcode budgets per tick.
- What we can learn: **opcode/time budgets for scripted AI** (scripts suspend when out of budget and resume next tick). This is the right model for budgeted cognition and for any LLM call queue.
- What we could integrate: ideas.
- What we should NOT integrate: code.
- Scalability: thousands of vehicles.

### OpenRCT2
- URL: https://github.com/OpenRCT2/OpenRCT2
- License: GPL-3.0-or-later [COPYLEFT]
- Language: C++
- Activity/maintenance: very active (~16.3k stars).
- Architecture (concrete): a reimplementation of RCT2 with deterministic simulation for co-op multiplayer and a plugin/scripting API. Guest (peep) thoughts are visible in the UI.
- Concept: theme-park management; guests with needs, thoughts and money.
- Computational cost / performance characteristics: thousands of guests with simple need-driven state machines.
- What we can learn: **legible micro-thoughts** ("I'm hungry", "This ride is too intense") as a cheap, always-available explanation layer. Our L1/L2 agents should emit the same kind of templated one-line thought from their top utility term.
- What we could integrate: ideas.
- What we should NOT integrate: code.
- Scalability: thousands of guests.

### Unknown Horizons
- URL: https://github.com/unknown-horizons/unknown-horizons
- License: GPL-2.0 [COPYLEFT]
- Language: Python 3 (FIFE engine)
- Activity/maintenance: low (~1.6k stars).
- Architecture (concrete): Anno-like economy with a scheduler of timed callbacks (the game uses a Scheduler class for delayed calls; **unverified detail**). SQLite for map data, YAML objects.
- Concept: economy city builder.
- Computational cost / performance characteristics: Python limits scale.
- What we can learn: SQLite as a world-data store is a viable *save/inspect* format for the abstract tiers (queryable history).
- What we could integrate: ideas.
- What we should NOT integrate: code / Python sim core.
- Scalability: low.

### Mindustry
- URL: https://github.com/Anuken/Mindustry
- License: GPL-3.0 [COPYLEFT]
- Language: Java (Arc framework)
- Activity/maintenance: very active (~29k stars).
- Architecture (concrete): a tile-based factory/tower-defence sim with an entity code-generation system. Logistics (conveyors) are simulated per tile.
- Concept: automation RTS.
- Computational cost / performance characteristics: big factories are CPU-heavy; known for careful batching.
- What we can learn: tile-entity batching and the code generation used for components.
- What we could integrate: ideas.
- What we should NOT integrate: code.
- Scalability: tens of thousands of tile entities.

### Space Station 14 / RobustToolbox
- URL: https://github.com/space-wizards/RobustToolbox
- License: MIT (engine) per page shows GPL-3.0 and MIT entries; treat engine as MIT, content repo separately **(verify)**
- Language: C#
- Activity/maintenance: very active (~695 stars engine; the content repo is larger).
- Architecture (concrete): a C# ECS engine with client/server and **ViewVariables (VV)**, a runtime inspector to view and edit any entity/component field by reflection, remotely on the server.
- Concept: multiplayer simulation-heavy roleplay game.
- Computational cost / performance characteristics: server ticks a large ECS with dozens of players.
- What we can learn: **reflection-driven, remote, editable inspector** with permissions. The god inspector should be generated from component metadata, not hand-written per field.
- What we could integrate: if MIT is confirmed for the engine, code ideas for reflection-based VV; otherwise design.
- What we should NOT integrate: content (licensing mix).
- Scalability: server-scale entity counts.

### D2. Simulation frameworks (ABM, DES, GPU)

### FLAME GPU 2
- URL: https://github.com/FLAMEGPU/FLAMEGPU2
- License: AGPL-3.0-only, commercial license available [COPYLEFT]
- Language: C++/CUDA, Python bindings (with experimental Python→CUDA transpilation)
- Activity/maintenance: active research software (~157 stars; release-candidate status).
- Architecture (concrete): agents are typed state machines. Agent functions run as CUDA kernels; communication goes through typed **message lists** (brute force, spatial 2D/3D, bucket, array). Supports submodels (nested models), birth/death, ensembles via MPI, runtime compilation and real-time visualisation.
- Concept: domain-independent GPU ABM.
- Computational cost / performance characteristics: the Software: Practice & Experience 2023 paper reports Sugarscape with up to 16M agents, and 3.5× and 10× speedups from ensembles and concurrent execution.
- What we can learn: the **message-based decoupling** (agents never read each other directly; they publish messages into spatially binned lists) is ideal for parallel determinism on CPU too. Submodels map to our "region sub-sims".
- What we could integrate: patterns only (AGPL; the commercial license is an option if ever needed).
- What we should NOT integrate: the library itself in a closed game (AGPL) or a CUDA dependency.
- Scalability: 10^6–10^7 simple agents per GPU.

### Mesa
- URL: https://github.com/projectmesa/mesa
- License: Apache-2.0 [PERMISSIVE]
- Language: Python
- Activity/maintenance: active (~3.9k stars; Mesa 4 pre-release; JOSS 2025 paper).
- Architecture (concrete): Model/Agent classes, AgentSet, spaces (grid, continuous, network), browser visualisation (Solara). Later versions add experimental discrete-event scheduling (DEVS/ABM simulators) **(feature names unverified on the fetched page)**.
- Concept: Python ABM toolkit.
- Computational cost / performance characteristics: Python; 10^3–10^5 simple agents.
- What we can learn: a **prototyping bench** for L0/L1 rules (demography, disease, economy) before porting to Rust. "Multi-Level Mesa" (arXiv:1904.08315, verified title) is directly about multi-level ABM.
- What we could integrate: use as an offline tool.
- What we should NOT integrate: runtime.
- Scalability: low-moderate.

### krABMaga
- URL: https://github.com/krABMaga/krABMaga
- License: MIT [PERMISSIVE]
- Language: Rust
- Activity/maintenance: research project, low activity (~224 stars).
- Architecture (concrete): a MASON-inspired discrete-event ABM engine in Rust. `Schedule` with ordered agent steps; State/Agent traits; fields (grids, networks); Bevy-based visualisation; wasm build; optional parallel scheduling; MPI distribution; parameter sweeps/GA/Bayesian model exploration.
- Concept: Rust ABM for research.
- Computational cost / performance characteristics: fast Rust core; examples are small.
- What we can learn: a working Rust **DES schedule + Bevy visualisation + wasm** stack; model-exploration tooling for balancing (sweeping parameters of L0 models to match L3 behaviour).
- What we could integrate: MIT, so code is usable (schedule, fields) as a starting point or reference.
- What we should NOT integrate: wholesale framework (it's research-shaped, not game-shaped).
- Scalability: medium; depends on model.

### NetLogo
- URL: https://github.com/NetLogo/NetLogo
- License: GPL-2.0 [COPYLEFT]
- Language: Scala/Java/JS
- Activity/maintenance: active (~1.2k stars).
- Architecture (concrete): turtles/patches/links with a tick counter; huge model library.
- Concept: educational ABM.
- Computational cost / performance characteristics: modest.
- What we can learn: the **Models Library** is a treasure of validated small models (wolf-sheep predation, epidemics, segregation, flocking) to port as L0/L1 rules.
- What we could integrate: model *ideas* (check each model's license).
- What we should NOT integrate: code.
- Scalability: low.

### Repast HPC
- URL: https://github.com/Repast/repast.hpc
- License: in repo, type not shown on page **(unverified; historically BSD-style)**
- Language: C++ (MPI, Boost, netCDF)
- Activity/maintenance: low (~29 stars).
- Architecture (concrete): distributed ABM with spatial partitioning across MPI ranks and agent migration ("buffer zones" / ghost agents).
- Concept: HPC ABM.
- Computational cost / performance characteristics: scales to clusters.
- What we can learn: **ghost-agent buffer zones** at region borders, a pattern for region-parallel CPU sim with deterministic merge.
- What we could integrate: ideas.
- What we should NOT integrate: MPI stack.
- Scalability: very high on clusters.

### SimPy
- URL: https://gitlab.com/team-simpy/simpy
- License: MIT [PERMISSIVE]
- Language: Python
- Activity/maintenance: maintained (~900 commits, 20 tags).
- Architecture (concrete): process-based DES; processes are generators that yield events (timeouts, resource requests); the environment advances to the next event time.
- Concept: classic DES.
- Computational cost / performance characteristics: cost ∝ events, not time.
- What we can learn: model long-duration agent activities ("work 8h", "travel 3 days", "pregnancy 9 months") as **yielding processes** that cost nothing between wake-ups; good for L2 agents at 100x–1000x.
- What we could integrate: design pattern (MIT).
- What we should NOT integrate: Python runtime.
- Scalability: 10^5–10^6 events/s in Python; far more in Rust.

### Taichi
- URL: https://github.com/taichi-dev/taichi
- License: Apache-2.0 [PERMISSIVE]
- Language: C++ with Python front-end
- Activity/maintenance: active (~28k stars).
- Architecture (concrete): JIT-compiled kernels to CUDA/Vulkan/Metal/OpenGL/CPU (wasm experimental), sparse SNode data structures.
- Concept: high-performance numerical DSL.
- Computational cost / performance characteristics: GPU speed with Python ergonomics.
- What we can learn: prototype field simulations (hydrology, erosion, vegetation, disease pressure) quickly before writing WGSL/wgpu.
- What we could integrate: offline tooling (worldgen pre-processing).
- What we should NOT integrate: as a game runtime dependency (Python).
- Scalability: high.

### BoidsWebGPU
- URL: https://github.com/jtsorlinis/BoidsWebGPU
- License: MIT [PERMISSIVE]
- Language: TypeScript (Babylon.js, WebGPU)
- Activity/maintenance: small demo (~44 stars).
- Architecture (concrete): compute-shader boids with a uniform spatial grid built per frame (fixed-radius nearest-neighbour approach from GTC 2014).
- Concept: performance demo.
- Computational cost / performance characteristics: ~16M 2D / ~4M 3D boids at 60 fps on an M1 Max; the author notes brute force caps around 50k even on GPU.
- What we can learn: proof that L1 crowd motion for 1M agents in a browser is GPU-feasible.
- What we could integrate: MIT, so the grid-sort approach and code.
- What we should NOT integrate: n/a.
- Scalability: 10^6–10^7.

### D3. ECS, engines and tooling

### Bevy
- URL: https://github.com/bevyengine/bevy
- License: MIT/Apache-2.0 [PERMISSIVE]
- Language: Rust
- Activity/maintenance: very active (~48.5k stars); releases about every 3 months with breaking changes (0.19 era per bevy-inspector-egui compatibility).
- Architecture (concrete): archetype + sparse-set hybrid ECS (`bevy_ecs`, usable standalone); parallel system scheduling with automatic conflict detection; change detection; `Reflect`-based reflection; wgpu renderer; wasm support.
- Concept: data-driven engine.
- Computational cost / performance characteristics: fast ECS iteration; scheduling overhead per system is non-trivial when you have hundreds of tiny systems.
- What we can learn/use: `bevy_ecs` as the sim core's storage/scheduler, or Bevy as the full front-end. Reflection gives inspector generation for free.
- What we could integrate: code (permissive).
- What we should NOT integrate: coupling the deterministic sim to Bevy's frame loop or `Time` resource. Keep the sim in its own world/stepper.
- Scalability: 10^5–10^6 entities with care.

### flecs
- URL: https://github.com/SanderMertens/flecs
- License: MIT [PERMISSIVE]
- Language: C99 / C++17
- Activity/maintenance: very active (~8.7k stars, 13k+ tests).
- Architecture (concrete): archetype SoA storage; first-class **entity relationships** (pairs, e.g. `(Likes, Bob)`, `(ChildOf, Village)`); a query language with joins/inheritance; pipelines with a lockless multithreaded scheduler; reflection + JSON; **Flecs Explorer**, a web UI that inspects a running app over REST; stats addon; runs in browsers via Emscripten.
- Concept: ECS with relationships.
- Computational cost / performance characteristics: "millions of entities every frame" for simple systems.
- What we can learn/use: relationship queries are ideal for kinship/faction/ownership graphs. **The Explorer is the model for a remote god inspector** (sim exposes a REST/WebSocket introspection API; UI is a separate web app).
- What we could integrate: code (MIT) if we go C/C++.
- What we should NOT integrate: n/a.
- Scalability: high.

### EnTT
- URL: https://github.com/skypjack/entt
- License: MIT [PERMISSIVE]
- Language: C++20 header-only
- Activity/maintenance: very active (~13.2k stars; used by Mojang for Minecraft).
- Architecture (concrete): sparse-set ECS, groups (owned/partial) for fast joins, **snapshots** (serialise registry), signals, runtime reflection (`meta`).
- Concept: pay-for-what-you-use ECS.
- Computational cost / performance characteristics: excellent; O(1) add/remove suits frequent tier flips.
- What we can learn/use: snapshot API design for saves; sparse sets for LOD tags.
- What we could integrate: code (MIT) if C++.
- What we should NOT integrate: n/a.
- Scalability: high.

### bitECS
- URL: https://github.com/NateTheGreatt/bitECS
- License: MPL-2.0 [PERMISSIVE, file-level copyleft]
- Language: TypeScript
- Activity/maintenance: active (~1.5k stars).
- Architecture (concrete): SoA typed arrays, sparse sets, queries, serialisation; ~5 kB.
- Concept: minimal, fast JS ECS.
- Computational cost / performance characteristics: best-in-class for JS; typed arrays are shareable with workers.
- What we can learn/use: if a browser-only prototype is built, bitECS + workers is the quickest path.
- What we could integrate: code (MPL: modifications to MPL files must be shared; using it unmodified is fine).
- What we should NOT integrate: as long-term core for 1M agents (JS limits).
- Scalability: ~10^5 in a browser main thread.

### Becsy
- URL: https://github.com/lastolivegames/becsy
- License: MIT [PERMISSIVE]
- Language: TypeScript
- Activity/maintenance: low (~300 stars).
- Architecture (concrete): typed components with multithreading designed around SharedArrayBuffer.
- Concept: multithreaded JS ECS.
- Computational cost / performance characteristics: good; multithreading complexity.
- What we can learn: system read/write declarations for scheduling in JS.
- What we could integrate: code (MIT).
- What we should NOT integrate: core for large scale.
- Scalability: medium.

### Specs and Legion (historical Rust ECS)
- URL: https://github.com/amethyst/specs ; https://github.com/amethyst/legion
- License: Specs MIT/Apache-2.0; Legion MIT [PERMISSIVE]
- Language: Rust
- Activity/maintenance: ~2.6k / ~1.7k stars. Both belong to the former Amethyst project and are effectively superseded by `bevy_ecs`/`hecs`. Treat them as historical (current maintenance level **unverified**).
- Architecture (concrete): Specs uses storage-per-component (Vec/Dense/Hash) with a parallel dispatcher and was used by Veloren. Legion uses archetype chunks with a scheduler.
- Concept: early Rust ECS.
- Computational cost / performance characteristics: good in their day.
- What we can learn: Veloren's experience with Specs led it to keep rtsim *outside* ECS.
- What we could integrate: no need.
- What we should NOT integrate: new dependency on unmaintained crates.
- Scalability: n/a.

### Godot Engine (+ C# / GDExtension)
- URL: https://github.com/godotengine/godot
- License: MIT [PERMISSIVE]
- Language: C++ (GDScript, C#, GDExtension bindings incl. Rust via godot-rust/gdext)
- Activity/maintenance: very active (~118k stars).
- Architecture (concrete): scene tree of nodes; servers for rendering/physics; editor tooling; export to desktop/mobile/web/consoles (console via third parties).
- Concept: general engine with an excellent editor and UI system.
- Computational cost / performance characteristics: a node per agent does not scale past ~10^4. Use MultiMesh/RenderingServer directly, with the sim in GDExtension.
- What we can learn/use: Godot's UI (Control nodes, themes, rich text, graphs) is far ahead of Bevy's for building a dense inspector UI quickly. Thrive proves Godot + C# + ECS (Arch) works for a simulation game.
- What we could integrate: engine (MIT).
- What we should NOT integrate: sim logic in GDScript or as per-agent Nodes.
- Scalability: front-end only for large sims.

### bevy-inspector-egui
- URL: https://github.com/jakobhellermann/bevy-inspector-egui
- License: MIT/Apache-2.0 [PERMISSIVE]
- Language: Rust
- Activity/maintenance: active (~1.6k stars; tracks Bevy releases up to 0.19).
- Architecture (concrete): displays any `Reflect` value with egui. WorldInspector shows entities/resources/assets; ResourceInspector takes attribute hints (`min`, `max`) and has a `highlight_changes` feature.
- Concept: zero-effort runtime inspector.
- Computational cost / performance characteristics: immediate-mode; fine for a few entities, not for 100k-row tables.
- What we can learn/use: day-1 developer inspector; a pattern for attribute-driven field display.
- What we could integrate: code.
- What we should NOT integrate: as the *player-facing* god inspector (needs bespoke UX).
- Scalability: debug-scale.

### Dear ImGui
- URL: https://github.com/ocornut/imgui
- License: MIT [PERMISSIVE]
- Language: C++ (bindings for Rust, C#, etc.; WebGPU backend)
- Activity/maintenance: very active (~76.5k stars; v1.92.x in 2026).
- Architecture (concrete): immediate-mode GUI emitting vertex buffers; docking, tables with clipping (ListClipper for huge lists); ImPlot ecosystem for time-series.
- Concept: debug/tool UI standard.
- Computational cost / performance characteristics: cheap per frame; the list clipper makes 1M-row virtual tables feasible.
- What we can learn/use: developer tools, perf HUD, decision-trace viewers. In Rust, `egui` is the equivalent.
- What we could integrate: code.
- What we should NOT integrate: player-facing UI (styling limits).
- Scalability: excellent with clipping.

### Tracy Profiler
- URL: https://github.com/wolfpld/tracy
- License: BSD-3-Clause per repository **(page said BSD 2-/3-clause; verify)** [PERMISSIVE]
- Language: C++ (Rust bindings `tracy-client`, Zig, C#)
- Activity/maintenance: very active (~16.8k stars).
- Architecture (concrete): nanosecond-resolution instrumented zones + sampling, remote telemetry, GPU zones (Vulkan/D3D/Metal/CUDA/WebGPU), memory and lock profiling, frame images.
- Concept: frame profiler.
- Computational cost / performance characteristics: a few ns per zone.
- What we can learn/use: mandatory from day one. Instrument every system, plus per-tier agent counts as plots.
- What we could integrate: yes.
- What we should NOT integrate: n/a.
- Scalability: n/a.

### Rerun
- URL: https://github.com/rerun-io/rerun
- License: MIT/Apache-2.0 [PERMISSIVE]
- Language: Rust (+ Python, C++ SDKs)
- Activity/maintenance: very active (~11.5k stars).
- Architecture (concrete): log multimodal time-stamped data to columnar storage; a viewer (native/web) with **timeline scrubbing** across multiple timelines; query via dataframes/SQL.
- Concept: data layer/visualiser for robotics/physical AI.
- Computational cost / performance characteristics: the README acknowledges limits with very large entity counts.
- What we can learn/use: an excellent **developer-side time-travel viewer** for decision traces of observed agents (log percepts, utilities, plans per sim tick on a "sim_time" timeline). It can be used during R&D immediately, without building UI.
- What we could integrate: SDK in dev builds.
- What we should NOT integrate: as player UI or for all agents.
- Scalability: thousands of entities per stream.

### GGRS
- URL: https://github.com/gschup/ggrs
- License: MIT/Apache-2.0 [PERMISSIVE]
- Language: Rust
- Activity/maintenance: active (~690 stars).
- Architecture (concrete): GGPO-style rollback returning "requests" (save/load/advance) to the caller; sync-test sessions that roll back every frame to detect non-determinism.
- Concept: rollback netcode.
- Computational cost / performance characteristics: needs cheap save/load of state.
- What we can learn/use: **SyncTest-style determinism test harness**: run N ticks, roll back, re-run, compare hashes. We need that in CI even with no multiplayer.
- What we could integrate: the testing idea; possibly the crate if co-op god mode is added.
- What we should NOT integrate: rollback for a 1M-agent world (state too large).
- Scalability: small states.

### Unity DOTS (Entities/Burst/Jobs) — reference only
- URL: (closed; Unity Companion License) [CLOSED]
- License: proprietary
- Language: C# (HPC# subset + Burst)
- Activity/maintenance: active commercial product.
- Architecture (concrete): archetype chunks (16 KB), job system with safety checks, Burst LLVM compilation, baking/subscenes.
- Concept: data-oriented Unity.
- Computational cost / performance characteristics: near-native; proven at 10^5–10^6 entities.
- What we can learn: chunk sizing, enableable components (flip tiers without structural changes), and the job dependency model.
- What we could integrate: n/a.
- What we should NOT integrate: engine lock-in for the sim core; licence and pricing risk.
- Scalability: high.

**Candidates considered but excluded or unverified:** Gnomoria (closed), Ultima Ratio Regum (closed source), The Bibites (closed), Species: ALRE (closed), WorldBox (closed), RimWorld (closed, but decompiled-modding knowledge is widespread), Songs of Syx (closed). Small "Primordial" repos on GitHub (Primordial-sim/primordial, jkh2/Primordial-Sim, zainKhushall/primordial) exist but are tiny and unvetted, so they are not recommended as study material. Ken Stauffer's *Evolve* (rubberduck203/Evolve) exists; license not checked. DESMO-J and MASON are known Java DES/ABM libraries but were not verified this session.

---
