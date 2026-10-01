# 06 — Culture, Religion, Social Relationships, Groups, Politics, Economics, Crime, War, History

Research track: (25) Culture, (26) Religion, (27) Social relationships, (28) Group formation,
(29) Politics, (30) Economics, (31) Crime & justice, (32) War, (35) History recording, plus the
social consequences of death (inheritance, vacated positions, grief).

Date of research: 2026-10-01. Method: web search plus direct GitHub page/raw-file fetches. The
GitHub REST API and many academic hosts (arXiv, Semantic Scholar, PMC, ETH, Oxford CSSC, author
homepages) were blocked by the sandbox egress proxy, so:

- **Repo metadata** (license, language, stars, commit count) was read from the GitHub HTML page or
  the raw LICENSE file. Exact last-commit dates were usually not visible in the page excerpt; I
  give the commit count and whatever activity signal was visible. Treat star counts as approximate.
- **Model formulas** for papers whose PDFs were blocked come from the search-result abstracts plus
  my own knowledge of the literature. Those are marked "(from literature; PDF not re-read)".
- Anything I could not confirm is marked **unverified**.

Already known to the user and deliberately not re-reviewed: Talk of the Town, Neighborly,
socialSimulator, SocialSim, Concordia, Project Sid, Generative Agents.

License key for a possibly commercial closed-source game:
- SAFE to embed code: MIT, BSD-2/3, Apache-2.0, Unlicense, AFL-3.0 (attribution; check patent clause).
- CAUTION: BSD-4-Clause (advertising clause — needs acknowledgment in ads; avoid or get permission).
- DO NOT embed: GPL-2/3, AGPL-3 (copyleft), CC-BY-SA for code (share-alike), any EULA/"research-only".
  Read papers and re-implement algorithms from scratch in our own words instead.

---

## Part A — Agent-based modeling frameworks (evaluation for our engine)

We will almost certainly write our own engine (ECS + data-oriented, Rust/C++/C#), but these
frameworks are where the reference implementations of social models live, and they show which
scheduling/space/LOD patterns scale.

### Mesa
- URL: https://github.com/projectmesa/mesa
- License: Apache-2.0
- Language: Python
- Activity/maintenance: ~3.9k stars, ~3.2k commits, Mesa 4 in active pre-release; monthly dev sessions. Very active.
- Architecture (concrete): `Model` owns an `AgentSet` (since Mesa 3 the old schedulers were replaced by agent-set operations such as shuffle-do), discrete spaces (`OrthogonalMooreGrid`, `HexGrid`, `Network`, `VoronoiGrid`) built on `Cell` objects with property layers (numpy arrays per cell), continuous space, a `DataCollector`, and an experimental discrete-event scheduler. The bundled examples include Boltzmann wealth, Schelling, Epstein Civil Violence, Sugarscape with traders (G1MT), Demographic Prisoner's Dilemma and an **Alliance Formation** model that uses meta-agents.
- Concept modeled: general ABM; the examples cover wealth inequality, rebellion, trade, cooperation, coalitions.
- Computational cost: pure Python, so roughly 10^4 agents is comfortable and 10^5 is slow.
- What we can learn: the **meta-agent** pattern in Alliance Formation. A group is itself an agent with a membership manager that supports overlapping and nested groups. That is exactly what our band→tribe→state nesting needs. Also the property-layer pattern (per-cell numpy arrays for "sacredness", "fear", "crime attractiveness").
- What we could integrate: Apache-2.0, so code is OK, but it is Python. Use it as the **prototyping/test-bed** language for social rules before porting them to the engine. Re-implement the alliance-formation idea (bilateral Shapley value test) natively.
- What we should NOT integrate: the runtime. It will not hit 10^5–10^6.
- Scalability: low (CPU, single-threaded Python).

### Mesa-examples (community models)
- URL: https://github.com/projectmesa/mesa-examples
- License: Apache-2.0 (per Mesa org; LICENSE file present — verify per-example)
- Language: Python
- Activity/maintenance: ~253 stars, ~334 commits, branches for mesa-2.x/3.x/main. Active.
- Architecture: separate example folders. Relevant ones are the **Axelrod Culture Model**, **Emperor's Dilemma** (enforcement of unpopular norms, after Centola/Willer/Macy), the **Continuous Double Auction** market, Bank Reserves, El Farol, Boltzmann-on-network, and the Humanitarian Aid distribution model.
- Concept modeled: culture dissemination, false enforcement of norms, markets.
- Computational cost: small toy models.
- What we can learn: the Emperor's Dilemma is the key insight for "taboos nobody privately believes". Agents with weak conviction comply with a norm, and some also *enforce* it to signal sincerity, so the norm can persist with minority private support. That gives us emergent hypocrisy and witch-hunts for free.
- What we could integrate: algorithms (all small); the code is permissively licensed.
- What we should NOT integrate: nothing harmful. It is just not production code.
- Scalability: toy.

### NetLogo
- URL: https://github.com/NetLogo/NetLogo
- License: GPL-2.0 (the Models Library has per-model licenses, often CC BY-NC-SA)
- Language: Scala/Java
- Activity/maintenance: ~1.2k stars, ~8.5k commits. Active.
- Architecture: turtles/patches/links world, a tick-based `go` loop, BehaviorSpace for parameter sweeps.
- Concept modeled: everything. Its Models Library and the Modeling Commons host canonical social models (Sugarscape, Kiyotaki–Wright money by Pedro Romero — modelingcommons.org model 4741, **unverified**, the host was not fetched).
- Computational cost: JVM, single-threaded per run; ~10^4 agents.
- What we can learn: the canonical reference implementations to sanity-check our own ports.
- What we could integrate: **nothing as code** (GPL, and the library models are often non-commercial). Read and re-derive.
- What we should NOT integrate: any NetLogo model source into our codebase.
- Scalability: low.

### Repast4Py (and Repast HPC)
- URL: https://github.com/Repast/repast4py
- License: BSD-3-Clause
- Language: Python + C++ core (MPI), Numba/NumPy/PyTorch
- Activity/maintenance: ~75 stars, ~551 commits, CI on master/develop. Maintained by Argonne.
- Architecture: distributed ABM. Each MPI rank owns a spatial partition. Agents near boundaries are mirrored as **ghost agents**, and shared projections (grid/continuous/network) are synchronized each tick. Agents migrate between ranks.
- Concept modeled: large-scale distributed ABM (epidemics, city-scale).
- Computational cost: scales roughly linearly with ranks if interactions are local.
- What we can learn: the ghost-agent and boundary-synchronization pattern is what we need if we ever shard a 1M-organism world across threads or servers. Spatial partitioning by region also maps well onto our LOD "regions".
- What we could integrate: patterns only. MPI is not a game-runtime fit.
- What we should NOT integrate: MPI dependency in a consumer game.
- Scalability: high (HPC).

### MASON
- URL: https://github.com/eclab/mason
- License: Academic Free License 3.0 (most files; some files BSD/Artistic/Sun licenses)
- Language: Java
- Activity/maintenance: ~191 stars, ~1.67k commits. Maintained at GMU.
- Architecture: strict separation of model and visualization. A `Schedule` of `Steppable`s ordered by time and ordering; sparse/dense grids, continuous 2D/3D fields, networks; checkpointing (serialize the whole model mid-run). Distributed MASON exists as a contrib.
- Concept modeled: general ABM; the original GeoSim-style and many swarm/social models were built on it.
- Computational cost: fast for Java; 10^5–10^6 simple agents possible.
- What we can learn: **checkpointing a running model and detaching visualization** is precisely what a 1,000-year fast-forward needs. Run headless at max speed, then attach a view.
- What we could integrate: AFL-3.0 is permissive-ish (attribution; includes a patent-termination clause). Patterns only in practice, since we will not ship Java.
- What we should NOT integrate: the JVM runtime.
- Scalability: medium-high.

### GAMA Platform
- URL: https://github.com/gama-platform/gama
- License: GPL-3.0
- Language: Java (GAML DSL)
- Activity/maintenance: ~118 stars, ~2.7k commits. Active (JDK 21).
- Architecture: species-based modeling language (GAML), GIS-native, **BDI architecture** extension (beliefs/desires/intentions with emotions and social relations, the "BEN" architecture), multi-level agents (an agent can contain micro-agents, which is useful for LOD).
- Concept modeled: spatial social simulation, cognition.
- Computational cost: moderate.
- What we can learn: GAMA's multi-level "capture/release" of agents into macro-agents (e.g., individuals captured into a "crowd" agent and released when needed) is a working precedent for our **LOD aggregation of organisms into population cohorts**. The BEN social-relation model (liking, dominance, solidarity, familiarity, trust) is a good relationship vector template.
- What we could integrate: ideas only (GPL).
- What we should NOT integrate: code.
- Scalability: medium.

### Agents.jl
- URL: https://github.com/JuliaDynamics/Agents.jl
- License: MIT
- Language: Julia
- Activity/maintenance: ~916 stars, ~2k commits. Active.
- Architecture: agents as structs, spaces (grid, continuous, graph, OpenStreetMap), both time-stepped and event-queue scheduling. Published benchmarks show it is several times faster than Mesa/NetLogo/MASON for standard models.
- Concept modeled: general ABM.
- Computational cost: near-native.
- What we can learn: an **event-queue (continuous-time) mode** next to the tick mode. For a 1,000-year fast-forward, rare social events (marriage, schism, war) are better handled as scheduled events than per-tick polling.
- What we could integrate: MIT, so code is legal, but we will not embed Julia. Algorithms only.
- What we should NOT integrate: a Julia runtime in a game.
- Scalability: medium-high.

### FLAME GPU 2
- URL: https://github.com/FLAMEGPU/FLAMEGPU2
- License: **AGPL-3.0** for open source, plus a separate commercial license
- Language: C++/CUDA with Python bindings
- Activity/maintenance: ~157 stars, ~1.1k commits, release-candidate status, active.
- Architecture: agents are state machines whose variables are stored as struct-of-arrays on the GPU. **Agent functions** read typed **message lists** (brute-force, spatial 2D/3D, bucket, array, graph) and run in **layers** defined by a dependency graph. Agent birth/death happens via output buffers.
- Concept modeled: massive ABM (10^6–10^8 agents).
- Computational cost: very low per agent on GPU.
- What we can learn: the message-specialization idea. Spatial messages for local cultural transmission and bucket messages for "everyone in village k" are how to run culture and gossip for 10^6 agents. The layer dependency graph is also a clean way to order our social systems.
- What we could integrate: **not the code** unless we buy the commercial license (AGPL). Re-implement the pattern in compute shaders.
- What we should NOT integrate: AGPL code in a closed game.
- Scalability: very high.

### krABMaga
- URL: https://github.com/krABMaga/krABMaga
- License: MIT
- Language: Rust
- Activity/maintenance: ~224 stars, ~493 commits (ISISLab, Salerno). Moderate activity.
- Architecture: MASON-inspired. A `State` trait, agents implementing an `Agent` trait, a `Schedule`, and `Field`s (dense/sparse grids, networks) with double-buffered read/write. Optional Bevy-based visualization, WASM target, parallel scheduling, MPI distribution, and parameter-sweep/Bayesian-optimization helpers.
- Concept modeled: general ABM.
- Computational cost: native Rust; parallel.
- What we can learn: if the game is in Rust/Bevy, this is the closest existing architecture. Double-buffered fields avoid order effects (everyone reads last tick's culture and writes next tick's).
- What we could integrate: MIT, so code is OK. Possibly fork bits (fields, schedule) if Rust.
- What we should NOT integrate: its visualization layer, since we will have our own renderer.
- Scalability: medium-high.

### Melodie
- URL: https://github.com/ABM4ALL/Melodie
- License: unverified (BSD-3 per JOSS paper, recalled; not confirmed on page)
- Language: Python (Cython-accelerated)
- Activity/maintenance: v1.1.1 released 2026-03-28 per PyPI listing in search; small project.
- Architecture: Model/Scenario/Environment/DataCollector split, plus a **Calibrator** (fits parameters to data) and a **Trainer** (evolutionary training of agent behaviour parameters).
- Concept modeled: general ABM with calibration.
- Computational cost: Python with Cython hot paths.
- What we can learn: an evolutionary "Trainer" is a cheap way to tune our cultural-transmission constants so that 1,000-year runs produce plausible distributions (number of religions, Gini, war frequency).
- What we could integrate: the approach (offline tuning harness).
- What we should NOT integrate: the runtime.
- Scalability: low-medium.

### AgentPy
- URL: https://github.com/jofmi/agentpy
- License: BSD-3-Clause
- Language: Python
- Activity/maintenance: ~387 stars; **no longer actively developed** (README recommends Mesa).
- Architecture: Model/AgentList/Grid/Network with built-in experiments and SALib sensitivity analysis.
- Concept modeled: general ABM.
- What we can learn: integrated sensitivity analysis (Sobol) is worth copying into our offline tuning harness.
- What we could integrate: nothing needed.
- What we should NOT integrate: unmaintained dependency.
- Scalability: low.

### AgentTorch (Large Population Models)
- URL: https://github.com/AgentTorch/AgentTorch
- License: **AGPL-3.0**
- Language: Python (PyTorch)
- Activity/maintenance: ~651 stars, ~589 commits, v0.6.0 on PyPI. Active (MIT Media Lab origin).
- Architecture: agents are **tensorized** (one row per agent, columns are properties). Substeps are differentiable functions over tensors. **LLM "archetypes"**: instead of calling an LLM per agent, it clusters agents into a small number of archetypes, queries the LLM once per archetype per decision, and broadcasts the result. Simulated millions of agents (e.g., NYC-scale).
- Concept modeled: population-scale behaviour with LLM guidance.
- Computational cost: GPU tensor ops; LLM calls scale with #archetypes, not #agents.
- What we can learn: the **archetype trick is the right LOD strategy for LLM-driven cognition** in our game. Only "named"/salient individuals (prophets, kings) get individual LLM calls. Everyone else inherits decisions from their cohort archetype.
- What we could integrate: the idea only (AGPL).
- What we should NOT integrate: code.
- Scalability: very high.

---

## Part B — Culture & cultural evolution

### Axelrod Culture Model (Mesa example + Rust reimplementation)
- URL: https://github.com/akitenkrad/axelrod1997 (also the "Axelrod Culture Model" in projectmesa/mesa-examples)
- License: MIT (akitenkrad); Apache-2.0 (mesa-examples)
- Language: Rust simulation + Python plots (akitenkrad); Python (Mesa)
- Activity/maintenance: akitenkrad ~0 stars, 17 commits (small reproduction); mesa-examples is active.
- Architecture: each site has a culture vector of F features, each taking one of q traits. Each step: pick a random site and a random neighbour; similarity s = (#equal features)/F; with probability s they interact, and the active site copies one randomly chosen *differing* feature from the neighbour. If s=0 or s=1 nothing happens. Absorbing states are stable cultural regions with zero overlap at their borders.
- Concept modeled: homophily + social influence → local convergence and global polarization.
- Computational cost: O(1) per interaction.
- What we can learn: the **"similarity gates interaction"** rule is the single most important primitive for us. Groups that share nothing stop talking, which is why distinct cultures persist side by side. Known extensions: add noise (cultural drift) or else everything freezes; with large q, many small cultures form.
- What we could integrate: the algorithm (trivial to re-implement).
- What we should NOT integrate: a pure lattice. Our agents interact over a social network plus spatial proximity.
- Scalability: excellent; it vectorizes and runs on the GPU.

### PrestigeBias (Egozi & Ram 2024)
- URL: https://github.com/yoavram-lab/PrestigeBias
- License: **CC-BY-SA-4.0 for source code** (share-alike, so do not copy code)
- Language: Python (Jupyter)
- Activity/maintenance: ~0 stars, 157 commits; paper in Royal Society Open Science 2024.
- Architecture: Wright-Fisher-style cultural generations. Each learner chooses a model with probability proportional to that model's prestige. Prestige here is the number of times the model was copied in the previous generation, so it is a self-reinforcing popularity signal. Compared with conformity bias and success bias.
- Concept modeled: prestige-biased transmission.
- Computational cost: O(N) per generation.
- What we can learn: prestige defined as "how often you were copied" (not success) produces runaway cults of personality and rapid fixation of arbitrary traits. That is useful for prophets, fashion and royal styles. The paper's conclusion is that prestige bias can spread *maladaptive* traits. Good for drama.
- What we could integrate: the formula (re-derive from the paper).
- What we should NOT integrate: code (CC-BY-SA).
- Scalability: excellent.

### Boyd & Richerson transmission biases (theory reference, no single repo)
- URL: literature (Boyd & Richerson 1985 *Culture and the Evolutionary Process*; Henrich & Boyd 1998 "The Evolution of Conformist Transmission", Evolution and Human Behavior 19:215–241)
- License: n/a
- Language: n/a
- Activity/maintenance: canonical theory.
- Architecture (from literature): with trait frequency p, **conformist bias** gives Δp = D·p(1−p)(2p−1) (the majority is over-adopted; D is conformity strength). **Directly biased (content) transmission** gives Δp = B·p(1−p) (an intrinsically attractive trait spreads). **Guided variation** means individual learning nudges traits toward a locally optimal value. **Prestige/success bias** means copying the high-payoff or high-status individual. Conformity is favoured when the environment is spatially variable and individual learning is costly.
- Concept modeled: cultural evolution forces.
- Computational cost: trivial.
- What we can learn: these four forces plus mutation (innovation) and drift are a **complete minimal toolkit**. Conformity keeps groups internally homogeneous, which group-level selection (war, migration) then acts on (Turchin's cultural multilevel selection).
- What we could integrate: the equations (population-level LOD) and their agent-level equivalents.
- What we should NOT integrate: n/a.
- Scalability: the population-level equation is O(#traits) per group, which is ideal for far-LOD regions.

### Sugarscape (Epstein & Axtell 1996) — Python implementation
- URL: https://github.com/nkremerh/sugarscape
- License: Unlicense (public domain)
- Language: Python
- Activity/maintenance: ~22 stars, ~453 commits, with a test suite and 20+ example configs. Active research codebase.
- Architecture: grid of sugar/spice with growback rules; agents have vision, metabolism, a lifespan and a **culture tag** (bit string). Culture rule: for each neighbour, pick a random tag position and set the neighbour's bit to match yours. Tribe = majority bit value. Trade rule: MRS = (w_spice/m_spice)/(w_sugar/m_sugar); neighbours trade if their MRS differ; bilateral price = geometric mean √(MRS_A·MRS_B); trade unit by unit while both welfare functions improve. Combat, credit, disease (immune-system bit strings), inheritance to children, pollution, seasons. This fork adds ethical decision models (altruist, egoist, utilitarian…).
- Concept modeled: an entire artificial society from simple rules: inequality, migration, trade, tribes, war, disease.
- Computational cost: O(N·vision) per tick.
- What we can learn: Sugarscape is still the best proof that **inheritance + finite lifespans + local resource gradients give a Pareto-like wealth distribution**, and that **tag-flipping culture + combat** gives tribal wars without coding "war". Its inheritance rule (wealth split among children) is a direct template for our death→inheritance pipeline.
- What we could integrate: code legally (Unlicense), but we would rewrite for performance anyway. The trade/MRS formula is especially valuable.
- What we should NOT integrate: the grid-only world; the 1990s rule granularity.
- Scalability: medium on CPU; trivially parallel per cell.

### Emperor's Dilemma / false enforcement (norm emergence)
- URL: https://github.com/projectmesa/mesa-examples (Emperor's Dilemma folder)
- License: Apache-2.0
- Language: Python
- Activity/maintenance: active repo.
- Architecture (Centola, Willer & Macy 2005, from literature): agents are believers or disbelievers with conviction strength. Compliance happens when neighbours' enforcement pressure exceeds an agent's own conviction. Enforcement of the norm (sanctioning deviants) happens when pressure exceeds conviction plus a cost; insincere agents enforce to *prove* sincerity. Network clustering lets a small cluster of true believers cascade the norm.
- Concept modeled: unpopular norms, hypocrisy, puritan cascades.
- Computational cost: O(edges).
- What we can learn: taboos and dress codes can be **self-sustaining with no believers at the center**. This is a great emergent source of religious zeal and of later "reformation" collapses when pressure drops.
- What we could integrate: the algorithm.
- What we should NOT integrate: n/a.
- Scalability: good.

### Axelrod Norms/Metanorms game (1986) — theory reference
- URL: literature (Axelrod 1986 "An Evolutionary Approach to Norms", APSR); a generalization at https://coin-workshop.github.io/coine-2023-london/Papers/Paper-5.pdf (not fetched)
- License: n/a
- Language: n/a
- Activity/maintenance: classic; many re-implementations, no canonical repo found (**unverified**).
- Architecture: agents have boldness b and vengefulness v (0..7/7). An agent defects when the chance of being seen is below b. Each observer of a defection punishes with probability v (punishing costs the punisher). In the **metanorms** variant, agents also punish those who *failed to punish*. Strategies evolve by fitness. Result: plain norms collapse, while metanorms stabilize them.
- Concept modeled: norm establishment through second-order punishment.
- What we can learn: our justice system needs **second-order sanctions** (shaming people who tolerate crime) or norms decay. This is the bridge from gossip to courts.
- What we could integrate: the algorithm.
- Scalability: O(observers per event).

### Multilevel altruistic punishment (Cooney et al.)
- URL: https://github.com/dbcooney/Multilevel-Altruistic-Punishment-Paper-Code
- License: BSD-3-Clause
- Language: Python
- Activity/maintenance: ~0 stars, 57 commits; accompanies the Bulletin of Mathematical Biology 2025 paper.
- Architecture: PDE (finite-volume) model of groups containing cooperators, defectors and altruistic punishers. Within-group selection favours defectors; between-group competition (Fermi, local, or Tullock contest functions) favours cooperative groups. Builds on Boyd, Gintis, Bowles & Richerson 2003 "The evolution of altruistic punishment" (PNAS).
- Concept modeled: why costly punishment survives: group competition.
- Computational cost: cheap (PDE on a grid of group compositions).
- What we can learn: a **population-level (far-LOD) representation**: track each group as a composition point and let inter-group conflict reshape the distribution. The Tullock contest function P(win) = S_a^γ/(S_a^γ+S_b^γ) is directly reusable in our war resolution.
- What we could integrate: code legally (BSD-3); in practice we just take the equations.
- What we should NOT integrate: n/a.
- Scalability: excellent.

### Crusader Kings III culture system (design reference)
- URL: https://www.pcgamer.com/crusader-kings-3-ck3-hybrid-culture/ (+ CK3 wiki, not fetched)
- License: proprietary game (reference only)
- Language: n/a
- Activity/maintenance: commercial, live.
- Architecture: a culture = **pillars** (Ethos ×1 of 7, Heritage, Language, Martial custom, Aesthetics) + up to ~5 **traditions** (each a modifier package with prerequisites and unique events) + innovations (tech). Inter-culture **acceptance** (0–100) rises with coexistence under the same ruler. A culture head can **hybridize** two cultures (needs acceptance and a different heritage) or **diverge** (change at least one pillar; costs prestige).
- Concept modeled: designed (not emergent) culture change.
- What we can learn: a clean **schema for a culture's "identity" layer** that players can read. Our emergent cultural traits can be *summarized* into pillar-like categories for UI. The acceptance meter between groups is a good, cheap inter-group relationship scalar.
- What we could integrate: UI/summary taxonomy ideas only.
- What we should NOT integrate: the top-down "culture head decides" mechanism. In our game hybridization and divergence must come from transmission statistics (trait distance thresholds), not a button.
- Scalability: n/a.

### Ultima Ratio Regum (Mark R. Johnson) — design reference
- URL: https://www.gamedeveloper.com/design/the-10-year-journey-of-ultima-ratio-regum-the-culture-generating-roguelike ; AISB 2015 paper "Modelling Cultural, Religious and Political Affiliation in…" (host blocked)
- License: proprietary/freeware game, not OSS
- Language: Python (game)
- Activity/maintenance: in development since ~2011; 0.11 updates through 2024 (religion, prayers, masks, relics, riddles).
- Architecture (from interviews/blog headlines; details not re-read): generation is driven by **"chains of meaning"**. No generated element may relate only to itself. A religion's colours, sacred animal, holy sites, relics, prayers, masks, clothing and architecture all reference each other and the culture's other systems. Players can formally join a religion by performing its rituals.
- Concept modeled: dense, cross-referenced procedural culture (generated, not simulated).
- What we can learn: the "chain of meaning" requirement is a **quality test for our emergent culture**. Every ritual should point back to a remembered event, a place, an object or a person. That is exactly what our event-grounded religion pipeline produces naturally (lightning → mountain → stone relic → storm-season festival).
- What we could integrate: design principle.
- What we should NOT integrate: authored up-front generation. We simulate.

---

## Part C — Religion

### Divergent Modes of Religiosity (Whitehouse) — Kivung simulations (Lane, Whitehouse et al.)
- URL: literature: Whitehouse et al. 2012 "The role for simulations in theory construction for the social sciences: case studies concerning Divergent Modes of Religiosity" (Religion, Brain & Behavior); Lane 2015 "Semantic network mapping of religious material…" (Cognitive Processing, https://link.springer.com/article/10.1007/s10339-015-0649-1). No public code found (**unverified**).
- License: n/a
- Language: multi-agent AI framework (unspecified)
- Activity/maintenance: research (2012–2015); continued in Lane's *Understanding Religion Through Artificial Intelligence* (2021).
- Architecture (from literature): agents hold religious motivation; **doctrinal mode** means frequent low-arousal rituals, which give efficient transmission of a large creed but accumulate **tedium**. When tedium crosses a threshold in a sub-community, a **splinter group** forms that performs rare, high-arousal (**imagistic**) rituals, giving intense local cohesion and episodic memory. Its arousal fades over time and members **re-assimilate** into the mainstream. Lane's work adds cognitive (semantic-network) representations of doctrine.
- Concept modeled: ritual frequency/arousal → group size, cohesion and schism cycles.
- Computational cost: low.
- What we can learn: this is the best-grounded **schism mechanism** for us. Schisms are not only doctrinal disagreements; they are driven by a **tedium-vs-arousal** cycle. Also: imagistic rituals (rare, painful, terrifying, like a lightning-strike rite) make small, fiercely loyal groups, while doctrinal rituals (weekly, routine) scale to states but need priests and texts.
- What we could integrate: the variables (tedium, arousal, episodic memory of a rite, cohesion) and the threshold rule.
- What we should NOT integrate: n/a.
- Scalability: high (group-level variables).

### Modeling Religion Project (Shults, Wildman, Gore, Lane, Diallo et al., CMAC)
- URL: https://mindandculture.org/projects/past-projects/modeling-religion-project/ ; e.g. Gore, Lemos, Shults et al. 2018 "Forecasting Changes in Religiosity and Existential Security with an Agent-Based Model" (JASSS 21(1)4)
- License: n/a (the only CMAC code repo I found, centerformindandculture/TAOPlusV, is MIT but is an epidemic model, not religion)
- Language: various (AnyLogic/Python/Java in papers)
- Activity/maintenance: project listed as "past".
- Architecture (from literature): agents with religiosity driven by **existential security** (health, wealth, safety). Threat/mortality salience (terror management theory) raises religiosity; secure, wealthy environments lower it. Social networks propagate religiosity. Models of Çatalhöyük studied "religious entanglement" in a Neolithic town.
- Concept modeled: secularization vs. revival as a function of threat and security.
- Computational cost: low.
- What we can learn: a **direct coupling from our god's interventions to religiosity**. Disasters, plagues and lightning raise threat → religiosity ↑; long peace and plenty → religiosity ↓ (secularization). This lets the player *feel* the effect of neglect versus terror.
- What we could integrate: the coupling (religiosity_dot = a·threat − b·security + social-conformity term).
- What we should NOT integrate: n/a.
- Scalability: high.

### Agency detection / HADD vs predictive-coding ABM (r-andre, Zenodo 3552174)
- URL: https://zenodo.org/records/3552174 (host blocked); the claimed repo `r-andre/abm-religious_beliefs` returned **404** (renamed/removed — **unverified**). The same author has https://github.com/r-andre/abm-social_complexity (GPL-3.0, NetLogo), reviewed below.
- License: unknown
- Language: Python
- Activity/maintenance: thesis-scale, ~2019.
- Architecture (from abstract): agents perceive ambiguous environmental stimuli and either attribute them to an agent (HADD: biased toward false positives because missing a predator is costlier than a false alarm) or update a predictive model (predictive coding: attribute agency when prediction error is high and unexplained). Beliefs spread socially.
- Concept modeled: origin of supernatural-agent beliefs from perception.
- What we can learn: the **two-hypothesis framing is exactly our pipeline's first step**. An event that is (a) salient, (b) poorly predicted by the agent's world model, and (c) consequential triggers a hypothesis of agency. The asymmetric-cost argument justifies a deliberately low threshold.
- What we could integrate: the concept; code unavailable.
- What we should NOT integrate: n/a.
- Scalability: high.

### Azgaar's Fantasy Map Generator (religion/culture/state generator)
- URL: https://github.com/Azgaar/Fantasy-Map-Generator
- License: MIT
- Language: JavaScript (migrating to TypeScript)
- Activity/maintenance: ~6k stars, ~1k forks, ~2.5k commits. Active.
- Architecture: a Voronoi cell map. Cultures expand from centres by **cost-based flood fill** (Dijkstra over cells, with biome, elevation, river and sea costs scaled by the culture's type: nomadic, highland, naval…). States expand similarly from capitals. Religions come in four groups: **Folk** (one per culture, spreads with it), **Organized** (crosses cultures/states), **Cult** (small, local), **Heresy** (split from an organized religion). Forms include Shamanism, Animism, Ancestor worship, Polytheism, Dualism, Monotheism, Non-theism. A supreme deity name is built from patterns (Being, Adjective+Animal, Colour+Genitive, Number…). Each religion has an "expansionism" multiplier and a scope (culture/state/global).
- Concept modeled: static generation of plausible religious geography.
- Computational cost: seconds for ~10k cells.
- What we can learn: the **Folk → Organized → Cult/Heresy typology** is a good *descriptor* vocabulary for what our simulation produces. The cost-based flood fill is also a cheap far-LOD approximation of how a faith spreads along easy terrain and trade routes.
- What we could integrate: MIT, so code is allowed, and its name/deity grammars are reusable for naming. The religion typology can serve as labels assigned *after* emergence.
- What we should NOT integrate: the generate-up-front approach for the live sim (we must not hard-code beliefs).
- Scalability: high.

### go_gens / genreligion (flokey82)
- URL: https://github.com/flokey82/go_gens (package `genreligion`, pkg.go.dev listing)
- License: Apache-2.0
- Language: Go
- Activity/maintenance: ~31 stars, ~463 commits; author warns "DO NOT USE YET" (experimental).
- Architecture: Group → Form → Type hierarchy (Folk/Organized/Cult/Heresy; Shamanism…Non-theism; "Church", "Pantheon", "Sect"…). A deity struct holds name, meaning and generation method, with word lists (beings, animals, adjectives, colours, genitives). Other packages: genvillage (self-sustaining village economy), gendemographics, genlanguage, genempire.
- Concept modeled: naming and labelling of religions; toy village economy.
- What we can learn: a compact taxonomy for **labelling emergent religions** and generating deity epithets from salient attributes (e.g., "the Thundering One of the Peak").
- What we could integrate: Apache-2.0, so code is allowed; but it is unstable, so take the ideas.
- What we should NOT integrate: random generation of beliefs disconnected from events.
- Scalability: n/a.

---

## Part D — Social relationships & social AI

### Ensemble (successor of Comme il Faut / Prom Week)
- URL: https://github.com/ensemble-engine/ensemble
- License: **BSD-4-Clause** (advertising clause — caution)
- Language: JavaScript
- Activity/maintenance: ~64 stars, ~351 commits; academic (UCSC/Expressive Intelligence lineage); low recent activity.
- Architecture: a **schema** declares predicate categories: traits (boolean, per character), relationships (boolean, between pairs, possibly directed), network values (numeric 0–100, directed, e.g. affinity/trust/attraction), statuses (boolean with duration, e.g. "embarrassed for 3 turns"). The **social record** stores all predicate values per timestep, so rules can reference history ("A was insulted by B within the last 5 turns"). **Trigger rules** fire state changes when conditions hold. **Volition rules** are condition→weighted-intent pairs: for each ordered pair (A,B), sum the weights of all matching rules into volitions such as "increase affinity", "start dating" or "antagonize". **Actions** are chosen by volition strength, have preconditions and their own influence rules, and resolve to accept/reject. Comes with an authoring tool.
- Concept modeled: rich, history-aware, authored social physics for small casts (~10–20 characters).
- Computational cost: O(characters² × rules) per decision, which is fine for tens and too heavy for thousands.
- What we can learn: the **predicate taxonomy (trait / binary relationship / numeric directed network / timed status)** and the **volition = sum of weighted matching rules** pattern are the right shape for our near-LOD social AI. The social-record-with-history makes grudges and gratitude natural.
- What we could integrate: the design. Code reuse is risky due to BSD-4 advertising, so re-implement.
- What we should NOT integrate: authored per-scenario rule libraries at our scale; the O(n²) all-pairs volition loop (restrict to each agent's ~150 known others).
- Scalability: low per se; medium with neighbourhood restriction.

### Comme il Faut / Prom Week (McCoy et al.) — paper reference
- URL: https://promweek.soe.ucsc.edu/2012/02/22/prom-weeks-social-exchanges/ ; "Prom Week: Social Physics as Gameplay" (FDG 2011)
- License: n/a (no public source found)
- Language: ActionScript (original)
- Activity/maintenance: historical (2012).
- Architecture: ~5,000 social-consideration rules; **social exchanges** (pick-up line, backstab, …) have an intent, preconditions, an "influence rule set" that scores the responder's acceptance, and outcome-specific effects. The **Cultural Knowledge Base (CKB)** gives each character opinions about objects/topics. **Social Facts DB** records past exchanges.
- Concept modeled: social physics with a cultural knowledge base.
- What we can learn: the **CKB idea maps onto our culture traits**. Characters' likes and dislikes of cultural items (a song, a god, a dish) feed into relationship formation. Shared CKB opinions mean homophily.
- What we could integrate: design.
- Scalability: low.

### CiF-CK (Guimarães, Santos, Jhala 2017)
- URL: https://visualnarrative.ncsu.edu/wp-content/uploads/sites/16/2017/08/GSJ_IEEECIG_2017_CIFCK.pdf (paper; a Skyrim mod "Social NPCs")
- License: mod distribution; no OSS license found (**unverified**)
- Language: Papyrus/C++ (Skyrim Creation Kit)
- Activity/maintenance: 2017 paper; follow-up "Emergent social NPC interactions in the Social NPCs Skyrim mod and beyond".
- Architecture: CiF ported into a commercial engine. NPCs track interactions with the player *and with each other*, keep persistent social state, and choose social exchanges using CiF-style volitions. It adapts the third-person "director" model of Prom Week to first-person play.
- Concept modeled: proof that CiF-style social AI runs inside a shipped game's update budget.
- What we can learn: they had to cap exchanges per frame and restrict to nearby NPCs. That is the same budget pressure we will face; an **opportunistic, proximity-triggered social update** is the practical answer.
- What we could integrate: design only.
- Scalability: ~dozens of NPCs.

### Versu / Praxis / Exclusion Logic (Evans & Short)
- URL: https://versu.com/about/how-versu-works/ ; https://www.cs.uky.edu/~sgware/reading/papers/evans2014versu.pdf ("Versu — A Simulationist Storytelling System", IEEE TCIAIG 2014)
- License: proprietary (Versu shut down); papers public
- Language: Praxis (custom logic DSL)
- Activity/maintenance: historical (2013–2014).
- Architecture: state is a tree in **exclusion logic**. `A.relationship.B!enemy` means the `!` slot holds exactly one value, so asserting `!friend` automatically retracts `!enemy`. This makes updates and norm conflicts cheap. **Social practices** (a dinner, a courtship, a funeral) are reactive joint plans that offer **affordances** to participants. Agents choose among affordances by utility over their desires. Norms are **deontic** statements inside practices (one *should* thank the host).
- Concept modeled: social practices and norms as first-class data.
- What we can learn: **social practices as data objects** is the right abstraction for our *rituals and customs*. A funeral, a harvest festival or the lightning-mountain rite is a practice object with roles, steps, affordances and norms. Practices can be invented (by copying and mutating existing ones), transmitted and abandoned like any cultural trait. Exclusion logic is a good compact encoding for single-valued relationship states.
- What we could integrate: the concepts (practice = roles + stages + affordances + norms).
- What we should NOT integrate: hand-authored practice libraries as the only source. We need a generative grammar of practices.
- Scalability: medium.

### Axelrod-Python (Iterated Prisoner's Dilemma library)
- URL: https://github.com/Axelrod-Python/Axelrod
- License: MIT
- Language: Python
- Activity/maintenance: ~852 stars, ~5.6k commits; 100% test coverage.
- Architecture: 200+ IPD strategies (Tit-for-Tat, Win-Stay-Lose-Shift, generous variants, finite-state machines, evolved neural strategies); round-robin tournaments, **Moran processes** (population dynamics with strategy replacement), noise, spatial tournaments, fingerprinting.
- Concept modeled: reciprocity and cooperation.
- What we can learn: a cheap **reciprocity layer** in our relationship model. Each dyad keeps a short memory of the other's last cooperation/defection, and WSLS/generous-TFT-like policies give trust that can recover from noise. The Moran process is a ready-made model for strategy (norm) turnover.
- What we could integrate: MIT. We could use it offline to evaluate which reciprocity policies are robust under our noise levels.
- What we should NOT integrate: as a runtime dependency.
- Scalability: fine offline.

### NetworkX (social network generators & analysis)
- URL: https://github.com/networkx/networkx
- License: BSD-3-Clause
- Language: Python
- Activity/maintenance: ~17.3k stars. Very active.
- Architecture: graph data structures and algorithms. Relevant generators: `powerlaw_cluster_graph` (Holme–Kim: preferential attachment plus a **triadic-closure** step with probability p), Watts–Strogatz, stochastic block models (homophily between communities), community detection (Louvain, label propagation), centrality measures.
- Concept modeled: network formation and analysis.
- What we can learn: our **offline validation** of emergent social networks. Do they have realistic clustering (~0.1–0.5), heavy-tailed degree and community structure? Louvain on the social graph is also the cheapest **"group detection"** pass: communities that persist become candidate groups.
- What we could integrate: BSD-3, so code is allowed for tools. Re-implement label propagation natively for runtime group detection.
- What we should NOT integrate: Python in the runtime.
- Scalability: 10^5–10^6 edges fine offline.

### Dominance hierarchy: DomWorld (Hemelrijk) — winner/loser effect
- URL: literature: Hemelrijk et al.; https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0243877 ("Hierarchical development of dominance through the winner-loser effect and socio-spatial structure", PLOS ONE 2020)
- License: n/a (no public repo found — **unverified**)
- Language: n/a
- Activity/maintenance: long-running research line (1999–2020s).
- Architecture: each agent has a dominance value DOM. When two meet, i wins if DOM_i/(DOM_i+DOM_j) > U(0,1). Update: DOM_i += StepDom·(w_i − DOM_i/(DOM_i+DOM_j)) and DOM_j −= the same amount, where w_i ∈ {0,1}. StepDom is aggression intensity. Spatial rules: losers flee, winners chase; agents try to stay near the group. Measured with the Average Dominance Index.
- Concept modeled: emergence of linear dominance hierarchies, and of spatial centrality of dominants, from self-reinforcing contests.
- Computational cost: O(1) per contest.
- What we can learn: **hierarchy for free**. With high StepDom (aggressive species/cultures) we get steep despotic hierarchies; with low StepDom we get egalitarian ones. This is our pre-political "status" substrate inside bands.
- What we could integrate: the formula (one line).
- What we should NOT integrate: n/a.
- Scalability: excellent.

---

## Part E — Group formation, hierarchy & politics

### Mesa Alliance Formation (meta-agents)
- URL: https://github.com/projectmesa/mesa/tree/main/mesa/examples/advanced/alliance_formation
- License: Apache-2.0
- Language: Python
- Activity/maintenance: part of Mesa core examples (2025+).
- Architecture: agents with power and preference ∈ [0,1]. Random partner search; the alliance forms if the **bilateral Shapley value** shows both gain. The alliance becomes a **meta-agent** that can itself ally with others, giving nested hierarchy. A membership manager tracks overlapping/nested groups.
- Concept modeled: coalition formation into hierarchies.
- What we can learn: groups-as-agents with recursive nesting. Bands → tribes → confederations without a hard-coded ladder.
- What we could integrate: the rule; the code is Apache-2.0.
- What we should NOT integrate: the abstract power/preference scalars as our only inputs. Ours come from kinship, culture overlap and threat.
- Scalability: medium.

### Circumscription / social-complexity ABM (r-andre)
- URL: https://github.com/r-andre/abm-social_complexity
- License: **GPL-3.0**
- Language: NetLogo 6.0.4
- Activity/maintenance: ~0 stars, 10 commits (student replication).
- Architecture: groups grow logistically to carrying capacity, then split and seek new patches. When a group meets a patch held by a different ethnicity, war happens: the larger population subjugates and assimilates the smaller. Random disasters remove groups. The habitability fraction controls circumscription. Measures time to political unification.
- Concept modeled: Carneiro's circumscription theory (states arise where people cannot flee).
- What we can learn: an **"exit option" variable** is key. Where an oppressed or defeated group can migrate to free land, hierarchy stays shallow; where land is circumscribed (valleys, islands, deserts), conquest and subjugation stack into states. Our terrain gives us this for free if migration costs are modelled.
- What we could integrate: the idea only (GPL).
- What we should NOT integrate: code.
- Scalability: high.

### Turchin et al. 2013 — cultural multilevel selection / imperiogenesis
- URL: https://www.pnas.org/doi/10.1073/pnas.1308825110 ("War, space, and the evolution of Old World complex societies", PNAS 110(41):16384–9). Follow-up assessment: Bennett 2019 arXiv:1903.11729 (not fetched).
- License: paper; code was in the SI (**unverified** availability)
- Language: n/a
- Activity/maintenance: highly cited; Seshat databank validation.
- Architecture (from literature; PDF not re-read): Afro-Eurasia gridded at ~100×100 km cells, steps of 2 years from 1500 BCE to 1500 CE. Each cell carries a binary vector of ~10 **"ultrasocial traits"** (institutions that enable large-scale cooperation). Polities are sets of cells. Polities attack neighbours; the success probability rises with the attacker's polity size times the share of ultrasocial traits, and with **military technology**, which diffuses outward from the Eurasian steppe. Elevation helps defenders. Conquered cells adopt the conqueror's traits (selection). Traits mutate (gain rarely, lose more often). Large polities fragment with a probability that increases with size and decreases with ultrasociality (asabiya-like cohesion). The model explained ~65% of variance in where historical empires arose; without military tech, ~16%.
- Concept modeled: war as the selective force producing cooperative institutions and large states.
- Computational cost: cheap (cells × neighbours).
- What we can learn: the cleanest **"institutions are cultural traits under group selection"** mechanism. In our game, any institution (courts, taxes, priesthood, standing army) is a group-level trait that raises cohesion and fighting capacity but costs something. War and collapse then sort which institutions persist. Also the **steppe frontier effect**: metaethnic frontiers (cultural distance between neighbours) breed cohesion.
- What we could integrate: the algorithm structure, re-implemented.
- What we should NOT integrate: Earth-specific calibration.
- Scalability: excellent (cell-level).

### Structural-Demographic Theory (Goldstone, Turchin, Nefedov) — theory reference
- URL: https://en.wikipedia.org/wiki/Structural-demographic_theory ; Turchin & Nefedov *Secular Cycles* (2009); https://escholarship.org/content/qt6qp8x28p/qt6qp8x28p.pdf ("Modeling Social Pressures Toward Political Instability")
- License: n/a. No OSS implementation found (**unverified**).
- Language: n/a
- Architecture (from literature): stocks are population N, elite numbers E, state fiscal health S, and wages/well-being W. Growth in N depresses W (labor surplus) and raises rents, so elites grow (E ↑) and **elite overproduction** follows. Intra-elite competition rises, state revenue falls relative to obligations, and the **Political Stress Indicator** PSI ≈ MMP (mass mobilization potential) × EMP (elite mobilization potential) × SFD (state fiscal distress) climbs. That leads to crisis, civil war and population decline, followed by a new integrative phase. Cycles run ~200–300 years.
- Concept modeled: secular cycles of agrarian states.
- What we can learn: a **far-LOD dynamical model of a state** that automatically produces rise/stagflation/crisis/depression phases during a 1,000-year fast-forward. It also gives a natural "significance" signal: a PSI spike means a chronicle-worthy era.
- What we could integrate: ODE/difference equations (our own implementation).
- What we should NOT integrate: n/a.
- Scalability: trivial per state.

### Cederman's GeoSim / Emergent Polarity — theory reference
- URL: literature: Cederman 1997 *Emergent Actors in World Politics*; 2003 "Modeling the size of wars" (APSR); PNAS 2002 colloquium "Endogenizing geopolitical boundaries with agent-based modeling" (https://www.pnas.org/doi/pdf/10.1073/pnas.082081099). Code not found (**unverified**; historically Repast/Java).
- License: n/a
- Language: Java/Repast (historical)
- Architecture (from literature; PDFs not re-read): a grid of indivisible **provinces**. States are hierarchies of a capital plus provinces. Each state allocates resources across its active fronts in proportion to the threat on each front. A war front is won when the local power ratio exceeds a **superiority threshold** (plus noise). The conquered province is annexed. Capitals decide war and peace for all provinces. Collective security/alliances in variants. Starting from ~200 states, it consolidates to ~15, and war sizes follow a power law (Richardson's law).
- Concept modeled: emergent state system and war-size distributions.
- What we can learn: **front-based resource allocation** and a **threshold for conquest** are simple and generate realistic power-law war sizes. That is a good target statistic for our war system.
- What we could integrate: algorithm.
- What we should NOT integrate: n/a.
- Scalability: excellent.

### Epstein Civil Violence (Mesa example)
- URL: https://github.com/projectmesa/mesa (mesa/examples/advanced/epstein_civil_violence)
- License: Apache-2.0
- Language: Python
- Activity/maintenance: maintained with Mesa.
- Architecture (Epstein 2002): citizens have hardship H ~ U(0,1) and risk aversion R; regime legitimacy L. Grievance G = H·(1−L). Estimated arrest probability P = 1 − exp(−k·floor(C/A)) with C cops and A active rebels in vision. A citizen turns active if G − R·P > threshold T. Cops arrest random active neighbours; jail terms are random. It produces punctuated outbursts, and with legitimacy shocks, sudden revolutions.
- Concept modeled: rebellion, crime-as-defiance, repression.
- What we can learn: a ready **unrest equation** for our states and for crime. Legitimacy L can come from religion (a divinely sanctioned ruler), from victories, and from fairness of courts. The god's interventions can shift L, e.g. lightning striking the palace equals an omen of illegitimacy.
- What we could integrate: the formula.
- What we should NOT integrate: n/a.
- Scalability: high.

### "Being a leader or being the leader" (Perret, Hart, Powers) — institutionalised hierarchy
- URL: literature, arXiv:1907.01927 (arXiv blocked; from abstract)
- License: n/a
- Architecture (from literature): leaders and followers co-evolve; leadership starts as **situational** (whoever has information or initiative) and becomes **institutionalised** (a persistent office) when groups grow and coordination costs rise. An office lets followers pay a "tax" to leaders in exchange for organisation. Hierarchy appears when group size crosses a threshold at which consensus decision-making is too slow.
- Concept modeled: from ad-hoc leadership to offices.
- What we can learn: **group size × coordination cost → the creation of an office** is our rule for when "chief" becomes a role that outlives its holder (and therefore needs succession rules).
- What we could integrate: the condition.
- Scalability: trivial.

---

## Part F — Economics

### bazaarBot (Doran & Parberry "Emergent Economies for RPGs")
- URL: https://github.com/larsiusprime/bazaarBot
- License: MIT (Copyright 2013 Lars A. Doucet)
- Language: Haxe
- Activity/maintenance: ~393 stars, 73 commits; dormant but a well-known reference. Ports exist in C# and other languages (not verified individually).
- Architecture: agents have a **role** (farmer, woodcutter, miner, refiner, blacksmith) with production recipes, inventory with ideal levels, money, and for each commodity a **price-belief interval [low, high]**, initialised at 0.5–1.5 × the historical mean. Each round an agent produces, then for each commodity with surplus creates an **ask**, and for each shortfall a **bid**. The price is drawn uniformly from its belief interval. Quantity depends on **favorability** (where the historical mean sits in the observed price range): sell more when prices are historically high, buy more when low. **Clearing**: shuffle, then sort bids descending and asks ascending, match top bid to top ask, trade the min quantity at the **average of the two prices**, and repeat until one book is empty. **Belief update** (from paper/subclass): on success, narrow the interval toward the clearing price; on failure, widen it and shift it toward the market mean (sellers down, buyers up). **Bankrupt agents are replaced** by a new agent in the most profitable role (or the role whose good has the highest demand/supply ratio), so labor reallocates by itself.
- Concept modeled: price discovery and job reallocation without an auctioneer.
- Computational cost: O(n log n) per commodity per round for sorting offers.
- What we can learn: the **single best drop-in for village-level markets**. Prices emerge, shortages propagate up supply chains (no wood means tool prices spike), and profession choice responds. The bankruptcy/role-switch rule maps directly to "a son abandons farming and becomes a smith".
- What we could integrate: MIT, so code is allowed; a port is cheap. Use per market site (village square), not globally.
- What we should NOT integrate: global unlimited money supply (bazaarBot starts everyone with coins). Our money must **emerge** (see the design). Before money exists, run the same double-auction with a **barter numeraire** (see design).
- Scalability: one market per settlement; thousands of markets fine.

### Sugarscape bilateral trade (MRS / geometric-mean price)
- See the Sugarscape entry above (Part B). Key reusable piece: **decentralized bilateral barter** with private marginal rates of substitution. Price = √(MRS_A·MRS_B) per trade. This works with *no money and no market place*, which makes it the right pre-market barter mechanism for bands and tribes.

### Gintis — emergence of a price system from decentralized bilateral exchange (theory reference)
- URL: https://www.umass.edu/preferen/gintis/The%20Price%20System.pdf (B.E. J. Theoretical Economics 6(1), 2006); also "The Dynamics of Generalized Market Exchange" (https://www.umass.edu/preferen/gintis/emergenceofmoney.pdf)
- License: n/a (no public code found — **unverified**)
- Architecture (from abstract): no public prices exist; each agent holds **private prices** (its barter strategy). Agents meet in random pairs and accept or reject offers given their private prices. Periodically, **low-scoring agents imitate the private prices of high-scoring agents** (replicator dynamics). Result: private prices converge to a public price system near Walrasian equilibrium. In the follow-up, a commodity can become a universal medium of exchange (money).
- Concept modeled: emergence of prices (and money) via imitation.
- What we can learn: price convergence via **imitation of successful traders** is exactly cultural transmission (prestige/success bias) applied to prices. So our culture engine and our economy can share one transmission mechanism.
- What we could integrate: algorithm.
- Scalability: O(pairs).

### Kiyotaki–Wright commodity-money emergence (theory + NetLogo model)
- URL: https://en.wikipedia.org/wiki/Kiyotaki%E2%80%93Wright_model_of_money ; NetLogo Modeling Commons model 4741 by Pedro Romero (http://modelingcommons.org/browse/one_model/4741 — **unverified**, not fetched); Moran & Ianni "Money Networks in Kiyotaki-Wright Model" (https://www.southampton.ac.uk/~ianni/Money_Networks_2013JunePDF.pdf).
- License: NetLogo Commons models are often CC-BY-NC-SA (do not copy)
- Architecture (from literature): three agent types and three goods. Type i consumes good i and produces good i+1 (mod 3), so no double coincidence of wants exists. Each good has a **storage cost** c1 < c2 < c3. Agents meet randomly and decide whether to accept a good they do not consume as a medium of exchange. In the *fundamental* equilibrium the cheapest-to-store good becomes money. In the *speculative* equilibrium agents accept a costlier good because it is more marketable. ABM versions use reinforcement learning over acceptance rules, and the network topology changes which good becomes money.
- Concept modeled: **which commodity becomes money** emerges from storage cost × marketability × search frictions.
- What we can learn: the **conditions for money**: (1) specialization without double coincidence, (2) goods that differ in durability/storability/divisibility, (3) frequent enough trade. If any is missing, barter persists. This is our gating rule.
- What we could integrate: algorithm (an acceptance learner per agent: P(accept good g as a medium) updated by reinforcement on how quickly g was re-traded).
- What we should NOT integrate: NC-licensed model code.
- Scalability: high.

### ABIDES (Agent-Based Interactive Discrete Event Simulation)
- URL: https://github.com/abides-sim/abides (the jpmorganchase/abides URL 404s; JPMC has a separate abides-jpmc-public — **unverified**)
- License: BSD-3-Clause
- Language: Python
- Activity/maintenance: ~580 stars, 43 commits on master; research code.
- Architecture: a **discrete-event kernel** with a global priority queue of timestamped messages, **per-agent-pair latency** models, and computation delays. An ExchangeAgent runs a limit order book (NASDAQ ITCH/OUCH-like). Trading agents include noise, value, momentum and market-maker agents.
- Concept modeled: high-fidelity market microstructure.
- What we can learn: the **message-latency model** matters for us at large scale. Information (prices, rumours, news of a war, word of a miracle) should travel with **distance-dependent delay**, which creates regional price differences, arbitrage traders and stale beliefs.
- What we could integrate: the idea of a latency-aware event kernel; code is BSD-3 but domain-specific.
- What we should NOT integrate: limit-order-book machinery (too modern).
- Scalability: ~10^4 agents.

### Jamel (Java Agent-based Macroeconomic Laboratory)
- URL: https://github.com/pseppecher/jamel
- License: **GPL-3.0**
- Language: Java
- Activity/maintenance: ~31 stars, 74 commits; research (Seppecher).
- Architecture: households, firms and one bank; **endogenous credit money** (the bank creates money by lending); stock-flow consistent accounting (every flow is a balance-sheet entry with a counter-entry); firms set prices and wages adaptively based on inventory and vacancies.
- Concept modeled: monetary macroeconomics with crises.
- What we can learn: **stock-flow consistency as an invariant**. In our economy, coins, grain and debts are conserved except at explicit sources and sinks (mining, minting, spoilage, destruction). Late-game banking and credit money should be modelled as balance-sheet entries, which gives emergent debt crises.
- What we could integrate: ideas only (GPL).
- What we should NOT integrate: code.
- Scalability: ~10^3–10^4 agents.

### Eurace@Unibi
- URL: https://github.com/ETACE/eurace_unibi
- License: **GPL-3.0 + EULA** (with an attribution requirement)
- Language: C on the FLAME framework (XParser, libmboard)
- Activity/maintenance: ~23 stars, 54 commits; academic (Bielefeld).
- Architecture: modules for consumption goods, investment goods with capital vintages, labour market with skills, credit, financial management, government, statistical office. Agents communicate via message boards (FLAME); markets are decentralised with local search.
- Concept modeled: a full macroeconomy.
- What we can learn: the **"statistical office" agent** pattern. A dedicated agent computes aggregates (prices, unemployment, Gini) and broadcasts them with delay. In our world this is literally how a state learns about its own economy, and pre-state societies have no such office. That is a good institutional emergence target (scribes → census → tax).
- What we could integrate: ideas only.
- What we should NOT integrate: code (GPL/EULA).
- Scalability: medium-high (FLAME parallel).

### Boltzmann Wealth (Mesa) — inequality baseline
- URL: https://github.com/projectmesa/mesa (examples/basic/boltzmann_wealth_model)
- License: Apache-2.0
- Language: Python
- Architecture: every agent with money gives 1 unit to a random agent each step. The distribution converges to exponential (Gini ≈ 0.5) with no skill or exploitation.
- What we can learn: a **null model**. Pure random exchange already produces inequality, so our emergent inequality should be compared against this baseline before we claim institutions "cause" it.
- Scalability: trivial.

---

## Part G — Crime & justice

### Short et al. burglary hotspot model (shortBurglary, Mesa)
- URL: https://github.com/jjapp/shortBurglary
- License: none stated (**treat as all rights reserved — do not copy**)
- Language: Python (Mesa)
- Activity/maintenance: ~2 stars, 15 commits.
- Architecture (Short, D'Orsogna, Pasour, Tita, Brantingham, Bertozzi, Chayes 2008): each house s has attractiveness A_s = A0_s + B_s. The burglary probability per step is p_s = 1 − exp(−A_s·δt). After a burglary, B_s jumps by θ (**repeat victimization**). B decays at rate ω and **diffuses** to neighbours with weight η (**broken-windows/near-repeat**). Burglars move preferentially toward attractive houses (biased random walk) and are generated at rate Γ. The PDE limit shows stationary vs. dynamic hotspots.
- Concept modeled: crime hotspots from repeat and near-repeat victimization.
- Computational cost: grid diffusion, cheap.
- What we can learn: crime should be **spatially self-exciting**. A theft raises nearby attractiveness, and policing or watchmen (or a feared shrine!) suppresses it. A *sacred* site with a feared deity could reduce A locally (perceived surveillance by supernatural watchers, the "Big Gods" hypothesis).
- What we could integrate: equations (from the paper), not the repo's code.
- What we should NOT integrate: repo code (no license).
- Scalability: excellent (field on the map).

### Altruistic punishment / Big-Gods monitoring — theory reference
- See the Multilevel altruistic punishment and Axelrod Norms entries. Additional design-relevant theory (from literature): Norenzayan's **Big Gods** hypothesis says belief in moralizing, watching gods reduces the need for costly human monitoring in large anonymous groups. In our sim, a belief trait "god_watches(theft)=w" reduces the subjective probability of getting away with a crime. Societies whose deity concept (emergently) became moralizing can scale with fewer guards.

---

## Part H — War

### Lanchester's laws (theory)
- URL: https://en.wikipedia.org/wiki/Lanchester%27s_laws
- License: n/a
- Architecture: **Linear law** (ancient melee, one-on-one, or unaimed area fire): losses per unit time are constant (one-on-one duels, dA/dt = −β) or proportional to both force sizes (unaimed area fire, dA/dt = −β·A·B). Either way the outcome depends on effectiveness × N. **Square law** (aimed fire, all can engage all): dA/dt = −β·B and dB/dt = −α·A, so strength is ∝ effectiveness × N². The winner's survivors are ≈ √(αA0² − βB0²)/√α. Mixed/"Lanchester–Osipov" exponents between 1 and 2 fit historical battles.
- Concept modeled: aggregate attrition.
- What we can learn: **far-LOD battle resolution in O(1)**. The exponent can be culture/tech dependent (spears ~1.2, massed archery ~1.6, firearms ~2), so technology and tactics emerge as different exponents.
- What we could integrate: equations.
- Scalability: trivial.

### Turchin 2013, Cederman GeoSim, circumscription model
- See Part E; together they cover state-level war (selection on institutions, front allocation, conquest threshold, subjugation vs. flight).

### Historia Extera (procedural history with wars, faiths, schisms)
- URL: https://github.com/lalmei/historia_extera
- License: **AGPL-3.0-only**
- Language: C# (.NET 10) engine, TypeScript viewer, Python tools
- Activity/maintenance: ~0 stars, ~331 commits, 51 open issues. New, active, small.
- Architecture: a deterministic, seeded world-history generator (same seed + settings + engine version + system order gives byte-identical history). Realms, settlements, genealogies with memories and life events, religious institutions with **schisms and holy sites**, campaigns and sieges, trade routes, plagues and disasters. Output is a finished history with a browser viewer (map, timeline, biographies, filtered chronicles).
- Concept modeled: full procedural history (generator, not a live world).
- What we can learn: **determinism contract** ("system order" is part of the seed) and the viewer design: filtered chronicles plus biographies plus timeline. Both are needed for our legends mode.
- What we could integrate: ideas only (AGPL).
- What we should NOT integrate: code.
- Scalability: centuries in batch.

---

## Part I — History recording, procedural history & story sifting

### Caves of Qud — "Sultan" mythic biography generator (Grinblat & Bucklew, FDG 2017)
- URL: paper "Subverting historical cause & effect: generation of mythic biographies in Caves of Qud" (https://www.researchgate.net/publication/319364267 ; PDF at freeholdgames.com/pcgworkshop.com was blocked by egress, so details here come from abstracts, the wiki excerpt in search results, and the GDC talk "Procedurally Generating History in Caves of Qud").
- License: proprietary game; paper public.
- Language: C# (game)
- Activity/maintenance: shipped (1.0 in Dec 2024).
- Architecture: a **state machine per sultan** steps through a life (born → … → dies). At each step a **random event type** is chosen (founds a city, forms an alliance, wages war, is betrayed, has a revelation…). The event is **parameterised by the sultan's state**, notably its **domains** (one assigned at birth, 0–2 more accrued: e.g. glass, ice, jewels, might) and properties (region, cognomen, allies). Each event emits a **"gospel"**, a text snippet generated by a **Tracery-like replacement grammar** keyed on those properties. Crucially, **causality is retrofitted**: events are chosen randomly, but the gospel text *professes* a cause drawn from the sultan's existing state ("because the sultan loved glass, she…"). Events also mutate state, so later events seem caused by earlier ones. Multiple, partly contradictory gospels for the same event exist, which the player pieces together, and later events can be **re-narrated** by cults with bias.
- Concept modeled: mythic history = random events + post-hoc rationalization.
- Computational cost: trivial.
- What we can learn: two key lessons. (1) **Post-hoc rationalization is psychologically true.** Our inhabitants should *explain* events (especially god interventions) using their existing state: domains, sacred places, enemies. That is exactly how myths form. (2) **Multiple biased gospels per event.** Each culture keeps its own account, with spin.
- What we could integrate: the "gospel per perspective + grammar keyed on state" approach for our chronicle text layer.
- What we should NOT integrate: purely random event choice. Our events come from the simulation; only the *interpretation* is rationalized.
- Scalability: trivial.

### Dwarf Fortress Legends + LegendsViewer-Next
- URL: https://github.com/Kromtec/LegendsViewer-Next (old: https://github.com/Kromtec/LegendsViewer, GPL-3.0, archived/legacy)
- License: MIT (LegendsViewer-Next); GPL-3.0 (legacy)
- Language: C# (.NET 10) backend + Vue 3/TypeScript frontend
- Activity/maintenance: ~201 stars, ~261 commits, active; supports the Steam release.
- Architecture: parses DF's `legends.xml` (+ `legends_plus.xml` from DFHack) and world-history text. DF's own data model (from the export structure): **historical figures** (race, caste, birth/death, entity links, family links, skills, kills); **entities** (civs, site governments, religions, guilds, bandit groups) with positions and position-holders over time; **sites** and structures; **artifacts**; **historical events** (a typed record with year, seconds, type and typed fields such as `hfid`, `site_id`, `entity_id`, `slayer_hfid`), e.g. hf died, hf born, change hf state, created site, attacked site, artifact created, entity position change, hf became leader, …; **event collections** (war, battle, duel, site conquered, beast attack, abduction, theft, purge, persecution, ceremony, procession, performance, competition, journey) that **group events hierarchically** (a war contains battles which contain duels); **eras** ("Age of Myth", "Age of Legends", named by the mix of megabeasts and civilizations alive); written content and poetic/musical forms.
- Concept modeled: a canonical **event-sourced world history** for a large simulated world.
- Computational cost: DF worlds generate ~10^5–10^6 events; the viewer handles large XML efficiently.
- What we can learn: **the schema**. (1) Atomic typed events with foreign keys to figures/entities/sites/artifacts. (2) **Event collections** as a hierarchy (war ⊃ battle ⊃ duel). (3) **Eras computed from world state**, not chosen by fiat. (4) Positions held over time as interval records. This is a battle-tested schema we should mirror.
- What we could integrate: LegendsViewer-Next is MIT, so its UI patterns (family tree, map scrubber, paginated tables) can be borrowed. DF itself is proprietary, so take the schema concept, not data.
- What we should NOT integrate: GPL legacy viewer code.
- Scalability: proven at 10^6 events.

### Felt (Kreminski et al.) — story sifting with Datalog
- URL: https://github.com/mkremins/felt
- License: not stated on the page (**unverified — treat as unlicensed**)
- Language: JavaScript (uses DataScript)
- Activity/maintenance: ~23 stars; reference version, with active development in project repos (Why Are We Like This?, Diarytown).
- Architecture: all events go into a DataScript (Datalog) database as entity/attribute/value facts. A **sifting pattern** is a Datalog query with logic variables binding events and characters, e.g. *guest enters town; host does something kind to guest; host later harms guest; guest never left in between* = "violation of hospitality". Felt also uses sifting *inside* the simulation: actions have preconditions written as sifting patterns, so characters can "react to stories" (e.g. seek revenge only if the betrayal pattern matched).
- Concept modeled: detecting narratively meaningful event *sequences* in a chronicle.
- Computational cost: Datalog joins; fine for 10^4–10^5 events, slower beyond without indexing.
- What we can learn: **significance is not only per-event magnitude; it is pattern completion** (betrayal, revenge, rise-and-fall, prophecy fulfilled). Patterns over our event log become both chronicle headlines and **agent motives** (a culture "remembers" the betrayal and that drives a feud).
- What we could integrate: the approach (Datalog-like patterns over an event store). Code licence unknown, so re-implement.
- What we should NOT integrate: DataScript at 10^7 events without windowing.
- Scalability: medium; incremental versions needed (see Winnow).

### Winnow (Kreminski, Dickinson, Mateas, AIIDE 2021) — incremental sifting
- URL: https://github.com/mkremins/winnow ; paper https://cdn.aaai.org/ojs/18903/18903-52-22669-1-2-20211004.pdf
- License: not displayed (**unverified — treat as unlicensed**)
- Language: JavaScript
- Activity/maintenance: ~32 stars, 22 commits; research.
- Architecture: a declarative DSL where a pattern is a sequence of event clauses with shared variables plus **"unless" clauses** (an intervening event that kills the match). It compiles to an **incremental state machine**: each new event advances partial matches, so the cost is per-event, not a re-query of the whole history. It supports **partial matches** ("a story in progress": e.g. betrayal happened, revenge pending) that the system can surface or *nudge*.
- Concept modeled: real-time story detection during simulation.
- Computational cost: O(active partial matches) per event.
- What we can learn: this is **the right runtime shape for our chronicle**. Thousands of partial-match automata run over the event stream during fast-forward, and completed matches get high significance. Partial matches also give *foreshadowing* for the player ("the exiled prince still lives…").
- What we could integrate: design (re-implement; licence unclear).
- What we should NOT integrate: unbounded partial matches. Expire them by time, and cap per pattern.
- Scalability: good if partial matches are pruned.

### "Select the Unexpected" (Kreminski et al., ICIDS 2022) & Synthesifter
- URL: https://www.researchgate.net/publication/365929507_Select_the_Unexpected_A_Statistical_Heuristic_for_Story_Sifting (author PDF host blocked)
- License: paper
- Architecture (from literature; PDF not re-read): rank sifted matches by **statistical unexpectedness**. Estimate how probable a match's attribute combination is under the background distribution of the chronicle, and surface the improbable ones (surprisal ≈ −log P). Synthesifter (NeurIPS 2023 workshop) uses examples and LLM help to author Felt patterns.
- Concept modeled: novelty-based significance.
- What we can learn: our significance score needs a **surprisal term**. The 40th border skirmish is boring; a peace treaty between two cultures with a 300-year feud is gold.
- What we could integrate: the surprisal formula over event-type × context counts.
- Scalability: counting tables are cheap.

### Tracery (Kate Compton)
- URL: https://github.com/galaxykate/tracery
- License: Apache-2.0
- Language: JavaScript (ports in many languages)
- Activity/maintenance: ~2.2k stars; development moved to tracery.io (2024 note).
- Architecture: a grammar object maps symbols to lists of expansions; `#symbol#` expands recursively; modifiers (capitalize, a/an, plural); push/pop of saved symbols for consistent naming within a story.
- Concept modeled: generative text from grammars.
- What we can learn: the **cheap, deterministic text layer** for chronicle sentences and gospels (Qud used a Tracery-like grammar). LLM narration can be layered on top for highlighted events only.
- What we could integrate: Apache-2.0, so code is allowed (or a native port).
- What we should NOT integrate: n/a.
- Scalability: trivial.

### civs (Federico Tomassetti) — civilization evolution simulator
- URL: https://github.com/ftomassetti/civs
- License: Apache-2.0
- Language: Clojure
- Activity/maintenance: ~152 stars, ~179 commits; dormant.
- Architecture: populations (groups) on a generated world (`lands`) progress band → tribe → chiefdom → nation by thresholds; they migrate to better land, found settlements, adopt agriculture, switch nomadic ↔ sedentary, split and merge, and **develop languages** (`langgen`), so daughter groups' languages diverge. The history is browsable via `civs-browser`.
- Concept modeled: group-level civilization history.
- What we can learn: language divergence on group split is an elegant **cultural-lineage marker**. Language distance between groups correlates with time since split and becomes an observable "family tree of peoples". Its fixed band→nation ladder is exactly what we must *not* do.
- What we could integrate: Apache-2.0, so ideas and code are allowed (Clojure, so ideas in practice).
- What we should NOT integrate: hard-coded progression thresholds.
- Scalability: high (group-level).

### Simman — "false history from honest mechanisms"
- URL: https://github.com/Senior-Execute-Vibe-Development-Chief/Simman-
- License: not specified on page (**unverified — treat as unlicensed**)
- Language: JavaScript (browser)
- Activity/maintenance: 0 stars, ~3.2k commits; very active; appears largely AI-assisted.
- Architecture (from README): tectonics → climate → rivers → Neolithic settlements on Voronoi territories with food production. A **closed money supply** (mining faucet, wear drain, conservation in trade). Roads and sea lanes emerge from trade. Polities expand via war fronts with overextension and fragmentation. Folk faiths become **churches in literate towns** and spread via trade networks. A generative phonology per culture names everything. Realms **write their own histories with information horizons, archive loss and court bias**.
- Concept modeled: emergent world history with *in-world historiography*.
- What we can learn: two ideas worth stealing. (1) **Money as a conserved quantity with explicit faucets and drains**. (2) **In-world chronicles are biased and lossy by mechanism**: a realm only records what reached its court, slanted toward its rulers, and archives burn. That gives "the god sees truth; the mortals' scriptures disagree", which suits a god game very well.
- What we could integrate: ideas only (no licence).
- What we should NOT integrate: code.
- Scalability: claims thousands of years in-browser (unverified).

### TERRA (KeyAIGit) — civilization simulator with sub-agent people
- URL: https://github.com/KeyAIGit/terra
- License: not stated in fetched summary (**unverified**)
- Language: Python 3.11 (numpy/scipy) + Three.js viewer
- Activity/maintenance: 0 stars, 31 commits (very new).
- Architecture (README claims): L0 planet, L1 societies (population, warfare, institutions), L2 people (traits, beliefs, kinship), with a "book of people" (biographies of inventors, rulers and decision-makers only). Technologies are gated by physical preconditions, not a tree. Language sound-laws on population split. Deterministic replay.
- Concept modeled: multi-layer LOD history.
- What we can learn: the **three-layer split (planet / society / notable people)** with a **"book of people" recording only notable individuals** matches our LOD needs. Promote individuals into the book when they cross significance thresholds.
- What we could integrate: ideas.
- What we should NOT integrate: unverified code.
- Scalability: unverified.

---

## Recommended design for our game

Design principles (from everything above):
1. **No ladders.** Nothing in code says "tribe" or "state". These are *labels computed after the fact* from measurable properties (size, number of nested levels, existence of offices, taxation, standing force).
2. **One transmission engine.** Culture, religion, norms, prices, techniques and even rumours spread through the same machinery: social-learning events with biases (conformist, prestige, success, content) and noise.
3. **Events are the substrate.** Every simulation outcome emits a typed event. Beliefs, myths, grudges, history books and the player's legends view are all *derived from the event log*, through different filters (truth for the god; biased, lossy perception for mortals).
4. **Three LOD levels** for every social system: L2 *individual* (near the camera / notable people), L1 *cohort/household* (aggregated distributions), L0 *population/polity* (equations: Boyd–Richerson Δp, SDT, Lanchester, Turchin-like selection). The same quantities exist at every level so that promotion and demotion are lossless enough.

### 1. Cultural trait representation & transmission

**Trait registry (data, not code).** A cultural trait is a typed record:
- `id`, `domain` (subsistence, cuisine, clothing, naming, kinship rule, marriage rule, burial, taboo, ritual/practice, belief, norm, aesthetic, technique, language-lexeme, law),
- `payload` (domain-specific parameters, e.g. a taboo = {action pattern, target class, sanction severity}; a naming rule = {patronymic | matronymic | ancestor-reuse | event-derived}; a cuisine trait = {ingredient set, preparation verbs}),
- `grounding` = references to events/places/objects/persons that gave rise to it (the URR "chain of meaning" requirement, enforced structurally),
- `parent_trait` (lineage, for mutation trees), `origin_event`, `origin_agent`.

New traits are created by **(a) innovation** (an agent mutates an existing trait's payload, more likely when stressed or under-performing: guided variation), **(b) recombination** (two traits combine, e.g. a funeral practice plus a storm deity gives a storm-funeral), and **(c) event grounding** (a salient event spawns a trait; see the religion pipeline). Traits are *not* drawn from an authored list, but payload grammars are authored (the space of possible taboos, rites and names), much like Tracery grammars.

**Per-agent (L2) state:** a sparse map `trait_id → (adoption ∈ [0,1], conviction ∈ [0,1], last_reinforced_tick)`. Adoption is behaviour (do I practise it); conviction is private belief (the Emperor's-Dilemma split). Capacity is bounded (e.g. ≤ 64 active traits per domain), and unused traits decay.

**Per-population (L1/L0) state:** for each group, a frequency vector `p[trait]` plus mean conviction, and a compact **culture signature** (MinHash/bit-sketch of the top traits) used for fast cultural-distance computation (Axelrod overlap) between groups.

**Transmission event (L2):** when agents A and B interact (co-work, eat, ritual, trade, gossip):
1. Interaction probability is gated by **similarity** s = overlap(signature_A, signature_B) with a floor ε (Axelrod plus a little noise to avoid freezing).
2. The learner picks a domain weighted by salience of the context (at a funeral → burial domain).
3. Adoption probability for trait t shown by the model M:
   `P = base · content_bias(t) · prestige(M)^β · conformity(f_t)` with `conformity(f) = f^α / (f^α + (1−f)^α)` (α>1 gives a conformist S-curve), `prestige(M)` = how often M is copied plus status/dominance, and `content_bias` = intrinsic attractiveness (taste, memorability, emotional arousal of a rite).
4. Behaviour (adoption) can change under **social pressure** without conviction changing. Pressure = local enforcement observed. Conviction updates only through direct experience or credible testimony.
5. Mutation rate μ per transmission (copy error), giving **drift**.

**Population-level (L0) update** for unobserved groups:
`Δp_t = μ(…) + B_t·p(1−p) + D·p(1−p)(2p−1) + migration_in − migration_out + noise/√N` (Boyd–Richerson forces plus drift scaled by group size). Promotion to L2 samples individual traits from p; demotion re-aggregates.

**Divergence/hybridization (emergent, not buttons):** if the cultural distance between two sub-populations of a group exceeds a threshold *and* their interaction rate falls (geography, endogamy), the group detector marks a **split** (new culture lineage; language lexemes diverge from that point). If two cultures share territory with high interaction for long, traits mix and the detector may label a **hybrid**. CK3's pillars are used only as a **UI summary** of the trait vector.

### 2. Emergent religion pipeline

Stages, each a measurable state transition, never a scripted outcome:

**Stage 0: Perception & agency attribution (individual).** Every simulation event carries `salience` (visual/audible magnitude, damage, deaths, rarity) and is perceived by agents within range (perception radius, line of sight; others hear rumours later with latency and distortion, ABIDES-style). Each agent has a lightweight **world model**: frequency expectations for event types by place and season. For a perceived event e:
- `surprise = −log P_model(e | place, season)`; `consequence = harm/benefit to self, kin, group`.
- If `surprise·consequence > θ_agency` (θ deliberately low, per HADD: false positives are cheap) **and** no naturalistic explanation in the agent's known causal set (a "known cause" trait like "storms come from the sea in autumn" lowers surprise), the agent forms an **agency hypothesis** H = {locus (place/object/sky), inferred disposition (angry/protective/demanding), associated cues (time, weather, preceding human action)}.
- **Post-hoc rationalization (Qud lesson):** the hypothesis's disposition is chosen by matching the agent's recent memory, e.g. "the lightning came after we cut the old tree", so the inferred motive is "the mountain punishes tree-cutting". This uses the agent's own state, so different agents and cultures produce different stories from the same strike.

**Stage 1: Shared myth (social).** Hypotheses are cultural traits of domain `belief` and spread by the transmission engine, with **content bias for minimally counterintuitive** agents (a person-like mountain spreads better than an abstract force) and arousal-weighted memorability. Repeated corroborating events raise conviction for everyone who witnessed or heard of them. Competing hypotheses about the same locus **merge** if similar or **compete** if not. The surviving cluster becomes a **myth object**: `{entity: "the one in the peak", locus, domains: [lightning, tree-cutting], dispositions, grounding_events[]}`. The entity is a *belief construct* in the inhabitants' minds. It has no link to the player except through the grounding events.

**Stage 2: Ritual (behaviour).** When conviction × perceived consequence is high, agents generate **practices** (Versu-style practice objects: roles, stages, place, timing, offerings, prohibitions) by mutating existing practices (feasts, funerals) toward the myth: an offering at the foot of the mountain; a taboo on cutting trees on its slopes; a ritual timed to the storm season. A practice's apparent efficacy is evaluated by agents (did lightning stop after the offering?), so **superstitious reinforcement** (Skinner-like) applies. The player's choice to strike or not strike *after* the ritual directly shapes doctrine.
Ritual mode follows DMR: rare high-arousal rites (imagistic) raise in-group cohesion and episodic memory strongly but transmit poorly beyond the group; frequent low-arousal rites (doctrinal) need a specialist to keep them consistent and spread widely, but accumulate tedium.

**Stage 3: Institution.** A **religious office** emerges when (a) the practice requires coordination or specialised knowledge (many stages, calendrical timing, a large population), (b) someone gains prestige from leading it (their copies count; they are the "one who knows the mountain's will"), and (c) group size makes ad-hoc leadership costly (Perret et al.). The office becomes persistent (succession rules needed), may collect offerings (an economic sink becomes a temple treasury), and may author **canon**: a frozen, high-conformity version of the myth (doctrinal mode). Literacy (when emergent) allows written canon, which lowers mutation rate. Existential-security coupling (MRP) means religiosity rises with threat and falls with sustained safety and wealth.

**Stage 4: Schism & reformation.** Triggers, each measurable:
- **Doctrinal drift**: a sub-population's belief vector diverges from canon beyond a threshold while the office enforces conformity.
- **Tedium**: frequent routine rites raise tedium; a charismatic innovator offering imagistic rites draws a splinter group (DMR). Arousal decays, and the group either re-assimilates or institutionalises.
- **Disconfirmation**: the canon predicts protection, the god (player) strikes the temple, and prophets reinterpret. This yields a reform movement *or* cognitive-dissonance-driven intensification ("we were not pure enough").
- **Political capture**: the ruler co-opts the office, and opposition elites back a rival interpretation (SDT elite competition).
- **Emperor's-Dilemma collapse**: privately weak conviction plus a drop in enforcement gives sudden mass apostasy.

**Worked example: the god's lightning on Mount K.**
1. Year 0, summer. Player strikes Mount K three times in a month. Band B (40 people) lives 5 km away; 25 see the flashes; one strike kills a hunter on the slope. Surprise is high (lightning on K is normally rare), consequence is high for the hunter's kin.
2. Agents generate hypotheses using their own memories. The hunter's widow recalls her husband boasted about taking eagle eggs from the summit, so H1 = "the peak is a jealous owner of its birds". An elder recalls the band burned scrub near K last week, so H2 = "the peak hates fire made by us". A child forms H3 = "the sky-beast lives on K".
3. Gossip over a month: H1 spreads fastest (the widow is grieving and high-arousal, and the elder's prestige is moderate; H1 is minimally counterintuitive and has a vivid grounding). H3 merges into H1 (both are an agent-on-K), giving the myth "a being on K owns the high places and kills thieves". Conviction for H1 ≈ 0.6 in the band; H2 survives as a minority trait.
4. Year 1. Nobody climbs K (an emergent taboo, a norm trait with a sanction of shunning). Player stays quiet, and the taboo is "confirmed" (no deaths). Someone leaves meat at the foot of K before a hunt; the hunt succeeds, so the offering practice is reinforced.
5. Year 3. Player strikes K again during a storm season. Because the culture already has the myth, surprise is lower but confirmation is strong, and conviction rises to 0.8. The rite gains a timing: offerings when the first storm comes.
6. Year 40. The band has grown into three villages. The rite now involves processions, songs and a fixed calendar, and an old woman whose family "has always led the offering" holds a hereditary office. Offerings accumulate at a shrine, which becomes a store of value (and a raid target).
7. Year 120. One village, downstream, rarely sees lightning (geography). Its version drifts: the being is now benevolent (it sends rain). The shrine office insists it is wrathful. Tension rises over the 'right' rite. A drought (or the player sending rain *after* the downstream rite) tips it into a **schism**: the "Rain-Giver" cult versus the "Wrath of K" orthodoxy, now with different calendars and food taboos, and later intermarriage restrictions.
8. Nothing in code ever said "mountain god". The deity name is generated from its grounding attributes ("the One of the Peak", "Eagle-Keeper") through a naming grammar, and different cultures' names for it diverge with language.

### 3. Relationship vector schema

Per **directed** dyad A→B, stored only for each agent's known others (cap ≈ 150 with Dunbar-like layers: 5 intimate / 15 close / 50 friends / 150 acquaintances; slots are evicted by lowest `salience`):
- `kin`: computed, not stored (see kinship below), cached as {relation_type, coefficient r}.
- `familiarity` [0,1]: grows with co-presence, decays slowly.
- `affection` [−1,1]: liking (warmth).
- `trust` [0,1]: reciprocity memory (WSLS-like). Updated on cooperate/defect events, with forgiveness noise.
- `respect/prestige_attributed` [0,1]: how much A copies or defers to B (feeds prestige-biased learning).
- `dominance` (DomWorld DOM ratio): pairwise outcome history plus a global DOM value.
- `attraction` [0,1] (sexual/romantic), gated by species/culture rules.
- `debt` (signed, in goods/favours/money): from gifts, loans, blood debts. Feeds economy and feuds.
- `grievance` [0,1]: from harm events, decays, with a `grudge_events[]` pointer list.
- `obligations`: typed edges (vassal-of, spouse-of, apprentice-of, member-of-office), as exclusion-logic single-valued slots where appropriate.
- `shared_identity`: derived from culture/religion overlap (not stored; computed from signatures).
- `last_interaction_tick`, `salience` (for eviction and LOD).

Updates come from **typed social events** through an Ensemble-like rule table: `(event_type, role_A, role_B, context) → Δ on dimensions`, modulated by A's personality and culture (e.g. cultures with honour norms convert insults into larger grievance). Formation dynamics: **homophily** (interaction probability ∝ similarity in culture signature, age, status), **triadic closure** (if A–C and C–B are strong, schedule an A–B introduction with probability p_tc), **focal closure** (shared workplace/ritual), and **decay**. Validate offline with NetworkX: clustering coefficient, degree tail, community modularity.

**Kinship:** store only `mother`, `father` (biological), and `social_parents[]` (adoption/fosterage), plus `spouses[]` intervals. Compute relations on demand by bounded BFS up the genealogy (depth ≤ 4–5) with a per-agent LRU cache. Kin terms are **culture-dependent mappings** from genealogical paths to categories (e.g. a culture may merge father and father's brother, giving a Hawaiian/Iroquois-like system). Kinship terminology is itself a cultural trait that can drift. Descent rules (patrilineal / matrilineal / cognatic), residence rules (patri-/matri-/neolocal) and marriage rules (exogamy, cousin preference, incest taboo radius) are cultural traits that determine clan formation.

**Death's social consequences (pipeline on `died(agent)`):**
1. **Grief:** for each related or known agent, `grief = f(affection, kin r, familiarity, dependency)`. This lowers mood and productivity and raises susceptibility to religious explanation (agency attribution threshold drops: "who caused this?"). Unexplained deaths can trigger witchcraft accusations (agency attributed to a human rival), a crime/justice link.
2. **Funerary practice:** the culture's burial-practice trait executes (practice object), giving cohesion and a chance for ritual innovation.
3. **Inheritance:** apply the culture's inheritance-rule trait (Sugarscape-style split among children, primogeniture, ultimogeniture, to the brother, to the clan, to the temple, partible vs. impartible). Disputes arise when multiple rules have partial adoption in the group (rule ambiguity creates conflict), which spawns potential feud/court events.
4. **Vacated positions:** for every office/role held, run the institution's succession rule (hereditary / elective / appointment / seniority / contest). If none is defined (a first-generation office), a **succession crisis** occurs: candidates with prestige/dominance/kin support compete, and the outcome **establishes precedent** (adds a succession-rule trait to the institution with conviction proportional to how peaceful the outcome was).
5. **Debts and obligations:** debts pass to heirs or are cancelled per norm; vassalage edges are renegotiated; marriage alliances may lapse.
6. **Memory:** the deceased becomes a **historical figure** if their significance exceeds a threshold (see the chronicle). Ancestor-veneration traits can attach to them, and a sufficiently revered ancestor can become a deity-like myth entity over generations (euhemerism emerges).

### 4. Group & institution emergence rules (no fixed progression)

A **group** is a first-class entity (meta-agent) with members (weighted), a culture frequency vector, a treasury/stores, territory claims, offices and norms. Groups nest and overlap (a person is in a household, a lineage, a cult, a guild, a village, a polity).

**Detection (bottom-up):** every N ticks (staggered by region), run incremental community detection (label propagation) over the strong-tie graph weighted by co-residence, kinship and cooperation. A community that persists for ≥ T periods with internal tie density > d becomes a **candidate group** and gets an entity if it engages in **collective action** (shared store, joint defence, a ritual with ≥ k participants). Kin-based clusters under unilineal descent rules give *lineages/clans*; ritual-based clusters give *congregations/cults*; occupation plus trade-based clusters give *guilds* (when specialists coordinate prices or apprenticeships).

**Formation by decision (top-down within agents):** agents (or groups) propose **alliances** when there is a common threat or gain. Use a bilateral-Shapley-like test (does each party's expected payoff rise?) as in Mesa Alliance Formation; the alliance becomes a meta-agent.

**Offices/institutions** are created when conditions hold, never by date:
- *Leader office*: group size > S_coord (coordination cost), plus repeated collective actions led by the same prestigious/dominant individual (situational → institutional leadership).
- *Council*: several sub-group heads with comparable power (no single dominant), giving collective decision (voting rule as a cultural trait: consensus/majority/elders-weighted).
- *Tax/tribute*: a leader office provides public goods (defence, ritual, arbitration), members contribute, and the contribution becomes a norm (and *coercive* when dominance asymmetry plus circumscription remove the exit option, per Carneiro).
- *Court/arbiter*: frequent intra-group disputes (grievance events) plus an office with legitimacy, so disputants bring cases (see crime).
- *Standing force*: repeated warfare plus treasury surplus, giving full-time warriors (and a coup risk).
- *Priesthood*: see Stage 3 of religion.
- *Scribes/census*: writing exists plus a tax office, enabling a "statistical office" that reduces the state's information delay.

Every institution is a **cultural trait at group level** (Turchin): it costs upkeep, raises cohesion or capacity, and can be copied by neighbours (imitation of successful polities) or lost (mutation toward loss is more likely than gain). War, collapse and migration select among them.

**Labels** (band / tribe / chiefdom / state / empire) are computed for UI from: population, number of nested levels of offices, presence of permanent hereditary office, presence of tax and courts, and territorial control. They may appear in any order (a theocratic city-state may arise before any "tribe" label appeared nearby), and they can **regress**.

**Politics within groups:** legitimacy L ∈ [0,1] per office = f(adherence to succession precedent, religious sanction, military success, perceived fairness, prosperity). Unrest uses the Epstein rule: G = hardship·(1−L); rebels activate if G − R·P_arrest > T. Elite dynamics use the SDT stocks (elite count, competition for offices; PSI as the crisis predictor) at polity level.

### 5. Economy: barter → commodity money → currency (conditional)

**Phase-agnostic core:** goods are conserved items with properties `{durability (spoil rate), divisibility, portability (value/weight), recognisability, intrinsic use value}`. Production follows recipes. Every transfer is logged (stock-flow consistent; explicit faucets: harvest, mining, crafting; sinks: consumption, spoilage, offerings burned, loss).

**Stage A: Gift/reciprocity (bands):** sharing within kin and close ties is governed by relationship `debt` and norms (no prices). Generalized reciprocity is cheap and keeps small groups fed.

**Stage B: Barter:** between non-intimates, bilateral exchange uses private valuations (MRS from need/inventory). Price per trade = geometric mean of the two MRS (Sugarscape), and trade proceeds unit by unit while both gain. Agents keep **private price beliefs** that converge by imitation of successful traders (Gintis), routed through the culture-transmission engine. Periodic gatherings (a ritual fair at the shrine!) concentrate trade and are a natural seed for markets.

**Stage C: Commodity money** emerges per Kiyotaki–Wright logic. Each agent tracks for each good g an **acceptability score** = how quickly and reliably g could be re-traded (reinforcement from experience, plus observed others' acceptance, i.e. conformity). An agent accepts g it does not need when `acceptability(g) − storage_cost(g) > threshold`. Positive feedback (everyone accepts g because everyone accepts g) gives a **medium of exchange** whose identity depends on local goods: cattle in pastoral cultures, salt, shells, metal rings. Conditions that make it happen: enough specialization (no double coincidence of wants), trade frequency above a threshold, and at least one durable, divisible, recognisable good. If conditions fail, barter persists indefinitely, which is fine. Detection: when > X% of non-consumption trades in a market involve good g, the UI labels "g-money".

**Stage D: Markets with posted prices:** where trade density is high (settlement with fairs), switch from bilateral barter to a **bazaarBot-style clearing house** per market site. Bids and asks are priced in the emergent money (or in a numeraire good before money: the market computes clearing in the most-accepted good). Price-belief intervals update, and bankrupt or under-earning producers **switch professions** toward the most profitable (division of labour emerges and responds to shocks).

**Stage E: Currency (coinage):** requires an institution with legitimacy plus metal supply plus a minting technique trait. A coin = commodity money plus a **trust premium** from the issuer's stamp (recognisability ↑). Debasement becomes possible: issuer legitimacy and coin acceptability are coupled, so fiscal distress tempts debasement, which brings inflation and loss of trust (SDT link). Later: **credit money** (temples and merchants as banks: balance-sheet entries, Jamel-style), enabling debt crises.

**Information latency:** price knowledge travels with traders and rumours (ABIDES-like delay), so regional price gaps give rise to **merchant roles and trade routes** (path frequency is reinforced, so roads emerge). Wealth inequality: compare against the Boltzmann null model; inheritance rules and rents (land ownership norms) are the main amplifiers.

### 6. Crime & justice emergence

- **Crime is defined by norms, not code.** An act is a "crime" in a culture iff a norm trait prohibits it with a sanction. Theft, violence and taboo violation (climbing Mount K!) are all norm violations of different kinds. Different cultures criminalize different things.
- **Motivation:** an agent considers a violating action when `gain − (P_detect·sanction + guilt + P_divine·divine_sanction) > 0`. `guilt` = own conviction in the norm; `P_detect` = local witness density plus guard/office presence; `P_divine` comes from belief in a watching/moralizing deity (Big Gods). Need, grievance (Epstein G) and low legitimacy raise propensity.
- **Spatial self-excitation:** a crime raises local "attractiveness" B (Short et al.), which decays and diffuses. Guards, shrines and dense kin networks lower it.
- **Enforcement ladder (emergent):**
  1. *Personal retaliation*: victim and kin retaliate (grievance → revenge). This creates feuds (dyadic grievance contagion along kin ties). Sifting patterns detect feud chains.
  2. *Social sanction*: gossip lowers the offender's trust/affection network-wide, ostracism follows, and **metanorm** punishment of those who tolerate offenders (Axelrod) stabilizes the norm.
  3. *Compensation norms*: when feuds are costly (deaths, lost production), cultures that innovate a **compensation trait** (blood money, weregild in whatever the medium of exchange is) out-compete feuding ones (group selection), and it spreads by imitation.
  4. *Arbitration*: disputants voluntarily approach a prestigious third party (elder, priest of K: oaths sworn at the mountain). Success gives the arbiter prestige, and the role institutionalises into an office (a court).
  5. *State justice*: courts with coercive power, codified law (written canon of norms), prisons and punishments. Punishment severity is a cultural trait. Over-harsh justice lowers legitimacy (Epstein loop).
- **Witchcraft/heresy as crime:** unexplained misfortune leads to agency attribution to a *human* rival (same pipeline as religion), producing accusations; the Emperor's-Dilemma dynamic gives witch-hunt cascades.

### 7. War resolution with demographic & economic consequences

**Causes (emergent):** competition for resources under circumscription, accumulated grievances (raids, insults to the sacred, e.g. a rival village climbs Mount K), elite competition (SDT: young surplus elites want glory and land), religious difference (cultural distance, metaethnic frontier), or a leader's prestige needs. **War is a group decision** taken by the group's decision rule (chief decides / council votes).

**Scales:**
- *Raid* (band/tribe): small party, surprise, aims at loot, captives or revenge. Resolve as several individual-level skirmishes (L2) or a simple probability from Lanchester-linear with surprise and terrain multipliers.
- *Battle* (L1): forces aggregated into units with `strength = N^k · quality`, where k ∈ [1,2] comes from the military technique traits (melee ≈ 1, missile ≈ 1.5, firearms ≈ 2); quality = training × equipment × morale × leadership. Integrate discrete Lanchester attrition until **morale break** (casualty threshold set by cohesion: imagistic-ritual groups break later). Rout casualties are the main killer. The Tullock contest P(A wins) = S_A^γ/(S_A^γ+S_B^γ) serves as a cheap alternative at far LOD.
- *Campaign/state war* (L0): Cederman-style fronts. Each polity allocates force across fronts by threat; a province is taken when the local ratio exceeds a superiority threshold plus noise. Logistics: force projection decays with distance from the capital and supply.

**Consequences (where most of the "history" comes from):**
- Demographic: deaths are concentrated in young men (sex ratio shifts lead to polygyny pressure, fewer marriages, lower fertility next generation); captives/slaves move population; refugees migrate (with their culture and religion: diffusion by war); epidemics spread through armies and sieges.
- Economic: destroyed fields/stores (famine risk), loot transfers wealth (inequality spike), war taxes (legitimacy cost), trade disruption (price spikes in the bazaar), veterans' land claims.
- Political: winner's leader gains prestige and legitimacy; the loser's ruler may be deposed (succession crisis); conquered groups are subjugated (tribute, assimilation over generations; Carneiro), or flee if exit is possible; institutions of the winner are copied by neighbours (Turchin selection).
- Religious: victory "proves" the winner's god (conviction ↑); defeat triggers theodicy, reform or conversion; captured shrines and relics become pilgrimage or revenge targets.
- Memory: battles and wars become event collections; grudges are stored as group-level grievance with a slow decay, so revenge wars generations later are possible.

### 8. History / chronicle schema and significance scoring

**Event store (event-sourced, append-only):**
```
Event {
  id, tick, region_id, type,                   // typed: Died, Born, Married, OfficeTaken, Schism, BattleFought,…
  actors: [{entity_id, role}],                 // agent/group/place/artifact/myth ids with roles
  payload: {...},                              // type-specific fields
  causes: [event_id],                          // known causal parents (simulation-level truth)
  collection_ids: [id],                        // war ⊃ battle ⊃ duel (DF-style hierarchy)
  witnesses: count or sample, salience,
  significance: float (computed, may be revised later)
}
EventCollection { id, type, start, end, parent_id, member_events, summary_stats }
Era { id, start, end, label, computed_from }  // labels from world-state features
HistoricalFigure / GroupRecord / PlaceRecord / ArtifactRecord / MythRecord  // promoted entities only
Account { id, event_ids, author_group, bias, text, lost? }  // in-world chronicles: biased, lossy views
```
Two layers: **Truth log** (what happened; visible to the god/player) and **Accounts** (what cultures believe happened; generated Qud-style from the truth log through each group's information horizon, with rationalized causes and biases toward their own heroes and gods; archives can be destroyed).

**Retention for 1,000 years at 10^5–10^6 organisms:** most events are routine. Store everything for a rolling window (e.g. 5–20 years). Beyond the window, keep only events whose significance is above a **dynamic percentile threshold** per region and type, plus all events referenced by a kept event or by a completed sifting pattern, plus aggregate counters (per region/year per type, for statistics and surprisal). Unpromoted individuals are retained as counts in cohort statistics.

**Significance score:**
```
S(e) = w_mag · log(1 + magnitude(e))        // deaths, people affected, wealth moved, territory changed
     + w_rank · rank(actors)                  // prestige/office of actors; promoted figures count more
     + w_sur · surprisal(e)                   // −log P(type | context), from the counting tables
     + w_conseq · downstream(e)               // number/significance of events citing e as cause (updated retroactively)
     + w_pattern · Σ pattern_completions(e)   // Winnow-style sifting matches it completes (betrayal, revenge, fall)
     + w_first · first_of_kind(e)             // first coin, first written law, first schism of a faith
     + w_div · divine(e)                      // directly caused by or attributed to the player-god
     + w_mem · cultural_memory(e)             // # of Accounts/myths/rituals grounded in e (living memory)
     − w_rep · repetition(e)                  // decays for the Nth similar event in the window
```
`downstream` and `cultural_memory` are **retroactive**: an event's significance can grow centuries later (the forgotten lightning strike that founded a world religion). Run a periodic back-propagation pass: S(cause) += λ·S(effect) along causal links (λ ≈ 0.3–0.5, depth-limited).

**Sifting patterns (incremental automata, Winnow-style):** authored library, e.g. *rise-and-fall* (commoner → office → deposed/killed), *betrayal* (trust high → harm), *revenge* (harm → later retaliation by kin), *prophecy fulfilled* (a myth's predicted event type occurs at its locus), *schism*, *exile and return*, *forbidden love* (attraction across taboo), *the god's favour* (repeated divine events on one group). Each pattern's partial matches expire by time. Completed matches create **Story** records with their own significance, which feeds the legends UI and becomes motives for agents (grudges, heroic emulation).

**Fast-forward narration:** at each era boundary (computed from regime changes: polity count jumps, PSI crisis, new religion, population shock), select the top-k events and stories per region by S. Render one-liners with a Tracery-like grammar keyed on entity properties; spend LLM calls only on top-ranked stories and on the player-requested "chronicle of X". Eras are labelled from their dominant features ("the Age of the Storm-Mountain Schism").

## Open questions / risks

1. **Calibration explosion.** Dozens of coupled rates (mutation, conformity, prestige exponent, θ_agency, tedium threshold, Lanchester exponent). We need an offline harness (Melodie-style calibration plus AgentPy/SALib-style sensitivity analysis) and target statistics (number of religions per million people, Gini 0.3–0.6, war-size power law, schism interval of centuries).
2. **LOD consistency.** Moving between L2 individuals and L0 equations must not create visible discontinuities, e.g. a myth that existed only in a few individuals' minds disappearing on aggregation. Mitigation: keep group-level copies of any trait above a minimum frequency, plus "seed carriers" for rare traits.
3. **Determinism vs. parallelism.** A 1,000-year fast-forward must be reproducible (Historia Extera's "system order is part of the seed"). Parallel updates need double-buffering (krABMaga/FLAME patterns) and per-entity RNG streams.
4. **Event-log size.** 10^6 organisms × 1,000 years can produce 10^9+ raw events. The retention and promotion policy must be designed early; retroactive significance needs causal links kept for at least the pruning horizon.
5. **Are religions recognisable?** Agency attribution may produce too many tiny cults or none. The thresholds and content biases need tuning so that player interventions are *legible* to the player (feedback UI showing "what they think happened") without hard-coding interpretations.
6. **Player exploitability.** If religiosity rises with threat, a player may farm devotion by terror. Is that desirable? It is perhaps a feature (moral commentary), but the game's goals must decide.
7. **Licence hygiene.** Many key references are GPL/AGPL/CC-BY-SA/unlicensed (NetLogo, GAMA, Jamel, Eurace, FLAME GPU 2, AgentTorch, Historia Extera, PrestigeBias, Felt/Winnow licence unclear, shortBurglary no licence). Use clean-room re-implementation from papers. The safe-to-reuse code set is Mesa (+examples), bazaarBot, Tracery, Azgaar FMG, go_gens, Axelrod-Python, NetworkX, nkremerh/sugarscape, krABMaga, Agents.jl, ABIDES, LegendsViewer-Next (UI ideas), civs. Ensemble is BSD-4, so re-implement.
8. **Unverified items.** Kiyotaki–Wright NetLogo model (host not fetched), Turchin 2013 code availability, Cederman GeoSim code, DomWorld code, Lane's DMR simulation code, the HADD ABM repo (404), and Melodie's licence. Formulas for those were taken from abstracts/literature, not re-read PDFs, because the academic hosts were blocked.
9. **Ethical/cultural sensitivity.** Emergent religion, witch-hunts, slavery and conquest will occur. Presentation must stay clearly fictional and avoid mapping onto real faiths or ethnic groups (avoid real-world-calibrated religious-demography models).
10. **LLM role.** LLMs should narrate and occasionally author *payloads* (a myth's text, a prayer), never decide simulation truth at scale. Use AgentTorch-style archetypes for cohort-level LLM decisions if used at all; budget per in-game year.
11. **Emergence too slow or too fast.** Without nudges, money or states may never appear in a small world. Decide whether the game guarantees anything (probably not), and expose world parameters (land circumscription, resource diversity) that make emergence likely.
