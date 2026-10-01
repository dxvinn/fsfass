# 02 — Language Emergence, Local Knowledge & Belief Spread, Memory

Research track 4/5/6 for the god-simulator. Research only, no game code.
Compiled 2026-10-01. Repository facts (license, language, stars, activity) were checked on the GitHub
page of each project during this session unless marked "unverified". Star counts are approximate
snapshots. arXiv, emergent-languages.org, mewo2.com and dwarffortresswiki.org were blocked by the
network proxy during research, so paper details come from search snippets, source files on GitHub,
and well-established literature; anything I could not confirm is flagged.

Core stance carried through the whole document:

- The **deterministic simulation owns the truth**. Every agent owns only a *belief store* (what it
  thinks is true, with provenance and confidence) and a *bounded memory* (what it can recall).
- LLMs see **only** an agent's belief store / memory, never the world state, so a narrated or
  conversing agent cannot become omniscient by accident.
- All three systems (language, knowledge, memory) must degrade gracefully across LOD tiers:
  T0 focus agents (tens, LLM-eligible), T1 named agents (1–10k, full deterministic model),
  T2 background individuals (~100k, compressed), T3 aggregate populations (statistics only).

---

## Part A — Key ideas and papers (the science, condensed)

### A.1 Language emergence

**Steels' naming game / language games (1995 onward).** Two agents, a speaker and a hearer, share
a context. The speaker picks a topic, looks up (or invents) a word, utters it; the hearer guesses
the referent; feedback says success/failure; both update. The "minimal naming game" (Baronchelli,
Felici, Loreto, Caglioti, Steels 2006; Baronchelli's 2016 "gentle introduction") is the cleanest form:

- Each agent has an inventory of words for one object (initially empty).
- Speaker: if inventory empty, invent a new word; otherwise utter a random word from it.
- Hearer has the word → **success**: both speaker and hearer delete all other words, keeping only
  the winner. Hearer lacks it → **failure**: hearer adds the word.
- In a fully mixed population of N agents, total vocabulary first explodes (peak ~N^1.5 words) and
  then collapses to consensus in time ~N^1.5 interactions per agent-scale.
- On low-dimensional lattices consensus is reached by *coarsening*: local dialect domains form and
  their boundaries random-walk; convergence time scales roughly as N^(1+2/d), i.e. very slow on a
  2D map. **This is exactly the mechanism we want for geographic dialects**: neighbors agree,
  distant villages don't, and boundaries sit at low-contact places (mountains, rivers, borders).
- The *lateral inhibition* variant keeps a score per (word, meaning): success → winner +δ_inc,
  competitors −δ_inh; failure → speaker's word −δ_dec. Words with score ≤ 0 are deleted. This is
  the "weighted lexicon" representation we will use.

**Talking Heads (Steels & Kaplan, 1999–2001; Steels 2015 book).** Robots with cameras played
guessing games about shapes on a whiteboard; each agent grew its *own* perceptual categories
(discrimination trees over color/position features) and lexicon; the lexicon and the categories
co-evolved. Lesson: grounding means words map to an agent's *own* concepts, which can differ
between agents (one agent's "red" may span what another splits into two). For us: meanings in
the lexicon point at simulation concept IDs, but agents may have coarser or finer concept
partitions (e.g. a forest culture distinguishes 6 tree kinds, a steppe culture 1).

**Iterated learning (Kirby 2001; Kirby, Cornish & Smith 2008 PNAS; Griffiths & Kalish 2007).**
Language passes through a *transmission bottleneck*: each generation learns from a finite sample
of the previous generation's output. Under a bottleneck, holistic (unanalyzable) languages are
unstable and compositional ones (reusable parts) win because they can be reconstructed from
partial data. Griffiths & Kalish proved that with Bayesian learners who *sample* hypotheses from
their posterior, the chain converges to the **prior** — i.e. what languages end up looking like
is a mirror of learners' biases. With MAP learners, biases get *amplified*. Practical lesson:
children learning from the adults around them is the main engine of drift, and the "bias" knob
(regularization, preference for short words, preference for compositional names) determines
what the world's languages trend toward.

**Neural emergent communication (Lazaridou et al. 2017; Havrylov & Titov 2017; Kottur et al. 2017
"Natural language does not emerge naturally"; Chaabouni et al. 2020 on compositionality and
generalization; Rita et al. 2022 "Emergent Communication: Generalization and Overfitting in Lewis
Games").** Sender/receiver nets trained with REINFORCE or Gumbel-Softmax on referential games
reach high accuracy but produce codes that are often non-compositional, anti-efficient (long
messages for frequent meanings unless length pressure is added) and not human-interpretable.
Population training (several speakers × listeners, periodic agent resets = neural iterated
learning, Ren et al. ICLR 2020) increases structure. **For a game: interesting as a lab, wrong as
a runtime system** — gradient training per agent is far too expensive and the outputs are
illegible to players.

**Sociolinguistics of change.**
- *Lexical diffusion* (Wang 1969): sound changes spread word by word, frequent words first (for
  reductive changes) — we can model a change as a per-word flag spreading through the lexicon.
- *S-curve adoption*: innovations spread slowly, then fast, then slowly; an emergent property of
  naming-game-like dynamics with prestige bias.
- *Utterance selection model* (Baxter, Blythe, Croft, McKane 2006): each speaker holds a frequency
  distribution over variants; after each interaction, it moves toward what it heard by a small
  step λ, weighted by the speaker's social weight. Neutral drift alone already yields eventual
  fixation; *weighted* interactors (prestige) produce directional change.
- *Nettle 1999; Raviv, Meyer & Lev-Ari 2019*: small, tight communities can sustain idiosyncratic
  variants and drift faster; larger communities produce more systematic, regular languages.
- *Stubborn vs. flexible learners*: Limor Raviv's ABM tutorial shows a minority of non-updating
  agents can block convergence — a cheap personality hook ("traditionalists").
- *Contact*: loanwords are adapted to the borrowing language's phonology (nearest native phoneme);
  heavy contact produces pidgins (reduced lexicon, no morphology) and creoles once children learn
  them natively.

### A.2 Knowledge, beliefs and information spread

**Rumor models.** Daley & Kendall (1964/65): ignorant (X), spreader (Y), stifler (Z). Pairwise
contacts: Y meets X → X becomes Y; Y meets Y → both become Z; Y meets Z → the Y becomes Z.
Maki & Thompson (1973) simplify: only the *initiating* spreader is converted on contact with a
Y or Z. Mean-field for Maki–Thompson (fractions i, s, r; contact rate normalized):

    di/dt = −s·i
    ds/dt =  s·i − s·(s + r) = s·(2i − 1)
    dr/dt =  s·(1 − i)

The final ignorant fraction θ solves θ = exp(−2(1 − θ)), θ ≈ 0.2032 (Sudbury 1985). So **a
rumor that stops when tellers find people who already know it reaches only ~80% of a well-mixed
population** — an elegant, physically plausible reason why not everyone knows everything. Unlike
SIR, there is no epidemic threshold: the stifling is caused by the rumor's own success.

**Serial reproduction / rumor psychology.** Bartlett (1932) "War of the Ghosts": retold stories
get shorter, conventionalized, and assimilated to the reteller's culture. Allport & Postman
(1947) named three distortion operators: **leveling** (details dropped), **sharpening** (a few
details exaggerated), **assimilation** (details bent toward expectations, stereotypes, prejudice).
These map directly onto deterministic transformation operators on a belief record.

**Misinformation and reconsolidation.** Loftus & Palmer (1974): wording ("smashed" vs "hit")
changes later recall of speed and of broken glass that never existed. Nader et al. (2000):
reactivated memories become labile and must be re-stored, so *every retelling is an opportunity to
alter the teller's own memory*. Repetition increases perceived truth (illusory truth effect).

**BDI and source annotation.** In AgentSpeak/Jason, every belief literal carries annotations,
e.g. which source asserted it (self, percept, or a named agent). Communication uses performatives
(tell, untell, achieve, askOne...). The useful idea: provenance is *part of the belief*, not
metadata held elsewhere.

**Dynamic epistemic logic (DEL).** Knowledge as Kripke models; public announcements shrink the
set of possible worlds; private announcements use action models. SMCDEL shows it can be done
symbolically with BDDs. For a game it is far too general; what transfers is the vocabulary:
*public vs. private vs. semi-private announcements*, *common knowledge* (a public execution
creates common knowledge in the crowd; a whisper doesn't) and *higher-order knowledge* ("she knows
that I know").

**Cheap theory of mind.** Full nested belief is exponential. Practical ToM in games keeps
first-order beliefs fully and second-order beliefs only as *sparse tags* on first-order records:
"I told X", "X told me", "X was present when it happened", "X is likely to know (same household)".
From these, an agent can infer "X knows P" without storing X's whole mind. Bayesian ToM
(Baker, Saxe, Tenenbaum 2009/2017) is the principled version — inverse planning — only affordable
for T0 agents.

**Talk of the Town (James Ryan, 2015–2018).** Detailed below in B.13 because the source code was
inspected; it is the single best prior art for our belief system.

### A.3 Memory

**Multi-store view.** Sensory register (hundreds of ms, huge), working memory (≈4 chunks,
Cowan 2001), long-term memory split into episodic (events, when/where), semantic (facts,
schemas), procedural (skills), plus emotional modulation via the amygdala.

**ACT-R declarative memory (Anderson & Schooler 1991; Anderson & Lebiere 1998).** The most
game-friendly formal memory model because it is a handful of closed-form equations:

- Base-level activation of chunk i, with presentations at times t_j ago (j = 1..n), decay d
  (default 0.5):

      B_i = ln( Σ_{j=1..n} t_j^(−d) )

- Optimized-learning approximation (no need to store every timestamp; n = presentations,
  L = lifetime since creation):

      B_i ≈ ln( n / (1 − d) ) − d · ln(L)

  A hybrid keeps the k most recent timestamps exactly and approximates the rest; this is what we
  want for bounded storage.

- Total activation with spreading activation from context elements j (weights W_j, usually
  W/|context| with W = 1) and associative strengths S_ji:

      A_i = B_i + Σ_j W_j · S_ji + PM_i + ε,     S_ji = S − ln(fan_j)

  fan_j = number of chunks associated with cue j (a cue linked to many facts is a weak cue —
  this *is* interference). PM_i = partial-matching penalty Σ_k P·M_ki for slots that only
  approximately match the request; ε is logistic noise with scale s.

- Retrieval probability against threshold τ, and latency:

      P(retrieve i) = 1 / (1 + exp( −(A_i − τ) / s ))
      T_i = F · exp(−f · A_i)

- *Blending* returns a weighted average of slot values across matching chunks, weighted by
  softmax of activations with temperature ~ s√2 — this is a ready-made model of "averaged,
  schematized recollection".

**Generative Agents (Park et al. 2023) — depth from the source code.** Already known to the user,
but the code differs from the paper:
- The paper scores memories by recency + importance + relevance with all weights = 1, recency =
  exponential decay 0.995 per sandbox hour since last access, importance = LLM rating 1–10,
  relevance = embedding cosine; each term min-max normalized.
- In `retrieve.py` the recency term is *rank-based* (decay^i over the chronologically sorted
  candidate list), all three terms are min-max normalized to [0,1], combined with global weights
  **0.5 (recency), 3 (relevance), 2 (importance)** times per-persona weights; top 30 nodes are
  returned.
- In `reflect.py` a counter (`importance_trigger_curr`, initialized from
  `importance_trigger_max`, 150 in the paper) is decremented by the importance of each new event;
  at ≤ 0 the agent generates 3 focal questions, retrieves for each, writes 5 insights, each with
  a list of evidence node IDs, then resets the counter. Post-conversation reflections create
  "planning" and "memo" thoughts pointing at the chat node.
- Lesson: *evidence pointers* from reflections back to episodes are what make insights auditable.
  We keep that idea at all tiers (semantic facts know which episodes produced them).

**Forgetting curves.** Ebbinghaus exponential R = e^(−t/S); Wixted & Ebbesen showed power laws fit
better; FSRS (open spaced-repetition) uses a power curve fit on hundreds of millions of reviews:

    R(t, S) = (1 + factor · t / S)^(−w20),  factor = 0.9^(−1/w20) − 1   (FSRS-6; R(S,S) = 0.9)
    FSRS-4.5:  R = (1 + (19/81) · t/S)^(−0.5)

S (stability) grows on successful recall, more so when R was low (desirable difficulty: a
memory retrieved when nearly forgotten gets strengthened more), and drops on lapse. This
"DSR" state (difficulty, stability, retrievability) is **3 floats per memory** and is a superb
cheap memory-strength model for game agents.

**Consolidation / complementary learning systems (McClelland, McNaughton & O'Reilly 1995;
Kumaran, Hassabis & McClelland 2016).** Fast hippocampal storage of specific episodes, slow
neocortical extraction of regularities via interleaved replay (often during sleep). Schemas speed
up consolidation of schema-consistent information (Tse et al. 2007). In game terms: a nightly
(sleep tick) pass that compresses similar episodes into semantic facts and schemas, with
emotional and schema-relevant episodes replayed preferentially.

**Event segmentation theory (Zacks et al. 2007).** People chunk continuous experience into events
at points of prediction error. EM-LLM applies the same idea to LLM context (surprise-based
boundaries). For us: an episode is closed and stored when the agent's activity/location/
interaction partner changes or when an unexpected event occurs — not every tick.

**Emotional / flashbulb memory.** High arousal improves consolidation and vividness/confidence
(Brown & Kulik 1977) but not necessarily accuracy (Talarico & Rubin 2003: flashbulb memories decay
in consistency like ordinary ones while confidence stays high). Great gameplay property: the
witness who is *most confident* is not necessarily *most accurate*.

**Distortion.** Bartlett's schema-driven reconstruction; DRM false memories (semantic lures);
source-monitoring errors (Johnson et al. 1993) — remembering the content but misattributing who
said it, which is exactly Talk of the Town's "transference" and a natural rumor amplifier.

**Interference.** Proactive and retroactive interference: similar memories compete. In ACT-R it is
the fan effect; in a slot-based store it is merging/overwriting of similar episodes.

**Collective memory (Halbwachs 1925; Jan & Aleida Assmann).** *Communicative memory* (living
oral memory) spans ~3 generations (80–100 years); events survive longer only by becoming
*cultural memory* (written, ritualized, monumentalized, sung). Directly implementable as a
settlement-level lore store whose items need periodic retelling or an external carrier.

**Memory models usable as cheap formulas:** MINERVA 2 (Hintzman 1984/1988: every trace is a
feature vector; probe activation = similarity^3; echo = activation-weighted sum → produces
schema/prototype effects for free), Temporal Context Model (Howard & Kahana 2002: a drifting
context vector explains recency and contiguity — "remembering one thing reminds you of what
happened next").

---

## Part B — Candidate repositories and systems

Grouped: Language (B.1–B.12), Knowledge/Beliefs/Spread (B.13–B.20), Memory (B.21–B.33).

### B.1 EGG — Emergence of lanGuage in Games (FAIR)
- URL: https://github.com/facebookresearch/EGG
- License: MIT (one sub-module BSD-3-Clause)
- Language: Python (PyTorch)
- Activity/maintenance: ~320 stars; ~1.4k commits; **archived (read-only) Aug 2026**.
- Architecture: `egg/core` provides Sender/Receiver wrappers that turn any PyTorch module into a
  discrete-message agent: either REINFORCE (sampled symbols, policy gradient with baseline and
  entropy bonus) or Gumbel-Softmax relaxation (differentiable straight-through), for single
  symbols or variable-length RNN messages with EOS. A `Game` object wires sender → channel →
  receiver → loss; a `Trainer` runs it. `egg/zoo` has reference games (signaling, compositional
  attribute-value reconstruction, MNIST autoencoder, channel games); `egg/nest` does grid search.
  Metrics: topographic similarity (correlation of meaning distances vs message edit distances),
  positional/bag-of-symbols disentanglement.
- Scientific concept modeled: Lewis signaling / referential games; emergent compositionality.
- Computational cost: neural forward+backward per game episode; thousands to millions of games
  to converge. Totally unsuitable per tick.
- What we can learn: the *metrics* (topographic similarity) to measure whether our procedural
  languages are compositional; the finding that length pressure and population turnover are
  needed to get natural-like codes.
- What we could integrate: offline only — e.g. to precompute "animal alarm call" codes for
  species or test-bench how bias settings shape our cheap model. MIT, safe.
- What we should NOT integrate: any neural training inside the game loop.
- Scalability: 10 agents in research; 1k+ infeasible at runtime.

### B.2 Emergent Communication at Scale (DeepMind)
- URL: https://github.com/google-deepmind/emergent_communication_at_scale
- License: Apache-2.0 (code), CC-BY-4.0 (other materials)
- Language: Python (JAX/Haiku/Jaxline)
- Activity/maintenance: ~40 stars; research code for ICLR 2022; low activity.
- Architecture: populations of speakers and listeners trained on a Lewis referential game with
  random pairing each step; experiments with periodic listener resets and "ease of learning"
  (how fast a fresh listener learns an existing language).
- Scientific concept modeled: population effects, cultural transmission via resets.
- Computational cost: GPU-scale training.
- What we can learn: "ease of learning by newcomers" is a good *metric* for our languages — a
  language that children and migrants learn quickly is the one that survives.
- What we could integrate: ideas only.
- What we should NOT integrate: the code.
- Scalability: n/a for runtime.

### B.3 Babel toolkit / Fluid Construction Grammar (Steels lab, VUB AI Lab + Sony CSL Paris)
- URL: https://github.com/martinodb/babel-core (GitHub mirror; upstream is
  https://gitlab.ai.vub.ac.be/ehai/babel-core); older public release
  https://github.com/dwarfmaster/Babel2 (unverified details)
- License: Apache-2.0 (core packages, per the Babel site/mirror)
- Language: Common Lisp
- Activity/maintenance: mirror ~1 star, upstream GitLab active; the EHAI group also hosts a
  Python toolkit "HERMES" for multi-agent emergent-communication sims on the same GitLab
  (seen in search results, last update April 2026; not inspected — unverified).
- Architecture: experiment framework where a population runs repeated *interactions*; each
  interaction has a script (conceptualize → produce → parse → interpret → feedback → align).
  Lexicons are sets of form–meaning constructions with scores; alignment applies lateral
  inhibition. FCG represents grammar as bidirectional constructions over feature structures;
  IRL (Incremental Recruitment Language) composes meaning as small programs over the context.
- Scientific concept modeled: language games, grounded lexicon/grammar formation.
- Computational cost: lexical games are cheap (dictionary lookup + score update ~O(inventory));
  FCG parsing/production is search and expensive.
- What we can learn: the canonical *interaction script* and the score-update constants; the
  separation of "conceptualization" (what to talk about, how to categorize) from "lexical
  lookup".
- What we could integrate: the algorithms (naming game with lateral inhibition, multi-word naming,
  guessing game). Apache-2.0 would allow code use, but it's Lisp — reimplement.
- What we should NOT integrate: FCG grammar machinery (overkill; players won't read grammar).
- Scalability: lexical games trivially scale to 100k agents if run only on actual conversations.

### B.4 SimLang (Simon Kirby et al., Edinburgh CLE course)
- URL: https://github.com/smkirby/SimLang
- License: MIT
- Language: Python (Jupyter notebooks)
- Activity/maintenance: ~25 stars; course material, last substantial activity ~2019.
- Architecture: progressively richer models: innate signaling systems (send/receive matrices
  meaning×signal), learned signaling (matrix updated by observation), iterated learning chains,
  Bayesian learners (prior × likelihood with noise ε, sample vs MAP learners), compositionality
  emergence under bottleneck, evolution of learning biases.
- Scientific concept modeled: iterated learning, Bayesian cultural transmission.
- Computational cost: a learner = small matrix; learning from m utterances is O(m·|meanings|).
- What we can learn: exact, minimal formulations of "child learns from a sample of adult speech";
  the sampler vs. MAP-learner distinction as a culture parameter (conservative vs innovative).
- What we could integrate: algorithms; MIT also allows code reuse.
- What we should NOT integrate: notebook code as-is (pedagogical).
- Scalability: per-child learning event is cheap enough for 100k agents if done once per life
  stage, not per tick.

### B.5 Neural Iterated Learning (Ren, Guo, Labeau, Cohen, Kirby — ICLR 2020)
- URL: https://github.com/Joshua-Ren/Neural_Iterated_Learning
- License: MIT
- Language: Python (PyTorch)
- Activity/maintenance: ~16 stars; "fundamental version", inactive.
- Architecture: three phases per generation — *learning* (new agents imitate the previous
  generation on a limited sample), *interacting* (sender/receiver game play), *transmitting*
  (output becomes next generation's data). Topographic similarity rises to ~1.0 over generations.
- Scientific concept modeled: bottleneck → compositionality in neural agents.
- Computational cost: neural training per generation.
- What we can learn: confirms that the *generation turnover* in our world (children replacing
  adults) is itself the structural pressure — we don't need a separate mechanism.
- What we could integrate: idea only.
- What we should NOT integrate: code.
- Scalability: n/a.

### B.6 skILMpy — generalized Smith–Kirby ILM
- URL: https://github.com/dhard/skILMpy
- License: Artistic-2.0
- Language: Python
- Activity/maintenance: ~4 stars; small project, recently touched (uses `uv`).
- Architecture: Smith–Kirby iterated learning with configurable meaning spaces, signal spaces,
  bottleneck and population parameters (sparse documentation; details unverified).
- Scientific concept modeled: classic ILM.
- Computational cost: small.
- What we can learn: parameterization of meaning spaces as feature tuples.
- What we could integrate: ideas only (Artistic-2.0 is permissive-ish but we gain little).
- What we should NOT integrate: code.
- Scalability: n/a.

### B.7 Lingo — naming game, dialects on networks
- URL: https://github.com/taabishahmad/lingo
- License: MIT
- Language: Python (+ small web visualizer)
- Activity/maintenance: small personal project; stars unverified (low).
- Architecture: `agent.py` word inventories with reinforcement, `game.py` naming-game rules,
  `networks.py` fully connected / grid / small-world topologies, `metrics.py` distinct words,
  success rate, dialect clusters; experiments on population size, noise, network type.
- Scientific concept modeled: consensus vs. dialect persistence as function of topology and noise.
- Computational cost: O(inventory) per interaction.
- What we can learn: a ready-made test bench: on grid topologies + noise, dialects persist; on
  small-worlds they merge. Use to tune our trade-route weighting.
- What we could integrate: algorithm; MIT.
- What we should NOT integrate: nothing problematic, just too simple.
- Scalability: 100k fine (interaction-driven).

### B.8 Tutorial_Agent_Based_Models (Limor Raviv & Bill Thompson, MPI Nijmegen)
- URL: https://github.com/Limor-Raviv/Tutorial_Agent_Based_Models
- License: unverified (not shown on page)
- Language: Python 2.7 notebooks
- Activity/maintenance: ~186 stars; old (Python 2.7).
- Architecture: agents are lists of variant counts; interactions sample a variant; agents update
  with a bias; "stubborn" agents never update; question: does a sound-change variant fixate?
- Scientific concept modeled: sound-change propagation, conformity, stubbornness.
- Computational cost: trivial.
- What we can learn: personality-driven linguistic conservatism with one parameter.
- What we could integrate: idea.
- What we should NOT integrate: code (Py2, unclear license).
- Scalability: trivially scalable.

### B.9 mewo2 naming-language (Martin O'Leary)
- URL: https://github.com/mewo2/naming-language
- License: MIT (Copyright 2016 Martin O'Leary)
- Language: JavaScript
- Activity/maintenance: ~320 stars; finished/inactive since ~2016; powers @unchartedatlas.
- Architecture: a language = consonant set + vowel set (+ optional sibilants, liquids, finals)
  chosen from presets, a syllable template string (e.g. "CVC", "CVV?C" with optional slots),
  regex-like restrictions forbidding bad clusters (e.g. double consonants), an orthography map
  phoneme→spelling, a morpheme cache per concept (generic morphemes for "city", "river", "land"
  are generated once and reused so place names share parts), word = 1–3 morphemes joined, names
  = word + optional generic word with a joiner. Generation is rejection sampling.
- Scientific concept modeled: phonotactics and morphological reuse — enough for "these names sound
  like the same culture".
- Computational cost: microseconds per word.
- What we can learn: the minimal parameter set that yields recognizable languages; reusing
  morphemes is what makes names feel related.
- What we could integrate: algorithm and, since MIT, even code ideas (we would reimplement in our
  engine language anyway).
- What we should NOT integrate: it has no change over time — we add that ourselves.
- Scalability: languages are per culture (tens to hundreds), so irrelevant to agent count.

### B.10 Lexurgy — sound change applier
- URL: https://github.com/def-gthill/lexurgy
- License: **GPL-3.0**
- Language: Kotlin (ANTLR grammar)
- Activity/maintenance: ~67 stars, ~730 commits; web app at lexurgy.com; maintained.
- Architecture: ordered rule blocks of the form "A ⇒ B / environment", feature-matrix phonology
  (classes like [+voiced −nasal]), syllabification rules, deromanizer/romanizer stages,
  intermediate stages for reconstructing a historical chain.
- Scientific concept modeled: regular sound change (Neogrammarian), ordered rules, feeding/bleeding.
- Computational cost: per word per rule a pattern match; ~µs.
- What we can learn: rule format and the practical rule catalogue conlangers use (lenition,
  palatalization, vowel shifts, final devoicing, cluster simplification); feature-based classes.
- What we could integrate: **algorithm only — GPL-3.0 code must not enter a closed-source game.**
- What we should NOT integrate: code, its DSL parser.
- Scalability: per culture, not per agent; fine.

### B.11 PHOIBLE — phonological inventories database
- URL: https://github.com/phoible/dev
- License: code MIT; data CC-BY-4.0 (attribution required)
- Language: Python/R processing scripts, CSV data
- Activity/maintenance: ~150 stars; PHOIBLE 2.0 (2019) with ongoing maintenance.
- Architecture: >3000 inventories for >2100 languages; one row per language–phoneme, with
  distinctive features per segment.
- Scientific concept modeled: typological frequency of phonemes (e.g. /m/, /k/, /i/, /a/ near
  universal; clicks rare) and feature geometry.
- Computational cost: offline data.
- What we can learn: realistic inventory sampling (sample segments weighted by cross-linguistic
  frequency, conditioned on implicational tendencies); feature vectors for computing phoneme
  distance (loanword adaptation, mutual intelligibility).
- What we could integrate: a derived frequency/feature table (CC-BY: credit in game credits).
- What we should NOT integrate: real-language identities (avoid making a culture "sound like"
  a real ethnic group in a stereotyped way).
- Scalability: n/a.

### B.12 bangyen/emergent — population referential games
- URL: https://github.com/bangyen/emergent
- License: unverified
- Language: Python (PyTorch)
- Activity/maintenance: unverified (seen in search results only).
- Architecture: Gumbel-Softmax referential games with populations and iterated learning.
  The README reports a striking finding: speakers in populations rarely converge on the same
  message (~3% agreement) yet all listeners understand all speakers — listeners become
  *multilingual* instead of the community converging.
- Scientific concept modeled: population heterogeneity, receptive multilingualism.
- Computational cost: neural.
- What we can learn: model *comprehension* separately from *production*. An agent can understand
  several dialect variants (passive lexicon) while producing one. This makes contact zones
  realistic and cheap: store a small "understood variants" set.
- What we could integrate: idea only.
- What we should NOT integrate: code.
- Scalability: n/a.

### B.13 Talk of the Town (James Ryan) — knowledge model from source
- URL: https://github.com/james-owen-ryan/talktown (engine, MIT, ~105 stars) and
  https://github.com/james-owen-ryan/talktown-testbed (knowledge phenomena, MIT, ~10 stars,
  ~765 commits; files `belief.py`, `evidence.py`, `config.py`, `conversation.py`, `mind.py`)
- License: MIT
- Language: Python 2
- Activity/maintenance: inactive research code (2015–2018); stable reference.
- Architecture (verified from `belief.py`, `evidence.py`, `config.py`):
  - Each character owns **mental models** keyed by entity: `PersonMentalModel`,
    `BusinessMentalModel`, `DwellingPlaceModel`. A person model groups beliefs into
    `StatusBelief`, `AgeBelief`, `NameBelief`, `WorkBelief`, `FaceBelief`, `WhereaboutsBelief`.
  - Each attribute (first/middle/last name, birth year, status alive/dead/departed, job title,
    workplace, shift, home address, skin/hair color, facial features, distinctive marks...) holds a
    **belief trajectory**: a list of `Facet` objects over time. A facet = (value, strength,
    accurate?, set of supporting evidence). New facets can be "challengers" to the current one;
    adoption replaces the predecessor but keeps history.
  - **Evidence types**: reflection, observation, examination, statement, declaration, lie,
    eavesdropping, confabulation, mutation, transference, forgetting, implant. Base strengths in
    config: reflection 9999, observation 20, examination 15, statement 5, lie 5, eavesdropping 5,
    confabulation 3, mutation 3, transference 3, declaration 2, forgetting 0.001.
  - **Told evidence strength** = base(type) × hearer's *trust* in the teller (0.5 for strangers
    overheard by eavesdropping) × teller-conviction boost × feature salience. The teller boost is
    clamp(sqrt(max(0.1, teller_strength)) / 15, 0.25, 1.5): a teller who strongly believes
    something sounds convincing. **Lies** get a random conviction boost instead (1–300 range
    before the transform), i.e. liars can sound very sure.
  - **Feature salience** multipliers: status ×1.5, age/hair/skin ×1.0, eyebrow color ×0.05;
    salience also decides what gets talked about (status nearly always, middle names almost never).
  - **Deterioration** each timestep per facet: chance = base_chance(feature) / owner.memory /
    facet.strength; base_chance per day ranges from 0.0025 (status) up to 0.65 (business address).
    When deterioration fires, it is **forgetting** with p = 0.7, **mutation** p = 0.2
    (plausible edit, e.g. each digit of a house number has 30% chance of ±1), **transference**
    p = 0.1 (copy the value from another entity of the same type — confusing two people).
  - **Confabulation**: if a facet is empty/unknown, 0.02/day chance of inventing a plausible value
    sampled from the population distribution.
  - Character `memory` trait ~ N(1.0, 0.05), clamped [0.5, 1.0], starts at 0.8 in childhood.
  - Conversations cover 2–7 subjects chosen by salience; each exchanged facet becomes evidence
    in the hearer's model, so misinformation propagates exactly like true information.
- Scientific concept modeled: source-dependent belief strength, misremembering (forgetting,
  mutation, transference/source confusion, confabulation), lying, eavesdropping.
- Computational cost: per facet per day a random check — O(facets); with ~20 attributes × a
  few hundred known people per agent it was slow in Python for ~300–500 characters.
- What we can learn: almost everything in our belief design: (1) beliefs as *trajectories with
  evidence*, (2) typed evidence with base strengths, (3) the three pollution operators with fixed
  split, (4) conviction-weighted telling, (5) salience-driven topic choice.
- What we could integrate: the mechanism and constants as starting points (MIT, so even direct
  ports are legally fine).
- What we should NOT integrate: the per-attribute-per-person-per-day stochastic loop for
  everyone (too costly at 100k); the town-only "everyone knows everyone" scale; Python 2 code.
- Scalability: 10 ✓; 1k ✓ if deterioration is event-driven (sample next-deterioration time from
  an exponential instead of per-day rolls); 100k only with compressed beliefs (see design).

### B.14 Jason — AgentSpeak BDI interpreter
- URL: https://github.com/jason-lang/jason
- License: **LGPL-3.0**
- Language: Java
- Activity/maintenance: ~260 stars, ~1.7k commits, v3.2+, maintained.
- Architecture: belief base of logical literals with annotations (e.g. which agent or percept was
  the source); plan library of triggering-event : context ← body rules; reasoning cycle per agent
  (perceive → update beliefs → select event → select applicable plan → execute one step);
  communication via KQML-like performatives (tell adds a belief annotated with the sender,
  untell removes it, askOne queries).
- Scientific concept modeled: BDI practical reasoning, belief provenance.
- Computational cost: logic unification per cycle; fine for hundreds of agents, heavy for 100k.
- What we can learn: **provenance as an annotation set on each belief** and "tell/untell" as the
  base communicative acts; belief revision triggered by percepts.
- What we could integrate: concepts. LGPL code would be linkable only dynamically and is Java;
  reimplement instead.
- What we should NOT integrate: the Prolog-style interpreter.
- Scalability: 10 ✓, 1k ✓ (slow), 100k ✗.

### B.15 SMCDEL — symbolic model checker for dynamic epistemic logic
- URL: https://github.com/jrclogic/SMCDEL
- License: **GPL-2.0**
- Language: Haskell
- Activity/maintenance: ~55 stars, ~250 commits; maintained academically.
- Architecture: knowledge structures (vocabulary, BDD state law, per-agent observable variables)
  replacing explicit Kripke models; public/private announcements as structure transformers;
  examples (muddy children, dining cryptographers, Russian cards).
- Scientific concept modeled: higher-order knowledge, common knowledge, announcements.
- Computational cost: exponential in the worst case; BDDs help for puzzles, not for 1k agents.
- What we can learn: *observability-based knowledge*: an agent knows what follows from the
  variables it can observe. Our perception system is the same idea: what you saw is determined
  by what you could observe (line of sight, presence).
- What we could integrate: nothing as code (GPL, Haskell). The announcement taxonomy (public,
  private, semi-private/overheard) is worth adopting.
- What we should NOT integrate: full DEL reasoning at runtime.
- Scalability: 10 (puzzle scale) only.

### B.16 NDlib — network diffusion library
- URL: https://github.com/GiulioRossetti/ndlib
- License: BSD-2-Clause
- Language: Python
- Activity/maintenance: ~300 stars; maintained (dashboard, model builder).
- Architecture: node-status maps over a NetworkX/iGraph network; models as per-iteration status
  transitions: SI/SIS/SIR/SEIR, threshold and independent cascades, opinion dynamics (Voter,
  Sznajd, majority rule, Deffuant and Hegselmann–Krause bounded confidence); a compositional model
  builder with status nodes and stochastic/threshold/conditional compartments.
- Scientific concept modeled: diffusion and opinion dynamics.
- Computational cost: O(edges) per iteration.
- What we can learn: a library of *aggregate* update rules usable for T2/T3 tiers: e.g. bounded
  confidence for how settlements' opinions of a ruler converge, independent cascades for rumor
  jumps along trade routes.
- What we could integrate: algorithms (BSD-2 also permits code).
- What we should NOT integrate: Python runtime in the engine.
- Scalability: 100k nodes OK offline; in-game we'd run settlement-graph versions (~1k nodes).

### B.17 EoN — Epidemics on Networks (Kiss, Miller, Simon)
- URL: https://github.com/springer-math/Mathematics-of-Epidemics-on-Networks
- License: MIT
- Language: Python
- Activity/maintenance: ~165 stars; v2.0 released Sept 2026 (second edition of the book).
- Architecture: event-driven (Gillespie) SIR/SIS simulation with priority queues of next-event
  times; generic "simple contagion" and "complex contagion" Gillespie engines where transition
  rates depend on node and neighbor states; pair-approximation and edge-based compartmental ODEs.
- Scientific concept modeled: continuous-time stochastic spreading on networks.
- Computational cost: O(log n) per event rather than per tick — key for scaling.
- What we can learn: **event-driven scheduling**: instead of rolling dice every tick for every
  belief, schedule the next telling/forgetting time from an exponential distribution and keep a
  priority queue. Also the edge-based ODEs to run T3 settlement-level rumor dynamics analytically.
- What we could integrate: algorithms; MIT permits code.
- What we should NOT integrate: none problematic.
- Scalability: 1M-node SIR runs are feasible in its event-driven mode.

### B.18 Ensemble engine (successor of Comme il Faut / Prom Week)
- URL: https://github.com/ensemble-engine/ensemble
- License: BSD-4-Clause (has an advertising clause — check with counsel)
- Language: JavaScript
- Activity/maintenance: ~64 stars; low activity.
- Architecture: social state = typed predicates (traits, directed relationships, network values
  0–100, statuses with durations, and a *social history* of past predicates by timestep); trigger
  rules update state; volition rules compute desires to perform actions toward others; actions
  have influence rules and effects.
- Scientific concept modeled: social physics; "social facts database" with history.
- Computational cost: rule evaluation over all character pairs — O(n²·rules).
- What we can learn: social history as queryable time-indexed facts ("did X insult Y within the
  last 5 turns?"); the knowledge store doubles as a rule-condition store.
- What we could integrate: ideas only.
- What we should NOT integrate: O(n²) rule evaluation for large populations; the code (BSD-4).
- Scalability: 10–50 characters.

### B.19 Graphiti (Zep) — bi-temporal knowledge graph
- URL: https://github.com/getzep/graphiti
- License: Apache-2.0
- Language: Python
- Activity/maintenance: ~31k stars; very active.
- Architecture: raw *episodes* are ground truth; LLM extraction creates entity nodes and fact
  edges; each edge is **bi-temporal**: when the fact was true in the world (valid_at/invalid_at)
  and when the system learned/recorded it (transaction time). Contradicting facts *invalidate*
  old edges rather than deleting them; every derived fact links back to source episodes; hybrid
  retrieval (embedding + BM25 + graph distance).
- Scientific concept modeled: temporal knowledge with provenance.
- Computational cost: LLM call(s) per episode ingestion + graph DB writes.
- What we can learn: **two time stamps per belief** — "when I think it happened" vs "when I came
  to believe it" — is exactly what a rumor system needs ("I heard yesterday that Tomas died last
  week"). Invalidate-don't-delete gives us belief trajectories (same as Talk of the Town).
- What we could integrate: the schema idea for all tiers; the library itself only for an
  out-of-engine T0 narrator memory (Apache-2.0 is fine).
- What we should NOT integrate: LLM extraction for simulation facts (the sim already produces
  structured events — no need to parse text).
- Scalability: library: 10 T0 agents fine; the schema idea: all tiers.

### B.20 Classical rumor models (no single repo) — Daley–Kendall / Maki–Thompson
- URL: n/a (papers: Daley & Kendall 1965; Maki & Thompson 1973; Sudbury 1985; survey-level
  results on networks, e.g. Moreno, Nekovee & Pacheco 2004)
- License: n/a
- Language: n/a
- Activity/maintenance: classical.
- Architecture: see A.2; network versions replace mixing by neighbor contact; refinements add
  forgetting rate δ (spreaders spontaneously stop), k-stifling (stop after k failed attempts) and
  multiple repetitions.
- Scientific concept modeled: self-limiting information spread.
- Computational cost: O(1) per contact.
- What we can learn: the *stifling* rule ("stop telling after hearing 'I know' k times") is the
  core behavioral rule for agents' tell-drive; θ ≈ 0.2 gives a principled default for how much
  of a population never hears a mundane piece of news.
- What we could integrate: all of it as algorithm.
- What we should NOT integrate: homogeneous mixing assumption.
- Scalability: any.

### B.21 Mem0
- URL: https://github.com/mem0ai/mem0
- License: Apache-2.0
- Language: Python (also TS SDK)
- Activity/maintenance: ~66k stars; very active.
- Architecture: memories scoped to user/session/agent; LLM extracts candidate facts from
  conversation; earlier versions compared each candidate to similar stored memories and chose
  ADD / UPDATE / DELETE / NOOP; the current README describes a single-pass *add-only*
  extraction with entity linking and multi-signal retrieval (semantic + BM25 + entity match)
  plus temporal reasoning at retrieval time. Optional graph memory.
- Scientific concept modeled: none explicitly; engineering of fact extraction.
- Computational cost: ≥1 LLM call per write; vector search per read.
- What we can learn: the ADD/UPDATE/DELETE/NOOP decision is a nice explicit *belief-revision
  vocabulary*; we can make the same decision deterministically (same subject+predicate → update
  confidence or open a challenger).
- What we could integrate: for T0 LLM agents only, as the store of *conversational* facts the
  player told them (e.g. prayers to the god). Apache-2.0 fine.
- What we should NOT integrate: as the general memory for simulated people.
- Scalability: 10 ✓; 1k costly; 100k ✗.

### B.22 A-MEM (agentic memory, Zettelkasten-style)
- URL: https://github.com/agiresearch/A-mem (system, MIT, ~1.2k stars) and
  https://github.com/WujiangXu/A-mem (paper reproduction for NeurIPS 2025, MIT, ~1k stars)
- License: MIT
- Language: Python
- Activity/maintenance: moderate; paper repo points to "A-mem-sys" for production use.
- Architecture: each memory *note* = content + timestamp + LLM-generated keywords, tags,
  contextual description + embedding + links. On insert: find nearest notes (ChromaDB), ask the
  LLM which to link, and **evolve** neighbors (rewrite their context/tags in light of the new
  note).
- Scientific concept modeled: associative networks; memory reconsolidation (old notes change when
  new related ones arrive).
- Computational cost: several LLM calls per insertion.
- What we can learn: "memory evolution" = reconsolidation implemented as rewriting neighbors.
  Deterministic analogue: when a new episode links to an old one, update the old one's summary
  tags (e.g. "the day I met Bera" becomes "the day I met my future murderer").
- What we could integrate: idea; code only for T0.
- What we should NOT integrate: LLM-maintained links for the simulation.
- Scalability: 10 ✓, otherwise ✗.

### B.23 MemoryOS (BAI-LAB, EMNLP 2025 oral)
- URL: https://github.com/BAI-LAB/MemoryOS
- License: Apache-2.0
- Language: Python
- Activity/maintenance: ~1.6k stars; updated Jan 2026.
- Architecture: OS-like hierarchy: short-term memory (default capacity 7 dialogue pages, FIFO),
  mid-term memory organized into *segments* of topic-related pages, long-term personal memory
  (user persona + agent persona, knowledge entries, capacity ~100). Mid-term segments carry a
  **heat** score; per the paper (not verified in source): Heat = α·N_visit + β·L_interaction +
  γ·R_recency with R_recency = exp(−Δt/μ); hot segments are promoted to long-term persona
  updates; cold ones are evicted.
- Scientific concept modeled: multi-store memory with consolidation by usage.
- Computational cost: LLM summarization on promotion; vector search on reads.
- What we can learn: the *heat* formula is a cheap, deterministic consolidation trigger we can
  copy for all tiers (visits = times recalled/retold; length = involvement; recency).
- What we could integrate: heat-based promotion as algorithm; library for T0 only.
- What we should NOT integrate: the LLM summarization path for T1+.
- Scalability: idea: all tiers; library: 10.

### B.24 MemoryBank / SiliconFriend
- URL: https://github.com/zhongwanjun/MemoryBank-SiliconFriend
- License: MIT
- Language: Python
- Activity/maintenance: ~450 stars; research code (AAAI 2024), inactive.
- Architecture: stores conversations + daily summaries + a user personality profile; each memory
  has strength S (starts at 1) and elapsed time t; retention R = e^(−t/S); each recall increments
  S and resets t; memories below a retention threshold are dropped.
- Scientific concept modeled: Ebbinghaus forgetting with spacing effect.
- Computational cost: O(1) per memory per check.
- What we can learn: the simplest possible "used memories persist" rule. FSRS (B.31) is a better
  calibrated version of the same idea.
- What we could integrate: formula.
- What we should NOT integrate: code.
- Scalability: formula scales to anything.

### B.25 cognee
- URL: https://github.com/topoteretes/cognee
- License: Apache-2.0
- Language: Python (SDKs in TS/Rust)
- Activity/maintenance: ~31k stars (per page); very active.
- Architecture: ingestion pipeline turning documents/conversations into a knowledge graph + vector
  index ("cognify"), then enrichment ("memify"/improve), recall with multiple search strategies,
  explicit *forget* of datasets; optional ontology grounding.
- Scientific concept modeled: semantic memory as KG.
- Computational cost: LLM-heavy ingestion.
- What we can learn: explicit **forget** API as a first-class operation; ontology-grounded KG.
- What we could integrate: possibly as the *world-lore archive* for the god-player UI (searching
  history), not inside agents.
- What we should NOT integrate: as agent memory.
- Scalability: 10 agents / 1 world archive.

### B.26 MemOS (MemTensor)
- URL: https://github.com/MemTensor/MemOS
- License: Apache-2.0
- Language: Python
- Activity/maintenance: ~11.7k stars; very active.
- Architecture: "MemCube" units that wrap plaintext memories, activation memory (KV caches) and
  parametric memory (adapters) with metadata (provenance, versioning, access policy);
  MemScheduler moves items between types asynchronously; graph-structured inspectable memory.
- Scientific concept modeled: memory types by substrate; scheduling between them.
- Computational cost: LLM infra-level.
- What we can learn: memory items with *access policy* — maps to secrets (who may be told).
- What we could integrate: nothing for the sim; maybe KV-cache reuse ideas for T0 narration cost.
- What we should NOT integrate: everything else.
- Scalability: T0 only.

### B.27 LangMem (LangChain)
- URL: https://github.com/langchain-ai/langmem
- License: MIT
- Language: Python
- Activity/maintenance: ~1.7k stars; active.
- Architecture: memory tools the agent calls in the "hot path" (manage/search), plus a background
  memory manager that extracts and consolidates; distinguishes semantic (facts/profile),
  episodic (past experiences as few-shot examples) and procedural (prompt rules updated by
  optimization) memory.
- Scientific concept modeled: CoALA-style memory taxonomy.
- Computational cost: LLM calls.
- What we can learn: *procedural memory as rewritten instructions* — for T0 LLM agents, their
  learned habits can be written into their system prompt from the deterministic skill/habit state.
- What we could integrate: T0 only.
- What we should NOT integrate: for simulation.
- Scalability: T0 only.

### B.28 Zep (community edition)
- URL: https://github.com/getzep/zep
- License: Apache-2.0
- Language: Python / TS / Go SDKs
- Activity/maintenance: ~4.9k stars; **Community Edition deprecated, moved to `legacy/`**; repo
  now holds examples/integrations for the hosted Zep Cloud.
- Architecture: hosted service built on Graphiti.
- What we can learn / integrate: use Graphiti directly instead; avoid cloud dependency.
- What we should NOT integrate: Zep Cloud (vendor lock, online dependency in a game).
- Scalability: n/a.
- Scientific concept modeled / Computational cost: see Graphiti.

### B.29 AI Town (a16z-infra)
- URL: https://github.com/a16z-infra/ai-town
- License: MIT
- Language: TypeScript (Convex backend, PixiJS frontend)
- Activity/maintenance: ~10.6k stars; moderate activity.
- Architecture: a tick-based game engine running inside a transactional backend; agents have
  memories with embeddings; after conversations, a summary memory with importance is written;
  retrieval takes top-k by a relevance/recency/importance mix (NUM_MEMORIES_TO_SEARCH); engine
  pauses when idle; agent "operations" (LLM calls) are scheduled asynchronously so the
  deterministic engine doesn't block on the LLM.
- Scientific concept modeled: generative-agents memory in production form.
- Computational cost: 1+ LLM call per conversation turn + summary.
- What we can learn: **async LLM operations decoupled from the engine tick** with results applied
  as input events — exactly how our T0 LLM layer should feed back into a deterministic sim
  (LLM outputs become proposals validated by the sim).
- What we could integrate: architecture pattern; MIT.
- What we should NOT integrate: the memory store for non-T0 agents.
- Scalability: ~25 agents realistically.

### B.30 EM-LLM — human-like episodic memory for LLMs (ICLR 2025)
- URL: https://github.com/em-llm/EM-LLM-model
- License: MIT
- Language: Python
- Activity/maintenance: research code; moderate.
- Architecture: segments the token stream into events at points of high *surprise* (negative
  log-likelihood above a moving threshold), refines boundaries with graph modularity, stores
  events, and retrieves in two stages: similarity-based plus **temporal contiguity** (neighbors of
  retrieved events are also pulled in).
- Scientific concept modeled: event segmentation theory; contiguity effect (TCM).
- Computational cost: LLM-internal.
- What we can learn: (1) surprise-triggered episode boundaries — cheap analogue: prediction error
  of the agent's routine (expected location/activity vs actual); (2) contiguity: recalling an
  episode also cues the next one, which makes narrated memories flow as stories.
- What we could integrate: both ideas, deterministically.
- What we should NOT integrate: the model code.
- Scalability: ideas: all tiers.

### B.31 FSRS (Free Spaced Repetition Scheduler) — fsrs-rs and bindings
- URL: https://github.com/open-spaced-repetition/fsrs-rs (algorithm wiki:
  https://github.com/open-spaced-repetition/awesome-fsrs/wiki/The-Algorithm)
- License: BSD-3-Clause (fsrs-rs)
- Language: Rust (bindings for Python, Node, C, Dart, PHP)
- Activity/maintenance: ~440 stars; active; FSRS-6 is current.
- Architecture: per item state (Difficulty D ∈ [1,10], Stability S in days, last review time);
  retrievability R(t,S) power curve (see A.3); on success S grows by a factor that is larger for
  low D, low current S and low R; on lapse S is recomputed smaller; parameters (≈21 weights)
  are fitted to data.
- Scientific concept modeled: spacing effect, desirable difficulty, power-law forgetting.
- Computational cost: a few flops per recall event; nothing per tick (R is computed lazily from
  elapsed time).
- What we can learn: **lazy decay**: never tick memories; compute R when queried. This is the
  single most important scaling trick for memory at 100k agents.
- What we could integrate: the formulas (and BSD-3 permits code). Use published default weights
  as a starting point, then re-tune for "game time".
- What we should NOT integrate: the optimizer.
- Scalability: 1M agents × dozens of memories is fine because cost is per access.

### B.32 Dwarf Fortress memory & thoughts (game reference, no code)
- URL: not open source (Bay 12 Games); described on the DF wiki "Memory (thought)" pages and in
  press coverage (Game Developer, PCGamesN).
- License: proprietary — reference only.
- Language: C++ (closed).
- Activity/maintenance: active commercial game.
- Architecture: each dwarf has **8 short-term and 8 long-term memory slots**. The 8 most
  emotionally intense recent events are relived periodically (stress changes); after a year in
  short-term, a memory moves to long-term only if stronger than the weakest long-term memory,
  otherwise it is discarded. Long-term memories persist, keep resurfacing, and can **permanently
  shift personality facets/values** as the associated emotion changes over time.
- Scientific concept modeled: emotional salience, rumination, trauma-driven personality change.
- Computational cost: O(16) per agent, periodic.
- What we can learn: **fixed tiny slot budgets ranked by emotional intensity are enough to produce
  great stories**; memory → personality feedback.
- What we could integrate: design pattern (budget numbers, promotion rule).
- What we should NOT integrate: n/a (no code).
- Scalability: proven at a few hundred dwarves; trivially scales.

### B.33 Generative-agents ecosystem notes (depth on known systems)
- URL: https://github.com/joonspk-research/generative_agents (Apache-2.0, Python) — already known.
- License/Language/Activity: see repo; inactive since 2023, many forks.
- Architecture: covered in A.3 from `retrieve.py`/`reflect.py` (weights 0.5/3/2, top-30,
  importance trigger, 3 focal points → 5 insights with evidence pointers).
- Scientific concept modeled: recency/importance/relevance retrieval; reflection = abstraction.
- Computational cost: dozens of LLM calls per agent per game hour in the original.
- What we can learn: keep the *scoring structure* but replace LLM importance with a deterministic
  importance = f(emotion intensity, goal relevance, novelty, social closeness of participants), and
  replace embeddings with structured cue overlap (same entities/place/action type).
- What we could integrate: retrieval formula and evidence pointers.
- What we should NOT integrate: LLM-per-observation importance rating; unbounded memory stream.
- Scalability: deterministic version: 100k ✓; LLM version: ~25.

---

## Part C — Recommended design for our game

Design principles:
1. **Structured, not textual.** All simulation knowledge is typed records referring to entity and
   event IDs. Text only appears when an LLM narrates a T0 agent or a UI renders a record.
2. **Lazy time.** Nothing decays per tick. Every record stores timestamps; strength/retrievability
   is computed at query time; stochastic events (forgetting, mutation) are *scheduled* (sampled
   next-event times in a priority queue) or resolved on access.
3. **Budgets everywhere.** Every agent has fixed-capacity stores; insertion into a full store
   triggers consolidation or eviction by a single scalar score.
4. **Deterministic RNG.** All stochastic draws use counter-based RNG keyed by (world seed, agent id,
   record id, event counter) so replays and LOD promotion/demotion are reproducible.

### C.(a) Per-agent belief store with provenance, confidence and decay

#### Record layout (~40–48 bytes per belief at T1)

    BeliefRecord
      subject      : EntityId | EventId        (who/what it is about)
      predicate    : u16 enum                  (is_dead, killed_by, location_of, married_to,
                                                 owes_debt, is_witch, harvest_failed, price_of...)
      value        : u32 (entity/enum/number)  (e.g. killer = Bera; weapon = axe)
      t_world      : game time the agent thinks it happened (with ± uncertainty in days)
      t_learned    : game time the agent acquired it              (Graphiti bi-temporal idea)
      logodds      : f16 confidence (log-odds; 0 = 50/50)
      stability    : f16 (FSRS-like S, in days)                    → retrievability lazily
      last_access  : game time of last recall/retelling
      source       : compact provenance:
                       kind  : {witness, examined, told, overheard, read, inferred, confabulated, own_lie}
                       from  : AgentId or DocumentId (immediate source)
                       hops  : u8 (distance from first-hand witness, saturates at 15)
                       origin: EventId (ground-truth event this chain started from, hidden from agent)
      emotion      : i8 valence, u8 arousal                         (tag for memory & telling)
      flags        : secret, told_to_count(4 bits), known_by_tags, challenger, sacred/taboo

`origin` is the debug/ground-truth link the *agent never sees*; it lets the god-player UI show how
a rumor mutated along its path, and lets LOD aggregation group variants of one story.

Competing values for the same (subject, predicate) are kept as a small set (max 3) of challenger
records — the Talk of the Town trajectory idea. The *believed* value is the one with highest
effective confidence; the others remain as "some say..." which is perfect for narration.

#### Evidence strength and update

Each incoming piece of evidence e with type k carries weight

    w(e) = base_k × trust(source) × conviction(source) × salience(predicate) × perception_quality

- base_k (start from Talk of the Town ratios, rescaled to log-odds units):
  witness 3.0, examined 2.3, read (document) 1.2, told 0.8, overheard 0.5, inferred 0.6,
  confabulated 0.3.
- trust(source) ∈ [0,1.2]: from relationship (kin, friend, rival), the source's reputation for
  honesty, in-group/out-group, and *language intelligibility* (see C.(c)).
- conviction(source) = clamp(sqrt(max(0.1, conf_src))/k_c, 0.25, 1.5) where conf_src is the
  teller's confidence mapped to a positive scale (ToT's formula); liars pick their conviction.
- perception_quality for witnessing: lighting × distance falloff × attention (was the agent
  busy?) × stress penalty (very high arousal narrows attention to the "weapon focus").

Belief update (log-odds, Bayesian-flavoured, bounded):

    logodds_new = clamp( logodds_old + w(e) − λ·max(0, w_challenger_existing), −L, +L )

Contradicting evidence for another value adds w to that challenger instead; if the challenger
exceeds the current belief by margin m, they swap (belief revision event — can trigger emotions:
"I was lied to!" when source of the old belief was a friend).

Retrievability R(t) = (1 + factor·Δt/S)^(−c) with Δt = now − last_access (FSRS form). Effective
confidence at query = sigmoid(logodds) × g(R), where g(R) blends toward "unsure" (0.5) as R drops.
Below R_forget the record is eligible for eviction (or degradation, below).

#### The operations (how information travels)

| Operation | Trigger | Effect on receiver | Effect on source |
|---|---|---|---|
| **witness** | perception system: agent in line-of-sight/earshot when event emitted | new record, kind=witness, hops=0, high arousal tag if violent | — |
| **examine** | agent inspects body/scene/object | adds facets (cause of death, weapon) at examined weight | — |
| **tell** | conversation topic selection | record kind=told, from=teller, hops=teller.hops+1, distortion operators applied | teller's own record is *reactivated* (S grows; reconsolidation chance); told_to_count++ |
| **overhear** | agent within earshot of a tell, not addressed | as tell, weight ×0.6, trust=0.5 default | — |
| **teach** | parent/mentor → child/apprentice, repeated sessions | semantic facts and skills; high trust; repeated ⇒ high S; not news | — |
| **write** | literate agent creates Document entity (letter, ledger, chronicle, carving) | document stores frozen copy of record (value, claimed source, date) | — |
| **read** | literate agent reads document | kind=read, from=DocumentId, hops = doc.hops+1, weight depends on document authority | document unchanged (durable!) |
| **lie** | agent with motive (self-protection, malice, loyalty) chooses to tell a value it doesn't believe | receiver cannot tell a lie from a tell; record flagged own_lie only in the liar | liar stores own_lie record to stay consistent (liars must remember their lies) |
| **misremember** | scheduled deterioration (rate ∝ 1/(memory_trait × strength)) | forget 70% / mutate 20% / transfer 10% (ToT split) | — |
| **exaggerate** | on tell, if teller's arousal or "dramatic" trait high | sharpen numeric/severity facets (count of attackers +1, "wounded" → "killed") | — |
| **forget** | eviction from full store or R below threshold | record removed or reduced to gist ("something happened at the mill") | — |
| **infer** | cheap rules at consolidation time | new record kind=inferred, e.g. "Tomas absent 3 days & blood at mill ⇒ Tomas likely dead" | — |

**Topic choice in conversation (who says what).** For each candidate record r, the speaker scores

    tell_score(r, listener) = salience(r) × novelty(r) × relevance(r, listener)
                              × R(r) × (1 − secrecy(r, listener)) × drive(r)

- novelty decays with age since t_learned (news is perishable);
- relevance high if the listener is related to the subject or lives near the place;
- secrecy uses the "known_by_tags" and relationship (don't tell the killer's brother);
- drive(r) is the Maki–Thompson *stifler* rule: starts at 1, multiplied by 0.5 each time a
  listener replies "I already know" (k-stifling), so mundane news self-limits and ~20% of a
  well-mixed group never hears it, while dramatic news (high salience) keeps circulating longer.
Top 2–5 topics per conversation (ToT used 2–7).

**Distortion operators applied on transmission** (deterministic given RNG key; probability
scaled by hops and teller traits):
- *leveling*: each low-salience facet (time of day, exact place, weapon type) is dropped with
  p = 0.15 per hop (scaled by 1/teller.memory);
- *sharpening*: severity/number facets get +1 step with p = 0.1 × drama_trait;
- *assimilation*: if the culprit facet is weak (low confidence) the teller substitutes the value
  best matching their prejudices/schema: argmax over plausible candidates of (prior suspicion ×
  disliked group) — this is how outsiders get blamed;
- *transference*: value swapped with a similar entity (same profession, similar name) at the
  ToT 10% share of deterioration events;
- *translation loss*: if teller and listener speak different dialects, facets whose words are not
  mutually intelligible are dropped or garbled (names are re-phonologized — "Bera" → "Pera").

**Cheap theory of mind.** No nested belief stores. Each record holds `told_to_count`, and a
small per-agent ring of (record, person) "I told X" / "X told me" pairs (16 entries at T1). The
agent infers `knows(X, r)` = X told me ∨ I told X ∨ X was co-witness ∨ X in same household and
r older than 1 day. Second-order ("does X know that I know?") is only computed for T0 agents
and only for records flagged secret.

#### Worked example — a murder in Village A reaches Village B

Setup: Village A (river mill, pop. 140) and Village B (market town, pop. 600) are 2 days apart
by road; a trader passes weekly. A and B speak dialects of the same language with
intelligibility 0.7 (A says *"bera"* /bera/, B pronounces it /pera/; A's word for "axe"
*tarum* has no cognate in B, which uses *hesk*).

Ground truth (event E#9001): Day 0, dusk, at the mill, **Bera** (miller's apprentice) kills
**Tomas** (miller) with an **axe** after a quarrel over debt.

Day 0, dusk — witnessing:
- **Ilsa** (Tomas's daughter, 40 m away, dusk light 0.4) witnesses: records `killed_by(Tomas)=Bera`
  with perception_quality 0.4×0.8 → w ≈ 3.0×1×1×1.5×0.32 ≈ 1.4 (log-odds ≈ 1.4, conf ≈ 0.8);
  `weapon=axe` lower (she saw "something heavy", value=heavy_tool, conf 0.6). Arousal 0.95 →
  flashbulb-tagged (pinned episodic slot; see C.(b)). Confidence will stay high regardless of
  accuracy.
- **Bera** has a witness record of his own act plus a *motive to lie*.

Day 1 — examination and telling inside A:
- Villagers find the body. **Old Maren** (healer) examines: `cause=struck_by_blade`
  (examined weight 2.3) — consistent with axe, so Ilsa's weapon facet sharpens to `axe` when
  they talk.
- Ilsa tells her family and the headman (trust 1.0, conviction 1.5): their records
  `killed_by=Bera`, hops=1, conf ≈ 0.75.
- Bera tells his friend **Odo** a lie: "a stranger from the hills was at the mill". Odo (trust in
  Bera 1.0, Bera chooses conviction 1.5) gets `killed_by=hill_stranger` hops=1 conf ≈ 0.7.
- Within A, two challenger values now compete. Because Ilsa's version has more and better
  sources, after a day most A households hold `killed_by=Bera` with conf 0.8–0.9; Bera's family
  and friends hold the stranger story. Bera flees on Day 2 (status `departed` observed by many).
  Maki–Thompson stifling: by Day 4 most A residents meet people who already know; drive for this
  topic within A drops, but its salience (murder, 1.5×) keeps it alive at the tavern.

Day 6 — the trader carries it across:
- **Kesh**, a trader from B, overnights in A on Day 5 and hears it three times: from the
  innkeeper (hops 2, "Bera did it, with an axe"), from Odo (hops 2, "a hill stranger"), and
  overheard (weight ×0.6). Kesh's store: challenger set {Bera: 0.72, hill_stranger: 0.41}.
  Leveling has already dropped "dusk" and "quarrel over debt" for Kesh (p 0.15/hop × 2 hops).
- Kesh doesn't know Bera personally → `Bera` is a *name-only* entity in Kesh's mind
  (a stub entity record: "Bera, the miller's apprentice in A").

Day 8 — Village B, market day:
- Kesh tells 9 people in B. Dialect intelligibility 0.7: the word *tarum* (axe) has no cognate
  and is lost for 6 of 9 listeners (weapon facet dropped); the name is re-phonologized to *Pera*.
  B listeners' records: `killed_by=Pera` (hops 3, trust in trader 0.6, conf ≈ 0.55),
  `victim=miller_of_A` (Tomas's name too low-salience for B, leveled to his role).
- Assimilation: two B listeners with high prejudice against hill folk and only weak conf in "Pera"
  adopt Kesh's secondary story (hill stranger) as primary. One of them, **Gorm** (drama trait
  high), sharpens: "hill raiders killed the miller of A".
- B's mean-field: the story spreads through B for ~10 days; by Day 20, ~75% of B adults have heard
  *some* version (consistent with θ≈0.2 never hearing): ~55% "a man called Pera killed the miller
  of A", ~30% "hill raiders killed the miller", ~15% vague gist "someone died up at A's mill".

Day 25 — consequences and durability:
- B's headman, who believes the raider version (conf 0.6), increases the night watch and is cold
  to hill-folk traders → a deterministic *action* driven by a *false belief* — the source of
  emergent drama the player can watch.
- A's priest, literate, **writes** in the parish record: "Tomas the miller slain by Bera his
  apprentice, Day 0". This document is now a durable carrier at hops=2 with high authority.
  When years later a B scholar reads it, she adopts `killed_by=Bera` (reading weight 1.2 ×
  authority 1.3), *reversing* the B oral tradition.

Year 3 — forgetting and folklore:
- In B, individual records' R falls (low rehearsal), most get evicted or reduced to gist. The
  settlement-level lore store (collective memory, C.(b)) keeps "the hill raiders who struck A"
  as a story item because it was retold during later conflict with the hill folk (rehearsal
  keeps S high) — the false version wins *culturally* because it is *useful* to someone.
- In A, the event persists as a strong communicative memory for ~3 generations unless written —
  here it is written, so it can be rediscovered.

The god-player can open E#9001 and see the **provenance tree** (via hidden `origin`), with each
hop's distortion operator highlighted.

### C.(b) Memory system with explicit budgets

#### Stores per agent and budgets by tier

| Store | Content | T0 (focus) | T1 (named) | T2 (background) | T3 (aggregate) |
|---|---|---|---|---|---|
| Sensory/percept buffer | events perceived this tick | transient, not saved | transient | transient (only salient events) | none |
| Working memory | current goal, ≤4 attended entities/topics | 4–7 slots | 4 slots | 2 slots | none |
| Episodic (recent) | episode records | 96 | 24 | 4 | none |
| Episodic (pinned/long-term) | emotionally defining memories (DF-style) | 16 | 8 | 2 | none |
| Semantic beliefs | BeliefRecords (C.(a)) | 512 | 128 | 24 (gist only) | distribution per settlement |
| Social model | per-known-person summary (affinity, trust, last seen, 3 key facts) | 200 | 60 | 10 | n/a |
| Procedural | skill levels + habit strengths | full | full | coarse | per-occupation averages |
| Schemas | learned generalizations ("strangers steal", "spring floods") | 32 | 12 | 3 (inherited cultural) | cultural set |
| LLM text memory | Graphiti/Mem0-like store of dialogue | yes (T0 only) | no | no | no |

Memory at T1: roughly 24×32 B + 8×32 B + 128×48 B + 60×24 B ≈ 9 KB/agent → 10k T1 agents ≈ 90 MB.
T2 at ~1.5 KB/agent → 100k ≈ 150 MB. Acceptable; tune numbers later.

#### Episode record (~32 bytes)

    Episode: event_type, place, t_start, t_end(coarse), participants[≤3], role_of_self,
             outcome, valence(i8), arousal(u8), importance(u8),
             n_rehearsals(u8), t_last(u32), links: prev/next episode index (contiguity),
             belief_refs[≤2] (facts it supports — evidence pointers)

#### Episode boundaries (event segmentation)

An episode closes when (a) activity type changes, (b) location changes zone, (c) interaction
partner changes, or (d) **surprise** > θ_s, where surprise = −ln P(observed | agent's routine
model) using the agent's daily schedule as the predictor. Routine, unsurprising time produces
*no* episodes (we don't remember most commutes) — natural compression.

#### Importance (deterministic replacement for LLM importance)

    importance = clamp( a1·arousal + a2·|valence| + a3·goal_relevance
                        + a4·social_weight(participants) + a5·novelty + a6·self_involvement , 0, 1)

suggested a = (0.30, 0.15, 0.20, 0.15, 0.10, 0.10).

#### Activation / retrieval score (ACT-R + generative agents + FSRS)

For episode or belief i, given a cue set C (current place, people present, topic):

    B_i   = ln( (n_i + 1) / (1 − d) ) − d·ln(L_i)            d = 0.5, L in game-hours
    S_i   = Σ_{j∈C} (1/|C|) · (S_max − ln fan_j) · [j matches i]
    E_i   = β_e · arousal_i · (1 + 0.5·|valence_i|)            emotional boost
    A_i   = B_i + S_i + E_i + ε,   ε ~ logistic(0, s), s = 0.25
    P_i   = 1 / (1 + exp(−(A_i − τ)/s))

The rank for "what comes to mind" is A_i; the probability P_i decides whether a specific recall
attempt succeeds (enables tip-of-the-tongue and genuine forgetting). For beliefs we can use the
FSRS R(t,S) instead of B_i — both are fine; FSRS is better calibrated over long gaps, ACT-R
handles cue-dependence. Recommended: **A_i = ln R_i(Δt, S_i) + S_i(cues) + E_i** combining both.
On successful recall: n_i += 1, t_last = now, S grows as in FSRS (more if R was low).
fan_j makes interference emerge: someone who has been to the market 300 times can't recall *which*
market day something happened unless another cue disambiguates.

#### Eviction and consolidation

When the episodic ring is full, compute keep_score = A_i(no cues) + κ·importance_i; the minimum is
removed — but first run consolidation:
1. **Cluster** with existing episodes of same (event_type, primary participant or place).
2. If cluster size ≥ k (k = 3 at T1), emit/update a **semantic belief or schema**:
   e.g. 3 episodes "Tomas shouted at me" → belief `disposition(Tomas)=cruel` (inferred,
   conf from count and recency) and/or schema "millers are harsh" if spanning ≥2 millers.
   The new belief keeps belief_refs → episodes (evidence pointers, as in generative agents).
3. The evicted episode's **gist** survives inside the belief ("Tomas was cruel to me, often").
4. If arousal_i ≥ θ_flash (0.85) the episode instead goes to the **pinned** store (DF-style). A
   pinned memory is replaced only by a stronger one. Pinned memories are periodically relived
   (every ~30 days at T1: random draw weighted by arousal) causing mood effects and, if relived
   enough with the same valence, nudging personality facets (trauma → neuroticism +δ) and values.

Consolidation runs during the agent's **sleep tick** (once per game day), budget ≤ 4
consolidation ops/agent/night at T1. Replay preference: emotional and goal-relevant episodes first
(sleep replay bias).

#### Reconsolidation / distortion on recall

Each successful recall in a social context (retelling) opens a lability window:

    p_modify = μ · (1 − R_i) · suggestibility · (1 + 0.5·hops_heard_since)

If triggered, apply one operator: *align to the version just heard* (misinformation effect),
*sharpen* toward the current mood, or *source confusion* (swap `from` with another person who
also talked about it). Confidence does **not** drop on modification — and for flashbulb memories
confidence is held high artificially — reproducing "confident but wrong" witnesses.

#### Lazy deterioration (Talk of the Town without per-day rolls)

Instead of rolling every day, when a belief is created/refreshed, sample its next deterioration
time: t_next = now + Exp(rate), rate = base_rate(predicate) / (memory_trait × strength). Store it
in a global min-heap keyed by time (or check on access if t_next < now — "resolve on read").
On firing: forget (70%), mutate (20%), transfer (10%); confabulate into empty slots at 2%/day
only when the agent is *asked* about the subject (on-demand confabulation is cheaper and more
narratively relevant).

#### Collective memory (settlement lore)

Each settlement (and each culture) has a lore store of ≤ 64 story items:
`(origin_event, canonical variant, variant distribution, last_retold, carriers: oral/written/
ritual/monument, stability)`. Retold at festivals/funerals/conflicts (rehearsal raises S).
Oral-only items whose S falls below a threshold and that have no living first-hand witnesses
get dropped after roughly 3 generations (Assmann's communicative memory horizon); written or
ritualized items persist but drift toward the canonical variant (folklorization) and can become
myth (agent names replaced by archetypes after K retellings — assimilation to cultural schemas).
Newborns and immigrants are seeded from the settlement lore (teaching), which is the bridge
from individual to cultural memory and the reason two villages tell different histories.

#### Which LLM-memory ideas apply only to T0 agents

- Graphiti-style bi-temporal KG with text episodes (store dialogue with the player/god, prayers,
  visions) — T0 only; the *schema* (two timestamps, invalidate-don't-delete) is used everywhere.
- Mem0/LangMem extraction from free text — only where text exists (T0 conversations).
- A-MEM "memory evolution" with LLM rewriting; MemoryOS persona updates; generative-agents LLM
  reflections — T0 only, and outputs must be converted back into structured beliefs that the sim
  validates (an LLM reflection may *propose* `inferred: Bera is dangerous`; the sim accepts it
  only if evidence pointers exist in that agent's memory).
- Prompt assembly for a T0 agent = working memory + top-k episodes by A_i + top beliefs (with
  "I think / I heard from X / people say" hedges generated from kind/conf/hops) + pinned memories
  + lexicon-rendered names. The LLM never receives ground truth.

### C.(c) Language model of the world

#### Per-culture language object

    Language
      inventory     : consonants[], vowels[] (sampled from PHOIBLE-derived frequencies + implicational rules)
      phonotactics  : syllable templates with optional slots ("(C)V(C)", "CV(N)"), cluster bans
      stress/length : simple rules (penultimate stress etc.) for flavour
      orthography   : phoneme → romanization (per culture, so names look consistent)
      sound_changes : ordered list of rules (feature-based "A → B / env") with an age stamp
      lexicon       : concept_id → form (phoneme sequence) + frequency class
      morphemes     : reusable roots for naming (water, hill, iron, strong...) — mewo2 style
      parent        : Language id (family tree) + split date
      contact_log   : borrowed concept_ids with donor language

Concept set: ~300 concepts: a Swadesh-like core of ~100 (body, kin, numbers, nature, basic verbs)
plus culture/technology concepts added when the culture discovers them (iron, horse, god names).
New concepts get words by (a) compounding existing morphemes (prob 0.5), (b) semantic extension
of an existing word (0.2), (c) borrowing from the culture that introduced the thing (0.3 if contact
exists) — so "horse" in a culture that got horses from the steppe will be a steppe loanword.

#### Granularity: who holds a lexicon

- **Culture/settlement level (all tiers):** the canonical lexicon per settlement dialect = a diff
  over the parent language (only changed entries stored). 1,000 settlements × few hundred diffs is
  tiny.
- **Per agent (T0/T1 only):** a weighted association list for a *subset* of concepts where the
  agent deviates or is mid-change: (concept, form_variant, score) triples, ≤ 32 entries. Everything
  else defaults to the agent's home dialect. Plus a receptive set: dialects the agent understands
  (list of (dialect_id, proficiency)).

#### Drift mechanisms

1. **Sound change (regular, Neogrammarian).** Each dialect has a hazard of spawning a new sound
   change: ~1 per 50–150 game years (lower for literate, large, well-connected settlements;
   higher for small isolated ones — Nettle). A change is drawn from a catalogue of natural
   changes (lenition of intervocalic stops, final devoicing, palatalization before front vowels,
   vowel raising/merger, cluster simplification, h-loss, nasal assimilation) as feature rules.
   It starts as an *innovation* at one settlement and is applied word-by-word with frequency-based
   lexical diffusion: P(word has undergone change) follows a logistic in time since innovation,
   ordered by word frequency. Once ≥ 95% of words have it, it is "complete" and committed to the
   dialect's rule list.
2. **Lexical replacement.** Per concept a small hazard (core vocabulary ~0.1–0.2 per millennium,
   following glottochronology-ish rates; cultural vocabulary higher) of a new form (compound,
   slang, taboo replacement — e.g. after a revered ruler named "Wolf" dies, the word for wolf is
   tabooed and replaced, a real attested phenomenon).
3. **Diffusion between settlements (naming game with lateral inhibition).** Treat each
   settlement as an agent holding scores for competing variants of each in-flux concept; every
   simulated contact (trade caravan, marriage, migration, festival) is a naming-game interaction
   along the edge with weight = contact volume × prestige(source)/(prestige sum). Success → winner
   +δ, competitors −δ. Coarsening on the geographic contact graph yields stable dialect regions
   with boundaries at low-contact barriers (B.7 / Baronchelli lattice results).
4. **Children (iterated learning).** At ages ~2–12 (T1) a child's lexicon is set once per life
   stage by sampling the variants heard from household (weight 0.6), peers (0.3) and others (0.1),
   with a regularization bias (pick the majority variant with prob ∝ freq^γ, γ = 1.5 — slight
   amplification, the MAP-learner tendency). Variants heard below a frequency bottleneck are
   lost. This makes **children of immigrants speak the local dialect** while parents keep an
   accent (adult learners: their own variant score is only nudged).
5. **Contact & borrowing.** Borrowed words are adapted: each foreign phoneme maps to the nearest
   native phoneme by feature distance (PHOIBLE features); syllables repaired with epenthetic vowels
   to fit native templates. Long, intense bilingual contact without shared language spawns a
   pidgin (concept set reduced to ~200, no morphology, lexicon mixing by prestige); children
   raised in it produce a creole (new Language with two parents).
6. **Writing.** Once invented, writing slows change in the written register (a "standard" form per
   concept frozen at writing time) while speech keeps drifting → spelling no longer matches
   pronunciation; old documents become harder to read: readability(doc, reader) =
   intelligibility(doc.language_snapshot, reader.dialect) × literacy.

#### Mutual intelligibility (the link to knowledge spread)

    MI(D1, D2) = Σ_c w_c · sim(form_D1(c), form_D2(c)) / Σ_c w_c

over the core concepts with frequency weights w_c, where sim = 1 − normalized feature-weighted
edit distance (phoneme substitution cost = feature distance). Cache MI per dialect pair, update
when a change commits. MI scales transmission: in a tell across dialects, each facet survives
with p = MI^(1/redundancy), names are re-phonologized, and trust is multiplied by
(0.5 + 0.5·MI) (people trust those who speak like them). Agents with a receptive proficiency
for the other dialect use max(MI, proficiency).

#### Names

Personal and place names are generated from morphemes of the *current* dialect (mewo2 style),
so names carry meaning (Tomas = "strong-river" in A's language) that the narrator can surface,
and old place names preserve archaic forms (they undergo sound change but are exempt from
lexical replacement — just like real toponyms that fossilize older languages).

#### Animals

Animal alarm calls/proto-communication: per species a fixed small signal set (Lewis game result
precomputed offline, e.g. with EGG, or hand-authored) mapping signal → meaning (predator class);
conspecifics react; cheap and fully deterministic. No drift unless species is "intelligent".

### C.(d) How all three degrade across LOD tiers

| System | T0 focus (≤ ~50) | T1 named (≤ ~10k) | T2 background (≤ ~100k+) | T3 aggregate (1M) |
|---|---|---|---|---|
| Beliefs | full records + 2nd-order ToM for secrets; LLM may narrate with hedges | full records, 128 budget, all ops | 24 gist beliefs; events only via settlement rumor layer; no per-belief challengers (only top value + conf) | per settlement: for each active story, fractions (ignorant / spreader / stifler) × variant distribution; Maki–Thompson ODE with inter-settlement coupling by traffic |
| Telling | full topic scoring, distortions, lies | full | batched: daily "gossip step" per household; transfer best 1–2 stories | ODE / stochastic coupling (EoN-style edge flows) |
| Memory | full + LLM text memory (Graphiti-like) | full numeric memory, 24+8 episodes | 4+2 episodes, no reconsolidation, consolidation weekly | none; only settlement lore |
| Language | per-agent variants, accents, receptive multilingualism | per-agent ≤ 32 variant entries | home dialect only | settlement dialect only |
| Clock | per tick | per tick / event-driven | coarse (hourly/daily) | weekly/monthly |

**Promotion (hydration).** When a T3/T2 agent is promoted (the player zooms in or the agent
becomes important), its beliefs are *sampled* from the settlement distributions with the
deterministic RNG key (agent id, story id): e.g. if 55% of B believes "Pera did it", the agent
holds that with conf drawn from the stored confidence distribution, hops = settlement's mean
hops + 1, source = a plausible local (a T2 neighbor or "people at the market"). Episodic memory is
hydrated from life-history events the sim already logged for that individual (birth, marriage,
deaths in family, crises at their settlement) — no invention needed beyond gist.

**Demotion (compression).** Beliefs are folded back into settlement counters (increment the
variant bucket), episodes are dropped except pinned ones (kept as 2 IDs), lexicon variants reduce
to a single "accent" dialect id.

Invariant to preserve: a promoted agent must never know more than its settlement aggregate
allows (no omniscience leak through LOD transitions), and demotion must never create
knowledge in a settlement that no carrier brought.

---

## Part D — Open questions / risks

1. **Legibility vs. realism.** Players need to *see* distortion; we need UI that shows a
   provenance tree and "what this person believes" vs "what's true". Without it, distortion just
   looks like bugs.
2. **Parameter explosion.** Base strengths, distortion probabilities, decay rates, budgets: dozens
   of knobs. Need an offline test harness (seeded scenarios like the murder example) with metrics:
   reach curve, variant entropy, time-to-B, fraction never hearing (~0.2 target for mundane news).
3. **Runaway falsehood / incoherence.** Reconsolidation + assimilation + lies can drive beliefs to
   nonsense (e.g. agents believing a living person is dead forever). Need "reality anchoring":
   direct observation (seeing Tomas alive) must dominate (witness weight 3.0 vs told 0.8), and
   contradiction-with-perception triggers revision.
4. **Determinism with LLMs.** T0 LLM outputs are non-deterministic; they must only produce
   *proposals* (speech acts, inferences) validated and recorded by the sim, logged for replay.
   Narration must use the agent's belief store, never world state — test for leakage.
5. **Event-driven scheduling at scale.** A global priority queue for deterioration/telling events
   at 1M agents may be a bottleneck; consider per-region timing wheels and resolve-on-read.
6. **Language drift speed vs. game time.** Real sound change is centuries; a typical game session
   may cover 50–500 years. We may need an accelerated "drift multiplier" while keeping relative
   rates believable, or players will never notice dialects.
7. **Procedural words can be offensive or real.** Generated words may collide with slurs or real
   words in player languages. Need a blocklist filter on generated forms.
8. **Cultural stereotyping.** Sampling phonologies that mimic specific real-world languages for
   specific "barbarian" cultures could read as stereotyping; prefer mixed/abstract inventories.
9. **Theory of mind depth.** Is first-order + tags enough for deception plots (CK3-style
   secrets/hooks)? Likely yes for T1; T0 may need second-order for secrets. Unclear how often.
10. **Collective-memory realism.** The 3-generation communicative-memory horizon is a heuristic;
    validate with play rather than literature.
11. **Licenses.** Lexurgy (GPL-3.0) and SMCDEL (GPL-2.0) must remain inspiration only; Jason is
    LGPL; Ensemble uses BSD-4 (advertising clause); PHOIBLE data is CC-BY (credits). Everything
    else cited is MIT/Apache/BSD.
12. **Unverified items.** HERMES (VUB GitLab), bangyen/emergent, Babel2 GitHub release details,
    MemoryOS heat formula constants, and the exact lattice convergence exponent quoted for
    the naming game should be re-checked when arXiv/the Babel site are reachable.

---

## Sources consulted (verified pages)

- EGG: https://github.com/facebookresearch/EGG
- DeepMind emergent communication at scale: https://github.com/google-deepmind/emergent_communication_at_scale
- Babel mirror: https://github.com/martinodb/babel-core
- SimLang: https://github.com/smkirby/SimLang
- Neural Iterated Learning: https://github.com/Joshua-Ren/Neural_Iterated_Learning
- skILMpy: https://github.com/dhard/skILMpy
- Lingo: https://github.com/taabishahmad/lingo
- Raviv ABM tutorial: https://github.com/Limor-Raviv/Tutorial_Agent_Based_Models
- mewo2 naming-language: https://github.com/mewo2/naming-language
- Lexurgy: https://github.com/def-gthill/lexurgy
- PHOIBLE: https://github.com/phoible/dev
- Talk of the Town: https://github.com/james-owen-ryan/talktown and https://github.com/james-owen-ryan/talktown-testbed (belief.py, evidence.py, config.py)
- Jason: https://github.com/jason-lang/jason
- SMCDEL: https://github.com/jrclogic/SMCDEL
- NDlib: https://github.com/GiulioRossetti/ndlib
- EoN: https://github.com/springer-math/Mathematics-of-Epidemics-on-Networks
- Ensemble: https://github.com/ensemble-engine/ensemble
- Graphiti: https://github.com/getzep/graphiti ; Zep: https://github.com/getzep/zep
- Mem0: https://github.com/mem0ai/mem0
- A-MEM: https://github.com/agiresearch/A-mem ; https://github.com/WujiangXu/A-mem
- MemoryOS: https://github.com/BAI-LAB/MemoryOS
- MemoryBank: https://github.com/zhongwanjun/MemoryBank-SiliconFriend
- cognee: https://github.com/topoteretes/cognee
- MemOS: https://github.com/MemTensor/MemOS
- LangMem: https://github.com/langchain-ai/langmem
- AI Town: https://github.com/a16z-infra/ai-town
- EM-LLM: https://github.com/em-llm/EM-LLM-model
- FSRS: https://github.com/open-spaced-repetition/fsrs-rs ; algorithm wiki https://github.com/open-spaced-repetition/awesome-fsrs/wiki/The-Algorithm
- Generative agents source: https://github.com/joonspk-research/generative_agents (retrieve.py, reflect.py)
- Maki–Thompson 0.203 result: https://arxiv.org/pdf/2002.06821 (search snippet; arXiv blocked for direct fetch)
- Baronchelli minimal naming game: https://arxiv.org/pdf/1701.07419 (search snippet)
- Dwarf Fortress memory: https://www.gamedeveloper.com/game-platforms/-i-dwarf-fortress-i-is-giving-its-dwarves-the-ability-to-form-and-dwell-on-memories (search snippet; DF wiki blocked)
