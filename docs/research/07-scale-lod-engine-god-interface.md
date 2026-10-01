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
- License: the fetched page lists both MIT and GPL-3.0 license files **(exact split unverified; check before reusing any code)**
- Language: C#
- Activity/maintenance: very active (~695 stars engine; the content repo is larger).
- Architecture (concrete): a C# ECS engine with client/server and **ViewVariables (VV)**, a runtime inspector to view and edit any entity/component field by reflection, remotely on the server.
- Concept: multiplayer simulation-heavy roleplay game.
- Computational cost / performance characteristics: server ticks a large ECS with dozens of players.
- What we can learn: **reflection-driven, remote, editable inspector** with permissions. The god inspector should be generated from component metadata, not hand-written per field.
- What we could integrate: design of the reflection-based VV inspector; code only after the license split is confirmed.
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
- License: BSD (the fetched page indicated BSD 2-Clause; verify exact variant) [PERMISSIVE]
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

## Part E — Recommended architecture for our game

### E0. Ground rules

1. **One canonical world model, many resolutions.** Every person who exists has exactly one identity record (L1) for their whole life. Higher tiers *attach* extra state to that identity; they never replace it. Dwarf Fortress's "same data structures in all modes" and Veloren's "dual in rtsim" both point here.
2. **Observed facts are canon.** Anything the god has seen (inspected, followed, read in a tooltip) is written to an *observed-facts store* and is never contradicted by later lazy generation (alibi generation).
3. **Determinism.** The world is a pure function of (world seed, build version, generator versions, god-command log, external-input log). LLM outputs and any other non-deterministic inputs are external inputs and are logged.
4. **LOD per subsystem, not only per agent.** An agent's locomotion, perception, deliberation, memory, social and physiology subsystems each have a tier. The "agent tier" is the default that sets them all.
5. **Budget-driven, not distance-driven.** A per-frame governor (an LOD-Trader-style solver) assigns tiers by importance score under a CPU budget that depends on the current speed.

### E1. Time base

- Integer game time: `tick` = 1 game second (u64; enough for billions of years).
- **1x = 1 game minute per real second** (a game day is 24 real minutes, a little faster than The Sims and slower than RimWorld). The speeds below are multiples of that.
- Calendar: 1 day = 86,400 ticks. Pulses: hourly, daily, monthly (30 days) and yearly (360 days), so pulses divide evenly.

### E2. The five tiers

Importance score per agent (recomputed every ~real 250 ms for L2+ and on events):
`I = w_obs·observed + w_cam·camera_proximity + w_rel·relation_to_observed + w_story·narrative_salience + w_god·recent_god_attention + w_role·social_rank − w_age·time_since_last_promoted`.
Each tier has a hard cap and a soft budget; the governor fills tiers top-down by importance.

#### L4 — Observed person (full cognition)
- **Who:** followed, possessed or open-in-inspector agents, plus their immediate interaction partners while interacting. Cap: 8 at ≥25x, 32 at ≤5x.
- **State kept (≈0.25–2 MB each):** full perception list (percepts with source, distance, salience); attention weights; working memory (7±2 typed slots); full episodic memory store (thousands of entries: 48–64 B each with embedding id, salience, emotion tag, participants, location, time, source/provenance); semantic beliefs with confidence and provenance chain; emotions (appraisal-based, e.g. OCC-like categories + PAD core affect); needs; goal stack with utility decomposition; hierarchical plan (HTN/GOAP) with current step; skills with XP; full relationship records; genetics (genotype + expressed phenotype); a **decision-trace ring buffer** (see E6); LLM context cache (dialogue summaries).
- **Tick rates (game time):** locomotion/physics every tick (1 s); perception + attention every 5 s; deliberation (goal re-selection) every 30 s or on salient interrupt; plan repair on failure; memory encoding on event; consolidation and reflection during sleep; LLM calls (dialogue, reflection, inner monologue) asynchronous and budgeted, only at ≤5x (≤25x for the followed agent's dialogue).
- **Promotion into L4:** the god selects, follows or possesses; or the agent becomes the counterpart of an L4 agent's interaction (temporarily). **Demotion:** 60 real seconds after deselection, or when the speed rises above the tier cap. Before leaving L4, run memory consolidation.

#### L3 — Active individual (reduced cognition, on stage)
- **Who:** agents visible on screen at "street" zoom, the household and close relations of L4 agents, agents in the settlement where the camera is, and participants in an ongoing important event (battle, trial, festival). Cap: ~5,000 at 1x, falling with speed (see E4).
- **State kept (≈8–16 KB):** the same *kinds* of state as L4 with bounded sizes: ~128 episodic memories, ~64 relationships, ~64 beliefs, needs/emotions, a goal + plan of ≤8 steps, and a small decision ring (32 × 16 B). No LLM state.
- **Tick rates:** locomotion every tick within the loaded zone (path following, local avoidance via the uniform grid); perception every 30 s; deliberation every 3 game-minutes or on interrupt; utility AI with ≤16 considerations, no deep planning (plans come from templates).
- **Promotion:** entering camera frustum at street zoom (with prefetch as the camera zooms in), interaction with L4, or a god action targeting them or their location. **Demotion:** out of view for 2 game-hours and importance below threshold. Demotion summarises: keep the top-16 memories by salience, write notable memories to the chronicle, and keep the top-32 relationships.

#### L2 — Background individual (schedule-level)
- **Who:** all humans in the "loaded region" (the god's current region and neighbours), all named and important characters world-wide (rulers, heroes, anyone the god ever inspected, the families of L3/L4 agents). Cap: ~100k (memory-bound ~150 MB); at 1000x CPU-bound to ~100k.
- **State kept (≈1–2 KB):** the L1 record plus a daily schedule (activity blocks), current activity + location (with movement as **linear path segments/curves**, so no per-tick movement), needs as last value + last update tick (analytic catch-up), a compact attitude vector toward ~8 categories (family, faith, ruler, strangers…), the top-16 memories (32 B each), the top-32 relationships (12 B each), inventory/wealth summary, and the last 8 decision records (16 B each).
- **Tick rates:** **process-style / event-driven**: one daily plan at dawn (template choice by utility, ~10–20 µs), plus wake-ups from the timing wheel for activity changes, plus interrupts (events in the same building or settlement: fire, raid, death of a relative). Social encounters between L2 agents resolve as dice rolls on co-location (event templates), producing relationship deltas and chronicle entries.
- **Promotion to L3:** camera/importance. **Demotion to L1:** leaving the loaded region and importance below threshold for ≥1 game-week. Agents the god has inspected never drop below L2 until they die. To avoid RimWorld-style bloat, cap the "protected" set and compact the oldest protected agents' memories into chronicle entries.

#### L1 — Statistical individual (identity + vital statistics)
- **Who:** every living human not in L2+ (and optionally named animals). This is what lets the world hold 1M people.
- **State kept (96–128 B, SoA):** id (u32), birth tick (u32, stored in days), sex/flags (u8), settlement id (u32), household id (u32), mother/father ids (2×u32), spouse id (u32), profession (u8), health state (u8: healthy/sick-k/injured/pregnant…), Big-Five personality (5×u8), a compact genome (24–32 B: loci for heritable traits incl. fertility, intelligence, disease resistance, appearance seeds), skill summary (4×u8), wealth bucket (u8), faith/culture ids (2×u16), notable-event count + chronicle head index (u32), "materialisation generation" (u8), and a protected flag.
- **Tick rates:** **pulses**, vectorised over SoA columns: monthly (pregnancy progress, illness transitions, migration decisions, employment changes), yearly (ageing, death hazard by age × health × settlement conditions, marriage matching within the settlement marriage market, fertility draw, skill progression by profession). Each draw uses counter-based RNG keyed by (seed, id, pulse index, stream), so it is order- and thread-independent.
- **Outputs:** compact chronicle entries for vital events (birth/death/marriage/migration: 16 B each). At 1M people that is ≈75k events/year ≈ 1.2 MB/year raw, ≈240 MB per 200 years before compression and summarisation.
- **Promotion to L2:** region load, kin of a promoted agent, or importance. **Demotion to L0** (rare): only for remote, never-observed settlements when the population cap is exceeded, or for pre-history/off-map peoples.

#### L0 — Macro ecology and civilisation (aggregates and fields)
- **What:** (a) settlements/peoples without individual records: cohort vectors (age band 5 y × sex × role ≈ 20×2×8 floats), household count, stocks (food, tools, wealth), culture/belief prevalence distributions, institutions, and a settlement-level chronicle; (b) animal and plant populations per region cell (logistic and Lotka–Volterra-style dynamics with migration); (c) environmental fields on grids (temperature, rainfall, soil fertility, vegetation biomass, water, disease pressure, "divine favour"), updated by GPU or SIMD stencil kernels (Lenia/Taichi-style); (d) civilisation-level actors: polities, trade networks, wars resolved as aggregate force ratios (Mount & Blade / Nemesis-style dice, recorded in the chronicle); (e) species-level evolution as Thrive-style batch "auto-evo" at epoch boundaries.
- **Tick rates:** fields daily (or hourly for weather near the camera); populations monthly; polity politics monthly; species evolution per epoch (decade/century jumps).
- **Hybrid switching:** disease and demography use hybrid ABM/EBM switching (Hunter et al. 2020). When an L1/L2 settlement's epidemic exceeds a threshold, the infection process for its L1 members switches to compartmental SIR/SEIR with the individual counts as initial conditions. When the epidemic falls back, or the settlement is promoted, individuals are re-sampled into states in proportion to the compartments, constrained by canon (observed sick people stay sick).

### E3. Continuity: zooming into a village simulated statistically for 200 years

Two cases.

**Case A — the village was L1 for 200 years (the default for humans).** Individuals, genealogy and vital events already exist. Promotion L1→L2→L3 *materialises* the rest:
1. **Prefetch.** When the camera starts zooming toward settlement V, enqueue promotion jobs for V's residents ordered by distance to the camera focus. Budget ~2–4 ms/frame. A typical village of 300 people at ~50 µs per materialisation costs ~15 ms, done across 4–8 frames during the zoom animation.
2. **Deterministic fields** come from `H(seed, id, field, generator_version)`: detailed personality facets, appearance, voice, handwriting, quirks.
3. **Derived fields** come from the L1 record plus the chronicle: skills = f(profession history, years, genome aptitude); body/health = f(age, health state, injuries in chronicle); wealth/inventory = sampled from settlement stock distribution × wealth bucket.
4. **Relationships.** Kin come exactly from genealogy. Non-kin ties come from a seeded stochastic block model over the village (blocks = household, neighbourhood, profession, age cohort), with degree targets ~5 close / ~15 friends / ~150 acquaintances (Dunbar-like). They are then *conditioned on the chronicle* (marriages ⇒ prior courtship; recorded feuds ⇒ rivalry; co-survivors of a raid ⇒ bond).
5. **Episodic memories.** Query the chronicle for events involving the person, their kin, household and settlement (famines, raids, plagues, festivals, births/deaths of kin, **god miracles and curses**). Instantiate memories with an appraisal computed from personality and relationship to the event, apply analytic salience decay from event time to now, and keep the top-K. Add a few *gist* semantic memories ("I've milled grain for 22 years"). Very old events persist as shared cultural memory (beliefs/stories), not personal episodes, unless very salient.
6. **Beliefs.** Sample from the settlement's belief-prevalence distribution conditioned on parents' beliefs (if materialised) and on personal events (a cured child ⇒ stronger faith in the god).
7. **Current state.** Sample needs and emotions from the steady-state distribution for the time of day, shifted by recent events (a death in the household last week ⇒ grief).
8. **Canon pass.** Apply observed facts last, overriding any sampled value; if a sample is inconsistent (e.g. generated hatred toward someone the god saw them hug yesterday), reject and resample that field.
9. **Persist.** Materialised values become canon. On later demotion they are summarised, but the chronicle and the observed-facts store guarantee the important parts regenerate the same way.

**Case B — the village was L0 (aggregate only).** First run **L0→L1 individualisation**: build households from cohort counts with a seeded household-formation procedure (couples matched by age, children assigned to mothers by age gaps, widows/orphans placed by rates). Then synthesise a genealogy *backwards* for 2–3 generations, consistent with the settlement chronicle's vital totals (e.g. the famine of year 1203 must show a mortality spike among the synthetic grandparents). Then proceed as in Case A. Alibi rule: individuals that the L0 chronicle names (the chieftain killed in the raid of 1150) are constrained to exist with those facts.

**Demotion invariants (tested in CI):** population count, food stock, wealth and genome allele frequencies are conserved through L3→L2→L1→L0→L1 round trips within tolerance; observed facts are never lost; no dangling relationship ids (Veloren's "unhappy path matters").

### E4. Scheduler design (multi-rate + event queue + budgets)

Components:
1. **Fixed micro-step** (1 tick = 1 game second) only for L3/L4 locomotion and physical interactions inside the loaded zone. At high speeds the micro-step is sub-sampled: agents switch to path-segment movement (curves) and contact/collision are only evaluated at segment boundaries.
2. **Multi-rate system table.** Each system declares (tier mask, period in ticks, phase policy, read set, write set). Entities are hashed into `period` buckets, and each tick processes one bucket. Load is flat and each entity is touched exactly once per period. Examples: L4 perception P=5; L3 perception P=30; L3 deliberation P=180; L2 daily plan at a per-agent dawn offset.
3. **Event queue.** A hierarchical timing wheel (second / minute / hour / day / year wheels) holds agent wake-ups and scheduled world events (activity end, travel arrival, pregnancy due, harvest, festival, decay of a god blessing). The ordering key is (tick, priority class, entity id, sequence). It is deterministic, cheap (O(1) insert), and supports cancellation via generation counters.
4. **Pulses** (hourly/daily/monthly/yearly) run L1/L0 vectorised jobs. They are scheduled as ordinary events, so pulses and agent events interleave deterministically.
5. **Budgeted async queues** for expensive cognition (deep planning, memory retrieval with embeddings, LLM calls) use anytime algorithms with per-frame µs budgets (the OpenTTD opcode-budget idea). A result is applied at a **deterministic apply tick** (request tick + fixed latency L). If it hasn't arrived by then: at ≤5x the sim briefly waits; above 5x a deterministic fallback is used. Either way the outcome (LLM text or "fallback used") goes into the external-input log.
6. **Region parallelism.** The world is partitioned into regions. Within a tick, systems run data-parallel over entities (rayon) with double-buffered writes. Cross-entity effects are emitted as messages (FLAME GPU / Biosim4 pattern) and applied after a deterministic sort by (target id, source id, seq).
7. **Speed governor.** Every real frame: `target_ticks = real_dt × speed × 60`. Run ticks until the sim budget is spent. If behind, first **demote** (raise tier thresholds) rather than drop speed; at 1x keep detail and show a "sim lagging" indicator instead. Tier caps per speed are below.

| Speed | Game time per real second | L4 cap | L3 cap (10k world / 100k / 1M) | L2 policy | LLM | Notes |
|---|---|---|---|---|---|---|
| Pause | 0 | any | unchanged | unchanged | allowed (inspection Q&A) | inspect, edit, god powers queue |
| 1x | 1 min | 32 | 10k / 5k / 5k | all loaded-region humans | on (async) | full animation |
| 5x | 5 min | 16 | 5k / 3k / 3k | loaded region | followed agent + its dialogue partner | |
| 25x | 25 min (1 day ≈ 58 s) | 8 | 1.5k / 1k / 1k | loaded region | off; end-of-day summaries only | animations time-compressed |
| 100x | 1.7 h (1 day ≈ 14 s) | 1–4 | 400 (followed household + settlement leaders) | loaded region (up to 100k) | off | activities resolved at block level; walking drawn as interpolated segments |
| 1000x | 16.7 h (1 year ≈ 8.6 min) | 1 (summary mode) | ~50 (followed household) | ≤100k nearest by importance | off | map heat-overlays; L4 only records activity-level decisions |
| Epoch (beyond 1000x: "skip 10/100 years") | ~1 year per 1–5 s | 0 (followed agent protected at L2) | 0 | named characters only | off | L1 pulses + L0; on return, generate a "while you were away" chronicle digest |

### E5. Per-agent memory and CPU budgets

Assumed target machine: an 8-core desktop with 6 worker threads available to the sim at ~85% ≈ **5×10⁶ µs of sim CPU per real second**, and ~1–1.5 GB of sim RAM at 1M (~4 GB available).

**Per-tier unit costs (design targets, to be validated by prototypes):**

| Tier | Bytes/agent | Cost per update | Updates per game day | µs per agent per game day |
|---|---|---|---|---|
| L4 | 0.25–2 MB | perception 30 µs; deliberation 300–2000 µs; plan 1–5 ms on goal change | perception 17,280; deliberation 2,880 | ~1.5–6 s (≈0.5–4 ms per real s at 1x) |
| L3 | 8–16 KB | perception 8 µs; deliberation 60–120 µs; locomotion 0.2 µs/tick | perception 2,880; deliberation 480; locomotion ≤86,400 when moving | ~60–100 ms |
| L2 | 1–2 KB | daily plan 15 µs; wake-ups ~6 × 2 µs; encounters ~3 × 3 µs | ~10 | ~35 µs |
| L1 | 96–128 B | monthly pulse 0.2 µs; yearly pulse 1–2 µs | 1/30 + 1/360 | ~0.01 µs |
| L0 | per settlement 2–8 KB; per field cell 16–64 B | stencils/ODEs | daily/monthly | negligible per person |

At speed *s*, game days per real second = s / 1440. Sim CPU per real second ≈ Σ_tiers N_tier × (µs per game day) × s / 1440.

**Worked budgets:**

| World | Speed | Tier mix | Sim CPU per real s | % of 5 s budget | RAM (agents) |
|---|---|---|---|---|---|
| 10k | 1x | 32 L4 / 10k L3 (all) | 32×4 ms + 10k×(100 ms/1440) ≈ 0.13 + 0.69 = 0.82 s | 16% | 32 MB + 160 MB ≈ 0.2 GB |
| 10k | 25x | 8 L4 / 1.5k L3 / 8.5k L2 | 8 × ~4 ms × 25 (LLM off) + 1.5k×100 ms×25/1440 + 8.5k×35 µs×25/1440 ≈ 0.8 + 2.6 + 0.005 = 3.4 s | 68% | ~0.05 GB |
| 10k | 1000x | 1 L4 / 50 L3 / 10k L2 | 50×100 ms×0.694 + 10k×35 µs×0.694 ≈ 3.5 + 0.24 = 3.7 s | 75% (lower L3 to 25 for headroom) | ~0.02 GB |
| 100k | 1x | 32 L4 / 5k L3 / 95k L2 | 0.13 + 5k×0.069 ms + 95k×35 µs/1440 ≈ 0.13 + 0.35 + 0.002 = 0.48 s | 10% | 32 MB + 80 MB + 150 MB ≈ 0.26 GB |
| 100k | 100x | 4 L4 / 400 L3 / 99.6k L2 | 400×100 ms×0.069 + 99.6k×35 µs×0.069 ≈ 2.8 + 0.24 = 3.0 s | 60% | ~0.2 GB |
| 100k | 1000x | 1 L4 / 50 L3 / 100k L2 | 3.5 + 100k×35 µs×0.694 ≈ 3.5 + 2.4 = 5.9 s | **118% ⇒ governor drops L3 to ~15 and L2 to ~60k** | ~0.15 GB |
| 1M | 1x | 32 L4 / 5k L3 / 200k L2 / 795k L1 | 0.13 + 0.35 + 0.005 + ~0 = 0.49 s (+ spatial/path costs) | 10–20% | 32 + 80 + 300 + 100 MB ≈ 0.5 GB (+ grids, paths, chronicle ≈ 0.3–0.5 GB) |
| 1M | 1000x | 1 L4 / 25 L3 / 60k L2 / 940k L1 | 1.7 + 60k×35 µs×0.694 + 940k×0.01 µs×0.694 ≈ 1.7 + 1.5 + 0.007 = 3.2 s | 64% | ~0.5 GB |
| 1M | Epoch (1 yr/s) | 0 L3 / ~5k named L2 / rest L1 + L0 | L1: 1M×(12×0.2 + 1.5) µs ≈ 3.9 s; L2 named: 5k×35 µs×360 ≈ 63 ms | ~80% ⇒ **epoch speed ≈ 1 year/real second at 1M** | ~0.4 GB |

Takeaways:
- **Per-agent µs per real second available:** 10k ⇒ 500 µs; 100k ⇒ 50 µs; 1M ⇒ 5 µs. At 1000x, per agent per *game minute*: 10k ⇒ 0.5 µs; 100k ⇒ 0.05 µs; 1M ⇒ 5 ns. Individual minute-level cognition at 1000x is impossible above ~10k, which proves the need for schedule-level L2 and pulse-level L1.
- **Per-agent byte budget:** 1M agents in 1 GB ⇒ ~1 KB average, so the median person must be L1 (~128 B).
- **Hot record ≤ 64 B** for anything iterated every tick (position i32×2, velocity i16×2, tier u8, activity u8, target u32, path segment u32, region u16, flags u16…). One streaming pass over 1M hot records costs ~3 ms.
- Spatial grid rebuild (counting sort) for ~200k moving agents: ~1–2 ms per rebuild on CPU. At 1x, rebuild every tick only for L3/L4 (≤5k: <0.1 ms).
- Pathfinding is budgeted separately (e.g. 1 ms/frame) with hierarchical paths and cache sharing. L2 travel uses precomputed region-graph routes.
- Browser (wasm, 4–8 threads, ~1–2 GB): divide by ~2–3 for CPU and keep 1M only with L1 at ≤128 B. A realistic web target is 100k individuals.

### E6. The god inspector

**Principles:** never pay for recording what nobody looks at; make everything reconstructible; be honest about resolution.

**Always recorded (all tiers, cheap):**
- **World chronicle**: typed, append-only events (16–32 B): vital events, migrations, crimes, battles, disasters, god actions, institution changes. Indexed by entity, settlement and time; stored in yearly segment files; old segments summarised ("decade digests"), preserving anything referenced by observed facts.
- **Per-agent decision stub (L2+):** a ring of the last 8 decisions × 16 B: tick, chosen action id, goal id, dominant consideration code (hunger/safety/love/duty/faith/greed/fear/curiosity/obedience-to-god…), top-3 alternative scores (u8). This is enough for a one-line "why": "Went to the temple (faith 0.82 > hunger 0.40) after the drought."
- **Per-agent decision ring at L3:** 32 entries with percept-summary hash and emotion snapshot.
- **Replay substrate:** periodic snapshots (full yearly + delta daily, plus hourly deltas for the loaded region), the god-command log and the external-input log (LLM outputs).
- **Cheap metrics** for overlays: per-settlement aggregates (needs satisfaction, belief prevalence, disease, LOD tier counts), sampled every game hour into ring buffers for charts.

**Recorded only when observed (L4, record-on-observe):**
- A full **decision-trace ring buffer** per observed agent: ~4,096 entries × 128–512 B in a dedicated arena (0.5–2 MB/agent). Each deliberation record holds: percept list (top-N with attention weights and why attended), working-memory contents, retrieved memories (query, hits, scores), candidate goals with per-consideration utility terms (input value, response curve, output), chosen goal, plan/HTN decomposition, rejected alternatives with reasons, emotion appraisal deltas, belief updates, LLM prompt/response references, and timing (µs spent).
- **Pin/bookmark** flushes the ring to disk so long stories survive. Developers can mirror the same stream to Rerun with a "sim_time" timeline.

**Retroactive explanation ("why did she do that 3 days ago?"):**
1. If the agent was L4 at that time, read the trace.
2. Otherwise, find the nearest snapshot before T and **re-simulate the region in a background worker** with tracing enabled for the target. Determinism guarantees the same outcome, and the trace shows the reasoning *at the tier the agent was actually simulated at*.
3. The UI labels the resolution: "Reconstructed from schedule-level simulation (L2): daily plan chosen at dawn because…". The UI must never present an L4-style inner monologue for a decision made at L2. An optional "counterfactual: what would she think at full detail?" mode runs L4 cognition in a sandbox fork and is clearly labelled hypothetical.

**Panels:**
- *Brain*: perception cone overlay on the map, attention heat, WM slots, needs bars, emotion (core affect + discrete), goal stack with stacked utility bars, plan tree with current step, memory list (salience, source, emotional tag, "materialised/reconstructed" badge), beliefs with confidence and provenance chain (Talk-of-the-Town style "heard from X on day Y"), skills, genetics (genotype ↔ phenotype with inheritance arrows), family tree (genealogy browser over L1 records).
- *Why?*: a causal chain from decision → dominant consideration → contributing percept/memory/belief → originating chronicle event (possibly a god action), all clickable.
- *Population grid*: a Dwarf-Therapist-style virtualised table over any filtered set (100k rows with clipping), custom formula columns and heatmap colours.
- *Timeline*: a scrubbable chronicle timeline (openage-style curves for aggregates); jump-to-snapshot; "rewind and fork" for experiments (sandbox branch; the main timeline is untouched unless the god commits).
- *Map overlays*: needs, beliefs, disease, fertility, divine favour, **LOD tier map** (developer and "lab mode" toggle).
- *Developer tools*: egui/ImGui panels, a reflection-generated component inspector (bevy-inspector-egui / RobustToolbox ViewVariables style), Tracy zones per system and per tier, determinism self-check (periodic state hash à la 0 A.D.; a GGRS SyncTest-style rerun-and-compare in CI).
- *Remote inspector API*: the sim exposes a query/introspection API over a local socket (like Flecs Explorer's REST). The game UI, a browser dashboard and test scripts all use it.

**Possession:** the possessed agent is L4; the player's inputs are god commands in the command log (so replays work). Its perception is shown "through their eyes" (only perceived entities visible, beliefs annotate the world).

**God powers and LOD:** any targeted god action promotes the target area to ≥L2 for its duration (so effects are individually felt and remembered) and writes a chronicle event that L1/L0 belief models consume ("the drought ended after the offering"). Area effects on L0 (plague, fertility change) modify rate parameters directly.

### E7. Engine, language, determinism and saves

**Recommendation: a headless Rust simulation core plus a separate presentation layer.**

- **`sim-core` (Rust, no engine dependency):**
  - Storage: dense SoA tables for hot per-individual state; slotmap tables for sparse cognitive state (allocated only at L3/L4); cohort tables for L0. `bevy_ecs` used standalone is acceptable if its scheduler is driven by our own deterministic stepper. The relational abstract tier follows Veloren rtsim's lesson: not everything belongs in an ECS.
  - Parallelism: rayon; message-passing for cross-entity effects; deterministic merges.
  - Numbers: fixed-point (Q16.16 / i32) for positions, needs and stocks; f32 allowed only in presentation and in non-canonical analytics. Counter-based RNG keyed by hashes. No iteration over randomly seeded hash maps; use index maps or sorted vectors.
  - Content: data-driven (RON/JSON/TOML) professions, needs, actions and event templates, à la CDDA JSON and nyan.
  - Scripting (modding): only at event level with a budgeted VM (Lua/Rhai/WASM modules with fuel limits, OpenTTD-style), never in per-agent hot loops.
- **Presentation (pick one):**
  - **Godot 4 + GDExtension (godot-rust/gdext)**. *Preferred for the shipped game*: mature 2D/2.5D rendering, the best UI toolkit of the options for a dense inspector, editor and tooling, MIT license, web/mobile export. Render agents with MultiMesh/RenderingServer, never one Node per agent. Thrive shows Godot + C# + ECS works for a sim game.
  - **Bevy**: one language end-to-end and wgpu compute for fields; the UI story is weaker and the API churns every ~3 months. A good choice for the **developer harness/lab build** with bevy-inspector-egui.
  - **Web UI** (TypeScript + WebGPU, sim compiled to wasm in a worker): choose this if a browser release is a goal. Expect ~100k individuals, not 1M.
  - **Unity DOTS**: technically strong, but closed and license-risky, and it couples the sim to the engine. Not recommended.
  - **C++ (flecs/EnTT + ImGui + SDL/bgfx)**: viable alternative core if the team is C++-native; flecs relationships and Explorer are excellent. Rust is preferred for safety in a heavily parallel, long-lived codebase.
- **Process model:** the sim runs on its own threads at its own cadence. Each frame the UI receives a **render extract** (double-buffered view state for the visible area) and issues **commands** (god actions, inspector queries) through a channel. The UI never mutates sim state directly. This enables headless servers, automated testing, fast-forward without rendering, and future co-op.
- **Saves:** a container file with a header (build/schema/generator versions, seed), per-table columnar chunks (postcard/bincode or FlatBuffers) compressed with zstd, chronicle segment files, an observed-facts store, snapshot deltas, and the command and external-input logs. Autosave from a copy-on-write snapshot at a tick boundary (dirty-chunk tracking) to avoid hitches. Each table has a version and migration functions; generator versions are pinned per world, so lazy generation stays stable across game updates.
- **Determinism tests:** golden-seed runs in CI comparing state hashes every N ticks; a SyncTest (run, roll back to snapshot, re-run, compare); a cross-thread-count test (1 vs 8 threads must give identical hashes); and round-trip promotion/demotion invariant tests.

### E8. Implementation order (de-risking)

1. L1 + L0 world (1M people, pulses, chronicle, fields): proves the 1M memory budget and epoch speed.
2. L2 schedule agents + timing wheel + path segments: proves 100k at 1000x.
3. L3 utility agents + uniform grid + locomotion: proves 5k at 1x.
4. Promotion/demotion with materialisation + observed-facts store + invariant tests.
5. L4 cognition + decision-trace ring + inspector UI + retroactive replay.
6. LLM integration (async, logged), last, behind feature flags.

---

## Part F — Open questions and risks

1. **Perceived inconsistency on re-promotion.** A re-materialised villager may differ from the player's memory of them in unrecorded details. Mitigations: the observed-facts store (record what the UI *displayed*, not just what was clicked), a generous protected set, and stable seeded generation. Open question: how much UI exposure counts as "observed" (hover tooltips? background crowds?).
2. **Calibration between tiers.** L1 hazard rates and L2 schedules must statistically match what L3/L4 agents would do, or the world changes character when you look at it. Plan: offline calibration, where we run L3-detailed simulations of sample villages and fit L2/L1 parameters (surrogate modelling), repeated whenever L3 behaviour changes. Mesa/krABMaga-style parameter sweeps help.
3. **LLM determinism, latency and cost.** Treat LLM output as logged external input; cap calls by speed; provide deterministic fallbacks. Open: local vs cloud model, privacy, offline play, and whether LLM-written memories become canon.
4. **History bloat over centuries** (the RimWorld world-pawn lesson). We need compaction policies for chronicles, protected characters and relationships to the dead. Without them, save sizes and load times grow without bound.
5. **Retroactive replay cost.** Re-simulating a region from a snapshot is cheap for hours and expensive for months. Snapshot cadence trades disk against explanation latency. Open: retention window for "why?" queries (e.g. full detail for the last 30 game days, summary beyond).
6. **Cross-platform determinism.** Fixed-point avoids most float issues, but SIMD and GPU field kernels can diverge across vendors. Decision needed: are GPU fields canonical (then they need a deterministic CPU fallback) or purely presentational?
7. **Camera teleport thrash.** Fast travel across the world may outrun materialisation prefetch. Options: a short "focusing" transition, progressive refinement (show L2 crowds first), and a promotion budget that grows when paused.
8. **God actions that cut across tiers** (cursing one person in an L0 village) force immediate L0→L1→L2 individualisation in a single frame. Budget spikes need a pause-and-materialise UX.
9. **1M at 1000x is a soft target.** It works only because most people are L1. If design later demands daily individual behaviour for everyone (e.g. a detailed economy), 1M becomes infeasible on CPU. A GPU L2 is possible for movement but not for branching decisions.
10. **Browser constraints:** wasm threads need COOP/COEP, memory limits apply, and Safari WebGPU/Memory64 parity is uncertain. Decide early whether the web is a first-class target.
11. **Engine risk:** Bevy API churn and gdext binding maturity. Mitigated by keeping `sim-core` engine-agnostic with a thin adapter.
12. **Honesty of explanations.** Reconstructed L2 rationales are coarse, and the inspector must not imply richer minds than were simulated. This is a design and trust issue for a "science lab" god game.
13. **Licence hygiene.** Most deep references (Veloren, openage, CDDA, Thrive, OpenTTD, FLAME GPU 2) are GPL/AGPL/CC BY-SA, so take ideas only, no code. Reusable permissive pieces: flecs, EnTT, Bevy, bevy-inspector-egui, egui/ImGui, Tracy, Rerun, GGRS, krABMaga, Biosim4, Lenia, ALIEN (BSD), Taichi, Mesa (offline), Talk of the Town.
14. **Unverified items to follow up:** Brockington 2002 full text; Osborne & Dickinson citation; Hitman: Absolution crowd talk title; Mesa DEVS module names; KeeperRL and Unknown Horizons scheduler internals; Polyworld/Repast HPC exact licenses; Tracy exact BSD variant; RobustToolbox licence split; Memory64 support in Safari.

### Sources (primary pages fetched or found this session)
- Veloren: https://github.com/veloren/veloren ; https://gitlab.com/veloren/veloren/-/raw/master/rtsim/src/lib.rs ; https://gitlab.com/veloren/veloren/-/raw/master/rtsim/src/data/mod.rs ; https://gitlab.com/veloren/veloren/-/raw/master/rtsim/src/rule/simulate_npcs.rs ; https://gitlab.com/veloren/veloren/-/raw/master/common/src/rtsim.rs
- openage docs: https://raw.githubusercontent.com/SFTtech/openage/master/doc/code/curves.md ; https://raw.githubusercontent.com/SFTtech/openage/master/doc/code/event_system.md
- Chenney GDC 2001: https://pages.cs.wisc.edu/~schenney/research/culling/chenney-gdc2001.pdf
- Brom et al. 2007: https://link.springer.com/chapter/10.1007/978-3-540-74997-4_1
- Wißner et al. 2010: https://link.springer.com/chapter/10.1007/978-3-642-16958-8_20
- Sunshine-Hill & Badler: https://repository.upenn.edu/hms/114/ ; thesis https://repository.upenn.edu/edissertations/435/ ; LOD Trader https://dl.acm.org/doi/10.1145/2522628.2541250
- AC Unity crowds: https://gdcvault.com/play/1022411/Massive-Crowd-on-Assassin-s
- Morvan survey: https://arxiv.org/abs/1205.0561 ; multi-level time/consistency: https://arxiv.org/abs/1703.02399 ; Multi-Level Mesa: https://arxiv.org/pdf/1904.08315
- Hybrid ABM/EBM: https://www.jasss.org/23/4/14.html
- FLAME GPU 2 paper: https://onlinelibrary.wiley.com/doi/10.1002/spe.3207
- CDDA dynamic NPCs: https://github.com/CleverRaven/Cataclysm-DDA/pull/35124
- CK3 event scripting dev diary #30: https://forum.paradoxplaza.com/forum/developer-diary/crusader-kings-3-dev-diary-30-event-scripting.1397140/
- Widelands implementation notes: https://www.widelands.org/documentation/implementation/
- Thrive auto-evo: https://thrive.fandom.com/wiki/Automatic_evolution ; https://github.com/Revolutionary-Games/Thrive/pull/7267
- Repository URLs are listed in each Part D entry.
