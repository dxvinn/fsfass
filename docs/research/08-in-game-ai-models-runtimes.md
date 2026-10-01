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

---

## 3. Cloud API option (Anthropic Claude)

### Claude API
- URL: docs https://platform.claude.com/docs/en/about-claude/models/overview.md ; pricing https://platform.claude.com/docs/en/about-claude/pricing.md
- License: commercial API under Anthropic's commercial terms + usage policy; outputs usable in the game. Player-facing generation must respect the usage policy (fine for violence-in-fiction, myths, war chronicles; not for e.g. sexual content involving minors).
- Type: hosted API (HTTPS), official SDKs for Python, TypeScript, Java, Go, Ruby, C#, PHP. Also available via Amazon Bedrock, Google Vertex AI, Microsoft Foundry.
- Model tiers (from Anthropic's model table cached 2026-09-25, $ per million input / output tokens): **Claude Haiku 4.5** ($1 / $5, 200K context) — cheap barks/batch; **Claude Sonnet 5.5** ($2 / $10, 1M context) — default for dialogue/narration quality per dollar; **Claude Opus 5.5** ($4 / $20, 1M context) — best general model for key chronicles, world-myth generation, design-time content; **Claude Fable 5.1** ($10 / $50) — most capable, overkill for runtime.
- Features useful here:
  - **Structured outputs** (`output_config.format` with a JSON schema) and **strict tool use** (`strict: true` on a tool) guarantee schema-valid JSON — the cloud equivalent of grammar-constrained decoding. https://platform.claude.com/docs/en/build-with-claude/structured-outputs.md ; https://platform.claude.com/docs/en/agents-and-tools/tool-use/overview.md
  - **Prompt caching**: mark the stable prefix (world bible, rules, character sheet) with `cache_control`; reads cost ~0.1x base input (0.05x on Opus 5.5), writes 1.25x (5-min TTL) or 2x (1-hour TTL). Any byte change in the prefix invalidates it — keep volatile state (time, current event) *after* the cached part. https://platform.claude.com/docs/en/build-with-claude/prompt-caching.md
  - **Message Batches API**: asynchronous, ~50% cheaper, results keyed by `custom_id` in any order — ideal for "overnight" reflections, chronicles and name generation during fast-forward. https://platform.claude.com/docs/en/build-with-claude/batch-processing.md
  - **Effort control** (`output_config.effort` low…max) to trade reasoning depth for cost/latency; use `low` for dialogue.
  - Rate limits per tier: https://platform.claude.com/docs/en/api/rate-limits.md
- Notes for current models: assistant-message prefill is rejected on the 4.6+ family (use structured outputs instead); the newest models reject forced `tool_choice` (use `auto` + strict tools or structured outputs); a `refusal` stop reason must be handled (fallback to template text).
- What it would do in our game: optional "Cloud Storyteller" mode (BYO API key or publisher-paid subscription), server-side batch generation of chronicles/myths, design-time generation of name banks and dialogue templates that ship *as data* (zero runtime cost), and an offline judge for evaluating local models.
- Strengths: far higher quality than any local model; long context for whole-civilisation histories; schema guarantees.
- Weaknesses / risks: per-token cost scales with players; latency 0.5–3 s+; requires connectivity; content-policy refusals on dark themes need graceful fallback; ongoing cost if publisher pays.
- Integrate? **yes, as optional tier + offline content pipeline**, never as a requirement.

### Cost-control patterns (cloud and local alike)
1. **Generate at design time, ship as data**: name banks, myth templates, thousands of bark lines per trait combination — cost once.
2. **Cache-friendly prompts**: fixed world bible + character card first; event summary last.
3. **Batch API** for anything not needed within the next minute (50% saving; latency hours-tolerant).
4. **Route by importance** (section 9): only "notable" agents (rulers, prophets, the god's chosen, characters the player is watching) ever get an LLM call.
5. **Summarise, don't stream**: send the LLM a compact structured digest (10–30 facts) not raw logs.
6. **Hard budgets**: per-in-game-day token budget and per-session $ cap; once spent, fall back to templates.
7. **Short outputs**: max_tokens 60–150 for barks/dialogue lines; chronicles are the only long outputs.

---

## 4. Embedding / retrieval models (memory for LLM-backed agents)

Only the few hundred "notable" agents need semantic memory retrieval; everyone else uses symbolic memory (event IDs, tags, salience scores) in the engine. Embeddings also help: dedupe generated names/myths, cluster events into "eras" for chronicles, and match player free-text commands to engine verbs.

### all-MiniLM-L6-v2 (sentence-transformers)
- URL: https://www.sbert.net/ (HF `sentence-transformers/all-MiniLM-L6-v2`)
- License: Apache 2.0.
- Sizes: 22M params, 384-dim, ~90 MB fp32 (~25 MB int8). Thousands of sentences/sec on CPU.
- Activity: 2021, stable, ubiquitous; ONNX + transformers.js versions exist.
- What it would do: baseline CPU/browser memory embedding.
- Weaknesses: English-only, short context (256 tokens), dated quality.
- Integrate? **yes** as fallback/browser default.

### Model2Vec "potion" static embeddings (MinishLab)
- URL: https://github.com/MinishLab/model2vec
- License: MIT (~2.2k stars). potion-base-8M (7.5M params), potion-base-32M, potion-retrieval-32M, potion-multilingual-128M.
- What it would do: **per-agent-scale** embeddings — static lookup + mean pooling, claimed up to 500x faster than the source transformer on CPU; small enough to embed *every* memory of *every* agent if ever needed, and trivially portable to C#/Rust/JS (it is just a token→vector table).
- Weaknesses: lower quality than transformer embedders; no contextual meaning.
- Integrate? **yes** — default for bulk/low-end; deterministic and engine-friendly.

### Qwen3-Embedding-0.6B
- URL: https://qwen.ai/blog?id=qwen3-embedding
- License: Apache 2.0. 595M params (~0.6 GB Q8 GGUF); top-tier MTEB for its size (~70 on MTEB-eng-v2 per secondary sources); multilingual; instruction-aware; runs in llama.cpp.
- Integrate? **yes** — quality tier (mid/high PC), shares runtime with the LLM.

### EmbeddingGemma-300M
- URL: https://ai.google.dev/gemma/docs/embeddinggemma
- License: **Gemma Terms of Use** (based on Gemma 3; not relicensed to Apache with Gemma 4 as of research date). Commercial OK but prohibited-use policy pass-through.
- Sizes: 308M, <200 MB quantized, Matryoshka dims (768→128), multilingual, on-device focus.
- Integrate? **maybe** — good model, but prefer Apache options unless legal is fine with Gemma terms.

### nomic-embed-text v1.5 / v2-moe
- License: Apache 2.0. v2: MoE 475M total / 305M active, multilingual, 8192 context; v1.5 ~137M, Matryoshka.
- Integrate? **maybe**.

### BGE-M3 / bge-small-en-v1.5 (BAAI), gte-multilingual-base (Alibaba), multilingual-e5-small (Microsoft)
- Licenses: BGE-M3 MIT (568M, dense+sparse+multi-vector); bge-small MIT; gte-multilingual-base Apache 2.0; e5 models MIT.
- Integrate? **maybe** — interchangeable; pick by our own retrieval eval.

Vector store: none needed for <100k vectors — brute-force cosine over a float16 array per agent is microseconds. SQLite (+ sqlite-vec, unverified license MIT/Apache) if persistence is wanted; godot-llm's LlmDB does this.

---

## 5. Speech (only if the god speaks / listens)

### Kokoro-82M (hexgrad)
- URL: https://github.com/hexgrad/kokoro
- License: Apache 2.0 code **and** weights (~9k stars). **Dependency caveat:** uses espeak-ng (GPL-3.0) for phonemisation fallback; ship espeak-ng as a separate process/binary or replace the G2P for English with a dictionary (misaki) — get legal sign-off.
- Sizes: 82M params, <1 GB VRAM, runs real-time on CPU; ONNX and kokoro.js (browser) ports.
- Languages: EN (US/UK), ES, FR, HI, IT, JA, PT-BR, ZH; dozens of preset voices; voices can be *blended* (vector mix) to create per-character voices deterministically.
- Integrate? **yes** — default TTS on every tier. Character voice = seeded blend of preset voice embeddings + speed/pitch params stored in the agent record.

### Piper (OHF-Voice/piper1-gpl; original rhasspy/piper archived)
- URL: https://github.com/OHF-Voice/piper1-gpl
- License: active fork is **GPL-3.0**; the archived original rhasspy/piper was MIT. Each voice model has its own license (many CC-BY, some non-commercial) — check per voice. Uses espeak-ng (GPL).
- Sizes: tiny VITS voices (~20–60 MB), very fast on CPU, Raspberry-Pi-class.
- Integrate? **maybe** — only via the old MIT code + carefully chosen voices, or as an external process; Kokoro is better quality anyway.

### Chatterbox (Resemble AI) — Turbo 350M, Nano 110M, Multilingual V3 500M
- URL: https://github.com/resemble-ai/chatterbox
- License: MIT (~27k stars). Zero-shot voice cloning from reference clips; paralinguistic tags ([laugh], [cough]); 23+ languages (multilingual).
- Caveat: every output carries Resemble's **Perth neural watermark** (imperceptible; fine for a game, but note it).
- Sizes: GPU recommended (Turbo ~2–4 GB VRAM est.; unverified).
- Integrate? **yes** for high-end tier — emotive "prophet/oracle" voices; clone from *our own licensed* voice actors only.

### Coqui TTS / XTTS-v2
- URL: https://github.com/coqui-ai/TTS (company shut down early 2024; community fork idiap/coqui-ai-TTS)
- License: code MPL-2.0; **XTTS-v2 weights under the Coqui Public Model License = non-commercial**. No one can sell a commercial license now that Coqui is defunct.
- Integrate? **no**.

### F5-TTS
- URL: https://github.com/SWivid/F5-TTS
- License: code MIT, **pretrained weights CC-BY-NC** (Emilia training data). **no** for shipping.

### StyleTTS2, Parler-TTS
- StyleTTS2: MIT code; LibriTTS-trained weights carry a condition to disclose synthetic speech / speaker consent (unverified exact terms). Kokoro is a StyleTTS2-derived descendant with cleaner licensing — prefer Kokoro.
- Parler-TTS (Hugging Face): Apache 2.0; describes voice with a text prompt ("an old raspy man speaking slowly") — attractive for procedurally describing voices; heavier than Kokoro. **maybe**.

### Newer 2026 options (license caveats)
- **Voxtral TTS** (Mistral, ~3–4B, released 2026-03-26): press coverage conflicts between Apache 2.0 and CC BY-NC 4.0 for the open weights — **unverified, treat as non-commercial until confirmed**.
- **Dia** (Nari Labs, Apache 2.0 per secondary sources): dialogue TTS with non-verbal tags; GPU-heavy.
- **NVIDIA Magpie / Qwen3-TTS via ACE**: available inside NVIDIA ACE (NVIDIA Open Model License / Apache respectively — unverified).
- **Fish Audio S2, Orpheus, VibeVoice**: check licenses individually (several are research/non-commercial) — unverified.

### STT: whisper.cpp, faster-whisper, Moonshine
- whisper.cpp — https://github.com/ggml-org/whisper.cpp — MIT, ~54k stars; CPU/CUDA/Vulkan/Metal/CoreML/WASM; models 75 MB (tiny) to ~3 GB (large); Silero VAD built in; streaming example. Whisper weights are MIT. **yes**.
- faster-whisper (SYSTRAN, CTranslate2) — MIT; Python-only, good for tooling/servers. **maybe**.
- Moonshine — https://github.com/moonshine-ai/moonshine — MIT code and models (except legacy non-English non-streaming models under a non-commercial community license); designed for live streaming with very low latency, tiny models, Python/JS-WASM/iOS/Android/desktop. **yes** for push-to-talk "pray to the god" input.
- Gemma 4 E2B/E4B accept audio natively — could replace STT + intent parsing in one call (to evaluate).

Voice consistency per character: store a *voice spec* in the agent (base voice IDs + blend weights + speed + pitch + seed); regenerate deterministically; cache audio per (line text hash, voice spec). Children inherit blended parent voices; age shifts pitch/speed parameters.

---

## 6. Visuals (portraits, location art, sprites)

### FLUX.2 [klein] 4B (Black Forest Labs)
- URL: HF `black-forest-labs/FLUX.2-klein-4B`
- License: **Apache 2.0** (4B only). **Klein 9B is FLUX Non-Commercial License**; FLUX.1 [dev] / FLUX.2 [dev] are non-commercial (outputs may be used commercially per BFL terms, but *running/shipping the model inside a product is not permitted* without a license); FLUX.1 [schnell] is Apache 2.0.
- Sizes: ~13 GB VRAM at bf16 (quantized GGUF/fp8 lower — ~6–8 GB est.); sub-second on high-end GPUs; text-to-image + editing + **multi-reference** generation from one checkpoint.
- Activity / release date: 2026-01-15.
- What it would do: portraits with identity consistency via multi-reference (feed the character's previous portrait to age them), location art.
- Integrate? **yes** — best permissive option for high-end PCs / server pre-generation.

### Z-Image-Turbo (Tongyi-MAI, 6B)
- License: Apache 2.0 (secondary sources; verify on model card). ~16 GB VRAM recommended (less quantized); few-step distilled, 5–10 s on RTX 4090.
- Integrate? **maybe** — quality alternative.

### Qwen-Image (20B MMDiT) / Qwen-Image-Edit
- License: original release Apache 2.0; **reports say Qwen-Image 2.1 changed to research/evaluation-only** — check each version. Heavy (16 GB+ even quantized).
- Integrate? **maybe** for server-side pre-generation only.

### Stable Diffusion family
- **SD 1.5 / SDXL**: CreativeML **OpenRAIL-M / OpenRAIL++-M** — commercial use allowed, no revenue cap, but use-based restrictions (attachment A) must be passed on to downstream users and included with any redistribution. SDXL ~6–8 GB VRAM, 1024px in ~5–10 s on an RTX 4060 (est.).
- **SD3 / SD3.5 (Large 8B, Medium 2.5B)**: **Stability AI Community License** — free commercially only under **US$1M annual revenue**, registration and "Powered by Stability AI" notice; above that an Enterprise license.
- Anime checkpoints: **Illustrious XL** (Fair AI Public License 1.0-SD; model-as-service monetisation restricted), **NoobAI-XL** (modified license **prohibiting commercial use**, including outputs), **Pony V6** (custom terms, varies) — **avoid shipping any of them**; their training data provenance (booru scrapes) is also a reputational risk.
- Integrate? **maybe** SDXL (OpenRAIL++) for low/mid-end portraits with our own trained LoRA; **no** SD3.5 unless revenue clause is acceptable; **no** community anime models.

### PixArt-Σ, Sana (NVIDIA)
- PixArt-Σ: license of code and weights unverified in this pass — lower priority; superseded by Sana/FLUX.2 klein.
- Sana: code Apache 2.0 since Jan 2025; 0.6B/1.6B/4.8B models, very fast (Sana-0.6B ~0.9 s per 1024px on high-end GPU; Sprint 0.1 s on H100). **Weights license must be checked** — early Sana weights used an NVIDIA non-commercial license (unverified for current checkpoints). Latest: SANA-Video 2.0 5B (Aug 2026).
- Integrate? **maybe** if weight license is commercial-friendly — the speed suits mid-range PCs.

### Consistency tools
- **LoRA** (per art style; trained on our own commissioned art): the most reliable consistency method. Train one *style* LoRA, not per-character LoRAs.
- **IP-Adapter** (Tencent AI Lab): Apache 2.0 — image-prompt conditioning for SD1.5/SDXL.
- **PhotoMaker** (TencentARC): Apache 2.0 (except third-party components; base SDXL/OpenCLIP terms apply).
- **InstantID**: code Apache 2.0 but depends on **InsightFace antelopev2 face models = non-commercial research only** — **no** without a commercial InsightFace license.
- **FLUX.2 klein multi-reference / Kontext-style editing**: consistency by editing the previous portrait ("same person, 20 years older") — the cleanest approach for aging.
- Deterministic recipe: portrait = f(genome-derived appearance prompt, style LoRA, seed = hash(agent_id), reference = parent/previous portrait). Cache to disk; regenerate only at life stages.

### Pixel art / sprites
- Many pixel-art LoRAs exist (e.g. for SDXL, FLUX.1, FLUX.2-klein) with mixed licenses (Civitai uploads often inherit OpenRAIL-M or unknown data). One reported FLUX.2-klein pixel-sprite LoRA claims CC0 training data (unverified).
- Retro Diffusion (Astropulse) is a commercial tool trained on licensed pixel art — could be licensed for *pre-production* asset creation.
- Practical advice: generate at higher res then quantise to palette + grid in engine code; better still, hand-author sprite parts and let the engine compose them (genome → parts), using AI only for portraits/illustrations.

### Runtimes for images
- **stable-diffusion.cpp** — https://github.com/leejet/stable-diffusion.cpp — MIT, ~7.5k stars; GGML-based, CPU/CUDA/Vulkan/Metal/OpenCL/SYCL; supports SD1.x/2.x/SDXL/SD3.5/FLUX/FLUX.2/Qwen-Image/Z-Image, LoRA, ControlNet, PhotoMaker, IP-Adapter; C#/Rust/Python/Go bindings. **yes** — embeddable sibling of llama.cpp.
- **ComfyUI** — GPL-3.0, ~136k stars. Excellent for *our* art pipeline and pre-generation; **do not embed in a closed-source game** (GPL). Calling a separately installed ComfyUI over its API is a user-side option.
- **diffusers** (Hugging Face) — Apache 2.0; Python; server-side batch generation.
- **ONNX Runtime / DirectML / TensorRT-RTX** — Windows-native path for SDXL-class models (Olive conversion).

Local cost per image: an RTX 4060-class GPU at ~150–200 W generating an SDXL/klein image in ~5–10 s uses ~0.0003–0.0006 kWh — effectively a fraction of a cent of electricity. The real costs are *VRAM contention with the game renderer* and *latency*, so generate in the background, at life-stage changes, and cache.

---

## 7. Game-specific AI/NPC SDKs and prior art

### NVIDIA ACE + In-Game Inferencing (NVIGI) SDK
- URL: https://developer.nvidia.com/ace-for-games ; https://developer.nvidia.com/rtx/in-game-inferencing
- License: SDK components vary — Audio2Face-3D SDK MIT, A2F weights NVIDIA Open Model License, training framework Apache; NVIGI plugins include a "GPT Local GGML" (llama.cpp-based) plugin supporting Nemotron Nano models; ACE now also lists Nemotron Speech ASR and Qwen3-TTS (secondary source).
- What it does: in-process C++ inference with "CUDA in Graphics" scheduling (shares GPU with rendering more gracefully), Unreal Engine 5 plugins, Game Agent SDK.
- Shipping examples: **inZOI "Smart Zoi"** — Mistral-NeMo-Minitron **0.5B** on-device, ~1 GB VRAM, generates character thoughts/behaviour; developers tried larger models but chose 0.5B for responsiveness/performance. RTX 3060 8 GB minimum for the feature; players questioned relevance of AI thoughts. **PUBG Ally** uses Minitron-2B (secondary source).
- Strengths: GPU scheduling alongside rendering is a real, hard problem NVIDIA has solved for its hardware.
- Weaknesses / risks: NVIDIA-only acceleration; custom licenses; vendor lock-in; AMD/Intel/Mac players excluded.
- Integrate? **maybe** — as an optional NVIDIA fast path behind our own backend interface.

### Inworld AI
- Status 2026: pivoted from turnkey NPC platform to B2B voice/TTS + "Agent Runtime" infrastructure (secondary source); TTS-2 launched 2026-08-31 (~$25 per 1M chars; Flash $15). Cloud only.
- Integrate? **no** for core; possible cloud TTS vendor.

### Convai
- Cloud character platform for games/XR; plans from free to $1,199/month, priced by monthly active end users (secondary source); Unity/Unreal plugins.
- Integrate? **no** — MAU pricing is hostile to a single-player sim with many agents.

### Ubisoft NEO NPC → "Teammates"
- 2024 NEO NPC prototype (with Inworld + NVIDIA Audio2Face); Nov 2025 "Teammates" closed playtest: FPS squadmates responding to voice commands; ~80-person team, **Google Gemini (cloud)** + internal middleware on Snowdrop.
- Lesson: AAA still runs this in the cloud and keeps the LLM inside tightly scripted gameplay affordances (commands map to a fixed action set) — exactly our "LLM proposes, engine disposes" pattern.

### Unity Sentis / Inference Engine, Godot ML plugins — see section 2.

### Microsoft
- Phi models + ONNX Runtime GenAI + DirectML / Windows ML (Windows-native local inference); Muse/WHAM world model research (gameplay generation, research only). Relevant to us mainly as the Windows runtime path.

### Shipped / notable LLM games — what worked, what failed
- **Suck Up!** (Proxima, 2024 own-store launch; Steam release 2025-10-01): cloud LLM vampire-persuasion game; ~12k Steam copies est. (secondary source) but huge streaming/TikTok virality. *Worked:* the LLM is the core mechanic (persuasion), the failure mode (weird replies) is funny. *Problem:* ongoing per-token cost → token/credit system.
- **Vaudeville** (Bumblebee, 2023): LLM murder-mystery interrogation + AI voices. Polarised: players "run in circles", conversations hard to steer; published a "pre-mortem". *Lesson:* open-ended chat without engine-tracked goals/clues feels aimless — gameplay state must be explicit.
- **Whispers from the Star** (Anuttacon, 2025): voice conversation with one character; ~83% positive of ~1.6k reviews; reaction times up to ~1.5 s noted; character "too forgetful" of player-invented facts. *Lesson:* memory consistency matters more than eloquence; latency ~1 s is tolerable for a single companion.
- **inZOI Smart Zoi** (Krafton, 2025): tiny on-device model for *thoughts*, not dialogue — a good precedent for our "inner monologue" flavour text at scale.
- **Skyrim/Fallout mods — Mantella, CHIM (formerly Herika), SkyrimNet**: STT (Whisper) → LLM (cloud or local via koboldcpp/OpenRouter) → TTS (Piper/xVASynth/XTTS). Community guidance: target <0.5 s LLM time-to-first-token or conversations feel broken; huge context dumps (inventory, surroundings, bio) slow responses; persistent per-NPC memory summaries are the most-loved feature; hallucinated quests/items that don't exist frustrate players.
- **AI Dungeon / Hidden Door / Infinite Craft-style games**: cloud LLMs; recurring issues are cost, moderation, and narrative drift — mitigated by engine-held state and structured "story cards".

Common failure modes to design against: latency (>1–2 s kills conversation), cost (cloud), hallucinated state (claims about nonexistent people/items/events), persona drift over long chats, repetitiveness, prompt injection by players ("ignore your instructions…"), VRAM contention causing frame drops, and moderation of player-visible text.

---

## 8. Small learned models that are NOT LLMs (per-creature brains)

### NEAT / neuroevolution
- **SharpNEAT** — https://github.com/colgreen/sharpneat — C# (.NET 9), ~424 stars, license file in repo (MIT per project history; verify). Evolves topology + weights.
- **neat-python** — BSD-3-Clause (unverified current maintainer status). Prototyping only.
- **TensorNEAT** — https://github.com/EMI-Group/tensorneat — JAX, GPU-batched NEAT (~500x vs CPU), GECCO 2024 best paper, **GPL-3.0** (tooling only, never ship).
- What it would do: *animals* — genomes encode small recurrent nets (10–50 neurons) mapping senses→motor drives; evolution happens in-world via reproduction, exactly like artificial-life labs. Deterministic if we own the RNG and float order (use fixed-point or careful single-thread float).
- Cost: a 30-neuron net is ~1k multiply-adds per tick → 1M creatures x 1k = 1 GFLOP per tick: fine on CPU with SIMD/ECS batching at a few ticks per second, or as a GPU compute shader.

### Tiny MLP / RL policies exported to ONNX
- Train offline (PyTorch, Stable-Baselines3 MIT, CleanRL MIT, Unity ML-Agents Apache 2.0 (~20k stars, Release 23 Aug 2025)), export ONNX, run via ONNX Runtime / Unity Sentis / our own SIMD matmul (for nets this small, hand-written inference is simplest and deterministic).
- Good for: locomotion/steering, foraging heuristics, predator evasion, "learned utility weights".
- Decision transformers / sequence models: too heavy per creature; maybe for a *species-level* or faction-level strategist evaluated rarely.

### When are learned nets worth it vs hand-built cognition?
- Worth it: (1) emergent *evolution* is a feature players watch (Spore/artificial-life fantasy); (2) low-level sensorimotor control where hand-tuning is tedious; (3) cheap stylistic variation between species.
- Not worth it: high-level human decision-making (jobs, relationships, politics) — utility AI / GOAP / HTN + needs + personality traits are more controllable, debuggable, explainable to players (and moddable). Use LLMs only on top of this for language and rare novel proposals.
- Hybrid recommended: hand-built cognition for humans; evolvable small nets for animal instincts and as a *modulation layer* (e.g. a net outputs personality-weighted biases into the utility scorer).

---

## 9. Practical patterns

1. **Model routing by importance and moment**
   - Tier 0 (everyone, every tick): no model — utility AI + templates.
   - Tier 1 (barks, thoughts, names): 0.5–2B model or pre-generated banks; ~20–60 tokens.
   - Tier 2 (dialogue with notable agents, reflections): 4–9B local, or Haiku/Sonnet in cloud mode.
   - Tier 3 (chronicles, myths, era summaries, key scenes): best available (9–14B / MoE local, or Sonnet/Opus via Batch API).
   - Importance score = f(fame, player attention, proximity to camera, plot relevance, kinship with god's chosen).
2. **Caching**: (a) output cache keyed by hash(prompt template id + compact state digest) — identical situations reuse lines; (b) KV/prefix cache: keep the system prompt + world bible as a fixed prefix so llama.cpp slot prompt cache or vLLM/SGLang prefix caching skips reprocessing; save per-character KV state to disk for long conversations (llama.cpp supports state save/load); (c) cloud prompt caching.
3. **Batching during fast-forward / overnight**: queue "reflection jobs" (notable agent X: summarise last season's events into 3 beliefs) and run them in parallel slots (llama.cpp `--parallel N` with continuous batching) at low priority or when the game is paused/minimised; cloud mode uses the Batch API. Results are *applied* only at the next deterministic sync point (see validation boundary).
4. **Determinism**: the simulation must never depend on model output timing. Pattern: LLM jobs are requested at tick T, results are queued, and applied at a fixed later tick (T+k) *if present*, otherwise a template/utility fallback is used. Store accepted model outputs in the save file (event log) so replays and multiplayer lockstep re-use the recorded output rather than re-generating. Fixed seed + temperature 0 is helpful but not sufficient (GPU kernels/batch composition change logits), so **record, don't recompute**.
5. **Guarding against hallucinated state**: (a) prompt with an explicit fact sheet; (b) constrained decoding with live enums for IDs/verbs; (c) engine-side validation of every proposal (exists? reachable? allowed by traits/laws/physics? cost affordable?); (d) post-hoc fact check of free text for named entities — unknown names are either stripped or *promoted to rumour/myth* objects (a nice in-fiction treatment: villagers can be wrong!).
6. **LoRA fine-tuning**: after building template content and collecting LLM outputs we like (curated with a big cloud model as teacher), fine-tune a 2–4B model with LoRA (Unsloth/PEFT/axolotl; Apache/MIT-type licenses — verify) on our dialogue style, JSON action formats and world vocabulary. llama.cpp/ORT GenAI/vLLM all hot-load LoRA adapters, so one base model can carry several adapters (dialogue, chronicle, naming). Keep training data 100% owned/generated to keep licensing clean (check the cloud provider's terms on using outputs to train models).
7. **Offline / no-GPU fallback**: every LLM feature has a template/grammar fallback (Tracery-style grammars, Markov name generators per language, hand-written bark tables keyed by needs/emotions). The game must be fully playable with the AI toggle off; the toggle changes flavour, not mechanics.
8. **GPU co-existence**: cap LLM VRAM budget per tier; schedule LLM work when frame budget allows; prefer CPU for tiny models on low-end; expose a "AI quality" slider (off / templates+tiny / balanced / max).
9. **Prompt-injection hardening** (if players type to agents): player text is placed in a quoted, tagged field; the model's output is still schema-constrained and engine-validated, so injection can at worst produce odd *dialogue*, never illegal *actions*.
10. **Evaluation harness**: a fixed set of scenarios with expected constraints; run every candidate model/quant/LoRA through it; score schema validity (should be 100% with grammars), semantic validity rate (engine acceptance), persona consistency and latency.

---

## Recommended AI model stack for our game

### Tiered table

| Tier | Dialogue model | Narration / chronicle model | Embedding | Constrained decoding | TTS | Image model | Runtime | Expected latency | VRAM / RAM for AI |
|---|---|---|---|---|---|---|---|---|---|
| **No-GPU fallback** (any PC, AI "templates" mode, or CPU-tiny) | Templates + bark banks; optional Qwen3.5-0.8B / Granite-4.0 350M on CPU | Template grammars; optional Qwen3.5-2B on CPU in background | Model2Vec potion-8M | GBNF in llama.cpp | Kokoro-82M on CPU (optional) | None — pre-rendered/composed sprite parts + pre-generated portrait library | llama.cpp CPU | barks 0.3–1 s; background narration minutes (async) | 0 VRAM; ~1–2 GB RAM |
| **Low-end PC** (6–8 GB VRAM, e.g. RTX 3060/4060, RX 7600) | Qwen3.5-4B or Gemma 4 E4B Q4_K_M (+ our dialogue LoRA) | same model, async | potion-32M or all-MiniLM-L6 | GBNF / JSON-schema → grammar | Kokoro-82M | SDXL + style LoRA via stable-diffusion.cpp, generated rarely and cached (or off) | llama.cpp (Vulkan/CUDA) + sd.cpp | first token 0.2–0.5 s; 30–60 tok/s; a 40-token line ~1–1.5 s | LLM ~3–3.5 GB; image gen only when LLM idle |
| **Mid PC** (12–16 GB VRAM) | Qwen3.5-9B or Gemma 4 12B Q4_K_M | Qwen3.5-9B / Ministral 3 14B | Qwen3-Embedding-0.6B | GBNF or llguidance | Kokoro (+ Chatterbox-Nano for key characters) | FLUX.2 klein 4B (quantized) multi-reference portraits | llama.cpp CUDA/Vulkan/Metal; sd.cpp | first token ~0.3 s; 40–70 tok/s | LLM 6–9 GB; image 6–8 GB (time-shared) |
| **High-end PC** (24 GB+ VRAM or 64 GB unified memory) | Qwen3.6-35B-A3B or Gemma 4 26B-A4B (MoE) | same MoE; batch reflections in parallel slots | Qwen3-Embedding-0.6B | llguidance / GBNF | Chatterbox-Turbo/Multilingual (cloned from licensed VA) | FLUX.2 klein 4B bf16 | llama.cpp (or mistral.rs) with `--parallel` slots | first token ~0.2 s; 60–100+ tok/s single stream; high aggregate with batching | 20–24 GB |
| **Cloud** (opt-in, BYO key or subscription) | Claude Haiku 4.5 (barks) / Claude Sonnet 5.5 (dialogue) | Claude Sonnet 5.5 or Claude Opus 5.5 via **Batch API** | local embeddings still | Structured outputs / strict tools | Local Kokoro or a cloud TTS vendor | Server-side FLUX.2 klein / Z-Image (our server) | Anthropic API; own vLLM/SGLang server for open models | 0.5–3 s interactive; batch: minutes–hours | 0 local |
| **Browser build** | Qwen3.5-0.8B/2B or SmolLM3-3B via WebLLM | same | transformers.js MiniLM / potion | XGrammar (WebLLM JSON schema) | kokoro.js | none / pre-generated | WebLLM (WebGPU), transformers.js | first token 0.5–2 s; 10–40 tok/s | 1–3 GB GPU memory; multi-GB download (cached) |

### Per-in-game-day LLM call budget (local mid PC; scale ×0.3 low-end, ×3 high-end)
Assume an in-game day lasts ~2 real minutes at 1x. Notable agents ≈ 200 of a 100k–1M population.

| Speed | Interactive dialogue (player-initiated) | Barks/thoughts (on-camera only) | Reflections (notable agents) | Proposals (novel plans) | Chronicle / myth | Names/words | Total generated tokens/day |
|---|---|---|---|---|---|---|---|
| **1x** (~2 min/day) | as many as player asks (~5–10 turns, 80 tok each) | ≤ 60 (cache hits first; 30 tok each) | ~20 (150 tok each) | ~10 (JSON, 60 tok) | 1 daily entry (300 tok) | ~10 (batched, 10 tok) | ~7–9k tokens (~2.5 min of single-stream GPU time at 50 tok/s; with 4–8 parallel slots it fits inside the 2-minute day) |
| **100x** (~1.2 s/day) | none (fast-forward suspends chat) | 0 (no camera dwell) | ~1 per day per *top-20* agent, batched; most skipped | ~1–2 (only for rulers/prophets) | 1 line per day; weekly summary 300 tok | from pre-generated pools; refill in batch | ~300–600 tokens/day (≈ budget-limited: what one batch slot can do in 1.2 s) |
| **1000x** (~0.12 s/day) | none | 0 | none per day; per-*season* digest for top-10 agents | none — engine utility AI only | per-*year* chronicle paragraph, generated in background after the fact | pools only | ~0–100 tokens/day amortised; LLM effectively a background historian catching up on a queue |

Rule: the LLM job queue has a fixed tokens-per-real-second budget (e.g. 70% of measured throughput); at high speed, jobs are coalesced (reflect on a season, not a day) or dropped by priority. **Simulation speed never waits for the LLM.**

### The validation boundary
1. **Engine selects** who gets a model call and builds a compact *fact digest* (IDs, needs, relationships, recent salient events, allowed verbs) — the model only sees engine truth.
2. **LLM output schema** (per job type), enforced by constrained decoding:
   - `Dialogue`: { speaker_id (enum), line (string ≤ N chars), emotion (enum), mentions[] (enum of known IDs) }
   - `Proposal`: { actor_id (enum = requester), verb (enum of currently allowed verbs), target_id (enum of visible entities | null), params (verb-specific typed object), justification (short string) }
   - `Reflection`: { beliefs[] { about_id (enum), stance (enum), strength (0–1) }, summary (string) }
   - `Chronicle` / `Myth`: { text, referenced_event_ids[] (enum of real events) }
3. **Engine validation** (deterministic, no model): existence, reachability, permissions (laws, traits, age, role), resource costs, cooldowns, physics; reject → fallback utility action; partially accept → clamp params. Free text is scanned for unknown proper nouns → stripped or recorded as rumour.
4. **Simulation resolves** the accepted proposal like any player/AI action through the same action system, at a deterministic tick; the accepted output is written to the event log/save for replay.
5. **Feedback**: rejection reasons are logged (for evals/LoRA data), never retried in a loop at runtime beyond one cheap re-ask.

### Licensing checklist (ship-blockers first)
- [ ] Every shipped weight file: license text included, NOTICE/attribution kept; record source URL + hash + license version in a manifest.
- [ ] Prefer **Apache 2.0 / MIT** weights: Qwen3.5/3.6, Gemma 4 (not Gemma 3/EmbeddingGemma), Ministral 3, Phi-4, SmolLM3, Olmo 3, Granite 4, gpt-oss, Qwen3-Embedding, MiniLM, Model2Vec, BGE (MIT), Kokoro, Chatterbox, whisper/Moonshine (English/streaming), FLUX.2 klein **4B**, FLUX.1 schnell, IP-Adapter, PhotoMaker.
- [ ] **Avoid / legal review**: Llama Community License ("Built with Llama", AUP pass-through, naming rules), Gemma Terms of Use (Gemma ≤3, EmbeddingGemma), NVIDIA Open Model License (guardrail clause), LFM Open License ($10M revenue cliff), Stability Community License ($1M revenue cliff, SD3.x), CreativeML OpenRAIL-M/++ (use restrictions must be passed to users), Tencent Hunyuan licences (territory exclusions).
- [ ] **Do not ship**: XTTS-v2 (CPML NC), F5-TTS weights (CC-BY-NC), FLUX dev / klein 9B (non-commercial), NoobAI-XL, InsightFace models (InstantID), unverified community finetunes/merges, Voxtral TTS until license confirmed, non-English legacy Moonshine models.
- [ ] Runtime licenses: MIT/Apache fine (llama.cpp, ORT, mistral.rs, candle, WebLLM, transformers.js, sd.cpp, whisper.cpp). **GPL**: ComfyUI, piper1-gpl, espeak-ng, TensorNEAT — keep out of the shipped binary or isolate as separate user-installed process after legal advice. **EUPL**: NobodyWho — OK to use, don't make proprietary forks. **LM Studio app**: not redistributable.
- [ ] Voice cloning only from voice actors under contract with explicit AI-synthesis consent; document it.
- [ ] Platform-holder rules (Steam AI disclosure for pre-generated and live-generated content; console certification for on-device generation; age ratings with live-generated text) — disclose and add moderation filters for player-visible text.
- [ ] Cloud: Anthropic commercial terms + usage policy; check terms on using outputs to train our LoRA models.

### Open questions / risks
1. **Role-play quality of 4B-class models** with our LoRA — must be measured on our own eval; if insufficient, low-end tier becomes "thoughts + barks only".
2. **VRAM contention** with the renderer on 8 GB cards; may need CPU-only LLM on low-end and to disable on-device image generation entirely there.
3. **Determinism vs. model nondeterminism** — solved by record-and-replay; multiplayer lockstep must sync model outputs as events.
4. **Download size**: a 4B Q4 model is ~2.5 GB, plus TTS/images — optional DLC-style downloads per tier.
5. **License drift**: vendors change licenses between versions (Gemma moved to Apache; Qwen-Image 2.1 reportedly moved away from Apache; Meta's plans unclear). Pin exact versions and archive license texts.
6. **Unverified items** in this doc (primary pages blocked during research): Meta "Muse Glimmer" Apache release, Voxtral TTS license, Sana weight license, Z-Image license, Chatterbox VRAM, SharpNEAT license text, Gemma 4 12B date.
7. **Content safety**: small local models can produce offensive text; we need a lightweight local classifier or wordlist filter and an in-fiction way to handle refusals.
8. **Player expectations**: open-ended chat can feel aimless (Vaudeville) — every conversation should touch engine-visible goals (gifts, quests, beliefs, relationships) so talk has consequences.
9. **Browser feasibility**: WebGPU coverage and multi-GB downloads may make the browser build "templates-only + tiny model".
10. **Energy/thermals on laptops** during long fast-forward batching — expose a power-saving cap.
