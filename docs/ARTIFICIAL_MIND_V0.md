# ARTIFICIAL MIND V0 (frozen)

Code: `simulation/mind` (crate `alife-mind`). It depends only on `alife-core` and `alife-interface`. Minds receive `SensoryFrame`s and return `MotorCommand`s, and never see world truth.

Components: innate sensory encoding (34 feature units: hue, brightness and warmth thermometer codes, flicker, size, shape and surface) → learned concepts (prototype/ART) → attention workspace (K = 2–7) → working memory → associative memory (Pavlovian and instrumental, fast and slow weights, context-gated extinction, Pearce–Hall associability, cue and outcome salience, TD chaining) → episodic memory (96 + 16 pinned, ACT-R retrieval, episodic-control estimates) → semantic beliefs (crystallised, with confidence, source and evidence) → emotions and PAD mood → homeostatic drives → personality and age parameters → three decision systems (reflex/gut, habit, deliberate) with reliability arbitration → sleep consolidation and forgetting.

Status and known issues: see `docs/PHASE2_RESULTS.md`.
