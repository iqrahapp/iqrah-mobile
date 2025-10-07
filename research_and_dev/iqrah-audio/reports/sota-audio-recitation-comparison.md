# Iqrah Recitation Comparison — Phase-2 Implementation Spec

## 0) Scope & Goal

Deliver a production-ready Phase-2 comparison & scoring engine that:

* Aligns **tempo-invariant rhythm**, compares **key-invariant melody**, scores **Madd (durations)**, and assesses **pronunciation quality**.
* Returns an **overall score** with a clear breakdown and **actionable feedback** in <5 s per comparison (with precomputation).
  Key choices are grounded in Soft-DTW *divergence* (for robust, differentiable alignment), relative-pitch contours, SSL-GOP for pronunciation, and FAISS for scale. ([Proceedings of Machine Learning Research][1])

---

## 1) TL;DR — Decisions You Should Implement

* **Rhythm backbone:** Soft-DTW **divergence** (not raw Soft-DTW). Use a multi-feature stack `[onset_strength, syllable_onset_flag, normalized_time, ΔF0]`, Sakoe-Chiba band 10–15%, γ≈0.1–0.3. Consider **Drop-DTW** if student audio has breathing/outlier spikes. ([Proceedings of Machine Learning Research][1])
* **Melody backbone:** Extract F0 with **RMVPE**, convert to semitones, compute **ΔF0** (contour), z-normalize per phrase, compare with DTW/cosine. Fallback: **HPCP/chroma** + local alignment when voicing is unreliable. ([Hugging Face][2])
* **Madd durations:** Tempo-adaptive **Laplace/Huber** scoring, not fixed Gaussian. Report “expected vs held” in **counts** with CI.
* **Pronunciation:** **SSL-GOP** (logit-based GOP from wav2vec2/WavLM) on CTC spans; show **confusion buckets** for Arabic (e.g., غ↔خ, س↔ص, ذ/ز/ظ). Benchmark when possible on **QuranMB.v1**. ([docs.pytorch.org][3])
* **Fusion:** Start with weighted sum (rhythm 0.30, melody 0.20, duration 0.30, pronunciation 0.20) → then **learn weights** via ordinal regression from expert preferences.
* **Scale:** Precompute reference features/embeddings per āyah & Qārī; use **FAISS** (HNSW or IVF-PQ) to shortlist top-k, then run DTW on shortlist. ([GitHub][4])

---

## 2) Inputs (from Phase-1) & Normalization

You already produce:

* **Onsets / IOIs** (syllable timings), **pitch** (F0), **syllable grid**, **Tajweed labels**.

**Standardization for alignment:**

```text
onset_strength := z-score per clip
syllable_onset_flag ∈ {0,1}
normalized_time ∈ [0,1]  (time / total_duration)
f0_semitones := 12 * log2(F0/55 Hz)  (or any base; used relatively)
ΔF0 := first difference of f0_semitones, then z-norm per phrase
```

Downsample all feature sequences to a common length **L≈150** (5–60 s clips fit well).

---

## 3) Rhythm (Tempo-Invariant) — Method & Params

**Why:** Rhythm is your primary pedagogical signal; student pace can differ but structure should match.

**Method:** Soft-DTW **divergence** on concatenated features `[onset_strength, syllable_onset_flag, normalized_time, ΔF0_scaled]` where `ΔF0_scaled = ΔF0/100` (≈ semitone per 100 cents).

* Distance: cosine for feature channels, L2 for time; implement as a small learned blend later.
* Constraints: **Sakoe-Chiba band** 10–15% of L; slope penalty 1.1–1.3.
* γ (soft-min temperature): 0.1–0.3 works well; smaller → crisper path.
* Use the **divergence** form to remove bias and get a proper similarity measure. ([Proceedings of Machine Learning Research][1])

**Outlier handling:** If recordings have breaths/noise spikes, test **Drop-DTW**; it keeps the common signal and drops outliers while staying differentiable. ([NeurIPS Proceedings][5])

---

## 4) Melody (Key-Invariant) — Method & Params

**Why:** Reciters can shift key; we care about **contour**, not absolute pitch.

**Primary track:**

* F0 via **RMVPE** (robust on voice/music), convert to semitones, compute **ΔF0**, per-phrase z-norm.
* Compare with DTW using **cosine distance** on ΔF0.

**Fallback track:**

* Compute **HPCP/chroma** (Essentia) to gain key invariance; align locally (cross-similarity + subsequence alignment).
* Fuse ΔF0 and HPCP distances (e.g., 0.7 / 0.3) when F0 has low confidence. ([Hugging Face][2])

**Report:** pitch-shift estimate (median semitone offset), contour similarity, and note-range coverage.

---

## 5) Madd (Elongations) — Duration Scoring

**Score each Madd event** with a **tempo-adaptive Laplace** (or Huber) penalty:

```
σ = 0.15 × expected_counts × (local_tempo / global_tempo)
score_event = 100 × exp(-|actual - expected| / σ)
```

Aggregate per type (2/4/6) and overall; flag >0.5-count shortfalls as **critical**.

---

## 6) Pronunciation Quality — SSL-GOP + Confusions

**Pipeline:**

1. **CTC forced alignment** (torchaudio’s API) to get phone spans. ([docs.pytorch.org][3])
2. Compute **logit-GOP** per span: mean(logit(target) − max(logits)).
3. Bucket to **OK / mild / severe** error levels; build **confusion matrix** across Arabic sets (e.g., غ/خ, س/ص, ذ/ز/ظ).
4. Map to Tajweed feedback (e.g., ghunna duration, emphatics).

**Benchmark:** When ready, evaluate on **QuranMB.v1** (first public test set for Qur’anic mispronunciation) to calibrate and compare. ([arXiv][6])

---

## 7) Overall Scoring & Explainability

**Cold-start weights (can be tweaked):**

* rhythm 0.30, melody 0.20, durations 0.30, pronunciation 0.20.

**Learned fusion:** Fit an **ordinal regression** or tiny MLP on expert pairwise preferences with monotonic constraints per component. Return:

* overall score + **confidence** (bootstrap across segments),
* component scores,
* top 3 contributing issues (for pedagogy).

---

## 8) Real-Time Feedback UX (What to show)

* **Hierarchy:** Critical (tajweed/duration) → Timing (rhythm) → Style (melody).
* **Overlays:**

  * DTW path over onset grid (shows where timing diverges),
  * ΔF0 vs reference contour, + key shift in cents,
  * Madd bars: expected vs held (counts), with variance band,
  * Pronunciation tips: “Likely غ→خ in word ___; focus on back-of-throat placement.”

---

## 9) Scale, Latency & Caching

* **Precompute** per reference (per āyah × Qārī): onset envelope, syllable grid, ΔF0, HPCP, embeddings.
* **Index** with **FAISS** (HNSW for RAM-resident; IVF-PQ for large). Query to shortlist **k=5–10** candidates; run DTW only on shortlist, meeting <5 s compare target. ([GitHub][4])
* **Cache** by `(ayah_id, qari_id, version_hash)`, and memoize recent student feature tensors.

---

## 10) Evaluation Plan

**No ground truth (early):**

* Synthetic tests: tempo ±20%, pitch shift ±300 cents, Madd ±{−1, −0.5, +0.5} counts → verify monotonic score response.
* Bootstrap CIs over segments.

**With experts / benchmark:**

* **Spearman** correlation vs expert sub-scores (per component).
* **Inter-rater reliability**: Krippendorff’s α or ICC.
* **Ablations:** remove one component → measure drop in correlation (importance).
* **QuranMB.v1:** report per-phone F1 and overall mispronunciation metrics. ([arXiv][6])

---

## 11) Code Skeleton (modules & contracts)

```
iqrah_scoring/
  features.py         # F0 (RMVPE), ΔF0, onsets/IOIs, syllable grid, normalization
  rhythm.py           # Soft-DTW divergence (+ optional Drop-DTW), path & score
  melody.py           # ΔF0 DTW + HPCP fallback, fusion & pitch-shift estimate
  duration.py         # Madd scoring (Laplace/Huber), per-type & overall
  pronunciation.py    # CTC spans (torchaudio) + logit-GOP + confusion matrix
  fusion.py           # weighted or learned fusion (+ uncertainty via bootstrap)
  ann.py              # FAISS/HNSW or IVF-PQ index utils
  api/routes.py       # /score, /explain, /compare JSON endpoints
```

**Key signatures:**

```python
@dataclass
class FeatPack:
    onset_strength: np.ndarray   # [T]
    syll_onset_mask: np.ndarray  # [T] {0,1}
    norm_time: np.ndarray        # [T] in [0,1]
    f0_semitones: np.ndarray     # [T], NaN on unvoiced
    df0: np.ndarray              # [T], z-norm per phrase
    hpcp: np.ndarray | None      # [T,12/24]
    frame_times: np.ndarray      # [T]

def rhythm_score(student: FeatPack, ref: FeatPack, ...) -> dict
def melody_score(student: FeatPack, ref: FeatPack, ...) -> dict
def madd_score(events: list[dict]) -> dict
def gop_scores(logits: torch.Tensor, spans: list, phone_map: dict) -> dict
def overall_score(components: dict, model=None) -> dict
```

---

## 12) Defaults & Hyperparameters

* Frame rate: 50–80 Hz; resample to **L≈150** for DTW.
* Soft-DTW **divergence** γ=0.1–0.3; Sakoe-Chiba band 10–15%; slope penalty 1.1–1.3. ([Proceedings of Machine Learning Research][1])
* Feature weights (rhythm tensor): `[0.5 (onset), 0.3 (ΔF0_scaled), 0.2 (time)]`.
* Melody fusion: ΔF0 0.7, HPCP 0.3 (enable HPCP only if F0 confidence low).
* Madd σ: `0.15 × expected_counts × (local_tempo / global_tempo)`.

---

## 13) Libraries (pin & ship)

* **Alignment:** `tslearn` SoftDTW for PyTorch; `dtaidistance` (fast C DTW). Optional Drop-DTW from paper repo. ([tslearn.readthedocs.io][7])
* **Pitch:** **RMVPE** (paper + repo), keep CREPE as fallback while testing. ([Hugging Face][2])
* **Tonal features:** **Essentia** (HPCP/chroma; cover-song similarity tutorial). ([essentia.upf.edu][8])
* **Pronunciation:** `torchaudio` **CTC forced alignment** tutorial/code path. ([docs.pytorch.org][3])
* **ANN:** **FAISS** (CPU/GPU). ([GitHub][4])

---

## 14) Risks & Mitigations

* **Alignment brittleness:** always band-constrain DTW; use downsampling (L≈150); consider Drop-DTW for noisy kids’ audio. ([NeurIPS Proceedings][5])
* **Key vs contour:** never compare raw Hz; use semitones & ΔF0; only bring HPCP when F0 is unreliable. ([essentia.upf.edu][8])
* **API churn:** torchaudio FA is evolving; pin versions and keep a minimal wrapper to ease migration. ([docs.pytorch.org][3])
* **Scale:** always shortlist with FAISS before DTW; cache feature tensors.

---

## 15) 7-Day Sprint (deliverable-oriented)

**D1–2 Rhythm:** Soft-DTW **divergence** + banding on `[onset, flag, time, ΔF0]`; return path + score. ([Proceedings of Machine Learning Research][1])
**D2–3 Melody:** RMVPE → ΔF0 DTW; HPCP fallback + fusion; return pitch-shift & contour score. ([Hugging Face][2])
**D3–4 Durations:** Laplace/Huber Madd; per-type and overall; CI via bootstrap.
**D4–5 Pronunciation:** CTC spans + logit-GOP + confusion buckets; stub QuranMB loader. ([docs.pytorch.org][3])
**D5–6 Fusion & UX:** Weighted fusion + uncertainty; JSON schema + waveform/overlay endpoints.
**D6–7 Scale:** Precompute refs; FAISS index; shortlist k=5–10; perf test to <5 s.

---

## 16) Output JSON (contract)

```json
{
  "overall": 0-100, "confidence": 0-1,
  "rhythm": {"score": ..., "notes": [...], "path": [[i,j], ...]},
  "melody": {"score": ..., "pitch_shift_cents": ..., "notes": [...]},
  "durations": {"overall": ..., "by_type": {"2":...,"4":...,"6":...}},
  "pronunciation": {
    "score": ...,
    "confusions": [{"from":"\u063a","to":"\u062e","word":"...", "severity":"mild"}]
  }
}
```

---

## 17) References (key, load-bearing)

* **Soft-DTW divergence** (why & formula; preferred over raw Soft-DTW). ([Proceedings of Machine Learning Research][1])
* **Drop-DTW** (robust alignment with outlier dropping; differentiable). ([NeurIPS Proceedings][5])
* **QuranMB.v1** (first public Qur’anic mispronunciation benchmark; use for eval). ([arXiv][6])
* **RMVPE** (robust vocal pitch estimation: paper + repo). ([Hugging Face][2])
* **HPCP/chroma + local alignment** (key-invariant melodic similarity; Essentia tutorial). ([essentia.upf.edu][8])
* **torchaudio CTC forced alignment** (API/tutorial). ([docs.pytorch.org][3])
* **FAISS** (ANN at scale). ([GitHub][4])

---

### Done.

If you want, I can also hand you a compact `iqrah_scoring` starter repo skeleton in a single file layout (FastAPI route + the four scorers) — just say the word.

[1]: https://proceedings.mlr.press/v130/blondel21a/blondel21a.pdf?utm_source=chatgpt.com "Differentiable Divergences Between Time Series"
[2]: https://huggingface.co/papers/2306.15412?utm_source=chatgpt.com "RMVPE: A Robust Model for Vocal Pitch Estimation ..."
[3]: https://docs.pytorch.org/audio/main/tutorials/ctc_forced_alignment_api_tutorial.html?utm_source=chatgpt.com "CTC forced alignment API tutorial"
[4]: https://github.com/facebookresearch/faiss?utm_source=chatgpt.com "facebookresearch/faiss: A library for efficient similarity ..."
[5]: https://proceedings.neurips.cc/paper/2021/hash/729c68884bd359ade15d5f163166738a-Abstract.html?utm_source=chatgpt.com "Drop-DTW: Aligning Common Signal Between Sequences ..."
[6]: https://arxiv.org/abs/2506.07722?utm_source=chatgpt.com "Towards a Unified Benchmark for Arabic Pronunciation Assessment: Quranic Recitation as Case Study"
[7]: https://tslearn.readthedocs.io/en/stable/gen_modules/metrics/tslearn.metrics.SoftDTWLossPyTorch.html?utm_source=chatgpt.com "tslearn.metrics.SoftDTWLossPyTorch - Read the Docs"
[8]: https://essentia.upf.edu/tutorial_similarity_cover.html?utm_source=chatgpt.com "Cover Song Identification"


---

Got it—here’s a grab-and-go shortlist of **actively maintained, SOTA-friendly Python libs** for each Phase-2 component, with **current GitHub stars** (as of **Oct 7, 2025**, Europe/Paris). I’ve ordered each bucket roughly by ecosystem maturity & popularity; quick “what to use it for” notes included.

# Alignment & rhythm (tempo-invariant)

* **FAISS** — ANN shortlist before alignment; CPU/GPU, production-grade (**25.8k★**).
* **tslearn** — DTW family + **Soft-DTW (PyTorch loss)**; clean API (**2.6k★**).
* **fastdtw** — sub-quadratic DTW approximation (fast) (**1.8k★**).
* **dtaidistance** — optimized DTW (C/NumPy), windows & LB_Keogh (**1.4k★**).
* **pytorch-softdtw-cuda** — GPU Soft-DTW (autograd-friendly) (**743★**).
* **soft-dtw-divergences (google-research)** — reference impl. of **Soft-DTW divergence** (great for a proper similarity) (**229★**).
* **Drop-DTW (SamsungLabs)** — official research code for outlier-robust DTW (**0★**, still useful as a reference).

# Pitch contour & melody (key-invariant)

* **librosa** — reliable MIR utilities, **onset strength**, CQT/chroma, handy ops (**7.9k★**).
* **Essentia** — high-quality tonal features (HPCP/chroma), robust DSP (**6.2k★**).
* **CREPE** — strong neural F0 baseline (**1.5k★**).
* **torchcrepe** — PyTorch/GPU CREPE re-impl; streaming-friendly (**476★**).
* **RMVPE** — modern vocal F0 tracker; good on singing/voice (**284★**).
* **praat-parselmouth** — Praat from Python (formants/voicing sanity checks) (**2.4k★**).

# Pronunciation assessment & forced alignment (Arabic-ready building blocks)

* **🤗 Transformers** — wav2vec2 / WavLM encoders for GOP-style scoring & logits (**151k★**). ([GitHub][1])
* **torchaudio** — CTC/forced-alignment tutorials, I/O & ops; now slimmed to core strengths (**2.7k★**). ([GitHub][2])
* **ctc-segmentation** — produce stable utterance/phoneme timings from CTC logits (**341★**). ([GitHub][3])
* **Montreal Forced Aligner (MFA)** — battle-tested Kaldi aligner; good Arabic lexicon flow (**1.6k★**). ([GitHub][4])

# Vector search / scaling (precompute → shortlist → align)

* **Milvus** — feature-rich vector DB (managed or local) (**26.3k★**).
* **FAISS** — (listed above) fastest CPU/GPU ANN for shortlisting candidates (**25.8k★**).
* **hnswlib** — in-memory HNSW (great recall/latency trade-off) (**4.9k★**). ([GitHub][5])
* **Annoy** — ultra-simple, mmap-able read-only indexes (**14k★**). ([GitHub][6])

# Audio embeddings / melodic similarity (for retrieval & cross-Qari search)

* **LAION-CLAP** — pretrained audio/text embeddings for contrastive similarity (**stars not shown in captured page; repo linked**). ([GitHub][7])
* **OpenL3** — open-source audio embeddings (music/env sounds) (**stars not shown in captured page; repo linked**). ([GitHub][8])

# Evaluation & MIR metrics

* **mir_eval** — canonical MIR metric suite (segmentation, pitch, onset, etc.) (**stars not shown in captured page; repo linked**). ([GitHub][9])
* **nnAudio** — GPU-accelerated spectrogram/CQT features via 1D convs (handy for real-time viz) (**stars not shown in captured page; repo linked**). ([GitHub][10])

---

## What to pick (TL;DR)

* **Alignment**: `tslearn` (Soft-DTW loss) + `pytorch-softdtw-cuda` for GPU; optionally study `soft-dtw-divergences` for a proper distance; ANN shortlist with **FAISS**.
* **Pitch & melody**: `torchcrepe` or `CREPE` for F0; `Essentia`/`librosa` for HPCP/chroma & onset strength.
* **Pronunciation**: `transformers` (wav2vec2/WavLM) + `ctc-segmentation`; or **MFA** for traditional alignment baselines. ([GitHub][1])
* **Scaling**: `Milvus` or `FAISS` depending on deployment needs; `hnswlib`/`Annoy` for lightweight local indices. ([GitHub][5])
* **Embeddings**: `LAION-CLAP` (contrastive audio/text) or `OpenL3` for quick audio embeddings. ([GitHub][7])
* **Metrics**: `mir_eval` + your task-specific scores; `nnAudio` for fast spectrograms in UI. ([GitHub][9])

> Note: GitHub stars above are what GitHub displayed in the linked pages we fetched today; a few repos didn’t surface the number in the captured HTML, so I linked them without a count (CLAP, OpenL3, mir_eval, nnAudio). Everything listed shows recent activity and remains a safe pick for production.

[1]: https://github.com/huggingface/transformers "GitHub - huggingface/transformers:  Transformers: the model-definition framework for state-of-the-art machine learning models in text, vision, audio, and multimodal models, for both inference and training."
[2]: https://github.com/pytorch/audio "GitHub - pytorch/audio: Data manipulation and transformation for audio signal processing, powered by PyTorch"
[3]: https://github.com/lumaku/ctc-segmentation "GitHub - lumaku/ctc-segmentation: Segment an audio file and obtain utterance alignments. (Python package)"
[4]: https://github.com/MontrealCorpusTools/Montreal-Forced-Aligner "GitHub - MontrealCorpusTools/Montreal-Forced-Aligner: Command line utility for forced alignment using Kaldi"
[5]: https://github.com/nmslib/hnswlib "GitHub - nmslib/hnswlib: Header-only C++/python library for fast approximate nearest neighbors"
[6]: https://github.com/spotify/annoy "GitHub - spotify/annoy: Approximate Nearest Neighbors in C++/Python optimized for memory usage and loading/saving to disk"
[7]: https://github.com/LAION-AI/CLAP "GitHub - LAION-AI/CLAP: Contrastive Language-Audio Pretraining"
[8]: https://github.com/marl/openl3 "GitHub - marl/openl3: OpenL3: Open-source deep audio and image embeddings"
[9]: https://github.com/craffel/mir_eval "GitHub - craffel/mir_eval: Evaluation functions for music/audio information retrieval/signal processing algorithms."
[10]: https://github.com/KinWaiCheuk/nnAudio "GitHub - KinWaiCheuk/nnAudio: Audio processing by using pytorch 1D convolution network"
