# 08 — In-Game AI Models and Runtimes

Track: which AI models and inference runtimes could run *inside* the god-simulator (locally on a consumer PC, possibly in a browser), plus an optional cloud path.
Research date: 2026-10-01. Research method: web search + GitHub pages (Hugging Face, Wikipedia and several vendor blogs were blocked by the sandbox proxy, so some facts come from secondary press coverage; those are marked "secondary source" or "unverified").

**Ground rule for the whole document:** the deterministic simulation is the source of truth. Every model in this file is an *optional enhancer* whose output is either (a) pure flavour text that never feeds back into state, or (b) a structured proposal that the engine validates and may reject. Nothing here is allowed to be required for the simulation to run.

## How to read the sizing numbers

Rules of thumb used throughout (approximate; verify with your own benchmarks):

- GGUF Q4_K_M weights ~= 0.55–0.65 GB per billion parameters. So 1B ~0.7 GB, 4B ~2.5 GB, 8–9B ~5–5.5 GB, 12–14B ~7.5–9 GB.
- KV cache adds memory proportional to context x layers; at 4–8k context an 8B model typically needs another 0.5–1.5 GB (less with quantized KV cache or hybrid linear-attention models such as Qwen3.5/3.6 and Granite 4.0-H).
- Single-stream generation on an RTX 4060 (8 GB) for an 8B Q4_K_M model: reported ~40–58 tok/s generation and ~1.5k tok/s prompt processing (localscore.ai / databasemart benchmarks; numbers vary 2x by setup). Rough scaling: 4B ~2x faster, 1B ~4–6x faster; CPU-only (modern 8-core DDR5) roughly 8–15 tok/s for 4B and 25–50 tok/s for 1B at Q4.
- Generation speed is mostly memory-bandwidth-bound; batching many agents' requests together (llama.cpp `--parallel`, vLLM continuous batching) multiplies aggregate throughput far more than it slows each stream.
- MoE models (e.g. 35B total / 3B active) need RAM/VRAM for *all* weights but generate at roughly the speed of the active size; with CPU offload of expert tensors they are viable on 16–32 GB system RAM machines.

---

## 1. Small / medium open-weight LLMs

### Qwen3.5 small (0.8B / 2B / 4B / 9B)
- URL: https://github.com/QwenLM/Qwen3.5 (weights on Hugging Face `Qwen/Qwen3.5-*`)
- License (commercial game use OK? restrictions?): Apache 2.0 (per Qwen announcement and multiple secondary sources; the GitHub README defers to the LICENSE file shipped with each HF checkpoint — confirm per checkpoint before shipping). Commercial redistribution OK, keep NOTICE/license text.
- Type / language / platforms: dense decoder, natively multimodal (text+image+video input), hybrid attention architecture (Gated DeltaNet linear attention + gated attention, same family as Qwen3.5/3.6 MoE). GGUF available; llama.cpp README itself uses `Qwen3.5-0.8B-GGUF` as its quick-start example.
- Sizes & hardware needs: 0.8B (~0.6 GB Q4), 2B (~1.4 GB), 4B (~2.7 GB), 9B (~5.8 GB). Context 262,144 native (you will use 2–8k). 201 languages claimed.
- Activity/maintenance / release date: released 2026-03-02. Qwen line very active (Qwen3.6 Apr 2026, Qwen3.8 Aug 2026 — larger models only).
- What it would do in our game: default family for everything — 0.8B/2B for barks, name/word generation and classification; 4B for routine dialogue and JSON plan proposals; 9B for key-scene dialogue, reflections and chronicles.
- Strengths: permissive license; one family covering all tiers (consistent tokenizer/prompt format; one LoRA recipe); strong tool-calling/JSON; hybrid attention = small KV cache at long context; thinking/non-thinking modes.
- Weaknesses / risks: prose for role-play is "assistant-flavoured" out of the box (needs persona prompting or a LoRA); thinking mode wastes tokens — disable it for in-game calls; Chinese-lab provenance can be a concern for some publishers/platform holders (policy, not license).
- Integrate? **yes** — primary candidate. GGUF via llama.cpp; grammar/JSON-schema constrained.

### Qwen3.6-35B-A3B (MoE) and Qwen3.6-27B
- URL: https://github.com/QwenLM/Qwen3.5 (README lists 3.6/3.8 releases)
- License: Apache 2.0 (Qwen's own launch post on X states Apache 2.0).
- Type: sparse MoE, 35B total / ~3B active, 256 experts (8 routed + 1 shared), Gated DeltaNet hybrid attention, multimodal. 27B is dense.
- Sizes & hardware needs: ~20–23 GB at 4-bit; runs on 24 GB GPU, or on 12–16 GB GPU + 32 GB RAM with expert offload, or Apple Silicon 32 GB+. Speed near a 3B dense model when fully in VRAM.
- Activity / release date: 35B-A3B 2026-04-16; 27B 2026-04-22.
- What it would do in our game: high-end-PC "narrator/chronicler" and key-NPC dialogue; also an excellent cheap self-hosted *server* model for batch reflections.
- Strengths: near-frontier quality per active parameter; fast; permissive.
- Weaknesses / risks: memory footprint far too big for min-spec; coding/agentic-tuned more than prose-tuned.
- Integrate? **maybe** — optional "Ultra" tier and server-side batch tier.

### Gemma 4 (E2B, E4B, 12B "Unified", 26B-A4B MoE, 31B dense)
- URL: https://github.com/google-deepmind/gemma ; announcement https://blog.google/innovation-and-ai/technology/developers-tools/gemma-4/
- License: **Apache 2.0** (first Gemma under Apache; replaces the restrictive "Gemma Terms of Use" + Prohibited Use Policy that applied to Gemma 1–3/3n). Confirmed by multiple press sources (VentureBeat, eWeek, gHacks, BetaNews). Older Gemma 3 / 3n / EmbeddingGemma checkpoints remain under the Gemma Terms of Use — do not mix them up.
- Type: multimodal (image+video input on all; native audio input on E2B/E4B). "E" = effective parameters (per-layer embeddings let E2B/E4B run with memory of a ~2B/4B model).
- Sizes & hardware needs: E2B ~1.5–2 GB at 4-bit, E4B ~3 GB, 12B ~7.5 GB, 26B-A4B ~16 GB (MoE, 4B active), 31B ~19 GB. Context 128K (E2B/E4B), 256K (26B/31B).
- Activity / release date: 2026-04-02; 12B Unified variant 2026-06-03 (secondary source). Day-one support in llama.cpp, LM Studio, mistral.rs (v0.8.2 notes Gemma 4).
- What it would do in our game: strong alternative to Qwen at every tier; Gemma models historically write warmer, more natural dialogue and handle many languages well; E4B's audio input could let the god *speak* to the world without a separate STT model.
- Strengths: Apache 2.0; designed for on-device; good multilingual and conversational tone; function calling.
- Weaknesses / risks: check that any GGUF/finetune you ship is derived from Gemma 4 not Gemma 3; multimodal towers add download size you may not need (text-only GGUF exports exist from community).
- Integrate? **yes** — co-primary with Qwen3.5; A/B both on our dialogue eval set.

### Ministral 3 (3B / 8B / 14B, base/instruct/reasoning)
- URL: https://mistral.ai/news/mistral-3/ (HF `mistralai/Ministral-3-8B-Instruct-2512` etc.)
- License: Apache 2.0 (all sizes and variants).
- Type: dense, image understanding, multilingual; versions tagged 25-12.
- Sizes & hardware needs: 3B ~2 GB, 8B ~5 GB, 14B ~8.5 GB at Q4_K_M.
- Activity / release date: 2025-12-02 (with Mistral Large 3).
- What it would do in our game: dialogue/narration for European-language-heavy markets; Mistral models traditionally have less "assistant" tone and good prose.
- Strengths: permissive; EU vendor (useful for some publishers); 14B is a good high-end single-GPU dialogue model.
- Weaknesses / risks: older than Qwen3.5/Gemma 4; smaller community finetune ecosystem than Mistral Nemo had.
- Integrate? **maybe** — keep in eval pool.

### Mistral Nemo 12B / Mistral Small 3.x 24B (older Mistral, Apache 2.0)
- URL: https://mistral.ai/news/ (unverified exact pages)
- License: Apache 2.0.
- Sizes: 12B ~7.5 GB Q4; 24B ~14 GB Q4.
- Activity: 2024–2025 releases.
- What it would do: Nemo 12B is the base of many popular role-play community finetunes ("MN-..." models). Community finetunes have mixed/unclear dataset provenance — **do not ship community RP merges in a commercial game**; instead do our own LoRA.
- Integrate? **no** (superseded), except as reference for RP quality.

### Phi-4-mini (3.8B), Phi-4 (14B), Phi-4-multimodal (5.6B), Phi-4-reasoning
- URL: https://huggingface.co/microsoft/Phi-4-mini-instruct
- License: MIT (very permissive).
- Type: dense; strong reasoning/function calling; ONNX builds published by Microsoft (good for ONNX Runtime GenAI/DirectML).
- Sizes: 3.8B ~2.5 GB Q4; 14B ~9 GB Q4. 128K context (mini).
- Activity / release date: Phi-4-mini 2025-02-26. No GA "Phi-5" as of mid-2026 (secondary source).
- What it would do: structured planner/validator-friendly proposals; logic puzzles; ONNX/DirectML path on Windows without CUDA.
- Strengths: MIT; reasoning; official ONNX.
- Weaknesses / risks: notoriously dry, "textbook" prose — poor for character voice; aging.
- Integrate? **maybe** — planner role only.

### SmolLM3-3B (and SmolLM2 135M/360M/1.7B)
- URL: https://huggingface.co/HuggingFaceTB/SmolLM3-3B
- License: Apache 2.0; fully open (data + recipes published).
- Sizes: 3B ~2 GB Q4; SmolLM2-135M/360M are tiny (<300 MB) — browser-friendly.
- Activity / release date: SmolLM3 2025-07-08. No SmolLM4 found.
- What it would do: browser build; ultra-low-end fallback; a fully-reproducible base if legal wants data provenance.
- Strengths: transparency of training data (helps legal review); tiny variants for WebGPU/WASM.
- Weaknesses: weaker than Qwen3.5-2B/4B in quality; English-centric (6 languages).
- Integrate? **maybe** — browser/no-GPU tier and "provenance-clean" option.

### Olmo 3 (7B / 32B; Base, Instruct, Think, RL-Zero) — Ai2
- URL: https://allenai.org (OpenRouter/Ai2 announcement 2025-11-20)
- License: Apache 2.0; fully open (data, checkpoints, code).
- Sizes: 7B ~4.5 GB Q4; 32B ~19 GB.
- Activity: 2025-11-20 (unverified whether an Olmo 3.x update shipped in 2026).
- What it would do: provenance-clean dialogue model if publisher demands auditable training data.
- Integrate? **maybe**.

### IBM Granite 4.0 (H-Micro 3B, H-Tiny 7B-A1B, H-Small 32B-A9B; Nano 350M / ~1B)
- URL: https://www.ibm.com/granite/docs/models/granite
- License: Apache 2.0; checkpoints cryptographically signed; ISO 42001.
- Type: hybrid Mamba-2/transformer (much lower KV memory at long context); MoE for Tiny/Small.
- Sizes: Nano 350M (CPU, ~250 MB Q4), Nano 1B, H-Micro 3B, H-Tiny 7B total / 1B active (~4.5 GB at Q4 but runs near 1B speed).
- Activity / release date: 2025-10-02 (4.0), 2025-10-28 (Nano).
- What it would do: CPU-only tier classification/extraction ("did this sentence mention a person?"), tool calling; H-Tiny is a fast CPU model.
- Strengths: enterprise-clean license story; tool-calling focus; hybrid architecture = cheap long-memory agents.
- Weaknesses: enterprise-flavoured prose; not role-play tuned.
- Integrate? **maybe** — utility/extractor model.

### LiquidAI LFM2 / LFM2.5 (230M–2.6B, VL variants)
- URL: https://www.liquid.ai/lfm-license
- License: **LFM Open License v1.0** — Apache-2.0-based but **commercial use is free only for companies with < US$10M annual revenue**; above that you need a paid license. This threshold also applies to derivatives. Risky for a commercial game that might succeed.
- Sizes: 230M, 350M, 700M, 1.2B, 2.6B; extremely fast on CPU.
- What it would do: CPU barks/extraction.
- Integrate? **no** for shipped game (license cliff); OK for prototyping.

### NVIDIA Nemotron Nano 9B v2 / Nemotron Nano family (incl. ACE "Nemotron Mini 4B")
- URL: https://developer.nvidia.com/downloads/assets/ace/model_card/nemotron-nano-9b-v2.pdf
- License: **NVIDIA Open Model License** (June 2025 version): commercial use and derivatives OK, must include license text + attribution, must not disable built-in "guardrails" without comparable replacement, export-control compliance. Workable but a custom license (legal review).
- Type: hybrid Mamba-transformer, reasoning on/off toggle. Distributed as part of NVIDIA ACE.
- Sizes: 9B (~5.5 GB Q4). (Nemotron Mini 4B Instruct was the ACE on-device NPC model in 2024–25.)
- What it would do: NVIDIA-GPU tier if we adopt ACE (see section 7).
- Integrate? **maybe** (only with ACE).

### OpenAI gpt-oss-20b
- URL: https://github.com/openai/gpt-oss
- License: Apache 2.0 (plus a usage policy that OpenAI publishes; Apache terms govern).
- Type: MoE 21B total (~3.6B active), MXFP4 native weights, *must* use "harmony" chat format; reasoning model with adjustable effort.
- Sizes: runs in ~16 GB memory.
- Activity / release date: 2025-08-05.
- What it would do: high-end planner/reasoner; good tool use.
- Weaknesses: reasoning tokens add latency; heavily safety-tuned (may refuse violent/dark plot content common in a god sim — wars, sacrifice); prose is bland.
- Integrate? **maybe** — planner/judge on high-end or server.

### Meta Llama (3.1 8B, 3.2 1B/3B, 4 Scout/Maverick; 2026 "Muse" models)
- URL: https://github.com/meta-llama/llama-models/blob/main/models/llama3_2/LICENSE ; https://www.llama.com/faq/
- License: **Llama Community License** (3.1/3.2/3.3/4): commercial OK below 700M MAU, but you must display **"Built with Llama"** prominently, include the license, name derivatives starting with "Llama", obey Meta's **Acceptable Use Policy** (which you must pass on to users), and Llama 3.2 *multimodal* models withhold rights from EU-domiciled entities (well-documented; verify against the exact license text). Not OSI open source. Meta can update the AUP.
- Sizes: 3.2-1B (~0.8 GB Q4), 3.2-3B (~2 GB), 3.1-8B (~4.9 GB). Llama 4 Scout/Maverick are 109B/400B MoE — not consumer.
- Activity: Meta pivoted toward closed models ("Avocado", Dec 2025 reports). Secondary sources report a 2026-08-10 release of "Muse Glimmer" (30B dense, Apache 2.0) by Meta Superintelligence Labs — **unverified** (primary pages were unreachable); re-check before relying on it.
- What it would do: Llama 3.1 8B remains the base of many RP finetunes and is still praised for English prose.
- Weaknesses / risks: license attribution + AUP pass-through is awkward in a game; family is stale at small sizes; strategic uncertainty.
- Integrate? **no** for Llama-licensed models (Apache alternatives are now equal or better). Re-evaluate "Muse" if confirmed Apache.

### DeepSeek-R1 distills (1.5B/7B/8B/14B/32B) and other reasoning distills
- License: DeepSeek's weights are MIT, but each distill inherits its base model's license (Qwen2.5-based: Apache 2.0 / Qwen license; Llama-based 8B/70B: Llama license).
- What it would do: nothing useful in-game — long chain-of-thought means high latency; superseded by Qwen3.5/Gemma 4 thinking modes.
- Integrate? **no**.

### Others noted, not recommended
- **Hunyuan / ERNIE / GLM / Kimi** small or MoE releases: check each license (several Tencent licenses exclude EU/UK/South Korea territories — a showstopper for a global game). Unverified for 2026 versions.
- **Gemma 3 / 3n / 270M**: Gemma Terms of Use (prohibited-use policy pass-through, Google can update policy) — prefer Gemma 4.
- **EXAONE (LG)**, **Command-R7B (Cohere)**: non-commercial / CC-BY-NC style licenses — **no**.
- Community RP finetunes (Stheno, Lumimaid, "MN-..." merges): unknown training data and frequently inherit Llama/NC terms — **no** for shipping; fine for private taste-tests.

### Quality notes for role-play, dialogue and JSON
- No standard benchmark captures character voice well; build our own eval set (200 prompts: in-character replies, refusal-to-break-character, staying within known facts, JSON proposals) and score with a large judge model offline.
- For JSON, *constrained decoding* (section 2) makes even 1–2B models produce 100% schema-valid output; the remaining problem is *semantic* validity (e.g. targeting a person who does not exist), which the engine must check.
- Small models (<4B) are good at short barks and slot-filling but lose persona consistency over multi-turn talk; 8–14B is the practical floor for satisfying open-ended conversation with a key character.

---

## 2. Inference runtimes and constrained decoding

### llama.cpp / GGML / GGUF
- URL: https://github.com/ggml-org/llama.cpp
- License: MIT (ggml also MIT). Commercial embedding OK; include notice.
- Type / language / platforms: C/C++ library + CLI + OpenAI-compatible `llama-server`; Windows/Linux/macOS/Android/iOS; backends CUDA, HIP (AMD), SYCL (Intel), Vulkan, Metal, plus CPU (AVX2/AVX-512/AMX/NEON), CANN etc. WebGPU backend in progress (unverified as production-ready).
- Sizes & hardware needs: tiny runtime (single-digit MB); quantizations 1.5–8 bit; hybrid CPU+GPU offload.
- Activity: ~130k stars, near-daily releases (b-numbered builds).
- What it would do in our game: **the default local runtime**. Embed as a library (or ship `llama-server` as a child process bound to localhost) — the game talks to it via a thin job queue.
- Strengths: broadest model + hardware support; GBNF grammars and JSON-schema-to-grammar built in; optional llguidance backend; parallel slots / continuous batching in server; prompt (KV) cache reuse per slot and save/restore of KV state to disk; LoRA adapters hot-loadable; speculative decoding.
- Weaknesses / risks: fast-moving API (pin a version); Vulkan path a bit slower than CUDA; GPU contention with the game's renderer (cap layers/threads, run at lower priority).
- Integrate? **yes** — core.

### LLamaSharp (C#/.NET bindings)
- URL: https://github.com/SciSharp/LLamaSharp
- License: MIT. ~3.8k stars.
- Platforms: CPU, CUDA 11/12, Vulkan, Metal backends via NuGet; Unity examples exist.
- What it would do: in-process llama.cpp from a C# (Unity/Godot-Mono/.NET) engine; includes grammar sampling, batched executor, KV save/load.
- Weaknesses: lags upstream llama.cpp by weeks; native binaries per platform must be bundled.
- Integrate? **yes** if engine is C#.

### LLMUnity ("LLM for Unity", undream ai)
- URL: https://github.com/undreamai/LLMUnity
- License: Apache 2.0 (~1.7k stars). Also on Unity Asset Store.
- Platforms: PC, mobile, VR; Nvidia/AMD/Metal GPUs + CPU; built on llama.cpp (via their LlamaLib).
- Features: grammar constraints / function calling, RAG with semantic search, multi-character samples, model manager that bundles GGUF at build time or downloads on first run.
- Integrate? **maybe** — fastest Unity prototype; for production prefer direct llama.cpp control.

### llama-cpp-rs (`llama-cpp-2` crate)
- URL: https://github.com/utilityai/llama-cpp-rs
- License: MIT / Apache-2.0 dual. ~655 stars. Thin, close-to-upstream bindings; CUDA/Vulkan/Metal features.
- Integrate? **yes** if engine is Rust (alternatives: mistral.rs, candle).

### Godot plugins: godot-llm, NobodyWho
- godot-llm — https://github.com/Adriankhl/godot-llm — MIT, ~250 stars; GDLlama (generation with JSON schema/GBNF), GDEmbedding, LlmDB (SQLite vector store), CPU/Vulkan; Windows/Linux/macOS/Android. Small maintainer base.
- NobodyWho — https://github.com/nobodywho-ooo/nobodywho — **EUPL-1.2**: proprietary games may *use* it, but modifications to the plugin itself must stay open (no proprietary fork). Godot 4.5+, llama.cpp-based, built for NPC dialogue; tool calling and embeddings.
- Integrate? **maybe** if engine is Godot; otherwise write our own GDExtension over llama.cpp.

### ONNX Runtime + ONNX Runtime GenAI
- URL: https://github.com/microsoft/onnxruntime-genai
- License: MIT (~1.1k stars for GenAI; ORT itself very large project).
- Platforms: Python, C#, C/C++, Java; Windows/Linux/macOS/Android; EPs: CPU, CUDA, **DirectML**, OpenVINO, QNN (Snapdragon NPUs), **WebGPU**, TensorRT-RTX.
- Features: KV cache management, multi-LoRA, continuous decoding, constrained decoding (llguidance).
- What it would do: Windows-native path via DirectML/TensorRT-RTX with no CUDA install; runs our small non-LLM models (section 8) and embeddings too.
- Weaknesses: models must be converted (Olive/model builder); fewer ready-made LLMs than GGUF.
- Integrate? **yes** for small nets/embeddings; **maybe** for LLMs.

### Unity Sentis / Inference Engine (`com.unity.ai.inference`)
- URL: https://docs.unity3d.com/Packages/com.unity.ai.inference@2.6/manual/index.html
- License: Unity package under Unity's terms; no separate license fee since Sentis 1.2. Renamed "Inference Engine" in Unity 6.2 (Aug 2025), display name reverted to "Sentis" in 2.4.
- What it would do: run ONNX per-creature policies, small classifiers, possibly embeddings inside Unity on all platforms (GPU compute or CPU, Burst).
- Weaknesses: Unity itself says LLMs are not its target use case.
- Integrate? **yes** for small nets if Unity; **no** for LLMs.

### MLC-LLM and WebLLM
- MLC-LLM — https://github.com/mlc-ai/mlc-llm — Apache 2.0, ~23k stars; TVM-compiled models for Vulkan, Metal, CUDA, ROCm, OpenCL, WebGPU, iOS/Android.
- WebLLM — https://github.com/mlc-ai/web-llm — Apache 2.0, ~19k stars; in-browser inference with WebGPU, OpenAI-style API, JSON mode / JSON schema via xgrammar, runs in Web Workers/Service Workers.
- What it would do: **browser build of the game** — Qwen/Gemma/SmolLM 0.5–4B in the tab.
- Weaknesses: WebGPU availability (Chrome/Edge good; Safari/Firefox improving), multi-GB downloads cached in browser storage, model zoo lags new releases (compile step).
- Integrate? **yes** for web build.

### transformers.js
- URL: https://github.com/huggingface/transformers.js
- License: Apache 2.0, ~16k stars. ONNX Runtime Web under the hood; WebGPU (marked experimental) and WASM; fp32/fp16/q8/q4.
- What it would do: browser embeddings (MiniLM, EmbeddingGemma), Kokoro TTS, Whisper/Moonshine STT, small text models.
- Integrate? **yes** for web build auxiliary models.

### ExecuTorch
- URL: https://github.com/pytorch/executorch
- License: BSD; ~5k stars. Android/iOS/desktop/embedded; XNNPACK, Vulkan, CoreML, QNN, CUDA, WebGPU backends.
- What it would do: console/mobile ports; per-creature PyTorch policies exported directly.
- Integrate? **maybe** (mobile/console only).

### mistral.rs
- URL: https://github.com/EricLBuehler/mistral.rs
- License: MIT, ~7.7k stars, v0.8.x (Gemma 4 support noted).
- Features: Rust crate; CUDA/Metal/CPU; continuous batching + PagedAttention; GGUF and many quant formats; grammar/strict-schema tool calling (llguidance).
- Integrate? **maybe** — strong alternative for a Rust engine wanting batching in-process.

### candle (Hugging Face, Rust)
- URL: https://github.com/huggingface/candle
- License: MIT / Apache-2.0, ~21k stars. CPU/CUDA/Metal/WASM; loads GGUF quantized weights.
- What it would do: pure-Rust embeddings, small nets, whisper; WASM demos.
- Integrate? **maybe** (for small models; less optimal for LLM throughput than llama.cpp).

### Ollama
- URL: https://github.com/ollama/ollama
- License: MIT, ~182k stars. Desktop daemon (macOS/Windows/Linux/Docker) wrapping llama.cpp-derived engine; structured outputs via a JSON-schema `format` field.
- What it would do: **dev tooling** and "bring your own Ollama" mod support. Not ideal to bundle (separate service, its own model store, auto-updates).
- Integrate? **maybe** — optional backend users can point the game at.

### LM Studio + lmstudio-js / lmstudio-python SDK
- URL: https://lmstudio.ai/blog/introducing-lmstudio-sdk ; https://lmstudio.ai/app-terms
- License: SDKs and `lms` CLI MIT; the **LM Studio app is proprietary** — free for personal and internal business use since July 2025, but **may not be redistributed or built into a product you sell** without an enterprise agreement.
- Integrate? **no** for bundling; OK as user-supplied optional backend (OpenAI-compatible endpoint).

### vLLM and SGLang (server-side)
- vLLM — https://github.com/vllm-project/vllm — Apache 2.0, ~93k stars; PagedAttention, continuous batching, automatic prefix caching, multi-LoRA, structured outputs (xgrammar/llguidance/outlines backends), OpenAI- and Anthropic-compatible APIs.
- SGLang — https://github.com/sgl-project/sglang — Apache 2.0, ~37k stars; RadixAttention prefix cache (excellent when thousands of prompts share the same world-lore prefix), structured outputs, very high throughput.
- What they would do: our *own* cloud/offline tier — batch-generate chronicles, myths, names and background reflections at tiny per-token cost; also server for an online/multiplayer version.
- Integrate? **yes** for server/batch tier; **no** inside the client.

### Constrained decoding libraries
| Library | License | Where it runs | Notes |
|---|---|---|---|
| llama.cpp GBNF + JSON-schema-to-grammar | MIT | in llama.cpp, LLamaSharp, godot-llm, LLMUnity | Built in; GBNF can encode an *enumerated* action list. |
| llguidance (guidance-ai) | MIT | llama.cpp (build flag), vLLM, SGLang, mistral.rs, ORT GenAI, Chromium; powers OpenAI structured outputs | ~50 µs/token mask; Lark grammars + JSON schema. v1.0 June 2025. |
| XGrammar (mlc-ai) | Apache 2.0 | vLLM, SGLang, TensorRT-LLM, MLC/WebLLM, OpenVINO | v2.0 May 2026; Python/C++/JS/Swift bindings; near-zero JSON overhead. |
| Outlines (dottxt) | Apache 2.0 | transformers, llama.cpp (python), vLLM, Ollama | Python-centric; good for offline tooling and evals. |
| lm-format-enforcer | MIT (unverified) | transformers, vLLM, llama-cpp-python | Older; lower priority. |
| guidance (Microsoft/guidance-ai) | MIT | Python | Template-interleaved generation; uses llguidance internally. |

Design pattern that matters most: **build the grammar per request from the live world**. E.g. the allowed `target_id` values are an enum of agent IDs actually visible to the speaker; the allowed `action` values are the verbs the engine currently permits for that agent. Grammar-constrained decoding then makes it *impossible* to emit a nonexistent target; the engine still re-checks preconditions at resolution time because the world may have changed between request and response.
