# 03 — Emotions, Personality, Needs/Motivation, Decision Making, LLM + Simulation Hybrids

Research track 7–11 for the god-simulator. Research only; no game code.
Date of survey: 2026-10-01.

## How this was checked

- I checked the repo metadata (stars, license, language, activity) by fetching each GitHub page during this session. The GitHub REST API and arxiv.org, elifesciences.org, alma.dfki.de and agi-conf.org were blocked by the egress proxy. So paper-level details (formulas) come from search-engine abstracts and from my knowledge of the published papers. Each one is marked "(paraphrased from paper)".
- "unverified" means I could not confirm the item during this session.
- Star counts are approximate as of Oct 2026.
- **License flags for a possibly closed-source commercial game:**
  - **GPL/AGPL**: do not link or copy code. Reimplementing the algorithm from the paper is fine.
  - **LGPL**: dynamic linking only, and only with care. In practice, reimplement.
  - **MIT / Apache / BSD**: OK with attribution.
- Already known to the user, so not repeated as candidates: pyactr, Soar, OpenNARS, OpenCog, LIDA, pymdp, Humanoid Agents, generative agents/genagents, FAtiMA, HippoRAG, Letta/MemGPT, Project Sid, Concordia. They are referenced only where they help a comparison.

---

## Part A — Emotion models (computational appraisal)

### GAMYGDALA (Broekens / Popescu, TU Delft)
- URL: https://github.com/broekens/gamygdala (project page ii.tudelft.nl/~joostb/gamygdala/). The old `GAMYGDALA/gamygdala` URL returns 404.
- License: MIT
- Language: JavaScript (Phaser plugin). C# and Java ports are announced in the README but not verified.
- Activity/maintenance: ~37 stars, 36 commits, effectively dormant (research artifact, 2013–2015).
- Architecture (paraphrased from Popescu, Broekens & van Someren, IEEE TAC 2014):
  - Each agent holds **goals** `{name, utility ∈ [-1,1], likelihood ∈ [0,1]}`.
  - A game **belief/event** is `{likelihood, causalAgent, affectedGoals[], goalCongruences[] ∈ [-1,1], isIncremental}`.
  - Per affected goal:
    - `desirability = congruence × utility`
    - the goal's likelihood is updated toward the belief's likelihood
    - `Δlikelihood` drives intensity
  - Internal OCC emotions:
    - likelihood < 1 → hope or fear
    - likelihood = 1 → joy or distress
    - confirmation or disconfirmation of earlier hopes/fears → satisfaction, fears-confirmed, relief, disappointment
  - Social emotions use a **relation** `{target, like ∈ [-1,1]}`:
    - happy-for, resentment, gloating, pity
    - anger and gratitude when a causal agent is set
  - Emotions map to fixed **PAD vectors**. Mood (PAD) is the intensity-weighted sum of active emotions.
  - Decay is linear or exponential per tick, set globally.
  - A gain parameter scales intensity.
- Scientific concept modeled: OCC appraisal, reduced to goal-congruence plus likelihood. Mood is derived from PAD.
- Computational cost: O(affected goals × agents that know the goal) per event; decay is O(active emotions) per tick. Microseconds.
- What we can learn:
  - The cheapest credible appraisal: everything reduces to (utility, congruence, likelihood, causal agent, liking).
  - Events are authored with explicit goal effects. That is exactly what a decision trace needs.
- What we could integrate: the algorithm (reimplement in our ECS). The MIT code could be ported directly, but it is small and JS-specific.
- What we should NOT integrate:
  - Its global decay constants. Ours must be personality-dependent.
  - Its lack of standards/norms. Shame, guilt and pride are weak or absent, and we need them.
- Scalability: 10 trivially; 1k easily; 100k only if events touch few goals (keep goal lists short and sparse).

### WASABI (Becker-Asano) — WASABIEngine
- URL: https://github.com/CBA2011/WASABIEngine (GUI: https://github.com/CBA2011/WASABIQtGui)
- License: LGPL-3.0 / GPL-3.0 per repo page. **Flag: copyleft.**
- Language: C++ (core does not depend on Qt).
- Activity/maintenance: ~10 stars, 44 commits, dormant.
- Architecture (paraphrased from Becker-Asano's thesis):
  - A 2-D **emotion/mood dynamics** system, coupled like spring–mass oscillators:
    - Valence impulses from appraisal push the *emotion* axis.
    - Emotion is pulled back to zero by a spring (`F = −k·x`) and damped.
    - Mood integrates emotion slowly and has its own spring.
  - The (emotion, mood) pair plus a dominance flag is mapped into PAD space.
  - **Boredom** accumulates while both values sit near zero.
  - Primary emotions (happy, sad, angry, fearful, surprised, bored, …) are PAD regions. Activation rises when the current PAD point falls in a region's radius.
  - Secondary emotions (hope, relief, fears-confirmed) need cognitive appraisal and are "elicited" with their own decay.
- Scientific concept modeled: dimensional affect dynamics, mood as a slow integrator of emotion, and the primary/secondary emotion distinction (Damasio).
- Computational cost: a few floating-point operations per agent per tick (second-order ODE, Euler step).
- What we can learn:
  - A **physical-dynamics view** gives natural overshoot, habituation and "mood inertia" for free.
  - Boredom emerges from low affect. That is a great idle-behaviour trigger.
- What we could integrate: the dynamics equations (reimplement). Do not ship the LGPL/GPL code.
- What we should NOT integrate: the Qt GUI and the network (UDP) interface. Also skip the oscillatory parameters as they are; underdamped mood looks bipolar in long runs.
- Scalability: 100k+ trivially; vectorizable as SoA.

### ALMA — A Layered Model of Affect (Gebhard, DFKI)
- URL: paper https://alma.dfki.de/papers/aamas05.pdf (fetch blocked; reached through search). Code: no public repository found (unverified).
- License: n/a (Java toolkit was distributed by DFKI historically; terms unverified).
- Language: Java (historical).
- Activity/maintenance: research artifact (AAMAS 2005).
- Architecture (paraphrased from paper):
  - Three layers: personality (Big Five), mood (PAD in [-1,1]³) and emotions (24 OCC types).
  - Default mood comes from personality via Mehrabian's mapping:
    - P = 0.21E + 0.59A + 0.19N
    - A = 0.15O + 0.30A − 0.57N
    - D = 0.25O + 0.17C + 0.60E − 0.32A
    - (N here is the *emotional-stability* direction. Check sign conventions before use.)
  - Mood octants are labelled: Exuberant (+P+A+D), Bored (−−−), Relaxed (+−+), Anxious (−+−), Dependent (++−), Disdainful (−−+), Hostile (−++), Docile (+−−).
  - Each tick, active emotions are mapped to PAD and averaged into a **virtual emotion center**. Mood moves by a **pull-and-push** rule:
    - If mood lies between the origin and the center, it is *pulled* toward the center.
    - If mood is already beyond the center, it is *pushed* further out (intensification).
  - Without emotions, mood decays back to the personality default over a configurable period (minutes in the original).
  - Appraisal rules are authored in AffectML (XML).
- Scientific concept modeled: the emotion / mood / personality time-scale hierarchy.
- Computational cost: O(active emotions) per tick; negligible.
- What we can learn:
  - The **exact 3-layer contract** we want.
  - A Big-Five→baseline-mood mapping we can cite.
  - The push/pull rule produces "a bad day gets worse" dynamics.
- What we could integrate: the algorithm and the coefficients (published science; not copyrightable as such).
- What we should NOT integrate: XML AffectML authoring (use data tables). Also the minutes-scale decay; game time needs hours to days.
- Scalability: 100k+.

### EMA — EMotion and Adaptation (Gratch & Marsella, USC ICT)
- URL: papers only, e.g. http://www.ccs.neu.edu/~marsella/publications/pdf/MarsellaCSR09.pdf and https://people.ict.usc.edu/~gratch/GratchMarsellaCOGSYS04.pdf. No public code found (unverified).
- License: n/a
- Language: originally Soar/Lisp (historical).
- Activity/maintenance: theory remains influential (e.g. 2026 Sentipolis cites it); no code.
- Architecture (paraphrased):
  - The agent's **causal interpretation** (a plan-like graph of beliefs, intentions, actions and effects) is appraised continuously.
  - For each (proposition, agent) pair there is an **appraisal frame** with these variables:
    - relevance
    - desirability
    - likelihood
    - expectedness
    - causal attribution (who caused it, intention or no intention)
    - controllability
    - changeability
    - urgency
    - ego involvement
  - Frames are labelled with discrete emotions (hope/fear/joy/distress/anger/guilt …) with intensity ≈ desirability × likelihood.
  - **Mood** is a running aggregate of the frames and biases which frame becomes the "focus".
  - **Coping** proposes strategies in parallel and adopts them one at a time:
    - Problem-focused (planning, seeking instrumental support, taking action) is preferred when control is appraised as high.
    - Emotion-focused (positive reinterpretation, denial or wishful thinking, mental disengagement, shifting blame, resignation) is used when controllability and changeability are low.
  - Coping *changes beliefs and intentions*, which triggers re-appraisal. That loop is the appraisal → coping → re-appraisal cycle.
- Scientific concept modeled: Lazarus' cognitive-motivational-relational theory, with coping as belief/intention manipulation.
- Computational cost: proportional to the size of the causal interpretation. With a 10–30 node plan graph, ~10–100 µs per re-appraisal.
- What we can learn:
  - **Emotion → coping → belief change** is the missing link for believable humans. Denial, blame-shifting and resignation become first-class *mental actions* that show up in decision traces.
  - Controllability decides whether fear leads to fleeing, freezing or preparing.
- What we could integrate: the appraisal-variable set and the coping-selection table (reimplement from the papers).
- What we should NOT integrate: full plan-graph appraisal for background agents. Restrict it to focus tiers.
- Scalability: 10–1k full. At 100k, only cached frame updates on salient events.

### PsychSim (USC)
- URL: https://github.com/usc-psychsim/psychsim
- License: MIT
- Language: Python
- Activity/maintenance: ~22 stars, 226 commits; low activity.
- Architecture:
  - Decision-theoretic agents (factored POMDP-like).
  - State features are stored in piecewise-linear (PWL) trees.
  - Each agent has a reward function (a weighted sum of state-feature objectives), a horizon, and **recursive models of other agents** (beliefs about their rewards and beliefs: theory of mind).
  - Action selection is bounded-lookahead expected reward over the agent's beliefs.
  - Si, Marsella & Pynadath (paraphrased) derived appraisal dimensions straight from these quantities:
    - motivational relevance = utility change
    - novelty = surprise vs. the predicted action
    - accountability = which agent's action caused the change
    - control = whether any reachable policy improves the outcome
- Scientific concept modeled: recursive theory of mind and decision-theoretic appraisal.
- Computational cost: exponential in horizon × modeled agents. Seconds per decision with ToM depth ≥ 2 in Python.
- What we can learn:
  - **Appraisal falls out of the decision model**, so there is no separate emotion rulebase: "Surprise" = prediction error on another's action, and "Blame" = an attributed negative utility change.
  - Shallow ToM (depth 1, horizon 1–2) is enough for most social cues.
- What we could integrate: the idea and the formulas. The MIT code is usable but far too slow; reimplement only depth-1 ToM.
- What we should NOT integrate: recursive depth > 1 except for scripted "schemer" characters at focus LOD.
- Scalability: 10 (depth 2) / 1k (depth 1, horizon 1, cached) / 100k no.

### FLAME — Fuzzy Logic Adaptive Model of Emotions (El-Nasr, Yen, Ioerger 2000)
- URL: paper only (Autonomous Agents and Multi-Agent Systems 3(3), 2000; PDF via researchgate/semanticscholar). No code.
- License: n/a
- Language: n/a
- Activity/maintenance: classic, frozen.
- Architecture (paraphrased):
  - Event desirability comes from fuzzy rules over goal impacts (`IF goal importance is high AND impact is very negative THEN desirability is very undesirable`).
  - OCC rules turn desirability + expectation into emotions.
  - Emotions pass through a filtering/mood step and a decay step.
  - **Learning components:**
    - reinforcement-style learning of event→goal-impact associations
    - learned expectations (probabilities of event sequences)
    - classical-conditioning associations of objects with emotions (e.g. a pet fears the user's hand after being hit)
- Scientific concept modeled: OCC with learned expectations and conditioning.
- Computational cost: tiny (rule evaluation).
- What we can learn: **conditioned emotional associations** (place/person/object → emotion tag). Animals and humans develop phobias and fondness from history. Cheap and very visible to a god-player.
- What we could integrate: the conditioning mechanism (store `assoc[agent][entity] += α·(felt valence − assoc)`).
- What we should NOT integrate: fuzzy inference for everything. Linear curves are easier to explain in a trace.
- Scalability: 100k with sparse association maps (top-k entities per agent).

### Cathexis (Velásquez 1997) and MAMID (Hudlicka)
- URL: papers only. No code found (unverified).
- License: n/a
- Language: n/a
- Activity/maintenance: historical.
- Architecture (paraphrased):
  - **Cathexis**: a network of "proto-specialists", one per basic emotion. Each has releasers (neural, sensorimotor, motivational, cognitive), an activation threshold, a saturation level and a decay function. Emotions inhibit and excite each other; e.g. anger inhibits fear.
  - **MAMID**: personality traits and affective states become **parameters on cognitive modules** (attention speed and capacity, threat bias in interpretation, goal-management priorities). Example: high trait anxiety → narrowed attention and a threat-interpretation bias.
- Scientific concept modeled: emotion as mutually inhibiting systems (Cathexis); emotion as a modulator of cognition (MAMID).
- Computational cost: tiny.
- What we can learn:
  - From Cathexis: **cross-inhibition** among emotions, so an agent is not simultaneously maximally angry and afraid.
  - From MAMID: affect changes *how* the agent perceives and decides, not only *what* it feels. This matches PSI modulators.
- What we could integrate: an inhibition matrix over our emotion set, and MAMID-style "affect → parameter" mappings (attention radius, perceived risk multiplier).
- What we should NOT integrate: n/a (concepts only).
- Scalability: 100k.

### Scherer's Component Process Model — CPM-MultiAgent (2026) as a modern implementation
- URL: https://arxiv.org/abs/2607.07824 (via search; arxiv fetch blocked). Code: unverified.
- License: unverified
- Language: unverified (LLM multi-agent pipeline)
- Activity/maintenance: new (mid-2026).
- Architecture (from abstract):
  - Emotion is a latent state reshaped by triggers.
  - A **Trigger Analyzer** agent extracts affectively salient stimuli.
  - Several appraisal agents then evaluate Scherer's **stimulus evaluation checks** (SECs) in sequence:
    - relevance (novelty, intrinsic pleasantness, goal relevance)
    - implication (causality, outcome probability, discrepancy from expectation, goal conduciveness, urgency)
    - coping potential (control, power, adjustment)
    - normative significance (internal and external standards)
  - Their results update an emotion state.
- Scientific concept modeled: CPM's sequential SECs.
- Computational cost: several LLM calls per trigger. Expensive (hundreds of ms to seconds).
- What we can learn:
  - The SEC sequence is a good **ordering for a cheap symbolic appraisal pipeline**. Each check can short-circuit: irrelevant → stop.
  - Normative significance is the explicit slot for shame/guilt/pride/indignation.
- What we could integrate: the SEC checklist as our appraisal struct (symbolic; no LLM).
- What we should NOT integrate: one LLM agent per check. That is far too costly for a simulation.
- Scalability: LLM version 10 at most. A symbolic SEC version reaches 100k.

### Sentipolis — Emotion-Aware Agents for Social Simulations (ACL Findings 2026)
- URL: https://aclanthology.org/2026.findings-acl.368/ (arXiv 2601.18027). Code: unverified.
- License: unverified
- Language: unverified (Python/LLM)
- Activity/maintenance: 2026 paper.
- Architecture (from abstract):
  - A continuous **PAD** state per LLM agent.
  - **Dual-speed dynamics** (fast emotion, slow mood).
  - **Emotion–memory coupling**: emotional state is stored with memories and recalled memories re-induce affect.
  - It targets "emotional amnesia" in LLM agents, where each turn forgets the previous affect.
- Scientific concept modeled: mood-congruent memory and two-timescale affect.
- Computational cost: LLM call per turn plus a small numeric state.
- What we can learn: evidence that **keeping affect in the engine** (numeric, persistent) and injecting it into prompts beats asking the LLM to "remember how it felt". That supports our LLM boundary.
- What we could integrate: the design pattern (numeric affect state owned by the simulation; memory records carry valence/arousal tags; retrieval is biased by current mood).
- What we should NOT integrate: n/a until code and license are verified.
- Scalability: the numeric part scales; the LLM part is limited to focus agents.

### Emotional contagion models (ASCRIBE — Bosse et al.; Tsai et al. evaluation)
- URL: papers. Survey: https://link.springer.com/article/10.1007/s10458-022-09589-z. Tsai et al.: https://projects.iq.harvard.edu/files/teamcore/files/2011_33_teamcore_tsai_iva.pdf
- License: n/a
- Language: n/a
- Activity/maintenance: active research area (2022 systematic review; 2026 LLM-crowd contagion preprint arXiv 2607.25140).
- Architecture (paraphrased):
  - For receiver r and senders s, ASCRIBE updates emotion level `q_r`:
    - `q_r ← q_r + γ_r·(q*_r − q_r)·Δt`
    - `q*_r` = weighted mean of sender emotions, weights `ε_s·α_sr·δ_r` (sender expressiveness × channel strength × receiver openness).
  - An amplification/absorption bias term lets groups escalate (spirals) or damp.
  - It is analogous to heat diffusion: susceptibility is like heat capacity.
  - It beat simpler epidemic (SIS-style) models on real panic incidents.
- Scientific concept modeled: emotional contagion and group emotion.
- Computational cost: O(neighbors) per agent per tick. With a spatial grid and k ≤ 8 neighbors, about 50–100 ns.
- What we can learn: a ready, validated formula for **crowd panic, mourning, celebrations and mob anger**.
- What we could integrate: the formula. Openness from Big-Five (Agreeableness, Neuroticism, and the HEXACO Emotionality factor); expressiveness from Extraversion.
- What we should NOT integrate: pure epidemic contagion without appraisal. Contagion should modulate mood, not overwrite appraised emotions.
- Scalability: 100k–1M on GPU or SoA; this is a stencil operation.

---

## Part B — Personality

### Dwarf Fortress personality facets / values / needs / stress (design reference)
- URL: https://dwarffortresswiki.org/index.php/Personality_facet , https://dwarffortresswiki.org/index.php/Personality_value , http://www.dwarffortresswiki.org/index.php/Need , https://dwarffortresswiki.org/index.php/Stress
- License: proprietary game; the wiki is community-written. Use as design reference only.
- Language: n/a
- Activity/maintenance: the game is active (Steam release 2022+).
- Architecture (from wiki):
  - About 50 **facets** in 0–100 with 7 report bands (40–60 is silent). Examples: anxiety, anger propensity, bravery, stress vulnerability, gregariousness, altruism, greed, curiosity, perseverance, vanity, discord.
  - About 30 **values** (beliefs) in −50..50 with 7 bands (−10..10 silent). Examples: tradition, cooperation, sacrifice, law, family, friendship, power, knowledge, nature, independence.
  - **Needs** are generated *from* facets and values, with per-dwarf weights: pray, socialize, drink, family, craft, learn, fight, wander, … Each has a focus level from "Unfettered" (+400..300) down to "Badly distracted" (≤ −100,000).
  - **Thoughts** (emotion events) add or remove stress, scaled by facets:
    - bravery → how fast stress accumulates
    - stress vulnerability → breaking threshold
    - anxiety → dissipation rate
  - Personality-dependent breakdowns: tantrum vs. depression vs. oblivious.
- Scientific concept modeled: lexical traits plus values. Needs derived from personality (close to McClelland/Murray).
- Computational cost: tiny per dwarf.
- What we can learn:
  - **Needs are personality-generated.** A gregarious person has a "socialize" need; a pious person a "pray" need.
  - **Values are separate from traits** and drive standards-based emotions (shame/pride/indignation).
  - Silent mid-bands keep the UI clean: only show notable traits.
  - Stress as a slow accumulator with personality-specific failure modes.
- What we could integrate: the design pattern only (original names/values of our own).
- What we should NOT integrate: their facet list verbatim (IP and redundancy). Ground ours in Big Five/HEXACO facets plus a values set (e.g. Schwartz values).
- Scalability: 100k (bytes per agent).

### Big Five / HEXACO → parameters, and the maturity principle (science reference)
- URL: Roberts, Walton & Viechtbauer 2006 meta-analysis (Semantic Scholar entry "Patterns of mean-level change in personality traits across the life course"); MIDUS summary https://midus.wisc.edu/findings/pdfs/1696.pdf
- License: n/a (science)
- Language: n/a
- Activity/maintenance: established findings, replicated cross-culturally.
- Architecture (findings):
  - From 92 longitudinal samples, ages 10–101: Conscientiousness, Agreeableness and social dominance (an Extraversion facet) rise with age; Neuroticism falls. Most change happens at ages 20–40, and some continues into old age.
  - Life events (first job, partnership, parenthood, bereavement) are associated with small shifts.
  - Rank-order stability increases with age (plasticity declines).
  - HEXACO adds **Honesty-Humility**, which predicts cheating, exploitation, greed and fairness better than Big-Five Agreeableness. That is useful for a world with theft, lying and politics.
- Scientific concept modeled: trait stability and change.
- Computational cost: one update per agent per in-game month.
- What we can learn: personality should **drift** with a target that depends on age and an event-driven perturbation. Plasticity must fall with age.
- What we could integrate: the formula in "Recommended design" (§R2).
- What we should NOT integrate: Neuroticism treated as only negative. It also increases threat vigilance, which can be adaptive in dangerous worlds.
- Scalability: 1M+ (monthly scalar update).

---

## Part C — Needs and motivation

### MicroPsi2 / Dörner's PSI theory (Joscha Bach)
- URL: https://github.com/joschabach/micropsi2
- License: a license.txt file is present in the repo, but its type was not identified during this session (unverified; historically MIT-like). **Verify before reuse.**
- Language: Python (web server Bottle; Theano for nets; Paper.js UI)
- Activity/maintenance: ~192 stars, ~4,067 commits; dormant for years (Theano dependency is dead).
- Architecture (paraphrased from Bach, "Modeling Motivation in MicroPsi 2", AGI 2015; fetch blocked, reached through search):
  - **Needs (demands)**, grouped as:
    - physiological: food, water, integrity/pain avoidance, rest
    - social: affiliation (legitimacy), status, nurturing, romance/sex
    - cognitive: competence (task-specific and general), certainty (predictability), aesthetics
  - Each need has a tank-like value with a target, a **decay/leak**, and weights for gain and loss.
  - **Urge** = deviation from the target × weight.
  - **Urgency** = how soon the deficit becomes critical.
  - A change toward satisfaction yields a **pleasure** signal. Increasing deficit (frustration, pain) yields a **displeasure** signal. These signals are the reinforcement used to learn associations between situations, actions and need changes.
  - **Motive selection**: candidate motives are scored about `urge × urgency × expected success`, where expected success comes from competence, i.e. learned probability. The current motive gets a **selection-threshold** bonus (hysteresis), which stops dithering.
  - **Modulators:**
    - arousal (rises with total urge and urgency)
    - resolution level (falls as arousal rises: hasty, coarse perception)
    - selection threshold (rises with arousal: harder to switch)
    - valence
  - **Emotions are not separate modules**. They are configurations of modulators plus the situation, e.g. anger ≈ high arousal + low resolution + obstacle attributed to an agent; anxiety ≈ certainty need high + low competence expectation.
- Scientific concept modeled: integrated motivation/emotion/cognition. Emotion is a modulation pattern.
- Computational cost: a need update is O(#needs ≈ 10–15) per tick; motive selection O(#motives). Microseconds. Node-net cognition is much more.
- What we can learn:
  - **Cognitive needs (certainty, competence) generate curiosity and exploration** without a separate curiosity module. They also explain why agents stick to known routines when stressed.
  - The selection-threshold hysteresis is crucial for stable, believable behaviour.
  - Modulators give a principled "stress makes you dumber/faster" effect: lower planning depth, shorter perception radius.
- What we could integrate: the need taxonomy, the urge/urgency/expected-success scoring, the modulators and the hysteresis (reimplement).
- What we should NOT integrate: the codebase (Theano, Python 3 web stack, node nets) or the node-net perception/memory.
- Scalability: the motivational core reaches 100k–1M as SoA. Node nets reach 10 at most.

### Homeostatic Reinforcement Learning (Keramati & Gutkin) and HRRL code
- URL: paper eLife 2014;3:e04811 (https://elifesciences.org/articles/04811, fetch blocked; found through search). Third-party code: https://github.com/vagansh/HRRL (Continuous-time HRRL, arXiv 2401.08999).
- License: paper CC-BY (eLife standard). The HRRL repo has **no license** → do not reuse its code.
- Language: Python (HRRL repo)
- Activity/maintenance: HRRL repo ~1 star, 11 commits. The theory is well cited; follow-ups include "empathic coupling of homeostatic states for prosociality" (arXiv 2412.12103, 2506.12894).
- Architecture (paraphrased from paper):
  - The internal state `H = (h_1..h_N)` has setpoints `H*`.
  - Drive: `D(H) = ( Σ_i |h*_i − h_i|^n )^(1/m)` with n > m > 1.
  - Reward for an outcome K: `r(H_t, K) = D(H_t) − D(H_t + K)`, i.e. drive reduction. Standard TD learning is then applied to this reward.
  - Predictions:
    - the same food is more rewarding when deprived
    - excess beyond the setpoint is aversive
    - **risk aversion** emerges from the drive function's convexity
    - **anticipatory responding**: eat before you are hungry if hunger is predictable
- Scientific concept modeled: unification of drive-reduction and reward-maximization.
- Computational cost: the drive is O(N) per evaluation. With the formula as a utility input, about 50 ns.
- What we can learn:
  - **Use the drive formula as the utility of consumption actions.**
  - It gives correct cross-need trade-offs, because the convexity makes the worst-off need dominate.
  - Satiation comes free.
  - It produces a natural risk-aversion signal for gambles that may worsen a deficit.
  - Empathic-coupling papers show how to make *another agent's* deficits enter one's own drive. That fits a parent with a hungry daughter (`D` includes `w_child·deficit_child`).
- What we could integrate: the formula (from the paper; CC-BY).
- What we should NOT integrate: full TD learning per agent at large scale. Use learned values only for "focus" agents, or as shared per-archetype tables.
- Scalability: 1M as a closed-form formula.

### Incentive salience: "wanting" vs "liking" (Zhang, Berridge et al. 2009)
- URL: https://journals.plos.org/ploscompbiol/article?id=10.1371%2Fjournal.pcbi.1000437 ; https://www.jneurosci.org/content/29/39/12220
- License: PLoS CC-BY
- Language: n/a (math model)
- Activity/maintenance: established; extensions exist (multiple-attribute model, arXiv 1812.08308).
- Architecture (paraphrased):
  - Cue value is computed on the fly: `Ṽ(s) = r̃(r, κ) + γ·V(s')`.
  - The current physiological/neuro state `κ` (hunger, satiety, drug sensitization) modulates learned reward. The form is multiplicative `κ·r` or additive `r + log κ`, depending on the case.
  - The result: "wanting" can change instantly without relearning, and can even be high for something never "liked" (the salt-appetite experiments).
- Scientific concept modeled: the dissociation of wanting from liking. Explains addiction, cravings and cue-triggered urges.
- Computational cost: one multiply per cue.
- What we can learn: separate **"wanting"**, which drives choice, from **"liking"**, which drives the hedonic emotion after consumption.
  - Addictions: wanting is sensitized, liking stays flat.
  - Cravings: a cue (smell of bread) spikes wanting.
  - All of this is visible and explainable in a trace ("wanting 0.9 due to cue; liking last time 0.3").
- What we could integrate: a `κ` modulation on cue-driven utility considerations.
- What we should NOT integrate: n/a.
- Scalability: 1M.

### Self-Determination Theory, Maslow critiques, allostasis (science reference, no repo)
- URL: SDT (Deci & Ryan); Kenrick et al. 2010 "Renovating the pyramid of needs" (evolutionary revision of Maslow); Sterling's allostasis. These were not separately fetched this session, and the citations are from domain knowledge.
- License: n/a
- Language: n/a
- Activity/maintenance: n/a
- Architecture (concepts):
  - **SDT**: three psychological needs (autonomy, competence, relatedness). Frustration of them lowers wellbeing and intrinsic motivation.
  - **Kenrick's revision**: fundamental motives (self-protection, disease avoidance, affiliation, status, mate acquisition, mate retention, kin care) are *activated* by cues and are **not** strictly hierarchical. Parenting sits at the top and can override lower needs, which is our daughter example.
  - **Allostasis**: setpoints themselves move with predicted demand and chronic stress, e.g. a chronically threatened agent has a raised vigilance setpoint and accumulates "allostatic load", which harms health.
- Scientific concept modeled: non-hierarchical, cue-activated motive systems and predictive regulation.
- Computational cost: n/a
- What we can learn:
  - **Do not implement Maslow as a strict hierarchy.** Use weighted, cue-activated motive systems whose weights come from personality, life stage and situation.
  - Allostatic load is a good slow variable linking stress to health and lifespan.
- What we could integrate: the motive taxonomy and allostatic load as a slow accumulator.
- What we should NOT integrate: strict prerequisite gating ("no social needs until fed").
- Scalability: 1M.

---

## Part D — Decision making

### Curvature (Infinite Axis Utility System editor; Mike Lewis / Guild Wars 2 lineage)
- URL: https://github.com/apoch/curvature
- License: custom License.txt (not identified; verify)
- Language: C#
- Activity/maintenance: ~290 stars, 205 commits; open beta, slow.
- Architecture:
  - Knowledge base → **inputs** → **considerations**. Each consideration is an input normalized to [0,1] through a parametric **response curve** (linear, quadratic, logistic, logit) with slope, exponent and x/y shifts.
  - **Behaviors** = sets of considerations plus a weight.
  - Behavior sets attach to **archetypes**.
  - IAUS scoring (Dave Mark, paraphrased):
    - multiply all consideration scores
    - correct for the count with a compensation factor `mod = 1 − 1/n`, `score += (1 − score)·mod·score`
    - multiply by the behavior weight
    - pick the top behavior, or a weighted random choice among the top-k
  - Scenario sandbox with **decision logs**.
- Scientific concept modeled: multi-attribute utility with nonlinear response.
- Computational cost: about (#behaviors × #considerations) curve evaluations: 20 × 5 = 100 evaluations ≈ 1–3 µs.
- What we can learn:
  - Utility AI is the **most explainable** fast decision method, because every number in the trace is a named consideration.
  - Curves are authored, not learned, so designers can tune "how much hunger matters".
  - A per-decision log is part of the authoring tool. Copy that UX for the god inspector.
- What we could integrate: the algorithm; ideas for a curve editor.
- What we should NOT integrate: the code (license unclear; C# WPF tool).
- Scalability: 100k at 1 decision per agent per game-minute on a few cores.

### big-brain (Bevy utility AI, Rust)
- URL: https://github.com/zkat/big-brain (archived Oct 2025; moved to Codeberg)
- License: Apache-2.0
- Language: Rust
- Activity/maintenance: ~1.3k stars. Archived on GitHub; continued on Codeberg (Codeberg activity unverified).
- Architecture:
  - ECS-native. **Scorers** are entities/components that write a Score each frame. **Actions** are components with an ActionState machine (Requested/Executing/Success/Failure/Cancelled). **Thinkers** combine scorers + actions with a **Picker** (FirstToScore, Highest, …).
  - Composite scorers (AllOrNothing, SumOfScorers, ProductOfScorers, WinningScorer).
  - Systems run in parallel under Bevy's scheduler.
- Scientific concept modeled: utility AI in a data-oriented ECS.
- Computational cost: ~1 µs-scale per thinker per frame. Overhead is dominated by ECS iteration.
- What we can learn: how to make utility AI **data-parallel**. Scorers are plain systems over component arrays; actions are long-running state machines that can be cancelled when a higher-scoring option appears.
- What we could integrate: code (Apache-2.0) if we choose Bevy/Rust; otherwise the pattern.
- What we should NOT integrate: per-entity scorer entities at 100k+ scale (entity bloat). Batch scorers instead.
- Scalability: 10k comfortable; 100k with batching.

### GOAP for Unity (crashkonijn)
- URL: https://github.com/crashkonijn/goap
- License: Apache-2.0
- Language: C# (Unity)
- Activity/maintenance: ~1.8k stars, 863 commits; active; used in shipped games.
- Architecture:
  - Sensors (world and target) fill a world state.
  - Goals and actions carry conditions, effects and costs.
  - The planner (A*-style) is multi-threaded through Unity's Job System.
  - A runtime visualizer shows the graph and the chosen path.
  - The README shows 2,000+ agents in a performance demo.
- Scientific concept modeled: STRIPS-like regression planning (Orkin's F.E.A.R. GOAP).
- Computational cost: plan search ~10–500 µs depending on action count (10–40) and depth (3–6). Run only on replan triggers.
- What we can learn:
  - GOAP's value is **chaining** ("to feed daughter: get food ← buy ← have money ← sell pelt").
  - Replan only on invalidation.
  - Visualizing the plan graph is directly useful for the god-inspector.
- What we could integrate: the code if we use Unity; otherwise the job-system batching pattern.
- What we should NOT integrate: GOAP as the *goal selector*. Utility should pick goals; GOAP only plans for them.
- Scalability: 1k comfortable; 10k with plan caching per archetype; 100k no (use templates).

### dogoap (data-oriented GOAP, Rust/Bevy)
- URL: https://github.com/victorb/dogoap
- License: MIT
- Language: Rust
- Activity/maintenance: ~203 stars, 91 commits; active CI.
- Architecture: states, actions and goals are *runtime data* (not compile-time types), so they can be defined from data files. There is a standalone `dogoap` crate plus `bevy_dogoap`. WASM demos.
- Scientific concept modeled: GOAP.
- Computational cost: similar to other GOAP (µs to ms per plan).
- What we can learn: data-driven action definitions. The action library can grow from mods or from LLM-proposed recipes validated by the engine.
- What we could integrate: code (MIT) if Rust.
- What we should NOT integrate: n/a
- Scalability: like GOAP above.

### BehaviorTree.CPP (+ Groot2)
- URL: https://github.com/BehaviorTree/BehaviorTree.CPP
- License: MIT
- Language: C++17
- Activity/maintenance: ~4.2k stars; very active (ROS2 Nav2 uses it).
- Architecture:
  - XML-defined trees loaded at runtime.
  - Typed **ports** with blackboard dataflow.
  - Asynchronous (non-blocking) actions are first class; reactive sequences and fallbacks re-check conditions.
  - Plugins for custom nodes.
  - Built-in loggers (file, SQLite, ZeroMQ to Groot2) record **every state transition** for replay.
- Scientific concept modeled: behavior trees (reactive control).
- Computational cost: a tick is O(visited nodes): about 0.1–1 µs per node; 50-node trees ≈ 5–20 µs.
- What we can learn: **transition logging and replay** is exactly the "why did X happen" history we want, and Groot2-style timelines are a UI reference.
- What we could integrate: code (MIT) for executing *action scripts* (how to "cook", "hunt", "nurse a child") below the decision layer.
- What we should NOT integrate: BTs as the top-level decider. Deep priority trees become unmaintainable and emotionally flat. Utility should choose; BTs should execute.
- Scalability: 10k with shared tree definitions and per-agent compact state. 100k needs custom flat BT interpreters.

### py_trees
- URL: https://github.com/splintered-reality/py_trees
- License: BSD (per repo; license type not shown on fetched page, unverified)
- Language: Python
- Activity/maintenance: ~641 stars, ~1,452 commits; active (supports Python 3.14).
- Architecture: behaviours/decorators/composites (Sequence, Selector, Parallel); a blackboard with access-controlled clients; visitors for tracing; ASCII/dot rendering of the tree with each node's status per tick.
- Scientific concept modeled: BTs.
- Computational cost: Python; ~10–50 µs per node.
- What we can learn: **Visitors + snapshot rendering** are good for prototyping the trace UI. Blackboard activity streams show who read and wrote what.
- What we could integrate: prototyping only.
- What we should NOT integrate: runtime at scale (Python).
- Scalability: 10–100.

### GTPyhop (HTN with goals and tasks)
- URL: https://github.com/dananau/GTPyhop
- License: BSD-3-Clause-Clear
- Language: Python
- Activity/maintenance: ~98 stars, 60 commits; maintained by Dana Nau.
- Architecture:
  - State = a Python object of dicts.
  - To-do list = tasks, goals or multigoals.
  - **Task methods** decompose tasks; **goal methods** achieve unigoals; actions modify state.
  - Depth-first backtracking decomposition.
  - Includes **Run-Lazy-Lookahead** (plan, act, replan on failure) for acting.
- Scientific concept modeled: HTN planning (SHOP-style, totally ordered) with goal reasoning.
- Computational cost: depends on domain. Small domains take ~1–10 ms in Python; a C++ port would be about 100× faster.
- What we can learn:
  - HTN methods are **culturally encoded know-how** ("to hold a funeral: wash body, gather kin, bury, feast"), which suits a civilization sim.
  - Method choice order can encode personality and culture.
  - Mixing goals and tasks lets the utility layer hand down either.
- What we could integrate: the algorithm (BSD code is usable for tooling and prototyping).
- What we should NOT integrate: Python at runtime.
- Scalability: in a C++/Rust port with cached decompositions, 10k (plans are reused heavily).

### SHOP3
- URL: https://github.com/shop-planner/shop3 (manual https://shop-planner.github.io/)
- License: MPL-2.0 (from memory; repo page not fetched — unverified)
- Language: Common Lisp
- Activity/maintenance: maintained by SIFT (activity unverified).
- Architecture: totally-ordered HTN with explicit stack search, PDDL-ish domains, plan trees with *dependency annotations*, and plan repair.
- Scientific concept modeled: HTN (SHOP2 successor).
- Computational cost: fast for HTN, but Lisp.
- What we can learn: **plan trees with causal dependencies** give "why" explanations ("bought bread *because* method feed-family needed food") and support **plan repair** instead of full replanning.
- What we could integrate: the ideas.
- What we should NOT integrate: Common Lisp runtime.
- Scalability: n/a for us (offline reference).

### PANDA / pandaPI (HTN planning system)
- URL: https://github.com/panda-planner-dev/pandaPIengine (also pandaPIparser, pandaPIgrounder; https://panda-planner-dev.github.io/)
- License: BSD-3-Clause (engine)
- Language: C++
- Activity/maintenance: engine ~20 stars, 481 commits; grounder handles IPC-2023 instances.
- Architecture: HDDL input → parse → ground → heuristic search (default greedy best-first with the RC-FF heuristic, visited lists). Also has partial-order HTN support and plan verification/repair tools.
- Scientific concept modeled: state-of-the-art HTN planning.
- Computational cost: grounding is heavy (offline); search ms to s.
- What we can learn: **HDDL** as a standard description language for our action/method library, and offline **verification** of authored methods. Not suited to run per agent.
- What we could integrate: offline validation tooling (BSD).
- What we should NOT integrate: runtime per-agent planning with full grounding.
- Scalability: offline only.

### Jason (AgentSpeak BDI interpreter)
- URL: https://github.com/jason-lang/jason
- License: LGPL-3.0. **Flag.**
- Language: Java
- Activity/maintenance: ~259 stars, ~1,679 commits; v3.2; active.
- Architecture: the BDI reasoning cycle:
  1. perceive → belief update
  2. event queue (belief/goal additions)
  3. select event
  4. find relevant plans (`trigger : context <- body`)
  5. applicable plans (context holds)
  6. select an option to create or extend an intention
  7. select an intention, execute one step

  Intentions are stacks of partially executed plans. The selection functions are overridable.
- Scientific concept modeled: Bratman's practical reasoning.
- Computational cost: about 10–100 µs per cycle in the JVM for modest plan libraries.
- What we can learn:
  - **Commitment and intention stacks.** Once committed, an agent keeps going unless a reconsideration trigger fires. This prevents utility-AI "flicker".
  - Plan context conditions are human-readable reasons.
- What we could integrate: the reasoning-cycle semantics (reimplement).
- What we should NOT integrate: the LGPL Java runtime or AgentSpeak as the authoring language for designers.
- Scalability: 1k in JVM; 100k would need our own compact implementation.

### SPADE-BDI
- URL: https://github.com/javipalanca/spade_bdi (the commonly cited `sfp932705/spade_bdi` was not verified)
- License: MIT
- Language: Python
- Activity/maintenance: ~15 stars, 105 commits.
- Architecture: AgentSpeak (ASL) interpreter on the SPADE XMPP multi-agent platform; beliefs set and read from Python; KQML-like tell/untell/achieve.
- Scientific concept modeled: BDI + agent messaging.
- Computational cost: Python + XMPP. Heavy per message.
- What we can learn: a lightweight Python BDI to prototype commitment rules.
- What we could integrate: prototype only.
- What we should NOT integrate: XMPP networking per agent.
- Scalability: 10–100.

### pomdp-py (POMCP / PO-UCT)
- URL: https://github.com/h2r/pomdp-py
- License: MIT
- Language: Python + Cython
- Activity/maintenance: ~281 stars, 372 commits; maintained.
- Architecture:
  - Interfaces for State, Action, Observation, TransitionModel, ObservationModel, RewardModel and PolicyModel.
  - The belief is a particle set.
  - POMCP: Monte-Carlo tree search over histories. UCB1 picks actions; rollouts use a default policy; particles reinvigorate on observation.
- Scientific concept modeled: online planning under partial observability.
- Computational cost: N simulations × depth. For example, 500 sims × depth 10 in Cython ≈ 10–50 ms.
- What we can learn:
  - Where **real uncertainty** matters (hunting, hiding, lying, searching for a lost child), MCTS gives principled risk-aware choices.
  - The search tree's visit counts and Q-values make a natural "options considered" trace.
- What we could integrate: the algorithm. Write a compact C++/Rust MCTS for a few high-stakes decisions.
- What we should NOT integrate: Python per agent at scale.
- Scalability: 10 (real time), 1k (rarely, budgeted), 100k no.

### Unity ML-Agents
- URL: https://github.com/Unity-Technologies/ml-agents
- License: Apache-2.0
- Language: C# + Python
- Activity/maintenance: ~19.7k stars; Release 23 (Aug 2025); maintained.
- Architecture: PPO/SAC/MA-POCA, self-play, BC/GAIL imitation, curriculum. Trained policies are exported to ONNX and run in-engine through Unity's Inference Engine.
- Scientific concept modeled: deep RL for embodied agents.
- Computational cost: inference with a small MLP (64–256 hidden) ≈ 5–50 µs per agent on CPU, and much less batched on GPU. Training is offline and costly.
- What we can learn:
  - Use RL for **low-level motor and animal behaviors** (flocking, predator–prey pursuit, foraging), where hand-tuning is hard and explanations matter less.
  - Batched inference makes 10k+ animals feasible.
- What we could integrate: offline training and ONNX export (Apache).
- What we should NOT integrate: RL for human social decisions. They are opaque, brittle under world changes, and hard to trace.
- Scalability: inference 100k batched; training is offline.

### PettingZoo
- URL: https://github.com/Farama-Foundation/PettingZoo
- License: MIT
- Language: Python
- Activity/maintenance: ~3.5k stars; maintained.
- Architecture: the AEC (Agent-Environment-Cycle, turn-based) API vs the Parallel API; versioned environments; SuperSuit wrappers.
- Scientific concept modeled: MARL environment standard.
- Computational cost: n/a (interface).
- What we can learn: expose a **headless PettingZoo-compatible wrapper** of our ecosystem to train animal policies offline and to benchmark AI tiers.
- What we could integrate: the API shape (MIT).
- What we should NOT integrate: n/a
- Scalability: training-time only.

---

## Part E — LLM + simulation hybrids

### Voyager (MineDojo / NVIDIA)
- URL: https://github.com/MineDojo/Voyager
- License: MIT
- Language: Python + JavaScript (Mineflayer)
- Activity/maintenance: ~7.2k stars; research artifact, no longer actively developed.
- Architecture:
  1. An automatic curriculum (GPT-4) proposes the next task from the current state and exploration progress.
  2. Iterative prompting writes **code** (JS skill functions), runs it in the environment, and reads execution errors and environment feedback.
  3. A self-verification critic confirms success.
  4. Successful programs go into a **skill library** indexed by an embedding of their description, and are retrieved for later tasks.
- Scientific concept modeled: lifelong learning through a growing library of executable skills.
- Computational cost: many GPT-4 calls per skill (minutes, dollars). The skills are then free to reuse.
- What we can learn:
  - **The LLM writes reusable artifacts once, and the engine reuses them forever.**
  - For us, the artifacts are *validated data*: new HTN methods, recipes and rituals. We do not want arbitrary code.
- What we could integrate: the pattern (curriculum → propose → execute in sandbox → verify → library).
- What we should NOT integrate: LLM-generated executable code inside the game. That is a security and determinism problem.
- Scalability: the library is shared by all agents, so cost is per *new skill*, not per agent.

### ReAct and Reflexion
- URL: https://github.com/ysymyth/ReAct ; https://github.com/noahshinn/reflexion
- License: MIT / MIT
- Language: Python
- Activity/maintenance: ~4.2k and ~3.3k stars; frozen paper code.
- Architecture:
  - ReAct interleaves "Thought / Action / Observation" in a single prompt loop.
  - Reflexion adds a self-reflection step after failure. The verbal lesson is stored in episodic memory and prepended to future attempts ("verbal reinforcement").
- Scientific concept modeled: reasoning traces plus acting; verbal credit assignment.
- Computational cost: one LLM call per step, plus one per reflection.
- What we can learn: Reflexion's **lesson memory** maps onto our symbolic memory. Store compact *rules learned* ("Wolves near river at dusk → avoid"), which the cheap utility layer can then read as considerations, not as prompts.
- What we could integrate: a reflection pass that outputs **structured lessons** (validated) for focus agents at night.
- What we should NOT integrate: free-form ReAct loops controlling bodies tick by tick.
- Scalability: 10 at most for live use.

### CoALA (Cognitive Architectures for Language Agents) — taxonomy
- URL: https://github.com/ysymyth/awesome-language-agents (paper arXiv 2309.02427)
- License: repo is a curated list (license not checked)
- Language: n/a
- Activity/maintenance: reference list (300+ citations).
- Architecture:
  - Memory types: working, episodic, semantic and procedural.
  - Action space: external (grounding) and internal (retrieval, reasoning, learning).
  - A decision cycle: **propose → evaluate → select** → execute.
- Scientific concept modeled: mapping production-system cognitive architectures onto LLM agents.
- Computational cost: n/a
- What we can learn: a vocabulary for our boundary.
  - The **LLM is one proposer** among others (utility, HTN, habits) in the *propose* stage.
  - Evaluation and selection stay symbolic.
  - "Learning" actions write to procedural or semantic memory only through validation.
- What we could integrate: the taxonomy for docs and trace field names.
- What we should NOT integrate: n/a
- Scalability: n/a

### AI Town (a16z-infra)
- URL: https://github.com/a16z-infra/ai-town
- License: MIT
- Language: TypeScript
- Activity/maintenance: ~10.6k stars; maintained.
- Architecture:
  - Convex backend (transactions, scheduled functions, vector search).
  - A **deterministic game engine loop** runs inputs and steps (movement, conversation state machine) in server transactions.
  - Agent "operations" (LLM calls: start or continue conversation, reflect, remember) run **asynchronously outside the tick**. They submit their results back as *inputs*, which the engine validates.
  - Memories are embedded and retrieved by vector search.
  - PixiJS front-end; Ollama local LLM by default.
- Scientific concept modeled: generative agents with a proper engine/LLM split.
- Computational cost: the engine is cheap. Each conversation turn is one LLM call (local: ~0.5–2 s).
- What we can learn: **the cleanest reference for async LLM → engine-validated input.**
  - The world never blocks on the LLM.
  - LLM results arrive as proposals that can be stale and rejected.
  - Conversations are an explicit state machine (invited → walking → talking → left).
- What we could integrate: the architecture pattern (MIT code, but the Convex stack is not a game engine).
- What we should NOT integrate: Convex/TS as the core sim, or memory-retrieval-per-tick.
- Scalability: 10–100 LLM agents.

### AgentSims
- URL: https://github.com/py499372727/AgentSims
- License: MIT
- Language: Python (Tornado, MySQL)
- Activity/maintenance: ~964 stars, 11 commits; dormant.
- Architecture: a sandbox town with buildings and equipment that have defined operations. Agents are LLM-driven with pluggable memory, planning and tool-use modules. Task-based evaluation by QA forms at intervals.
- Scientific concept modeled: environment-grounded evaluation of LLM agent abilities.
- Computational cost: LLM call(s) per agent action.
- What we can learn: **equipment-defined affordances**. Every object exposes a finite operation list. That is the right way to bound what an LLM may propose.
- What we could integrate: the affordance pattern.
- What we should NOT integrate: the stack.
- Scalability: 10s.

### Lyfe Agents (MIT / metaconscious group)
- URL: paper arXiv 2310.02172 (through search); code https://github.com/metaconsciousgroup/lyfe-agent-paper (verified to exist; the GitHub page shows it as "Gen-Agents", ~1 star).
- License: MIT (repo page)
- Language: Python (+ Unity environment)
- Activity/maintenance: 6 commits; research drop.
- Architecture (from abstract and summaries):
  1. A **hierarchical option–action framework**. The LLM picks a high-level *option* (e.g. "talk", "go to", "search memory"), then cheap lower-level steps execute until the option terminates. This sharply cuts LLM calls.
  2. **Asynchronous self-monitoring**: a background summary of recent events and goals keeps the agent consistent without a full context every call.
  3. **Summarize-and-Forget memory**: cluster recent memories, summarize them, and drop near-duplicates before long-term storage.
  - Reported ≈ $0.5 per agent per human-hour with GPT-3.5; 10–100× cheaper than Smallville-style agents.
- Scientific concept modeled: options framework (Sutton) applied to LLM agents.
- Computational cost: about one LLM call per option, i.e. tens of seconds to minutes of game time.
- What we can learn: **invoke the LLM only at option boundaries**, and dedupe memory aggressively. Both are directly applicable.
- What we could integrate: the pattern.
- What we should NOT integrate: the Unity/Python code (thin).
- Scalability: ~10–100 live agents per GPU.

### Affordable Generative Agents (AGA, Tencent)
- URL: https://github.com/AffordableGenerativeAgents/Affordable-Generative-Agents (TMLR 2024; arXiv 2402.02053)
- License: Apache-2.0
- Language: Python
- Activity/maintenance: ~58 stars, 7 commits.
- Architecture:
  - **Lifestyle Policy**: when an agent faces a plan or situation similar to one already solved by the LLM, it reuses the cached decomposition and action sequence (embedding similarity) instead of calling the LLM.
  - **Social Memory**: relationship state and compressed summaries of past dialogues replace raw dialogue history, which shrinks prompts.
  - Tested on Smallville (GA) and VirtualHome; large token savings reported.
- Scientific concept modeled: amortizing LLM reasoning into reusable policies.
- Computational cost: falls over time as the policy cache fills.
- What we can learn: **semantic caching of LLM plans** keyed by (situation embedding, archetype) is the single biggest cost lever. Relationship state belongs in the engine.
- What we could integrate: the algorithm pattern; the Apache code for reference.
- What we should NOT integrate: unvalidated cache reuse. A cached plan must re-pass engine validation in the new context.
- Scalability: enables 100s of LLM-backed agents.

### S3 — Social-network Simulation System (Gao et al., Tsinghua) + Rust replication
- URL: paper arXiv 2307.14984 (through search); independent replication https://github.com/akitenkrad/gao2023
- License: replication MIT (original code unverified)
- Language: replication in Rust (core) + Python (viz)
- Activity/maintenance: replication ~0 stars, 16 commits.
- Architecture:
  - Agents on a directed follow-graph.
  - Each round, the LLM updates **emotion** (calm/moderate/intense), **attitude** (±) and **behavior** (post/repost/inactive) from the messages perceived.
  - The replication separates a deterministic, seeded core (network, delivery, activation, metrics) from the LLM layer. The LLM layer is made **pseudo-deterministic through a `hash(prompt+model) → response` cache**, with temperature 0.
- Scientific concept modeled: information/attitude/emotion propagation.
- Computational cost: one LLM call per agent per round (cache hits are free).
- What we can learn: the **replay-by-cache** trick makes LLM-touched simulations reproducible. That is essential for save/load and for "rewind time" god powers.
- What we could integrate: the cache-for-determinism pattern (MIT replication).
- What we should NOT integrate: LLM-for-every-agent updates.
- Scalability: 1k–10k with caching and batching.

### OASIS (camel-ai)
- URL: https://github.com/camel-ai/oasis
- License: Apache-2.0
- Language: Python
- Activity/maintenance: ~5.2k stars; active (Aug 2026).
- Architecture:
  - An async environment server that emulates Twitter/Reddit-like platforms with a SQL-backed state.
  - A **recommender system** (interest-based and hot-score, embedding-based) decides what each agent sees.
  - 23 actions (follow, comment, repost, like/dislike, search, …).
  - A **time engine** activates agents per step with configurable probabilities, so not everyone acts each step.
  - ModelFactory supports many backends (OpenAI, vLLM-served Qwen, …).
  - Demonstrated up to 1M agents. The README cites about 335.6k input tokens for 100 agents per timestep, i.e. ~3.4k tokens per active agent per step.
- Scientific concept modeled: social media dynamics (herding, polarization, information spread).
- Computational cost: ~3k input tokens per active agent-step. At 1M scale this needs heavy GPU clusters and low activation rates.
- What we can learn:
  - **Activation probability** as an LOD knob: most agents are idle most steps.
  - A **recommender / attention model** as the perception filter.
  - Real numbers for token budgeting.
- What we could integrate: the patterns (Apache code usable for offline experiments).
- What we should NOT integrate: per-agent LLM at 100k+ in a real-time game.
- Scalability: 1M offline on clusters; a real-time game can afford roughly 10–100 LLM-active agents.

### AgentTorch (MIT Media Lab) — Large Population Models with LLM archetypes
- URL: https://github.com/AgentTorch/AgentTorch
- License: **AGPL-3.0. Flag: do not integrate code in a closed-source game.**
- Language: Python (PyTorch)
- Activity/maintenance: ~651 stars, 589 commits; active.
- Architecture:
  - Agents are **tensors** (population × properties) and update by vectorized substeps (observation → action → transition).
  - Simulation is **differentiable**, so it can be calibrated by gradient.
  - **LLM archetypes**: instead of querying an LLM per agent, query it once per *archetype* (a demographic/behavioral bucket such as age × occupation × region) for a behavior probability or decision rule, then apply it to millions of tensor agents with sampling.
  - Used for epidemiology (e.g. COVID labor-force decisions).
- Scientific concept modeled: LLM-informed agent-based models at population scale.
- Computational cost: GPU tensor ops ≈ ns per agent. LLM cost ∝ #archetypes × #decision contexts (hundreds of calls, not millions).
- What we can learn: **this is our T0/T1 LLM strategy.**
  - Query the LLM per (archetype × situation class) at low frequency.
  - Turn answers into numeric policy tables or utility-curve tweaks.
  - Apply them to every member of the cohort with per-agent noise.
- What we could integrate: the idea only (AGPL).
- What we should NOT integrate: any code.
- Scalability: millions.

### GenSim (NAACL 2025 demo)
- URL: https://github.com/TangJiakai/GenSim
- License: MIT
- Language: Python (built on AgentScope; vLLM inference)
- Activity/maintenance: ~88 stars; small.
- Architecture:
  - Single-agent module (profiles, memory, actions), multi-agent module (script mode vs agent mode), environment module.
  - Distributed agent servers reach up to 100k agents.
  - **Error-correction**: anomalous or invalid agent outputs are detected, and models are improved over rounds through SFT/PPO.
- Scientific concept modeled: large-scale LLM social simulation with self-correction.
- Computational cost: GPU-cluster-scale.
- What we can learn: an **"invalid proposal" dataset** collected from validation failures can fine-tune our small local model over time.
- What we could integrate: the pattern (log every rejected proposal with the reason, then fine-tune offline).
- What we should NOT integrate: the AgentScope distributed runtime.
- Scalability: 100k offline (not real time).

### YuLan-OneSim (RUC)
- URL: https://github.com/RUC-GSAI/YuLan-OneSim
- License: Apache-2.0
- Language: Python
- Activity/maintenance: ~245 stars; last notable update June 2025.
- Architecture:
  - Natural-language → scenario code generation, with 50+ default scenarios.
  - Distributed mode up to 100k agents.
  - Pluggable planning/memory strategies (e.g. "ShortLongStrategy").
  - vLLM local serving.
  - Exports event and training data.
  - Follows the **ODD protocol** (Overview, Design concepts, Details) for documenting ABMs.
- Scientific concept modeled: automated ABM construction.
- Computational cost: cluster-scale.
- What we can learn: **ODD** is a good template for documenting each of our subsystems so that emergent results are interpretable. Scenario-from-text is a possible "god command" feature (offline).
- What we could integrate: the ODD documentation practice.
- What we should NOT integrate: runtime.
- Scalability: 100k offline.

### mkturkcan/generative-agents (local, low-cost Smallville re-implementation)
- URL: https://github.com/mkturkcan/generative-agents
- License: MIT
- Language: Python (notebooks)
- Activity/maintenance: ~993 stars, 19 commits; dormant.
- Architecture: the generative-agents loop (observe → retrieve → plan → reflect) rebuilt for small local models (runs in < 8 GB VRAM), in a D&D-style town.
- Scientific concept modeled: generative agents on small models.
- Computational cost: local-GPU-bound; seconds per agent step.
- What we can learn: a reality check. Small models degrade planning coherence, which is why **we should not let small models plan freely**. Give them menus (constrained choice), not open text.
- What we could integrate: n/a beyond lessons.
- What we should NOT integrate: unconstrained small-model planning.
- Scalability: ~10.

### Constrained decoding and serving stack: Outlines, XGrammar, SGLang, llama.cpp GBNF
- URL: https://github.com/dottxt-ai/outlines ; https://github.com/mlc-ai/xgrammar ; https://github.com/sgl-project/sglang ; https://github.com/ggml-org/llama.cpp (grammars/README.md)
- License:
  - Outlines Apache-2.0
  - XGrammar Apache-2.0
  - SGLang Apache-2.0
  - llama.cpp MIT (known; not re-verified on page)
- Language:
  - Outlines: Python
  - XGrammar: C++/Python (bindings for JS and Swift; community Rust)
  - SGLang: Python
  - llama.cpp: C/C++
- Activity/maintenance:
  - Outlines ~15.9k stars
  - XGrammar ~1.9k stars (XGrammar-2 May 2026)
  - SGLang ~36.7k stars
  - llama.cpp very active
- Architecture:
  - Constrained decoding masks invalid tokens at each step, so output is guaranteed to match a JSON Schema, regex or CFG.
  - XGrammar claims near-zero overhead and is the default in vLLM; it is integrated in SGLang, TensorRT-LLM and MLC.
  - llama.cpp converts JSON Schema → GBNF (subset; no `not`, conditionals or uniqueItems).
  - SGLang adds **RadixAttention prefix caching**, so shared system/world prompts are computed once across many agents.
- Scientific concept modeled: n/a (infrastructure).
- Computational cost: grammar masking ≈ µs per token; prefix caching cuts prefill for shared context by large factors.
- What we can learn:
  - **The LLM's output space can be exactly the engine's action schema**: enum of legal action ids, enum of known entity ids, numeric ranges.
  - Invalid structure becomes impossible; only *semantic* validation remains.
  - Put stable content (world rules, persona) first in prompts to maximize prefix-cache hits.
- What we could integrate: llama.cpp (MIT) embedded for a shippable local model with GBNF; XGrammar (Apache) if we run a server.
- What we should NOT integrate: Python serving stacks inside the shipped game client (fine for a dedicated server or dev).
- Scalability: one consumer GPU running a 3–8B model at batch 16–64 gives roughly tens to hundreds of short structured calls per second.

---

## Part F — Cross-cutting lessons

1. **Every serious emotion model is appraisal over goals and beliefs.** GAMYGDALA, EMA, FLAME, PsychSim and CPM differ only in which appraisal variables they keep and where they get them. Our engine already has goals (needs → goals), beliefs (memory/perception) and causal attribution (event logs). So appraisal is mostly *bookkeeping*, and cheap.
2. **Mood is a slow integrator; personality sets the baseline** (ALMA, WASABI, Sentipolis). Keep three time constants: emotion (minutes to hours of game time), mood (hours to days), temperament (years).
3. **Emotion's job is to modulate decisions** (PSI modulators, MAMID, EMA coping), not just to be displayed:
   - curve weights
   - risk multiplier
   - planning depth
   - attention radius
   - commitment threshold
   - action-tendency bonuses
4. **Needs are personality-generated and cue-activated, not a strict pyramid.** Use the Kenrick motives, the DF pattern, PSI cognitive needs, and the HRL drive formula for trade-offs.
5. **Decisions: utility picks, a planner chains, a BT/state machine executes, BDI-style commitment stabilizes.** MCTS is reserved for high-stakes uncertain choices. RL is used for animal and motor policies.
6. **LLM: proposer, not decider.** Successful hybrids keep the world deterministic and treat LLM output as async, validated, cacheable proposals:
   - AI Town: async proposals
   - AGA: cached policies
   - S3 replication: hash cache
   - AgentTorch: archetypes
   - Lyfe: options
   - Voyager: verified library

---

# Recommended design for our game

## R1. Layered affect

```
Temperament (Big Five + HEXACO-H, 0..1 per factor; ~12 facets)   τ ≈ years   (drift, §R2)
   │ ALMA mapping
   ▼
Baseline mood (PAD)                                                fixed per personality, shifted by allostatic load
   │
Mood (PAD ∈ [-1,1]^3)                                              τ ≈ 6–48 game hours
   ▲ pull/push from emotion center (ALMA), contagion (ASCRIBE) adds to mood only
   │
Emotions (discrete, intensity 0..1, with target/cause)            τ ≈ 10 min – 1 day (per type, per personality)
   ▲ appraisal of events vs goals/standards/attachments (GAMYGDALA core + EMA variables + CPM order)
Events (perceived, remembered, or anticipated)
```

**Appraisal record (per event × affected agent; short-circuit in CPM order):**
1. **Relevance:** does the event touch any active goal, need, attachment or standard? If not, stop (most events stop here, so this is cheap).
2. **Implication:**
   - `desirability = Σ_goals congruence × goal.utility` (GAMYGDALA)
   - likelihood and Δlikelihood
   - expectedness, i.e. prediction error vs. the agent's model
   - causal agent and intentionality
   - urgency
3. **Coping potential:** controllability, i.e. "do I have an action with positive expected utility that changes it?" This reuses the decision layer's last option scores, at zero extra cost. Also changeability.
4. **Normative significance:** match against the agent's **values** (DF-style) and cultural norms. Gives self/other blame and praise.

**Emotion derivation table (examples; intensity = |desirability| × f(likelihood) × personality gain):**

| Emotion | Rule (appraisal pattern) | Personality modulators |
|---|---|---|
| joy / distress | desirable / undesirable, likelihood = 1, self-relevant | E raises joy gain; N raises distress gain |
| hope / fear | prospective, likelihood < 1 | N and low H-Emotionality bravery modulate fear |
| relief / disappointment | earlier fear disconfirmed / hope disconfirmed | — |
| anger | undesirable + other agent blamed + controllability high | low A, low H-Honesty raise anger; high A lowers it |
| sadness | undesirable + low controllability + low changeability | N |
| grief | loss of an **attachment** target (kin/partner/pet). Long-lived; re-triggered by memory cues; yearning component raises "seek reminders" | attachment strength × N |
| loneliness | relatedness/affiliation need deficit persisting > threshold, with low perceived availability of partners | E (gregariousness) raises need weight |
| guilt | own action violated own standard and **harmed another** | C, H-Honesty, A raise it |
| shame | own action/attribute violated standard, **public** and global-self attribution | N, low self-esteem |
| embarrassment | minor public norm violation, low harm, short-lived | E lowers it |
| pride | own action matched/exceeded standard; praise from others boosts it | E, low H (vanity) |
| envy | other's desirable outcome in a status-relevant domain + self lacks it + upward social comparison | low H-Humility, N |
| jealousy | rival threatens an attachment/relationship (third-party triad) | N, low A |
| love / affection | sustained positive liking × attachment; mostly a *relationship variable*, with episodic warmth emotions | A, E |
| gratitude | desirable outcome caused intentionally by other | A, H |
| disgust | contamination cue or moral violation (separate "moral disgust" tag) | O (low), C |
| surprise | large prediction error regardless of valence. Very short; boosts memory encoding | — |
| anxiety | **mood-level** fear with no specific object: certainty need deficit + low competence expectation (PSI) | N |

- **Decay:** `I(t+Δ) = I(t)·exp(−Δ/τ_type·k_personality)`. Grief and loneliness use a *floor* that only drops when the underlying cause is resolved, which also makes rumination possible. Cathexis-style cross-inhibition matrix (anger ↓ fear; joy ↓ sadness).
- **Mood update per tick:** ALMA pull/push toward the intensity-weighted PAD center of active emotions, plus an ASCRIBE contagion term from nearby visible agents, plus a slow return to baseline.
- **Memory coupling:** each episodic memory stores (valence, arousal, emotions[]). Encoding strength rises with arousal and surprise. Retrieval is biased toward mood-congruent memories (Sentipolis, mood-congruent memory). Recalling a memory can re-elicit a weaker version of its emotion. That is how "recalled memory 'daughter nearly died last winter'" enters the decision.
- **Conditioning** (FLAME): `assoc[entity] += α·(felt valence − assoc)` gives fears and fondness for places, people and species.

**Emotion → behavior coupling (the whole point):**
- **Consideration multipliers:**
  - fear multiplies perceived risk (×1 + 1.5·fear)
  - anger adds an approach/aggression tendency bonus toward the blamed agent
  - sadness lowers the energy for effortful options
  - guilt adds a bonus for repair actions toward the victim
  - love/attachment raises the weight of the target's needs in one's own drive (empathic homeostasis)
- **PSI modulators from arousal:** resolution (planning depth, perception radius) and selection threshold (commitment hysteresis).
- **EMA coping** as *mental actions* in the option set: replan, seek support, reappraise, deny, blame-shift, resign. Choosing "deny" changes a belief's weight. That is visible in traces ("Coping: denial — discounted report of wolf attack").

## R2. Personality model and change

- **Storage:** Big Five + HEXACO Honesty-Humility as 6 factor floats, plus ~12 game-relevant facets (bravery, anger propensity, gregariousness, curiosity, impulsivity, nurturance, dominance, trust, …) each derived as `factor + facet_noise`. Also ~10 values in −1..1 (family, tradition, fairness, power, knowledge, nature, piety, independence, honesty, martial). Animals get a reduced set: boldness, sociability, aggression, exploration (animal-personality "behavioral syndromes").
- **Trait → parameter mapping (design proposal, tune in playtests):**

| Parameter | Formula sketch |
|---|---|
| risk tolerance | `0.5 + 0.3·(E_facet_excitement) − 0.35·N + 0.15·bravery − 0.2·C` |
| forgiveness (rate anger/grudge decays; trust repair) | `base·(1 + 0.8·A + 0.3·H − 0.4·N)` |
| curiosity (weight of certainty/competence needs, exploration bonus) | `0.2 + 0.6·O + 0.2·(1−N)` |
| honesty (prob. of truthful communication when lying pays) | `sigmoid(4·(H − 0.5) + 2·(C−0.5) − gain·greed)` |
| cooperation (weight on others' utility in social options) | `0.3·A + 0.4·H + 0.2·value_fairness + kinship term` |
| parenting investment | `nurturance + 0.3·A + 0.3·C + value_family` × hormonal/life-stage factor |
| leadership tendency | `0.4·E_dominance + 0.3·C + 0.2·(1−N) + status motive` |
| emotion gains/decay | N raises negative gains and slows their decay; E raises positive gains; A lowers anger; C raises guilt |
| commitment (selection threshold) | `0.1 + 0.2·C − 0.1·impulsivity` |

- **Lifespan change (maturity principle + events):**
  `trait_{m+1} = trait_m + p(age)·[ κ·(target_age(trait) − trait_m) + Σ event_impulses ] + ε`
  - `p(age)` is plasticity: high in youth, falling after ~30, small in old age.
  - `target_age` encodes the mean-level trends: C and A up, N down, social dominance up in the 20s–40s.
  - Event impulses: parenthood → C+, A+; bereavement → N+ temporarily; trauma → N+, trust−; success as leader → dominance+.
  - Run monthly in game time.
  - Log every change with its cause, so the inspector can answer "why did she become so bitter?"

## R3. Motivation: how goals emerge

1. **Needs** (per agent; set and weights generated from personality, life stage and culture, as in DF):
   - physiological: food, water, sleep, warmth, health/pain, safety
   - social (Kenrick / PSI): affiliation, status, mate, kin-care
   - cognitive (PSI / SDT): certainty, competence, autonomy
   - plus value-derived needs (piety, craft, wander)
2. **Drive** = Keramati–Gutkin `D(H)` over physiological needs (convex: the worst need dominates) plus weighted social and cognitive urges, plus **empathic coupling**: `w_attach(target)·D_target` for kin and loved ones. This is what makes "feed daughter first" come out naturally when the attachment weight is high.
3. **Wanting vs liking:** cue-triggered `κ` multipliers (Berridge) on specific options (smell of food, addiction cues). Liking updates hedonic memory after consumption.
4. **Goal generators** (periodic + event-triggered). Each produces candidate goals `{type, target, source_needs, source_memories, source_values, utility, deadline}` from:
   - need urge × urgency × expected success (PSI)
   - **opportunities** from perception and beliefs (affordance query)
   - **memories**: recalled episodes raise the salience of related goals ("last winter the daughter nearly starved" → raise utility of `stockpile_food` and `protect_daughter`)
   - **values/standards** (avenge, uphold tradition)
   - **emotions** (anger → `confront(X)`, guilt → `make_amends(Y)`, grief → `mourn`, `visit_grave`)
   - **social obligations** (promises, roles)
5. **Adoption and commitment** (BDI/Jason semantics):
   - Goals become *intentions* only if `utility > current_intention.utility + selection_threshold`.
   - Reconsideration fires on significant appraisal (|desirability| > θ), plan failure, or a periodic review.
   - This removes utility-AI dithering and gives traces a clear "committed at t because …".
6. **Allostatic load:** a slow accumulator from chronic stress. It shifts mood baseline and N-like gains and harms health. It is the long-term cost of a hard life.

## R4. Hybrid decision stack per LOD tier

Assume a 4-core (game) / 8-core (sim server) budget with AI ≈ 30–40% of CPU, and 1 game-minute = 1 real second at normal speed. Budgets are *per real second* and scale with time acceleration (the LOD scheduler degrades tiers when sped up).

| Tier | Who | Decision machinery | Decision frequency | Cost per decision | Trace kept |
|---|---|---|---|---|---|
| **T4 Inspected** (1–10) | agents the god is watching | T3 + LLM proposer (dialogue, plans, reflection) + full trace + counterfactuals | every 1–5 s real | ≤ 5 ms CPU + async LLM | full, permanent while inspected |
| **T3 Focus** (10–100) | near camera / story-relevant | full appraisal (EMA vars, depth-1 ToM), utility over ~30–60 options, HTN plans with plan repair, MCTS/POMCP for flagged high-stakes uncertain choices (200–1000 sims), BT execution | every 2–10 s real; MCTS rarely | utility 5–20 µs; HTN 50–500 µs; MCTS 2–10 ms | full ring buffer (last ~50 decisions) |
| **T2 Town** (100–10k) | active settlement | appraisal limited to relevance-filtered events (GAMYGDALA-level), utility over ~15–30 options, GOAP/HTN *plan templates* cached per archetype, BTs | every 10–60 s real (staggered) | 2–10 µs; plan lookup ~1 µs | compact trace: top-3 options + top-3 factors per decision, last ~10 |
| **T1 Background** (10k–100k) | distant villages, wild animals | needs + mood scalars; utility over ~6–10 coarse activities (work, eat, sleep, socialize, travel, fight, flee); animals use small ONNX policies or utility | every game-hour (batched SoA) | 0.2–1 µs | "last decision + dominant reason" (8 bytes) |
| **T0 Aggregate** (100k–1M) | off-screen populations | cohort/archetype model (AgentTorch-style): per-cohort rates from needs, mood distribution and LLM-archetype policy tables; individuals sampled on promotion | every game-day per cohort | ~10–50 ns per individual (vectorized) | cohort statistics only; individual history synthesized on promotion |

**CPU budget sketch (per real second at 1×):**
- 10 agents: everything at T3/T4 → ≈ 10 × (200 µs utility+appraisal + amortized planning + occasional 5 ms MCTS) ≈ 5–20 ms. Trivial.
- 100 agents: T3 → ≈ 100 × 50 µs/s + MCTS budget capped at 20 ms/s → ≈ 25 ms/s.
- 1k agents: 100 T3 + 900 T2 → ≈ 25 + 900 × (10 µs / 30 s staggered) ≈ 25–30 ms/s.
- 10k agents: 100 T3 + 9.9k T2/T1 mix → ≈ 30 + ~5 ms (decisions) + appraisal/event fan-out ~10 ms → ≈ 50 ms/s on one core, or parallel across 4.
- 100k agents: 100 T3 + 5k T2 + 95k T1 (hourly batches = 95k decisions per 60 s ≈ 1.6k/s × 1 µs ≈ 2 ms/s) → ≈ 50–80 ms/s total. Feasible.
- 1M: add T0 cohorts (GPU or SIMD) at ≈ 10–50 ms per game-day tick.

**Promotion/demotion:**
- On promotion (e.g. the god zooms in), synthesize individual state from cohort distributions and a *plausible recent history*. That can use the LLM offline-style for T4 narrative, but symbolic for state.
- On demotion, compress the trace into a "life summary" record.

**Stack per agent (T2–T4):**
`Perception (attention radius ∝ resolution) → Appraisal → Goal generation → Utility (goal & option scoring; IAUS curves) → Commitment filter (BDI threshold) → Planner (HTN template / GOAP / plan repair; MCTS for flagged decisions) → BT executor (async actions) → Effects → Events → Appraisal…`

**Explainability rules:**
- Every consideration has a stable id and a human label.
- Every plan step stores its parent method/goal (HTN tree).
- Every belief used carries a source (perceived/told-by/remembered + memory id).
- Randomness is drawn from a per-agent seeded stream, so decisions replay exactly.

## R5. Decision trace format (example)

```jsonc
{
  "trace_id": "d-000917-a4412",
  "agent": {"id": "a4412", "name": "Maren", "tier": "T4", "age": 34, "role": "mother"},
  "tick": 1048213, "game_time": "Y12 Winter D18 07:40",
  "rng": {"stream": "a4412/decide", "draw": 3391},
  "context": {
    "location": "hut-17", "nearby": [{"id": "a5120", "name": "Ilsa", "rel": "daughter", "dist_m": 2}],
    "perceived_food": [{"item": "barley_bread", "qty": 1, "owner": "a4412"}]
  },
  "state": {
    "needs": {"hunger": 0.86, "fatigue": 0.40, "warmth": 0.55, "affiliation": 0.20},
    "drive": {"self": 0.71, "empathic": {"a5120": {"w_attach": 0.92, "hunger": 0.78, "contrib": 0.66}}},
    "mood_pad": [-0.32, 0.41, -0.18], "mood_label": "anxious",
    "emotions": [{"type": "fear", "i": 0.55, "target": "a5120", "cause": "mem:m-2231"},
                 {"type": "love", "i": 0.80, "target": "a5120"}],
    "modulators": {"arousal": 0.62, "resolution": 0.48, "selection_threshold": 0.18}
  },
  "personality_used": {"A": 0.71, "N": 0.64, "nurturance": 0.83, "value_family": 0.9, "risk_tolerance": 0.31},
  "memories_recalled": [
    {"id": "m-2231", "summary": "Ilsa nearly died of hunger last winter", "valence": -0.9,
     "arousal": 0.8, "relevance": 0.94, "effect": "+fear(Ilsa), +utility(feed_child) x1.25"}
  ],
  "goals_active": [
    {"id": "g-keep-ilsa-fed", "source": ["empathic_drive", "mem:m-2231", "value_family"], "utility": 0.88, "committed_since": "D17 18:00"},
    {"id": "g-eat", "source": ["need:hunger"], "utility": 0.71}
  ],
  "options": [
    {"action": "give_food(a5120, barley_bread)", "score": 0.81,
     "factors": [["daughter_hunger(curve:quad)", 0.78], ["attachment", 0.92], ["memory_boost m-2231", 1.25],
                 ["own_hunger_cost", -0.12], ["nurturance", 0.83]]},
    {"action": "eat(barley_bread)", "score": 0.62,
     "factors": [["own_hunger(curve:logistic)", 0.86], ["guilt_anticipated", -0.21]]},
    {"action": "search_food(forest)", "score": 0.54, "risk": "high",
     "factors": [["expected_food", 0.6], ["risk(wolves, belief b-77 told-by a3001)", -0.38], ["fear_risk_mult", 1.55]]},
    {"action": "coping:seek_support(a3001)", "score": 0.47}
  ],
  "commitment_check": {"incumbent": "g-keep-ilsa-fed", "threshold": 0.18, "switched": false},
  "decision": {"action": "give_food(a5120, barley_bread)", "method": "argmax", "margin": 0.19},
  "plan": {"htn": "feed_family > hand_over_food", "steps": ["walk_to(a5120)", "hand(barley_bread)"]},
  "counterfactuals": [
    "Would eat instead if own hunger >= 0.95 or memory m-2231 were not recalled",
    "Would search forest if risk_tolerance >= 0.55"
  ],
  "predicted_emotions": [{"type": "pride", "i": 0.3}, {"type": "relief", "i": 0.4, "target": "a5120"}],
  "llm": null
}
```

- **Counterfactuals are cheap.** For the top 2 options, solve for the single factor change that flips the ranking. That is one linear solve per factor, because scores are products of curves.
- **UI renders a natural-language line from the trace, deterministically:** "Hunger severe; daughter nearby; remembered 'Ilsa nearly died last winter' → gave her the bread (81 vs eat 62, search 54/high risk)."
- The LLM may *optionally* rephrase it for T4, but it is never the source of truth.

## R6. The LLM boundary

**May propose (structured, grammar-constrained, ids from enums only):**
1. **Dialogue content** for T4/T3 conversations: utterance text plus *structured speech acts* (inform(fact_id), request(action), promise(action, deadline), threaten, insult, comfort, lie(claim)). Only the speech acts change world state. Text is cosmetic.
2. **Daily plan sketch** for T4 agents: an ordered list of goal ids / HTN task ids with time windows, chosen from the agent's current candidate goals and affordances.
3. **Reflections → structured lessons/beliefs** (Reflexion-style): `{belief: "wolves_near(river, dusk)", confidence, evidence: [mem ids]}`.
4. **Appraisal hints for novel events** that have no authored rule: congruence estimates for the agent's goals, with a numeric range clamp. Used rarely; results are cached by event-type signature.
5. **New know-how** (Voyager-style, offline or at low frequency): a new HTN method / recipe / ritual as data. It needs simulation sandbox verification before it enters the shared library.
6. **Archetype policies** (AgentTorch-style) for T0/T1 cohorts: probabilities or curve-weight adjustments per (archetype × situation class).
7. **Narration** of traces for the god's inspector (read-only).

**Validation pipeline (engine-owned):**
1. Schema (guaranteed by XGrammar/GBNF).
2. Referential integrity: ids exist and are perceived or known by this agent; there is no knowledge the agent cannot have (epistemic check against its belief store).
3. Preconditions: action legal now, or plan steps decomposable by HTN.
4. Personality/value plausibility: proposal utility under the agent's own model must be within δ of the best option, or above a floor. Otherwise reject as out-of-character. This keeps the LLM from overriding the psyche.
5. Physics/economy clamps: quantities, distances, prices.
6. Rate limits per agent.
7. On reject: log the reason (feeds the GenSim-style fine-tuning dataset) and fall back to the symbolic choice.
8. LLM results are **async proposals** (AI Town pattern) with a validity window. Stale proposals are revalidated or discarded. The world never waits.

**When invoked (triggers, not ticks):**
- A conversation starts between ≥ 1 T3/T4 agent and another agent.
- An option boundary for a T4 agent (Lyfe pattern), e.g. a morning plan or the end of a major activity.
- The night reflection for T4 agents (and for T3 agents with a high-arousal day).
- A novel event type with no appraisal rule (then cached).
- The god asks "why?" (narration) or speaks to an agent.
- Offline or low-priority: archetype policy refresh, know-how generation.

**Caching / determinism:**
- `hash(model, prompt_template_version, canonicalized structured context) → response` (S3 replication pattern). Replays are reproducible and rewinds are free.
- Semantic plan cache keyed by (archetype, situation embedding) (AGA Lifestyle Policy). Cached plans are always revalidated.
- Prefix-cache-friendly prompts: world rules + persona first (SGLang RadixAttention / llama.cpp KV reuse).
- Batch requests per sim step.

**Budget per in-game day** (assume a 3–8B local model, ~600 input / ~120 output tokens per structured call; 1 game-day ≈ 24 real min at 1×):

| Use | Calls per game-day |
|---|---|
| T4 agents (≤ 10): plan 1 + reflection 1 + dialogue turns ~10 + narration on demand ~5 | ≈ 17 per agent → ~170 |
| T3 agents (≤ 100): dialogue only when with T4, or 1 short reflection on high-arousal days | ≈ 1–2 per agent → ~150 |
| Novel-event appraisal (cached) | ~20 |
| Archetype policy refresh (spread over days) | ~10 |
| **Total** | **~350 calls/game-day** ≈ 250k tokens |

This is ≈ 15 calls per real minute: comfortable for a mid-range GPU or a cheap API. Hard caps:
- 1,000 calls/day
- ≤ 2 concurrent in flight per agent
- automatic degradation (fewer reflections, template dialogue) when the queue latency exceeds 3 s

With no LLM present, the game must still run identically except for dialogue flavor, which uses a template grammar.

## R7. Concrete integration list

| Component | Take | How |
|---|---|---|
| GAMYGDALA appraisal | algorithm | reimplement (MIT allows porting) |
| ALMA mood layer + Big5→PAD | algorithm + coefficients | reimplement |
| WASABI dynamics, boredom | equations | reimplement (code is LGPL/GPL) |
| EMA appraisal vars + coping table | algorithm | reimplement |
| CPM SEC order | checklist | reimplement symbolic |
| ASCRIBE contagion | formula | reimplement |
| PSI/MicroPsi needs, modulators, hysteresis | algorithm | reimplement |
| Keramati–Gutkin drive | formula | reimplement |
| Berridge κ | formula | reimplement |
| IAUS scoring + curve editor UX | algorithm, tooling ideas | reimplement; Curvature for UX inspiration |
| Utility AI in ECS | pattern / code | big-brain (Apache) or our own |
| GOAP | code if Unity/Rust | crashkonijn/goap (Apache) or dogoap (MIT) |
| BT executor | code | BehaviorTree.CPP (MIT) |
| HTN | algorithm (+ offline validation) | GTPyhop-style port; pandaPI (BSD) for offline HDDL checks |
| BDI commitment | semantics | reimplement (Jason is LGPL) |
| MCTS/POMCP | algorithm | reimplement (pomdp-py MIT for prototyping) |
| Animal RL | tooling | ML-Agents/PettingZoo offline, ONNX runtime |
| LLM constrained decoding | code | llama.cpp GBNF (MIT) / XGrammar (Apache) |
| LLM archetypes | idea only | AgentTorch is **AGPL**: do not use code |

---

# Open questions / risks

1. **Over-parameterization.** Personality × 6 factors × 12 facets × 10 values × 20 emotions × curves is a huge tuning space. Risk: emergent behavior looks random. Mitigations:
   - Automated "personality unit tests": scripted scenarios with expected *distributions* of choices per archetype.
   - Sensitivity analysis on a PettingZoo-style headless harness.
2. **Emotion flicker vs inertia.** Tuning τ for emotions and mood across time-acceleration (1× vs 100×) is non-trivial. We need time-scale-invariant integration (exact exponential decay, not per-tick constants).
3. **Appraisal fan-out at scale.** A village-wide event (a death) touches thousands of agents' goals and attachments. We need relevance indexing (who has an attachment or goal referencing entity X) and per-tier caps.
4. **Explainability vs fidelity.** MCTS and RL decisions are harder to explain:
   - MCTS: report the top branches with visit counts and the dominant risk.
   - RL (animals): report only "policy: forage (confidence 0.8)".
   - Is that acceptable to players?
5. **LLM character drift and epistemic leaks.** The LLM may "know" things the agent can't, or speak out of character. The validator catches *actions* but not subtle dialogue content. We may need a cheap classifier over the speech-act structure, plus restricting facts in dialogue to belief ids.
6. **Determinism with an LLM in the loop.** Model or driver updates break caches. Version the model and prompt templates in save files; on mismatch, fall back to logged responses (they are stored in the save) rather than re-querying.
7. **Licensing.** Several attractive sources are copyleft (WASABI LGPL/GPL, Jason LGPL, AgentTorch AGPL) or have unclear licenses (MicroPsi2 license type unverified, Curvature custom, HRRL none, SHOP3 MPL unverified). Policy: algorithms from papers only for these. Do a legal review before any code import.
8. **Scientific validity of complex emotions.** Envy, jealousy, shame and grief rules are design approximations of contested theories. Present them as "the world's psychology", not as clinical truth. Grief and trauma loops need care (player experience, sensitivity).
9. **Promotion artifacts.** Agents synthesized from T0 cohorts need plausible memories and relationships. If the LLM writes backstory, it must be constrained to facts that are consistent with recorded world history.
10. **Personality change balance.** If event impulses are too strong, the population converges (everyone bitter after a famine). If they are too weak, nothing changes. The maturity targets also need a cultural and species override (animals, other races).
11. **Unverified items to re-check when network allows:**
    - MicroPsi2 license text
    - SHOP3 license
    - py_trees license
    - Curvature license
    - Sentipolis and CPM-MultiAgent code availability
    - exact ALMA coefficients and sign conventions in the original PDF
    - Keramati–Gutkin parameter values (n, m) from eLife
