# Iqrah Audio - SOTA Production Architecture v1.0
**Target Timeline:** 2025-2028 (3-year commitment)  
**Phase:** Offline E2E → Real-time → Mobile  
**Generated:** 2025-10-23

---

## Executive Summary

This document defines the **definitive production architecture** for Iqrah Audio, a state-of-the-art Quranic recitation analysis system. The design achieves:

- **90%+ accuracy** on all basic Tajweed rules (madd, ghunnah, qalqalah)
- **Comprehensive prosodic analysis** (pitch, rhythm, timbre, style)
- **Sub-500ms latency** path to real-time deployment
- **Modular architecture** suitable for AI-assisted development
- **Progressive enhancement** from offline → real-time → mobile

### Core Philosophy

1. **Modularity First:** Each component is a black box with well-defined inputs/outputs
2. **Progressive Rollout:** Basic rules → Advanced rules → Prosody → Real-time
3. **Validation at Every Step:** Test accuracy before adding complexity
4. **Pluggable Design:** Swap models/algorithms without rewriting core
5. **AI-Agent Friendly:** Clear boundaries, minimal context bleed between modules

---

## System Overview

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         USER INPUT                              │
│                    (Audio + Metadata)                           │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
         ┌───────────────────────────────────────┐
         │         PREPROCESSING LAYER           │
         │  • Audio normalization (16kHz)        │
         │  • Noise reduction (optional)         │
         │  • VAD segmentation                   │
         │  • Quality checks (SNR, clipping)     │
         └──────────────┬────────────────────────┘
                        │
                        ▼
         ┌───────────────────────────────────────┐
         │       ACOUSTIC ANALYSIS LAYER          │
         │  ┌─────────────────────────────────┐  │
         │  │ 1. Pitch Extraction             │  │
         │  │    - SwiftF0 (primary)          │  │
         │  │    - RMVPE (melodic fallback)   │  │
         │  ├─────────────────────────────────┤  │
         │  │ 2. Phoneme Alignment            │  │
         │  │    - Wav2Vec2-BERT (fine-tuned) │  │
         │  │    - CTC forced alignment       │  │
         │  ├─────────────────────────────────┤  │
         │  │ 3. Voice Quality Analysis       │  │
         │  │    - OpenSMILE eGeMAPS          │  │
         │  │    - Formant tracking           │  │
         │  │    - Vibrato/breathiness        │  │
         │  └─────────────────────────────────┘  │
         └──────────────┬────────────────────────┘
                        │
                        ▼
         ┌───────────────────────────────────────┐
         │     TAJWEED VALIDATION LAYER           │
         │  ┌─────────────────────────────────┐  │
         │  │ Rule-Based Validators           │  │
         │  │ • Madd (duration)               │  │
         │  │ • Ghunnah (formant + duration)  │  │
         │  │ • Qalqalah (burst detection)    │  │
         │  │ • Idghaam/Ikhfaa (phonetic)     │  │
         │  ├─────────────────────────────────┤  │
         │  │ Acoustic Validators             │  │
         │  │ • Neural GOP scoring            │  │
         │  │ • Spectral validation           │  │
         │  │ • Duration verification         │  │
         │  └─────────────────────────────────┘  │
         └──────────────┬────────────────────────┘
                        │
                        ▼
         ┌───────────────────────────────────────┐
         │      PROSODIC ANALYSIS LAYER           │
         │  ┌─────────────────────────────────┐  │
         │  │ 1. Rhythm Analysis              │  │
         │  │    - Soft-DTW alignment         │  │
         │  │    - nPVI/Varco metrics         │  │
         │  │    - Isochrony scoring          │  │
         │  ├─────────────────────────────────┤  │
         │  │ 2. Melody Analysis              │  │
         │  │    - Fujisaki decomposition     │  │
         │  │    - Tilt parametrization       │  │
         │  │    - Maqam classification       │  │
         │  ├─────────────────────────────────┤  │
         │  │ 3. Style Analysis               │  │
         │  │    - X-vector embeddings        │  │
         │  │    - GST tokens                 │  │
         │  │    - Timbre fingerprinting      │  │
         │  └─────────────────────────────────┘  │
         └──────────────┬────────────────────────┘
                        │
                        ▼
         ┌───────────────────────────────────────┐
         │        COMPARISON & SCORING             │
         │  • Multi-dimensional fusion            │
         │  • Weighted component scores           │
         │  • Confidence intervals                │
         │  • Explainability generation           │
         └──────────────┬────────────────────────┘
                        │
                        ▼
         ┌───────────────────────────────────────┐
         │      FEEDBACK GENERATION                │
         │  • Pedagogical text generation         │
         │  • Visual overlays                     │
         │  • Actionable corrections              │
         │  • Progress tracking                   │
         └──────────────┬────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────────┐
│                         OUTPUT                                  │
│  {scores, violations, visualizations, feedback, metadata}       │
└─────────────────────────────────────────────────────────────────┘
```

---

## Core Modules

### Module 1: Audio Preprocessing Pipeline

**Purpose:** Transform raw user audio into clean, standardized format for analysis

**Inputs:**
- Raw audio file (any format: MP3, WAV, WebM, M4A)
- Surah/ayah metadata (optional, for reference alignment)

**Outputs:**
```python
{
    "audio_path": str,           # Path to preprocessed WAV
    "sample_rate": 16000,
    "duration": float,           # Seconds
    "segments": [                # VAD-detected speech segments
        {"start": float, "end": float, "confidence": float}
    ],
    "quality_metrics": {
        "snr_db": float,         # Signal-to-noise ratio
        "clipping_ratio": float, # Proportion of clipped samples
        "rms_energy": float,
        "quality_flag": str      # "excellent"/"good"/"poor"
    }
}
```

**Key Components:**

1. **Audio Loading & Validation**
   - Library: `soundfile` or `librosa`
   - Validate format, check for corruption
   - Extract metadata (original sample rate, bit depth)

2. **Resampling**
   - Target: 16kHz (optimal for ASR models)
   - Method: Kaiser windowed sinc interpolation
   - Anti-aliasing filter before downsampling

3. **Normalization**
   - Peak normalization to -1 to +1 dB
   - Optional: LUFS normalization for loudness consistency

4. **Noise Reduction (Optional)**
   - **When:** If SNR < 15 dB
   - **Method:** Spectral subtraction or Wiener filtering
   - **Library:** `noisereduce` or `scipy.signal`
   - **Risk:** Can damage speech if too aggressive (configurable threshold)

5. **Voice Activity Detection (VAD)**
   - **Model:** Silero VAD (ONNX, fast)
   - **Purpose:** Trim silence, split long audio into segments
   - **Parameters:** 
     - Threshold: 0.5 (confidence)
     - Min speech duration: 250ms
     - Min silence duration: 300ms

6. **Quality Checks**
   - **SNR estimation:** Compare signal power in voiced vs unvoiced regions
   - **Clipping detection:** Count samples at ±1.0
   - **Dynamic range:** Compute crest factor
   - **Flags:**
     - `excellent`: SNR > 20 dB, no clipping
     - `good`: SNR 10-20 dB, < 1% clipping
     - `poor`: SNR < 10 dB or > 5% clipping (warn user)

**Implementation Notes:**
- **Idempotency:** Preprocessed files cached by hash (SHA256 of original)
- **Streaming:** For real-time, process in overlapping 5s windows
- **Fallback:** If VAD fails, analyze entire audio (no segmentation)

**Dependencies:**
```python
soundfile>=0.12.1
librosa>=0.10.0
noisereduce>=3.0.0
silero-vad>=3.1
scipy>=1.10.0
```

**Estimated Latency:**
- Offline: 200-500ms per minute of audio
- Real-time: <50ms per chunk (streaming mode)

---

### Module 2: Pitch Extraction

**Purpose:** Extract F0 (fundamental frequency) contour with high accuracy and speed

**Inputs:**
- Preprocessed audio (16kHz WAV)
- Optional: Voiced/unvoiced mask from VAD

**Outputs:**
```python
{
    "pitch_hz": np.ndarray,      # F0 values (NaN for unvoiced)
    "times": np.ndarray,         # Timestamps (seconds)
    "confidence": np.ndarray,    # Per-frame confidence (0-1)
    "voicing": np.ndarray,       # Boolean mask (voiced=True)
    "method": str,               # "swiftf0" or "rmvpe"
    "stats": {
        "mean_hz": float,
        "std_hz": float,
        "range_hz": tuple,       # (min, max)
        "voiced_ratio": float    # Proportion voiced
    }
}
```

**Key Components:**

1. **Primary Extractor: SwiftF0**
   - **Why:** 91.8% accuracy, 42× faster than CREPE, 95K params
   - **Frequency range:** 46.875-2093.75 Hz (covers male/female Quranic voices)
   - **Hop length:** 10ms (100 Hz sampling rate)
   - **Post-processing:**
     - Median filter (5-frame window) to smooth octave jumps
     - Linear interpolation for short unvoiced gaps (<100ms)
     - Confidence thresholding (discard frames < 0.5)

2. **Fallback Extractor: RMVPE**
   - **When:** Melodic passages (detected by high pitch variance)
   - **Why:** Better for polyphonic/ornamented recitation
   - **Trigger:** If SwiftF0 confidence < 0.7 for >30% of frames
   - **Trade-off:** 2-3× slower but handles harmonics better

3. **Confidence Weighting**
   - Combine both extractors using inverse variance weighting
   - Formula: `pitch_final = (p1*c1² + p2*c2²) / (c1² + c2²)`
   - Only if both extractors ran (fallback mode)

4. **Pitch Smoothing**
   - **Savitzky-Golay filter:** Polynomial order 3, window 51ms
   - **Purpose:** Remove microprosodic jitter, preserve contour shape
   - **Configurable:** Can disable for analysis requiring raw pitch

5. **Statistics Computation**
   - Mean/std in Hz (avoid semitone conversion bias)
   - Voiced ratio (critical for comparing different recording qualities)
   - Pitch range (for dynamic range analysis)

**Model Selection Logic:**
```python
def select_pitch_extractor(audio, sr=16000):
    """Automatically choose best extractor"""
    # 1. Always try SwiftF0 first (fast)
    pitch_swift, conf_swift = extract_swiftf0(audio, sr)
    
    # 2. Check if RMVPE needed
    if np.mean(conf_swift) < 0.7:
        # Low confidence → melodic passage
        pitch_rmvpe, conf_rmvpe = extract_rmvpe(audio, sr)
        # Blend using confidence weights
        return blend_pitch(pitch_swift, conf_swift, pitch_rmvpe, conf_rmvpe)
    else:
        return pitch_swift, conf_swift
```

**Dependencies:**
```python
swift-f0>=1.0.0          # PyPI package
rmvpe                    # GitHub: Dream-High/RMVPE
scipy>=1.10.0            # Savitzky-Golay filter
numpy>=1.23.0
```

**Estimated Latency:**
- **SwiftF0:** 130ms for 5s audio (offline)
- **RMVPE:** 400ms for 5s audio (fallback)
- **Real-time:** 15-20ms per 100ms chunk (SwiftF0 only)

**Validation Metrics:**
- Gross Pitch Error (GPE): <10% on Quranic test set
- Voicing Decision Error (VDE): <5%
- Fine Pitch Error (FPE): <30 cents RMS

---

### Module 3: Phoneme Alignment

**Purpose:** Align audio to phoneme-level timestamps with Tajweed rule annotations

**Inputs:**
- Preprocessed audio (16kHz WAV)
- Surah:ayah reference (for ground truth text)
- Word-level timestamps (from `husary-ayah-segments.json`)
- Phoneme-level Tajweed rules (from `quran-phoneme-tajweed.json`)

**Outputs:**
```python
{
    "phonemes": [
        {
            "phoneme": str,        # IPA or Buckwalter notation
            "start": float,        # Seconds
            "end": float,          # Seconds
            "duration": float,     # Seconds
            "word_idx": int,       # Which word in ayah
            "confidence": float,   # CTC posterior probability
            "tajweed_rule": str,   # "madd_lazim", "ghunnah", etc.
            "expected_duration": float,  # For madd rules (optional)
            "gop_score": float     # Goodness of Pronunciation (-∞ to 0)
        }
    ],
    "alignment_method": str,   # "wav2vec2_bert" or "mms_fa"
    "quality_score": float,    # Overall alignment confidence
    "warnings": [str]          # e.g., "Low confidence in word 5"
}
```

**Key Components:**

1. **Model: Wav2Vec2-BERT Fine-Tuned**
   - **Base:** `facebook/w2v-bert-2.0` (580M params)
   - **Fine-tuning:** Task-adaptive pretraining on Quranic recitations
     - **Stage 1:** Continue pretraining on 50-100 hours Tarteel dataset
     - **Stage 2:** Fine-tune with phoneme labels (MSA Phonetiser)
     - **Target PER:** <1% (SOTA is 0.16%)
   - **Phoneme Vocabulary:** 45 phonemes (Arabic + diacritics)
     - Consonants: 28 letters + pharyngealized variants (ص، ض، ط، ظ)
     - Vowels: Short (a, i, u), Long (aa, ii, uu)
     - Special: Sukoon, shadda, tanween, madda

2. **CTC Forced Alignment**
   - **Library:** `ctc-forced-aligner` (GPU-accelerated)
   - **Process:**
     - Extract phoneme sequence from reference text
     - Compute CTC posteriors for each frame
     - Viterbi decoding with phoneme constraints
     - Smooth boundaries using HMM post-processing
   - **Windowing:** Align within word boundaries (from segment data)
     - Reduces error propagation
     - Improves accuracy near word edges

3. **Fallback: MMS Forced Aligner**
   - **When:** Low Wav2Vec2 confidence (<0.6 mean)
   - **Model:** `facebook/mms-1b-all` (1B params, 1126 languages)
   - **Trade-off:** More robust but less accurate (2-5% higher PER)

4. **Tajweed Rule Mapping**
   - **Source:** `quran-phoneme-tajweed.json` (ground truth)
   - **Process:**
     - Match aligned phonemes to ground truth by position
     - Fuzzy matching if exact alignment fails (edit distance)
     - Assign Tajweed rule labels (madd type, ghunnah, etc.)
   - **Validation:** Check phoneme count matches expectation

5. **GOP (Goodness of Pronunciation) Scoring**
   - **Method:** SSL-GOP using Wav2Vec2 posteriors
   - **Formula:**
     ```
     GOP(p) = log P(p | audio) - log P(audio)
     ```
   - **Interpretation:**
     - GOP > -1.0: Excellent pronunciation
     - GOP -1.0 to -3.0: Good
     - GOP < -3.0: Poor (likely mispronunciation)
   - **Per-phoneme:** Enables fine-grained feedback

6. **Quality Checks**
   - **Alignment confidence:** Mean CTC posterior > 0.7
   - **Phoneme count:** ±10% of expected (from transliteration)
   - **Duration sanity:** Phoneme durations 20ms - 500ms (typical range)
   - **Warnings:** Flag low-confidence regions for manual review

**Training Pipeline (One-Time Setup):**

```bash
# Stage 1: Continue pretraining (1-2 days on 8×A100)
python scripts/pretrain_wav2vec2_bert.py \
  --base_model facebook/w2v-bert-2.0 \
  --train_data tarteel-ai-everyayah \
  --hours 100 \
  --output models/wav2vec2_bert_quranic

# Stage 2: Fine-tune with phoneme labels (4-8 hours)
python scripts/finetune_phoneme.py \
  --base_model models/wav2vec2_bert_quranic \
  --phoneme_vocab data/arabic_phonemes_45.txt \
  --train_data data/phoneme_annotations/ \
  --output models/wav2vec2_bert_phoneme_final
```

**Inference:**
```python
from iqrah_audio.alignment import Wav2Vec2PhonemeAligner

aligner = Wav2Vec2PhonemeAligner(
    model_path="models/wav2vec2_bert_phoneme_final",
    device="cuda"  # or "cpu"
)

result = aligner.align(
    audio_path="recitation.wav",
    surah=1, ayah=1,
    word_segments=load_segments(1, 1)
)
```

**Dependencies:**
```python
transformers>=4.35.0     # Wav2Vec2-BERT
ctc-forced-aligner>=0.1
torch>=2.0.0
torchaudio>=2.0.0
```

**Estimated Latency:**
- **Offline (CPU):** 1-2s per ayah
- **Offline (GPU):** 200-400ms per ayah
- **Real-time (GPU):** Achievable with streaming CTC

**Validation Metrics:**
- Phoneme Error Rate (PER): <1%
- Boundary precision: 80% within 20ms, 90% within 50ms
- GOP correlation with human ratings: r > 0.6

---

### Module 4: Tajweed Rule Validators

**Purpose:** Acoustic validation of Tajweed rules using signal processing and ML

**Design Philosophy:**
- **Hybrid approach:** Rule-based + acoustic validation
- **Progressive rollout:** Start with easiest rules (madd), add complex ones iteratively
- **Independent validators:** Each rule is a separate module

**Outputs (per validator):**
```python
{
    "rule_name": str,          # e.g., "madd_lazim"
    "violations": [
        {
            "phoneme_idx": int,
            "severity": str,   # "critical"/"moderate"/"minor"
            "expected": Any,   # Expected characteristic
            "actual": Any,     # Observed value
            "confidence": float,
            "feedback": str    # Human-readable explanation
        }
    ],
    "overall_score": float,    # 0-100
    "rule_coverage": float     # % of applicable phonemes checked
}
```

---

#### Validator 4.1: Madd Duration

**Accuracy Target:** 99.87% (achievable with rule-based, per SOTA research)

**Algorithm:**
1. Identify madd phonemes from Tajweed mapping
2. Extract duration from alignment
3. Classify madd type:
   - **Natural (Tabi'i):** 2 harakat (200-250ms)
   - **Connected (Muttasil):** 4-5 harakat (400-500ms)
   - **Separated (Munfasil):** 4-5 harakat (400-500ms)
   - **Required (Lazim):** 6 harakat (600ms+)
4. Check against expected duration ±20% tolerance
5. Score: Percentage of madds within tolerance

**Implementation:**
```python
class MaddDurationValidator:
    EXPECTED_DURATIONS = {
        "madd_tabii": (0.18, 0.28),      # 2h ± 20%
        "madd_muttasil": (0.36, 0.56),   # 4-5h
        "madd_munfasil": (0.36, 0.56),   # 4-5h
        "madd_lazim": (0.54, 0.72),      # 6h ± 20%
        "madd_leen": (0.18, 0.28),       # 2h
    }
    
    def validate(self, phonemes):
        violations = []
        for p in phonemes:
            if "madd_" in p["tajweed_rule"]:
                expected = self.EXPECTED_DURATIONS[p["tajweed_rule"]]
                if not (expected[0] <= p["duration"] <= expected[1]):
                    violations.append({
                        "phoneme_idx": p["idx"],
                        "severity": self._compute_severity(
                            p["duration"], expected
                        ),
                        "expected": f"{expected[0]*5:.1f}-{expected[1]*5:.1f} harakat",
                        "actual": f"{p['duration']*5:.1f} harakat",
                        "confidence": 0.95,  # High for rule-based
                        "feedback": self._generate_feedback(p, expected)
                    })
        return violations
```

**Latency:** <10ms per ayah (pure Python)

---

#### Validator 4.2: Ghunnah Detection

**Accuracy Target:** 85% (requires ML, per SOTA research)

**Algorithm:**
1. Identify ghunnah phonemes (noon saakinah, meem, tanween)
2. Extract formant frequencies (F1, F2, F3)
3. Compute acoustic features:
   - **Nasal formant:** 250-350 Hz energy peak
   - **F1 amplitude reduction:** Compared to surrounding vowels
   - **Spectral flatness:** Higher for nasal sounds
4. Train binary classifier (MLP or SVM) on labeled data
5. Duration check: Ghunnah should last ≥2 harakat (180-250ms)

**Feature Extraction (OpenSMILE or Parselmouth):**
```python
import parselmouth

def extract_ghunnah_features(audio, start, end, sr=16000):
    """Extract formants and acoustic correlates for ghunnah"""
    sound = parselmouth.Sound(audio[int(start*sr):int(end*sr)], sr)
    
    # Formants
    formant = sound.to_formant_burg()
    f1 = formant.get_value_at_time(1, (start+end)/2)
    f2 = formant.get_value_at_time(2, (start+end)/2)
    f3 = formant.get_value_at_time(3, (start+end)/2)
    
    # Nasal formant energy (250-350 Hz)
    spectrum = sound.to_spectrum()
    nasal_energy = spectrum.get_band_energy(250, 350)
    
    # Spectral flatness
    flatness = sound.to_harmonicity().values.mean()
    
    return {
        "f1_hz": f1,
        "f2_hz": f2,
        "f3_hz": f3,
        "nasal_energy_db": 10 * np.log10(nasal_energy),
        "spectral_flatness": flatness,
        "duration_s": end - start
    }
```

**Classifier Training (One-Time):**
```python
from sklearn.neural_network import MLPClassifier

# Train on labeled dataset (1000+ examples)
clf = MLPClassifier(hidden_layer_sizes=(64, 32), max_iter=500)
clf.fit(X_train, y_train)  # X = formant features, y = ghunnah/no-ghunnah

# Save model
joblib.dump(clf, "models/ghunnah_classifier.pkl")
```

**Inference:**
```python
class GhunnahValidator:
    def __init__(self, model_path="models/ghunnah_classifier.pkl"):
        self.clf = joblib.load(model_path)
    
    def validate(self, phonemes, audio, sr=16000):
        violations = []
        for p in phonemes:
            if p["tajweed_rule"] == "ghunnah":
                features = extract_ghunnah_features(
                    audio, p["start"], p["end"], sr
                )
                X = np.array([[
                    features["f1_hz"], features["f2_hz"], features["f3_hz"],
                    features["nasal_energy_db"], features["spectral_flatness"],
                    features["duration_s"]
                ]])
                
                prob = self.clf.predict_proba(X)[0, 1]  # P(ghunnah)
                
                if prob < 0.7:  # Threshold for "ghunnah present"
                    violations.append({
                        "phoneme_idx": p["idx"],
                        "severity": "moderate" if prob < 0.5 else "minor",
                        "expected": "Nasal resonance (ghunnah)",
                        "actual": f"Nasalization confidence: {prob:.2f}",
                        "confidence": prob,
                        "feedback": self._generate_feedback(p, features, prob)
                    })
        return violations
```

**Dependencies:**
```python
praat-parselmouth>=0.4.3
scikit-learn>=1.3.0
librosa>=0.10.0
```

**Latency:** 50-100ms per phoneme (formant extraction is slow)

**Training Data Required:**
- 500-1000 labeled ghunnah examples
- 500-1000 negative examples (non-ghunnah nasals)
- Annotation tool: Label Studio or Prodi.gy

---

#### Validator 4.3: Qalqalah Detection

**Accuracy Target:** 80-85% (exploratory, no strong SOTA baseline)

**Algorithm:**
1. Identify qalqalah letters (ق، ط، ب، ج، د) with sukoon
2. Extract acoustic features:
   - **Burst detection:** Sharp transient at consonant release
   - **Zero-crossing rate (ZCR):** High during burst
   - **Spectral centroid:** Higher for qalqalah
   - **Energy spike:** Localized increase at release
3. Train classifier (SVM or CNN on spectrograms)
4. Verify "echo" characteristic: amplitude decay after burst

**Feature Extraction:**
```python
def extract_qalqalah_features(audio, start, end, sr=16000):
    """Extract burst and transient features"""
    segment = audio[int(start*sr):int(end*sr)]
    
    # Zero-crossing rate (high during plosive burst)
    zcr = librosa.feature.zero_crossing_rate(segment)[0]
    
    # Spectral centroid (brightness)
    centroid = librosa.feature.spectral_centroid(y=segment, sr=sr)[0]
    
    # Energy envelope
    rms = librosa.feature.rms(y=segment)[0]
    
    # Burst detection: local maxima in energy
    burst_idx = np.argmax(rms)
    has_burst = rms[burst_idx] > 1.5 * np.median(rms)
    
    return {
        "zcr_mean": np.mean(zcr),
        "zcr_std": np.std(zcr),
        "centroid_mean": np.mean(centroid),
        "rms_max": np.max(rms),
        "rms_std": np.std(rms),
        "has_burst": has_burst,
        "duration_s": end - start
    }
```

**Classifier (Binary: Qalqalah or Not):**
```python
from sklearn.svm import SVC

# Train on labeled dataset
clf = SVC(kernel='rbf', probability=True)
clf.fit(X_train, y_train)
```

**Validation Logic:**
```python
class QalqalahValidator:
    QALQALAH_LETTERS = {'q', 'T', 'b', 'j', 'd'}  # Buckwalter notation
    
    def validate(self, phonemes, audio, sr=16000):
        violations = []
        for p in phonemes:
            # Check if letter is qalqalah and has sukoon
            if (p["phoneme"] in self.QALQALAH_LETTERS and 
                p["tajweed_rule"] == "qalqalah"):
                
                features = extract_qalqalah_features(
                    audio, p["start"], p["end"], sr
                )
                X = self._features_to_array(features)
                prob = self.clf.predict_proba(X)[0, 1]
                
                if prob < 0.6 or not features["has_burst"]:
                    violations.append({
                        "phoneme_idx": p["idx"],
                        "severity": "moderate",
                        "expected": "Sharp burst with echo",
                        "actual": f"Burst confidence: {prob:.2f}",
                        "confidence": prob,
                        "feedback": "Qalqalah requires a short, explosive release"
                    })
        return violations
```

**Latency:** 30-50ms per phoneme

**Training Data Required:**
- 300-500 qalqalah examples (each of 5 letters)
- 500-1000 negative examples (non-qalqalah consonants)

---

#### Validator 4.4: Complex Rules (Idghaam, Ikhfaa, Iqlaab)

**Accuracy Target:** 75-85% (complex, require context awareness)

**Strategy:**
- **Phonetic validation:** Check if assimilation occurred
- **Context-aware:** Examine surrounding phonemes
- **Rule-based + acoustic hybrid**

**Example: Idghaam (Assimilation)**
- Rule: Noon saakinah + specific letter → merge into single prolonged sound
- Acoustic check: Duration of merged phoneme > sum of individual durations
- Spectral check: Smooth transition (no discontinuity in formants)

**Example: Ikhfaa (Concealment)**
- Rule: Noon saakinah + certain letters → partial nasalization
- Acoustic check: Intermediate nasal formant (not full ghunnah, not absent)
- Duration check: Slightly longer than normal noon

**Implementation Deferred to Phase 3** (focus on madd, ghunnah, qalqalah first)

---

### Module 5: Voice Quality Analysis

**Purpose:** Quantify timbre, breathiness, vibrato, roughness for style comparison

**Inputs:**
- Preprocessed audio (16kHz WAV)
- Phoneme alignment (for voiced regions)

**Outputs:**
```python
{
    "vibrato": {
        "rate_hz": float,        # Oscillations per second (4-7 typical)
        "extent_semitones": float, # Amplitude (0.5-2 typical)
        "regularity": float      # 0-1 (higher = more consistent)
    },
    "breathiness": {
        "h1_h2_db": float,       # >5 dB = breathy
        "hnr_db": float,         # <10 dB = breathy/hoarse
        "cpp": float             # Cepstral peak prominence
    },
    "roughness": {
        "jitter_percent": float, # >1.04% = pathological
        "shimmer_percent": float # >3.81% = pathological
    },
    "timbre": {
        "spectral_centroid_hz": float,  # Brightness (2-4k typical)
        "spectral_flux": float,         # Dynamic quality
        "spectral_rolloff_hz": float,   # 85% energy point
        "formants": {                   # F1-F4 means
            "f1_hz": float,
            "f2_hz": float,
            "f3_hz": float,
            "f4_hz": float
        }
    },
    "embeddings": {
        "x_vector": np.ndarray,  # 512-d style embedding
        "wav2vec2_cls": np.ndarray  # 768-d from [CLS] token
    }
}
```

**Key Components:**

1. **OpenSMILE eGeMAPS**
   - **Purpose:** Standardized prosodic features (88 dimensions)
   - **Library:** `opensmile` (Python wrapper)
   - **Usage:**
     ```python
     import opensmile
     
     smile = opensmile.Smile(
         feature_set=opensmile.FeatureSet.eGeMAPSv02,
         feature_level=opensmile.FeatureLevel.Functionals
     )
     
     features = smile.process_file("recitation.wav")
     # Returns DataFrame with 88 features
     ```
   - **Includes:** Jitter, shimmer, HNR, formants, MFCC, spectral features
   - **Latency:** ~200ms per minute of audio (CPU)

2. **Vibrato Detection (Parselmouth)**
   ```python
   import parselmouth
   
   def detect_vibrato(pitch_contour, times):
       """Analyze vibrato characteristics"""
       # 1. Bandpass filter pitch (2-15 Hz for vibrato range)
       from scipy.signal import butter, filtfilt
       b, a = butter(4, [2, 15], btype='band', fs=100)  # 100 Hz pitch sampling
       filtered = filtfilt(b, a, pitch_contour)
       
       # 2. Autocorrelation to find rate
       autocorr = np.correlate(filtered, filtered, mode='full')
       autocorr = autocorr[len(autocorr)//2:]
       peaks = find_peaks(autocorr)[0]
       if len(peaks) > 0:
           rate_hz = 100 / peaks[0]  # Period in samples → Hz
       else:
           rate_hz = 0  # No vibrato
       
       # 3. Extent (amplitude of oscillation)
       extent_semitones = 12 * np.log2(np.ptp(filtered) + 1e-6)
       
       # 4. Regularity (CV of peak intervals)
       if len(peaks) > 1:
           intervals = np.diff(peaks)
           regularity = 1 - (np.std(intervals) / np.mean(intervals))
       else:
           regularity = 0
       
       return {
           "rate_hz": rate_hz,
           "extent_semitones": extent_semitones,
           "regularity": max(0, regularity)
       }
   ```

3. **Breathiness Features (Parselmouth)**
   ```python
   def extract_breathiness(audio, sr=16000):
       """Compute H1-H2, HNR, CPP"""
       sound = parselmouth.Sound(audio, sr)
       
       # H1-H2 (difference between first two harmonics)
       harmonicity = sound.to_harmonicity()
       h1_h2 = harmonicity.values[:, :2].diff(axis=1).mean()
       
       # HNR (Harmonic-to-Noise Ratio)
       hnr = harmonicity.values.mean()
       
       # CPP (Cepstral Peak Prominence) - via pitch object
       pitch = sound.to_pitch()
       cpp = pitch.to_cpp()  # Requires Praat 6.2+
       
       return {
           "h1_h2_db": h1_h2,
           "hnr_db": hnr,
           "cpp": cpp
       }
   ```

4. **X-Vector Embeddings (SpeechBrain)**
   ```python
   from speechbrain.pretrained import EncoderClassifier
   
   # Load pretrained model
   classifier = EncoderClassifier.from_hparams(
       source="speechbrain/spkrec-xvect-voxceleb",
       savedir="models/xvector"
   )
   
   # Extract 512-d embedding
   embedding = classifier.encode_batch(audio_tensor)
   # Use for style comparison (cosine similarity)
   ```

5. **Wav2Vec2 [CLS] Token (HuggingFace)**
   ```python
   from transformers import Wav2Vec2Model, Wav2Vec2Processor
   
   model = Wav2Vec2Model.from_pretrained("facebook/wav2vec2-base")
   processor = Wav2Vec2Processor.from_pretrained("facebook/wav2vec2-base")
   
   inputs = processor(audio, sampling_rate=16000, return_tensors="pt")
   outputs = model(**inputs)
   
   # Extract [CLS] token (layer 12, position 0)
   embedding = outputs.last_hidden_state[:, 0, :].numpy()  # 768-d
   ```

**Dependencies:**
```python
opensmile>=3.0.1
praat-parselmouth>=0.4.3
speechbrain>=0.5.16
transformers>=4.35.0
scipy>=1.10.0
librosa>=0.10.0
```

**Estimated Latency:**
- OpenSMILE: 200ms per minute (CPU)
- Parselmouth: 100-200ms per phoneme
- X-vectors: 500ms per ayah (GPU), 2-3s (CPU)
- Wav2Vec2: 300ms per ayah (GPU), 1-2s (CPU)

**Usage in Comparison:**
- Compute embeddings for both student and reference
- Cosine similarity for overall style matching (0-1 scale)
- Component-wise comparison for detailed feedback
  - "Your vibrato rate is 6.2 Hz vs reference 5.5 Hz"
  - "Breathiness (HNR) is 8.5 dB vs reference 12.3 dB (more breathy)"

---

### Module 6: Prosodic Analysis

**Purpose:** Comprehensive rhythm, melody, and style characterization

**Outputs:**
```python
{
    "rhythm": {
        "tempo_sps": float,          # Syllables per second
        "nPVI": float,               # Pairwise variability index
        "varco": float,              # Coefficient of variation
        "ioi_distribution": np.ndarray,  # Inter-onset intervals
        "rhythm_class": str          # "stress-timed"/"syllable-timed"
    },
    "melody": {
        "fujisaki_params": {
            "phrase_commands": [      # List of phrase components
                {"time": float, "amplitude": float, "duration": float}
            ],
            "accent_commands": [      # List of accent components
                {"time": float, "amplitude": float}
            ]
        },
        "declination": {
            "slope": float,           # Hz/second
            "r_squared": float        # Fit quality
        },
        "tilt_features": {            # Per-phrase contour shape
            "rising_ratio": float,
            "falling_ratio": float,
            "convex_ratio": float,
            "level_ratio": float
        },
        "maqam": {
            "predicted": str,         # "Bayati", "Rast", etc.
            "confidence": float,
            "probability_distribution": dict  # All maqams
        }
    },
    "style": {
        "x_vector": np.ndarray,       # 512-d embedding
        "gst_tokens": np.ndarray,     # Global Style Tokens (optional)
        "style_label": str            # "Egyptian", "Gulf", etc.
    }
}
```

---

#### Component 6.1: Advanced Rhythm Analysis

**Beyond Soft-DTW: Distributional Metrics**

1. **Pairwise Variability Index (nPVI)**
   ```python
   def compute_npvi(durations):
       """
       Quantifies rhythmic variability
       >60: stress-timed (English)
       <50: syllable-timed (Arabic, French)
       """
       m = len(durations) - 1
       if m == 0:
           return 0
       sum_term = sum(
           abs(durations[i] - durations[i+1]) / 
           ((durations[i] + durations[i+1]) / 2)
           for i in range(m)
       )
       return 100 * (1/m) * sum_term
   ```

2. **Varco (Normalized Variability)**
   ```python
   def compute_varco(ioi_intervals):
       """
       Coefficient of variation
       Lower = more isochronous (metronome-like)
       """
       return 100 * np.std(ioi_intervals) / np.mean(ioi_intervals)
   ```

3. **Isochrony Scoring**
   ```python
   def score_isochrony(ioi_intervals, target_ioi=0.3):
       """
       How regular is the rhythm?
       Score 0-100 (higher = more regular)
       """
       deviations = np.abs(ioi_intervals - target_ioi) / target_ioi
       score = 100 * (1 - np.mean(np.minimum(deviations, 1.0)))
       return score
   ```

4. **Integration with Soft-DTW**
   - **Use case 1:** Soft-DTW for alignment (tempo-invariant)
   - **Use case 2:** nPVI/Varco for rhythm style characterization
   - **Comparison:** Align using DTW, then compare distributions
     ```python
     # After DTW alignment
     student_ioi_warped = warp_sequence(student_ioi, dtw_path)
     
     # Compare distributions
     from scipy.stats import wasserstein_distance
     rhythm_dist = wasserstein_distance(
         student_ioi_warped, reference_ioi
     )
     ```

**Latency:** <50ms per ayah (pure NumPy)

---

#### Component 6.2: Melody Analysis

**Fujisaki Model Decomposition**

**Purpose:** Separate phrase-level intonation from word-level accents

```python
from scipy.optimize import minimize

def fit_fujisaki(f0_contour, times):
    """
    Decompose F0 into phrase + accent components
    Returns: phrase_commands, accent_commands
    """
    def fujisaki_model(t, phrase_params, accent_params):
        # Phrase component: low-pass filtered
        Fp = sum(
            Ap * (1 - (1 + (t-T1p)/Tp) * np.exp(-(t-T1p)/Tp))
            for Ap, T1p, Tp in phrase_params
            if t >= T1p
        )
        
        # Accent component: high-pass filtered
        Fa = sum(
            Aa * (1 - (1 + (t-T1a)/Ta) * np.exp(-(t-T1a)/Ta))
            for Aa, T1a, Ta in accent_params
            if t >= T1a
        )
        
        return Fb + Fp + Fa  # Fb = baseline
    
    # Optimize parameters to fit observed F0
    result = minimize(
        lambda params: np.sum((f0_contour - fujisaki_model(times, ...))**2),
        initial_params,
        method='L-BFGS-B'
    )
    
    return result.x  # Optimized phrase/accent parameters
```

**Declination Modeling**

```python
def fit_declination(f0_contour, times):
    """
    Model baseline F0 drop over utterance
    Typical: 10-15% decrease from start to end
    """
    # Extract F0 minima (baseline)
    from scipy.signal import argrelextrema
    minima_idx = argrelextrema(f0_contour, np.less)[0]
    f0_minima = f0_contour[minima_idx]
    t_minima = times[minima_idx]
    
    # Fit exponential decay: f0(t) = a * exp(-b*t) + c
    from scipy.optimize import curve_fit
    popt, _ = curve_fit(
        lambda t, a, b, c: a * np.exp(-b*t) + c,
        t_minima, f0_minima
    )
    
    # Remove declination trend
    f0_normalized = f0_contour / (popt[0] * np.exp(-popt[1]*times) + popt[2])
    
    return {
        "slope": -popt[1],  # Decay rate
        "r_squared": compute_r2(f0_minima, popt),
        "f0_normalized": f0_normalized
    }
```

**Tilt Parametrization**

```python
def extract_tilt_features(f0_contour):
    """
    Classify contour shapes: rising, falling, level, convex, concave
    """
    # Find peak
    peak_idx = np.argmax(f0_contour)
    
    # Rise phase
    rise_amp = f0_contour[peak_idx] - f0_contour[0]
    rise_dur = peak_idx
    
    # Fall phase
    fall_amp = f0_contour[peak_idx] - f0_contour[-1]
    fall_dur = len(f0_contour) - peak_idx
    
    # Tilt parameters
    tilt_amp = (rise_amp - fall_amp) / (rise_amp + fall_amp + 1e-6)
    tilt_dur = (rise_dur - fall_dur) / (rise_dur + fall_dur + 1e-6)
    
    # Classify shape
    if abs(tilt_amp) < 0.2:
        shape = "level"
    elif tilt_amp > 0.5:
        shape = "rising"
    elif tilt_amp < -0.5:
        shape = "falling"
    elif rise_dur < fall_dur:
        shape = "convex"
    else:
        shape = "concave"
    
    return {
        "tilt_amp": tilt_amp,
        "tilt_dur": tilt_dur,
        "shape": shape
    }
```

**Latency:** 100-200ms per ayah (optimization is slow)

---

#### Component 6.3: Maqam Recognition

**Purpose:** Identify Arabic musical mode for style-aware comparison

**Approach:**
1. Extract pitch histogram (12-bin chroma)
2. Extract MFCCs (20 coefficients)
3. Combine features → CNN classifier
4. Predict maqam (8-10 common modes)

**Implementation:**
```python
from sklearn.preprocessing import StandardScaler
from tensorflow import keras

class MaqamClassifier:
    def __init__(self, model_path="models/maqam_cnn.h5"):
        self.model = keras.models.load_model(model_path)
        self.scaler = joblib.load("models/maqam_scaler.pkl")
        self.maqam_labels = [
            "Bayati", "Rast", "Saba", "Hijaz", "Nahawand",
            "Ajam", "Kurd", "Sikah"
        ]
    
    def extract_features(self, audio, sr=16000):
        # 1. Pitch histogram (chroma)
        chroma = librosa.feature.chroma_cqt(y=audio, sr=sr)
        chroma_mean = np.mean(chroma, axis=1)  # 12-bin
        
        # 2. MFCCs
        mfccs = librosa.feature.mfcc(y=audio, sr=sr, n_mfcc=20)
        mfcc_mean = np.mean(mfccs, axis=1)  # 20 coefficients
        
        # 3. Concatenate
        features = np.concatenate([chroma_mean, mfcc_mean])
        return features  # 32-d vector
    
    def predict(self, audio, sr=16000):
        features = self.extract_features(audio, sr)
        features_scaled = self.scaler.transform([features])
        
        # Predict probabilities
        probs = self.model.predict(features_scaled)[0]
        predicted_idx = np.argmax(probs)
        
        return {
            "predicted": self.maqam_labels[predicted_idx],
            "confidence": float(probs[predicted_idx]),
            "probability_distribution": {
                label: float(prob)
                for label, prob in zip(self.maqam_labels, probs)
            }
        }
```

**Training (One-Time):**
```python
# Use Maqam478 dataset or similar
# CNN architecture:
model = keras.Sequential([
    keras.layers.Input(shape=(32,)),
    keras.layers.Dense(128, activation='relu'),
    keras.layers.Dropout(0.3),
    keras.layers.Dense(64, activation='relu'),
    keras.layers.Dropout(0.3),
    keras.layers.Dense(8, activation='softmax')  # 8 maqams
])

model.compile(
    optimizer='adam',
    loss='categorical_crossentropy',
    metrics=['accuracy']
)

# Train on labeled data
model.fit(X_train, y_train, epochs=50, validation_split=0.2)
```

**Accuracy Target:** >90% on 8-maqam classification (per SOTA research)

**Latency:** 100-200ms per ayah (CNN inference)

**Usage:** 
- Compare student vs reference maqam
- Warn if different modes used (style mismatch)
- Track maqam transitions within long recitations

---

### Module 7: Comparison Engine

**Purpose:** Multi-dimensional fusion of all analysis components

**Inputs:**
- Student analysis results (all modules 1-6)
- Reference analysis results (all modules 1-6)

**Outputs:**
```python
{
    "overall_score": float,       # 0-100
    "component_scores": {
        "tajweed": {              # From Module 4
            "madd": float,
            "ghunnah": float,
            "qalqalah": float,
            "overall": float
        },
        "prosody": {              # From Module 6
            "rhythm": float,
            "melody": float,
            "style": float,
            "overall": float
        },
        "pronunciation": {        # From Module 3
            "gop_score": float,
            "phoneme_accuracy": float
        },
        "voice_quality": {        # From Module 5
            "timbre_similarity": float,
            "vibrato_match": float,
            "breathiness_match": float
        }
    },
    "violations": [               # Aggregated from all validators
        {
            "rule": str,
            "severity": str,
            "phoneme_idx": int,
            "feedback": str,
            "timestamp": float
        }
    ],
    "confidence": float,          # Overall confidence in comparison
    "warnings": [str]             # System warnings (low quality, etc.)
}
```

---

#### Scoring Weights (Configurable)

**Default Configuration (Balanced):**
```python
WEIGHTS = {
    "tajweed": 0.40,      # Most important for correctness
    "prosody": 0.30,      # Style and rhythm
    "pronunciation": 0.20, # Phoneme-level quality
    "voice_quality": 0.10  # Timbre matching
}

TAJWEED_WEIGHTS = {
    "madd": 0.50,         # Most critical and reliable
    "ghunnah": 0.25,
    "qalqalah": 0.15,
    "complex_rules": 0.10
}

PROSODY_WEIGHTS = {
    "rhythm": 0.40,
    "melody": 0.40,
    "style": 0.20
}
```

**User-Adjustable Profiles:**
1. **Beginner:** Focus on tajweed (0.60), less on prosody (0.20)
2. **Intermediate:** Balanced (default)
3. **Advanced:** Heavy prosody weight (0.40), style matching (0.15)

---

#### Fusion Algorithm

```python
class ComparisonEngine:
    def __init__(self, weights=WEIGHTS):
        self.weights = weights
        self.validators = {
            "madd": MaddDurationValidator(),
            "ghunnah": GhunnahValidator(),
            "qalqalah": QalqalahValidator(),
        }
    
    def compare(self, student_data, reference_data):
        """
        Main comparison function
        Returns: Overall score + component breakdowns
        """
        # 1. Tajweed validation
        tajweed_results = self._compare_tajweed(
            student_data["phonemes"],
            reference_data["phonemes"],
            student_data["audio"]
        )
        
        # 2. Prosody comparison
        prosody_results = self._compare_prosody(
            student_data["prosody"],
            reference_data["prosody"]
        )
        
        # 3. Pronunciation (GOP)
        pronunciation_results = self._compare_pronunciation(
            student_data["phonemes"],
            reference_data["phonemes"]
        )
        
        # 4. Voice quality
        voice_results = self._compare_voice_quality(
            student_data["voice_quality"],
            reference_data["voice_quality"]
        )
        
        # 5. Fusion
        overall_score = (
            self.weights["tajweed"] * tajweed_results["overall"] +
            self.weights["prosody"] * prosody_results["overall"] +
            self.weights["pronunciation"] * pronunciation_results["overall"] +
            self.weights["voice_quality"] * voice_results["overall"]
        )
        
        return {
            "overall_score": overall_score,
            "component_scores": {
                "tajweed": tajweed_results,
                "prosody": prosody_results,
                "pronunciation": pronunciation_results,
                "voice_quality": voice_results
            },
            "violations": self._aggregate_violations(...),
            "confidence": self._compute_confidence(...),
            "warnings": self._generate_warnings(...)
        }
    
    def _compare_tajweed(self, student_phonemes, ref_phonemes, audio):
        """Run all tajweed validators"""
        results = {}
        violations = []
        
        # Madd
        madd_result = self.validators["madd"].validate(student_phonemes)
        results["madd"] = 100 - (len(madd_result) / num_madd_phonemes * 100)
        violations.extend(madd_result)
        
        # Ghunnah
        ghunnah_result = self.validators["ghunnah"].validate(
            student_phonemes, audio
        )
        results["ghunnah"] = 100 - (len(ghunnah_result) / num_ghunnah * 100)
        violations.extend(ghunnah_result)
        
        # Qalqalah
        qalqalah_result = self.validators["qalqalah"].validate(
            student_phonemes, audio
        )
        results["qalqalah"] = 100 - (len(qalqalah_result) / num_qalqalah * 100)
        violations.extend(qalqalah_result)
        
        # Weighted average
        results["overall"] = (
            TAJWEED_WEIGHTS["madd"] * results["madd"] +
            TAJWEED_WEIGHTS["ghunnah"] * results["ghunnah"] +
            TAJWEED_WEIGHTS["qalqalah"] * results["qalqalah"]
        )
        
        results["violations"] = violations
        return results
    
    def _compare_prosody(self, student_prosody, ref_prosody):
        """Compare rhythm, melody, style"""
        # Rhythm: DTW divergence + distributional metrics
        rhythm_score = self._score_rhythm(student_prosody, ref_prosody)
        
        # Melody: Fujisaki + tilt + maqam
        melody_score = self._score_melody(student_prosody, ref_prosody)
        
        # Style: X-vector cosine similarity
        style_score = self._score_style(student_prosody, ref_prosody)
        
        overall = (
            PROSODY_WEIGHTS["rhythm"] * rhythm_score +
            PROSODY_WEIGHTS["melody"] * melody_score +
            PROSODY_WEIGHTS["style"] * style_score
        )
        
        return {
            "rhythm": rhythm_score,
            "melody": melody_score,
            "style": style_score,
            "overall": overall
        }
    
    def _score_rhythm(self, student, reference):
        """
        Combine Soft-DTW alignment + distributional metrics
        """
        # 1. Soft-DTW divergence (normalize by path length)
        from tslearn.metrics import soft_dtw
        dtw_cost = soft_dtw(
            student["rhythm"]["features"],
            reference["rhythm"]["features"],
            gamma=1.0  # Smoothing parameter
        )
        dtw_score = 100 * np.exp(-dtw_cost / path_length)
        
        # 2. nPVI comparison
        npvi_diff = abs(student["rhythm"]["nPVI"] - reference["rhythm"]["nPVI"])
        npvi_score = 100 * (1 - npvi_diff / 100)  # Normalize to 0-100
        
        # 3. IOI distribution (Wasserstein distance)
        from scipy.stats import wasserstein_distance
        ioi_dist = wasserstein_distance(
            student["rhythm"]["ioi_distribution"],
            reference["rhythm"]["ioi_distribution"]
        )
        ioi_score = 100 * np.exp(-ioi_dist * 10)
        
        # Weighted combination
        rhythm_score = (
            0.6 * dtw_score +
            0.2 * npvi_score +
            0.2 * ioi_score
        )
        return rhythm_score
    
    def _score_melody(self, student, reference):
        """
        Fujisaki parameters + tilt + maqam
        """
        # 1. Fujisaki parameter similarity
        fujisaki_score = self._compare_fujisaki(
            student["melody"]["fujisaki_params"],
            reference["melody"]["fujisaki_params"]
        )
        
        # 2. Tilt distribution (shape similarity)
        tilt_score = self._compare_tilt(
            student["melody"]["tilt_features"],
            reference["melody"]["tilt_features"]
        )
        
        # 3. Maqam match
        if student["melody"]["maqam"]["predicted"] == reference["melody"]["maqam"]["predicted"]:
            maqam_score = 100
        else:
            # Partial credit for similar maqams
            maqam_score = 50  # Could use maqam similarity matrix
        
        melody_score = (
            0.4 * fujisaki_score +
            0.4 * tilt_score +
            0.2 * maqam_score
        )
        return melody_score
    
    def _score_style(self, student, reference):
        """
        X-vector cosine similarity
        """
        from scipy.spatial.distance import cosine
        
        # Cosine similarity (convert to 0-100 scale)
        similarity = 1 - cosine(
            student["style"]["x_vector"],
            reference["style"]["x_vector"]
        )
        return 100 * similarity
    
    def _compare_pronunciation(self, student_phonemes, ref_phonemes):
        """
        GOP score comparison
        """
        # Compute GOP delta
        student_gops = [p["gop_score"] for p in student_phonemes]
        ref_gops = [p["gop_score"] for p in ref_phonemes]
        
        # Mean absolute difference
        gop_delta = np.mean(np.abs(
            np.array(student_gops) - np.array(ref_gops)
        ))
        
        # Convert to 0-100 scale (lower delta = better)
        gop_score = 100 * np.exp(-gop_delta)
        
        # Phoneme accuracy (via CTC confidence)
        phoneme_acc = np.mean([p["confidence"] for p in student_phonemes])
        
        overall = 0.6 * gop_score + 0.4 * (phoneme_acc * 100)
        
        return {
            "gop_score": gop_score,
            "phoneme_accuracy": phoneme_acc * 100,
            "overall": overall
        }
    
    def _compare_voice_quality(self, student_vq, reference_vq):
        """
        Timbre, vibrato, breathiness matching
        """
        # 1. Timbre (spectral features)
        timbre_diff = np.abs(
            student_vq["timbre"]["spectral_centroid_hz"] -
            reference_vq["timbre"]["spectral_centroid_hz"]
        )
        timbre_score = 100 * np.exp(-timbre_diff / 1000)
        
        # 2. Vibrato rate matching
        vibrato_diff = np.abs(
            student_vq["vibrato"]["rate_hz"] -
            reference_vq["vibrato"]["rate_hz"]
        )
        vibrato_score = 100 * np.exp(-vibrato_diff / 2)
        
        # 3. Breathiness (HNR)
        breath_diff = np.abs(
            student_vq["breathiness"]["hnr_db"] -
            reference_vq["breathiness"]["hnr_db"]
        )
        breath_score = 100 * np.exp(-breath_diff / 5)
        
        overall = (
            0.5 * timbre_score +
            0.25 * vibrato_score +
            0.25 * breath_score
        )
        
        return {
            "timbre_similarity": timbre_score,
            "vibrato_match": vibrato_score,
            "breathiness_match": breath_score,
            "overall": overall
        }
```

**Latency:** <100ms (mostly NumPy operations, no heavy computation)

---

### Module 8: Feedback Generation

**Purpose:** Convert numerical scores into actionable pedagogical feedback

**Inputs:**
- Comparison results (from Module 7)
- User proficiency level (beginner/intermediate/advanced)

**Outputs:**
```python
{
    "summary": str,           # High-level feedback (2-3 sentences)
    "detailed_feedback": [    # Per-component feedback
        {
            "component": str,  # e.g., "madd_duration"
            "score": float,
            "feedback": str,   # Actionable text
            "examples": [      # Specific violations with timestamps
                {
                    "timestamp": float,
                    "text": str,
                    "audio_snippet": str  # Optional: path to audio clip
                }
            ]
        }
    ],
    "progress_tracking": {    # Compare to previous attempts
        "improvement": float, # +/- change from last attempt
        "streak": int,        # Consecutive improvements
        "best_score": float   # Personal best
    },
    "next_steps": [str]       # Recommendations
}
```

---

#### Feedback Templates

**Summary Generation:**
```python
def generate_summary(overall_score, component_scores):
    """Generate high-level summary"""
    if overall_score >= 90:
        level = "excellent"
        emoji = "🌟"
    elif overall_score >= 75:
        level = "very good"
        emoji = "👍"
    elif overall_score >= 60:
        level = "good"
        emoji = "✓"
    elif overall_score >= 40:
        level = "needs improvement"
        emoji = "⚠️"
    else:
        level = "requires attention"
        emoji = "❌"
    
    # Find weakest component
    weakest = min(component_scores, key=lambda x: x["overall"])
    
    summary = f"""{emoji} Your recitation is {level} (score: {overall_score:.0f}/100). 
Your strongest area is {strongest_component} ({strongest_score:.0f}/100), 
while {weakest_component} needs work ({weakest_score:.0f}/100). 
Focus on improving {weakest_component} in your next practice session."""
    
    return summary
```

**Detailed Feedback Templates:**

1. **Madd Duration:**
```python
if violation["rule"] == "madd_lazim":
    feedback = f"""
At {format_timestamp(violation['timestamp'])}, the required madd (مد لازم) 
was too short. You held it for {violation['actual']}, but it should be 
{violation['expected']}. 

**How to fix:** Take a deep breath before this word, and count slowly to 6 
in your head while extending the vowel. Practice holding the sound until 
it feels uncomfortably long—that's the correct duration.
"""
```

2. **Ghunnah:**
```python
if violation["rule"] == "ghunnah":
    feedback = f"""
At {format_timestamp(violation['timestamp'])}, the ghunnah (غنّة) 
is not strong enough. The nasal resonance confidence is {violation['actual']:.0%}.

**How to fix:** Close your lips lightly and hum through your nose while 
pronouncing the noon. You should feel vibration in your nasal cavity. 
Try placing a finger on your nose—if it doesn't vibrate, the ghunnah is missing.
"""
```

3. **Rhythm:**
```python
if component == "rhythm":
    feedback = f"""
Your rhythm score is {score:.0f}/100. The main issue is tempo consistency—
your nPVI is {student_npvi:.1f} vs reference {ref_npvi:.1f}.

**What this means:** You're rushing through some syllables and dragging others. 
Aim for more steady, even timing.

**How to fix:** Practice with a metronome set to {target_tempo:.0f} BPM. 
Tap your finger on each syllable to maintain consistent rhythm.
"""
```

4. **Melody:**
```python
if component == "melody":
    if maqam_mismatch:
        feedback = f"""
You're using a different maqam (musical mode) than the reference. 
You used {student_maqam}, but the reference uses {ref_maqam}.

**Why this matters:** Each maqam has a distinct emotional character. 
Switching modes changes the intended feeling of the recitation.

**How to fix:** Listen closely to the reference's pitch contour, especially 
at the beginning and end of phrases. Try to match the rise and fall patterns.
"""
```

**Progress Tracking:**
```python
class ProgressTracker:
    def __init__(self, user_id, db_connection):
        self.user_id = user_id
        self.db = db_connection
    
    def record_attempt(self, surah, ayah, score, timestamp):
        """Store attempt in database"""
        self.db.execute(
            "INSERT INTO attempts (user_id, surah, ayah, score, timestamp) "
            "VALUES (?, ?, ?, ?, ?)",
            (self.user_id, surah, ayah, score, timestamp)
        )
    
    def get_progress(self, surah, ayah):
        """Compare to previous attempts"""
        history = self.db.query(
            "SELECT score FROM attempts "
            "WHERE user_id=? AND surah=? AND ayah=? "
            "ORDER BY timestamp DESC LIMIT 5",
            (self.user_id, surah, ayah)
        )
        
        if len(history) < 2:
            return {"improvement": None, "streak": 0, "best_score": history[0]}
        
        improvement = history[0] - history[1]
        
        # Calculate improvement streak
        streak = 0
        for i in range(len(history) - 1):
            if history[i] > history[i+1]:
                streak += 1
            else:
                break
        
        best_score = max(history)
        
        return {
            "improvement": improvement,
            "streak": streak,
            "best_score": best_score
        }
```

**Next Steps Recommendations:**
```python
def recommend_next_steps(component_scores, violations):
    """AI-generated practice recommendations"""
    recommendations = []
    
    # Priority: Address critical violations first
    critical_violations = [v for v in violations if v["severity"] == "critical"]
    if critical_violations:
        rules = set(v["rule"] for v in critical_violations)
        recommendations.append(
            f"Focus on mastering {', '.join(rules)} before moving forward. "
            "These are fundamental rules that affect recitation accuracy."
        )
    
    # Second priority: Weakest component
    weakest = min(component_scores, key=lambda x: x["overall"])
    if weakest["overall"] < 70:
        recommendations.append(
            f"Dedicate extra practice time to {weakest['name']}. "
            "Consider taking a course or finding a teacher who specializes in this area."
        )
    
    # Suggest progressive difficulty
    if component_scores["tajweed"]["madd"] > 90:
        recommendations.append(
            "You've mastered madd duration! Ready to tackle ghunnah and qalqalah next."
        )
    
    # Encourage consistency
    if streak >= 3:
        recommendations.append(
            f"Great work! You've improved {streak} sessions in a row. "
            "Keep practicing daily to maintain this momentum."
        )
    
    return recommendations
```

**Latency:** <50ms (template rendering + database queries)

---

## Progressive Rollout Strategy

### Phase 1: Offline E2E Pipeline (Months 1-6)

**Goal:** 90% accuracy on basic Tajweed rules, comprehensive prosodic analysis

**Deliverables:**
1. ✅ **Preprocessing Pipeline** (Module 1)
   - Audio normalization, VAD, quality checks
   - **Target:** Process any user audio format

2. ✅ **Pitch Extraction** (Module 2)
   - SwiftF0 primary, RMVPE fallback
   - **Target:** <10% GPE on Quranic test set

3. ✅ **Phoneme Alignment** (Module 3)
   - Fine-tune Wav2Vec2-BERT on Quranic data
   - **Target:** <1% PER, 90% boundary accuracy within 50ms

4. 🚧 **Tajweed Validators** (Module 4)
   - **Madd:** 99% accuracy (rule-based)
   - **Ghunnah:** 85% accuracy (formant + MLP)
   - **Qalqalah:** 80% accuracy (burst detection + SVM)
   - **Target:** Progressive rollout (madd → ghunnah → qalqalah)

5. 🚧 **Voice Quality** (Module 5)
   - OpenSMILE eGeMAPS extraction
   - X-vector embeddings
   - **Target:** Correlation r > 0.7 with human ratings

6. 🚧 **Prosodic Analysis** (Module 6)
   - Rhythm: Soft-DTW + nPVI/Varco
   - Melody: Fujisaki + tilt + maqam
   - **Target:** Comprehensive characterization

7. ✅ **Comparison Engine** (Module 7)
   - Multi-dimensional fusion
   - **Target:** Overall score r > 0.8 with expert ratings

8. 🚧 **Feedback Generation** (Module 8)
   - Pedagogical text generation
   - Progress tracking
   - **Target:** Actionable feedback for 90%+ of users

**Validation:**
- Collect 100 expert-rated recitation pairs (inter-rater agreement κ > 0.75)
- Correlation study: Automated scores vs human ratings (r > 0.7)
- User study: 20-30 users, qualitative feedback on usefulness

**Infrastructure:**
- Desktop CPU for development
- GPU server for training (Lambda Labs/RunPod)
- PostgreSQL for user data
- S3/MinIO for audio storage

**Estimated Cost:**
- Training compute: €500-1,000 (Wav2Vec2 fine-tuning, validator training)
- Development time: 6 months (solo developer)
- Validation study: €1,000-2,000 (expert annotations)

---

### Phase 2: Real-Time Optimization (Months 7-12)

**Goal:** <500ms latency, streaming analysis

**Key Changes:**

1. **Streaming Architecture**
   - Replace batch processing with incremental analysis
   - Use WebSocket for bi-directional communication
   - Implement VAD-based chunking (5-second overlapping windows)

2. **Model Quantization**
   - INT8 quantization for Wav2Vec2 (4× speedup, <2% accuracy loss)
   - ONNX Runtime for inference (2-3× speedup)
   - Pruning + distillation for mobile deployment

3. **GPU Acceleration**
   - Move pitch extraction, phoneme alignment to GPU
   - Batch processing (2-8 concurrent users)
   - TensorRT optimization for NVIDIA GPUs

4. **Caching Strategy**
   - Cache reference recitations (6,236 ayahs precomputed)
   - Redis for fast lookup (<100ms)
   - CDN for audio delivery

**Target Latency Breakdown:**
- Preprocessing: 50ms
- Pitch extraction (SwiftF0, GPU): 20ms
- Phoneme alignment (Wav2Vec2-BERT, GPU, INT8): 150ms
- Tajweed validation: 30ms
- Prosodic analysis: 100ms
- Comparison + feedback: 50ms
- **Total:** ~400ms (within <500ms target)

**Infrastructure:**
- GPU server: A100 or T4 (AWS, CoreWeave)
- Redis cluster for caching
- Load balancer for horizontal scaling
- Monitoring: Prometheus + Grafana

**Estimated Cost:**
- GPU server: €500-1,000/month (T4), €3,000-5,000/month (A100)
- Optimization: ONNX conversion (free), TensorRT (free)
- Development time: 3-6 months

---

### Phase 3: Mobile Deployment (Months 13-18)

**Goal:** On-device inference, <300ms latency on modern smartphone

**Key Changes:**

1. **Model Optimization**
   - Quantize to INT8 or even INT4 (TFLite, CoreML)
   - Distill Wav2Vec2-BERT into smaller student model (<100MB)
   - Use lightweight variants: DistilHuBERT, TinyWav2Vec

2. **On-Device Inference**
   - iOS: CoreML with Neural Engine acceleration
   - Android: TFLite with NNAPI or GPU delegate
   - Feature extraction on-device (pitch, formants)
   - Heavy models (comparison, prosody) on server (hybrid approach)

3. **Hybrid Architecture**
   - **On-device:** Phoneme alignment, basic Tajweed (madd, duration)
   - **Server:** Advanced Tajweed (ghunnah, qalqalah), prosody, style
   - Progressive enhancement: Offline mode with basic feedback, online for advanced

4. **Streaming Protocol**
   - WebRTC for low-latency audio streaming
   - Incremental feedback (phoneme-by-phoneme)
   - Visual feedback: Real-time pitch overlay, cursor

**Target Model Sizes:**
- On-device phoneme model: 50-100MB
- On-device Tajweed validators: 10-20MB
- Total app size: <200MB (acceptable for iOS/Android)

**Infrastructure:**
- Mobile SDK (React Native or Flutter)
- Backend API for advanced features
- CDN for reference audio and models

**Estimated Cost:**
- Model distillation: €1,000-2,000 (training compute)
- Mobile development: 3-6 months (iOS + Android)
- Beta testing: €500-1,000 (TestFlight, Play Console)

---

## Technology Stack Summary

### Core Python Libraries

| Purpose | Library | Version | Notes |
|---------|---------|---------|-------|
| Audio I/O | soundfile, librosa | >=0.12.1, >=0.10.0 | Fast loading, resampling |
| Pitch extraction | swift-f0, rmvpe | Latest | SwiftF0 primary, RMVPE fallback |
| Phoneme alignment | transformers, ctc-forced-aligner | >=4.35.0, >=0.1 | Wav2Vec2-BERT fine-tuning |
| Prosodic features | opensmile, praat-parselmouth | >=3.0.1, >=0.4.3 | eGeMAPS, formants |
| Style embeddings | speechbrain | >=0.5.16 | X-vectors |
| Rhythm alignment | tslearn | >=0.6.0 | Soft-DTW |
| ML/DL | torch, scipy, scikit-learn | >=2.0.0, >=1.10.0, >=1.3.0 | Core ML |
| Web framework | fastapi, uvicorn | >=0.100.0, >=0.23.0 | REST API |
| Database | sqlalchemy, psycopg2 | >=2.0.0, >=2.9.0 | PostgreSQL |

### Model Zoo

| Model | Size | Purpose | Download |
|-------|------|---------|----------|
| Wav2Vec2-BERT (fine-tuned) | 2.2GB | Phoneme alignment | HuggingFace Hub |
| SwiftF0 | 0.4MB | Pitch extraction | PyPI |
| RMVPE | 50MB | Pitch fallback | GitHub |
| Ghunnah classifier | 1MB | Ghunnah detection | Custom trained |
| Qalqalah classifier | 5MB | Qalqalah detection | Custom trained |
| Maqam CNN | 10MB | Maqam recognition | Custom trained |
| X-vector model | 20MB | Style embeddings | SpeechBrain |

### Infrastructure

| Component | Technology | Purpose |
|-----------|------------|---------|
| Compute | Lambda Labs, RunPod | GPU training/inference |
| Storage | S3, MinIO | Audio files |
| Database | PostgreSQL | User data, progress |
| Cache | Redis | Precomputed features |
| Monitoring | Prometheus, Grafana | Performance metrics |
| CI/CD | GitHub Actions | Automated testing |

---

## Validation & Testing Strategy

### Accuracy Validation

**Test Sets:**
1. **Phoneme Alignment:**
   - 100 ayahs with manual phoneme boundaries
   - Metrics: PER, boundary precision (20ms/50ms thresholds)
   - Target: PER <1%, 90% within 50ms

2. **Tajweed Rules:**
   - **Madd:** 500 examples (all types), target 99% accuracy
   - **Ghunnah:** 300 examples, target 85% accuracy
   - **Qalqalah:** 200 examples, target 80% accuracy

3. **Prosodic Analysis:**
   - 100 expert-rated pairs (10-point scale per dimension)
   - Correlation: Automated scores vs human ratings
   - Target: r > 0.7 for rhythm, melody, style

**Expert Annotation:**
- Hire 3-5 qualified Qaris for validation
- Inter-rater agreement: Krippendorff's α > 0.75
- Budget: €2,000-3,000 for 200 hours annotation

### Performance Testing

**Latency Benchmarks:**
```python
import time

def benchmark_pipeline(audio_path, num_runs=10):
    timings = {}
    
    for _ in range(num_runs):
        t0 = time.time()
        audio, sr = load_audio(audio_path)
        timings["load"] = time.time() - t0
        
        t0 = time.time()
        pitch = extract_pitch(audio, sr)
        timings["pitch"] = time.time() - t0
        
        t0 = time.time()
        phonemes = align_phonemes(audio, sr, surah=1, ayah=1)
        timings["alignment"] = time.time() - t0
        
        # ... (all modules)
        
        t0 = time.time()
        comparison = compare(student, reference)
        timings["comparison"] = time.time() - t0
    
    # Compute p50, p95, p99
    return {k: np.percentile(v, [50, 95, 99]) for k, v in timings.items()}
```

**Target Benchmarks (Offline, CPU):**
- Preprocessing: p95 <500ms
- Pitch extraction: p95 <300ms
- Phoneme alignment: p95 <2s
- Tajweed validation: p95 <100ms
- Prosody analysis: p95 <500ms
- Comparison: p95 <100ms
- **Total pipeline:** p95 <4s per ayah

**Target Benchmarks (Real-time, GPU):**
- **Total pipeline:** p95 <500ms per chunk

### User Testing

**Alpha Testing (N=10):**
- Internal users, experienced practitioners
- Qualitative feedback on accuracy and usefulness
- Iterate on feedback generation

**Beta Testing (N=50-100):**
- Public beta, diverse proficiency levels
- Quantitative metrics: Engagement, retention, improvement over time
- A/B testing different feedback styles

**Validation Study (N=60-100):**
- Formal study with pre/post measurements
- Compare learning outcomes: Iqrah vs traditional methods
- Publish results for academic validation

---

## Risk Mitigation

### Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Low phoneme alignment accuracy | Medium | High | Fine-tune on domain data, fallback to MMS |
| Ghunnah/Qalqalah low accuracy | High | Medium | Start with rule-based, improve iteratively |
| Real-time latency >500ms | Medium | High | GPU optimization, caching, quantization |
| Model drift over time | Low | Medium | Continuous monitoring, quarterly retraining |
| User audio quality poor | High | Medium | Preprocessing, quality warnings, graceful degradation |

### Operational Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| GPU cost exceeds budget | Medium | High | Caching (70%+ hit rate), spot instances, ARM Graviton |
| Scaling issues (>1000 users) | Low | High | Horizontal scaling, load balancing, auto-scaling |
| Data privacy concerns | Low | High | GDPR compliance, data encryption, user consent |
| Competitor feature parity | Medium | Low | Focus on phoneme-level accuracy (hard to replicate) |

### User Adoption Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Feedback too technical | Medium | Medium | User proficiency levels, simplified language |
| Lack of engagement | Medium | High | Gamification, progress tracking, social features |
| Accuracy skepticism | Low | High | Transparency, expert validation, publish metrics |

---

## Future Enhancements (Post-Phase 3)

### Advanced Features (Years 2-3)

1. **Multi-Reciter Support**
   - Train on multiple Qaris (Husary, Minshawi, Sudais, etc.)
   - Style transfer: "Recite like Qari X"
   - Dialectal variations (Egyptian, Gulf, Levantine)

2. **Adaptive Learning**
   - Personalized practice plans based on weaknesses
   - Spaced repetition for difficult ayahs
   - AI-generated exercises targeting specific rules

3. **Social Features**
   - Peer comparison (anonymized leaderboards)
   - Teacher dashboards (track student progress)
   - Community challenges (group recitation goals)

4. **Advanced Prosody**
   - Emotion recognition (reverence, joy, sadness)
   - Contextual analysis (surah themes)
   - Waqf (pause) optimization recommendations

5. **B2B Features**
   - Bulk user management (Islamic schools)
   - Curriculum integration
   - Detailed analytics reports

### Research Directions

1. **Zero-Shot Tajweed Detection**
   - Use large language models (GPT-4) to explain rules
   - Few-shot learning for rare Tajweed rules

2. **Multimodal Analysis**
   - Video analysis (mouth movements, tongue position)
   - AR overlays for real-time feedback

3. **Synthetic Recitation Generation**
   - TTS for practice audio generation
   - Style control (maqam, tempo, emotion)

---

## Conclusion

This architecture provides a **comprehensive, modular, and scalable** foundation for Iqrah Audio. Key strengths:

✅ **Modularity:** Each component is independent, testable, and swappable  
✅ **SOTA Integration:** Incorporates latest research (Wav2Vec2-BERT, SwiftF0, OpenSMILE)  
✅ **Progressive Rollout:** Start simple (offline), scale to real-time and mobile  
✅ **AI-Agent Friendly:** Clear interfaces, minimal context bleed  
✅ **Validation-First:** Accuracy targets, expert validation, user studies  
✅ **Production-Ready:** Handles edge cases, graceful degradation, monitoring  

**Next Steps:**
1. Implement Module 1-3 (already mostly done)
2. Train Tajweed validators (madd → ghunnah → qalqalah)
3. Integrate prosodic analysis (Module 5-6)
4. Build comparison engine (Module 7)
5. Validate on expert-rated test set
6. Deploy alpha version for user testing

**Commitment:** This design is stable for 3 years. Focus on execution, not redesign.
