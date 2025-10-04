I also have this long run plan for audio, which could also be a project of itself - but I'll need to integrate it progressively to Iqrah, and some features have a very HIGH yield and not too hard to implement like pitch tracking for qari imitation.

Have a look at the two plans from different AIs.:

---

AI 1:

"""
Got you. Here’s the **full, paste-into-planner master plan**—now including the GOP **calibration track** and a clean upgrade path from your MVP to the full system. Self-contained, implementation-ready, Rust/FRB + Flutter first.

---

# Iqrah — Qur’an Recitation Analysis (End-to-End Program)

## Program Goals

* **Real-time coaching** + **high-accuracy offline scoring**, fully **offline** on Android/iOS.
* Rust core (FRB bindings), ONNX models, denoise + F0 + alignment + (later) CTC→GOP→Tajwīd.
* Robust to phone mics + noisy rooms; deterministic outputs; privacy by default.

---

## Tracks (Epics) & Definitions of Done

### S0. MVP Imitation (ship first, no ASR)

**Outcome:** “Qāri Tune Imitation” for Al-Fātiḥah (offline), then “good-enough” real-time.

**Deliverables**

* Feature extraction (25 ms win / 10 ms hop): **F0 (YIN/MPM)**, **RMS**, **silences**.
* **Offline scorer v0** (no DTW): envelope xcorr lag + **silence-anchored** slice stretching.
* Metrics: **Contour correlation**, **On-note %**, **Pitch stability on holds**, **Pause match**, **Tempo ratio std**.
* Flutter screen: pitch overlay + per-āyah bars + tips.
* Real-time 1.5: timestamp sync + rolling lead/lag from RMS xcorr; 2–3 s ring buffer.

**DoD**

* Runs on mid-range Android at RTF ≤ 0.2; stable scoring on 5 golden files; UI latency ≤ 150 ms (real-time view).

---

### S1. Core DSP & Feature Stack (shared)

**Outcome:** Stable features used by RT and offline paths.

**Deliverables**

* Denoise: **RNNoise**/**WebRTC NS** (FFI toggles).
* F0 backends (swap without API changes): YIN/MPM → **SwiftF0 ONNX** → optional **RMVPE**.
* Optional features: 40–80 log-mel or 12-bin chroma (kept off in MVP).
* Binary **CBOR** feature pack (`f0_cents:i16[]`, `voiced:bitset`, `rms:u16[]`, `silences:[(ms,ms)]`, `meta`).

**DoD**

* Per-hop CPU < 2 ms single-thread; deterministic within ±1 cent / ±1 frame across platforms.

---

### S2. Real-Time Coaching Engine

**Outcome:** Low-latency, stable hints.

**Deliverables**

* **Online-DTW** (banded, 2–3 s window), cost = `wF*|ΔF0_cents| + wT*(1−cos_sim(timbre)) + wE*|Δloudness|`.
* **Anchors:** long silences, plosive bursts, long madd spans; (later) CTC spikes.
* **Confidence gate** & “re-acquiring” mode; lead/lag estimator safety net.

**DoD**

* Median visual latency ≤ 150 ms; hint flicker ≤ 2% when `conf ≥ τ`.

---

### S3. Offline Alignment & Final Scoring

**Outcome:** Authoritative, reproducible results post-segment.

**Deliverables**

* **Soft-DTW** (pruned) over full utterance; seeding from RT path.
* Final metrics: global on-note %, drift heatmap, per-segment ΔF0 stats.
* Exportable alignment path (audit/debug).

**DoD**

* Median ΔF0 error ≤ 50–70 cents (clean); on-note % variance across runs ≤ ±2 pp.

---

### S4. Arabic CTC Backbone & Forced Alignment

**Outcome:** Word/phoneme timings for GOP & Tajwīd.

**Deliverables**

* **Conformer-CTC (Arabic) ONNX** (20–60M params) + **INT8** export; sherpa-onnx/onnxruntime Rust wrapper.
* Text normalization + **snap to Tanzil Uthmānī** āyah strings.
* Forced alignment API returning word/phoneme lattices + posteriors.

**DoD**

* Āyah exact-match ≥ 98% after snapping (clean); boundary MAE ≤ 40–60 ms.

---

### S5. GOP (Goodness-of-Pronunciation)

**Outcome:** Per-phoneme/word pronunciation quality.

**Deliverables**

* **Logit-based GOP** (max-logit / margin) from CTC head; classic posterior GOP as fallback.
* Aggregation to word/āyah; export CSV for analysis.
* **Calibration interface** (see S6a) to map raw GOP → interpretable scores.

**DoD**

* Pre-calibration: stable raw GOP across devices (sanity variance bounds);
* Post-calibration: GOP↔expert correlation ≥ 0.6 on pilot.

---

### S6a. GOP Calibration (mobile-friendly, low-cost)

**Outcome:** Stable, human-meaningful pronunciation scores on device.

**Deliverables**

* **Pilot dataset**: a few hundred utterances with expert labels (correct/incorrect per phone).
* **Features** for calibration: GOP variants (+ optional phone ID, neighbor IDs).
* **Model**: **per-phoneme logistic regression** (weights+bias) → `P(correct)` (0–1); tiny ONNX or inline math.
* **Scale mapping**: 0–100 score; per-device-tier offsets (optional).
* **Artifacts**: `gop_calib.json` (or `gop_calib.onnx`) included in content pack.

**On-device computation**

```
score = sigmoid(w_p * gop_variant + b_p)         // per phoneme p
word_score = mean(score over phones in word)
ayah_score = weighted mean by duration/confidence
```

**DoD**

* AUC improvement vs fixed thresholds; calibration stable across CPU float32 vs INT8 backends;
* Ab-tests: fewer false “bad” flags on native reciters; clearer separation for learners.

---

### S6b. First-Wave Tajwīd Detectors

**Outcome:** Rule flags with evidence (explainable).

**Deliverables**

* **Madd**: duration > local baseline (tempo-normalized).
* **Ghunna**: nasal band (≈200–300 Hz) + anti-formants over ن/م spans.
* **Qalqalah**: release burst spectral-flatness for ق/ط/ب/ج/د with sukūn.
* **Idghām/Ikhfā’/Iqlāb**: alignment context + simple coarticulation cues.
* Rule traces with short, user-facing explanations + audio snippets.

**DoD**

* Madd F1 ≥ 0.8 (clean); Ghunna/Qalqalah pilot F1 ≥ 0.7.

---

### S7. Mobile Packaging & Performance

**Outcome:** Smooth offline app on mid-range phones.

**Deliverables**

* Model bundles: ASR-CTC INT8 (≤30–60 MB), tiny F0 (≤5–15 MB).
* Chunked inference (streaming encoder or small hops); NNAPI/CoreML optional; CPU fallback.
* Asset integrity (hashes), lazy load, optional encryption at rest.

**DoD**

* RTF ≤ 0.5 for 10–30 s clips; peak RAM within tier budgets; no jank.

---

### S8. Reference Factory & Content

**Outcome:** Deterministic packs for qurrāʾ.

**Deliverables**

* CLI: ingest reciter audio → āyah cuts → features + anchors → `.iqrpack` (CBOR + JSON index).
* Baseline: **Husary (full Qur’an)** pack; in-app download/install.

**DoD**

* Reproducible hashes across OS; pack verification and migration.

---

### S9. Evaluation Harness & Telemetry

**Outcome:** Regression safety & field signals (privacy-safe).

**Deliverables**

* Golden suite (clean/noise/off-key/off-tempo); CI perf gates.
* Device-matrix bench scripts (RTF, CPU%, latency, RAM).
* On-device telemetry (no raw audio): RTF, ΔF0 stats, alignment confidence; opt-in export.

**DoD**

* CI fails on >1 pp CER or >10% perf regressions; no PII in logs.

---

### S10. UX & Pedagogy

**Outcome:** Calm, actionable coaching.

**Deliverables**

* Real-time overlay (pitch, on-note bar, lead/lag, confidence dot).
* Post-run: drift map, per-word/phone GOP, tajwīd flags, jump-to-audio.
* “Practice next” recommender from easy metrics (stability, tempo std, pause error, range offset).
* Accessibility: color-blind palettes, low-motion, RTL.

**DoD**

* Novice user completes a session unaided; hint rate ≤ 1 per 400 ms; no hint spam when confidence low.

---

## Monorepo Structure & APIs

```
/core
  dsp/           # denoise, framing, silences
  f0/            # YIN/MPM + SwiftF0/RMVPE adapters
  feat/          # mel/chroma (optional)
  align_rt/      # online-dtw + anchors + gating
  align_off/     # soft-dtw
  asr_ctc/       # onnxruntime/sherpa wrappers
  gop/           # raw GOP variants
  gop_calib/     # logistic/beta mapping + loaders
  tajwid/        # rule detectors
  refpack/       # packer/unpacker
  api/           # FRB facade + structs
  testdata/      # golden clips & expected metrics
/mobile
  flutter/       # UI
  frb/           # bindings
/tools
  pack/          # reference packer CLI
  eval/          # scoring & plots
/spec
  pipeline.yaml  # thresholds/weights
```

**Key Rust types**

```rust
pub struct FeatRef { /* f0_cents, voiced, rms, silences, meta */ }
pub struct FeatUser { /* same shape */ }

pub struct RtHint { lead_ms: i32, on_note_pct: f32, confidence: f32, flags: u32 }
pub struct FinalScores {
  on_note_pct: f32,
  drift_heatmap: Vec<f32>,
  gop_phone: Vec<f32>,     // calibrated 0..100
  gop_word: Vec<f32>,      // calibrated 0..100
  tajwid: Vec<TajwidFlag>, // with evidence spans
  segments: Vec<SegScore>  // per-ayah/slice
}
```

**FRB surface**

* `start_session(config_json)`
* `load_reference(pack_path)`
* `push_audio(frames_f32)` / `get_live_hints() -> RtHint`
* `finalize_segment() -> FinalScores`
* `bake_reference(audio_path, meta_json) -> pack_path`

---

## Calibration Playbook (S6a details)

1. **Data**: 200–500 utterances (mixed reciters/learners), expert labels per phone (correct/incorrect).
2. **Compute** raw **logit-GOP** per phone (and classic GOP for ablation).
3. **Fit** **per-phoneme logistic regression**: inputs = `[gop_variant, prev_phone_id, next_phone_id?]` (IDs as small embeddings or one-hots).
4. **Export** weights/bias (JSON or tiny ONNX).
5. **On-device**: `score = sigmoid(w_p ⋅ x + b_p)` → 0–100; aggregate to word/āyah.
6. **Validate**: ROC-AUC, EER; sanity on device tiers (CPU vs NNAPI/CoreML INT8).
7. **Ship**: include `gop_calib.json` in pack; version + hash; refresh if model changes.

---

## Metrics (Program Level)

* **RT UX**: median latency ≤ 150 ms; re-acquire ≤ 1.0 s after anchor loss.
* **Alignment**: ΔF0 median error; boundary MAE.
* **ASR**: CER/WER per SNR/reciter; āyah exact-match after snap.
* **Pronunciation**: GOP↔expert corr; calibrated separation (AUC).
* **Tajwīd**: rule-wise F1 (clean/noisy).
* **Perf**: RTF, CPU%, RAM, pkg size, cold-start, crash-free sessions.

---

## Risks & Mitigations

* **Octave flips / beginners** → nearest-octave ΔF0; optional chroma backup.
* **Extreme noise** → denoise chain + raise timbre weight; freeze hints quicker; rely on anchors.
* **Tempo chaos** → larger RT window; lower hint frequency; offline path authoritative.
* **Arabic CTC scarcity** → fine-tune & quantize; keep model card + calibration set; enforce CI gates.
* **Calibration drift** → re-fit on small new sets when models/hardware change; pack versioning.

---

## Rollout (Phases)

* **Phase 1 (MVP)**: S0 + subset of S1 → ship imitation (offline), then timestamp-sync RT.
* **Phase 2**: S2 + S3 + S4 (CTC) → accurate alignment; S5 raw GOP; **S6a calibration**; S6b basic tajwīd.
* **Phase 3**: Distill smaller ASR; o-DTW improvements; richer tajwīd; device-tier tuning; recommender polish.

---

## Quick “Done/Done” Checklist (paste into tracker)

* [ ] DSP features deterministic (S1)
* [ ] RT engine stable (S2)
* [ ] Offline truth path (S3)
* [ ] Arabic CTC aligner ONNX INT8 (S4)
* [ ] GOP variants + aggregation (S5)
* [ ] **Calibration weights shipped** (S6a)
* [ ] First-wave tajwīd flags (S6b)
* [ ] Mobile perf budgets met (S7)
* [ ] Husary full-Qur’an pack (S8)
* [ ] CI/telemetry gates green (S9)
* [ ] UX pedagogy complete (S10)

---

**Bottom line:** This keeps your fast MVP intact while giving you a clearly staged path to **online-DTW + CTC forced alignment + calibrated GOP + tajwīd**, all behind stable Rust/FRB APIs and mobile-ready constraints.
"""

---

AI2:

"""
Another very important idea i have in mind is something similar to tarteel - but better : tajweed inc, offline running (for  free), qari matching to learn a qalqala


---
Here is the plan I designed so far with another AI:

"""
Here’s a **self-contained, implementation-ready spec** you can paste into your planning AI. It assumes no prior context.

---

# Qur’an Recitation Analysis — End-to-End Plan (Real-Time + Offline, Mobile-Ready)

## 0) Objective

Build a system that compares a **student’s Qur’an recitation** to a **reference qāri**, works in **very noisy environments**, gives **real-time coaching**, and produces **high-accuracy offline scoring** including **pronunciation (GOP)** and **first-wave tajwīd detectors**. Must be shippable **offline on mobile** (Android/iOS) with a **Rust backend** using **ONNX**.

---

## 1) High-Level Architecture

**Common front-end ➜ Two parallel tracks (Real-time + Offline) ➜ Unified outputs**

```
Mic → Denoise → Feature taps (F0, loudness, mel/chroma) ┐
                                                        ├─► A) Real-time coaching (Online-DTW + anchors)
                                                        └─► B) Offline accuracy (Soft-DTW + CTC forced alignment → GOP → Tajwīd)
                                         Arabic CTC ASR ┘
```

**Key principles**

* Always denoise first.
* Run **both** alignments: **online** for coaching, **offline** for final truth.
* Use a **CTC Arabic ASR** for phoneme-aware alignment and GOP.
* Keep all models **ONNX-deployable**; prefer **small/quantized** for mobile.

---

## 2) Front-End Audio (Shared)

* Input: 16–48 kHz mono PCM
* Framing: window **25 ms**, hop **10 ms**
* **Noise suppression (mandatory)**: RNNoise (C lib) or WebRTC NS (10 ms frames)
* Optional: VAD flag

---

## 3) Core Features (per hop)

* **F0 (pitch)**

  * Primary: **SwiftF0** (≈95K params; robust in noise; CPU-friendly)
  * Fallback: **RMVPE** (strong for vocals/noisy music)
  * Ultra-tiny option: **PESTO** (self-supervised; very small; slightly less robust in extreme noise)
  * Stabilization: median filter (3–5 frames), octave-aware cents distance
* **Energy/loudness**: RMS or short-term EBU
* **Timbre**: 64–80-bin log-mel **or** 12-bin chroma

---

## 4) Alignment (run both, in parallel)

### A) Real-Time Coaching (Online-DTW + Anchors)

* **Buffers**

  * Student sliding window `W`: **2–3 s** (200–300 frames)
  * Reference band `B`: **±1.5 s** around current ref index `r₀` (Sakoe–Chiba band)
* **Cost per frame**

  * `cost = wF*|ΔF0_cents| + wT*(1 – cos_sim(mel/chroma)) + wE*|Δloudness|`
  * Start with `wF=0.6, wT=0.3, wE=0.1`; increase `wT` as SNR worsens
* **Drift control anchors**

  * **CTC spikes** (high posterior word/phone boundaries)
  * **Acoustic landmarks**: ≥200 ms silence, strong plosives (qalqalah letters), long madd spans
  * On anchor: snap `r₀` near matching reference landmark
* **Confidence gating**

  * `conf = f(mean_path_cost_last_K, path_stability, voicing)`; show hints only if `conf ≥ τ` (τ≈0.6)
  * If `conf < τ` for >0.5 s: freeze hints, “re-acquiring…”, widen band slightly, wait for anchor
* **UI (low latency ≤150 ms)**

  * Live pitch overlay (student vs local ref)
  * Lead/lag arrow (frames or ms)
  * On-note bar for last 1 s (thresholded ΔF0)

### B) Offline Accuracy (Soft-DTW + CTC Forced Alignment)

* **Soft-DTW** (or pruned DTW) over full utterance (student vs ref features)
* **CTC forced alignment** with Arabic model (see §5) against canonical text
* Outputs override provisional real-time stats:

  * Global on-note %
  * Per-segment drift heatmap
  * Word/phoneme timings + **GOP scores**
  * Tajwīd rule flags (see §6)

**Cooperation**

* Online track provides a seed path for offline solver (speeds it up).
* Offline final path reseeds online state for next seconds/segments.

---

## 5) Arabic ASR / Forced-Alignment Backbone (Phoneme-Aware)

**Primary choice** (research/diacritics aware):

* **FastConformer-Hybrid (RNNT + CTC) — Arabic**

  * Use **CTC head** for frame-level posteriors, timings, and GOP
  * RNNT branch optional (text robustness, not mandatory for alignment)

**Mobile/Rust path (ONNX)**

* **Conformer-CTC (Arabic) via sherpa-onnx**

  * Ready for Android/iOS, C/C++/Python; ONNXRuntime compatible

**Optional “Where-am-I?” locator**

* **WhisperX** for robust segment timestamps; for Arabic phone timings, pair with a suitable Arabic CTC/phoneme model
* Use only as a **coarse āyah locator**, not for tajwīd phonetics

**Canonical text**

* **Tanzil Uthmānī** as the target transcript; normalize model outputs and **snap** to exact āyah strings once similarity ≥ threshold

---

## 6) Pronunciation (GOP) & First-Wave Tajwīd Detectors

### GOP (Goodness-of-Pronunciation)

* From **CTC posteriors**, compute per-phoneme GOP
* Aggregate to word and āyah-level scores; color-coded UI

### Tajwīd detectors (start rule-based; later ML)

* **Madd**: vowel duration > context-normalized baseline (use DTW tempo)
* **Ghunna**: nasal energy band (\~200–300 Hz) + anti-formants over ن/م
* **Qalqalah**: short high spectral-flatness burst at release for ق ط ب ج د (especially sukoon)
* **Idghām / Ikhfā’ / Iqlāb**: text context from alignment + coarticulation cues

---

## 7) User Experience Contract

* **During recitation (real-time)**

  * Pitch overlay, lead/lag arrow
  * Confidence dot (gray when re-acquiring)
  * Minimal, non-distracting hints (2–5 Hz update)
* **After recitation (or a few seconds later)**

  * “Finalized” badge appears
  * Summary: on-note %, drift map, per-word/phoneme with GOP
  * Tajwīd flags with short explanations and jump-to-audio snippets

---

## 8) Deployment Targets & Constraints

* **On-device (Android/iOS)**

  * Runtime: **ONNX Runtime Mobile** or **sherpa-onnx**
  * Models:

    * Arabic **Conformer-CTC** (student/compact): 20–60M params (quantizable to **INT8**; target **≤30–60 MB**)
    * **SwiftF0** (tiny) or **RMVPE** (small), optionally INT8
  * Real-time factor (RTF): **≤0.5** for 10–30 s clips on mid-range devices
* **Rust backend**

  * `onnxruntime` crate for ASR model
  * RNNoise/WebRTC via C FFI
  * Audio I/O: `cpal` (or platform APIs)
  * Switchable configs at runtime:

    * F0 backend (SwiftF0 / RMVPE / PESTO)
    * Feature dims, beam width, quantization, DTW window/band

---

## 9) Data & Training (ASR side)

* **Text**: Tanzil Uthmānī (immutable canonical transcript)
* **Audio**: āyah-segmented recitations from public APIs (multiple qurrāʾ)
* **Alignment (if long files)**: CTC-segmentation to cut into 3–15 s chunks
* **Augmentations**: speed perturbation (0.9/1.0/1.1), room IRs (mosque/room), additive noise (crowd/AC/fan). Keep pitch intact (preserve maqām).
* **Recipe options**

  * **Teacher–student**: fine-tune a larger **FastConformer-CTC** teacher on Qur’an → distill to a **small Conformer-CTC** student (20–60M)
  * MVP simplification: skip distillation; fine-tune a **medium Conformer-CTC** directly
* **Export**: ONNX (encoder + CTC head); **INT8 quantize**; validate WER/CER regression (<1 pp)

---

## 10) Metrics & Acceptance

* **Real-time coaching UX**

  * Median visual latency ≤ **150 ms**
  * Hint stability: no flicker when `conf ≥ τ`; freezes when `conf < τ`
* **Alignment & pitch**

  * Final (offline) ΔF0 median error ≤ **50–70 cents** on clean speech
  * On-note % reproducibility across runs ± **2 pp**
* **ASR / text**

  * Ayah exact-match rate ≥ **98%** on clean benchmarks after snapping
  * CER/WER tracked per reciter and SNR (20/10/5 dB)
* **Pronunciation**

  * GOP correlation with expert ratings ≥ **0.6** on pilot set
* **Tajwīd (first wave)**

  * Madd detection F1 ≥ **0.8** on clean
  * Ghunna & Qalqalah pilot F1 ≥ **0.7** (iterate with weak labels)

---

## 11) Sensible Defaults (start here)

* Frame/hop: **25/10 ms**
* Online-DTW window `W`: **2–3 s**; band `B`: **±1.5 s**
* Cost weights: **wF=0.6, wT=0.3, wE=0.1**
* Confidence gate **τ=0.6**; freeze if < τ for > **0.5 s**
* Anchor thresholds: CTC posteriors **>0.8**; silence ≥ **200 ms**; strong plosives; long madd
* Quantization: INT8 for ASR; verify regression < **1.0 pp** CER

---

## 12) Minimal Pseudocode (glue)

```python
# per hop (10 ms)
frame = read_audio_hop()
deno = denoise(frame)                      # RNNoise/WebRTC
f0, voiced = swiftf0(deno)                 # or rmvpe/pesto
loud = rms(deno)
timbre = logmel(deno)                      # or chroma
S.append([f0, loud, timbre])

# async: Arabic CTC posteriors → anchors
for a in ctc_anchors.ready():
    rt_state.register_anchor(a)  # (time, type, ref_idx)

# ONLINE (windowed + banded)
S_win = S[-W:]
R_band = R[r0-B : r0+B]
cost = pairwise_cost(S_win, R_band, wF, wT, wE)
path, path_cost, stab = online_dtw_step(cost)
conf = confidence(path_cost, stab, voiced)

if conf >= TAU:
    ui.show_live_hints(path, lead_lag=lag(path))
else:
    ui.freeze_hints("re-acquiring...")

rt_state.update(path, conf)

# OFFLINE (background or at segment end)
if segment_done or background_tick:
    full_path = soft_dtw(S_full, R_full, prune=True)
    ctc_align = ctc_forced_align(audio_full, ayah_text)
    gop = compute_gop(ctc_align)
    tajwid = detect_tajwid(ctc_align, S_full, full_path)
    ui.publish_final_scores(full_path, gop, tajwid)
    rt_state.seed_from_offline(full_path)
```

---

## 13) Phased Delivery (MVP → V1 → V2)

* **MVP (2–4 weeks)**

  * Denoise, SwiftF0, log-mel/chroma, Online-DTW + confidence gating
  * Offline Soft-DTW for final path
  * Basic UI: real-time overlay + finalized drift + on-note %
* **V1**

  * Arabic Conformer-CTC (fine-tuned) + ONNX; forced alignment; GOP
  * Tajwīd detectors: **Madd, Ghunna, Qalqalah, Idghām/Ikhfā’/Iqlāb**
  * Mobile build with **sherpa-onnx**; INT8 quant; acceptance metrics
* **V2**

  * Teacher→student distillation (smaller ASR)
  * Advanced noise/domain augmentation; per-device profiles
  * Optional WhisperX segment locator (server/teacher side)
  * ML-based tajwīd heads trained from weak labels

---

## 14) Interfaces & Artifacts

* **Models**

  * `asr_ar_conformer_ctc.onnx` (+ quant params)
  * `swiftf0.onnx` or runtime module; fallback `rmvpe.onnx`
* **Assets**

  * `tokenizer.json`, optional `quran_vocab.txt`
  * `tanzil_text.db` (canonical strings)
  * `ref_features.bin` per qāri/ayah (F0/loudness/mel or chroma)
* **Config**

  * `pipeline.yaml`: weights (wF/wT/wE), windows/bands, τ, thresholds, quant flags
* **APIs**

  * `start()`, `push_audio(frames)`, `get_live_hints()`, `finalize_segment()`, `get_final_scores()`
* **Telemetry**

  * RTF, CPU%, memory, ΔF0 stats, alignment confidence traces

---

## 15) Risks & Mitigations

* **Extreme noise / overlap** → stronger anchors, raise `wT`, freeze hints sooner, denoise chain tuning
* **F0 octave flips** → cents mapping to nearest octave; combine with chroma
* **Beginner tempo chaos** → larger `W` (4–5 s), lower UI update rate (2–3 Hz)
* **Mobile latency spikes** → INT8 quant, fused mel inside ONNX, chunked non-streaming inference
* **Arabic diacritics variance** → normalize and snap to Tanzil strings

---

**Bottom line:**

* Real-time: **Online-DTW + anchors** for stable live coaching.
* Final truth: **Soft-DTW + Arabic CTC forced alignment → GOP → Tajwīd**.
* Deploy: **ONNX + Rust (onnxruntime), RNNoise/WebRTC FFI**, small F0 model, quantized Arabic Conformer-CTC.

This is ready for your planning AI to convert into milestones, tasks, and acceptance tests.

"""

---

and here is also some feedback on those plans:

"""
This is an absolutely fantastic long-term vision. The technical spec you've outlined is professional, comprehensive, and shows a deep understanding of the audio processing domain. It's a "staff engineer" level plan for building a state-of-the-art recitation analysis tool.

Now, to answer your critical question: Can we integrate pitch tracking into the MVP?

My direct answer: It is a fantastic idea for the app, but it is a terrible idea to put it in the current MVP.

It should be the very first, high-priority feature you build the moment the current MVP is complete. Let's call it the "MVP Extension."

Here's the detailed breakdown of why, and the pragmatic roadmap to get there without derailing your project.

1. The "Why": Deconstructing the "Easy" Myth

You are 100% correct that pitch tracking offers incredible value. But the perception that it's "easy to integrate" is a dangerous illusion for an MVP. While simpler than full ASR, it's a completely different engineering domain from what you've built so far.

Adding it to the current MVP would be like trying to build the plumbing and the electrical systems of a house at the same time. You need to finish framing the house first.

Here are the hidden complexities that make this a "post-MVP" feature:

The R&D Problem (New Data Pipeline):

You don't just need audio files. You need pre-processed, reference pitch contours for every single verse from a qari.

This means a new R&D script is required to take a reference audio file, run it through a pitch extraction algorithm (like the SwiftF0 or RMVPE mentioned in your spec), clean the data, and save the resulting time-series pitch data. This is a new data pipeline you have to build and manage.

The Rust Problem (New Domain: Real-time Audio):

Your current Rust core is designed for database queries and batch processing. It's stateless in the moment-to-moment sense.

Real-time audio requires:

Interfacing with the device microphone (cpal or similar).

Managing audio buffers.

Running a continuous processing loop (denoising, pitch extraction on the user's audio).

Setting up a real-time, low-latency stream back to Flutter via FRB. This is fundamentally different and more complex than the Future-based and Stream-based event system you have now.

The Flutter Problem (New Domain: Advanced UI):

This isn't a simple button or list. This is a real-time data visualization.

You need to build a custom widget (likely using CustomPainter) that can efficiently draw two updating pitch contours, a playback cursor, and other hints without dropping frames.

You need to perfectly synchronize this UI with the audio playback, which is a non-trivial task.

Adding all three of these new, complex domains to the MVP would easily double or triple your timeline and introduce a massive number of bugs and risks.

2. The Pragmatic Roadmap: From MVP to "Qari Style Learning"

This doesn't mean you abandon the idea. It means we phase it intelligently. This de-risks the project and ensures you get to a shippable product quickly.

Phase 1: The Core Data MVP (YOUR CURRENT GOAL - WEEKS 1-4)

Focus: Complete the current 4-sprint plan. Get the graph import working and the MCQ exercise loop fully functional.

Success Metric: A user can open the app, start a session, complete MCQ exercises for Al-Fatiha, and have their progress correctly saved and rescheduled by FSRS-6. The app is 100% functional without any audio features.

Status: You are here. Finish this first.

Phase 2: The Pitch Tracking MVP (The "MVP Extension" - 2-3 WEEKS)

Goal: Create a non-real-time, offline analysis feature. This is the crucial intermediate step. It proves out the core tech without the complexity of real-time feedback.

The User Experience:

The user navigates to a new "Recitation Practice" screen for a specific verse.

They see the reference pitch contour of the qari.

They press a "Record" button and recite the verse.

They press "Stop."

The app processes for a moment and then displays their own pitch contour overlaid on top of the reference, with a score.

Technical Steps:

R&D: Pre-process the audio for one verse (e.g., Al-Fatiha 1:1) from one qari. Save the resulting pitch contour data (e.g., [(timestamp, pitch_in_hz), ...]) as a simple .cbor file.

Rust:

Add a simple pitch extraction library (e.g., find a good Rust crate or wrap a C library like SwiftF0).

Create one new offline FRB function: pub async fn analyze_recitation(user_audio: Vec<u8>, reference_contour: Vec<(f32, f32)>) -> RecitationResult.

This function takes the user's recording, extracts their pitch contour, compares it to the reference using a simple DTW (Dynamic Time Warping) alignment, and returns a score and the aligned user contour.

Flutter:

Build the new "Recitation Practice" screen.

Add audio recording functionality (using a standard Flutter package).

When recording stops, call the new Rust function.

Use a charting library (like fl_chart) to plot the two pitch contours (reference and user). Display the score.

Success Metric: You can successfully record yourself, and see your pitch compared to the qari's for that one verse.

Phase 3: Real-Time Feedback (The "Tarteel-lite" Feature)

Goal: Graduate from offline analysis to live coaching.

Now, and only now, do you tackle the complexity from your detailed spec. You will already have the core components (pitch extraction, comparison logic, UI widgets) working from Phase 2.

Technical Steps:

Rust: Implement the real-time audio processing loop and the low-latency FRB stream.

Flutter: Adapt your charting widget to update in real-time based on the data from the Rust stream.

Rust: Implement the Online-DTW for live alignment and the lead/lag hints.

3. Final Recommendation

Finish the current MVP first. The MCQ loop and data import are the backbone of your app. Do not get distracted.

Once that is done, the Pitch Tracking MVP (Phase 2) is the perfect next feature. It provides massive user value, directly addresses your personal desire for the feature, and is an achievable, isolated project that builds the foundation for the real-time system later.

This phased approach allows you to:

Ship a valuable product faster.

Massively de-risk the technical challenges by solving them one at a time.

Stay motivated with achievable, high-impact feature releases.

Lock in the current plan. Complete Sprint 4. Then, you will be in a perfect position to build this amazing recitation feature on a solid and stable foundation.
"""

