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

