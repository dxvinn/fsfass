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

