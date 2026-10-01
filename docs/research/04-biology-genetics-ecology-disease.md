# 04 — Biology, Genetics, Reproduction, Evolution, Animals, Ecology, Plants, Disease, Death

Track: (12) human body / physiology, (13) genetics, (14) reproduction, (15) evolution, (16) animals,
(17) food chain / ecology, (18) plants, (33) disease, (34) death (biological + demographic).

Research date: 2026-10-01. Method: WebSearch + WebFetch of GitHub pages / project sites. Star counts are the
approximate values shown on the GitHub page when fetched. "Unverified" means the page did not show the
fact or could not be fetched (e.g. `gitlab.kitware.com` is blocked by the egress proxy in this sandbox).
The GitHub REST API was not reachable from this session, so exact "last commit" dates are mostly
unavailable; activity is described from what the repository page showed (commit counts, releases,
archive banners, maintenance notices).

License legend for a possibly commercial, closed-source game:
- **OK** = permissive (MIT / BSD / Apache-2.0 / MPL-2.0 file-level copyleft): code may be integrated with attribution.
- **LGPL** = usable only as a dynamically linked, replaceable library; in practice we re-implement.
- **GPL/AGPL** = do NOT link or copy code; learn the algorithm, re-implement from papers.
- **NC / proprietary** = reference only.

---

## Part A — Candidate catalogue

### A.1 Human body & physiology engines / references

### Pulse Physiology Engine (Kitware)
- URL: https://pulse.kitware.com/ (source hosted at gitlab.kitware.com/physiology/engine — not reachable from this sandbox)
- License: Apache-2.0 (stated on Kitware's FAQ/About pages) — OK
- Language: C++ (with C#, Python, Java bindings per docs)
- Activity/maintenance: actively maintained by Kitware per their site; exact last-commit date unverified
- Architecture: forked from BioGears 6.1.1. Whole-body physiology built from lumped-parameter "circuits"
  (electrical analogue: pressures = voltages, flows = currents, compliances = capacitors) for
  cardiovascular, respiratory, renal compartments; substance transport across compartments; separate
  "systems" (Cardiovascular, Respiratory, Energy, Endocrine, Nervous, Drugs, Environment) each with
  preprocess/process/postprocess steps; fixed timestep of 0.02 s. Actions (hemorrhage, drug, exercise,
  airway obstruction) and conditions (chronic anemia, COPD) are injected as events.
- Scientific concept modeled: integrated cardiovascular/respiratory/metabolic physiology, pharmacokinetics
  and pharmacodynamics, trauma response.
- Computational cost: ~50 engine steps per simulated second, each solving several circuit systems;
  realistically faster-than-real-time only by a modest factor for ONE patient. Unusable per-organism
  for a crowd.
- What we can learn: the decomposition into systems + actions + conditions; the idea of "conditions"
  (chronic states) vs "actions" (acute events) maps neatly to our injuries vs chronic disease; validated
  baseline numbers (heart rate, blood volume, O2 consumption) for sanity checks.
- What we could integrate: Apache-2.0 so code is legally usable, but practically only as an offline
  calibration tool (e.g., generate lookup tables: blood loss → consciousness → death time).
- What we should NOT integrate: the real-time engine in the game loop.
- Scalability: 10 = feasible offline / 1 "hero patient" in a hospital scene only; 1k+ = no.

### BioGears
- URL: https://github.com/BioGearsEngine/core
- License: Apache-2.0 — OK
- Language: C++
- Activity/maintenance: ~69 stars; v8.2.0 release January 2025
- Architecture: same lineage as Pulse — lumped-parameter circuits, compartments, substance transport,
  scenario XML files; includes nutrition/energy system, thermoregulation, sepsis, burns.
- Scientific concept modeled: whole-body human physiology for medical training.
- Computational cost: same order as Pulse (sub-second timestep, heavy per patient).
- What we can learn: energy system (glucose/ketones/fat stores, starvation timeline), thermal model,
  burn/sepsis progression; good source of validated curves for starvation and dehydration death times.
- What we could integrate: tables precomputed offline (Apache allows code reuse, but we only need data).
- What we should NOT integrate: runtime engine.
- Scalability: 10 offline only; 1k+ no.

### HumMod / Physiomodel
- URL: https://github.com/physiology/Physiomodel (Modelica port of HumMod 1.6.1); HumMod itself at hummod.org (unverified)
- License: Physiomodel License 1.0 — GPL-3.0, or BSD-3 for "registered parties". Treat as GPL unless registered.
- Language: Modelica (HumMod original: XML model + C++ solver)
- Activity/maintenance: ~14 stars, ~105 commits; low activity
- Architecture: >9000 variables, coupled ODE system of cardiovascular, renal, endocrine, neural, metabolic
  physiology (Guyton-style integrative model).
- Scientific concept modeled: long-horizon integrative physiology (fluid balance, blood pressure regulation).
- Computational cost: very high per individual; stiff ODE solve.
- What we can learn: which variables matter on day/week timescales (fluid volume, sodium, glucose,
  fat, lean mass); confirms that a game body needs ~10 slow state variables, not thousands.
- What we could integrate: nothing at runtime; papers/equations only.
- What we should NOT integrate: code (GPL path) or model.
- Scalability: 1 individual offline.

### bw — Hall dynamic body-weight model (R)
- URL: https://github.com/INSP-RH/bw
- License: MIT — OK
- Language: R
- Activity/maintenance: ~6 stars, ~265 commits; also on CRAN (bw 1.0.0)
- Architecture: implements Hall et al. adult model (fat mass F, lean mass L, glycogen, extracellular
  fluid as ODE states; energy intake/expenditure per day; partition of energy imbalance between fat and
  lean via Forbes' relation p = C / (C + F), C ≈ 10.4 kg) and Hall's child growth model.
- Scientific concept modeled: day-by-day body composition change from energy imbalance; adaptive
  thermogenesis; child growth energy cost.
- Computational cost: a handful of floating-point ops per individual per day — trivially cheap.
- What we can learn: the core equations for our body model:
  rho_F * dF/dt = (1 - p) * (EI - EE),  rho_L * dL/dt = p * (EI - EE),
  rho_F ≈ 9400 kcal/kg, rho_L ≈ 1800 kcal/kg. Rule of thumb: ~24 kcal/day per kg of eventual steady
  state change, half-time ≈ 1 year.
- What we could integrate: MIT, so even code translation is fine; we will re-implement the 2-state version.
- What we should NOT integrate: sodium/ECF sub-model (no gameplay value).
- Scalability: 1M individuals per day trivial (vectorizable).

### Cataclysm: Dark Days Ahead (body-part model reference)
- URL: https://github.com/CleverRaven/Cataclysm-DDA
- License: CC BY-SA 3.0 (main), some Apache-2.0/OFL parts — share-alike: do not copy code/data into closed product
- Language: C++ with JSON data
- Activity/maintenance: ~13.3k stars, ~130k commits, very active
- Architecture: JSON-defined body parts (head, torso, arms, legs, eyes, mouth, hands, feet) each with
  HP, encumbrance, temperature, wetness; "effects" with intensity/duration (bleeding, infection,
  bite, broken limb, pain); vitamins & calories; stomach/guts two-stage digestion; fatigue & sleep
  deprivation; morale. Wounds become infected with probability modified by disinfectant/bandage.
- Scientific concept modeled: game-grade survival physiology.
- Computational cost: per player each turn (6 s); NPCs much lighter. Fine for dozens.
- What we can learn: data-driven "effect" system with stacking intensity — exactly the right abstraction
  for injuries/diseases in observed agents; two-stage digestion produces realistic hunger lag.
- What we could integrate: ideas only (CC BY-SA contaminates).
- What we should NOT integrate: per-turn temperature per body part for background agents.
- Scalability: 10–100 detailed; not for 1k+.

### Dwarf Fortress (design reference only)
- URL: https://www.bay12games.com/dwarves/
- License: proprietary (closed source)
- Language: C++ (unverified, by common knowledge)
- Activity/maintenance: commercial Steam release, still updated (unverified date)
- Architecture: creatures defined by raws: body = tree of parts, each with tissue layers (skin, fat,
  muscle, bone) with material properties; attacks resolved by momentum vs layer material; wounds per
  layer; bleeding, pain, nausea, syndromes; creatures have caste-specific attributes and personality
  facets. Genetics: appearance genes inherited from parents (simple blending/selection per feature).
- Scientific concept modeled: material-based injury, layered anatomy.
- Computational cost: notoriously CPU-bound at ~200 creatures.
- What we can learn: tissue-layer injury depth is expressive; syndrome system (effects from contaminants).
- What we could integrate: nothing (closed); design concepts only.
- What we should NOT integrate: per-layer anatomy for anything beyond the focused agent.
- Scalability: ~hundreds.

### A.2 Population genetics engines

### SLiM
- URL: https://github.com/MesserLab/SLiM
- License: GPL-3.0 — do not link/copy
- Language: C++ (own scripting language Eidos)
- Activity/maintenance: ~233 stars, ~4,500 commits, actively developed (SLiM 4/5 line)
- Architecture: forward-time individual-based simulator; genomes stored as sorted vectors of mutation
  pointers per haplosome; Wright-Fisher (WF) and non-Wright-Fisher (nonWF) model types; nonWF gives
  explicit age, overlapping generations, density-dependent fitness, and individual reproduction
  callbacks; continuous space with spatial interactions (k-d tree); tree-sequence recording via tskit.
- Scientific concept modeled: selection, drift, mutation, recombination, migration, QTL traits, spatial
  population genetics.
- Computational cost: ~microseconds per individual per generation for sparse mutations; scales with
  number of segregating mutations.
- What we can learn: nonWF design (individual-level fitness → survival, density regulation) is the model
  for our organisms; QTL approach for quantitative traits; tree-sequence recording for cheap genealogy.
- What we could integrate: algorithms only (GPL).
- What we should NOT integrate: Eidos, code.
- Scalability: 10k–1M individuals per generation feasible in SLiM itself (non-real-time).

### simuPOP
- URL: https://github.com/BoPeng/simuPOP
- License: GPL-2.0 — do not link/copy
- Language: C++ core, Python API
- Activity/maintenance: ~37 stars, ~4,700 commits, v1.1.18; low recent activity (unverified date)
- Architecture: populations of individuals with dense genotype arrays (binary/short/long allele types),
  operators applied pre/during/post mating (mutation, selection, migration, penetrance, quantitative
  traits); many built-in mating schemes (random, monogamous, polygamous, heteroMating).
- Scientific concept modeled: forward genetic epidemiology, disease penetrance, mating systems.
- Computational cost: dense arrays → O(loci) per individual per generation.
- What we can learn: "operator pipeline" structure and penetrance models (genotype → disease
  probability) — good for heritable disease risk.
- What we could integrate: algorithm only.
- What we should NOT integrate: code.
- Scalability: 1M individuals × few hundred loci OK offline.

### fwdpy11 (fwdpp)
- URL: https://github.com/molpopgen/fwdpy11
- License: GPL-3.0-or-later — do not link/copy
- Language: C++ with Python bindings
- Activity/maintenance: ~34 stars; maintained (dates unverified)
- Architecture: mutations stored once in a global array; genomes are vectors of mutation indices;
  recycling of extinct mutation slots; multivariate trait models with Gaussian stabilizing selection;
  tree sequence recording.
- Scientific concept modeled: quantitative traits under stabilizing selection, pleiotropy.
- Computational cost: efficient for many neutral mutations.
- What we can learn: "mutation table + index lists" memory layout; Gaussian stabilizing selection
  w = exp(-(z - opt)^2 / (2 VS)) as a fitness function for trait optima per biome.
- What we could integrate: algorithm only.
- What we should NOT integrate: code.
- Scalability: 1M offline.

### msprime / tskit
- URL: https://github.com/tskit-dev/msprime
- License: GPL-3.0 (msprime); tskit itself is MIT (unverified in this session)
- Language: Python + C
- Activity/maintenance: ~218 stars, active
- Architecture: backward-time coalescent with recombination producing tree sequences (succinct ARGs).
- Scientific concept modeled: ancestry / genealogies of samples.
- Computational cost: very cheap to generate ancestral diversity for a founding population.
- What we can learn: we can generate realistic standing genetic variation of "first humans/animals"
  without simulating their past; tree sequences compress millions of genealogies.
- What we could integrate: offline seeding tool idea; tskit format (if MIT confirmed) for lineage export.
- What we should NOT integrate: runtime coalescent.
- Scalability: millions of samples offline.

### A.3 Artificial life & evolution platforms

### Avida
- URL: https://github.com/devosoft/avida (project site https://avida.devosoft.org/)
- License: LGPL (per Wikipedia/project; license file not visible on fetch) — re-implement
- Language: C++
- Activity/maintenance: ~669 stars; legacy, slow
- Architecture: 2D grid of cells, each holding one digital organism = circular genome of assembly-like
  instructions run on a virtual CPU; CPU cycles ("merit") allocated proportionally to rewarded logic
  tasks performed (NOT, AND, EQU...); copy errors on replication = mutation; offspring placed in neighbor
  cell (replacing occupant).
- Scientific concept modeled: open-ended evolution of complex features, mutational robustness.
- Computational cost: tens of instructions per organism per update; ~10^4 organisms typical.
- What we can learn: resource-as-CPU-time metaphor; reward-for-tasks → selection pressure; demonstrates
  that evolution of complex traits needs many generations (≈10^4) — too slow for visible gameplay.
- What we could integrate: ideas only.
- What we should NOT integrate: program-genomes for creatures (illegible to the player).
- Scalability: 10k fine; 1M heavy.

### Tierra (mirror of v6.02)
- URL: https://github.com/acisternino/tierra
- License: not stated on page (unverified); original distributed by Tom Ray
- Language: C
- Activity/maintenance: archived Dec 2018, ~26 stars
- Architecture: "soup" of shared memory; organisms are self-replicating machine-code programs;
  templates (complementary nop patterns) for addressing; a "reaper" queue kills oldest/erroneous; time
  slicer gives CPU.
- Scientific concept modeled: emergent parasitism, hyper-parasitism, ecology without fitness function.
- Computational cost: per-instruction emulation; cheap per organism, but evolution is slow.
- What we can learn: reaper queue = simple density-dependent mortality; emergent parasites as an
  existence proof that interaction ecology emerges from resource scarcity.
- What we could integrate: concepts only.
- What we should NOT integrate: code.
- Scalability: thousands of programs.

### Salis v2
- URL: https://github.com/PaulTOliver/salis-v2
- License: not specified on page (unverified)
- Language: C + Python controller
- Activity/maintenance: ~15 stars, ~50 commits, quiet
- Architecture: Tierra-like, 32-instruction set, "seeker pointer" makes distant memory access cost time.
- Scientific concept modeled: locality cost in digital evolution.
- Computational cost: like Tierra.
- What we can learn: distance cost = spatial coherence; analogous to movement cost in our world.
- What we could integrate: concept only.
- What we should NOT integrate: code.
- Scalability: thousands.

### Polyworld
- URL: https://github.com/polyworld/polyworld
- License: Apple Public Source License 2.0 (LICENSE.txt) — weak copyleft, file-level; re-implement
- Language: C++ (by project knowledge; page did not show language)
- Activity/maintenance: ~212 stars; legacy
- Architecture: 3D flat world, agents with vision (1-D retina rendered from agent's viewpoint), neural
  network brains with Hebbian learning, genome encodes brain topology clusters + body size, speed,
  color; energy from food patches; behaviors eat/mate/fight/move/turn/light/focus; mating needs both
  agents choosing to mate and energy cost.
- Scientific concept modeled: evolution of neural complexity in an ecology.
- Computational cost: rendering vision per agent per tick = expensive; ~hundreds of agents.
- What we can learn: "mate" as a voluntary behavior with energy cost → sexual selection emerges;
  brain complexity measures.
- What we could integrate: ideas only.
- What we should NOT integrate: rendered vision for agents.
- Scalability: ~300 agents.

### Framsticks SDK
- URL: https://www.framsticks.com/sdk (SVN at https://www.framsticks.com/svn/framsticks/)
- License: SDK is LGPL (per site); full Framsticks app is freeware with its own license
- Language: C++ (Python/JS bindings exist)
- Activity/maintenance: long-running academic project; recent date unverified
- Architecture: multiple genetic encodings (f0 direct, f1 recurrent, f4 developmental, etc.) all
  converted to f0 phenotype (parts, joints, neurons); physics-based 3D bodies; similarity measures.
- Scientific concept modeled: genotype→phenotype mapping and its effect on evolvability.
- Computational cost: physics per body part per tick; tens to hundreds of creatures.
- What we can learn: separating "genetic encoding" from "phenotype description" — we should also keep
  a phenotype record derived once at birth (cache), not re-decode the genome every tick.
- What we could integrate: concepts.
- What we should NOT integrate: physical morphology evolution for humans/animals (too costly, illegible).
- Scalability: ~100.

### ALIEN (Artificial Life Environment)
- URL: https://github.com/chrxh/alien
- License: BSD-3-Clause — OK
- Language: CUDA / C++
- Activity/maintenance: ~5.5k stars, ~9k commits, very active
- Architecture: GPU particle engine; organisms are networks of cells (particles) connected by bonds
  (springs); cell types (sensor, muscle, neuron, constructor, digestor, attacker, transmitter);
  genome executed by constructor cells to build offspring cell by cell; energy is conserved and
  transferred between cells; everything runs in CUDA kernels.
- Scientific concept modeled: open-ended evolution of multicellular soft-bodied creatures.
- Computational cost: millions of particles in real time on RTX-class GPU.
- What we can learn: GPU struct-of-arrays design, energy conservation as hard rule, "construction"
  genomes; proof that 1M simple entities on GPU is feasible.
- What we could integrate: BSD so legally fine — but a particle-physics organism model does not fit
  humans/animals. Possibly use for a "microbe layer" visualization or ponds.
- What we should NOT integrate: as the main creature model.
- Scalability: 1M particles yes (GPU); organisms ≈ 10k–100k.

### Lenia
- URL: https://github.com/Chakazul/Lenia
- License: MIT (shown on repo page) — OK
- Language: Python (also JS/Matlab versions)
- Activity/maintenance: ~3.9k stars, ~174 commits; reference implementation, low activity
- Architecture: continuous cellular automaton; state A(x) ∈ [0,1]; update
  A ← clip(A + dt * G(K * A)), K = ring kernel, G = Gaussian growth function; FFT convolution.
- Scientific concept modeled: emergent self-organizing "creatures" in continuous CA.
- Computational cost: O(N log N) per frame for N cells via FFT.
- What we can learn: field-based "life" for microbes/slime molds/plankton as a visual biome layer.
- What we could integrate: MIT; could power a decorative-but-systemic microbial mat.
- What we should NOT integrate: as macro-organism model.
- Scalability: 1M cells per frame on GPU.

### Flow-Lenia (experiment code)
- URL: https://github.com/tsua0002/FlowLenia-experiment (paper arXiv:2212.07906, 2506.08569)
- License: unverified
- Language: Python (JAX)
- Activity/maintenance: ALIFE 2025 companion code; activity unverified
- Architecture: Lenia with mass conservation: matter flows along the gradient of an affinity map;
  update-rule parameters are themselves local fields that are transported with mass → multi-species.
- Scientific concept modeled: open-ended evolution in CA via localized parameters.
- Computational cost: GPU, similar to Lenia.
- What we can learn: "parameters travel with mass" = cheap genetic inheritance in a field model;
  mass conservation = no free biomass.
- What we could integrate: idea for microbe/algae biomass layer.
- What we should NOT integrate: code until license confirmed.
- Scalability: 1M cells GPU.

### biosim4
- URL: https://github.com/davidrmiller/biosim4
- License: MIT — OK
- Language: C++ (Python tools)
- Activity/maintenance: ~3.4k stars; maintenance-only
- Architecture: 2D grid, individuals ("Indiv") with a genome of N genes; each gene is a 32-bit
  connection (source type/id, sink type/id, weight); genome → small neural net (sensors → internal
  neurons → actions); generations with selection by a "challenge" region; pheromone signal layer;
  mutation by bit-flip, sexual reproduction optional.
- Scientific concept modeled: evolution of behavior via genome-encoded neural wiring.
- Computational cost: ~tens of neurons per indiv per step; ~1000s of indivs at high speed.
- What we can learn: compact "gene = synapse" encoding is very cheap; good template for insect/fish
  tier brains that evolve.
- What we could integrate: MIT; encoding idea or even code for low-tier animal brains.
- What we should NOT integrate: generation-synchronous selection (our world is continuous).
- Scalability: 10k fast; 100k OK; 1M with SIMD/GPU.

### Evolving-Protozoa (ProtoEvo)
- URL: https://github.com/DylanCope/Evolving-Protozoa
- License: MIT — OK
- Language: Java
- Activity/maintenance: ~275 stars, ~107 commits
- Architecture: 2D physics; cells with evolvable morphology, vision, adhesion (multicellularity),
  NEAT-like brains; herbivore vs predator strategies emerge; chemical/light fields.
- Scientific concept modeled: emergent trophic strategies.
- Computational cost: physics + small NN per cell; thousands.
- What we can learn: emergence of predation from same genome space; plant-like cells from light.
- What we could integrate: MIT, ideas.
- What we should NOT integrate: physics-based bodies.
- Scalability: ~thousands.

### Critterding2
- URL: https://github.com/bobke/Critterding2 (original: https://github.com/bobke/Critterding)
- License: GPL-3.0 — do not copy
- Language: C++
- Activity/maintenance: ~6 stars (v2), ~113 commits
- Architecture: 3D physics creatures, random spiking-like neural brains, energy from food, entity-component.
- Scientific concept modeled: brain-body co-evolution in a petri dish.
- Computational cost: physics heavy; ~hundreds.
- What we can learn: energy-closed petri dish (fixed total energy) prevents runaway populations.
- What we could integrate: nothing (GPL).
- What we should NOT integrate: code.
- Scalability: hundreds.

### Thrive
- URL: https://github.com/Revolutionary-Games/Thrive
- License: GPL-3.0 (code); assets under separate terms (unverified)
- Language: C# (Godot)
- Activity/maintenance: ~3.7k stars, ~10.5k commits, active
- Architecture: player species in a microbe stage; NPC species evolve via "auto-evo": an abstract
  per-patch population simulation that each generation generates mutated candidate species, scores
  them against niches (energy sources per patch, predation), and updates populations — evolution is
  computed statistically, not by simulating individuals.
- Scientific concept modeled: speciation and niche filling at species level.
- Computational cost: per species per patch per generation — very cheap.
- What we can learn: auto-evo is the exact pattern we need for fast-forward evolution: species-level
  mutation proposals scored by an energy-budget niche model. (Mechanics described from project
  knowledge; the repo page did not show the details — verify in Thrive wiki before relying on specifics.)
- What we could integrate: algorithm idea only (GPL).
- What we should NOT integrate: code/assets.
- Scalability: thousands of species × hundreds of patches.

### Evolution (Keiwan Donyagard)
- URL: https://github.com/keiwando/evolution
- License: custom, not open source (no redistribution/selling) — reference only
- Language: C# (Unity)
- Activity/maintenance: ~455 stars
- Architecture: Karl-Sims-style creatures (joints, bones, muscles) with NN controllers evolved by GA
  per generation for tasks (running, jumping).
- Scientific concept modeled: locomotion evolution.
- Computational cost: physics per creature; batch of tens per generation.
- What we can learn: the player appeal of watching "weird gaits" evolve; not for our ecology.
- What we could integrate: nothing.
- What we should NOT integrate: anything.
- Scalability: tens.

### neat-python
- URL: https://github.com/CodeReclaimers/neat-python
- License: BSD-3-Clause — OK
- Language: Python
- Activity/maintenance: ~1.6k stars, v2.1.0, maintained
- Architecture: NEAT: genome = node genes + connection genes with historical innovation numbers;
  crossover aligns by innovation number; speciation by compatibility distance
  δ = c1·E/N + c2·D/N + c3·W̄; fitness sharing within species; structural mutations add node/connection.
- Scientific concept modeled: neuroevolution with topology growth; speciation to protect innovation.
- Computational cost: network eval O(connections); evolution step per generation.
- What we can learn: compatibility distance is a ready-made speciation metric for any genome; we can
  reuse it to define species of animals automatically.
- What we could integrate: BSD — ideas or ported code; but NEAT is generation-batch oriented.
- What we should NOT integrate: Python runtime in the sim loop.
- Scalability: 1k networks per generation easy.

### SharpNEAT
- URL: https://github.com/colgreen/sharpneat
- License: MIT (LICENSE.txt) — OK
- Language: C# (.NET 9)
- Activity/maintenance: ~424 stars, maintained
- Architecture: high-performance NEAT; acyclic and cyclic network activation; vectorized activation
  functions; speciation via k-means on genome vectors.
- Scientific concept modeled: neuroevolution.
- Computational cost: optimized CPU neural evaluation.
- What we can learn: fast activation of evolved sparse networks (sorted connection arrays).
- What we could integrate: MIT — usable directly if our engine is C#.
- What we should NOT integrate: generation-batch GA loop for continuous-time agents.
- Scalability: 10k networks per generation.

### TensorNEAT
- URL: https://github.com/EMI-Group/tensorneat
- License: GPL-3.0 — do not copy
- Language: Python (JAX)
- Activity/maintenance: ~400 stars, active
- Architecture: pads NEAT genomes into fixed-size tensors so heterogeneous topologies are evaluated in
  parallel on GPU; claims ~500× speedup.
- Scientific concept modeled: GPU neuroevolution.
- Computational cost: batched GPU.
- What we can learn: padding-to-fixed-size is the trick for batch-evaluating 100k animal brains.
- What we could integrate: idea only.
- What we should NOT integrate: code.
- Scalability: 100k+ networks on GPU.

### A.4 Animals, predator-prey and agent-based ecology

### NetLogo (incl. Wolf Sheep Predation model)
- URL: https://github.com/NetLogo/NetLogo
- License: GPL-2.0 (platform); Models Library sample models carry per-model licenses (many CC BY-NC-SA — check each)
- Language: Scala/Java
- Activity/maintenance: ~1.2k stars, ~8.5k commits, active
- Architecture: turtles (agents), patches (grid cells), links. Wolf Sheep Predation: sheep move randomly
  and lose 1 energy/tick, gain energy eating grass; grass patches regrow after countdown; wolves eat
  sheep; reproduce with probability p, splitting energy with offspring; die at energy ≤ 0.
- Scientific concept modeled: agent-based Lotka–Volterra with explicit resource; oscillations and
  extinction; stability needs grass dynamics.
- Computational cost: few ops per agent per tick.
- What we can learn: the "energy split on birth" rule and grass regrowth timer are the minimal
  viable ecosystem; without resource limitation the system goes extinct quickly.
- What we could integrate: rules (algorithms aren't copyrightable); not the code.
- What we should NOT integrate: GPL platform.
- Scalability: 10k turtles fine; 1M no.

### Mesa
- URL: https://github.com/projectmesa/mesa
- License: Apache-2.0 — OK
- Language: Python
- Activity/maintenance: ~3.9k stars, Mesa 3 stable, Mesa 4 pre-releases
- Architecture: Agent/Model classes, grid & continuous spaces, schedulers/activation, data collectors;
  example models include Wolf-Sheep, Boids flockers, Sugarscape.
- Scientific concept modeled: generic ABM.
- Computational cost: Python object per agent; ~µs–ms per agent per step.
- What we can learn: prototyping test bed for our ecology rules before engine implementation; Sugarscape
  carrying capacity emerging from resource regrowth.
- What we could integrate: use as offline prototyping tool (Apache).
- What we should NOT integrate: as runtime.
- Scalability: 10k slow; prototyping only.

### Sebastian Lague — Ecosystem-2
- URL: https://github.com/SebLague/Ecosystem-2
- License: not specified on page — treat as all rights reserved
- Language: C# (Unity) (unverified on page)
- Activity/maintenance: ~538 stars, ~25 commits, WIP
- Architecture: rabbits/foxes/plants on a tile world; needs (hunger, thirst, mate urge) select
  actions; genes for speed/sensing; simple pathfinding.
- Scientific concept modeled: needs-driven animal AI with heritable traits.
- Computational cost: low; hundreds of animals.
- What we can learn: needs-urgency action selection is legible to players.
- What we could integrate: nothing (no license).
- What we should NOT integrate: code.
- Scalability: hundreds–thousands.

### Preylife
- URL: https://github.com/codevalley/preylife
- License: BSD-2-Clause — OK
- Language: TypeScript (Three.js)
- Activity/maintenance: ~6 stars, ~68 commits
- Architecture: ECS with spatial partitioning; prey/predator with four evolving attributes (strength,
  stealth, learnability, longevity); seasonal resource blooms every 90 days; social learning.
- Scientific concept modeled: evolutionary arms race with trade-offs.
- Computational cost: hundreds of entities real time.
- What we can learn: trade-off design — each trait must cost something (longevity vs fecundity,
  stealth vs speed) or evolution maxes everything.
- What we could integrate: BSD; ideas.
- What we should NOT integrate: n/a.
- Scalability: hundreds–low thousands.

### Modular Megafauna Model
- URL: https://github.com/wtraylor/modular_megafauna_model
- License: LGPL-3.0-or-later — re-implement
- Language: C++
- Activity/maintenance: ~3 stars, ~1,300 commits
- Architecture: daily herbivore cohorts (not individuals) per habitat cell: forage intake limited by
  digestion and forage availability → energy budget → fat reserves → body condition → reproduction
  and mortality; designed to plug into a dynamic vegetation model.
- Scientific concept modeled: process-based large herbivore population dynamics.
- Computational cost: per cohort per cell per day — very cheap.
- What we can learn: cohort-per-cell aggregation with body-condition state is exactly our
  aggregate-mode animal representation.
- What we could integrate: algorithm (LGPL code avoided).
- What we should NOT integrate: code statically linked.
- Scalability: millions of animals represented by thousands of cohorts.

### A.5 Plants & forests

### iLand
- URL: https://github.com/edfm-tum/iland-model
- License: GPL-3.0 — algorithm only
- Language: C++ (Qt)
- Activity/maintenance: ~30 stars, active academic
- Architecture: individual trees on a 2 m light grid; each tree stamps a precomputed "Light Influence
  Field" (LIF) pattern by species/size onto the grid; tree's light resource = reading of the grid
  under its crown; 3-PG-style production at 100 m resource units; allocation; mortality = intrinsic +
  stress; regeneration on 2 m cells; disturbances modules (fire, wind, bark beetle).
- Scientific concept modeled: landscape-scale individual-tree competition and succession.
- Computational cost: stamping is O(stamp area) per tree per year; millions of trees on clusters/desktop.
- What we can learn: the stamp method is the cheapest realistic light competition; resource unit
  aggregation (fine grid for competition, coarse grid for productivity).
- What we could integrate: algorithm idea (GPL code avoided).
- What we should NOT integrate: code.
- Scalability: 1M trees per simulated year feasible.

### SORTIE-ND (Core Model)
- URL: https://github.com/LMurphy186232/Core_Model (site sortie-nd.org)
- License: not shown on GitHub page (unverified; historically GPL)
- Language: C++
- Activity/maintenance: ~6 stars, ~114 commits
- Architecture: spatially explicit individual trees; neighborhood competition index
  NCI = Σ (DBH_j^α / dist_ij^β) over neighbors; growth = max growth × size effect × crowding effect;
  light via GLI fisheye; seed dispersal by Weibull/lognormal kernels; behaviors are pluggable modules.
- Scientific concept modeled: neighborhood dynamics of mixed forests.
- Computational cost: O(neighbors) per tree per timestep (year).
- What we can learn: NCI formula is a one-line competition rule usable per plant agent.
- What we could integrate: formula only.
- What we should NOT integrate: code.
- Scalability: 100k trees per year fine.

### LPJ-GUESS
- URL: https://github.com/LPJ-GUESS (organization; main model code via Lund University / Zenodo release v4.1.1)
- License: MPL-2.0 (per search results) — file-level copyleft, usable with care
- Language: C++
- Activity/maintenance: maintained by Lund University; public repo access partial (unverified)
- Architecture: plant functional types (PFTs) with bioclimatic limits; "gap model" patches with
  cohorts of individuals; daily photosynthesis/water balance, annual allocation, establishment,
  mortality, disturbance; soil carbon/nitrogen pools (CENTURY-style).
- Scientific concept modeled: global vegetation dynamics + biogeochemistry.
- Computational cost: per patch per day; fine for a grid of thousands of cells.
- What we can learn: PFT abstraction (we need ~8–12 plant types, not species); bioclimatic envelope
  for where plants can establish; litter → soil pools.
- What we could integrate: concepts.
- What we should NOT integrate: full biogeochemistry.
- Scalability: cell-based; unlimited plant count.

### L-Py (OpenAlea)
- URL: https://github.com/openalea/lpy
- License: CeCILL (GPL-compatible) — reference only
- Language: Python / C++
- Activity/maintenance: ~99 stars, ~924 commits, CI active
- Architecture: L-systems (parallel rewriting of module strings) with Python productions; turtle
  interpretation to 3D geometry.
- Scientific concept modeled: plant architecture growth.
- Computational cost: per module per derivation step.
- What we can learn: visual-only plant shapes can be generated from 3–4 numbers (age, vigor, species
  rule) — keep simulation state tiny and generate geometry from it.
- What we could integrate: our own small L-system for visuals.
- What we should NOT integrate: code.
- Scalability: visual LOD only.

### FORMIND (no public repo found)
- URL: http://www.formind.org (unverified; no open source repository located)
- License: unverified (available on request, historically)
- Language: C++ (unverified)
- Activity/maintenance: academic (UFZ Leipzig)
- Architecture: 20×20 m patches, trees in height layers, light by Lambert–Beer through crown layers
  I(z) = I0·exp(-k·LAI_above(z)), growth from photosynthesis − respiration, plant functional types.
- Scientific concept modeled: tropical forest gap dynamics.
- Computational cost: per patch per year cheap.
- What we can learn: patch + height-layer light is cheaper than iLand stamps for our coarser grid.
- What we could integrate: formula only.
- What we should NOT integrate: n/a.
- Scalability: large regions.

### A.6 Disease / epidemiology

### Covasim
- URL: https://github.com/InstituteforDiseaseModeling/covasim (now under starsimhub)
- License: MIT — OK
- Language: Python (NumPy/Numba)
- Activity/maintenance: ~290 stars, ~5,400 commits; mature, folded into Starsim
- Architecture: People stored as struct-of-arrays (age, sex, state booleans, dates of each transition);
  contact layers (household, school, work, community) as edge lists; each day: per-edge transmission
  probability = beta × layer weight × viral load factor × susceptibility; durations drawn from
  lognormal distributions at infection time (so future transitions are scheduled, not polled).
- Scientific concept modeled: stochastic ABM SEIR with symptoms, severity, death by age.
- Computational cost: ~µs per agent per day vectorized; 100k agents in seconds per year.
- What we can learn: "sample all future dates at infection" trick — no per-tick state checks; age-specific
  severity/death tables; layered contact networks.
- What we could integrate: MIT — can port.
- What we should NOT integrate: COVID-specific parameters.
- Scalability: 1M agents feasible offline; for game, 100k observed agents ok.

### Starsim
- URL: https://github.com/starsimhub/starsim
- License: MIT — OK
- Language: Python
- Activity/maintenance: ~44 stars, ~4,500 commits, active
- Architecture: modular: People (array states), Networks (static/dynamic, e.g., MF sexual, random),
  Diseases (SIR, SIS, HIV, etc.), Demographics (births, deaths, pregnancy), Interventions; integration
  loop with explicit module ordering; supports agent-based, metapopulation and compartmental levels.
- Scientific concept modeled: co-transmitting diseases + demography + pregnancy.
- Computational cost: vectorized per-agent arrays.
- What we can learn: the module ordering (demographics → networks → disease transmission → disease
  progression → deaths) and that pregnancy/birth are just another module; multi-disease interactions.
- What we could integrate: MIT — architecture reference and possibly offline calibration.
- What we should NOT integrate: Python in the loop.
- Scalability: 1M agents (slow-ish); designed for scaling via levels.

### EMOD (IDM)
- URL: https://github.com/InstituteforDiseaseModeling/EMOD
- License: MIT — OK
- Language: C++
- Activity/maintenance: ~107 stars; officially no longer maintained by IDM
- Architecture: nodes (spatial units) with individuals; "individual sampling" — each simulated agent can
  represent many people (Monte Carlo weights); transmission via a node-level contagion pool (agents shed
  into a pool, others draw exposure from it) instead of explicit contacts; migration between nodes;
  vector (mosquito) and malaria within-host models; intrahost immunity.
- Scientific concept modeled: malaria, TB, HIV, polio; vector-borne and zoonotic dynamics.
- Computational cost: per agent per day; pool transmission is O(N) not O(N·contacts).
- What we can learn: (1) contagion pool per location = cheap transmission; (2) agent sampling weights
  = bridge between individual and aggregate; (3) vector-borne model structure for zoonoses.
- What we could integrate: MIT; algorithms and possibly code.
- What we should NOT integrate: malaria-detail within-host model.
- Scalability: millions of represented people via sampling.

### OpenABM-Covid19
- URL: https://github.com/BDI-pathogens/OpenABM-Covid19
- License: GPL-3.0 — algorithm only
- Language: C (Python/R bindings)
- Activity/maintenance: ~130 stars, ~1,320 commits; quiet
- Architecture: households + daily-rebuilt workplace/random networks; disease state machine with
  scheduled transitions; contact tracing app model.
- Scientific concept modeled: network transmission & tracing.
- Computational cost: C-efficient; 1M agents designed for.
- What we can learn: rebuild random networks daily (cheap) while keeping household network static.
- What we could integrate: no code.
- What we should NOT integrate: code.
- Scalability: 1M.

### Epiabm
- URL: https://github.com/SABS-R3-Epidemiology/epiabm
- License: BSD-3-Clause — OK
- Language: Python + C++ backend
- Activity/maintenance: ~21 stars, ~2,600 commits
- Architecture: re-implementation of the Ferguson CovidSim model: population → cells → microcells →
  households → persons; spatial infection kernel between cells; place-based infection (households,
  workplaces); age-dependent progression.
- Scientific concept modeled: spatial hierarchical SEIR.
- Computational cost: hierarchical aggregation makes spatial transmission O(cells²) not O(people²).
- What we can learn: cell/microcell hierarchy matches our map tiles; spatial kernel for between-village spread.
- What we could integrate: BSD — algorithm and code allowed.
- What we should NOT integrate: n/a.
- Scalability: 100k–1M.

### FRED
- URL: https://github.com/PublicHealthDynamicsLab/FRED
- License: custom EULA, non-commercial — reference only
- Language: C++
- Activity/maintenance: ~83 stars, release FRED-v5.7.0
- Architecture: synthetic populations of US census agents with places (homes, schools, workplaces);
  FRED language for conditions/state machines with rules.
- Scientific concept modeled: place-based ABM with behavior rules.
- Computational cost: per agent per day.
- What we can learn: "conditions as state machines" DSL — unified model for disease, behavior, addiction.
- What we could integrate: nothing (NC).
- What we should NOT integrate: code.
- Scalability: millions.

### Epydemix
- URL: https://github.com/epistorm/epydemix
- License: GPL-3.0 — algorithm only
- Language: Python
- Activity/maintenance: ~70 stars
- Architecture: compartmental models with arbitrary transitions, age-structured contact matrices
  (home/school/work/community layers), stochastic chain-binomial simulation, ABC calibration.
- Scientific concept modeled: age-structured stochastic compartmental epidemics.
- Computational cost: O(compartments × age groups) per step — independent of population size.
- What we can learn: chain-binomial with age contact matrices is the fast-forward disease engine.
- What we could integrate: algorithm only.
- What we should NOT integrate: code.
- Scalability: any population size.

### EpiModel
- URL: https://github.com/EpiModel/EpiModel
- License: GPL-3.0 — algorithm only
- Language: R
- Activity/maintenance: ~281 stars, ~3,900 commits, active
- Architecture: three model classes (deterministic compartmental ODE, stochastic individual contact,
  stochastic network via ERGMs/statnet).
- Scientific concept modeled: same disease simulated at three fidelity levels — directly analogous to LOD.
- Computational cost: from trivial (DCM) to heavy (network).
- What we can learn: keep a single parameterization valid across levels; validate the ABM against the ODE.
- What we could integrate: idea.
- What we should NOT integrate: code.
- Scalability: DCM unlimited.

### LASER (IDM)
- URL: https://github.com/InstituteforDiseaseModeling/laser
- License: MIT — OK
- Language: Python (NumPy/Numba, optional GPU)
- Activity/maintenance: ~4 stars, ~227 commits, active
- Architecture: struct-of-arrays agents; communities can be full ABM or "eXtremely Light Agents"
  (stochastic compartmental) with spatial coupling — hybrid ABM/compartmental by design.
- Scientific concept modeled: large spatial epidemics (measles, polio).
- Computational cost: tens of millions of agents on laptop per their aims.
- What we can learn: the strongest reference for our hybrid disease LOD design.
- What we could integrate: MIT; architecture and code ideas.
- What we should NOT integrate: n/a.
- Scalability: 1M–100M.

---

## Part B — Algorithm & science references (no repo needed; re-implement from literature)

| Topic | Model | Equation / rule (our paraphrase) | Use |
|---|---|---|---|
| BMR | Mifflin–St Jeor | BMR = 10·W(kg) + 6.25·H(cm) − 5·age + 5 (male) or −161 (female) kcal/day | daily energy need |
| TEE | PAL multiplier | TEE = BMR × PAL (1.4 sedentary, 1.7 active, 2.0+ heavy labor) | activity cost |
| Weight | Hall 2-compartment | ΔF, ΔL partitioned by Forbes p = 10.4/(10.4+F) | fat/lean dynamics |
| Growth | WHO standards (0–5 y), Preece–Baines model 1 | h(t) = h1 − 2(h1 − hθ) / (exp(s0(t−θ)) + exp(s1(t−θ))) | stature curve with adolescent spurt |
| Aging mortality | Gompertz–Makeham | μ(x) = A + B·exp(γx), human γ ≈ 0.08–0.10/yr (doubling ~7–8 y) | adult death hazard |
| Full-life mortality | Siler | μ(x) = a1·exp(−b1·x) + a2 + a3·exp(b3·x) | infant + background + senescence |
| Life tables | Coale–Demeny / Brass logit | logit(l_x) = α + β·logit(l_x^standard) | 2-parameter regional mortality |
| Sleep | Borbély two-process | S rises toward 1 while awake (τ ≈ 18 h), decays while asleep (τ ≈ 4 h); circadian C sinusoid; sleep when S > upper threshold H0 + a·C | fatigue |
| Wound healing | 4 phases | hemostasis (hours) → inflammation (days) → proliferation (1–3 wk) → remodeling (months); infection risk per day in inflammation phase | injuries |
| Within-host | target-cell limited | dT/dt = −βTV, dI/dt = βTV − δI, dV/dt = pI − cV | (abstracted to pathogen load + immunity) |
| Selection response | Breeder's equation | R = h²·S | fast-forward trait means |
| Multi-trait | Lande equation | Δz̄ = G·β | correlated responses |
| Drift | Wright–Fisher | p' ~ Binomial(2Ne, p)/(2Ne) | aggregate allele frequencies |
| Predator-prey | Lotka–Volterra / Rosenzweig–MacArthur | dN/dt = rN(1−N/K) − aNP/(1+ahN); dP/dt = e·aNP/(1+ahN) − mP | aggregate animals |
| Foraging | Marginal value theorem | leave patch when instantaneous intake rate falls to habitat average rate | animal patch leaving |
| Flocking | Boids (Reynolds 1987) | separation + alignment + cohesion steering within radius | fish/birds/herds |
| Motivation | Lorenz hydraulic | action-specific drive accumulates over time; released by stimulus strength + drive > threshold | animal drives |
| Behavior hierarchy | Tinbergen | instinct centers → appetitive → consummatory acts | behavior tree layout |
| Dominance | winner/loser effects (Hemelrijk DomWorld) | after fight, winner's rank score += k(1 − P_win), loser −= same (Elo-like) | pack hierarchy |
| Grid ecosystem | Wa-Tor (Dewdney 1984) | fish breed after n ticks; sharks eat fish, starve after m ticks | test bed |
| Light | Beer–Lambert | I = I0·exp(−k·LAI_above) | plant competition |
| Decomposition | Olson | dM/dt = −k·M, k by litter type & temperature/moisture | nutrient cycle |
| Epidemic | SEIR / chain-binomial | new infections ~ Binomial(S, 1 − exp(−β·I/N·dt)) | aggregate disease |
| Fertility | Bongaarts proximate determinants | TFR = TF × C_marriage × C_contraception × C_abortion × C_postpartum-infecundity | demographic calibration |
| Fecundability | age curve | monthly conception prob ≈ 0.20–0.25 peak (20–30 y), ~0.1 at 38, ~0 at 45–50; menopause mean ~50 | conception |
| Life history | r/K, Charnov invariants | age at maturity × adult mortality ≈ const; litter size vs offspring size trade-off | species templates |

---

## Part C — Recommended design for our game

### C.1 Core principles
1. **One parameterization, several fidelities.** Every biological process has an individual rule and an
   aggregate rule derived from the same parameters (e.g., the per-contact transmission β also defines the
   compartmental β). Validate by running both in tests and comparing means.
2. **Schedule, don't poll.** At birth, infection, pregnancy, injury: sample the future event dates
   (maturity day, symptom day, due date, healing day, death-from-senescence day) and push them into a
   priority queue/calendar. Per-tick work is only for things that truly change continuously.
3. **Conservation.** Energy (biomass) and organism counts are conserved across LOD transitions;
   nutrients are conserved in the nutrient cycle (except explicit inputs: weathering, rain N deposition).
4. **Trade-offs everywhere.** Every heritable trait has a cost so evolution is visible and non-trivial.
5. **Legibility.** Genes map to named traits the god can inspect.

### C.2 Genome representation

**Layout (humans and all animals share the format, species differ in trait maps):**
- Diploid. 2 haplotypes × 128 biallelic loci = 2 × 128 bits = **32 bytes per organism**.
- Loci grouped onto **8 chromosomes × 16 loci**. Recombination: per chromosome, pick 0–2 crossover points
  (Poisson mean 1) — implemented as bitmask splice, a few ns.
- Plus a per-species **lineage ID** (u32) and a **mitochondrial/maternal marker** (u16) for ancestry views.
- Optional 16 "Mendelian" loci inside the 128 with dominance flags (recessive disease, albinism, eye
  color major gene, lactase persistence, sickle-like heterozygote advantage).

**Trait mapping (example for humans; ~20 traits):**
- Each trait k uses a fixed list of 6–12 loci (some shared → pleiotropy). Additive value
  A_k = Σ_i w_ki · (g_i − 2p0_i), where g_i ∈ {0,1,2} = count of "+" alleles, p0_i = founder frequency.
- Phenotype z_k = μ_k + σ_A,k · A_k / norm_k + E_k, with environmental deviation E_k ~ N(0, σ_E,k²) drawn
  once at birth (+ developmental modifiers like childhood nutrition for height). Heritability is then
  h² = σ_A²/(σ_A²+σ_E²) — set per trait to realistic values: height 0.8, BMR/metabolic efficiency 0.4,
  temperament axes 0.4, cognitive potential 0.5, disease resistance 0.3, longevity 0.25, fertility 0.2.
- Trait list (human): height potential, build/muscularity, fat-storage propensity, metabolic efficiency,
  skin pigmentation, hair color, eye color (Mendelian-ish), face-shape morph (3 loci sets → blend shapes),
  immune strength, specific resistance to 4 disease families, senescence rate (Gompertz B multiplier),
  fecundity, 5 temperament axes (Big-Five-like: novelty seeking, sociability, conscientiousness,
  aggression, anxiety), cognitive potential (caps learning rate), cold/heat tolerance.
- Animals: same 128 loci; species template defines ~12 traits: size, speed, sense range, aggression,
  boldness, litter size, maturity age, longevity, coat color, cold tolerance, diet breadth, sociality.
- Pleiotropic trade-offs (built into weights): size↑ ⇒ maturity age↑ and energy need↑;
  fecundity↑ ⇒ senescence rate↑; immune strength↑ ⇒ BMR↑; aggression↑ ⇒ injury risk↑.

**Mutation:** per-locus flip probability **μ = 2×10⁻⁴ per generation** by default (≈ 0.05 new
mutations per gamete over 128 loci); god slider ×0 to ×100. Biologically this is high, but it compensates
for few loci and few generations so populations visibly shift within ~20–50 generations. Rare
"macro-mutation" event (1e-4/birth) re-rolls a whole trait's weight vector within a lineage = novelty.

**Visible change cheaply:** selection acts on phenotypes through survival/mating; with h²≈0.5 and
truncation-like selection of S = 0.5 σ, mean moves 0.25 σ/generation (breeder's equation) — for animals
with 1-year generations, visible within decades of game time; at 1000× fast-forward, within minutes.

**Speciation:** genetic distance = Hamming distance between population consensus genomes + trait distance;
when two subpopulations' distance exceeds threshold D* (and they are geographically separated ≥ N
generations), reduce interbreeding fertility (hybrid viability multiplier e^(−d/d0)) and assign a new
species ID — NEAT-style compatibility clustering run once per in-game year on population summaries.

### C.3 Body model (humans; animals use a reduced set)

State variables per individual (updated once per game day; focused agents may update hourly):

| Variable | Unit | Update per day |
|---|---|---|
| age | days | +1 |
| height | cm | follows genotype-scaled Preece–Baines curve × nutrition factor (stunting if energy deficit during growth: growth velocity × (1 − 0.5·deficit_ratio)) |
| fat mass F | kg | F += (1−p)·(EI − EE)/9400 |
| lean mass L | kg | L += p·(EI − EE)/1800 (p = 10.4/(10.4+F)); plus training stimulus, minus disuse |
| energy store (short-term glycogen/stomach) | kcal | intra-day buffer for hunger signal |
| hydration | 0..1 | − (loss by temperature/work) + drink; death at ~3–5 days without water |
| fatigue S | 0..1 | Borbély process S; circadian C from clock |
| health / HP (systemic) | 0..1 | aggregates injuries + disease + starvation |
| injuries | list ≤ 4 | each: body region (6 regions), severity, phase, infection flag, healing date |
| pathogen loads | ≤ 3 active | per infection: load, phase dates (sampled), immunity to that family |
| immunity vector | 8 floats | per disease family; waning e^(−t/τ_w) |
| fertility state | enum | prepubertal / cycling / pregnant / postpartum-amenorrhea / menopausal |
| frailty | 0..1 | individual multiplier on Gompertz hazard (genetic senescence rate × accumulated damage) |
| stress / pain | 0..1 | feeds mood/cognition track |

Daily equations:
- EE = BMR(Mifflin with current W=F+L, H, age, sex) × PAL(activity) × thermal factor + pregnancy
  (+300 kcal/day in T2–T3) + lactation (+500) + growth cost (children) + fever (+10% per °C).
- EI = food eaten (kcal), limited by availability and stomach capacity.
- Starvation: if F < essential fat (3 kg men / 10 kg women), lean mass is catabolized faster;
  death hazard rises sharply when BMI < 13.
- Hazard of death today: h = Siler/Gompertz–Makeham baseline(age) × frailty × (1 + disease risk +
  injury risk + starvation risk + exposure risk). Death if U < 1 − exp(−h·Δt). For background agents,
  the senescence part is pre-sampled as a death date at birth (inverse-CDF of Gompertz), adjusted when
  frailty changes materially.
- Injury healing: severity_t = severity_0 · exp(−t/τ_heal), τ_heal ∝ severity / (nutrition · age factor);
  daily infection probability during inflammation phase depends on hygiene/medicine; infected wound
  converts into a disease episode (sepsis) with mortality.
- Calibration targets (pre-modern default): life expectancy at birth ~30–35, e15 ≈ +35–40 years,
  infant mortality ~20–30%, maternal death ~1% per birth, TFR 5–7. God-controlled "medicine/hygiene"
  modifiers move these toward modern values (e0 ~80, infant mortality <1%).

Animals: age, mass (single compartment: dM/dt = (intake − maintenance)/ρ, maintenance = a·M^0.75
Kleiber), energy reserve, hydration (optional), injuries (count+severity only), pathogen load,
reproductive state, frailty. Insects/fish in aggregate only, except observed individuals.

### C.4 Reproduction pipeline
1. **Maturity:** puberty age sampled at birth (human girls 12–14 menarche, adolescent sterility ~2 y;
   boys 13–15), shifted by nutrition. Animals: species maturity age × genetic modifier.
2. **Mate search & choice:** candidate set from social network / spatial neighborhood (≤ 20 candidates).
   Score = Σ preference weights × partner traits (health cues, status, similarity in temperament,
   kinship penalty, age compatibility) + noise. Preferences are partly heritable (loci) and partly
   culturally learned. Mutual choice: pair forms if both scores exceed their aspiration level; aspiration
   decays with time single (sequential search model).
3. **Pair bonding:** bond strength variable grows with time together and shared children, decays with
   conflict/absence; dissolution hazard ∝ (1 − bond). Animal species flags: monogamous / harem /
   promiscuous / lek, plus parental care type (none / female / biparental / cooperative).
4. **Conception:** per cycle (monthly tick for humans) P = fecundability(age_f) × male factor(age_m) ×
   coital frequency factor × energy-balance factor (low fat suppresses ovulation) × genetics; incest
   increases recessive disease expression automatically through genotype.
5. **Pregnancy:** due date sampled (~280 ± 10 days); miscarriage hazard front-loaded (~15% total);
   energy cost; childbirth risk event: maternal death prob baseline 0.5–1.5% × (age<17 or >35 factor) ×
   (parity first-birth factor) × medicine; stillbirth ~3–5% pre-modern; twins ~1.2%.
6. **Offspring genome:** each parent contributes one recombined gamete; mutation applied; phenotype
   computed and cached; environmental deviations drawn.
7. **Birth spacing:** postpartum amenorrhea while breastfeeding (6–24 months; duration depends on
   nursing intensity and maternal energy balance) → natural inter-birth interval ~2.5–4 years.
8. **Parental investment:** child energy needs drawn from parents' household food; child survival
   hazard multiplied by care deficit; for animals, litter size × per-offspring investment = species
   budget (r/K continuum).

Life-history template per species (the god's "species designer" exposes these): body mass, maturity
age, gestation, litter size, inter-birth interval, max lifespan (Gompertz B), care type, diet, sociality.

### C.5 Animal cognition tiers

| Tier | Example | Perception | Motivation | Decision | Memory | Learning | Social | Cost/agent/tick |
|---|---|---|---|---|---|---|---|---|
| T0 insect | ant, bee, locust | local gradient sampling (pheromone/food field) | 2–3 drives | reactive rules / tiny evolved NN (biosim4-style, ≤ 16 genes) | none (stigmergy via fields) | none (evolution only) | stigmergy, swarm | ~50 ns; usually aggregated as fields |
| T1 fish / small bird | sardine, sparrow | neighbors within radius (spatial hash) | hunger, fear | Boids steering + flee/forage switch | none / 1 last food spot | none | flocking | ~200 ns |
| T2 rabbit / deer | prey mammal | vision cone + smell; predator alarm | Lorenz-style drives: hunger, thirst, fear, mate, rest | utility-based action selection over ~8 actions; MVT patch leaving | home range center, 3–5 remembered patches | habituation, simple conditioning (predator places) | herd cohesion, alarm propagation | ~1 µs |
| T3 wolf / lion | social predator | multi-sense, target tracking | drives + social drives (rank, pack bond) | utility + cooperative hunt roles (chaser/flanker) | territory map (coarse), pack member IDs, prey locations | reinforcement of hunting success per prey type | dominance hierarchy (Elo winner/loser effect), territory scent marking | ~5 µs |
| T4 ape / elephant | great ape | rich + individual recognition | drives + curiosity | utility + short planning (HTN depth 2–3) | episodic-lite: relationships ledger (affinity per known individual), tool spots | social learning (copy successful behaviors), tool use | coalitions, grooming as bond currency | ~20 µs |
| T5 human | human | full | needs + emotions + goals | HTN/GOAP planning; LLM only for focused agents | episodic + semantic memory (other track) | individual + cultural learning | language, kinship, institutions | 50 µs–ms (LOD) |

Behavior hierarchy for T2–T4 follows Tinbergen: drive (instinct center) → appetitive behavior (search)
→ consummatory act (eat/drink/mate), with Lorenz-style drive accumulation so behaviors like
"vacuum activity" and displacement occur naturally when drive is high and stimulus absent.

### C.6 Plants and ecology on a grid with nutrient cycle

Grid: **ecological cell = 16 m × 16 m** (adjust to world scale). Each cell stores:
- climate inputs (temp, precipitation, light, season), soil: water (mm), mineral N pool, P pool
  (optional), soil organic matter (SOM) fast & slow pools, litter pool, detritus/carcass pool.
- plant layer per **plant functional type** (8–10 PFTs: grass, forb, shrub, deciduous tree, conifer,
  crop-cereal, crop-tuber, aquatic plant, fungus, moss/lichen): biomass (leaf/stem/root/fruit-seed),
  cover, mean height, age-class histogram (4 bins), seed bank, genetic mean of 4 traits (growth rate,
  drought tolerance, defense, seed size).
- Individual trees only where observed/important (orchards, sacred trees, forests near settlements):
  promoted to agents with DBH, height, crown radius, using SORTIE NCI or iLand-like stamps.

Daily/weekly update per cell:
1. Light: canopy layers by height; each PFT gets I = I0·exp(−k·LAI_above).
2. Growth: NPP = ε·I_absorbed·f(temp)·f(water)·f(N) − respiration; logistic cap by space.
3. Allocation to leaf/stem/root/seed by PFT rules; seed production → seed bank of this cell and
   dispersal kernel to neighbors (exponential kernel for wind; animal-dispersed seeds move with
   herbivores' gut-passage time — a real cross-system link).
4. Herbivory: grazing removes leaf/fruit biomass consumed by animal agents or cohorts.
5. Mortality & litter: senescence turns biomass into litter (fixed C:N per PFT).
6. Decomposition: litter → SOM → mineral N with rate k = k0·Q10^((T−10)/10)·moisture factor;
   decomposer activity (fungi/microbes) modeled as a rate modifier, optionally a Lenia/Flow-Lenia field.
7. Uptake: plants take mineral N proportional to root biomass, limited by pool.
8. Inputs/outputs: N fixation (legume PFT), deposition, leaching with water flow, fire (volatilizes N,
   resets biomass, releases ash minerals). Carcasses and dung add to detritus — predators "fertilize"
   kill sites.
9. Establishment: seed bank germinates if light/water/temp within PFT envelope (LPJ-GUESS idea).

Carrying capacity emerges: herbivore numbers limited by NPP of edible PFTs; predators by herbivore
production; humans by foraging/agriculture yields. No hard-coded K anywhere.

Animal ecology coupling: each herbivore species has a diet preference vector over PFTs; intake via
Holling type II functional response to biomass density; predators use Holling II over prey density with
handling time; scavengers draw from carcass pool. Extinction is real: when a species' total count hits 0
(across all LOD tiers), it's gone unless the god reseeds; small-population inbreeding automatically
increases recessive disease expression (genome-level).

### C.7 Disease model (individual + compartmental)

**Pathogen definition** (data, can mutate): host range (species list → zoonoses), transmission route
(contact, airborne, fecal-oral/water, vector, sexual, food), β per route, latent period distribution,
infectious period distribution, severity by age curve, case fatality by age & nutrition, immunity type
(none/waning/lifelong), waning τ, environmental persistence (days in water/soil), reservoir species,
virulence-transmission trade-off parameter. Pathogen strains carry 16-bit antigen code; mutations change
antigen distance → partial immune escape (immunity reduced by antigen Hamming distance).

**Individual mode (observed / active agents):**
- Exposure sources: (a) household/family edges, (b) location contagion pools (EMOD-style: each
  location accumulates infectious pressure from present infectious agents; each susceptible present
  draws P = 1 − exp(−β·pressure·time)), (c) water/food sources with contamination level, (d) animal
  contact for zoonoses, (e) vectors as a cell-level field.
- At infection: sample latent end, infectious end, symptom onset, severity, outcome date (recover/
  die) — Covasim-style scheduled transitions; symptoms feed behavior (stay home, seek healer).
- Within-host abstracted to a 1-D load curve shape for visuals and infectiousness weighting.

**Aggregate mode (cohorts / fast-forward):**
- Per settlement or ecological region × age-band (5 bands) × species: S, E, I, R (+ D) counts, with
  stochastic chain-binomial steps (tau-leaping) each day/week; force of infection λ = Σ_routes β_r ·
  C_r(age matrix) · I/N; spatial coupling via migration/trade flows between settlements (gravity model).
- Immunity waning: R → S at rate 1/τ_w; vaccination/medicine modify rates.
- Zoonotic spillover: λ_spill = β_animal · contact_rate(human-animal, from land use/hunting/herding) ·
  prevalence in reservoir species (from animal aggregate SIR).

**Consistency:** β per contact in individual mode is calibrated so that the expected R0 equals
β_aggregate/γ in aggregate mode (automated calibration test: run ABM on 10k agents vs chain-binomial,
compare final size and peak timing).

### C.8 Death and demography
- Each individual: hazard composition (senescence, infection, injury, starvation, exposure, violence,
  childbirth). Cause of death recorded → god's statistics panel and the inheritance track.
- Cohort mode: per region a Leslie/age-structured matrix with 1-year age bins × sex; survival from
  regional Siler parameters adjusted by aggregate food sufficiency, disease deaths (from SEIR), and war;
  fertility from age-specific fertility rates (ASFR) adjusted by Bongaarts factors and food.
- Life tables computed live per region from death records (l_x, q_x, e_x) — displayed to god, also used
  to sanity-check that the simulated mortality matches the configured model (drift detection).

### C.9 LOD switching

| Tier | Who | Body | Genetics | Reproduction | Cognition | Disease | Tick |
|---|---|---|---|---|---|---|---|
| L0 Focused | camera-followed agents (≤ 50) | full table C.3 hourly; injuries per region | full genome | full pipeline | full tier brain (+LLM for humans) | individual, load curve | 1 game-min to 1 h |
| L1 Active | near camera / important (≤ 5k) | daily body | full genome | full pipeline, simplified choice | tier brain at reduced frequency | individual scheduled transitions | 1 h–1 day |
| L2 Background agents | rest of individuals (≤ 100k) | weekly: energy balance, scheduled death date | full genome (32 B) | event-scheduled (conception/birth dates) | utility-only, no memory detail | individual, scheduled; location pools only | 1 week |
| L3 Cohorts | aggregated groups (per cell/settlement × species × age × sex) | mean & variance of mass/condition | allele frequency vector (128 floats) + trait mean/variance | ASFR × females × condition | none (rates) | SEIR counts | 1 week–1 month |
| L4 Statistical | whole regions/species at 1000× | aggregate biomass | per-species allele freq + G matrix | Leslie matrix | none | ODE/chain-binomial | 1 month–1 year |

Transitions:
- **Down (individual → cohort):** bin individuals; sum counts; accumulate allele counts into frequency
  vector; compute trait mean/variance; move infected individuals into E/I/R by state; store the IDs of
  "notable" individuals (named, relatives of focused agents) in a small persistent roster that keeps
  L2 fidelity even in aggregate regions.
- **Up (cohort → individual):** sample N individuals from cohort: ages from age histogram, sex ratio,
  genotypes by sampling each locus from allele frequencies (Hardy–Weinberg, optionally preserving
  linkage via a few stored "haplotype archetypes" per cohort), body condition from mean/variance,
  disease state from SEIR proportions, family structure by a household-generation routine (pair adults,
  attach children with plausible ages). Mark them "unobserved-generated" so the history panel does not
  claim invented ancestry details.
- **Conservation checks:** counts, total biomass, total infected per pathogen preserved exactly at the
  transition step; allele frequency preserved in expectation (and exactly in counts if we sample
  without replacement from a precomputed allele pool).
- **Evolution in aggregate:** per generation: Δp_i ≈ p_i(1−p_i)·Σ_k w_ki·β_k / norm + drift
  (binomial with Ne) + mutation; trait means from allele freqs; β_k (selection gradient) computed from
  environment-trait mismatch (stabilizing selection toward cell optimum: β = (opt − z̄)/V_S) and from
  predation/disease pressures. This is Thrive "auto-evo" logic at population level, but grounded in
  quantitative genetics so switching back to individuals reproduces the same trait distributions.
- **Hysteresis:** promote/demote with time and distance hysteresis to avoid thrashing at boundaries.

### C.10 Performance budget (target 1M organisms)
- L2 individual record: ~96–128 bytes (genome 32 B + body 32 B + state/IDs 32–64 B) → 100k = ~12 MB.
- L3/L4 cohort: ~1–2 KB each (allele freq vector dominates) → 10k cohorts = 20 MB.
- Plants: per cell ~200 B × 1M cells = 200 MB worst case → use 64×64 m cells for unobserved regions.
- Daily L2 update ≈ 50–100 ns/agent (SoA, SIMD) → 100k agents ≈ 10 ms per game day; at 1000× (≈ 3
  game years per real minute) switch most to L3/L4.

---

## Part D — Open questions / risks
1. **Visible evolution vs realism.** Elevated mutation rates and strong selection create visible change
   but can produce runaway trait drift; need stabilizing selection + trait costs; must playtest.
2. **LOD artefacts.** Up-sampling cohorts invents families and genealogies; players may notice
   inconsistencies (e.g., siblings with improbable ages). Need a "genealogy fog" UI concept.
3. **Linkage loss in aggregate mode** destroys co-adapted gene combinations; haplotype archetypes may
   be needed for species under strong correlational selection.
4. **Calibration debt.** Mortality, fertility, disease, and ecological parameters interact; without
   automated calibration tests (target life tables, TFR, predator/prey ratios) systems will drift.
5. **Ecological collapse is the default.** Agent-based predator-prey systems frequently go extinct in
   small worlds. Need spatial refuges, prey switching (type III functional response), and density-dependent
   predator mortality; decide whether the god gets warning indicators.
6. **Disease frustration.** Realistic pre-modern epidemics (plague-like 30% mortality) can wipe out the
   player's favorite population; needs tuning and god intervention tools.
7. **License hygiene.** Many of the best references (SLiM, iLand, NetLogo, OpenABM, Thrive, TensorNEAT,
   Critterding) are GPL; FRED is non-commercial; Cataclysm-DDA is CC BY-SA. Only MIT/BSD/Apache items
   (Pulse, BioGears, bw, ALIEN, Lenia, biosim4, Evolving-Protozoa, neat-python, SharpNEAT, Mesa,
   Preylife, Covasim, Starsim, EMOD, Epiabm, LASER) are candidates for code reuse.
8. **Unverified facts** to check before relying on them: exact last-commit dates (API blocked here),
   tskit license, Flow-Lenia repo license, SORTIE license, Pulse repository activity (gitlab blocked),
   Thrive auto-evo internals, Avida license file.
9. **Human ethics/sensitivity.** Heritable "cognitive potential" and pigmentation in humans risk
   reading as eugenics/racial determinism. Recommend: keep cognitive potential low-heritability,
   uncorrelated with appearance loci, strongly modulated by environment/learning, and never surface
   group-level comparisons in UI.
10. **Brain evolution vs designed AI.** Evolving neural brains (biosim4/NEAT) is attractive for insects/fish
    but makes mammal behavior unreadable; recommend evolving only parameters (drive weights, thresholds)
    of designed utility AI for T2+ tiers.
11. **Sleep/circadian across LOD.** Borbély model matters only for L0–L1; L2+ uses a fixed daily
    schedule — need care so that a promoted agent doesn't instantly collapse from undefined fatigue.

---

## Sources (fetched/searched this session)
- https://github.com/MesserLab/SLiM, https://github.com/BoPeng/simuPOP, https://github.com/molpopgen/fwdpy11, https://github.com/tskit-dev/msprime
- https://pulse.kitware.com/_f_a_q.html, https://pulse.kitware.com/_about_pulse.html, https://github.com/BioGearsEngine/core
- https://github.com/physiology/Physiomodel, https://github.com/INSP-RH/bw, https://github.com/CleverRaven/Cataclysm-DDA
- https://github.com/devosoft/avida, https://en.wikipedia.org/wiki/Avida_(software), https://github.com/acisternino/tierra, https://github.com/PaulTOliver/salis-v2
- https://github.com/polyworld/polyworld, https://www.framsticks.com/sdk, https://github.com/chrxh/alien, https://github.com/Chakazul/Lenia
- https://github.com/tsua0002/FlowLenia-experiment, https://arxiv.org/pdf/2212.07906, https://github.com/davidrmiller/biosim4
- https://github.com/DylanCope/Evolving-Protozoa, https://github.com/bobke/Critterding2, https://github.com/Revolutionary-Games/Thrive, https://github.com/keiwando/evolution
- https://github.com/CodeReclaimers/neat-python, https://github.com/colgreen/sharpneat, https://github.com/EMI-Group/tensorneat
- https://github.com/NetLogo/NetLogo, https://github.com/projectmesa/mesa, https://github.com/SebLague/Ecosystem-2, https://github.com/codevalley/preylife, https://github.com/wtraylor/modular_megafauna_model
- https://github.com/edfm-tum/iland-model, https://github.com/LMurphy186232/Core_Model, https://github.com/LPJ-GUESS, https://github.com/openalea/lpy
- https://github.com/InstituteforDiseaseModeling/covasim, https://github.com/starsimhub/starsim, https://github.com/InstituteforDiseaseModeling/EMOD, https://github.com/BDI-pathogens/OpenABM-Covid19
- https://github.com/SABS-R3-Epidemiology/epiabm, https://github.com/PublicHealthDynamicsLab/FRED, https://github.com/epistorm/epydemix, https://github.com/EpiModel/EpiModel, https://github.com/InstituteforDiseaseModeling/laser
