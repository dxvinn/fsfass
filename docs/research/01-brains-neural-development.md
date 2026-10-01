# 01 — Artificial Brains, Neural Pathways / Associative Learning, Brain Development

Research track for the god-simulator. Scope: what per-creature "brain" machinery produces believable,
*learned* behaviour (child touches fire -> pain -> avoids fire for life) at a cost compatible with
10k–1M organisms under level-of-detail (LOD) simulation. The deterministic sim is the source of truth;
LLMs only narrate/reflect for a handful of focal agents.

Verification note: repository facts (license, language, stars, activity) were checked against the live
GitHub/project pages on 2026-10-01 via web fetch. Star counts are approximate. Where a fact could not be
confirmed it is marked "unverified". Already-known systems (ACT-R, Soar, NARS, OpenCog, LIDA, Nengo,
Monty, Leabra/Axon, pymdp, generative agents, etc.) are only referenced where a specific mechanism adds depth.

## 0. Executive summary (read this first)

1. **No off-the-shelf library is the brain.** Every spiking/neural simulator below is 3–6 orders of magnitude
   too expensive per creature for 10k–1M organisms, and most of the "best" open-source cognitive code is
   GPL/AGPL or non-commercial. The value is in *algorithms*, which are cheap, well-documented and free to
   re-implement: Rescorla–Wagner / TD prediction errors, eligibility traces (three-factor plasticity),
   Pearce–Hall associability, reliability-based habit/goal arbitration, learning-progress curiosity,
   episodic-to-semantic consolidation, perceptual-access belief tracking.
2. **The single most relevant precedent is Creatures (1996, Steve Grand).** Its Norn brain already did
   "touch fire -> punished -> stop" with ~1000 neurons per creature on a 1996 PC, using what is in modern terms
   a reward-modulated three-factor rule (dendrite "susceptibility" = eligibility trace, reward/punishment
   chemicals = neuromodulator, short-term weight relaxing to a long-term weight = consolidation).
3. **Cheapest abstraction that produces fire->pain->avoidance:** a per-creature *sparse associative graph*
   over a world-wide concept vocabulary, whose edges hold a weight + lazily-computed eligibility, updated by a
   surprise-gated (prediction-error) rule, read out as Pavlovian value that biases action selection.
   ~8–12 bytes per edge, a few hundred edge touches per decision, stored as *deltas* over a species/culture
   prior so that 1M creatures fit in memory. Details in the final sections.
4. **Development = parameter schedules + gated capacities**, not separate code paths: learning rate,
   aversive/appetitive asymmetry, curiosity weight, working-memory slots, planning depth, habit strength,
   theory-of-mind depth, forgetting rate, and trust in parent vs. peer information all follow age curves,
   with a few hard "critical-period" windows (attachment, first-language phonology, imprinting for animals).

---

## 1. Candidate systems — A. Neural simulation frameworks (reference / offline tooling, not runtime)

These are useful for *prototyping and validating* small circuits (e.g. "does my cheap rule reproduce
blocking, extinction and renewal like a spiking amygdala does?"), not for running inside the game.

### Brian2
- URL: https://github.com/brian-team/brian2
- License: CeCILL 2.1 (French GPL-compatible copyleft licence — treat as GPL-like for distribution)
- Language: Python (generates C++/Cython code)
- Activity/maintenance: very active; latest commit 2026-09-21; ~1.2k stars
- Architecture (how it works, concretely — data structures, update rules, formulas where useful): users write
  neuron/synapse models as text equations (e.g. `dv/dt = (v_rest - v + I)/tau`), Brian2 parses them into
  state-variable arrays and generates vectorised C++ for a clock-driven loop: integrate ODEs, check threshold,
  reset, propagate spikes through synapse objects, run "on_pre/on_post" plasticity code. STDP is written
  as two traces: pre-trace `a_pre` jumps on a presynaptic spike and decays with tau_pre; post-trace likewise;
  on a post spike the weight grows by `a_pre`, on a pre spike it shrinks by `a_post`.
- Scientific concept modeled: spiking networks, STDP, short-term plasticity, conductance-based neurons.
- Computational cost (per agent per tick, rough): an amygdala-sized model (1k–10k neurons) costs ~ms per
  0.1 ms simulated timestep; i.e. ~10^4–10^6 times our budget per creature.
- What we can learn: the *trace formulation* of STDP — keep one exponentially decaying number per neuron
  instead of spike histories. This is exactly how our cheap eligibility traces should work.
- What we could integrate: algorithm only (trace-based STDP, lazy exponential decay). Use Brian2 offline in
  a research notebook to validate the game's simplified fear circuit against published spiking models.
- What we should NOT integrate: the runtime itself; CeCILL copyleft is a problem for a closed binary.
- Scalability (10 / 1k / 100k agents): 10 = only offline; 1k/100k = impossible in real time.

### NEST Simulator
- URL: https://github.com/nest/nest-simulator
- License: GPL-2.0-or-later
- Language: C++ with Python front-end (PyNEST); NESTML for model description
- Activity/maintenance: very active; latest commit 2026-10-01; ~670 stars
- Architecture: event-driven/hybrid simulator for very large point-neuron networks distributed with MPI;
  includes dopamine-modulated STDP synapse (`stdp_dopamine_synapse`, after Izhikevich 2007 / Potjans et
  al. 2010) where an eligibility trace c is driven by STDP and the weight changes as dw/dt = c * (d - d_baseline).
- Scientific concept modeled: cortical-scale spiking dynamics, three-factor reward-modulated STDP.
- Computational cost: designed for supercomputers; irrelevant for per-creature runtime.
- What we can learn: the canonical *distal reward* solution — Izhikevich 2007 "Solving the distal reward
  problem through linkage of STDP and dopamine signaling": a slowly decaying eligibility tag on each synapse
  bridges the seconds-long gap between "touched glowing thing" and "pain arrived". That is the theory
  behind our per-edge eligibility.
- What we could integrate: algorithm only.
- What we should NOT integrate: code (GPL).
- Scalability: N/A for runtime.

### GeNN (GPU-enhanced Neuronal Networks)
- URL: https://github.com/genn-team/genn
- License: LGPL-2.1
- Language: C++ code generator (CUDA / HIP / CPU backends) + PyGeNN
- Activity/maintenance: active; latest commit 2026-08-24; ~280 stars
- Architecture: you describe neuron and synapse "snippets"; GeNN generates GPU kernels that update all
  neurons of a population in parallel and use sparse/procedural connectivity (connectivity can be regenerated
  on the fly instead of stored — "procedural connectivity").
- Scientific concept modeled: large spiking networks on GPUs (insect mushroom-body models, cortical models).
- Computational cost: millions of neurons in real time on one GPU — i.e. a *few thousand* tiny brains, not 1M.
- What we can learn: two ideas transfer directly: (a) batch the same small brain update over many creatures
  as a GPU/SIMD kernel (structure-of-arrays), (b) *procedural connectivity* — don't store what you can
  regenerate from a seed (e.g. innate wiring from the genome seed).
- What we could integrate: LGPL allows dynamic linking in closed software, but we would not need it.
- What we should NOT integrate: spiking dynamics at runtime.
- Scalability: 10 brains of 10k neurons fine; 1k of 1k neurons plausible on GPU; 100k no.

### BindsNET
- URL: https://github.com/BindsNET/bindsnet
- License: AGPL-3.0 (flag: incompatible with closed-source distribution, even network use)
- Language: Python (PyTorch)
- Activity/maintenance: maintained (latest commit 2026-09-22, mostly dependency bumps); ~1.7k stars
- Architecture: PyTorch tensors for layers of LIF/Izhikevich neurons, connections with pluggable learning
  rules: PostPre (STDP), WeightDependentPostPre, Hebbian, MSTDP and MSTDPET (reward-modulated STDP, with and
  without eligibility trace, after Florian 2007), plus Gym-environment wrappers for RL experiments.
- Scientific concept modeled: STDP, reward-modulated STDP, spiking RL.
- Computational cost: hundreds of µs–ms per network step on GPU.
- What we can learn: the MSTDP-ET formulation — weight change = learning rate x reward x eligibility, where
  eligibility integrates the STDP term with a time constant; a clear reference for three-factor rules.
- What we could integrate: nothing as code (AGPL); equations only.
- What we should NOT integrate: any source.
- Scalability: research scale only.

### snnTorch
- URL: https://github.com/jeshraghian/snntorch
- License: MIT
- Language: Python (PyTorch)
- Activity/maintenance: active; latest commit 2026-09-25; ~2.1k stars
- Architecture: spiking neurons (Leaky, Synaptic, Alpha, recurrent variants) as PyTorch modules trained with
  surrogate gradients (backprop through time with a smooth stand-in for the spike derivative).
- Scientific concept modeled: gradient-trained SNNs, neuromorphic computing.
- Computational cost: like a small RNN per step.
- What we can learn: surrogate-gradient training lets you *pre-train* innate circuits offline (e.g. a
  species' prey-detection reflex) and export a tiny fixed network; inference of a few hundred LIF units is cheap.
- What we could integrate: MIT code is safe, but we'd rather export trained weights to our own C++/Rust kernel.
- What we should NOT integrate: online backprop per creature.
- Scalability: frozen tiny nets: 100k+ feasible on GPU; online learning: no.

### Norse
- URL: https://github.com/norse/norse
- License: LGPL-3.0
- Language: Python (PyTorch)
- Activity/maintenance: active; latest commit 2026-07-07; ~820 stars
- Architecture: functional + module API for LIF, adaptive LIF, Izhikevich, LSNN; STDP utilities; NIR export.
- Scientific concept modeled: bio-inspired deep learning with spiking primitives.
- Computational cost: as snnTorch.
- What we can learn: same as snnTorch; NIR (Neuromorphic Intermediate Representation) export means trained
  circuits can be moved between tools — handy if we ever prototype a creature reflex circuit offline.
- What we could integrate: nothing at runtime; LGPL is acceptable for tooling.
- What we should NOT integrate: runtime SNNs.
- Scalability: research scale.

## 2. Candidate systems — B. Associative learning, plasticity and memory substrates

### Creatures Norn brain (via the openc2e engine re-implementation)
- URL: https://github.com/openc2e/openc2e
- License: LGPL-2.1 (openc2e engine). The original Creatures games and genomes are proprietary.
- Language: C++
- Activity/maintenance: active; latest commit 2026-09-27; ~170 stars. README states C3+ creatures "work
  somewhat", C1/C2 creatures do not — so brain fidelity is partial.
- Architecture (from Grand, Cliff & Malhotra 1997, "Creatures: Artificial Life Autonomous Software Agents for
  Home Entertainment", and Grand 1997 IEEE Expert "Creatures: an exercise in creation"; details below are from
  those papers and fan documentation, not verified line-by-line in openc2e code):
  - ~1000 neurons in ~9–10 genetically specified *lobes*: perception (sensory/drive inputs), attention
    (which object is "it"), concept (hundreds of cells, each sampling a small conjunction of perception cells
    — a sparse random conjunctive code), decision (one cell per verb: eat, push, pull, retreat, ...), drive lobe.
  - Each neuron's behaviour is a tiny genetically defined *state-variable rule* (SVRule) program: input sum,
    leak toward rest, threshold.
  - Each dendrite has a short-term weight (STW) and a long-term weight (LTW). Reinforcement moves STW;
    STW relaxes back toward LTW; LTW slowly drifts toward STW. This is a two-timescale memory (fast "working"
    synapse + slow consolidated synapse).
  - When a concept->decision dendrite is active at the moment an action fires, it becomes *susceptible*
    (a decaying flag). Reward or punishment *chemicals* released by the biochemistry (e.g. pain raises
    punishment) then change the weights of only the susceptible dendrites, proportionally to susceptibility.
  - Weak dendrites "migrate" to new active cells, so unused memory capacity is recycled.
  - Drives (hunger, pain, boredom, fear...) are chemical levels; reducing a drive releases reward.
  - Instincts were injected by genes that "replay" a stimulus-action-reward pattern during embryology/sleep,
    so creatures are born with priors that experience can overwrite.
- Scientific concept modeled: operant conditioning with neuromodulators; drive reduction; consolidation;
  instincts as pre-experienced training data.
- Computational cost (per agent per tick): ~1k neurons + a few thousand dendrites, updated a few times per
  second on 1996 hardware with ~10 creatures. On modern hardware ~10–50 µs per creature-tick.
- What we can learn: (1) "susceptibility + chemical reinforcer" is a production-proven three-factor rule;
  (2) the STW/LTW split gives forgetting-of-detail-but-keeping-the-gist for free; (3) instincts as
  *pre-trained weights from synthetic experiences* is an elegant way to author innate behaviour without
  `if` rules; (4) a big lesson from Creatures players: dense neural brains are *illegible* — it was very
  hard for designers and players to know *why* a Norn did something. Our graph must be inspectable.
- What we could integrate: design ideas only. Do not copy engine code into our core (LGPL is workable via
  dynamic linking, but there's nothing to link — we only want the ideas).
- What we should NOT integrate: fixed-size dense lobes (wasteful), opaque concept lobes, verb-only decisions
  with no notion of outcome (Norns had no explicit action->outcome model, so no goal-directed planning).
- Scalability: 10 = trivial; 1k = fine; 100k = too heavy with dense lobes; sparse version feasible.

### Rescorla–Wagner, Pearce–Hall and Mackintosh family (via StatsRat as a reference implementation)
- URL: https://github.com/SamPaskewitz/statsrat
- License: MIT
- Language: Python
- Activity/maintenance: small research package (README says in development as of April 2021); ~3 stars;
  last commit date unverified.
- Architecture: modular "learning model" objects for Pavlovian and category learning, fit to trial data.
  The core equations (classical literature, not specific to this repo):
  - Rescorla–Wagner (1972): for every cue i present on a trial, ΔV_i = α_i · β · (λ − Σ_{k present} V_k).
    The *summed* prediction in the error term creates **blocking** (a cue added to an already-predictive cue
    learns nothing) and **overshadowing** (salient cues take most of the credit).
  - Pearce–Hall (1980): associability is not fixed: α_i(t+1) = γ·|λ − ΣV| + (1−γ)·α_i(t). Surprising
    situations make creatures *pay attention and learn faster*; predictable ones slow learning.
  - Mackintosh (1975): α rises for cues that are better predictors than others.
  - Extinction in these models is unlearning, which is empirically wrong (fear returns: spontaneous recovery,
    renewal, reinstatement). Fix: latent-cause / context models (Gershman, Blei & Niv 2010; Gershman et al.
    "Explaining the return of fear with revised Rescorla–Wagner models", Computational Psychiatry 2023
    https://cpsyjournal.org/articles/10.5334/cpsy.88).
- Scientific concept modeled: classical (Pavlovian) conditioning, cue competition, attention in learning.
- Computational cost: O(number of active cues × number of outcomes) per event, a few dozen multiply-adds.
- What we can learn: RW with a summed prediction is *the* cheapest rule that gives fire->pain plus realistic
  side-effects (blocking explains why a child who was already warned "HOT!" learns less about the stove's look).
  Pearce–Hall gives one-trial learning after big surprises for free.
- What we could integrate: the equations (public domain science); MIT code safe if desired for test oracles.
- What we should NOT integrate: plain RW extinction (erases memories) — use context-gated extinction instead.
- Scalability: 10 / 1k / 100k / 1M all fine.

### Temporal-difference learning and dopamine reward prediction error (concept block; key papers)
- URL: no single repo; key references: Sutton 1988 (TD(λ)); Schultz, Dayan & Montague 1997 Science "A neural
  substrate of prediction and reward"; Sutton & Barto, *Reinforcement Learning: An Introduction* (2nd ed. 2018).
- License: n/a (algorithms)
- Language: n/a
- Activity/maintenance: foundational; still the main computational account of phasic dopamine.
- Architecture: δ_t = r_t + γ·V(s_{t+1}) − V(s_t); V(s_t) += α·δ_t·e; eligibility e decays by γλ each step
  and is bumped for the visited state. Dopamine neurons fire for unexpected reward, shift to the earliest
  predictive cue after learning, and dip when a predicted reward is omitted. Distributional TD (Dabney et al.
  2020 Nature) keeps several value estimates with different optimism — a neat knob for personality
  (optimists vs pessimists = asymmetric learning rates for positive vs negative δ).
- Scientific concept modeled: reward prediction error, credit assignment over time, second-order conditioning.
- Computational cost: a few flops per updated state/edge.
- What we can learn: TD extends RW across time: the fire *seen from afar* predicts the fire *near the hand*
  which predicts pain, so fear propagates backward to earlier cues (smoke smell, crackling sound) without
  ever pairing them with pain directly (second-order conditioning).
- What we could integrate: algorithm; asymmetric α+/α− as a personality trait (anxious = large α−).
- What we should NOT integrate: deep TD (DQN-style) per creature.
- Scalability: all tiers.

### Modern Hopfield networks (hopfield-layers)
- URL: https://github.com/ml-jku/hopfield-layers
- License: BSD-style (per README; exact variant unverified)
- Language: Python (PyTorch)
- Activity/maintenance: dormant; latest commit 2022-01-31; ~2.0k stars
- Architecture: Ramsauer et al. 2020, "Hopfield Networks is All You Need": stored patterns X (columns),
  query ξ, one-step update ξ_new = X · softmax(β · Xᵀ ξ). It is attention. Large β retrieves a single stored
  pattern; small β returns a blend (metastable state). Capacity is exponential in dimension.
- Scientific concept modeled: content-addressable associative memory / pattern completion (CA3-like).
- Computational cost: O(N_patterns × d) per query; for 64 memories of d=64 that's ~4k MACs.
- What we can learn: one-shot storage + pattern completion from partial cues ("smell of smoke" retrieves the
  whole "house fire" episode). β is a lovely developmental/emotional knob: low β (child, stressed, elderly)
  = blurry over-generalised recall; high β = crisp specific recall.
- What we could integrate: the retrieval rule for the *episodic memory* lookup of focal (LOD0/1) agents.
- What we should NOT integrate: trainable Hopfield layers / PyTorch dependency.
- Scalability: 10–1k with per-agent stores; 100k only with tiny stores (≤16 patterns).

### Sparse Distributed Memory (Kanerva) — msbrogli/sdm
- URL: https://github.com/msbrogli/sdm
- License: unverified (not visible on repo page)
- Language: C with Python wrapper
- Activity/maintenance: research/thesis code; ~49 stars; last commit unverified
- Architecture: Kanerva 1988. A fixed set of M random "hard locations" in a huge binary space (e.g. 1000 bits).
  Writing pattern p at address a adds ±1 to counters of every hard location within Hamming radius r of a.
  Reading sums counters of locations within r of the query and thresholds. Similar addresses retrieve similar
  contents → natural generalisation and graceful degradation. Bricken & Pehlevan 2021 showed transformer
  attention approximates SDM.
- Scientific concept modeled: cerebellar/hippocampal-like distributed associative memory.
- Computational cost: O(M × bits/64) popcounts per read; M=1k, 256 bits → ~4k popcounts (~µs).
- What we can learn: generalisation by *address similarity* — a sabre-tooth cat and a lion share many bits,
  so fear of one transfers partially. We get the same effect cheaper with hashed feature bit-vectors on concepts.
- What we could integrate: idea; possibly a *species-shared* SDM for culture-level knowledge (one per culture,
  not per creature).
- What we should NOT integrate: per-creature SDMs (counter arrays are memory-hungry: 1k × 256 × int8 = 256 KB).
- Scalability: shared-per-culture: fine at all scales; per-creature: ≤1k.

### Torchhd — hyperdimensional computing / vector-symbolic architectures
- URL: https://github.com/hyperdimensional-computing/torchhd
- License: MIT
- Language: Python (PyTorch)
- Activity/maintenance: moderate; latest commit 2025-06-19; ~390 stars
- Architecture: random high-dimensional vectors (1k–10k dims) as symbols; *bind* (elementwise multiply / XOR
  / circular convolution) builds role-filler pairs, *bundle* (sum + sign) superimposes sets, *permute* encodes
  sequence. Supports MAP, BSC (binary), HRR, FHRR, sparse block codes and more; includes memory modules.
- Scientific concept modeled: compositional distributed representations (Plate, Kanerva, Gayler).
- Computational cost: binary BSC with 1024 bits: bind = 16 XORs of 64-bit words; similarity = 16 popcounts.
- What we can learn: a fixed-size *vector* can hold a whole small fact set ("FIRE∘HURTS + WOLF∘HURTS +
  MOTHER∘COMFORTS"), enabling compact LOD2/LOD3 summaries of a creature's beliefs and fast similarity of
  novel stimuli to known concepts.
- What we could integrate: algorithm (binary spatter codes are ~50 lines to write ourselves); MIT code safe.
- What we should NOT integrate: PyTorch at runtime.
- Scalability: 1024-bit vector = 128 bytes → 1M creatures × 4 summary vectors = 512 MB; fine at 100k.

### HTM (htm.core)
- URL: https://github.com/htm-community/htm.core
- License: AGPL-3.0 (flag)
- Language: C++ with Python bindings
- Activity/maintenance: community-maintained; latest commit 2026-08-21 (dependency bump); ~170 stars
- Architecture: Hierarchical Temporal Memory. Sparse distributed representations (~2% active bits);
  Spatial Pooler (columns compete via k-winners, Hebbian permanence increments on active synapses); Temporal
  Memory (cells within columns have distal dendritic segments that predict the next input; correctly predicted
  cells fire alone, unpredicted input causes the whole column to "burst" = surprise signal). Permanence values
  cross a threshold to become connected synapses.
- Scientific concept modeled: sequence learning and prediction in neocortical minicolumns.
- Computational cost: 2048 columns × 32 cells: ~0.1–1 ms per step.
- What we can learn: (1) "bursting" as a binary novelty/surprise signal; (2) permanence-with-threshold is a
  cheap way to have synapses that are learned gradually but read out as booleans; (3) sequence memory for
  routines ("wake, fetch water, light hearth").
- What we could integrate: ideas only (AGPL).
- What we should NOT integrate: code; full TM per creature (too heavy).
- Scalability: ≤100 agents at full size.

### Model-Free Episodic Control (astier/model-free-episodic-control)
- URL: https://github.com/astier/model-free-episodic-control
- License: MIT
- Language: Python
- Activity/maintenance: small reference implementation; ~18 stars; last commit unverified
- Architecture: Blundell et al. 2016. A per-action table mapping embedded states to the *best return ever
  obtained* from there; value of a new state = mean over k nearest stored neighbours; table capped with
  least-recently-used eviction. Embedding = random projection. Neural Episodic Control (Pritzel 2017) adds a
  learned embedding and differentiable kNN.
- Scientific concept modeled: hippocampal one-shot learning; "remember what worked last time".
- Computational cost: kNN over a few hundred entries: µs with brute force on small embeddings.
- What we can learn: a fast, one-shot "I once found honey in a hollow tree" system that complements slow
  incremental value learning — exactly the complementary-learning-systems split (McClelland, McNaughton &
  O'Reilly 1995). Its optimism (max return) also explains superstitious behaviour.
- What we could integrate: the algorithm in the episodic store of LOD0/LOD1 agents.
- What we should NOT integrate: Atari/VAE specifics.
- Scalability: 1k agents × 256 entries fine; 100k only with ~16 entries each.

### Spiking amygdala model of fear conditioning, extinction, renewal (psipeter/amygdala)
- URL: https://github.com/psipeter/amygdala (paper: Duggins & Eliasmith, "A scalable spiking amygdala model
  that explains fear conditioning, extinction, renewal and generalization", Eur. J. Neurosci. 2024,
  https://onlinelibrary.wiley.com/doi/full/10.1111/ejn.16338)
- License: unverified for the repo; it depends on Nengo, whose current main-branch LICENSE reads GPL-2.0
  (older Nengo releases used a separate restrictive "Nengo licence"; check before any reuse).
- Language: Python (Nengo)
- Activity/maintenance: paper code; latest commit 2024-01-24; ~1 star
- Architecture: nuclei as populations representing vectors: lateral amygdala (LA) learns CS->US association
  by error-driven learning (PES rule: Δw ∝ −error × presynaptic activity); basolateral amygdala (BA) holds
  separate "fear" and "extinction" neurons; extinction neurons are *context-gated* (hippocampal context
  input), and drive intercalated cells (ITC) which inhibit central amygdala (CeA) output; CeA output = fear
  response (freezing). Hence fear returns in a new context (renewal) because the extinction memory is
  context-bound while the fear memory is not.
- Scientific concept modeled: amygdala circuit; acquisition, extinction, renewal, generalisation.
- Computational cost: thousands of spiking neurons — offline only.
- What we can learn: the *circuit logic* is cheap to mimic with 3 numbers per cue: w_fear(cue),
  w_ext(cue, context), and output = max(0, w_fear − Σ_ctx w_ext). That reproduces renewal and spontaneous
  recovery (decay w_ext faster than w_fear) — a big believability win ("she was fine with campfires for
  years, but the forest fire brought it all back").
- What we could integrate: the 3-variable abstraction; nothing from the code.
- What we should NOT integrate: Nengo runtime (GPL, cost).
- Scalability: abstraction: all tiers.

### Successor Representation (awjuliani/successor_examples) and RatInABox
- URL: https://github.com/awjuliani/successor_examples ; https://github.com/RatInABox-Lab/RatInABox
- License: MIT (both)
- Language: Python (notebooks / package)
- Activity/maintenance: successor_examples is a tutorial (~55 stars, date unverified); RatInABox active
  (latest commit 2026-07-09, ~270 stars).
- Architecture: Dayan 1993 SR: M(s, s') = expected discounted future occupancy of s' starting from s, learned by
  TD: M(s,·) += α·(1_s + γ·M(s_next,·) − M(s,·)). Value = M · R, so when reward changes (water hole dried up)
  values update instantly without relearning the map — "latent learning". RatInABox simulates place/grid/head
  direction/boundary/object-vector cells for an agent moving in continuous space and includes successor
  feature and value-neuron learners.
- Scientific concept modeled: hippocampal predictive map (Stachenfeld, Botvinick & Gershman 2017);
  a middle ground between habits and planning (Momennejad et al. 2017).
- Computational cost: tabular SR is |S|² — too big; a *sparse* SR over ~50 landmark places is cheap.
- What we can learn: creatures that wander as children build a map for free and later exploit it when a
  need arises (Tolman's rats). Reward revaluation ("the berries at the river are poisonous now") propagates
  through the map immediately.
- What we could integrate: sparse SR over landmark places for LOD0/LOD1 creatures; MIT code safe.
- What we should NOT integrate: dense SR; per-tick place-cell simulation.
- Scalability: 1k agents × 50 landmarks² = fine; 100k with per-culture shared maps + personal overrides.

### Tolman–Eichenbaum Machine (torch_tem)
- URL: https://github.com/jbakermans/torch_tem
- License: unverified (not shown)
- Language: Python (PyTorch)
- Activity/maintenance: research code; ~180 stars; date unverified
- Architecture: Whittington et al. 2020 Cell. Separates *structure* (grid-cell-like abstract location code
  that path-integrates actions) from *content* (sensory code); hippocampal cells bind the two with fast
  Hebbian memory; generalises map structure across environments.
- Scientific concept modeled: cognitive maps, structural generalisation, place/grid cells.
- Computational cost: deep recurrent model — offline only.
- What we can learn: the conceptual split "where/how things relate" vs "what is here" maps to our design:
  a culture-shared spatial/relational schema + personal content bindings.
- What we could integrate: concept only.
- What we should NOT integrate: the network.
- Scalability: N/A at runtime.

## 3. Candidate systems — C. Action selection, habits vs goals, belief updating, workspaces

### Basal-ganglia action selection: GPR model (ABRG-Models/GPR-BasalGanglia, benoit-girard/CBG)
- URL: https://github.com/ABRG-Models/GPR-BasalGanglia ; https://github.com/benoit-girard/CBG
- License: unverified for both (no licence shown on the repo pages)
- Language: SpineML model files (GPR repo); Python (CBG)
- Activity/maintenance: archival research code; 1–2 stars; dates unverified
- Architecture: Gurney, Prescott & Redgrave 2001 (Biol. Cybern.): each candidate action is a *channel* with a
  "salience" input. Striatal D1 ("selection") units inhibit GPi/SNr for their channel; D2 ("control") units
  act via GPe; the STN sends diffuse excitation to all channels. Net effect: GPi output for the winning
  channel drops below threshold (disinhibiting thalamus = action released) while others stay inhibited.
  Dopamine raises D1 gain and lowers D2 gain → high dopamine = more vigorous/impulsive switching; low dopamine
  = difficulty initiating (parkinsonian). Girard's CBG ("contracting" BG) is a variant with provable
  convergence. Prescott et al. 2006 embedded it in a foraging robot (EASA), see also https://github.com/ferdiex/easa.
- Scientific concept modeled: action selection as disinhibition; persistence; dopamine gain.
- Computational cost: 5 nuclei × n_channels rate units, a few iterations: ~100 flops for 10 actions.
- What we can learn: two things missing from plain softmax: (1) **hysteresis / persistence** — the incumbent
  action gets a bonus so creatures don't dither between eating and drinking every tick ("lock-in" via the
  thalamic loop); (2) a single **dopamine-like tonic level** that scales vigor and switching readily
  (exhausted/depressed creatures barely act; manic ones flit).
- What we could integrate: a 10-line "salience → disinhibition with incumbent bonus and global gain" selector.
- What we should NOT integrate: full nucleus dynamics.
- Scalability: all tiers.

### Model-based / model-free arbitration (Daw, Niv & Dayan 2005; Lee, Shimojo & O'Doherty 2014)
- URL: https://pure.kaist.ac.kr/en/publications/neural-computations-underlying-arbitration-between-model-based-an/
  (Lee et al. 2014, Neuron). No official code located; two-step task implementations exist, e.g.
  https://github.com/mtrazzi/two-step-task (not verified in detail).
- License: n/a (paper)
- Language: n/a
- Activity/maintenance: theory is mature; widely replicated with the two-step task (Daw et al. 2011).
- Architecture: two controllers run in parallel. Model-free (habit) caches Q(s,a) via TD from reward
  prediction errors (RPE). Model-based (goal-directed) learns a transition model and plans; its learning
  signal is a *state* prediction error (SPE). Each controller's **reliability** = 1 − running mean of |its
  prediction error|. The probability of handing control to the model-based system is a sigmoid of the
  reliability difference, biased by cost (planning is effortful). Daw 2005 used Bayesian uncertainty instead:
  pick whichever controller is more certain. Keramati et al. 2011: plan only when the value of information
  beats the time cost. Habits form because, with overtraining, the cached system becomes reliable and cheap.
- Scientific concept modeled: habit vs goal-directed behaviour (dorsolateral vs dorsomedial striatum),
  outcome devaluation insensitivity of habits.
- Computational cost: arbitration itself is ~10 flops; the cost is the model-based lookahead.
- What we can learn: this is the principled answer to "when does a creature think vs. act on autopilot":
  in familiar stable routines → habit (cheap, and also *wrong* when the world changes — a farmer walks to the
  dry well out of habit, which looks alive); in novel/high-stakes/changed situations → deliberate.
- What we could integrate: reliability-weighted arbitration as the central switch of our decision loop,
  and also as an LOD knob (low LOD = forced habit mode).
- What we should NOT integrate: full Bayesian arbitration.
- Scalability: arbitration all tiers; planning only LOD0/LOD1.

### pyhgf — Hierarchical Gaussian Filter
- URL: https://github.com/ComputationalPsychiatry/pyhgf
- License: MIT
- Language: Python (JAX) with a Rust backend
- Activity/maintenance: very active; latest commit 2026-09-30; ~165 stars
- Architecture: Mathys et al. 2011/2014 HGF: a hierarchy of Gaussian beliefs; level 1 tracks the quantity
  (e.g. "is this path safe"), level 2 tracks how fast it changes (volatility), level 3 how fast *that*
  changes. Each update is a closed-form precision-weighted prediction error: μ_new = μ + (π_input/π_new)·δ.
  The effective learning rate rises automatically when the world is volatile and falls when stable.
- Scientific concept modeled: Bayesian belief updating under volatility; computational psychiatry (anxiety =
  overestimated volatility; autism/psychosis hypotheses about precision).
- Computational cost: ~20–50 flops per node update; a 2-level HGF per tracked variable is trivial.
- What we can learn: an extremely cheap principled *adaptive learning rate*. Applying a 2-level HGF to a few
  key beliefs per creature (food reliability of a place, trustworthiness of a neighbour, danger of a region)
  gives belief revision that reacts sensibly to regime change (famine, war) — and personality via priors on
  volatility.
- What we could integrate: the update equations (simple; re-implement); MIT code safe.
- What we should NOT integrate: JAX runtime.
- Scalability: all tiers for a handful of variables per creature.

### Predictive coding networks (Bogacz-Group/PredictiveCoding; also infer-actively/pypc)
- URL: https://github.com/Bogacz-Group/PredictiveCoding ; https://github.com/infer-actively/pypc
- License: unverified for Bogacz-Group repo (not shown); pypc unverified
- Language: Python (PyTorch)
- Activity/maintenance: Bogacz-Group repo ~76 stars, date unverified
- Architecture: Rao & Ballard 1999; Whittington & Bogacz 2017. Each layer holds value nodes x_l and error
  nodes ε_l = x_l − f(W_l x_{l+1}); inference relaxes x to minimise Σ‖ε‖² (iterative, local), then weights
  update with a local Hebbian-like rule ΔW ∝ ε_l · f(x_{l+1})ᵀ. Temporal predictive coding predicts the next
  input from the previous state.
- Scientific concept modeled: perception as prediction-error minimisation; local learning approximating backprop.
- Computational cost: tens of inference iterations × layer size² — too heavy per creature at scale.
- What we can learn: the general principle we actually need is "*learn only from what you failed to
  predict*" — prediction error gates both learning and attention. We implement this symbolically (expected
  vs. observed outcome per active edge), not with relaxation dynamics.
- What we could integrate: principle only.
- What we should NOT integrate: iterative inference per tick.
- Scalability: ≤10 agents if ever.

### RxInfer.jl and ActiveInference.jl (active inference beyond pymdp)
- URL: https://github.com/ReactiveBayes/RxInfer.jl ; https://github.com/ComputationalPsychiatry/ActiveInference.jl
- License: MIT (both)
- Language: Julia
- Activity/maintenance: RxInfer very active (latest commit 2026-09-28, ~420 stars); ActiveInference.jl ~42
  stars, date unverified.
- Architecture: RxInfer does reactive message passing on factor graphs (Forney-style), closed-form for
  conjugate pairs, so inference is a set of local message updates that can be triggered incrementally as
  observations stream in. ActiveInference.jl provides POMDP-style active inference agents (A, B, C, D
  matrices; expected free energy = risk + ambiguity) similar to pymdp.
- Scientific concept modeled: free-energy principle / active inference; perception & action as inference.
- Computational cost: POMDP active inference scales with |states|×|policies|; policy enumeration explodes
  beyond a few steps.
- What we can learn: the **expected free energy decomposition** is a useful design lens: action value =
  pragmatic value (expected preference satisfaction) + epistemic value (expected information gain). That is
  exactly "goal-seeking + curiosity" in one currency — we can approximate epistemic value with a novelty /
  uncertainty bonus on edges with low evidence count.
- What we could integrate: the scoring decomposition; reactive "update only on new evidence" scheduling.
- What we should NOT integrate: full EFE policy search per creature.
- Scalability: full AIF: ≤100 agents with tiny state spaces; heuristic decomposition: all tiers.

### Shimmer (Global Latent Workspace)
- URL: https://github.com/ruflab/shimmer
- License: MIT
- Language: Python (PyTorch)
- Activity/maintenance: small (~7 stars); date unverified; from VanRullen's lab (VanRullen & Kanai 2021,
  "Deep learning and the Global Workspace Theory").
- Architecture: several pretrained domain modules (vision, language...) each with encoder/decoder to a shared
  latent workspace; training with translation, cycle-consistency and contrastive losses so content in one
  modality can be broadcast and decoded into others; attention selects which module "writes" the workspace.
- Scientific concept modeled: global workspace theory (Baars; Dehaene's neuronal workspace): many specialist
  processes, one limited-capacity broadcast stage.
- Computational cost: deep nets — offline only.
- What we can learn: GWT reduces to a **cheap, very useful game mechanic**: a K-slot buffer where candidate
  items (percepts, drives, memories, plans) compete by salience; only winners are (a) eligible for learning,
  (b) used in planning, (c) reportable to the LLM narrator ("what is she thinking about?"). This also gives
  attentional blindness and distraction for free.
- What we could integrate: the competition-for-broadcast idea; nothing from code.
- What we should NOT integrate: multimodal deep workspace.
- Scalability: K-slot workspace: all tiers (K drops at low LOD).

### PsyNeuLink
- URL: https://github.com/PrincetonUniversity/PsyNeuLink
- License: Apache-2.0
- Language: Python
- Activity/maintenance: active; latest commit 2026-09-15; ~120 stars
- Architecture: block-modelling environment for cognitive neuroscience: Mechanisms (e.g. DDM, LCA,
  TransferMechanisms, recurrent/Hebbian/Kohonen mechanisms), Projections and a Composition scheduler;
  ControlMechanisms implement Expected Value of Control (Shenhav, Botvinick & Cohen 2013) — allocating
  costly cognitive control where expected payoff exceeds effort cost. (Mechanism list from documentation
  knowledge; the repo page itself highlights DDM, Stroop and backprop examples.)
- Scientific concept modeled: cognitive control, decision diffusion, conflict monitoring.
- Computational cost: Python modelling tool; not runtime-suitable.
- What we can learn: (1) **drift-diffusion / leaky competing accumulators** as decision mechanisms give
  realistic *reaction-time and hesitation* behaviour (close calls take longer, more errors under pressure);
  (2) **Expected Value of Control** is a principled "effort budget" for deliberation — tired or stressed
  creatures plan less.
- What we could integrate: Apache-2.0 is commercial-friendly, but we would re-implement the 2–3 equations.
- What we should NOT integrate: the framework itself.
- Scalability: equations: all tiers.

### pyClarion (CLARION cognitive architecture)
- URL: https://github.com/cmekik/pyClarion
- License: MIT
- Language: Python
- Activity/maintenance: active but "highly experimental"; latest commit 2026-08-06; ~68 stars
- Architecture: Ron Sun's CLARION: every subsystem has an *implicit* bottom level (distributed, learned by
  RL/backprop) and an *explicit* top level (symbolic rules). Bottom-up learning (Rule-Extraction-Refinement):
  when an implicit action succeeds, extract a rule; generalise or specialise it based on an information-gain
  measure of its success rate. Motivational subsystem: drives (food, water, sleep, avoid danger, affiliation,
  dominance, autonomy, curiosity...) with strengths set by deficit × weight; meta-cognitive subsystem sets goals
  and learning parameters from drive state.
- Scientific concept modeled: implicit vs explicit learning; drive-based motivation; skill → rule extraction.
- Computational cost: per-decision small; Python implementation not runtime-grade.
- What we can learn: **rule extraction from successful implicit behaviour** is how we make creatures'
  learned knowledge *legible and transmissible*: once an edge pattern is strong and reliable, crystallise it into
  an explicit "belief" ("fire burns") that can be spoken, taught and narrated by an LLM. CLARION's drive list is
  also a well-motivated starting set for human needs.
- What we could integrate: RER idea + drive taxonomy; MIT code safe as reference.
- What we should NOT integrate: the generic chunk/feature machinery at scale.
- Scalability: abstracted: all tiers; full: ≤1k.

### MicroPsi2 (Psi theory)
- URL: https://github.com/joschabach/micropsi2
- License: MIT (license.txt: "All parts of MicroPsi2 are under MIT license, unless marked otherwise")
- Language: Python
- Activity/maintenance: dormant; latest commit 2016-04-22; ~190 stars
- Architecture: Dörner's Psi theory as implemented by Joscha Bach: node nets with typed links; *urges*
  (physiological: food, water, integrity; cognitive: competence, certainty; social: affiliation) whose
  deviation produces urge signals; *modulators* computed from urges — arousal (activation), resolution level
  (how carefully to perceive/plan), selection threshold (how sticky the current goal is) — emotions are not
  separate modules but *configurations of modulators* (anger = high arousal, low resolution, high selection
  threshold, high competence urge frustrated). Reinforcement = change of urge level ("pleasure" when an urge
  is reduced, "displeasure" when it rises).
- Scientific concept modeled: motivation, emotion as cognitive modulation, need-driven goal formation.
- Computational cost: modulators are a handful of scalars per creature.
- What we can learn: the **modulator layer** is a cheap, elegant bridge between drives and cognition:
  arousal ↑ → planning depth ↓, softmax temperature ↓ (impulsive), working-memory slots ↓, persistence ↑.
  It turns emotion into parameter changes of the very same brain rather than scripted states. Certainty and
  competence urges = intrinsic motivation.
- What we could integrate: modulator equations as design; MIT code reusable but stale.
- What we should NOT integrate: the node-net runtime / web UI.
- Scalability: all tiers.

## 4. Candidate systems — D. Curiosity, intrinsic motivation and goal formation

### Intrinsic Curiosity Module (pathak22/noreward-rl)
- URL: https://github.com/pathak22/noreward-rl
- License: MIT (LICENSE file present per page summary; licence text not read directly — treat as likely MIT)
- Language: Python (TensorFlow 1)
- Activity/maintenance: archival (10 commits, 2017 paper code); ~1.5k stars
- Architecture: Pathak et al. 2017 (ICML). Learn a feature encoder φ via an *inverse model* (predict action
  from φ(s_t), φ(s_t+1)) so features ignore things the agent can't affect; a *forward model* predicts
  φ(s_t+1) from φ(s_t), a_t; intrinsic reward = η/2 · ‖φ̂(s_t+1) − φ(s_t+1)‖².
- Scientific concept modeled: curiosity as prediction error on controllable features.
- Computational cost: two small networks per step — too much per creature at scale.
- What we can learn: the inverse-model trick is the important idea: **be curious only about what your
  actions influence** (avoids the "noisy TV" trap — a creature staring at a waterfall forever). In symbolic
  form: novelty bonus only for (action, object) pairs whose outcome is uncertain *and* action-dependent.
- What we could integrate: idea; MIT code if prototyping.
- What we should NOT integrate: per-creature deep forward models.
- Scalability: idea: all tiers.

### Random Network Distillation (openai/random-network-distillation)
- URL: https://github.com/openai/random-network-distillation
- License: unverified (licence file not retrievable); repository archived 2026-04-08
- Language: Python (TensorFlow)
- Activity/maintenance: archived; ~930 stars
- Architecture: Burda et al. 2018. A fixed random network f maps observations to features; a predictor
  network g is trained to match f; intrinsic reward = ‖g(s) − f(s)‖². Frequently seen states are well
  predicted → low novelty. It is a learned, generalising *visit counter*.
- Scientific concept modeled: novelty / count-based exploration.
- Computational cost: two small nets per step.
- What we can learn: the cheap equivalent is count-based novelty: bonus = c / sqrt(1 + n(concept or
  place)) using the visit counts we already store on graph nodes; generalisation comes from concept similarity.
- What we could integrate: count-based version (trivial).
- What we should NOT integrate: networks.
- Scalability: count version all tiers.

### Explauto (Inria FLOWERS): IAC, R-IAC, goal babbling
- URL: https://github.com/flowersteam/explauto
- License: GPL-3.0 (flag)
- Language: Python
- Activity/maintenance: dormant; latest commit 2020-07-16; ~70 stars
- Architecture: Oudeyer, Kaplan & Hafner 2007 IAC ("Intrinsic Motivation Systems for Autonomous Mental
  Development", IEEE TEC): sensorimotor space is recursively split into regions (split when a region holds too
  many exemplars, choosing the cut that maximises dissimilarity of children); each region keeps a history
  of prediction errors; **learning progress** LP = (mean error over older window) − (mean error over recent
  window); the agent picks regions with highest LP (with ε-random). R-IAC (Baranes & Oudeyer 2009) adds
  multi-resolution regions and goal-space exploration. Goal babbling (SAGG-RIAC) samples *goals* in outcome
  space instead of motor commands.
- Scientific concept modeled: developmental curiosity; self-organised curricula ("Playground Experiment":
  a robot dog autonomously went from flailing → biting → "talking" to another robot).
- Computational cost: per decision O(#regions) ≈ tens of floats; per outcome: append error to one region.
- What we can learn: **learning progress is the best single curiosity signal for believable development**:
  children ignore both trivially mastered and hopelessly hard things and gravitate to the "zone of proximal
  development". Mastered topics become boring → developmental stages appear on their own.
- What we could integrate: the algorithm (re-implement; GPL code must not be copied).
- What we should NOT integrate: any explauto source.
- Scalability: ~10–30 coarse "skill regions" per creature → all tiers down to LOD2.

### ALP-GMM teacher (flowersteam/teachDeepRL) and MAGELLAN
- URL: https://github.com/flowersteam/teachDeepRL ; https://github.com/flowersteam/MAGELLAN
- License: MIT (both)
- Language: Python
- Activity/maintenance: teachDeepRL ~94 stars (2019 paper code, date unverified); MAGELLAN ~15 stars (2025)
- Architecture: Portelas et al. 2019: track absolute learning progress |competence_now − competence_before|
  per sampled task parameter; fit a Gaussian mixture over (task-params, ALP); sample new tasks from components
  with high mean ALP (plus 20% random). MAGELLAN (Gaston et al. 2025) lets an LLM agent predict its own
  competence and LP over a large language-described goal space by generalising across semantically similar goals.
- Scientific concept modeled: automatic curriculum learning; metacognitive estimation of one's own progress.
- Computational cost: ALP bookkeeping is trivial; GMM refits occasional.
- What we can learn: **absolute** LP also captures *forgetting* (competence dropping → renewed interest),
  e.g. an elder returns to practising a skill once it begins to fade. MAGELLAN shows how an LLM-backed focal
  agent could pick self-goals ("learn to fish") using LP estimates the deterministic sim exports.
- What we could integrate: ALP formula; the "teacher" idea for parent creatures selecting tasks for children.
- What we should NOT integrate: GMM/LLM machinery for background agents.
- Scalability: ALP: all tiers; MAGELLAN-like: LOD0 only.

### Empowerment (concept block)
- URL: no canonical repo verified; key papers: Klyubin, Polani & Nehaniv 2005 ("Empowerment: a universal
  agent-centric measure of control"); Salge, Glackin & Polani 2014 review; Mohamed & Rezende 2015 (variational).
- License: n/a
- Language: n/a
- Activity/maintenance: active research area.
- Architecture: empowerment = channel capacity between an agent's n-step action sequences and its resulting
  future state, max_p(a) I(A; S'). In deterministic discrete worlds it collapses to log2(number of distinct
  states reachable in n steps).
- Scientific concept modeled: drive to keep options open / gain control (cf. competence urge, autonomy).
- Computational cost: exact = exponential in n; heuristic = count reachable cells via BFS (n≤3) ~ hundreds of ops.
- What we can learn: a cheap "stay where you have options" pressure produces lifelike behaviour: avoid corners
  and cliffs, prefer holding tools, value money/allies (all increase reachable futures).
- What we could integrate: proxy features (tools held, allies near, exits available) as an empowerment term.
- What we should NOT integrate: exact computation.
- Scalability: proxy: all tiers.

## 5. Candidate systems — E. Development, schemas, social cognition, language

### Drescher's Schema Mechanism (hqm/jschema; also SimHacker/moollm schema skill)
- URL: https://github.com/hqm/jschema (primary reference: Drescher 1991, *Made-Up Minds*, MIT Press;
  AAAI-87 paper https://cdn.aaai.org/AAAI/1987/AAAI87-052.pdf)
- License: unverified (no licence shown)
- Language: Java (Processing + jbox2d microworld, JRuby console)
- Activity/maintenance: tiny (0 stars); date unverified
- Architecture: knowledge = **schemas**: (context items) — action → (result items). Every schema keeps an
  *extended context* and *extended result*: for every other binary item, statistics of how often the result
  follows the action when that item is on vs. off ("marginal attribution"). When an item is significantly
  more relevant, the mechanism *spins off* a more specific schema (adding the item to context or result).
  Reliable schemas can be chained into composite actions (goal-directed plans). When a schema is unreliable
  but sometimes works, the mechanism invents a **synthetic item** — a new hidden concept meaning "whatever
  makes this schema work" — which is how it re-derived object permanence (an item standing for "the object
  is there though unseen"). Recent work "Schema Mechanisms 2.0 for Developmental AI" (Springer 2025) revisits it.
- Scientific concept modeled: Piagetian sensorimotor development; constructivist causal learning;
  object permanence emerging from experience.
- Computational cost: the original is very expensive (statistics for every item × every schema); bounded
  versions with sparse candidate items are affordable.
- What we can learn: our action→outcome edges with context conditions *are* schemas. Two valuable
  mechanisms: (1) **marginal attribution** = keep per-edge counts conditioned on a few candidate context
  features, spin off a context-specific edge when a feature is predictive ("fire burns *when touched*, not when
  near"); (2) **synthetic items** as a principled way for a creature to invent hidden causes (spirits, curses,
  germs!) — a gift for emergent superstition/religion in a god game.
- What we could integrate: algorithm, bounded (top-k candidate context features, spin-off thresholds).
- What we should NOT integrate: unbounded statistics over all items.
- Scalability: bounded: 1k–100k; synthetic-item invention only LOD0/LOD1.

### Machine Theory of Mind / ToMnet (CILAB-MA/Machine_ToM; variants Nik-Kras/ToMnet-N)
- URL: https://github.com/CILAB-MA/Machine_ToM ; https://github.com/Nik-Kras/ToMnet-N
- License: MIT (Machine_ToM); ToMnet-N unverified
- Language: Python
- Activity/maintenance: small reimplementations (~27 stars); dates unverified
- Architecture: Rabinowitz et al. 2018 (ICML), "Machine Theory of Mind": a *character net* embeds past
  trajectories of a particular agent into e_char; a *mental-state net* embeds the current episode into
  e_mental; a *prediction net* predicts next action, goal consumption and successor representation.
  Trained by meta-learning across populations of agents; passes a Sally–Anne-style false-belief test.
- Scientific concept modeled: learned ToM; individual "character" embeddings.
- Computational cost: neural — offline only.
- What we can learn: the clean decomposition *trait model (slow, per individual) + current mental-state
  model (fast, per episode)* is what each creature should hold about others. Ours: per-known-individual
  small record (traits: aggressiveness, generosity, reliability as HGF beliefs) + current belief/goal guess.
- What we could integrate: decomposition; MIT code not needed.
- What we should NOT integrate: neural ToM per creature.
- Scalability: decomposition: all tiers (cap number of modelled individuals, Dunbar-like ~5/15/50/150 layers).

### Bayesian inverse planning: Plinf.jl (and the BToM tradition)
- URL: https://github.com/ztangent/Plinf.jl
- License: unverified (licence file not retrievable)
- Language: Julia (Gen probabilistic programming + PDDL)
- Activity/maintenance: latest commit 2023-10-21; ~44 stars (successor work moved to related packages by the
  same author — unverified)
- Architecture: Baker, Saxe & Tenenbaum 2009 / 2017 ("Rational quantitative attribution of beliefs, desires
  and percepts"): observers assume others act approximately rationally (Boltzmann: P(a|g) ∝ exp(β·Q_g(a)));
  goal posterior P(g | actions) ∝ P(g) Π P(a_t | g). Plinf (Zhi-Xuan et al. 2020, "Online Bayesian Goal
  Inference for Boundedly-Rational Planning Agents") models the observed agent as a *bounded* planner
  (partial search, replanning) and does sequential inverse planning with particle filtering, so it can infer
  goals even from failed or suboptimal plans.
- Scientific concept modeled: theory of mind as inverse planning; goal and belief attribution.
- Computational cost: per observer per observed agent: O(#candidate goals × cost of evaluating Q) per step.
  With 3–8 candidate goals and cheap distance-based Q, ~100 flops.
- What we can learn: a tiny version is very affordable: "the stranger walks toward our granary, not toward
  the well → he probably wants grain" — with Q approximated by path-distance reduction toward each goal.
  Enables suspicion, helping and deception to emerge.
- What we could integrate: the Bayesian goal-update rule with 3–8 hypotheses from a goal vocabulary.
- What we should NOT integrate: Gen/PDDL planning at runtime.
- Scalability: 1k agents × 5 watched others fine; 100k only for agents in active social encounters.

### EGG (Emergence of lanGuage in Games) and naming-game frameworks (Babel2)
- URL: https://github.com/facebookresearch/EGG ; https://github.com/dwarfmaster/Babel2
- License: EGG MIT (parts BSD-3), archived 2026-08-10; Babel2 Apache-2.0
- Language: EGG Python (PyTorch); Babel2 Common Lisp
- Activity/maintenance: EGG archived (~320 stars); Babel2 mirror ~7 stars, date unverified
- Architecture: EGG: sender/receiver neural agents trained (REINFORCE or Gumbel-softmax) to communicate over a
  discrete channel in referential/reconstruction games. Babel2 (Steels' group): language games incl. the
  **naming game** — each agent keeps a lexicon of (word, meaning, score); speaker names a topic with its
  highest-scoring word (inventing one if none); on success both raise that word's score and *lower competing
  words* (lateral inhibition); on failure the speaker lowers it and the hearer adopts the word. A population
  converges to a shared vocabulary with no central authority. Fluid Construction Grammar extends to grammar.
- Scientific concept modeled: cultural emergence of conventions, lexicons, grammar.
- Computational cost: naming game: a hash-map lookup and a few score updates per interaction.
- What we can learn: **naming-game lexicons are the right cheap model of language for us** — each creature's
  words are just more nodes in its associative graph, linked to concepts with scores; dialects and language
  drift emerge between isolated villages; a parent's word "HOT!" becomes a learned cue for danger.
- What we could integrate: naming game + lateral inhibition (trivial to re-implement); Apache/MIT safe.
- What we should NOT integrate: neural emergent-communication training per creature.
- Scalability: all tiers (lexicon capped, e.g. 50–2000 words by age/LOD).

### Minigrid / BabyAI (Farama-Foundation/Minigrid)
- URL: https://github.com/Farama-Foundation/Minigrid
- License: MIT
- Language: Python
- Activity/maintenance: active; latest commit 2026-09-10; ~2.5k stars
- Architecture: lightweight grid-worlds (keys, doors, balls, boxes, lava) under the Gymnasium API; BabyAI
  levels issue synthetic grounded-language instructions with a curriculum and a "bot" expert for imitation.
- Scientific concept modeled: grounded language learning, sample efficiency, curriculum.
- Computational cost: environment only.
- What we can learn: BabyAI's central finding — learning grounded language from scratch is enormously
  sample-hungry (hundreds of thousands of demonstrations) — is a warning: do not try to *learn* language
  neurally inside creatures. Use symbolic grounding (naming game) and leave fluent language to the LLM layer.
- What we could integrate: as a **test harness** for prototyping creature brains offline in a headless grid
  (lava = fire!) before integrating.
- What we should NOT integrate: as runtime.
- Scalability: N/A.

### iCub software (robotology/icub-main) and developmental robotics
- URL: https://github.com/robotology/icub-main
- License: mixed GPL-2.0 and BSD (per repo; per-file — flag)
- Language: C++ (YARP middleware)
- Activity/maintenance: maintained; ~120 stars; date unverified
- Architecture: control, perception and kinematics stack for the iCub child-sized humanoid; research on top
  includes developmental curricula (reaching → grasping → tool use), imitation, joint attention and
  ERA ("Epigenetic Robotics Architecture", Morse et al. 2010: self-organising maps for each modality linked
  by Hebbian associations through a body-posture "hub", which reproduces infants' A-not-B and word-learning
  binding effects).
- Scientific concept modeled: embodied development, sensorimotor contingencies, joint attention.
- Computational cost: real-robot stack — irrelevant.
- What we can learn: ERA is a nice cheap mechanism: **Hebbian links between modality maps via a shared hub**
  explain how a word heard while looking at an object gets bound, and how habits cause A-not-B errors.
- What we could integrate: ideas only (GPL portions).
- What we should NOT integrate: the robot stack.
- Scalability: N/A.

## 6. Already-known systems: only the specific mechanisms worth stealing

(Not re-reviewed; listed because a concrete mechanism slots into the proposal below.)

- **ACT-R base-level learning** — activation of a memory chunk B = ln(Σ_j t_j^(−d)), d≈0.5, over the ages
  t_j of past uses. One formula gives the power law of forgetting *and* the spacing effect. We use a cheap
  approximation (stored count + last-use tick) for edge/episode retrieval and pruning priority.
- **Soar chunking** — compile a successful deliberate reasoning chain into a single production. Our analogue:
  a model-based plan that succeeds repeatedly is cached as a model-free habit entry (habitisation).
- **LIDA / GWT** — codelets form coalitions that compete for the workspace; the winner is broadcast and
  becomes eligible for learning. Our K-slot workspace is this, minus the codelet machinery.
- **Leabra / PVLV (emergent)** — Pavlovian learning split into a "primary value" system (learns at the US)
  and a "learned value" system (fires at CSs) with dopamine combining them; justifies keeping Pavlovian value
  separate from instrumental value.
- **Spaun / Nengo BG** — action selection as a basal-ganglia winner-take-all over "utility" of production
  rules; same as our selector.
- **Generative agents (Park et al. 2023)** — memory retrieval score = recency + importance + relevance, plus
  periodic "reflection". For our LOD0 agents, importance = |prediction error| × |valence| from the sim (no LLM
  needed to score importance), and reflection = sleep consolidation + rule crystallisation.
- **pymdp / active inference** — expected free energy = pragmatic + epistemic value; we approximate the
  epistemic term with uncertainty/novelty bonuses.
- **Thousand Brains / Monty** — many parallel "columns" voting on object identity; inspiration for
  generalisation by feature-signature similarity, not adopted directly.

## 7. Summary comparison

| Candidate | Licence (risk for closed game) | Runtime per creature? | Main contribution to us |
|---|---|---|---|
| Brian2 / NEST / GeNN / BindsNET / snnTorch / Norse | CeCILL / GPL / LGPL / AGPL / MIT / LGPL | No (offline validation) | trace-based STDP, distal-reward eligibility, batched kernels |
| Creatures Norn brain (openc2e) | LGPL engine; ideas free | Yes, in sparse form | susceptibility × reward chemical; STW/LTW; instincts as pre-training |
| Rescorla–Wagner / Pearce–Hall (statsrat) | MIT / science | Yes | core cue→outcome rule, blocking, adaptive attention |
| TD / dopamine RPE | science | Yes | temporal credit assignment, second-order fear |
| Spiking amygdala (Nengo) | Nengo GPL-2 (check) | No | fear vs context-gated extinction → renewal |
| Modern Hopfield | BSD | LOD0/1 episodic recall | pattern completion, β as sharpness knob |
| SDM / torchhd VSA | unverified / MIT | summaries only | similarity-based generalisation, compact belief vectors |
| HTM | AGPL | No | bursting = surprise; permanence thresholds |
| Episodic control | MIT | LOD0/1 | one-shot "what worked last time" |
| Successor representation / RatInABox | MIT | LOD0/1 (sparse) | predictive maps, latent learning |
| GPR basal ganglia | unverified | Yes (tiny) | persistence/hysteresis, tonic dopamine vigour |
| MB/MF arbitration | science | Yes | habit vs deliberation switch; LOD knob |
| pyhgf (HGF) | MIT | Yes (few beliefs) | volatility-adaptive learning rate |
| Predictive coding / RxInfer / ActiveInference.jl | unverified / MIT / MIT | No | prediction-error gating; EFE decomposition |
| Shimmer (GWT) | MIT | idea | K-slot salience workspace |
| PsyNeuLink | Apache-2.0 | equations | DDM/LCA hesitation; expected value of control |
| pyClarion | MIT | idea | rule extraction → explicit, teachable beliefs; drives |
| MicroPsi2 | MIT | Yes (scalars) | emotions as modulators of cognition |
| ICM / RND | MIT / unverified | count proxies | controllable-novelty curiosity |
| Explauto (IAC/R-IAC) | GPL-3 | algorithm | learning-progress curiosity, self-organised stages |
| ALP-GMM / MAGELLAN | MIT | ALP yes; MAGELLAN LOD0 | absolute LP (incl. forgetting); LLM self-goals |
| Empowerment | science | proxies | keep-options-open drive |
| Drescher schema mechanism | unverified | bounded | marginal attribution, synthetic items (hidden causes) |
| ToMnet / Plinf (BToM) | MIT / unverified | tiny Bayesian version | trait + mental-state models; goal inference |
| EGG / Babel2 naming game | MIT / Apache-2.0 | Yes | lexicon as graph nodes; dialect drift |
| Minigrid / BabyAI | MIT | No (test harness) | offline testbed; warning on neural language learning |
| iCub / ERA | GPL-2/BSD | idea | Hebbian hub binding, A-not-B |

## 8. Recommended abstraction for our creatures: the "Associative Mind Graph" (AMG)

Design goal: the cheapest machinery in which *fire → pain → lifelong caution* (and hundreds of similar
stories) emerges from experience, is inspectable by designers and the LLM narrator, is deterministic, and
degrades gracefully across LOD tiers. No rule anywhere says "if fire: avoid".

### 8.1 Representation

**World concept vocabulary (shared, global).** Every perceivable or experienceable thing maps to a concept
ID: percept categories (FIRE, SMOKE_SMELL, CRACKLE, WOLF, RED_BERRY), outcome/"unconditioned" concepts with
innate valence (PAIN −1.0, BURN −1.0, SATIATION +0.6, WARMTH +0.3, COMFORT +0.5, FALL −0.7, NAUSEA −0.8),
actions (TOUCH, APPROACH, EAT, FLEE, HIDE, GIVE), places (landmarks), individuals (agent IDs), words
(lexicon tokens), and *synthetic* concepts invented by creatures (Drescher-style hidden causes). Each
concept carries a 64-bit **feature signature** (hash of features such as glowing, hot, orange, moving,
furry, large-teeth). Similarity = 1 − Hamming/64, used for generalisation to never-seen things. Only outcome
concepts have hard-coded valence; everything else acquires value through learning. This vocabulary is the
one place where authoring leaks in — keep it *perceptual*, never behavioural (FIRE yes, DANGEROUS_THING no).

**Per-creature state (structure-of-arrays, fixed-point int16 weights with scale 1/4096):**

| Component | Contents | Size (adult human, LOD0/LOD1) |
|---|---|---|
| Node table | known concept IDs, last-active tick, activation intensity, Pearce–Hall associability α_i, visit count n_i | ~8 B × 500–5000 nodes |
| Edge table (sorted by source) | target, w_fast, w_slow, evidence count, flags (flashbulb, crystallised, taught-by, context-specific), last-used tick | ~12 B × 2k–50k edges |
| Extinction table | (cue, context) → w_ext | ~6 B × 100–1000 |
| Habit cache | (context-cluster, action) → Q_MF, plus reliability EMA | ~6 B × 200–2000 |
| Workspace | K slots of concept refs + salience | ≤ 8 × 4 B |
| Drives & modulators | 10 drives (u8) + arousal, mood, tonic dopamine, fatigue | ~16 B |
| Episodic buffer | ring of salient events: tick, place, ≤4 cues, action, outcome, δ, valence | 32 B × 16–2000 |
| Social models | per known individual: trait beliefs (HGF μ,σ for trust/aggression/generosity), last goal guess, "has seen" tags | 32–64 B × 5–150 |
| Lexicon | (word, concept, score) | ~8 B × 50–2000 |
| Learning-progress regions | per skill region: two error EMAs | 8 B × 10–30 |

**Priors and deltas.** Each creature's effective weight is w = w_prior(species, culture) + w_delta(own).
The species prior encodes instincts and *prepared* associability (primates: snakes ×3 learning rate, not
innate fear — matching Öhman & Mineka's preparedness findings); the culture prior encodes taught common
knowledge. Only deltas are stored per creature (copy-on-write). This is the single biggest memory saving and
also how culture enters the brain.

**Eligibility without per-edge traces.** Eligibility lives on the *source node*:
e_i(t) = I_i · exp(−(t − t_active_i)/τ_e), computed lazily via a lookup table when an outcome happens.
This is Izhikevich's distal-reward trace / Creatures' susceptibility at node granularity — no per-tick cost.

### 8.2 Learning rules (all prediction-error driven; "learn only what surprised you")

1. **Pavlovian (cue → outcome), Rescorla–Wagner with summed prediction.** When an outcome concept j
   occurs with intensity I_j (or is *expected* but fails to occur):
   P_j = Σ_i e_i · w_ij over eligible cues i (plus similarity-weighted generalisation from neighbours);
   δ_j = λ(I_j) − P_j;
   Δw_fast_ij = clamp(α_age · α_i · κ_sign(δ) · prep_i) · δ_j · e_i.
   κ_− > κ_+ gives negativity bias (one burn outweighs one pleasant warmth); prep_i is species preparedness.
2. **Attention (Pearce–Hall).** α_i ← γ·|δ| + (1−γ)·α_i, γ≈0.3. Novel cues start at α=1 (attention to new
   things); boring repeated cues drop (latent inhibition — pre-exposed things are learned about slowly).
3. **Temporal chaining (TD).** If a cue i predicts another cue k that itself carries value V_k, update the
   predictive edge i→k and propagate value: V_i ← V_i + α·(γ·V_k − V_i). Smoke smell → fire → pain.
4. **Extinction is new, context-gated learning.** When δ < 0 for a cue with w > 0, update
   w_ext(cue, context) instead of w (context = coarse place/situation cluster, e.g. HEARTH, FOREST, VILLAGE).
   w_ext decays faster than w. Net fear output = max(0, w − w_ext(ctx)). This yields renewal, spontaneous
   recovery and reinstatement (Bouton), mirroring the amygdala/ITC circuit.
5. **Instrumental (context, action → outcome) schemas.** The same Pavlovian rule on composite source nodes
   "ACTION|CUE" (e.g. TOUCH|FIRE). Marginal attribution (Drescher): each such edge keeps success counts
   conditioned on ≤3 candidate context features; if one feature makes the outcome much more reliable, spin
   off a specific edge (TOUCH|FIRE → BURN is reliable; APPROACH|FIRE → BURN is not, but APPROACH|FIRE|COLD →
   WARMTH is).
6. **Habits (model-free).** After each action, Q_MF(ctx, a) += η_hab · (r + γ·V(next) − Q_MF). Reliability
   R_MF = 1 − EMA(|RPE|). Habit strength also grows with repetition count (overtraining).
7. **Two-timescale consolidation (sleep).** During sleep, replay the top-N episodic events by
   |δ|·|valence| (+ recency): for each, transfer ρ·(w_fast − w_slow) into w_slow, with ρ = 0.3 + 0.6·tag,
   tag = min(1, |δ|·I) (synaptic tagging: surprising, intense events consolidate strongly). w_fast then
   relaxes toward w_slow with τ ≈ days. Very high tags set the *flashbulb* flag (forgetting τ ×10).
8. **Forgetting.** w_slow decays toward the prior with τ_forget = τ_0 · (1 + log(1 + uses)) (ACT-R-like:
   rehearsed knowledge persists), ×10 for flashbulb edges. Pruning removes edges with |w − prior| tiny.
9. **Crystallisation (CLARION-style rule extraction).** When an edge has |w_slow| > 0.4, evidence ≥ 3 and
   stable sign, mark it *explicit*: it becomes a sayable belief ("fire burns"), teachable to others and
   visible to the LLM narrator.
10. **Social / observational learning.** Seeing another creature experience an outcome after a cue applies
    rule 1 with α × trust(observed) × 0.5 (observational fear learning, Olsson & Phelps 2007). Being *told*
    ("HOT!" + pointing at fire, or a crystallised belief transmitted in conversation) writes an edge with
    α × trust(speaker) and flag taught-by, capped at w ≤ 0.6 until confirmed by own experience.
11. **Lexicon (naming game).** Hearing word W while concept C is in the workspace: score(W,C) += a; competing
    words for C −= a·inhibition. Words become cues: WORD_HOT acquires PAIN prediction through rule 1 if
    spoken before pain — this is how parental warnings work *mechanistically*.
12. **Belief tracking (HGF-lite).** For a handful of important uncertain quantities (is the well reliable?
    is Bran trustworthy?) keep μ, σ and a volatility estimate; precision-weighted update. Regime changes
    (drought, betrayal) raise volatility → faster belief revision.

### 8.3 Decision loop (runs every 0.5–2 s of game time, not every tick)

1. **Perceive → activate.** Visible/heard things activate concept nodes with intensity (distance, size).
   Unknown things activate their nearest known concepts by signature similarity (generalisation).
2. **Workspace competition (GWT).** Candidates: active percepts, top drives, retrieved memories, current
   goal. Salience = |predicted value| + novelty + drive relevance + social relevance. Top K win. Only
   workspace items get full learning rate (others ×0.2) and enter planning. Attention narrows under high
   arousal (K drops by 1–2).
3. **Generate options** from affordances of workspace objects + current goal + habits (4–12 options).
4. **Score each option.**
   - Habit value Q_MF(ctx, a).
   - Goal-directed value Q_MB(a) = Σ_o P(o | a, ctx) · U(o | drives), searched to depth d with beam 3 over
     schema edges; U depends on current drives, so a sated creature stops valuing food instantly
     (outcome-devaluation sensitivity — habits lack it).
   - Pavlovian bias: approach-type actions toward cue c get +k_pav · V(c); negative V suppresses approach and
     boosts flee/freeze (Pavlovian–instrumental transfer). Anxiety = large k_pav.
   - Curiosity: c_LP · learning-progress(region of a) + c_nov / sqrt(1 + n) restricted to action-dependent
     outcomes (ICM's lesson) + empowerment proxy.
   - Persistence bonus for the incumbent action (basal-ganglia hysteresis); effort cost.
5. **Arbitrate:** w_MB = sigmoid(b · (R_MB − R_MF) − cost_plan · fatigue); total = w_MB·Q_MB + (1−w_MB)·Q_MF +
   Pavlovian + curiosity + persistence.
6. **Select** with softmax (temperature from arousal/age/tonic dopamine) or a race (DDM-like) when the top
   two are close → visible hesitation.
7. **Act; on outcome, run learning rules; store episode if |δ| > θ_episode.**

**Cost estimate (compiled, cache-friendly SoA):** activation ~20 nodes; workspace sort ~20 items;
8 options × (1 + 3 + 9) lookahead nodes × ~6 edges ≈ 600 edge reads; learning ~50 edge writes on outcome
events only. ≈ 2–6 µs per decision at LOD0/1, < 0.5 µs at LOD2 (no lookahead). At 1 decision/s/creature:
10k LOD1 creatures ≈ 60 ms CPU per game-second on one core — fine with a job system; LOD3 never decides
individually.

### 8.4 Worked trace: the fire story, with numbers

Parameters for **Ana, age 3**: α_age = 0.5, κ_− = 1.6, κ_+ = 1.0, τ_e = 2 s, θ_episode = 0.3,
species prior HOT_SURFACE → BURN = 0.10, signature similarity(FIRE, HOT_SURFACE) = 0.4. FIRE is known only
visually (humans have no innate fire fear; children are drawn to it), FIRE→BURN prior = 0.

*Day 1, evening, at the hearth.*
- t = 0.0 s: FIRE enters the workspace: never interacted with, n = 0 → novelty bonus 1.0; learning-progress
  region "hot objects" is empty → high curiosity. Mother says "HOT!" (WORD_HOT, unknown word) at t = 0.5 s.
- t = 1.2 s: Ana selects TOUCH (curiosity dominates; no Pavlovian fear yet).
- t = 1.5 s: BURN occurs, intensity I = 0.9 → λ = 0.9.
- Eligibilities: FIRE e = 1.0 (still visible); TOUCH|FIRE e = exp(−0.3/2) = 0.86; WORD_HOT e = exp(−1.0/2)
  = 0.61; MOTHER/HEARTH are context (α × 0.1).
- Prediction P_BURN = 0 + 0 + 0 + generalisation 0.4 × 0.10 = 0.04 → δ = 0.86 (huge surprise).
- Novel cues have α_i = 1, so effective rate = min(1, 0.5 × 1 × 1.6) = 0.8:
  - Δw(FIRE→BURN) = 0.8 × 0.86 × 1.0 = **0.69**
  - Δw(TOUCH|FIRE→BURN) = 0.8 × 0.86 × 0.86 = **0.59**
  - Δw(WORD_HOT→BURN) = 0.8 × 0.86 × 0.61 = **0.42** (the word now *means* danger)
  - Δw(HEARTH→BURN) = 0.08 × 0.86 × 1 = 0.07 (slight unease about the hearth corner)
- Pearce–Hall: α_FIRE ← 0.3 × 0.86 + 0.7 × 1 = 0.96 (stays attention-grabbing).
- Episode stored (|δ| = 0.86 > 0.3): {hearth, [FIRE, WORD_HOT, MOTHER], TOUCH, BURN 0.9, δ 0.86}; tag =
  min(1, 0.86 × 0.9) = 0.77 → flashbulb candidate. Crying raises COMFORT-seeking; mother's comfort follows →
  MOTHER → COMFORT edge strengthens (attachment).
- Lexicon: WORD_HOT co-occurred with FIRE in the workspace → score(HOT, FIRE) = 0.2.

*Night 1, sleep consolidation.* ρ = 0.3 + 0.6 × 0.77 = 0.76 → w_slow(FIRE→BURN) = 0.52,
w_slow(TOUCH|FIRE→BURN) = 0.45, w_slow(WORD_HOT→BURN) = 0.32; flashbulb flag set on the first two
(tag > 0.7). The fast residue (0.17, 0.14, 0.10) fades over ~3 days.

*Days 2–14.* Ana sees the hearth fire every evening. Pavlovian value V(FIRE) = 0.52 × |−1.0| → approach is
suppressed (k_pav × 0.52) so she keeps ~2 m away. No burn occurs at 2 m → δ = −0.52 each evening → this goes
into w_ext(FIRE, HEARTH) with α_ext = 0.5 × 0.3 = 0.15: after 10 evenings w_ext ≈ 0.52 × (1 − 0.85^10) ≈
0.42 → net fear at the hearth 0.10: she is relaxed by the hearth. Crucially TOUCH|FIRE is *never tested
again* because avoidance prevents it — so that edge never extinguishes (the classic reason avoidance-based
fears persist).

*Age 7.* Her little brother reaches for the fire. ToM level 2 + crystallised belief "fire burns" (|w| 0.45,
evidence via observation of others) → she says "HOT!" (teaching). Her brother's WORD_HOT edge is now
learned from *her* pain history — cultural transmission without anyone coding it.

*Age 25, the forest fire.* 22 years later: w_slow(FIRE→BURN) decayed toward prior with flashbulb τ ≈ 60 y:
0.52 × exp(−22/60) ≈ 0.36. Extinction w_ext(FIRE, HEARTH) is irrelevant here: context = FOREST,
w_ext(FIRE, FOREST) = 0 → **renewal**: net fear = 0.36, × percept intensity clamp 1.5 for a huge blaze →
V = −0.54. Meanwhile TD has made SMOKE_SMELL → FIRE predictive (learned over years of cooking) so she
becomes anxious from the smell before seeing flames. Arbitration: novel, high-stakes, unreliable habits
→ model-based planning dominates: FLEE options scored by distance-from-fire outcomes; she also pulls her
child away (social goal). Nobody wrote `if fire: avoid`; and the same machinery lets her *cook* with fire
daily, because at the hearth the extinction memory and the positive APPROACH|FIRE|COLD → WARMTH and
COOK|FIRE → FOOD schemas outweigh the residual fear.

*Generalisation bonus:* first sight of a lava flow: signature similarity to FIRE 0.7 → predicted BURN ≈
0.7 × 0.36 = 0.25 → caution without any lava experience.

### 8.5 Development: one brain, age-dependent parameters

Ages for humans; animals scale by species maturity (e.g. a wolf "child" stage ≈ months).

| Parameter | Infant 0–2 | Child 3–11 | Adolescent 12–19 | Adult 20–55 | Elder 56+ |
|---|---|---|---|---|---|
| Base learning rate α_age | 0.6 | 0.5 → 0.4 | 0.4 | 0.25 | 0.25 → 0.10 |
| Aversive/appetitive asymmetry κ−/κ+ | 1.2 / 1.0 | 1.6 / 1.0 | 1.2 / 1.4 (reward-seeking) | 1.5 / 1.0 | 1.1 / 1.1 (positivity effect) |
| Extinction learning rate α_ext | 0.2 | 0.3 | 0.12 (adolescent extinction deficit, Pattwell et al. 2012) | 0.3 | 0.2 |
| Curiosity (LP + novelty) weight | 1.0 | 0.8 | 0.6 (social novelty high) | 0.35 | 0.2 |
| Workspace slots K | 1–2 | 3 → 5 | 6–7 | 7 | 7 → 4 |
| Planning depth d | 0 | 1 → 2 | 2–3 | 3–4 | 3 → 2 |
| Pavlovian/impulse weight k_pav | 1.0 | 0.8 | 0.9 | 0.5 | 0.5 |
| Habit learning η_hab / habit reliance | high reliance, no inhibition (A-not-B) | medium | medium | high (most routine) | very high |
| Object persistence τ_obj (belief in unseen objects) | 2 s → minutes (≈ 8–18 months) | days | days | days | days, but more source confusion |
| Theory-of-mind level | 0 → joint attention (~1 y) | level 1 at ~4–5 (false belief), level 2 ~7 | 2 | 2–3 | 2–3 |
| Trust in parent / peer / prestige elder | 0.95 / 0.1 / 0.3 | 0.8 / 0.4 / 0.5 | 0.4 / 0.8 / 0.4 | self-weighted | self-weighted, high teaching drive |
| Episodic store size / retrieval sharpness β | tiny (infantile amnesia: no consolidation before ~3) | growing / sharp | large / sharp | large / sharp | large / lower β → gist, false memories |
| Decision interval | ×1.5 | ×1.0 | ×0.9 | ×1.0 | ×1.3 → ×1.8 |
| Forgetting τ for new edges | short | long | long | long | shorter for new, old flashbulbs kept |

**Critical periods** (windows where α for a specific learning channel is ×2–5 and afterwards ×0.2):
- Attachment target: 6–24 months (MOTHER→COMFORT edge plasticity); for precocial animals, *imprinting* on
  the first large moving thing in the first hours/days.
- Native-lexicon phonology: 0–7 years (words learned later get lower α and an "accent" flag the LLM can render).
- Species-specific (e.g. birdsong-like dialect learning for songbirds).
- Closure can be partially reopened by extreme events (trauma, conversion), an interesting story lever.

**Emergent developmental phenomena to verify** (these must *fall out*, not be scripted): A-not-B error
(small K + strong just-formed habit), object permanence growing with τ_obj, false-belief passing (once
social models track "has seen" tags), stage-like skill sequences from learning-progress curiosity,
adolescent risk-taking, elder reliance on routine and crystallised knowledge.

**Genetics hook:** each parameter above is the product of an age curve × an individual heritable factor
(e.g. α_age × g_α, κ_− × g_anx, curiosity × g_openness), giving personality variance and evolvable minds.

### 8.6 LOD tiers: how the brain degrades

| Tier | Population (target) | Brain fidelity | Memory / creature | Notes |
|---|---|---|---|---|
| LOD0 focal | ≤ 100 | full AMG; depth 3–4; Hopfield/kNN episodic recall; ToM inverse planning; LLM narration/dialogue reads workspace, crystallised beliefs, top episodes | ~0.5–1 MB | LLM output maps back only to speech acts / concept tokens; never writes weights directly |
| LOD1 near | ≤ 10k | full rules; depth 1–2; episodic 128; social 50 | ~30–60 KB | same learning, cheaper planning |
| LOD2 far | ≤ 100k | Pavlovian + habits only (no lookahead); edges ≤ 256 deltas; 16 "scars" | ~3–5 KB | decisions every 5–10 s; learning only on salient outcomes |
| LOD3 statistical | ≤ 1M | archetype (species + culture + age band) + 8–16 scar overrides + 2 × 128 B belief hypervectors | ~300–500 B | behaviour sampled from archetype policy biased by scars; no per-tick brain |
| LOD4 aggregate | unlimited | population distributions only | — | cultural knowledge lives in the culture prior |

- **Demotion:** rank edges by |w − prior| × (1 + 3·flashbulb) × recency; keep top-K for the new tier; fold the
  rest into a hypervector summary; compress episodes into scars ("cue → outcome, valence, age at event").
- **Promotion:** rehydrate = prior + kept deltas + scars re-expanded into edges; the LLM may write *narrative
  colour* for the backstory but may not create sim facts that contradict scars.
- **Consistency guard:** a creature's 20 most important beliefs are always preserved across tiers so the
  player never sees a promoted villager "forget" their famous phobia.
- **Rough totals:** 100 × 1 MB + 10k × 50 KB + 100k × 4 KB + 1M × 400 B ≈ 0.1 + 0.5 + 0.4 + 0.4 ≈ 1.4 GB.
  Tighten via smaller caps for animals (most of the 1M will be animals with ≤ 64 edges, no lexicon/ToM).

### 8.7 Determinism and engineering notes

- Fixed-point int16 weights, integer ticks, table-based exp decay, sorted edge arrays, per-creature RNG
  seeded from (world_seed, creature_id, tick) → bit-identical replays and lockstep multiplayer possible.
- Batch updates by component (all creatures' perception, then all workspaces...) in SoA form; this is
  GPU/SIMD-friendly in the GeNN spirit.
- Stagger sleep consolidation across creatures to avoid nightly CPU spikes.
- Build a **behavioural test suite from learning psychology** and run it in a headless grid (Minigrid-like):
  acquisition curve, blocking, overshadowing, latent inhibition, second-order conditioning, extinction +
  renewal + spontaneous recovery + reinstatement, partial-reinforcement extinction effect, avoidance
  persistence, outcome devaluation (goal-directed early / habitual after overtraining), observational fear,
  A-not-B, false belief, naming-game convergence. Each is a few dozen trials and becomes a regression test.

## 9. Open questions / risks

1. **The ontology is the hidden script.** If designers create concepts like DANGER or ENEMY, behaviour is
   authored, not learned. Keep the vocabulary perceptual/physical; let value attach by learning. Open: how
   much perceptual *feature* learning (concept formation from raw features) do we need beyond fixed categories?
   Drescher-style synthetic items and signature clustering are candidate answers; both need prototyping.
2. **Legibility vs. emergence.** Creatures showed dense brains are opaque. Our graph is inspectable, but with
   10k+ edges designers still need tools: "why did she do that?" = show the top contributing edges, the
   workspace and the arbitration weights for the last decision. Budget for this debug UI early.
3. **Composite-cue explosion.** "ACTION|CUE|CONTEXT" nodes can multiply. Spin-off thresholds, ≤3 candidate
   context features and pruning must be tuned; risk of either under-specific (superstitious) or memory-blowing
   behaviour.
4. **Superstition and runaway fear.** Prediction-error learning with one-trial aversive learning and
   avoidance can create phobias that never extinguish and spread socially. That is realistic and a great god-game
   lever — but needs population-level dampers (bounded taught-weight caps, counter-evidence from peers) to avoid
   whole villages paralysed by fear.
5. **Parameter tuning.** ~40 parameters × age curves × genetics. Need automated tuning against the
   behavioural test suite plus "fun" metrics; risk of brittle balance.
6. **LOD transitions.** Promotion/demotion may produce visible personality "pops". The preserved-top-beliefs
   rule mitigates; still needs play-testing. Also save-file size with 1M minds.
7. **LLM contamination.** If LLM dialogue can teach beliefs (via speech acts), hallucinations could inject
   facts. All LLM outputs must pass through the same concept-token interface with trust caps, and the sim must
   validate referents exist.
8. **Causal vs. correlational learning.** RW/TD learn correlations; real children also learn causal structure
   (interventions, "blicket detectors", Gopnik). Marginal attribution helps; full causal-Bayes-net learning is
   probably too expensive except at LOD0. Open research question for us.
9. **Language depth.** Naming games give words, not grammar. Fluent speech is LLM-only for focal agents; for
   others, we only simulate lexicons and speech acts. Need a clear contract between the two layers.
10. **Theory of mind cost.** Inverse planning per observed agent is cheap with few goals but grows with
    crowd size; cap social attention (workspace-limited) and Dunbar-like social model counts.
11. **Determinism across platforms** if any float math slips in (e.g. softmax). Use integer/LUT softmax or a
    deterministic float library.
12. **Ethical/design framing.** Creatures that genuinely learn pain and fear invite player cruelty; the god-game
    framing makes this a feature, but consider how suffering is surfaced (and whether trauma should be
    simulated at full intensity for children).
13. **Licence hygiene.** Many of the best references are GPL/AGPL (Explauto, BindsNET, htm.core, NEST, Nengo,
    iCub parts). Policy: algorithms re-implemented from papers in our own words/code only; no source copied;
    keep a provenance log per algorithm.

## 10. Key references (papers behind the mechanisms)

- Rescorla & Wagner 1972; Pearce & Hall 1980; Mackintosh 1975 — Pavlovian learning and associability.
- Sutton 1988; Schultz, Dayan & Montague 1997 (Science) — TD and dopamine RPE; Dabney et al. 2020 (Nature) —
  distributional RPE.
- Izhikevich 2007 (Cerebral Cortex) — distal reward via STDP eligibility + dopamine; Frémaux & Gerstner 2016
  — three-factor rules review; Gerstner et al. 2018 — eligibility traces and neuromodulated plasticity.
- Hebb 1949; Oja 1982 (normalised Hebbian); Bienenstock, Cooper & Munro 1982 (BCM sliding threshold) — useful
  if we ever need unsupervised feature learning: BCM's sliding threshold is the classic stabiliser.
- Bouton 2004 — context and extinction; Gershman, Blei & Niv 2010 — latent causes; Duggins & Eliasmith 2024 —
  spiking amygdala.
- Gurney, Prescott & Redgrave 2001 — basal-ganglia selection; Daw, Niv & Dayan 2005; Lee, Shimojo &
  O'Doherty 2014; Keramati et al. 2011 — habit/goal arbitration; Dickinson 1985 — outcome devaluation.
- Dayan 1993; Stachenfeld et al. 2017; Momennejad et al. 2017 — successor representation.
- Mathys et al. 2011 — HGF; Rao & Ballard 1999; Friston 2010 — predictive coding / free energy.
- Baars 1988; Dehaene & Changeux 2011; VanRullen & Kanai 2021 — global workspace.
- Oudeyer, Kaplan & Hafner 2007 — IAC; Baranes & Oudeyer 2009/2013 — R-IAC/SAGG-RIAC; Portelas et al.
  2019 — ALP-GMM; Pathak et al. 2017 — ICM; Burda et al. 2018 — RND; Klyubin et al. 2005 — empowerment.
- Drescher 1991 — schema mechanism; Piaget 1952; Gopnik & Wellman — causal learning in children.
- Rabinowitz et al. 2018 — ToMnet; Baker, Saxe & Tenenbaum 2009/2017 — Bayesian ToM; Zhi-Xuan et al. 2020 —
  bounded inverse planning.
- Steels 1995/2011 — naming game, language games; Chevalier-Boisvert et al. 2019 — BabyAI.
- Olsson & Phelps 2007 — social fear learning; Öhman & Mineka 2001 — preparedness; Pattwell et al. 2012 —
  adolescent fear-extinction deficit; Hensch 2005 — critical periods.
- Grand, Cliff & Malhotra 1997; Grand 1997 — Creatures.
- Blundell et al. 2016 — model-free episodic control; McClelland, McNaughton & O'Reilly 1995 — complementary
  learning systems; Ramsauer et al. 2020 — modern Hopfield; Kanerva 1988 — SDM.
