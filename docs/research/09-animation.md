# 09 — Animation: procedural, data-driven and AI-generated animation for a living world

Research track 9 of 9. Date of research: 2026-10-01. No game code here — research only.

**Verification legend.** Licenses, star counts and dates were checked against the GitHub repo page,
project page or license text via web fetch on 2026-10-01 unless marked *unverified*. Star counts are
approximate (rounded as GitHub displays them). The GitHub API was not available from this sandbox, so
"last commit" dates are often inferred from READMEs/news entries — treat them as "at least this recent".

**The single most important licensing finding:** almost every *code* repo in data-driven / AI motion
research is MIT or Apache, but the *data and body models* those repos depend on are mostly
**non-commercial** (LAFAN1 = CC BY-NC-ND 4.0, AMASS = non-commercial research licence, SMPL/SMPL-X =
non-commercial unless licensed via Meshcapade, AI4Animation = CC BY-NC 4.0 including its mocap). Any
pretrained text-to-motion checkpoint trained on HumanML3D/AMASS inherits that taint. For a commercial
game, the safe 3D data sources are: our own mocap/hand-keyed clips, CMU mocap (free incl. in commercial
products, no direct resale), Mixamo (royalty-free in games, cannot redistribute raw files, cannot be
used to train ML models), and purchased/commissioned packs.

---

## Part A — Procedural animation

### Spore creature animation (Hecker et al., "Real-time Motion Retargeting to Highly Varied User-Created Morphologies")
- URL: https://www.chrishecker.com/Real-time_Motion_Retargeting_to_Highly_Varied_User-Created_Morphologies (paper PDF linked there; SIGGRAPH 2008 record: https://history.siggraph.org/learning/real-time-motion-retargeting-to-highly-varied-user-created-morphologies-by-hecker-raabe-enslow-deweese-maynard-et-al/)
- License (commercial game use OK? restrictions?): Paper = ideas only; techniques are free to re-implement (no known patent check done — *unverified*). No code released.
- Type / language / engine: SIGGRAPH 2008 technical paper (Maxis/EA), custom engine, in-house tool "Spasm".
- Activity/maintenance / date: 2008; still the canonical reference for "animate arbitrary bodies".
- How it works (concrete): Animators author motion once on a reference creature but the data is stored in a *morphology-independent* form: targets are expressed relative to semantic body parts (e.g. "this grasper", "those feet", "the spine end") and in normalised coordinates relative to limb lengths/body size, plus style channels. At runtime, for each concrete creature, the generalised data is evaluated to give pose goals per limb, which feed a custom IK solver (per-limb, with priorities); locomotion gait is procedurally generated from leg count/positions with keyed style overlays.
- Runtime cost / scalability: Per-creature IK every frame; fine for tens to low-hundreds of on-screen creatures in 2008 hardware; not a 10k-crowd solution by itself, but the *output* can be baked.
- What it would do in our game: The core idea for animating genetically varied bodies: author "intent" animations (eat, threaten, nurse, carry) once per *body plan class*, evaluate them on each individual's proportions (height, limb length, child proportions, quadruped vs biped).
- What we can learn: Store animation as relative goals ("hand to mouth", "front feet at shoulder width × 1.1") instead of joint angles; retargeting becomes trivial across genetic variants.
- Integrate? yes (as a design pattern) — implement a small "semantic pose goal + IK" layer on top of whatever skeleton/sprite rig we choose.
- Risks: Building a general IK + gait system is significant engineering; Spore's results look "rubbery" without polish; patents not checked.

### Rain World procedural creatures (GDC 2016, "The Rain World Animation Process", Jakobsson & Therrien)
- URL: https://www.youtube.com/watch?v=-iXwvoFhPuU ; writeup https://www.gamedeveloper.com/art/video-animating-i-rain-world-i-and-its-many-squishy-stretchy-creatures
- License: Talk only; game is proprietary. Technique free to reuse.
- Type / language / engine: GDC 2016 Animation Bootcamp talk; game built in Unity (C#) with 2D rendering.
- Activity/maintenance / date: 2016; Rain World DLC ("Watcher") still shipped in 2025, same tech.
- How it works (concrete): Each creature is a handful of physics points (verlet-style) connected at fixed distances (body chunks), with limbs solved by simple IK toward ground-grab points chosen by the AI's pathing. Sprites ("paper doll" parts) are drawn stretched/rotated between the points every frame. Behaviour AI decides *where* to put limbs; physics makes it squishy; nothing is keyframed.
- Runtime cost / scalability: ~10–30 points per creature, trivially cheap on CPU; thousands of point-creatures are feasible, though the game only shows dozens.
- What it would do in our game: Ideal model for 2D/2.5D animals (lizards, snakes, fish, insects, worms) and even stylised humans; the sim says "move to X, grab Y", the creature body physically follows.
- What we can learn: Animation emerges from (AI goal) + (cheap constraint physics) + (sprite skinning between points); creature variation = change point count/lengths/sprite set.
- Integrate? yes — the most direct template for our 2D/2.5D creatures.
- Risks: Can look janky ("wobbly") for humans; needs careful tuning per species; physically-driven bodies can tunnel or tangle.

### Overgrowth procedural animation (GDC 2014, "An Indie Approach to Procedural Animation", David Rosen)
- URL: https://www.gamedeveloper.com/design/video-an-indie-approach-to-procedural-animation ; GDC Vault (Animation Bootcamp 2014)
- License: Talk. Note: Overgrowth's engine source was later released by Wolfire under Apache-2.0 (github.com/WolfireGames/overgrowth — *unverified this session*), so code reading may be possible.
- Type / language / engine: Talk; custom C++ engine (Phoenix).
- Activity/maintenance / date: 2014.
- How it works (concrete): Only ~13 key poses for all character motion. Locomotion = interpolation between a few poses driven by a *phase* derived from distance travelled (not time), with speed-dependent blending; plus procedural layers: spring-damped body lean on acceleration, head look-at, IK foot planting on slopes, active-ragdoll blending on hits, procedural breathing.
- Runtime cost: Very cheap (few poses + IK).
- What it would do in our game: Proves we can make expressive, responsive humans/animals from a tiny keyframe set — huge win for a project with many species and body types.
- What we can learn: Drive cycles from distance travelled (no foot sliding even with genetic leg length variation); layer springs for lean/secondary motion; keep keyframes as "poses" not "clips".
- Integrate? yes (pattern) for 3D or 2.5D; also applicable to 2D skeletal.
- Risks: Requires an animator-minded programmer; style is "loose".

### Exanima (Bare Mettle) — physics-driven characters
- URL: https://www.baremettle.com/ (game page, *not fetched*)
- License: Proprietary game; technique reference only.
- Type: Custom engine; fully physics-simulated characters with motor-driven joints.
- Activity/date: Early access since 2015, still in development (*unverified current status*).
- How it works: Every character is an active ragdoll; animation provides target poses, joint motors try to reach them under gravity, balance and collisions; combat/stumbling emerges.
- Runtime cost: High per character (full articulated physics); dozens on screen.
- What it would do: Reference only for "followed person up close" mode: falls, injuries, fights look unscripted.
- What we can learn: Active ragdoll is appropriate only at the highest LOD tier.
- Integrate? maybe (concept) — only for 1–20 close-up characters.
- Risks: Tuning hell, instability, high CPU.

### argonautcode/animal-proc-anim (chain-based procedural fish/snake/lizard)
- URL: https://github.com/argonautcode/animal-proc-anim
- License: MIT — commercial OK.
- Type / language: Processing (Java), 2D.
- Activity / date: ~711 stars; accompanies a popular 2024 YouTube explainer (*date approx*).
- How it works: A spine is a chain of points with a distance constraint (each point dragged behind the previous one) plus an angle constraint limiting bend; body outline is drawn by offsetting circles of per-segment radius along the chain; legs use a simple two-bone IK toward foot targets that step when they lag too far behind the body.
- Runtime cost: Tens of points per animal → thousands of animals per frame on CPU easily; trivial on GPU.
- What it would do: Template for 2D fish, snakes, lizards, worms, insects; body width profile and segment count can be genes.
- What we can learn: "Follow-the-leader chain + angle limits + stepping IK legs" is the minimum viable procedural animal.
- Integrate? yes — re-implement (it is ~a few hundred lines) in our renderer.
- Risks: Purely kinematic (no balance), only top-down 2D.

### IK solvers: FABRIK, CCD, analytic two-bone
- URL: FABRIK paper (Aristidou & Lasenby 2011, Graphical Models) — http://www.andreasaristidou.com/FABRIK.html (*not fetched*); Caliko FABRIK library (Java, MIT) https://github.com/FedUni/caliko (*license unverified this session*); engines: Godot 4 `SkeletonModifier3D` / `TwoBoneIK3D`, Unity Animation Rigging package, Bevy community crates (see Part H).
- License: Algorithms are free; library licenses vary (prefer MIT/Apache).
- Type: Algorithms.
- How it works: Two-bone IK is closed-form (law of cosines + pole vector) — the right tool for legs/arms of bipeds and quadrupeds. CCD rotates each joint in turn toward the target from tip to root — good for tails/necks/tentacles. FABRIK alternates backward and forward passes moving joint *positions* along the chain preserving bone lengths — fast, stable, easy to add constraints; maps naturally to verlet/point creatures.
- Runtime cost: Two-bone: a few dozen flops per limb. FABRIK/CCD: ~5–20 iterations × bones. 10k characters × 4 limbs is fine with two-bone in a SIMD/compute pass.
- What it would do: Foot planting on terrain, hands reaching tools/children/food, head look-at at social targets.
- What we can learn: Use analytic two-bone for all limbs, FABRIK/CCD only for long chains; run IK only at high LOD tiers.
- Integrate? yes.
- Risks: Joint-limit handling; knee popping; must clamp reach.

### Procedural gait generation for quadrupeds/hexapods (phase-offset stepping)
- URL: references — Spore paper above; "procedural spider/hexapod" tutorials are many (*not individually verified*); research baseline: Starke et al. MANN (Part B) for learned quadruped gaits.
- License: Technique.
- How it works: Each leg has a phase offset in a global gait cycle (walk: LH, LF, RH, RF at 0, 0.25, 0.5, 0.75; trot: diagonal pairs in phase; gallop: asymmetric; hexapod tripod gait). Gait chosen by speed (Froude-number-based transitions, cf. Alexander's dynamic similarity), step length scales with leg length, body bobs/rolls follow foot contacts. Each foot moves along an arc to a predicted landing point using two-bone IK.
- Runtime cost: Cheap; parameters only.
- What it would do: One system animates wolves, deer, rabbits (bound gait = in-phase hind legs), insects, spiders; leg length / body mass genes change cadence automatically.
- What we can learn: Use the Froude number (v²/(g·leg length)) to choose gait so small and large animals look correctly scaled.
- Integrate? yes.
- Risks: Rabbits/hopping and birds need special cases; turning in place and slopes need extra logic.

### Secondary motion: springs, verlet chains, jiggle
- URL: Godot 4 `SpringBoneSimulator3D` (added in Godot 4.4, 2025 — *verify per version*), VRM "spring bone" spec, Unity "Dynamic Bone"-style assets (commercial).
- License: Godot core MIT.
- How it works: Damped springs on extra bones (hair, ears, tails, clothes, bellies, carried items) integrated each frame; colliders keep them out of the body.
- Runtime cost: Low per bone; cut at mid/far LOD.
- What it would do: Ear flops for rabbits, tail wag for dogs, hair/clothes for close-up humans, a baby bouncing in a sling.
- Integrate? yes, close LOD only.
- Risks: Jitter at low framerate; disable when time-scaled (god speeds up time).

