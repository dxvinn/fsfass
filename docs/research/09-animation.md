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
- License: Talk. Overgrowth's engine source is public at https://github.com/WolfireGames/overgrowth under Apache-2.0 (verified; ~2.9k stars); game *assets* are NOT included and remain proprietary. So the animation code itself can legally be studied and reused with attribution.
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
- URL: FABRIK paper (Aristidou & Lasenby 2011, Graphical Models) — http://www.andreasaristidou.com/FABRIK.html (*not fetched*); Caliko FABRIK library https://github.com/FedUni/caliko (Java, MIT, ~180 stars — verified); engines: Godot 4.4 added `SkeletonModifier3D`/`LookAtModifier3D`, Godot 4.6 added `IKModifier3D` with TwoBoneIK3D, FABRIK3D, CCDIK3D, ChainIK3D, SplineIK3D, JacobianIK3D (https://godotengine.org/article/inverse-kinematics-returns-to-godot-4-6/); Unity Animation Rigging package, Bevy community crates (see Part H).
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
- URL: Godot `SpringBoneSimulator3D` (added in Godot 4.5; docs https://docs.godotengine.org/en/stable/classes/class_springbonesimulator3d.html), VRM "spring bone" spec, Unity "Dynamic Bone"-style assets (commercial).
- License: Godot core MIT.
- How it works: Damped springs on extra bones (hair, ears, tails, clothes, bellies, carried items) integrated each frame; colliders keep them out of the body.
- Runtime cost: Low per bone; cut at mid/far LOD.
- What it would do: Ear flops for rabbits, tail wag for dogs, hair/clothes for close-up humans, a baby bouncing in a sling.
- Integrate? yes, close LOD only.
- Risks: Jitter at low framerate; disable when time-scaled (god speeds up time).

### Active ragdolls / physics-based characters (general)
- URL: reference implementations — MimicKit (Part B) for learned controllers; Unity "PuppetMaster" (commercial Asset Store, *not verified*); Euphoria/NaturalMotion (proprietary middleware, now in-house at Zynga/Take-Two — *unverified*); Godot `PhysicalBoneSimulator3D` (core, MIT).
- License: engine-core options MIT; middleware proprietary.
- How it works: Ragdoll bodies with joint motors (PD controllers) tracking an animated pose; blend weight between "pure animation" and "pure physics" changes on hits, falls and death. Balance controllers (keep centre-of-mass over support polygon) add stumbles.
- Runtime cost: 10–20 rigid bodies + constraints per character; ~10–100 simultaneous in a typical physics engine.
- What it would do: Deaths, falls from cliffs, being hit by a falling tree, drowning, knockdowns during fights — only for characters near the camera.
- Integrate? yes for death/impact moments at LOD0; at all other tiers play a canned "fall" clip or sprite.
- Risks: Must never feed back into sim truth (sim decides "X died, fell north"; the ragdoll just shows it).

### Age / injury / body-type variation driven by sim state (research + patents)
- URL: "Procedural Locomotion of Multi-Legged Characters in Dynamic Environments" (Abdul Karim et al., CAVW 2013) https://liris.cnrs.fr/Documents/Liris-5511.pdf ; elderly-gait forward-dynamics study (Gait & Posture 2021) https://www.sciencedirect.com/science/article/abs/pii/S096663622100014X ; **US patent 11129551 "Method for providing age-related gait motion transformation based on biomechanical observations"** (https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/11129551).
- License: Papers are ideas; **the patent is a real IP risk** if we implement a near-identical "transform young gait to age-N gait from biomechanical tables" method — get a freedom-to-operate opinion or use a clearly different approach (e.g. procedural layers + physical parameters).
- How it works (our version): Body descriptors from genetics/age/health modulate a procedural layer stack:
  - **Proportions**: per-bone scale (head/limb/torso ratios by age curve; infants ~4 heads tall, adults ~7–7.5); stride length ∝ leg length; cadence from speed and leg length.
  - **Elderly**: forward trunk flexion (+10–20°), reduced stride and arm swing, wider base of support, slower cadence, small vertical bob, occasional pause; cane/staff attachment shifts weight via additive lean.
  - **Child/toddler**: higher cadence, wide stance, arms raised for balance (toddler), more bounce; run/skip cycles.
  - **Limp (left/right leg, severity 0–1)**: shorten stance time on injured leg (asymmetric phase: injured leg stance ≈ 35–45% vs 60% normal), reduced knee flexion, pelvis drop and trunk lean over the good leg, head bob on good-leg contact; severe = hop/crutch.
  - **Fatigue/hunger/sickness**: slower playback, slumped spine additive, heavier footfalls (sound only).
  - **Build/mass**: wider stance, slower arm swing, larger vertical COM motion for heavy builds.
  - **Pregnancy / carrying**: COM shift backwards with lumbar extension; carried items use hand IK + upper-body mask.
- Runtime cost: Additive pose offsets + phase warping — cheap enough for LOD0–2; baked into variant clip bins for LOD3.
- Integrate? yes — core of how variation is shown.
- Risks: Overly mechanical look if all offsets are linear; patent above.

---

## Part B — Data-driven 3D character animation

### Motion Matching (Clavet, GDC 2016 "Motion Matching and the Road to Next-Gen Animation")
- URL: https://www.gdcvault.com/play/1023280/Motion-Matching-and-The-Road
- License: Technique; free to implement. Unreal Engine 5.4+ ships Motion Matching ("Pose Search" plugin) under the UE EULA (5% royalty) — *not re-verified*.
- Type: Talk (Ubisoft Montreal; tech used in For Honor).
- How it works: Every frame (or every N frames), build a query feature vector (future trajectory positions/directions sampled at e.g. 0.33/0.66/1.0 s, current foot positions/velocities, hip velocity) and brute-force or KD-tree/AABB search an entire mocap database for the best matching frame; jump there with inertialization blending. No state machines or blend trees.
- Runtime cost: Search ~O(database frames × features), accelerated with bounding-volume hierarchies; ~0.05–0.5 ms per character per search; fine for tens–hundreds; not for 10k.
- What it would do: Gorgeous locomotion for the followed person up close, if we have a big enough licensed mocap set.
- What we can learn: "Match the future trajectory" is how to make agents' paths look natural; inertialization blending is reusable even without MM.
- Integrate? maybe — only at LOD0 in a 3D art direction; needs lots of clean licensed mocap.
- Risks: Data hungry (several minutes of unstructured locomotion per style, per body type); doesn't natively handle genetic proportion variation (retarget first).

### orangeduck/Motion-Matching (Daniel Holden) + "Learned Motion Matching" (SIGGRAPH 2020)
- URL: https://github.com/orangeduck/Motion-Matching ; paper page https://theorangeduck.com/page/learned-motion-matching
- License: Code MIT (verified). **Bundled data is LAFAN1 = CC BY-NC-ND 4.0 → not usable commercially.**
- Type / language: C++ + raylib; WebAssembly demo; Python (PyTorch) training scripts.
- Activity: ~919 stars; reference implementation, low churn.
- How it works: Plain MM implementation (feature DB, AABB-accelerated search, inertialization, foot-locking IK, adjustment for trajectory) plus Learned MM: three small MLPs (decompressor, stepper, projector) replace the database, cutting memory from hundreds of MB to a few MB while producing the same output.
- Runtime cost: MM search as above; Learned MM is a few tiny MLP evaluations per character — memory-scalable, could run for hundreds of characters on CPU.
- What it would do: The best educational base for our own C++/Rust motion matching at LOD0–1.
- What we can learn: Inertialization, foot locking, simulation-bone/character-entity separation (the sim's capsule leads, the pose follows) — exactly matches "sim is truth, animation follows".
- Integrate? yes for the code ideas (MIT); **replace data** with commercially licensed mocap.
- Risks: Data licence trap (LAFAN1). Holden's site articles are an excellent learning resource but are prose, not licensed code.

### Ubisoft LaForge Animation Dataset (LAFAN1)
- URL: https://github.com/ubisoft/ubisoft-laforge-animation-dataset
- License: **CC BY-NC-ND 4.0 — no commercial use, no derivatives.** (verified)
- Type: ~4.6 h BVH mocap, 5 subjects (*hour count from memory, unverified*); ~1.6k stars.
- What it would do: Prototype/R&D only.
- Integrate? no (for shipping). Fine for internal prototyping only if legal agrees; any model trained on it should be treated as tainted.
- Risks: Licence.

### Motion Matching plugins: JLPM22/MotionMatching (Unity) and GuilhermeGSousa/godot-motion-matching (Godot 4)
- URL: https://github.com/JLPM22/MotionMatching ; https://github.com/GuilhermeGSousa/godot-motion-matching (also Remi123/MotionMatching, V-Sekai/motion_matching for Godot)
- License: Both MIT (JLPM22 via LICENSE file; Godot plugin per search listing — *star count ~227, licence from search snippet, not repo page*).
- Type: Unity C# package (BVH import, feature DB, inertialization, controllers); Godot 4.4 GDExtension integrated into AnimationTree.
- Activity: JLPM22 based on a master's thesis, author also published Environment-aware Motion Matching (SIGGRAPH Asia 2025, https://github.com/UPC-ViRVIG/Environment-aware-Motion-Matching).
- How it works: as Holden's article; the Godot one exposes MM as an AnimationTree node so it can be layered with IK/blend.
- Runtime cost: tens of characters.
- What it would do: Engine-ready MM for LOD0 humans if we go Unity or Godot 3D.
- Integrate? maybe (LOD0 humans in 3D only).
- Risks: Small maintainer teams; check data used in samples.

### AI4Animation (Sebastian Starke) — PFNN, MANN (quadrupeds), Local Motion Phases, DeepPhase, Codebook Matching
- URL: https://github.com/sebastianstarke/AI4Animation
- License: **CC BY-NC 4.0 — README says research/education only, not for commercial use or redistribution; the mocap (incl. the dog data used for MANN) is also CC BY-NC 4.0.** (verified) ~8.9k stars; includes a 2026 Python remake "AI4AnimationPy".
- Type: Unity (C#) + PyTorch.
- Activity: Papers SIGGRAPH 2017 (PFNN, Holden/Komura/Saito), 2018 (Mode-Adaptive NN for quadrupeds), 2020 (Local Motion Phases), 2022 (DeepPhase), 2024 (Categorical Codebook Matching).
- How it works: PFNN: network weights are a cyclic function of a gait phase variable; MANN: a gating network blends expert weights so a dog controller handles walk/pace/trot/canter/jump/sit without labelled phases; DeepPhase: a periodic autoencoder learns multi-dimensional phase manifolds from unstructured motion that can drive MM or NN controllers and align motions across morphologies (used by WalkTheDog, SIGGRAPH 2024, human↔dog).
- Runtime cost: Small MLPs, ~0.1–1 ms per character on CPU; hundreds feasible; GPU-batched could reach thousands.
- What it would do: Proof that neural controllers give natural quadruped gaits; the ideas (phase variable, mixture of experts) can be re-implemented and trained on *our own* licensed data.
- What we can learn: Phase is the key latent for locomotion; for animals, mixture-of-experts gating handles gait transitions well.
- Integrate? no for code/data (NC); yes for ideas re-implemented from papers.
- Risks: Training data for animals is scarce and mostly non-commercial; re-implementation is a research project.

### MimicKit (Xue Bin Peng) — DeepMimic, AMP, ASE, ADD, SMP; successor to xbpeng/DeepMimic
- URL: https://github.com/xbpeng/MimicKit (old: https://github.com/xbpeng/DeepMimic — MIT, deprecated in favour of MimicKit)
- License: Apache-2.0 (verified) ~2.4k stars, 2025 paper references; DeepMimic MIT ~3.1k stars.
- Type: Python, GPU physics (Isaac Gym/Isaac Lab style); RL training.
- How it works: Train a physics-simulated character policy with RL to imitate reference clips (DeepMimic) or to match the *style distribution* of a motion dataset via an adversarial discriminator (AMP), or learn a reusable latent skill space (ASE) that high-level tasks can steer.
- Runtime cost: Inference cheap (MLP), but needs a full rigid-body sim per character; training costs GPU-days.
- What it would do: Physically plausible injured/elderly/child gaits via changed body parameters (mass, joint torque limits = weakness, stiffened knee = limp) — the policy adapts. Mostly an offline tool to *generate* clips.
- What we can learn: Injury & age can be modelled as physical constraints, and a controller produces the right compensation automatically.
- Integrate? maybe (offline clip-generation research; not runtime for crowds).
- Risks: RL expertise, GPU cost; reference data licensing still applies.

### NVlabs/ProtoMotions (v3, 2025)
- URL: https://github.com/NVlabs/ProtoMotions
- License: Apache-2.0 (verified) ~2.4k stars.
- Type: Python; GPU physics with backends Newton, MuJoCo, IsaacGym, IsaacLab, Genesis.
- How it works: Framework for training physically simulated humanoids on large mocap sets (e.g. AMASS in ~12 h on 4×A100), includes MaskedMimic (one generalist controller that fills in motion from partial constraints like "head target + root path") and PyRoki-based retargeting.
- Runtime cost: As MimicKit — physics per character.
- What it would do: Offline generation of physically-valid variants (limping, carrying load) for our clip library; optional LOD0 physics character.
- Integrate? maybe (offline).
- Risks: Pretrained checkpoints trained on AMASS → non-commercial taint; must retrain on licensed data.

### Blend trees / state machines (baseline, every engine)
- URL: Unity Mecanim, Godot AnimationTree (BlendSpace1D/2D, StateMachine), Bevy `AnimationGraph` (Bevy 0.15+), ozz-animation (C++, MIT).
- License: engine licences; ozz MIT.
- How it works: Parametric blending of authored clips by speed/direction, with layered masks (upper body carry, lower body walk) and additive layers (lean, breathing, limp offset).
- Runtime cost: Cheap; scales to hundreds with CPU skinning, thousands with GPU skinning.
- What it would do: The workhorse for LOD0–1 in any 3D/2.5D direction; additive layers are the cleanest place to express "elderly hunch", "limp left", "sad posture".
- Integrate? yes (baseline).
- Risks: Combinatorial explosion if every state gets its own clip — counter with additive/procedural layers.

### Retargeting (Godot RetargetModifier3D, Unity Humanoid avatar, Blender Rokoko/Auto-Rig retargeting)
- URL: Godot 4.4 RetargetModifier3D (see Godot IK article above); Unity Humanoid rig docs.
- License: engine core.
- How it works: Map clips between skeletons via a canonical humanoid bone set and rest-pose alignment; scale root motion by leg length.
- What it would do: One clip library drives every genetically different human (height, limb ratios, child proportions). Root speed scaled by leg-length ratio avoids foot sliding.
- Integrate? yes.
- Risks: Extreme proportions (toddlers: big head, short legs) break hand contacts — add IK fix-ups.

### Open motion datasets & licences (summary)
- **CMU Graphics Lab Mocap** (mocap.cs.cmu.edu): "free for all uses"; may be included in commercially sold products but cannot be resold directly even converted; attribution requested. ~2,600 clips, noisy. → **usable commercially** (verified via secondary sources; official site blocked from sandbox).
- **Mixamo** (Adobe): royalty-free for personal/commercial/non-profit projects incl. games; may not redistribute raw files as asset packs; **may not be used to train ML models** (Adobe FAQ https://helpx.adobe.com/creative-cloud/faq/mixamo-faq.html). → usable as in-game clips; not as AI training data.
- **LAFAN1**: CC BY-NC-ND 4.0 → no.
- **AMASS** (MPI): non-commercial scientific research/education/artistic only; commercial licence via Meshcapade (ps-license@tue.mpg.de). → no unless licensed.
- **SMPL / SMPL-X body models**: non-commercial model licence (https://smpl.is.tue.mpg.de/modellicense.html), commercial via Meshcapade. → anything outputting SMPL params is suspect.
- **HumanML3D** (https://github.com/EricGuo5513/HumanML3D, MIT for scripts, ~1.5k stars): text annotations over AMASS+HumanAct12 motions; repo cannot redistribute AMASS → inherits AMASS non-commercial restriction.
- **Bones SEED** (bones.studio, 142k annotated motions, SOMA/G1 format): academic or "qualifying startup" only; commercial licence on request (licensing@bones.studio).
- **AI4Animation data**: CC BY-NC 4.0 → no.
- **Quadruped data**: DogML (T2QRM, SCUT), Quad-Imaginarium, QuadFM (2026), AiM3D/Kirin (2025, reconstructed from in-the-wild video) — licences mostly research-oriented, *not individually verified*; treat as non-commercial until confirmed.

---

## Part C — AI-generated motion (text-to-motion, in-betweening)

**Key distinction:** code licences are usually permissive, but **checkpoints trained on HumanML3D/AMASS/SMPL are not commercially clean**. 2026 changed this: NVIDIA's Kimodo and ARDY are trained on commercially licensed mocap (Bones Rigplay) and ship weights under the NVIDIA Open Model License, which permits commercial use. Tencent HY-Motion 1.0 is open-weights but its community licence excludes the EU, UK and South Korea.

### MDM — Motion Diffusion Model (Tevet et al., ICLR 2023)
- URL: https://github.com/GuyTevet/motion-diffusion-model
- License: Code MIT; depends on SMPL/SMPL-X, CLIP, HumanML3D → checkpoints not commercially clean. ~4.1k stars; last news Feb 2025.
- Type: Python/PyTorch.
- How it works: Transformer denoiser predicts the clean motion (not the noise) over ~1000 diffusion steps conditioned on CLIP text embedding; geometric losses (foot contact, velocity) improve physicality; supports in-betweening and body-part editing by masking.
- Runtime cost: Seconds to tens of seconds per clip on a GPU → offline only.
- What it would do: Offline prototype-generation of clips like "an old man sits down slowly".
- Integrate? no for shipped content (data taint); maybe as an internal previs tool.
- Risks: Licence; foot skating; SMPL skeleton needs retargeting.

### T2M-GPT (CVPR 2023)
- URL: https://github.com/Mael-zys/T2M-GPT
- License: Apache-2.0 code (verified, ~779 stars); weights trained on HumanML3D/KIT → same taint.
- How it works: VQ-VAE tokenizes motion; GPT autoregressively predicts motion tokens from CLIP text features.
- Runtime cost: Fast-ish (sub-second to seconds per clip on GPU).
- Integrate? no (weights); architecture ideas yes.
- Risks: As above.

### MoMask (CVPR 2024)
- URL: https://github.com/EricGuo5513/momask-codes
- License: MIT code (verified ~1.3k stars); depends on SMPL/HumanML3D.
- How it works: Residual VQ tokenization; a masked transformer predicts base tokens bidirectionally then a residual transformer fills finer layers — high quality, few iterations; supports temporal inpainting.
- Integrate? no (weights) / architecture reference.
- Risks: Licence.

### MotionGPT (NeurIPS 2023)
- URL: https://github.com/OpenMotionLab/MotionGPT
- License: MIT code (verified ~2.0k stars); SMPL/dataset dependencies.
- How it works: Treat motion tokens as a foreign language for a T5-style LM; one model does text→motion, motion→text, prediction, in-between.
- What we can learn: Motion→text captioning could label our own clip library automatically.
- Integrate? no (weights).

### MotionLCM / MotionLCM-V2 (ECCV 2024; V2 Dec 2024)
- URL: https://github.com/Dai-Wenxun/MotionLCM
- License: **Custom "MotionLCM LICENSE" — prohibits commercial use** (verified). ~469 stars.
- How it works: Latent consistency distillation of a motion latent diffusion model → 1–4 step generation, with ControlNet-style spatial control (e.g. pelvis trajectory) — "real-time controllable".
- Runtime cost: Tens of ms per clip on GPU → near-runtime.
- Integrate? no.
- What we can learn: Consistency distillation makes diffusion motion fast enough to consider runtime use.

### NVIDIA Kimodo (March 2026, v1.1 April 2026)
- URL: https://github.com/nv-tlabs/kimodo ; weights e.g. https://huggingface.co/nvidia/Kimodo-SOMA-RP-v1
- License: Code Apache-2.0; weights **NVIDIA Open Model License — commercial use and derivative models permitted** (verified via repo + HF listing). Trained on ~700 h of commercially licensed optical mocap ("Bones Rigplay 1"); a variant trained on Bones SEED exists (SEED data itself is not commercially open). ~3.7k stars.
- Type: Python/PyTorch kinematic motion diffusion (two-stage transformer denoiser, ~282M params per secondary source); outputs SOMA 77-joint skeleton, Unitree G1, or SMPL-X.
- How it works: Text prompt (+ optional constraints: keyframes, root paths) → full-body kinematic motion; ~17 GB VRAM (<3 GB with CPU text encoding).
- Runtime cost: Seconds per clip on one GPU → offline/batch.
- What it would do: **Best current candidate for an offline AI pipeline that generates our human clip library**: "elderly woman walks with a cane", "child skipping", "man carrying a heavy log on shoulder", "mother rocking baby", then retarget SOMA → our rig, clean up, bake.
- What we can learn: Use the SOMA (non-SMPL) output for a cleaner licence chain; avoid the SMPL-X output option unless we have an SMPL-X commercial licence.
- Integrate? yes (offline pipeline), pending legal review of the NVIDIA Open Model License text.
- Risks: Humans only; quality variable on rare actions; NVIDIA OML has attribution/AI-ethics clauses to read carefully.

### NVIDIA ARDY (SIGGRAPH 2026, released July 2026)
- URL: https://github.com/nv-tlabs/ardy ; project https://research.nvidia.com/labs/sil/projects/ardy/
- License: Code Apache-2.0; weights NVIDIA Open Model License (verified). ~946 stars. Trained mainly on Bones Rigplay 1.
- Type: Autoregressive diffusion, hybrid representation (explicit root features + latent body embedding); skeletons Core, G1, SOMA (coming).
- How it works: Streams motion in real time while accepting changing text prompts and constraints (waypoints, keyframes, sparse joint targets); TensorRT-accelerated; tested on RTX 4090.
- Runtime cost: Real time for *one or a few* characters on a high-end GPU — not for crowds, and not appropriate as a player-hardware dependency.
- What it would do: (a) offline: generate long, varied, transition-rich sequences for clip mining / motion-matching databases; (b) speculative: a "director's camera" mode where the followed hero's motion is generated live from sim events.
- Integrate? maybe — offline yes; runtime only as an optional high-end feature.
- Risks: GPU requirement; non-determinism; must remain a pure view layer.

### Tencent HY-Motion 1.0 (Dec 30, 2025)
- URL: https://github.com/Tencent-Hunyuan/HY-Motion-1.0 ; paper https://arxiv.org/html/2512.23464
- License: **Tencent HY-Motion 1.0 Community License — territory excludes EU, UK and South Korea** (per GitHub issue #49 discussion; licence text not directly fetched). ~2.6k stars.
- Type: 1.0B-parameter DiT flow-matching model (Lite 0.46B); trained on 3,000 h + 400 h curated data; 200+ motion categories; ≥24–26 GB VRAM.
- Integrate? no for a globally sold game unless legal clears the territory clause (EU sales would be outside licence).
- Risks: Territory clause, likely MAU clause typical of Hunyuan licences (*unverified for HY-Motion*).

### Quadruped / animal motion generation (research state, 2024–2026)
- URL: T2QRM/DogML https://github.com/SCUT-BIP-Lab/T2QRM ; Kirin / AiM3D https://kirin-ani.github.io/ (2025–26, motion reconstructed from in-the-wild video across 23 quadruped categories); QuadFM (2026, real-dog mocap); OmniMotionGPT (CVPR 2024, animal motion from limited data); WalkTheDog (SIGGRAPH 2024) https://github.com/PeizhuoLi/walk-the-dog-unity (no licence file found → *all rights reserved by default*).
- License: mostly research / unclear — **treat as non-commercial until proven**.
- How it works: Either text→quadruped motion (robot-dog skeletons) or video→3D animal motion reconstruction to build datasets.
- What it would do: Not ready to ship. Animals in our game should be **procedural** (gait generator + IK, Part A) with a few hand-keyed accents (howl, sniff, groom).
- Integrate? no (watch-list).
- Risks: Immature, skeleton mismatch, licences.

---

## Part D — 2D animation (skeletal, paper-doll, procedural pixel art, AI sprites, portraits)

### Spine (Esoteric Software) — commercial 2D skeletal animation
- URL: https://en.esotericsoftware.com/spine-runtimes ; runtimes https://github.com/EsotericSoftware/spine-runtimes ; editor licence https://en.esotericsoftware.com/spine-editor-license
- License: **Commercial.** Runtimes' source is public but may only be integrated into products if you held a valid Spine Editor licence at integration time; distribution continues after expiry. Organisations with >US$500k revenue/funding need Spine Enterprise. Runtime licence text must ship with the product. (verified via official pages/blog)
- Type: Editor (Java) + runtimes for C/C++, C#, Unity, Unreal, Godot (spine-godot), TS/WebGL, PixiJS (pixi-spine/spine-pixi), Three.js, Phaser, etc.
- Activity: Very active, industry standard.
- How it works: Bones + mesh deformation + weights + IK constraints + path constraints + skins (swap attachments per skin); runtime can set bone transforms each frame, so procedural control (look-at, lean, limp offsets) is easy; skin combining at runtime supports paper-doll outfits.
- Runtime cost: CPU skinning per character; hundreds on screen fine; thousands need baking to sprite sheets or GPU instancing.
- What it would do: High-quality 2D/2.5D humans and animals with genetic variation via bone scale (height, build) and skin combinations (hair, clothes, wounds).
- Integrate? maybe — best-in-class tooling for 2D; cost is modest (one-off seats) but adds a licence dependency.
- Risks: Licence ties; revenue threshold.

### DragonBones (runtime) / LoongBones
- URL: https://github.com/DragonBones/DragonBonesJS
- License: MIT runtime (verified), ~851 stars. Editor historically free (Egret); maintainers now point to "LoongBones" for authoring (*status unverified*).
- Type: TS/JS runtimes for PixiJS, Phaser, Egret, Cocos; C++/C# runtimes exist in sibling repos.
- How it works: Spine-like bones, meshes, FFD; JSON format.
- Integrate? maybe (budget OSS alternative for web/PixiJS); tooling ecosystem is weaker and partly Chinese-language.
- Risks: Long-term maintenance uncertain.

### Godot Skeleton2D / Unity 2D Animation (engine-native 2D skeletal)
- URL: Godot docs (Skeleton2D, Bone2D, Polygon2D skinning, SkeletonModification2D: FABRIK/CCD/LookAt/TwoBoneIK/Jiggle); Unity 2D Animation package (Sprite Skin, Sprite Library/Resolver for swappable parts, 2D IK).
- License: Godot MIT; Unity package under Unity Companion License (usable with Unity only).
- How it works: Bone hierarchy deforming sprite meshes; Unity's Sprite Library/Category/Label system swaps parts per character (ideal for genes: "nose_3", "ears_wolf"); Godot 2D modifiers provide procedural IK and jiggle.
- Runtime cost: CPU skinning; hundreds per frame; bake for crowds.
- What it would do: OSS path for 2D skeletal with procedural control without Spine fees.
- Integrate? yes if we choose Godot/Unity 2D.
- Risks: Godot 2D modification stack has historically been less polished than 3D (*varies by version*).

### Universal LPC Spritesheet Character Generator (Liberated Pixel Cup)
- URL: https://github.com/LiberatedPixelCup/Universal-LPC-Spritesheet-Character-Generator (old fork sanderfrenken/… now redirects)
- License: Code GPL-3.0; **art is per-asset mixed: CC0, CC-BY, OGA-BY, CC-BY-SA (3.0/4.0), GPL-3.0**; attribution mandatory for non-CC0; tool exports a credits CSV per image. ~1.8k stars. (verified)
- Type: Web tool (JS) + layered PNG sprite sheets (64×64 frames), 4 directions.
- How it works: Layered paper-doll: body base (male/female adult; LPC Expanded adds child and elderly — incomplete coverage), then head, hair, clothes, weapons layers, each a full sprite sheet aligned to the same frame grid; animations: walk, spellcast, slash, thrust, shoot, hurt, plus bow, climb, run, jump, sit, etc. in expanded sets.
- Runtime cost: Pre-composite per individual into one atlas (or composite in shader) → extremely cheap; thousands of sprites trivial.
- What it would do: Instant prototype art for top-down humans with huge combinatorial variety.
- Integrate? maybe — for **prototype only**, or for ship if we filter to CC0/CC-BY/OGA-BY layers. **CC-BY-SA layers**: derivative art must be shared alike (our composited sprites become CC-BY-SA — acceptable for art files but contaminates any art combined with them, and DRM/"effective technological measures" clauses in CC-BY-SA 3.0 are debated). **GPL-3.0 art layers**: avoid in a closed game.
- Risks: Licence hygiene per layer; generic look; 64-px RPG style may not fit.

### Procedural pixel-art / sprite animation (rotsprite, palette swaps, sub-pixel squash)
- URL: references — rotsprite algorithm (Xenowhirl, public description), "Dead Cells 3D→2D pipeline" (Motion Twin, GDC/Game Developer article 2018 "Art Design Deep Dive: Using a 3D pipeline for 2D animation in Dead Cells" — *not fetched*).
- License: Technique.
- How it works: (a) Render low-poly 3D models to low-res pixel sprites without anti-aliasing with toon shading (Dead Cells approach) → all animation done in 3D, output looks hand-pixelled; (b) procedural modifiers on sprites: squash-stretch, bob, palette swap for genes (skin/fur/eye colour), overlay layers for wounds/age (grey hair, wrinkles).
- Runtime cost: Baked → negligible.
- What it would do: Lets us keep a 3D skeletal + procedural pipeline (genetic proportions, limp) while shipping a 2D pixel look: render each *archetype × body-type bin × action × direction* to an atlas offline, or render live at low res for the followed person.
- Integrate? yes (strong option to bridge art directions).
- Risks: Atlas memory explosion with combinatorics → limit bins, composite layers at runtime.

### AI sprite-sheet generation: PixelLab (commercial SaaS) and similar
- URL: https://www.pixellab.ai/docs/options/character
- License: Commercial service; terms (per review sites, *not read first-hand*) permit commercial use of outputs and forbid using outputs to train other models; user responsible for third-party rights.
- How it works: Text/image → pixel character; skeleton-based posing and text-driven animation; 4/8-direction rotation incl. isometric.
- What it would do: Speed up concepting and sprite variants (species, clothing) — then cleaned by an artist.
- Integrate? maybe (offline tool).
- Risks: Consistency across frames; vendor lock-in; ToS changes; copyright status of AI outputs (limited protectability in the US for purely AI output).

### AnimateDiff
- URL: https://github.com/guoyww/AnimateDiff
- License: Apache-2.0 code (verified, ~12.3k stars, v3 Dec 2023). Base model weights carry their own licences (SD1.5 = CreativeML OpenRAIL-M; SDXL = OpenRAIL++).
- How it works: A motion module inserted into a frozen text-to-image UNet adds temporal attention; any SD1.5 fine-tune (e.g. a pixel-art LoRA) can then generate short loops.
- What it would do: Idle/ambient loops (fire, banners, fur ruffle) and concept animation; poor at consistent sprite sheets.
- Integrate? maybe (offline, artist-in-the-loop).
- Risks: Temporal flicker; OpenRAIL use-restrictions are behavioural (OK for games).

### Wan 2.2 (Alibaba) — open video generation incl. Wan-Animate
- URL: https://github.com/Wan-Video/Wan2.2
- License: Apache-2.0 for code and weights (verified), ~17.7k stars; released July–Sept 2025; models TI2V-5B (720p, runs on a single consumer GPU), I2V/T2V-A14B (MoE), **Animate-14B** (character animation/replacement driven by a reference video), S2V-14B.
- How it works: Diffusion transformer video models; Animate-14B transfers motion from a driving video onto a reference character image.
- What it would do: Offline: take a character portrait or sprite + a reference motion video → frames → downscale/quantise to sprite sheet; or generate VFX loops (smoke, water, fire). The most permissively licensed strong video model as of late 2026 (newer versions may exist — *check Wan 2.5+/3 licences, some later Wan releases were API-only*).
- Integrate? maybe (offline asset pipeline).
- Risks: Heavy GPU; frame consistency at small sprite sizes; must post-process heavily.

### Stable Video Diffusion (Stability AI)
- URL: https://stability.ai/license
- License: Stability AI Community License — commercial use free under US$1M annual revenue; Enterprise licence above (verified on licence page/secondary).
- Integrate? no (Wan 2.2 is more permissive and stronger).
- Risks: Revenue threshold.

### Meta AnimatedDrawings
- URL: https://github.com/facebookresearch/AnimatedDrawings
- License: MIT (verified), ~12.8k stars; **archived Sept 3, 2025**.
- How it works: Detect + segment a drawn humanoid, predict joints, build an ARAP mesh rig, retarget BVH motion onto the 2D rig.
- What it would do: Proof-of-concept for "player/god draws a creature, it walks"; also a recipe for auto-rigging 2D art.
- Integrate? maybe (fork for an in-game "drawn idol/totem comes alive" feature or tooling).
- Risks: Unmaintained; humanoid-only.

### Inochi2D — open Live2D alternative
- URL: https://github.com/Inochi2D/inochi2d
- License: BSD-2-Clause (verified), ~1.8k stars; D language with C FFI, official Unity binding, Godot GDExtension binding, WebAssembly/JS. Actively developed (editor "Inochi Creator" still in development).
- How it works: Layered 2D art with mesh deformation, parameters (e.g. head angle X/Y, mouth open, eye open, brow) that blend deformations; physics for hair.
- Runtime cost: One puppet = a few meshes; fine for a handful of portraits on screen, not crowds.
- What it would do: **Character portraits/inspection panel** with expressions driven by the agent's emotion state (fear, grief, joy), blinking, breathing, lip flaps during dialogue.
- Integrate? yes (portrait UI).
- Risks: Tooling maturity; D toolchain in build.

### Live2D Cubism (commercial reference)
- URL: https://www.live2d.com/en/sdk/license/
- License: Proprietary; free SDK release for "General Users/Small-Scale Enterprises" with <¥10M annual sales; above that a publication licence (running-royalty plan for games). (verified via Live2D help)
- Integrate? maybe — industry-standard tooling; licence becomes paid once revenue grows. Inochi2D preferred for openness.
- Risks: Fees, proprietary format.

---

## Part E — Facial / emotional animation and lip-sync

### FACS-based blendshape rigs (ARKit 52 standard) driven by appraisal emotion
- URL: FACS (Ekman & Friesen); ARKit blendshape list (Apple developer docs `ARFaceAnchor.BlendShapeLocation`) — *docs not fetched*.
- License: Concept; ARKit names are a de-facto standard used by many tools (VRM, Audio2Face, Unreal Live Link).
- How it works: Faces get ~52 blendshapes (browInnerUp, eyeSquintLeft, mouthSmileLeft, jawOpen…). An emotion layer maps sim emotion (e.g. PAD: pleasure/arousal/dominance, or discrete appraisal outputs: joy, distress, fear, anger, grief, disgust, surprise, pride, shame) to Action Unit weights — e.g. sadness ≈ AU1+AU4+AU15; fear ≈ AU1+2+4+5+20+26; joy ≈ AU6+12 — then AUs → blendshapes. Add noise/blinks/saccades and intensity decay.
- Runtime cost: Blendshapes on GPU; only meaningful at LOD0–1 (faces invisible beyond that).
- What it would do: Read a person's inner state when the god zooms in; also drives portrait (Inochi2D parameters) and sprite face overlays (2D: swap eyes/mouth/brow sprites by emotion).
- Integrate? yes (data table: emotion → AU weights → blendshape/sprite).
- Risks: Uncanny valley in realistic 3D — stylised faces strongly recommended.

### NVIDIA Audio2Face-3D (open-sourced Sept 2025)
- URL: https://github.com/NVIDIA/Audio2Face-3D ; SDK https://github.com/NVIDIA/Audio2Face-3D-SDK ; weights e.g. https://huggingface.co/nvidia/Audio2Face-3D-v3.0
- License: SDK MIT; training framework Apache; Maya/UE5 plugins MIT; **models (v2.3 regression, v3.0 diffusion) under NVIDIA Open Model License; Audio2Emotion models: custom licence, "use allowed with Audio2Face only"**; sample training data evaluation-only. (verified, ~459 stars on the collection repo)
- Type: C++/CUDA SDK (TensorRT), Python training.
- How it works: Audio → network → facial mesh deformation / blendshape weights (ARKit-style), tongue, jaw, eyes; Audio2Emotion infers emotion from voice to modulate.
- Runtime cost: GPU inference per speaking character; real time for a few.
- What it would do: Only relevant if our agents *speak audibly* (TTS from LLM dialogue). Then the followed person's face lip-syncs and emotes.
- Integrate? maybe (LOD0, optional, NVIDIA GPU dependence of SDK — *CPU/other-GPU paths unverified*).
- Risks: CUDA/NVIDIA-only runtime; the game probably uses text/simlish rather than voice.

### Rhubarb Lip Sync
- URL: https://github.com/DanielSWolf/rhubarb-lip-sync
- License: MIT (verified), ~2.6k stars.
- How it works: Offline CLI: audio (+ optional transcript) → timed mouth shapes A–F (+ G, H, X) in Hanna-Barbera style; PocketSphinx for English, phonetic recogniser for other languages.
- Runtime cost: Offline; output is tiny cue lists.
- What it would do: Perfect for 2D/stylised characters speaking pre-generated "simlish" barks or TTS lines; can also run on generated TTS at load time.
- Integrate? yes (2D/2.5D stylised).
- Risks: Offline speed (seconds per clip); unmaintained-ish (*last release date not verified*).

### uLipSync (Unity)
- URL: https://github.com/hecomi/uLipSync
- License: MIT (verified), ~1.7k stars.
- How it works: Real-time MFCC analysis of the audio buffer compared with calibrated vowel profiles (a/i/u/e/o) → blendshape weights; Job System + Burst; bakeable.
- Integrate? yes if Unity + voice; the MFCC idea ports to any engine.
- Risks: Vowel-only (Japanese-centric) visemes.

### Gibberish/"simlish" voice + procedural mouth flaps (no-audio-analysis approach)
- URL: technique (The Sims, Animal Crossing "Animalese") — no single source.
- How it works: The dialogue system emits syllable timing with the generated text; mouth opens per syllable, amplitude by emotion arousal.
- Runtime cost: Negligible; works for hundreds of speaking agents.
- Integrate? yes — the default for crowds of chatting villagers.

---

## Part F — Crowd & LOD animation

### Skinned instancing / GPU skinning with animation textures (GPU Gems 3, ch. 2 "Animated Crowd Rendering")
- URL: https://developer.nvidia.com/gpugems/gpugems3/part-i-geometry/chapter-2-animated-crowd-rendering
- License: Free-to-read technique (NVIDIA, 2007).
- How it works: All bone matrices of all clips are baked into a texture; each instance carries (clip id, time, LOD) in an instance buffer; the vertex shader fetches bone matrices from the texture and skins on the GPU; one draw call per mesh-LOD for thousands of characters.
- Runtime cost: Near-zero CPU per character; GPU cost ∝ vertices. 10k–100k characters feasible with mesh LODs.
- What it would do: Village- and map-scale 3D crowds of humans/animals; per-instance tint and bone-scale (height/build genes) still possible because skinning is per-instance.
- Integrate? yes — the core 3D crowd technique.
- Risks: No per-instance IK/blending beyond what the shader does (can add 2-clip crossfade and simple procedural bone offsets in shader).

### Vertex Animation Textures (VAT) + OpenVAT
- URL: https://github.com/sharpen3d/openvat ; https://openvat.org/ ; Bevy plugin https://github.com/HK416/bevy_open_vat
- License: OpenVAT Blender add-on **GPL-3.0** (tool only — outputs are our data, not GPL); its engine decoder samples for Unity/Unreal/Godot are **MIT**; bevy_open_vat MIT/Apache-2.0 (~15 stars, Bevy 0.18–0.19). (verified)
- How it works: Bake per-vertex positions (and normals) per frame into textures; vertex shader offsets vertices by (frame, vertex id). Works for any deformation — cloth, soft bodies, fluid-like meshes, even destruction.
- Runtime cost: Cheapest possible per-instance animation; memory ∝ vertices × frames, so suited to low-poly far-LOD meshes, small animals, birds, fish, insects, plants.
- What it would do: Far-LOD humans (200-vertex meshes), flocks of birds, fish schools, insect swarms, flags, tree sway variations.
- Integrate? yes.
- Risks: No variation of proportions except uniform/axis scale; memory blow-up on detailed meshes.

### Babylon.js BakedVertexAnimationManager / VertexAnimationBaker
- URL: https://doc.babylonjs.com/typedoc/classes/BABYLON.VertexAnimationBaker ; blog https://babylonjs.medium.com/creating-thousands-of-animated-entities-in-babylon-js-ce3c439bdacf
- License: Babylon.js Apache-2.0.
- How it works: Bake skeletal animation to a texture and play it per thin-instance (per-instance animation params buffer); demo: ~5,000 instances with different animations at ~50 fps (per Babylon docs/blog).
- Integrate? yes if web 3D front-end.

### Unity options: Animation-Instancing, Entities Graphics, Latios Kinemation
- URL: https://github.com/Unity-Technologies/Animation-Instancing ; https://github.com/Dreaming381/Latios-Framework
- License: Animation-Instancing: Unity Companion License (Unity-only), ~1.8k stars, 2018 vintage (stale). Latios Framework: Unity Companion License, ~1.4k stars, v0.16.1 targeting Entities 1.4 / Unity 6000.3. (verified)
- How it works: Animation-Instancing bakes skinned meshes for GPU instancing with LOD/culling/attachments. Latios Kinemation: DOTS/ECS animation with ACL-compressed clips, GPU skinning, IK, inertial blending, root motion, advanced LOD.
- Runtime cost: Kinemation targets thousands of animated entities in ECS.
- Integrate? maybe — only if engine = Unity. Commercial alternative: Rukhanka (Asset Store, *unverified*).
- Risks: Unity licensing/pricing history; DOTS churn.

### Assassin's Creed Unity crowd ("Massive Crowd on Assassin's Creed Unity: AI Recycling", GDC 2015)
- URL: https://gdcvault.com/play/1022411/Massive-Crowd-on-Assassin-s ; https://www.youtube.com/watch?v=Rz2cNWVLncI
- License: Talk.
- How it works: ~10,000 crowd NPCs on screen with only ~40 full AIs and ~120 high-res models; a pooling system swaps low-res "puppets" to high-res characters near the player without visible popping; low-res crowd runs a simplistic brain and cheap animation.
- What we can learn: Exactly our problem — **promotion/demotion between animation LOD tiers keyed by camera distance, with state continuity** (same clip phase, same pose class) to hide swaps.
- Integrate? yes (pattern).

### Ultimate Epic Battle Simulator 2 (Brilliant Game Studios)
- URL: https://steamcommunity.com/app/1468720 (dev statements in forums; no formal talk found)
- License: Proprietary.
- How it works (per developer statements): Everything — culling, LOD, transforms, AI and **full bone animation** — runs on the GPU; custom software rasteriser on GPU; millions of units; units even look at targets and blink.
- What we can learn: For 100k–1M, animation state must live on the GPU (state buffers), with CPU only issuing high-level changes. Our "semantic animation tags" should be compact (a few bytes) so they can be streamed to a GPU state buffer each tick.
- Integrate? pattern only.
- Risks: Extreme engineering.

### Total War (Creative Assembly) — reference only
- URL: no authoritative public tech talk found in this session (*unverified*); community analyses note matched-combat animations are reserved for some unit pairs and unit-wide animation sync issues at scale.
- What we can learn: Paired/matched animations (two people fighting, hugging, a mother lifting a child) are expensive; reserve them for LOD0–1 and use independent loops at distance.

### Animation impostors / sprite billboards for far LOD
- URL: technique; octahedral impostors (Ryan Brucks, Epic, 2018 blog — *not fetched*); classic "Polypostors"/animated impostors in crowd literature (Tecchia et al. 2002 — *unverified*).
- How it works: Pre-render each archetype × few animation frames × N view angles to an atlas; draw a camera-facing quad picking the right angle/frame.
- Runtime cost: One quad per agent → 1M agents feasible.
- What it would do: Map-scale tier in 3D; in 2D this *is* the normal sprite path.
- Integrate? yes (far tier).
- Risks: Popping between angles; lighting mismatch.

### Boids flocks/herds/schools + cheap animation
- URL: Reynolds "Flocks, Herds, and Schools" (SIGGRAPH 1987) — https://www.red3d.com/cwr/boids/ (*not fetched*).
- How it works: Separation/alignment/cohesion steering per agent (spatial hash), animation is a looping VAT/sprite cycle whose playback rate is proportional to speed (wingbeat/tailbeat/gait frequency) with per-agent phase offset; banking from turning rate.
- Runtime cost: GPU compute boids reach 100k+.
- What it would do: Birds, fish, insects, herds of deer at distance. When the god zooms in on one wolf, promote it to the procedural gait system.
- Integrate? yes.

---

## Part G — Environment animation

### L-systems and space colonization for plant growth
- URL: "The Algorithmic Beauty of Plants" (Prusinkiewicz & Lindenmayer, free PDF at algorithmicbotany.org — *not fetched*); Runions et al. 2007 "Modeling Trees with a Space Colonization Algorithm"; https://github.com/jasonwebb/2d-space-colonization-experiments (JS, ~225 stars, LICENSE file present but type *unverified*).
- How it works: L-systems rewrite symbol strings per growth step (parametric/stochastic L-systems encode genes: branching angle, internode length, phyllotaxis); space colonization grows branches toward attraction points (light/space), naturally competing with neighbours. Growth animation = interpolate between successive growth states (scale new internodes from 0, thicken older ones by pipe-model rule).
- Runtime cost: Generation per plant is cheap on growth events (days, not frames); rendering of mature plants uses instancing.
- What it would do: Plants visibly grow over sim days; plant genome → L-system parameters; drought → wilting angle offset; herbivory → removed branches.
- Integrate? yes (own implementation; algorithms are free).
- Risks: Mesh regeneration cost if many trees grow at once — batch growth updates across frames.

### EZ-Tree (procedural trees for three.js)
- URL: https://github.com/dgreenheck/ez-tree ; https://eztree.dev
- License: MIT (verified), ~1.7k stars.
- How it works: Parametric recursive branching (levels, angles, children, length, radius, taper, twist, gnarliness), leaves as cards, built-in wind animation via an update(time) call, LODs, GLB export.
- What it would do: Web front-end trees or an offline generator of tree variants exported to any engine.
- Integrate? yes (web) / maybe (as offline generator).

### Wind sway shaders (vertex displacement) and GPU grass (Ghost of Tsushima)
- URL: https://gdcvault.com/play/1027033/Advanced-Graphics-Summit-Procedural-Grass (Eric Wohllaib, GDC 2021); wind talk "Blowing from the West: Simulating Wind in Ghost of Tsushima" (Bill Rockenbeck, GDC 2021) https://www.youtube.com/watch?v=d61_o4CGQd8
- License: Talks; open re-implementations exist (e.g. https://github.com/cainrademan/Unity-Grass — *licence unverified*).
- How it works: Blades generated on GPU per tile with per-blade procedural shape; animation = bezier blade bent by a global scrolling wind noise field + per-blade phase; trees use hierarchical sway (trunk/branch/leaf frequencies) from vertex colour masks. Wind field is a low-res 2D/3D grid the sim can also own (storm direction).
- Runtime cost: GPU only; millions of blades.
- What it would do: Wind state from the weather sim drives every plant; trampled grass where herds walked (write to a "flattening" texture).
- Integrate? yes.

### Weather, fire and water VFX
- URL: engine particle systems (Godot GPUParticles3D/2D; Unity VFX Graph; Bevy `bevy_hanabi` https://github.com/djeedai/bevy_hanabi — GPU particles, MIT/Apache-2.0, ~1.4k stars, tracks Bevy 0.19 (verified)); VAT/flipbook pipelines (EmberGen is commercial).
- How it works: GPU particles for rain/snow/embers/smoke; flipbook textures for fire; water via scrolling normal maps + flowmaps, shoreline foam from depth; puddles via wetness mask; fire spread visualised by a per-cell "burning" texture the sim writes.
- Integrate? yes — the sim owns fire/water cells; VFX just reads them.

### Terrain change (erosion, floods, disasters)
- URL: https://github.com/SebLague/Hydraulic-Erosion (Unity, MIT, ~1k stars, verified).
- How it works: Droplet-based hydraulic erosion: many simulated raindrops pick up and deposit sediment based on speed/slope. For animation, the visual heightmap lerps toward the sim's new heightmap over seconds, with dust/water particles, so landslides/floods "play" rather than pop.
- Integrate? yes (algorithm; sim-side ownership).

### Building aging and damage
- How it works: Per-building "age/wear/damage/soot/moss" scalars from the sim drive shader blends (dirt and moss masks via world-space noise + AO), decals for cracks, mesh swaps for structural stages (built → worn → damaged → ruin); construction animation = scaffold prop + progressive reveal (clip plane rising with build progress).
- Runtime cost: Shader parameters only.
- Integrate? yes.

---

## Part H — Engine fit and per-engine animation libraries

| Engine / front-end | Built-in animation | Key OSS add-ons (licence) | Crowd path | Notes |
|---|---|---|---|---|
| **Godot 4.4–4.6** (MIT) | AnimationTree (blend spaces, state machines), Skeleton3D modifiers: LookAt/Retarget (4.4), SpringBoneSimulator3D (4.5), IKModifier3D family incl. TwoBone/FABRIK/CCD/Jacobian (4.6); Skeleton2D + 2D modifications | godot-motion-matching (MIT), Inochi2D GDExtension (BSD-2), spine-godot (Spine licence), OpenVAT decoder (MIT) | MultiMesh + VAT shader; custom GPU skinning | Best OSS fit for 2D and stylised 3D; C# or GDExtension (C++/Rust via godot-rust) for sim. |
| **Bevy** (MIT/Apache) | AnimationGraph with masks + additive blending (0.15+), GPU-driven rendering incl. skinned meshes (0.16) | bevy_open_vat (MIT/Apache), bevy_animation_graph (community editor, *licence unverified*), bevy_hanabi | Automatic instancing + VAT; custom compute | Great for a Rust ECS sim; tooling (editor) immature; API churn per release. |
| **Unity** (proprietary, runtime fee history) | Mecanim, Animation Rigging, 2D Animation (Sprite Library swapping) | JLPM22 MotionMatching (MIT), uLipSync (MIT), Latios Kinemation (UCL), Animation-Instancing (UCL) | DOTS/Entities Graphics; Latios | Most mature tooling; licence/business risk. |
| **Unreal 5** (EULA, 5% royalty over threshold) | Motion Matching (Pose Search), Control Rig, IK Rig/Retargeter, Mass crowd, AnimToTexture plugin | Audio2Face UE5 plugin (MIT) | Mass + AnimToTexture VAT | Overkill/heavy for a stylised god-game; strong for realistic 3D. |
| **Custom Rust/C++** | — | ozz-animation (C++, MIT, ~3k stars: SoA SIMD sampling/blending, glTF/FBX tooling, wasm), Holden Motion-Matching (MIT) | Own GPU skinning + VAT | Max control for 1M agents; must build tools. |
| **Web: Three.js** (MIT) | AnimationMixer, SkinnedMesh, InstancedMesh | EZ-Tree (MIT), spine-threejs (Spine licence) | InstancedMesh + VAT/bone textures | Easy distribution; perf ceiling lower; WebGPU helps. |
| **Web: Babylon.js** (Apache-2.0) | Animation groups, BakedVertexAnimationManager, thin instances | — | VAT on thin instances (~5k animated demo) | Strongest built-in crowd tooling on web. |
| **Web: PixiJS** (MIT) | Sprites/AnimatedSprite, ParticleContainer | pixi-spine / spine-pixi (Spine licence), DragonBones runtime (MIT) | Sprite batching, 100k+ sprites | Natural fit for 2D top-down. |

---

## Part I — Linking animation to simulation

Principle: **the simulation is the only source of truth; animation is a pure, lossy, deterministic-optional view.**
- The sim emits, per agent per tick, a compact *Animation Intent Record* (AIR): action, locomotion, posture, modifiers, targets and body descriptors (schema below).
- The animation layer, per LOD tier, resolves the AIR into whatever it can afford: a sprite row, a VAT clip id, a blended skeletal pose, a procedural IK pose, or an AI-generated clip.
- Nothing flows back except *cosmetic* events (footstep sound triggers, "contact frame reached" for VFX timing). Gameplay outcomes (hit landed, child picked up, fell off cliff) are decided by the sim, and animation is time-warped to meet them (e.g. sim says "pickup completes at t+0.8 s": the animation scales its contact phase to 0.8 s).
- Time-scaling: when the god runs the world at 10×–1000×, animation tiers drop automatically (at 100× nobody sees gait), and tags are interpolated or sampled.
- Determinism: animation randomness (idle fidgets, phase offsets) is seeded from agent id so replays look the same but nobody depends on it.

---

## Recommended animation architecture for our game

### 1. The sim → animation semantic interface (Animation Intent Record, AIR)

Emitted by the sim per agent when it changes (event-driven) plus a cheap per-tick transform. Kept small (≈16–48 bytes packed) so it can live in a GPU buffer for the crowd tiers.

**Body descriptors (slow-changing; set at birth, updated on aging/injury):**
- `species` / `body_plan` (biped, quadruped, hexapod, octopod, serpentine, avian, fish, plant)
- `age_norm` (0 = newborn … 1 = species max age) and derived `life_stage` (infant, toddler, child, adolescent, adult, elder)
- `height`, `limb_ratio`, `build` (lean↔heavy), `mass`, `sex/morph` (cosmetic), `pregnancy` 0–1
- `injury[]`: {`part` (leg_L, leg_R, arm_L, arm_R, spine, head, wing_L…), `kind` (fracture, wound, missing, burn), `severity` 0–1}
- `appearance`: palette genes, pelt/hair pattern ids, clothing/equipment layer ids, scars

**Locomotion (per tick):** `velocity` (vector), `desired_path` (next 2–3 waypoints, for trajectory-matching/foot prediction), `gait_hint` (auto | sneak | walk | trot | run | gallop | swim | fly | crawl | hop), `terrain` (ground/water/snow/mud — for footprints and gait), `stance` (stand, crouch, sit, lie, prone)

**Action (event):** `action` ∈ {idle, eat, drink, sleep, work(tool, verb: chop/dig/hammer/plant/harvest/cook/craft), carry(object/child, mode: arms/back/shoulder/sling/mouth), pick_up, put_down, give, hunt_stalk, pounce, attack(target, kind), block, flee, fall, die, mate/court(display_kind), nurse/feed_young, groom, play, talk(target, tone), pray/worship(god), mourn(target), celebrate, build(structure)}, with `target_id`/`target_pos`, `start_time`, `duration`, and `contact_time` (when the gameplay effect happens — animation must hit it).

**Affect/condition modifiers (continuous 0–1):** `valence`, `arousal`, `dominance` + top discrete `emotion` (joy, sadness/grief, fear, anger, disgust, surprise, shame, pride, love/affection), `fatigue`, `hunger`, `pain`, `sickness`, `intoxication`, `cold/heat`, `wetness`, `attention_target` (look-at).

**Social / paired:** `pair_id` + role (leader/follower) for hugs, fights, carrying another person, mother-infant nursing — only fulfilled as true paired animation at LOD0–1.

Rules: tags are *requests*; the animation layer degrades gracefully (unknown action → nearest generic: "work_generic", "interact_generic"). Animation never writes to the sim except cosmetic event callbacks (footstep, impact frame for VFX/sound).

### 2. Recommendations per art direction

**A. 2D top-down / isometric sprites (cheapest, scales best to 1M):**
- Humans: layered paper-doll sprite sheets (body base by life-stage × build bin, then head/hair/clothes/equipment layers, palette-swapped for genes); composite per individual into an atlas page on demand, or composite in a shader from layer indices.
- Generate those sheets from a **3D → 2D bake pipeline** (Dead Cells approach): low-poly rig in Blender/Godot, procedural age/limp layers applied in 3D, rendered to pixel art at 4 or 8 directions. This keeps genetic proportion and injury variation without hand-drawing every combo.
- Animals: Rain-World / argonautcode style **point-chain bodies** (MIT reference) with sprite segments for snakes, fish, lizards, insects; gait-generator + 2D IK legs for quadrupeds when zoomed in; baked flipbooks at distance.
- Close-up / followed person: optional runtime 2D skeletal rig (Godot Skeleton2D or Spine) driven procedurally; portraits with **Inochi2D** (BSD-2) for emotion.
- Engines/libraries: Godot 4 (2D) or PixiJS + DragonBones runtime (MIT); Spine if budget allows (commercial).

**B. 2.5D (3D world, fixed-ish camera, stylised):**
- Same as C below for close tiers, but far tiers are billboard impostors/VAT; favours Godot 4.6 (IK family, spring bones, retargeting) or Bevy.

**C. Stylised low-poly 3D:**
- One shared humanoid skeleton per body plan; **retarget** all clips; per-bone scale for genes; **additive procedural layers** (lean, limp, age posture, emotion posture, breathing) + **two-bone IK** foot planting + look-at + spring bones at LOD0.
- Locomotion: Overgrowth-style few-pose, distance-driven cycles or a small blend space; **motion matching only for the followed hero** if we have enough licensed mocap (Holden MIT code + our data).
- Crowds: GPU skinning with bone textures (LOD2) → VAT (LOD3) → impostors (LOD4).
- Animals: procedural gait generator with Froude-number gait selection, two-bone IK legs, CCD/FABRIK tails & necks; VAT at distance; boids for flocks/schools.
- Libraries: Godot 4.6 (MIT) **or** Bevy (MIT/Apache) + bevy_open_vat + bevy_hanabi; ozz-animation (MIT) if custom C++/Rust (via FFI); Babylon.js for a web build.

### 3. Animation LOD table

| Tier | When | Agents (typ.) | Humans | Animals | Face/emotion | Update rate |
|---|---|---|---|---|---|---|
| **LOD0 Observed individual** | followed/selected, < ~10 m | 1–20 | Full skeletal: clip/blend or motion matching + all procedural layers (age, limp, emotion posture), foot IK, hand IK for carry/tools, look-at, spring bones, active ragdoll on fall/death, paired interactions, lip flaps/visemes | Procedural gait + IK, tail/ear springs, breathing | Blendshapes/FACS (3D) or sprite face swaps / Inochi2D portrait (2D) | every frame |
| **LOD1 Nearby** | on screen, close | 20–300 | Skeletal blend tree + additive age/limp/emotion layers, foot IK only on slopes, simple look-at; no ragdoll (canned fall) | Procedural gait, no springs | Coarse expression (3–5 presets) | every frame, IK every 2nd |
| **LOD2 Village-scale crowd** | mid distance | 300–10k | GPU-skinned instanced meshes; clip id + phase + per-instance bone scale + 1 additive posture slot (elderly/limp/sad) | GPU-skinned or VAT; boids for groups | none | anim state from AIR on change; GPU playback |
| **LOD3 Region** | far | 10k–200k | VAT on 100–300-vertex meshes or 8-direction flipbook sprites; ~6 clip classes (idle, walk, run, work, carry, lie) | VAT/flipbook, boids | none | GPU only |
| **LOD4 Map-scale** | whole map | 200k–1M+ | Impostor quads / dots with 2–4-frame bob; colour by activity | dots/particles, herd blobs | none | GPU, sim-tick rate |

Promotion/demotion keeps continuity (same clip class & phase) — AC Unity "AI recycling" pattern. Time-scaling > ~20× forces everything to ≤ LOD2.

### 4. How genetic / age / injury variation is expressed procedurally
- **Skeleton level** (3D or bake pipeline): per-bone scale curves by `age_norm` × `height` × `limb_ratio`; mass/build → spine/hip width and blend-shape "build" morph.
- **Timing level**: cadence & stride from leg length and speed (no foot sliding: drive phase from distance travelled); gait selection by Froude number for animals; asymmetric phase for limps.
- **Pose level**: additive layers weighted by modifiers (elderly hunch, pain guarding of the injured part, sadness slump, fear crouch, pride chest-out, fatigue droop).
- **Physics level** (LOD0 only, optional): spring bones, ragdoll falls; offline RL controllers (MimicKit) to generate physically-plausible limp/cane/elderly clips for the library.
- **Surface level**: palette genes, pattern masks, scars/bandages decals, grey hair with age, clothing layers.
- **2D**: the same parameters choose bins in the baked atlas (life-stage × build × limp side/severity bin × emotion posture) plus runtime squash/bob tweaks.

### 5. Offline AI-generated animation pipeline vs runtime
- **Offline (recommended):** prompt library derived from the AIR action list × modifiers ("an elderly woman with a limp on her left leg carries firewood") → **NVIDIA Kimodo** (Apache code / NVIDIA Open Model weights, trained on commercially licensed mocap) or **ARDY** for long streamed sequences → retarget SOMA → our rig → automated cleanup (foot-contact detection + IK locking, root smoothing, loop-closing) → human animator review → tag clip with AIR metadata → compress (ACL/ozz) → bake to VAT/sprite atlases for lower tiers. Use motion→text captioning only from commercially clean models. Keep a provenance log (model, version, licence, prompt, date) for every clip.
- **Video models for 2D** (Wan 2.2, Apache-2.0; Animate-14B): concept and VFX loops; sprite frames only with heavy artist cleanup.
- **Runtime AI:** not for crowds. Possible later as an *optional* high-end "cinematic follow" mode (ARDY real-time on RTX-class GPU), strictly view-only and with a procedural fallback.
- Animals: no commercially clean generative models yet → procedural + hand-keyed accents.

### 6. Licensing checklist
- [ ] Every mocap/clip source logged with licence: allowed = own capture, CMU (no resale), Mixamo (no redistribution of raw files, **no ML training**), purchased packs (check "AI training" clauses), Kimodo/ARDY outputs (NVIDIA Open Model License — legal to review attribution & use clauses).
- [ ] **Banned for shipping:** LAFAN1 (CC BY-NC-ND), AMASS & SMPL/SMPL-X (non-commercial unless licensed via Meshcapade), AI4Animation code+data (CC BY-NC), MotionLCM (custom non-commercial), any checkpoint trained on HumanML3D/KIT/AMASS (MDM, MoMask, T2M-GPT, MotionGPT weights), Bones SEED (academic/startup only without commercial licence), Audio2Emotion outside Audio2Face.
- [ ] Territory-restricted: Tencent HY-Motion 1.0 (excludes EU/UK/South Korea) — don't use for a global release.
- [ ] Revenue-threshold licences: Spine (>US$500k → Enterprise), Live2D (>¥10M sales → publication licence), Stability Community (>US$1M → Enterprise), Unity/Unreal engine terms.
- [ ] Copyleft: LPC art layers (CC-BY-SA / GPL-3.0 pieces) — filter to CC0/CC-BY/OGA-BY or accept share-alike on art; GPL tools (OpenVAT add-on) are fine as tools, never link GPL code into the game.
- [ ] Unity Companion Licence code (Latios, Animation-Instancing) only usable with Unity.
- [ ] Patents: age-related gait transformation (US 11129551) — avoid copying that method; general motion-matching/IK are long-standing public techniques but get counsel review before launch.
- [ ] Credits file auto-generated from the provenance log (CC-BY attributions, MIT/Apache notices, Spine runtime licence text).
- [ ] AI-output copyright: purely AI-generated clips may be unprotectable (US) — keep human edits documented.

### 7. Open questions / risks
1. **Art direction decision** drives everything: 2D sprites cut cost and scale to 1M, but bake combinatorics (species × life stage × build × injury × action × direction) can explode atlas memory — need a bin budget and runtime layer compositing.
2. **Animal animation is the hardest content problem**: no commercial-clean data/models for quadrupeds; we must build a solid procedural gait system early (prototype wolf, rabbit, bird, fish, insect).
3. **Paired interactions** (carry child, fight, mate, nurse, hug) are where believability lives and where procedural systems struggle; budget hand-keyed paired clips with IK fix-ups for the main ~20 interactions.
4. **Time-scaling**: animation tiers must degrade instantly when the god speeds time; avoid systems whose state can't be fast-forwarded (springs/ragdolls must reset).
5. **Consistency across LOD swaps** (pop when zooming): keep clip class/phase continuous; cross-fade impostor→mesh.
6. **GPU state for 1M agents**: AIR must be packed; sim→GPU upload bandwidth per tick needs measuring (~32 B × 1M = 32 MB per full update — so send deltas only).
7. **Licence review** of NVIDIA Open Model License and any future Kimodo/ARDY versions; models update fast (2026 alone: Kimodo, ARDY, HY-Motion) — re-check quarterly.
8. **Uncanny valley**: realistic faces + generated motion risk creepiness; prefer stylisation.
9. **Engine choice** for a Rust ECS sim (Bevy) vs Godot tooling vs custom: animation tooling maturity favours Godot; scale favours custom/Bevy GPU-driven paths.
10. Items marked *unverified* above (exact star counts/dates of some repos, Spore patent status, cainrademan/Unity-Grass licence, 2d-space-colonization licence, PixelLab ToS) should be re-checked before decisions.
