# IQRAH AUDIO - SOTA ARCHITECTURE REFERENCE
**Timeline**: 2025-2028 (3-year commitment)
**Phase**: Offline E2E → Real-time → Mobile
**Generated**: 2025-10-23

---

## SYSTEM SPECIFICATIONS

**Targets**:
- Accuracy: 90%+ all basic Tajweed
- Latency: <5s offline, <500ms real-time, <300ms mobile
- Modularity: 8 black-box components
- Rollout: Basic → Advanced → Prosody → Real-time

**Architecture**: M1_PREPROC → M2_PITCH → M3_PHONEME → M4_TAJWEED → M5_VOICE → M6_PROSODY → M7_COMPARE → M8_FEEDBACK

---

## M1: AUDIO PREPROCESSING

**Input**: Raw audio (MP3/WAV/WebM/M4A), optional surah/ayah metadata
**Output**:
```python
{
    "audio_path": str,
    "sample_rate": 16000,
    "duration": float,
    "segments": [{"start": float, "end": float, "confidence": float}],
    "quality_metrics": {
        "snr_db": float,
        "clipping_ratio": float,
        "rms_energy": float,
        "quality_flag": str  # "excellent"/"good"/"poor"
    }
}
```

### M1.1: Audio Loading
- Library: `soundfile` or `librosa`
- Validate format, check corruption
- Extract metadata (original SR, bit depth)

### M1.2: Resampling
- Target: 16kHz (ASR optimal)
- Method: Kaiser windowed sinc interpolation
- Anti-aliasing before downsample

### M1.3: Normalization
- Peak: -1 to +1 dB
- Optional: LUFS for loudness consistency

### M1.4: Noise Reduction (Optional)
- Trigger: SNR < 15 dB
- Method: Spectral subtraction / Wiener filtering
- Library: `noisereduce` or `scipy.signal`
- Risk: Can damage speech if aggressive

### M1.5: VAD (Voice Activity Detection)
- Model: Silero VAD (ONNX, fast)
- Threshold: 0.5 confidence
- Min speech: 250ms
- Min silence: 300ms

### M1.6: Quality Checks
- SNR: Compare signal power voiced vs unvoiced
- Clipping: Count samples at ±1.0
- Dynamic range: Crest factor
- Flags:
  - `excellent`: SNR > 20 dB, no clipping
  - `good`: SNR 10-20 dB, < 1% clipping
  - `poor`: SNR < 10 dB or > 5% clipping

### M1.7: Caching
- Method: SHA256 hash of original
- TTL: Configurable
- Hit/miss logging

**Dependencies**:
```python
soundfile>=0.12.1
librosa>=0.10.0
noisereduce>=3.0.0
silero-vad>=3.1
scipy>=1.10.0
```

**Latency**:
- Offline: 200-500ms per minute
- Real-time: <50ms per chunk

---

## M2: PITCH EXTRACTION

**Input**: Preprocessed audio (16kHz WAV), optional voiced/unvoiced mask
**Output**:
```python
{
    "pitch_hz": np.ndarray,      # F0 contour
    "times": np.ndarray,         # Time stamps
    "confidence": np.ndarray,    # Per-frame confidence
    "voicing": np.ndarray,       # Binary voiced/unvoiced
    "method": str,               # "swiftf0" or "rmvpe"
    "stats": {
        "mean_hz": float,
        "std_hz": float,
        "range_hz": tuple,       # (min, max)
        "voiced_ratio": float    # Proportion voiced
    }
}
```

### M2.1: SwiftF0 (Primary)
**Specs**:
- Accuracy: 91.8%
- Speed: 42× faster than CREPE
- Range: 46-2093 Hz
- Hop: 10ms
- Model: Lightweight CNN

**Code**:
```python
import swiftf0

pitch_tracker = swiftf0.PitchTracker()
pitch_hz, times, confidence = pitch_tracker.predict(audio, sr=16000)
```

**Post-processing**:
- Median filter: 5-frame window
- Linear interpolation: <100ms gaps
- Octave jump removal: >1200 cents change

### M2.2: RMVPE (Fallback)
**Trigger**: Mean SwiftF0 confidence < 0.7
**Method**: Deep U-Net for robust pitch
**Latency**: ~3× slower than SwiftF0

**Code**:
```python
from rmvpe import RMVPE

model = RMVPE("rmvpe.pt", device="cuda")
pitch_hz = model.infer_from_audio(audio, sr=16000)
```

### M2.3: Confidence Weighting
Inverse variance weighting when both available:
```python
w_swift = 1 / (1 - conf_swift + 1e-6)
w_rmvpe = 1 / (1 - conf_rmvpe + 1e-6)
pitch_final = (w_swift * pitch_swift + w_rmvpe * pitch_rmvpe) / (w_swift + w_rmvpe)
```

### M2.4: Smoothing
- Savitzky-Golay filter: polynomial order 3, window 51ms
- Removes jitter while preserving contour shape

**Dependencies**:
```python
swift-f0>=1.0.0
rmvpe @ git+https://github.com/yxlllc/RMVPE
scipy>=1.10.0
```

**Latency**:
- SwiftF0: 50-100ms per minute (GPU), 200-300ms (CPU)
- RMVPE: 150-300ms per minute (GPU), 600-1000ms (CPU)

---

## M3: PHONEME ALIGNMENT

**Input**: Preprocessed audio (16kHz), surah, ayah
**Output**:
```python
{
    "phonemes": [
        {
            "phoneme": str,          # Buckwalter notation
            "start": float,          # Seconds
            "end": float,
            "confidence": float,     # CTC posterior
            "tajweed_rule": str,     # Mapped rule
            "gop_score": float       # Goodness of Pronunciation
        }
    ],
    "alignment_method": str,
    "quality_score": float
}
```

### M3.1: Wav2Vec2-BERT Fine-Tuning

**Base Model**: `facebook/w2v-bert-2.0`

**Training Protocol**:
1. **Stage 1**: Continue pretraining on 50-100h Tarteel audio
   - Objective: Masked prediction (MLM-style)
   - LR: 1e-5
   - Batch: 4 (grad accumulation 8)
   - Epochs: 3
   - Warmup: 500 steps

2. **Stage 2**: Fine-tune CTC head on phoneme labels
   - Labels: MSA Phonetiser output
   - LR: 3e-5
   - Batch: 8
   - Epochs: 5
   - Decoder: CTC loss

**Target**: PER < 1% (SOTA: 0.16% with QPS encoding)

**Dataset**: Tarteel-ai-everyayah (https://huggingface.co/datasets/Salama1429/tarteel-ai-everyayah-Quran)

**Code**:
```python
from transformers import Wav2Vec2ForCTC, Wav2Vec2Processor

model = Wav2Vec2ForCTC.from_pretrained("models/wav2vec2_bert_quranic")
processor = Wav2Vec2Processor.from_pretrained("models/wav2vec2_bert_quranic")

inputs = processor(audio, sampling_rate=16000, return_tensors="pt")
logits = model(**inputs).logits
predicted_ids = torch.argmax(logits, dim=-1)
transcription = processor.batch_decode(predicted_ids)
```

### M3.2: CTC Forced Alignment

**Library**: `ctc-forced-aligner>=0.1`

**Method**:
1. Get CTC posteriors from Wav2Vec2-BERT
2. Windowed alignment per word (±500ms window)
3. HMM Viterbi for boundary smoothing
4. GPU acceleration via CUDA

**Code**:
```python
from ctc_forced_aligner import align

alignment = align(
    audio=audio,
    sample_rate=16000,
    transcript=phoneme_sequence,
    model=wav2vec2_model,
    window_size=500  # ms
)
# alignment = [{"phoneme": "a", "start": 0.12, "end": 0.25, ...}, ...]
```

**Boundary Accuracy Target**: 90% within 50ms

### M3.3: Tajweed Mapping

**Resource**: `quran-phoneme-tajweed.json` (6,236 ayahs × phonemes × rules)

**Fuzzy Matching**:
- Edit distance < 2 for mapping aligned phonemes to reference
- Rule label assignment from JSON
- Expected duration lookup

**Code**:
```python
def map_tajweed_rules(aligned_phonemes, surah, ayah):
    reference = tajweed_json[surah][ayah]
    for i, phon in enumerate(aligned_phonemes):
        best_match = fuzzy_match(phon["phoneme"], reference, max_dist=2)
        phon["tajweed_rule"] = best_match["rule"]
        phon["expected_duration"] = best_match["duration"]
    return aligned_phonemes
```

### M3.4: GOP Scoring

**Method**: Goodness of Pronunciation via CTC posteriors

**Formula**:
```python
GOP = log P(phoneme | audio, model)
# Higher = better pronunciation
```

**Threshold**: GOP < -5 typically indicates mispronunciation

**Code**:
```python
def compute_gop(phoneme, audio_segment, model):
    posteriors = model(audio_segment).logits
    phoneme_id = processor.tokenizer.convert_tokens_to_ids(phoneme)
    log_prob = torch.log_softmax(posteriors, dim=-1)[:, phoneme_id].mean()
    return log_prob.item()
```

### M3.5: Quality Validation

**Checks**:
- Mean confidence > 0.7
- Phoneme count ±10% of expected
- Duration sanity: 20-500ms per phoneme
- Generate warnings if failed

**Dependencies**:
```python
transformers>=4.35.0
torch>=2.0.0
ctc-forced-aligner>=0.1
```

**Latency**:
- Fine-tuned inference: 1-2s per ayah (GPU), 3-5s (CPU)
- Alignment: 200-500ms


### M3.5: CONTENT ACCURACY VERIFICATION (ASR GATEKEEPER)

**Purpose**: Verify the user recited the correct verse before analyzing pronunciation quality. Prevents catastrophic failure mode of providing pronunciation feedback on wrong content.

**Architecture**: Two-stage decision gate
1. **Transcription**: ASR model generates text hypothesis
2. **Verification**: Compare transcript to ground-truth reference
3. **Gate**: WER threshold determines analysis path

### ASR Model Selection

**Phase 1 (Prototyping)**:
- **Primary Option**: `obadx/recitation-segmenter-v2` (Wav2Vec2-BERT, Quranic fine-tuned)
  - Advantages: Pre-trained, fast inference, community-validated
  - Expected WER: 5-8% on learner speech
- **Fallback**: Whisper-large-v3 with Arabic model
  - Advantages: Lower WER, multilingual robustness
  - Disadvantages: Higher latency, may need LoRA fine-tuning

**Phase 2 (Production)**:
- Custom Wav2Vec2-BERT trained on Tarteel (850+ hours) or ArDA datasets
- Target: <1% PER on expert reciters, <3% WER on learners

### Algorithm

```python
def verify_content(audio: np.ndarray, reference_text: str) -> dict:
    """
    Verify that the recited audio matches the expected Quranic text.

    Args:
        audio: Preprocessed audio array (16kHz, mono)
        reference_text: Ground-truth Quranic text for the expected verse

    Returns:
        Dictionary with WER, transcript, and error details
    """
    # 1. Transcribe using ASR
    raw_transcript = asr_model.transcribe(audio)

    # 2. Normalize both texts (CRITICAL)
    def normalize_arabic(text: str) -> List[str]:
        """
        Normalize Arabic text to prevent false mismatches.

        Operations:
        - Remove tashkeel (diacritics): ً ٌ ٍ َ ُ ِ ّ ْ
        - Standardize alef forms: أ إ آ → ا
        - Remove tatweel: ـ
        - Normalize hamza variations
        - Split into word list
        """
        # Remove diacritics
        text = remove_diacritics(text)
        # Standardize alef forms: أ إ آ → ا
        text = text.replace('أ', 'ا').replace('إ', 'ا').replace('آ', 'ا')
        # Remove tatweel: ـ
        text = text.replace('ـ', '')
        # Remove hamza variations
        text = normalize_hamza(text)
        return text.split()

    transcript_words = normalize_arabic(raw_transcript)
    reference_words = normalize_arabic(reference_text)

    # 3. Compute edit distance (Levenshtein)
    ops = levenshtein_operations(reference_words, transcript_words)
    substitutions = sum(1 for op in ops if op.type == 'substitute')
    deletions = sum(1 for op in ops if op.type == 'delete')
    insertions = sum(1 for op in ops if op.type == 'insert')

    # 4. Calculate WER
    wer = (substitutions + deletions + insertions) / len(reference_words)

    # 5. Extract error details
    errors = [
        {
            "type": op.type,
            "reference_word": op.ref_word,
            "recited_word": op.hyp_word,
            "position": op.position
        }
        for op in ops if op.type != 'correct'
    ]

    return {
        "wer": wer,
        "transcript": raw_transcript,
        "normalized_transcript": ' '.join(transcript_words),
        "errors": errors
    }
```

### Output Schema

```python
{
    "wer": float,                    # 0.0 to 1.0
    "transcript": str,               # Raw ASR output (with diacritics if model produces them)
    "normalized_transcript": str,    # After normalization
    "errors": [
        {
            "type": "deletion" | "insertion" | "substitution",
            "reference_word": str | None,
            "recited_word": str | None,
            "position": int              # Word index in reference text
        }
    ]
}
```

### WER Threshold Strategy

**Adaptive Thresholding**:
```python
WER_THRESHOLD_STRICT = 0.05    # 5% - High confidence, proceed with full analysis
WER_THRESHOLD_PERMISSIVE = 0.08  # 8% - Borderline, proceed with caution flag
WER_THRESHOLD_FAIL = 0.08      # >8% - Stop analysis, report content errors only

if wer <= WER_THRESHOLD_STRICT:
    confidence = "high"
elif wer <= WER_THRESHOLD_PERMISSIVE:
    confidence = "medium"  # Proceed but warn user
else:
    confidence = "fail"    # Stop, report content errors
```

**Rationale**:
- Native-level recitation: WER <3%
- Learner with correct content but pronunciation issues: WER 3-7%
- Severe content errors (wrong verse, skipped words): WER >10%

### Dependencies

```python
transformers>=4.35.0      # For Wav2Vec2-BERT or Whisper models
torch>=2.0.0
python-Levenshtein>=0.21.0  # Fast edit distance computation
pyarabic>=0.6.15            # Arabic text normalization utilities
```

### Testing Requirements

**Unit Tests** (required before integration):

```python
def test_content_verification_exact_match():
    """Test: Perfect recitation should yield WER ~0%"""
    audio = load_test_audio("fatiha_verse1_correct.wav")
    reference = "بِسْمِ اللَّهِ الرَّحْمَٰنِ الرَّحِيمِ"
    result = verify_content(audio, reference)
    assert result['wer'] < 0.01
    assert len(result['errors']) == 0

def test_content_verification_wrong_verse():
    """Test: Wrong verse should yield high WER"""
    audio = load_test_audio("fatiha_verse2.wav")  # User recites verse 2
    reference = "بِسْمِ اللَّهِ الرَّحْمَٰنِ الرَّحِيمِ"  # Reference is verse 1
    result = verify_content(audio, reference)
    assert result['wer'] > 0.50
    assert len(result['errors']) > 0

def test_normalization_alef_variants():
    """Test: Normalization handles alef variants correctly"""
    text1 = "أَحْمَد"
    text2 = "احمد"
    assert normalize_arabic(text1) == normalize_arabic(text2)

def test_normalization_diacritics():
    """Test: Diacritics are removed consistently"""
    with_diacritics = "بِسْمِ اللَّهِ"
    without_diacritics = "بسم الله"
    assert normalize_arabic(with_diacritics) == normalize_arabic(without_diacritics)
```

**Integration Tests**:

```python
def test_gate_blocks_high_wer():
    """Test: Comparison engine stops analysis when WER > 8%"""
    # Simulate audio with WER = 12%
    result = comparison_engine.compare(wrong_verse_audio, reference_text)
    assert result['analysis_type'] == "content_error"
    assert 'tajweed_violations' not in result  # Should NOT run pronunciation

def test_gate_proceeds_low_wer():
    """Test: Comparison engine proceeds when WER < 5%"""
    result = comparison_engine.compare(correct_audio, reference_text)
    assert result['analysis_type'] == "pronunciation_and_prosody"
    assert 'component_scores' in result  # Should run full analysis
```

---

## M4: TAJWEED VALIDATORS

### M4.1: Madd Duration Validator

**Accuracy Target**: 99%+

**Algorithm**: Rule-based duration checking

**Madd Types & Durations** (harakat = short vowel duration):
| Type          | Arabic   | Duration      | Tolerance |
| ------------- | -------- | ------------- | --------- |
| Madd Tabi'i   | مد طبيعي | 2 harakat     | ±20%      |
| Madd Lazim    | مد لازم  | 6 harakat     | ±20%      |
| Madd Muttasil | مد متصل  | 4-5 harakat   | ±20%      |
| Madd Munfasil | مد منفصل | 2-5 harakat   | ±20%      |
| Madd 'Arid    | مد عارض  | 2/4/6 harakat | ±20%      |

**Harakat Calibration**:
- Estimate from corpus: Mean vowel duration ≈ 80-120ms
- Use: 100ms as baseline harakat

**Code**:
```python
class MaddDurationValidator:
    DURATIONS = {
        "madd_tabii": (2, 0.2),     # (harakat, tolerance)
        "madd_lazim": (6, 0.2),
        "madd_muttasil": (4.5, 0.2),
        "madd_munfasil": (3.5, 0.3),
        "madd_arid": (4, 0.3)
    }

    def __init__(self, harakat_ms=100):
        self.harakat_ms = harakat_ms

    def validate(self, phonemes):
        violations = []
        for p in phonemes:
            if "madd" in p["tajweed_rule"]:
                expected_harakat, tolerance = self.DURATIONS[p["tajweed_rule"]]
                expected_ms = expected_harakat * self.harakat_ms
                actual_ms = (p["end"] - p["start"]) * 1000

                deviation = abs(actual_ms - expected_ms) / expected_ms

                if deviation > tolerance:
                    severity = "critical" if deviation > 2*tolerance else "moderate"
                    violations.append({
                        "phoneme_idx": p["idx"],
                        "severity": severity,
                        "expected": f"{expected_harakat} harakat ({expected_ms:.0f}ms)",
                        "actual": f"{actual_ms:.0f}ms",
                        "confidence": 1.0 - min(deviation, 1.0),
                        "feedback": self._generate_feedback(p, expected_ms, actual_ms)
                    })
        return violations
```

**Latency**: <10ms per phoneme (pure NumPy)

### M4.2: Ghunnah Validator

**Accuracy Target**: 85%+

**Algorithm**: Formant analysis + MLP classifier

**Acoustic Correlates**:
- Nasal formants: F1 ≈ 250-350 Hz, F2 ≈ 2200-2600 Hz
- Nasal energy: 250-350 Hz band elevated
- Spectral flatness: Higher (diffuse energy)
- Duration: Typically 150-300ms

**Feature Extraction**:
```python
import parselmouth

def extract_ghunnah_features(audio, start, end, sr=16000):
    segment = audio[int(start*sr):int(end*sr)]
    sound = parselmouth.Sound(segment, sr)

    # Formants via Praat
    formants = sound.to_formant_burg()
    f1_hz = formants.get_value_at_time(1, (start+end)/2)
    f2_hz = formants.get_value_at_time(2, (start+end)/2)
    f3_hz = formants.get_value_at_time(3, (start+end)/2)

    # Nasal energy (250-350Hz)
    from scipy.signal import butter, filtfilt
    b, a = butter(4, [250, 350], btype='band', fs=sr)
    nasal_band = filtfilt(b, a, segment)
    nasal_energy_db = 10 * np.log10(np.mean(nasal_band**2) + 1e-10)

    # Spectral flatness
    from scipy.signal import get_window
    spec = np.abs(np.fft.rfft(segment * get_window('hann', len(segment))))
    flatness = np.exp(np.mean(np.log(spec + 1e-10))) / (np.mean(spec) + 1e-10)

    return {
        "f1_hz": f1_hz,
        "f2_hz": f2_hz,
        "f3_hz": f3_hz,
        "nasal_energy_db": nasal_energy_db,
        "spectral_flatness": flatness,
        "duration_s": end - start
    }
```

**Classifier Training**:
```python
from sklearn.neural_network import MLPClassifier

# Train on 1000+ labeled examples
clf = MLPClassifier(hidden_layer_sizes=(64, 32), max_iter=500)
clf.fit(X_train, y_train)  # X = features, y = ghunnah/no-ghunnah

import joblib
joblib.dump(clf, "models/ghunnah_classifier.pkl")
```

**Inference**:
```python
class GhunnahValidator:
    def __init__(self, model_path="models/ghunnah_classifier.pkl"):
        self.clf = joblib.load(model_path)

    def validate(self, phonemes, audio, sr=16000):
        violations = []
        for p in phonemes:
            if p["tajweed_rule"] == "ghunnah":
                features = extract_ghunnah_features(audio, p["start"], p["end"], sr)
                X = np.array([[
                    features["f1_hz"], features["f2_hz"], features["f3_hz"],
                    features["nasal_energy_db"], features["spectral_flatness"],
                    features["duration_s"]
                ]])

                prob = self.clf.predict_proba(X)[0, 1]  # P(ghunnah)

                if prob < 0.7:
                    violations.append({
                        "phoneme_idx": p["idx"],
                        "severity": "moderate" if prob < 0.5 else "minor",
                        "expected": "Nasal resonance (ghunnah)",
                        "actual": f"Confidence: {prob:.2f}",
                        "confidence": prob,
                        "feedback": f"Nasal resonance too weak. Hum through nose while pronouncing."
                    })
        return violations
```

**Dependencies**:
```python
praat-parselmouth>=0.4.3
scikit-learn>=1.3.0
librosa>=0.10.0
```

**Latency**: 50-100ms per phoneme (formant extraction slow)

**Training Data**: 500-1000 labeled examples (ghunnah present/absent)

### M4.3: Qalqalah Validator

**Accuracy Target**: 80-85%

**Algorithm**: Burst detection + SVM classifier

**Qalqalah Letters** (Buckwalter): q, T, b, j, d (with sukoon)

**Acoustic Correlates**:
- Burst detection: Sharp transient at release
- ZCR: High during burst
- Spectral centroid: Higher (brightness)
- Energy spike: Localized increase

**Feature Extraction**:
```python
def extract_qalqalah_features(audio, start, end, sr=16000):
    segment = audio[int(start*sr):int(end*sr)]

    # Zero-crossing rate
    zcr = librosa.feature.zero_crossing_rate(segment)[0]

    # Spectral centroid
    centroid = librosa.feature.spectral_centroid(y=segment, sr=sr)[0]

    # Energy envelope
    rms = librosa.feature.rms(y=segment)[0]

    # Burst detection
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

**Classifier**:
```python
from sklearn.svm import SVC

clf = SVC(kernel='rbf', probability=True)
clf.fit(X_train, y_train)
joblib.dump(clf, "models/qalqalah_classifier.pkl")
```

**Validation**:
```python
class QalqalahValidator:
    QALQALAH_LETTERS = {'q', 'T', 'b', 'j', 'd'}

    def validate(self, phonemes, audio, sr=16000):
        violations = []
        for p in phonemes:
            if (p["phoneme"] in self.QALQALAH_LETTERS and
                p["tajweed_rule"] == "qalqalah"):

                features = extract_qalqalah_features(audio, p["start"], p["end"], sr)
                X = self._features_to_array(features)
                prob = self.clf.predict_proba(X)[0, 1]

                if prob < 0.6 or not features["has_burst"]:
                    violations.append({
                        "phoneme_idx": p["idx"],
                        "severity": "moderate",
                        "expected": "Sharp burst with echo",
                        "actual": f"Burst confidence: {prob:.2f}",
                        "confidence": prob,
                        "feedback": "Qalqalah requires short, explosive release"
                    })
        return violations
```

**Latency**: 30-50ms per phoneme

**Training Data**: 300-500 qalqalah examples, 500-1000 negatives

### M4.4: Complex Rules (Deferred Phase 3)

**Idghaam** (Assimilation):
- Check merged phoneme duration > sum individual
- Smooth formant transition

**Ikhfaa** (Concealment):
- Intermediate nasal formant
- Duration slightly longer

**Iqlaab** (Conversion):
- Acoustic transformation validation

---

## M5: VOICE QUALITY ANALYSIS

**Input**: Preprocessed audio (16kHz), phoneme alignment
**Output**:
```python
{
    "vibrato": {
        "rate_hz": float,           # 4-7 typical
        "extent_semitones": float,  # 0.5-2 typical
        "regularity": float         # 0-1
    },
    "breathiness": {
        "h1_h2_db": float,          # >5 dB = breathy
        "hnr_db": float,            # <10 dB = breathy/hoarse
        "cpp": float                # Cepstral peak prominence
    },
    "roughness": {
        "jitter_percent": float,    # >1.04% = pathological
        "shimmer_percent": float    # >3.81% = pathological
    },
    "timbre": {
        "spectral_centroid_hz": float,   # 2-4k typical
        "spectral_flux": float,
        "spectral_rolloff_hz": float,    # 85% energy point
        "formants": {
            "f1_hz": float,
            "f2_hz": float,
            "f3_hz": float,
            "f4_hz": float
        }
    },
    "embeddings": {
        "x_vector": np.ndarray,      # 512-d
        "wav2vec2_cls": np.ndarray   # 768-d
    }
}
```

### M5.1: OpenSMILE eGeMAPS

**Features**: 88 standardized prosodic dimensions

**Code**:
```python
import opensmile

smile = opensmile.Smile(
    feature_set=opensmile.FeatureSet.eGeMAPSv02,
    feature_level=opensmile.FeatureLevel.Functionals
)

features = smile.process_file("recitation.wav")
# Returns DataFrame with 88 features
```

**Includes**: Jitter, shimmer, HNR, formants, MFCC, spectral features

**Latency**: ~200ms per minute (CPU)

### M5.2: Vibrato Detection

**Code**:
```python
def detect_vibrato(pitch_contour, times):
    """Analyze vibrato characteristics"""
    # Bandpass filter pitch (2-15 Hz vibrato range)
    from scipy.signal import butter, filtfilt
    b, a = butter(4, [2, 15], btype='band', fs=100)  # 100 Hz pitch sampling
    filtered = filtfilt(b, a, pitch_contour)

    # Autocorrelation for rate
    autocorr = np.correlate(filtered, filtered, mode='full')
    autocorr = autocorr[len(autocorr)//2:]
    peaks = find_peaks(autocorr)[0]
    if len(peaks) > 0:
        rate_hz = 100 / peaks[0]
    else:
        rate_hz = 0

    # Extent (amplitude of oscillation)
    extent_semitones = 12 * np.log2(np.ptp(filtered) + 1e-6)

    # Regularity (CV of peak intervals)
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

### M5.3: Breathiness Features

**Code**:
```python
def extract_breathiness(audio, sr=16000):
    """Compute H1-H2, HNR, CPP"""
    sound = parselmouth.Sound(audio, sr)

    # H1-H2 (first two harmonics difference)
    harmonicity = sound.to_harmonicity()
    h1_h2 = harmonicity.values[:, :2].diff(axis=1).mean()

    # HNR (Harmonic-to-Noise Ratio)
    hnr = harmonicity.values.mean()

    # CPP (Cepstral Peak Prominence)
    pitch = sound.to_pitch()
    cpp = pitch.to_cpp()

    return {
        "h1_h2_db": h1_h2,
        "hnr_db": hnr,
        "cpp": cpp
    }
```

### M5.4: X-Vector Embeddings

**Code**:
```python
from speechbrain.pretrained import EncoderClassifier

classifier = EncoderClassifier.from_hparams(
    source="speechbrain/spkrec-xvect-voxceleb",
    savedir="models/xvector"
)

embedding = classifier.encode_batch(audio_tensor)  # 512-d
```

### M5.5: Wav2Vec2 [CLS] Token

**Code**:
```python
from transformers import Wav2Vec2Model, Wav2Vec2Processor

model = Wav2Vec2Model.from_pretrained("facebook/wav2vec2-base")
processor = Wav2Vec2Processor.from_pretrained("facebook/wav2vec2-base")

inputs = processor(audio, sampling_rate=16000, return_tensors="pt")
outputs = model(**inputs)

embedding = outputs.last_hidden_state[:, 0, :].numpy()  # 768-d CLS token
```

**Dependencies**:
```python
opensmile>=3.0.1
praat-parselmouth>=0.4.3
speechbrain>=0.5.16
transformers>=4.35.0
```

**Latency**:
- OpenSMILE: 200ms/min (CPU)
- Parselmouth: 100-200ms/phoneme
- X-vectors: 500ms/ayah (GPU), 2-3s (CPU)
- Wav2Vec2: 300ms/ayah (GPU), 1-2s (CPU)

---

## M6: PROSODIC ANALYSIS

**Output**:
```python
{
    "rhythm": {
        "tempo_sps": float,              # Syllables per second
        "nPVI": float,                   # Pairwise variability index
        "varco": float,                  # Coefficient of variation
        "ioi_distribution": np.ndarray,  # Inter-onset intervals
        "rhythm_class": str              # "stress-timed"/"syllable-timed"
    },
    "melody": {
        "fujisaki_params": {
            "phrase_commands": [{"time": float, "amplitude": float, "duration": float}],
            "accent_commands": [{"time": float, "amplitude": float}]
        },
        "declination": {
            "slope": float,              # Hz/second
            "r_squared": float
        },
        "tilt_features": {
            "rising_ratio": float,
            "falling_ratio": float,
            "convex_ratio": float,
            "level_ratio": float
        },
        "maqam": {
            "predicted": str,
            "confidence": float,
            "probability_distribution": dict
        }
    },
    "style": {
        "x_vector": np.ndarray,          # 512-d
        "gst_tokens": np.ndarray,        # Optional
        "style_label": str               # "Egyptian", "Gulf", etc.
    }
}
```

### M6.1: Advanced Rhythm Analysis

**nPVI (Pairwise Variability Index)**:
```python
def compute_npvi(durations):
    """
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

**Varco**:
```python
def compute_varco(ioi_intervals):
    """Coefficient of variation. Lower = more isochronous"""
    return 100 * np.std(ioi_intervals) / np.mean(ioi_intervals)
```

**Isochrony Scoring**:
```python
def score_isochrony(ioi_intervals, target_ioi=0.3):
    """Score 0-100 (higher = more regular)"""
    deviations = np.abs(ioi_intervals - target_ioi) / target_ioi
    score = 100 * (1 - np.mean(np.minimum(deviations, 1.0)))
    return score
```

**Soft-DTW Integration**:
```python
from tslearn.metrics import soft_dtw

dtw_cost = soft_dtw(student_rhythm, reference_rhythm, gamma=1.0)
dtw_score = 100 * np.exp(-dtw_cost / path_length)
```

**Wasserstein Distance**:
```python
from scipy.stats import wasserstein_distance

rhythm_dist = wasserstein_distance(student_ioi, reference_ioi)
```

**Latency**: <50ms per ayah

### M6.2: Melody Analysis

**Fujisaki Model Decomposition**:
```python
from scipy.optimize import minimize

def fit_fujisaki(f0_contour, times):
    """
    F0(t) = F_b * exp(A_p(t) + Σ A_a(t))
    F_b = baseline
    A_p = phrase component
    A_a = accent components
    """
    def phrase_component(t, params):
        amp, onset, duration = params
        if t < onset or t > onset + duration:
            return 0
        alpha = 0.9  # Phrase command decay rate
        return amp * (1 - np.exp(-alpha * (t - onset)))

    def accent_component(t, params):
        amp, onset = params
        beta = 20  # Accent command decay rate
        if t < onset:
            return 0
        return amp * np.exp(-beta * (t - onset))

    def fujisaki_model(params):
        f_b = params[0]
        phrase_params = params[1:4]
        accent_params = params[4:].reshape(-1, 2)

        f0_pred = []
        for t in times:
            phrase = phrase_component(t, phrase_params)
            accents = sum(accent_component(t, acc) for acc in accent_params)
            f0_pred.append(f_b * np.exp(phrase + accents))

        return np.sum((f0_contour - f0_pred)**2)

    # Optimize
    initial_params = [100, 10, 0, 1, 5, 0.5, 10, 0.3]  # Example
    result = minimize(fujisaki_model, initial_params)

    return {
        "baseline": result.x[0],
        "phrase_commands": result.x[1:4],
        "accent_commands": result.x[4:].reshape(-1, 2)
    }
```

**Declination Modeling**:
```python
def fit_declination(f0_contour, times):
    """Exponential decay fit"""
    from scipy.optimize import curve_fit

    def exp_decay(t, a, b, c):
        return a * np.exp(-b * t) + c

    params, _ = curve_fit(exp_decay, times, f0_contour)
    slope_hz_per_sec = -params[0] * params[1]

    # R-squared
    predicted = exp_decay(times, *params)
    ss_res = np.sum((f0_contour - predicted)**2)
    ss_tot = np.sum((f0_contour - np.mean(f0_contour))**2)
    r_squared = 1 - (ss_res / ss_tot)

    return {"slope": slope_hz_per_sec, "r_squared": r_squared}
```

**Tilt Parametrization**:
```python
def compute_tilt(f0_contour):
    """Classify contour shape"""
    rising = np.sum(np.diff(f0_contour) > 0)
    falling = np.sum(np.diff(f0_contour) < 0)
    total = len(f0_contour) - 1

    return {
        "rising_ratio": rising / total,
        "falling_ratio": falling / total,
        "level_ratio": 1 - (rising + falling) / total
    }
```

**Maqam Classification**:
```python
# Train CNN on Maqam478 dataset
import torch.nn as nn

class MaqamCNN(nn.Module):
    def __init__(self, num_maqams=10):
        super().__init__()
        self.conv = nn.Sequential(
            nn.Conv1d(1, 32, kernel_size=3),
            nn.ReLU(),
            nn.MaxPool1d(2),
            nn.Conv1d(32, 64, kernel_size=3),
            nn.ReLU(),
            nn.MaxPool1d(2)
        )
        self.fc = nn.Linear(64 * 10, num_maqams)  # Adjust based on input length

    def forward(self, x):
        x = self.conv(x)
        x = x.view(x.size(0), -1)
        return self.fc(x)

# Train on chroma features + MFCCs
# Dataset: Maqam478 (https://github.com/MTG/otmm_makam_recognition_dataset)
```

**Latency**: 100-200ms per ayah


## M7: COMPARISON ENGINE (ORCHESTRATOR)

**Purpose**: Coordinate all analysis modules with intelligent gating logic. Implements the "fail-fast" principle: detect content errors early, analyze pronunciation only when content is correct.

### Architecture: Two-Path Decision Flow

```
┌─────────────────┐
│  User Audio +   │
│ Reference Text  │
└────────┬────────┘
         │
         ▼
┌─────────────────────────┐
│ M3.5: Content Verify    │
│ (ASR → WER calculation) │
└────────┬────────────────┘
         │
         ▼
    ┌────────┐
    │WER > 8%│───YES──→ STOP: Report content errors only
    └───┬────┘           Return: {type: "content_error", errors: [...]}
        │
        NO
        │
    ┌───▼────┐
    │5% < WER│───YES──→ PROCEED with "medium confidence" flag
    │  ≤ 8%  │           Set: analysis_confidence = "medium"
    └───┬────┘
        │
        NO (WER ≤ 5%)
        │
        ▼
    analysis_confidence = "high"
        │
        ▼
┌─────────────────────────────────────┐
│ FULL PRONUNCIATION ANALYSIS         │
│                                     │
│ 1. M3: Forced Alignment (MFA)      │
│    └─ Precise phoneme boundaries    │
│                                     │
│ 2. M4: Tajweed Validation          │
│    ├─ Madd (duration-based)        │
│    ├─ Ghunnah (formant + MLP)      │
│    └─ Qalqalah (burst + SVM)       │
│                                     │
│ 3. M2,M5,M6: Prosody & Voice       │
│    ├─ Pitch extraction (SwiftF0)   │
│    ├─ Voice quality (OpenSMILE)    │
│    └─ Prosody (rhythm, melody)     │
│                                     │
│ 4. Fusion: Weighted scores         │
│    ├─ Tajweed: 40%                 │
│    ├─ Prosody: 30%                 │
│    ├─ Pronunciation: 20%           │
│    └─ Voice quality: 10%           │
└─────────────────────────────────────┘
         │
         ▼
┌─────────────────────────┐
│ M8: Feedback Generation │
└─────────────────────────┘
```

### Implementation

```python
class ComparisonEngine:
    """
    Orchestrates the full recitation analysis pipeline with content gating.

    Implements two-path decision logic:
    - PATH A: WER > 8% → Stop, report content errors only
    - PATH B: WER ≤ 8% → Proceed with full pronunciation analysis
    """

    def __init__(self):
        # Module initialization
        self.content_verifier = ContentVerifier()  # M3.5
        self.forced_aligner = MFA()                # M3 (Montreal Forced Aligner)
        self.tajweed_validators = {
            'madd': MaddValidator(),
            'ghunnah': GhunnahClassifier(),
            'qalqalah': QalqalahDetector()
        }
        self.prosody_analyzer = ProsodyAnalyzer()  # M2, M5, M6

        # Configurable thresholds (can be tuned per-user or per-surah)
        self.WER_THRESHOLD_FAIL = 0.08    # Stop analysis
        self.WER_THRESHOLD_WARN = 0.05    # Proceed with caution

    def compare(self,
                student_audio: np.ndarray,
                reference_text: str,
                surah: int,
                ayah: int) -> dict:
        """
        Perform full recitation comparison with content gating.

        Args:
            student_audio: Preprocessed audio (16kHz, mono)
            reference_text: Ground-truth Quranic text
            surah: Surah number (1-114)
            ayah: Ayah number within surah

        Returns:
            Analysis result (content error OR full pronunciation analysis)
        """

        # ============================================================
        # STAGE 1: CONTENT VERIFICATION (THE GATEKEEPER)
        # ============================================================
        content_result = self.content_verifier.verify(student_audio, reference_text)
        wer = content_result['wer']

        # ============================================================
        # DECISION GATE: EVALUATE WER
        # ============================================================
        if wer > self.WER_THRESHOLD_FAIL:
            # --------------------------------------------------------
            # PATH A: FAILED GATE - Report content errors only
            # --------------------------------------------------------
            return {
                "overall_score": 0.0,
                "analysis_type": "content_error",
                "wer": wer,
                "transcript": content_result['transcript'],
                "word_errors": content_result['errors'],
                "feedback": self._generate_content_feedback(content_result['errors']),
                "recommendation": "Review the correct verse text before attempting pronunciation analysis"
            }

        # --------------------------------------------------------
        # PATH B: PASSED GATE - Proceed with full analysis
        # --------------------------------------------------------
        analysis_confidence = "high" if wer <= self.WER_THRESHOLD_WARN else "medium"
        warnings = []
        if analysis_confidence == "medium":
            warnings.append(
                f"Content accuracy is borderline (WER={wer:.1%}). "
                "Some pronunciation feedback may be inaccurate. "
                "Consider re-recording for more reliable analysis."
            )

        # ============================================================
        # STAGE 2: FORCED ALIGNMENT (using ground-truth text)
        # ============================================================
        alignment = self.forced_aligner.align(
            audio=student_audio,
            transcript=reference_text,  # Use reference, NOT ASR output
            language='ar'
        )

        # ============================================================
        # STAGE 3: TAJWEED RULE VALIDATION
        # ============================================================
        tajweed_results = {}
        for rule_name, validator in self.tajweed_validators.items():
            tajweed_results[rule_name] = validator.validate(
                phonemes=alignment['phonemes'],
                audio=student_audio
            )

        # ============================================================
        # STAGE 4: PROSODY & VOICE ANALYSIS
        # ============================================================
        prosody_results = self.prosody_analyzer.analyze(
            audio=student_audio,
            phonemes=alignment['phonemes'],
            reference_pitch=self._load_reference_pitch(surah, ayah)
        )

        # ============================================================
        # STAGE 5: SCORE FUSION
        # ============================================================
        component_scores = {
            "tajweed": self._score_tajweed(tajweed_results),      # Weight: 40%
            "prosody": self._score_prosody(prosody_results),      # Weight: 30%
            "pronunciation": self._score_pronunciation(alignment), # Weight: 20%
            "voice_quality": self._score_voice(prosody_results)   # Weight: 10%
        }

        overall_score = (
            0.40 * component_scores['tajweed'] +
            0.30 * component_scores['prosody'] +
            0.20 * component_scores['pronunciation'] +
            0.10 * component_scores['voice_quality']
        )

        # ============================================================
        # RETURN FULL ANALYSIS
        # ============================================================
        return {
            "overall_score": overall_score,
            "analysis_type": "pronunciation_and_prosody",
            "analysis_confidence": analysis_confidence,
            "wer": wer,
            "transcript": content_result['transcript'],
            "component_scores": component_scores,
            "tajweed_violations": self._extract_violations(tajweed_results),
            "prosody_feedback": prosody_results['feedback'],
            "warnings": warnings,
            "timestamp": datetime.now().isoformat()
        }

    def _generate_content_feedback(self, errors: List[dict]) -> str:
        """Generate user-friendly feedback for content errors"""
        if not errors:
            return "No content errors detected."

        error_summary = []
        for err in errors[:5]:  # Show top 5 errors
            if err['type'] == 'deletion':
                error_summary.append(f"Missing word: '{err['reference_word']}'")
            elif err['type'] == 'insertion':
                error_summary.append(f"Extra word: '{err['recited_word']}'")
            elif err['type'] == 'substitution':
                error_summary.append(
                    f"Incorrect word: Said '{err['recited_word']}' "
                    f"instead of '{err['reference_word']}'"
                )

        return (
            f"Content verification failed. {len(errors)} error(s) detected:\n" +
            "\n".join(error_summary)
        )

    def _load_reference_pitch(self, surah: int, ayah: int) -> np.ndarray:
        """Load reference pitch contour for this ayah (if available)"""
        # Implementation: Load from database or compute from reference audio
        pass

    def _score_tajweed(self, tajweed_results: dict) -> float:
        """Compute aggregate Tajweed score from rule validations"""
        # Implementation: Weight different rules, convert violations to score
        pass

    def _score_prosody(self, prosody_results: dict) -> float:
        """Compute aggregate prosody score"""
        pass

    def _score_pronunciation(self, alignment: dict) -> float:
        """Compute GOP-based pronunciation score"""
        pass

    def _score_voice(self, prosody_results: dict) -> float:
        """Compute voice quality score from OpenSMILE features"""
        pass

    def _extract_violations(self, tajweed_results: dict) -> List[dict]:
        """Extract user-facing violation details from raw results"""
        violations = []
        for rule_name, result in tajweed_results.items():
            for violation in result['violations']:
                violations.append({
                    "rule": rule_name,
                    "phoneme": violation['phoneme'],
                    "expected": violation['expected'],
                    "actual": violation['actual'],
                    "severity": violation['severity'],
                    "time_range": [violation['start'], violation['end']],
                    "feedback": violation['feedback']
                })
        return violations
```

### Output Schemas

**Content Error Response** (PATH A):
```python
{
    "overall_score": 0.0,
    "analysis_type": "content_error",
    "wer": float,                    # Word Error Rate (0.0 to 1.0)
    "transcript": str,               # What the ASR heard
    "word_errors": [
        {
            "type": "deletion" | "insertion" | "substitution",
            "reference_word": str | None,
            "recited_word": str | None,
            "position": int
        }
    ],
    "feedback": str,                 # User-friendly explanation
    "recommendation": str            # Actionable next step
}
```

**Full Analysis Response** (PATH B):
```python
{
    "overall_score": float,          # 0-100, weighted fusion of components
    "analysis_type": "pronunciation_and_prosody",
    "analysis_confidence": "high" | "medium",  # Based on WER threshold
    "wer": float,                    # For transparency
    "transcript": str,               # What the ASR heard
    "component_scores": {
        "tajweed": float,            # 0-100, weight=40%
        "prosody": float,            # 0-100, weight=30%
        "pronunciation": float,      # 0-100, weight=20%
        "voice_quality": float       # 0-100, weight=10%
    },
    "tajweed_violations": [
        {
            "rule": "madd" | "ghunnah" | "qalqalah",
            "phoneme": str,
            "expected": Any,         # e.g., duration=120ms for madd
            "actual": Any,           # e.g., duration=80ms (too short)
            "severity": "minor" | "moderate" | "major",
            "time_range": [float, float],  # Start and end time in seconds
            "feedback": str          # Pedagogical explanation
        }
    ],
    "prosody_feedback": {
        "rhythm": {...},
        "melody": {...},
        "style": {...}
    },
    "warnings": List[str],           # e.g., "borderline WER"
    "timestamp": str                 # ISO format
}
```

### Forced Alignment Strategy

**Phase-Dependent Implementation**:

**Phase 1 (Offline, Accuracy Priority)**:
- **Primary Aligner**: Montreal Forced Aligner (MFA)
- **Rationale**: MFA provides the highest phoneme boundary accuracy, critical for duration-based Tajweed rules (madd, ghunnah)
- **Performance**: 200-500ms latency acceptable for offline analysis
- **Configuration**:
  ```python
  aligner = MFA(
      acoustic_model="arabic_msa",
      dictionary="arabic_quranic_extended.dict",  # Include Tajweed phonemes
      beam_width=10,
      retry_beam=40
  )
  ```

**Phase 2 (Real-time, Latency Priority)**:
- **Strategy**: Benchmark fast neural aligners against MFA
- **Candidates**:
  - WhisperX (Whisper + Wav2Vec2 alignment)
  - Wav2Vec2-CTC forced alignment
  - Custom neural aligner trained on Quranic data
- **Decision Rule**:
  ```python
  if fast_aligner.confidence > 0.85:
      use fast_aligner  # ~50ms latency
  else:
      fallback to MFA   # ~300ms latency, higher accuracy
  ```

### Testing Requirements

**Integration Tests**:

```python
def test_comparison_engine_content_gate_blocks():
    """Test: Engine blocks analysis when WER > 8%"""
    wrong_verse_audio = load_test_audio("fatiha_verse2.wav")
    reference_text = get_reference_text(surah=1, ayah=1)  # Verse 1

    result = engine.compare(wrong_verse_audio, reference_text, surah=1, ayah=1)

    assert result['analysis_type'] == "content_error"
    assert result['wer'] > 0.08
    assert 'tajweed_violations' not in result
    assert 'component_scores' not in result

def test_comparison_engine_content_gate_proceeds():
    """Test: Engine proceeds with analysis when WER < 5%"""
    correct_audio = load_test_audio("fatiha_verse1_good.wav")
    reference_text = get_reference_text(surah=1, ayah=1)

    result = engine.compare(correct_audio, reference_text, surah=1, ayah=1)

    assert result['analysis_type'] == "pronunciation_and_prosody"
    assert result['analysis_confidence'] == "high"
    assert result['wer'] < 0.05
    assert 'component_scores' in result
    assert len(result['component_scores']) == 4

def test_comparison_engine_medium_confidence():
    """Test: Engine flags medium confidence when 5% < WER ≤ 8%"""
    borderline_audio = load_test_audio("fatiha_verse1_borderline.wav")
    reference_text = get_reference_text(surah=1, ayah=1)

    result = engine.compare(borderline_audio, reference_text, surah=1, ayah=1)

    assert result['analysis_confidence'] == "medium"
    assert 0.05 < result['wer'] <= 0.08
    assert len(result['warnings']) > 0
    assert "borderline" in result['warnings'][0].lower()

def test_score_fusion_weights():
    """Test: Overall score uses correct component weights"""
    # Mock component scores
    component_scores = {
        "tajweed": 90.0,
        "prosody": 80.0,
        "pronunciation": 85.0,
        "voice_quality": 75.0
    }

    expected_overall = (
        0.40 * 90.0 +  # Tajweed: 40%
        0.30 * 80.0 +  # Prosody: 30%
        0.20 * 85.0 +  # Pronunciation: 20%
        0.10 * 75.0    # Voice quality: 10%
    )

    # Mock the internal scoring methods
    with patch.object(engine, '_score_tajweed', return_value=90.0), \
         patch.object(engine, '_score_prosody', return_value=80.0), \
         patch.object(engine, '_score_pronunciation', return_value=85.0), \
         patch.object(engine, '_score_voice', return_value=75.0):

        result = engine.compare(audio, reference_text, surah=1, ayah=1)
        assert abs(result['overall_score'] - expected_overall) < 0.01
```

**Performance Benchmarks**:

```python
def test_latency_benchmarks():
    """Test: System meets latency targets"""
    audio_10s = generate_test_audio(duration=10.0)

    start = time.time()
    result = engine.compare(audio_10s, reference_text, surah=1, ayah=1)
    latency = time.time() - start

    # Phase 1 target: <5s for 10s audio
    assert latency < 5.0

    # If content gate blocks, should be <1s
    if result['analysis_type'] == "content_error":
        assert latency < 1.0
```


## M7.1: GATEKEEPER RATIONALE

**Why Two-Stage Architecture vs. End-to-End?**

End-to-end CTC/Attention models that perform transcription and alignment simultaneously are actively researched but not production-ready for high-stakes Quranic assessment. The two-stage architecture provides critical advantages:

### 1. Alignment Precision

**Evidence**: A June 2024 comparative study (Interspeech 2024) directly tested modern ASR methods (WhisperX, MMS) against traditional GMM-HMM forced aligners (Montreal Forced Aligner) on manually aligned datasets (TIMIT, Buckeye).

**Result**: MFA significantly outperformed both modern end-to-end systems for phoneme boundary detection at all tolerance levels.

**Impact for Iqrah**: Duration-based Tajweed rules (madd elongation, ghunnah nasalization) require phoneme boundaries accurate to within 20-50ms. End-to-end aligners currently cannot achieve this precision reliably.

### 2. Error Type Separation

**Problem with End-to-End**: A single model that performs both transcription and alignment conflates two fundamentally different error types:
- **Content Error**: User said the wrong word entirely
- **Pronunciation Error**: User said the right word but articulated a phoneme incorrectly

**Example Scenario**:
- Reference: "الرَّحْمَٰنِ" (Ar-Rahman)
- User recites: "الرَّحِيمِ" (Ar-Raheem) — wrong word entirely

**End-to-End Response**: Model might flag low confidence on phonemes, producing ambiguous feedback like "pronunciation quality: 45%". Was it mispronunciation or the wrong word?

**Two-Stage Response**:
1. ASR detects WER > 8% (wrong word substitution)
2. System stops, reports clearly: "Expected 'Ar-Rahman', heard 'Ar-Raheem'"
3. User receives actionable feedback: "Review verse text"

### 3. Catastrophic Failure Prevention

**Failure Mode**: User recites Surah Al-Ikhlas (Chapter 112) but system expects Surah Al-Fatiha (Chapter 1).

**Without Gatekeeper**:
- Forced aligner attempts to align Al-Ikhlas audio to Al-Fatiha phonemes
- Produces nonsensical, garbage timings
- Tajweed validators analyze these invalid alignments
- System returns detailed but completely invalid pronunciation feedback

**With Gatekeeper**:
- ASR produces high WER (>>10%)
- Gate blocks further processing
- System returns: "Incorrect verse detected. Please select the correct Surah and Ayah."

**Risk Assessment**: For a religious education application, providing incorrect Tajweed feedback is a severe failure mode that erodes user trust. The WER gate prevents this entirely.

### 4. Production Validation

**Industry Implementations**:
- **Microsoft Azure Pronunciation Assessment**: Uses two-stage pipeline (ASR transcription → phoneme-level scoring). Explicitly separates "completeness" score (content) from "accuracy" score (pronunciation).
- **Amazon Alexa English Learning**: RNN-Transducer predicts phonemes → Levenshtein alignment detects errors. Two-stage approach for reliability.
- **Tarteel AI**: World's leading Quranic recitation platform (millions of users) employs multi-stage architecture: ASR → error detection → feedback generation.

**Academic Consensus**: A 2025 ArXiv paper on Quranic pronunciation assessment (2509.00094) introduced a "Tasmeea algorithm" for transcript verification before pronunciation analysis—functionally identical to the ASR-Gatekeeper concept. They achieved 0.16% PER with Wav2Vec2-BERT.

### 5. Modularity and Future-Proofing

**Advantage**: Each module (ASR, Forced Aligner, Tajweed Validators) can be independently upgraded without redesigning the entire system.

**Example Upgrade Path**:
- **2025**: Use MFA (GMM-HMM) for highest accuracy
- **2026**: Replace with fast neural aligner when boundary accuracy improves
- **2027**: Integrate alignment-free GOP as auxiliary cross-check
- **2028**: Evaluate end-to-end models if they achieve production-grade reliability

The two-stage architecture allows incremental adoption of new research without breaking the system.

### 6. Research Trajectory

**Current State (2024-2025)**:
- Alignment-free GOP (Goodness of Pronunciation) methods using CTC are emerging in research papers (Interspeech 2024, NOCASA 2025 Challenge)
- These methods eliminate forced alignment but currently:
  - Do not provide explicit phoneme boundaries (needed for Tajweed visualization)
  - Have not been validated on Arabic/Quranic domain at scale
  - Work best as auxiliary features, not primary graders

**Recommendation**: Monitor alignment-free methods for Phase 3. Add as auxiliary cross-check (if GOP and FA disagree, flag as "uncertain"). Do not replace forced alignment in Phase 1-2.

### Trade-offs Acknowledged

| Aspect                            | Two-Stage                        | End-to-End                     |
| --------------------------------- | -------------------------------- | ------------------------------ |
| **Accuracy**                      | High (MFA alignment)             | Moderate (CTC implicit timing) |
| **Interpretability**              | Excellent (clear error taxonomy) | Limited (confidence scores)    |
| **Latency**                       | Higher (2 passes)                | Lower (1 pass)                 |
| **Computational Cost**            | Higher                           | Lower                          |
| **Catastrophic Error Prevention** | Excellent                        | Poor                           |
| **Modularity**                    | High                             | Low                            |

**Verdict**: For a high-stakes religious education application, accuracy and interpretability outweigh computational efficiency.

### Alternative Considered: Hybrid Architecture

**Concept**: Use lightweight end-to-end model for initial screening, fall back to two-stage for high-stakes assessment.

**Implementation**:
```python
if assessment_mode == "practice":
    # Use fast end-to-end model (lower accuracy acceptable)
    result = lightweight_e2e_model.assess(audio)
elif assessment_mode == "evaluation":
    # Use two-stage pipeline (highest accuracy)
    result = comparison_engine.compare(audio, reference_text)
```

**Decision**: Defer hybrid approach to Phase 3. Phase 1-2 focus on validating the high-accuracy two-stage pipeline. Add fast mode only after core quality is proven.


### M7.1: Tajweed Comparison

**Code**:
```python
def _compare_tajweed(self, student_phonemes, ref_phonemes, audio):
    """Run all validators"""
    results = {}
    violations = []

    # Madd
    madd_violations = self.validators["madd"].validate(student_phonemes)
    num_madd = sum(1 for p in student_phonemes if "madd" in p.get("tajweed_rule", ""))
    results["madd"] = 100 - (len(madd_violations) / num_madd * 100) if num_madd > 0 else 100
    violations.extend(madd_violations)

    # Ghunnah
    ghunnah_violations = self.validators["ghunnah"].validate(student_phonemes, audio)
    num_ghunnah = sum(1 for p in student_phonemes if p.get("tajweed_rule") == "ghunnah")
    results["ghunnah"] = 100 - (len(ghunnah_violations) / num_ghunnah * 100) if num_ghunnah > 0 else 100
    violations.extend(ghunnah_violations)

    # Qalqalah
    qalqalah_violations = self.validators["qalqalah"].validate(student_phonemes, audio)
    num_qalqalah = sum(1 for p in student_phonemes if p.get("tajweed_rule") == "qalqalah")
    results["qalqalah"] = 100 - (len(qalqalah_violations) / num_qalqalah * 100) if num_qalqalah > 0 else 100
    violations.extend(qalqalah_violations)

    # Weighted average
    TAJWEED_WEIGHTS = {"madd": 0.5, "ghunnah": 0.25, "qalqalah": 0.15, "complex": 0.1}
    results["overall"] = (
        TAJWEED_WEIGHTS["madd"] * results["madd"] +
        TAJWEED_WEIGHTS["ghunnah"] * results["ghunnah"] +
        TAJWEED_WEIGHTS["qalqalah"] * results["qalqalah"]
    ) / (TAJWEED_WEIGHTS["madd"] + TAJWEED_WEIGHTS["ghunnah"] + TAJWEED_WEIGHTS["qalqalah"])

    results["violations"] = violations
    return results
```

### M7.2: Prosody Comparison

**Code**:
```python
def _compare_prosody(self, student_prosody, ref_prosody):
    """Compare rhythm, melody, style"""
    # Rhythm: DTW + distributional metrics
    rhythm_score = self._score_rhythm(student_prosody, ref_prosody)

    # Melody: Fujisaki + tilt + maqam
    melody_score = self._score_melody(student_prosody, ref_prosody)

    # Style: X-vector cosine similarity
    style_score = self._score_style(student_prosody, ref_prosody)

    PROSODY_WEIGHTS = {"rhythm": 0.5, "melody": 0.3, "style": 0.2}
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
    """Combine Soft-DTW + distributional metrics"""
    # Soft-DTW divergence
    from tslearn.metrics import soft_dtw
    dtw_cost = soft_dtw(student["rhythm"]["features"], reference["rhythm"]["features"], gamma=1.0)
    dtw_score = 100 * np.exp(-dtw_cost / path_length)

    # nPVI comparison
    npvi_diff = abs(student["rhythm"]["nPVI"] - reference["rhythm"]["nPVI"])
    npvi_score = 100 * (1 - npvi_diff / 100)

    # IOI Wasserstein
    from scipy.stats import wasserstein_distance
    ioi_dist = wasserstein_distance(student["rhythm"]["ioi_distribution"], reference["rhythm"]["ioi_distribution"])
    ioi_score = 100 * np.exp(-ioi_dist * 10)

    # Weighted
    rhythm_score = 0.6 * dtw_score + 0.2 * npvi_score + 0.2 * ioi_score
    return rhythm_score

def _score_melody(self, student, reference):
    """Fujisaki + tilt + maqam"""
    # Fujisaki parameter similarity
    fujisaki_score = self._compare_fujisaki(student["melody"]["fujisaki_params"], reference["melody"]["fujisaki_params"])

    # Tilt distribution
    tilt_score = self._compare_tilt(student["melody"]["tilt_features"], reference["melody"]["tilt_features"])

    # Maqam match
    if student["melody"]["maqam"]["predicted"] == reference["melody"]["maqam"]["predicted"]:
        maqam_score = 100
    else:
        maqam_score = 50  # Partial credit

    melody_score = 0.4 * fujisaki_score + 0.4 * tilt_score + 0.2 * maqam_score
    return melody_score

def _score_style(self, student, reference):
    """X-vector cosine similarity"""
    from scipy.spatial.distance import cosine
    similarity = 1 - cosine(student["style"]["x_vector"], reference["style"]["x_vector"])
    return 100 * similarity
```

### M7.3: Pronunciation Comparison

**Code**:
```python
def _compare_pronunciation(self, student_phonemes, ref_phonemes):
    """GOP score comparison"""
    student_gops = [p["gop_score"] for p in student_phonemes]
    ref_gops = [p["gop_score"] for p in ref_phonemes]

    gop_delta = np.mean(np.abs(np.array(student_gops) - np.array(ref_gops)))
    gop_score = 100 * np.exp(-gop_delta)

    phoneme_acc = np.mean([p["confidence"] for p in student_phonemes])

    overall = 0.6 * gop_score + 0.4 * (phoneme_acc * 100)

    return {
        "gop_score": gop_score,
        "phoneme_accuracy": phoneme_acc * 100,
        "overall": overall
    }
```

### M7.4: Voice Quality Comparison

**Code**:
```python
def _compare_voice_quality(self, student_vq, reference_vq):
    """Timbre, vibrato, breathiness matching"""
    # Timbre
    timbre_diff = np.abs(student_vq["timbre"]["spectral_centroid_hz"] - reference_vq["timbre"]["spectral_centroid_hz"])
    timbre_score = 100 * np.exp(-timbre_diff / 1000)

    # Vibrato rate
    vibrato_diff = np.abs(student_vq["vibrato"]["rate_hz"] - reference_vq["vibrato"]["rate_hz"])
    vibrato_score = 100 * np.exp(-vibrato_diff / 2)

    # Breathiness
    breath_diff = np.abs(student_vq["breathiness"]["hnr_db"] - reference_vq["breathiness"]["hnr_db"])
    breath_score = 100 * np.exp(-breath_diff / 5)

    overall = 0.5 * timbre_score + 0.25 * vibrato_score + 0.25 * breath_score

    return {
        "timbre_similarity": timbre_score,
        "vibrato_match": vibrato_score,
        "breathiness_match": breath_score,
        "overall": overall
    }
```

**Latency**: <100ms (mostly NumPy)

---

## M8: FEEDBACK GENERATION

**Input**: Comparison results, user proficiency level
**Output**:
```python
{
    "summary": str,
    "detailed_feedback": [
        {
            "component": str,
            "score": float,
            "feedback": str,
            "examples": [{"timestamp": float, "text": str, "audio_snippet": str}]
        }
    ],
    "progress_tracking": {
        "improvement": float,
        "streak": int,
        "best_score": float
    },
    "next_steps": [str]
}
```

### M8.1: Summary Generation

**Code**:
```python
def generate_summary(overall_score, component_scores):
    if overall_score >= 90:
        level, emoji = "excellent", "🌟"
    elif overall_score >= 75:
        level, emoji = "very good", "👍"
    elif overall_score >= 60:
        level, emoji = "good", "✓"
    elif overall_score >= 40:
        level, emoji = "needs improvement", "⚠️"
    else:
        level, emoji = "requires attention", "❌"

    weakest = min(component_scores, key=lambda x: x["overall"])
    strongest = max(component_scores, key=lambda x: x["overall"])

    summary = f"""{emoji} Your recitation is {level} (score: {overall_score:.0f}/100).
Your strongest area is {strongest['name']} ({strongest['overall']:.0f}/100),
while {weakest['name']} needs work ({weakest['overall']:.0f}/100).
Focus on improving {weakest['name']} in your next practice session."""

    return summary
```

### M8.2: Detailed Feedback Templates

**Madd Duration**:
```python
if violation["rule"] == "madd_lazim":
    feedback = f"""
At {format_timestamp(violation['timestamp'])}, required madd (مد لازم) too short.
You held {violation['actual']}, should be {violation['expected']}.

**Fix**: Take deep breath, count slowly to 6 while extending vowel.
Practice until it feels uncomfortably long—that's correct duration.
"""
```

**Ghunnah**:
```python
if violation["rule"] == "ghunnah":
    feedback = f"""
At {format_timestamp(violation['timestamp'])}, ghunnah (غنّة) not strong enough.
Nasal resonance confidence is {violation['actual']:.0%}.

**Fix**: Close lips lightly, hum through nose while pronouncing noon.
Feel vibration in nasal cavity. Place finger on nose—should vibrate.
"""
```

**Rhythm**:
```python
if component == "rhythm":
    feedback = f"""
Rhythm score: {score:.0f}/100. Main issue: tempo consistency.
Your nPVI is {student_npvi:.1f} vs reference {ref_npvi:.1f}.

**Meaning**: Rushing some syllables, dragging others. Aim for steady timing.

**Fix**: Practice with metronome at {target_tempo:.0f} BPM.
Tap finger on each syllable for consistent rhythm.
"""
```

**Melody**:
```python
if component == "melody":
    if maqam_mismatch:
        feedback = f"""
Different maqam (musical mode) than reference.
You used {student_maqam}, reference uses {ref_maqam}.

**Why matters**: Each maqam has distinct emotional character.
Switching modes changes intended feeling.

**Fix**: Listen closely to reference's pitch contour, especially
at phrase beginnings/endings. Match rise and fall patterns.
"""
```

### M8.3: Progress Tracking

**Code**:
```python
class ProgressTracker:
    def __init__(self, user_id, db_connection):
        self.user_id = user_id
        self.db = db_connection

    def record_attempt(self, surah, ayah, score, timestamp):
        self.db.execute(
            "INSERT INTO attempts (user_id, surah, ayah, score, timestamp) VALUES (?, ?, ?, ?, ?)",
            (self.user_id, surah, ayah, score, timestamp)
        )

    def get_progress(self, surah, ayah):
        history = self.db.query(
            "SELECT score FROM attempts WHERE user_id=? AND surah=? AND ayah=? ORDER BY timestamp DESC LIMIT 5",
            (self.user_id, surah, ayah)
        )

        if len(history) < 2:
            return {"improvement": None, "streak": 0, "best_score": history[0] if history else 0}

        improvement = history[0] - history[1]

        streak = 0
        for i in range(len(history) - 1):
            if history[i] > history[i+1]:
                streak += 1
            else:
                break

        return {
            "improvement": improvement,
            "streak": streak,
            "best_score": max(history)
        }
```

### M8.4: Next Steps Recommendations

**Code**:
```python
def recommend_next_steps(component_scores, violations):
    recommendations = []

    # Critical violations priority
    critical = [v for v in violations if v["severity"] == "critical"]
    if critical:
        rules = set(v["rule"] for v in critical)
        recommendations.append(f"Focus on mastering {', '.join(rules)} first. These are fundamental.")

    # Weakest component
    weakest = min(component_scores, key=lambda x: x["overall"])
    if weakest["overall"] < 70:
        recommendations.append(f"Dedicate extra practice to {weakest['name']}. Consider specialized course/teacher.")

    # Progressive difficulty
    if component_scores["tajweed"]["madd"] > 90:
        recommendations.append("You've mastered madd! Ready for ghunnah and qalqalah next.")

    # Consistency encouragement
    if streak >= 3:
        recommendations.append(f"Great work! {streak} sessions improved in a row. Keep daily practice.")

    return recommendations
```

**Latency**: <50ms

---

## TECHNOLOGY STACK SUMMARY

### Core Libraries

| Purpose   | Library                          | Version                    | Notes              |
| --------- | -------------------------------- | -------------------------- | ------------------ |
| Audio I/O | soundfile, librosa               | >=0.12.1, >=0.10.0         | Fast load/resample |
| Pitch     | swift-f0, rmvpe                  | Latest                     | SwiftF0 primary    |
| Alignment | transformers, ctc-forced-aligner | >=4.35.0, >=0.1            | Wav2Vec2-BERT      |
| Prosody   | opensmile, praat-parselmouth     | >=3.0.1, >=0.4.3           | eGeMAPS, formants  |
| Style     | speechbrain                      | >=0.5.16                   | X-vectors          |
| Rhythm    | tslearn                          | >=0.6.0                    | Soft-DTW           |
| ML/DL     | torch, scipy, scikit-learn       | >=2.0.0, >=1.10.0, >=1.3.0 | Core ML            |
| Web       | fastapi, uvicorn                 | >=0.100.0, >=0.23.0        | REST API           |
| DB        | sqlalchemy, psycopg2             | >=2.0.0, >=2.9.0           | PostgreSQL         |

### Model Zoo

| Model                    | Size  | Purpose            | Source          |
| ------------------------ | ----- | ------------------ | --------------- |
| Wav2Vec2-BERT fine-tuned | 2.2GB | Phoneme alignment  | HuggingFace Hub |
| SwiftF0                  | 0.4MB | Pitch extraction   | PyPI            |
| RMVPE                    | 50MB  | Pitch fallback     | GitHub          |
| Ghunnah MLP              | 1MB   | Ghunnah detection  | Custom          |
| Qalqalah SVM             | 5MB   | Qalqalah detection | Custom          |
| Maqam CNN                | 10MB  | Maqam recognition  | Custom          |
| X-vector                 | 20MB  | Style embeddings   | SpeechBrain     |

### Infrastructure

| Component  | Tech                | Purpose                |
| ---------- | ------------------- | ---------------------- |
| Compute    | Lambda Labs, RunPod | GPU training/inference |
| Storage    | S3, MinIO           | Audio files            |
| Database   | PostgreSQL          | User data, progress    |
| Cache      | Redis               | Precomputed features   |
| Monitoring | Prometheus, Grafana | Performance metrics    |
| CI/CD      | GitHub Actions      | Automated testing      |

---

## VALIDATION STRATEGY

### Test Sets

**Phoneme Alignment**:
- 100 ayahs with manual phoneme boundaries
- Metrics: PER, boundary precision (20ms/50ms thresholds)
- Target: PER <1%, 90% within 50ms

**Tajweed Rules**:
- Madd: 500 examples (all types) → 99%
- Ghunnah: 300 examples → 85%
- Qalqalah: 200 examples → 80%

**Prosody**:
- 100 expert-rated pairs (10-point scale per dimension)
- Correlation: Automated vs human
- Target: r > 0.7 for rhythm, melody, style

**Expert Annotation**:
- Hire 3-5 qualified Qaris
- Inter-rater: Krippendorff's α > 0.75
- Budget: €2,000-3,000 for 200 hours

### Performance Benchmarks

**Latency Targets (Offline, CPU)**:
- Preprocessing: p95 <500ms
- Pitch: p95 <300ms
- Alignment: p95 <2s
- Tajweed: p95 <100ms
- Prosody: p95 <500ms
- Comparison: p95 <100ms
- **Total: p95 <4s per ayah**

**Latency Targets (Real-time, GPU)**:
- **Total: p95 <500ms per chunk**

**Latency Benchmark Code**:
```python
import time

def benchmark_pipeline(audio_path, num_runs=10):
    timings = {k: [] for k in ["load", "pitch", "alignment", "tajweed", "prosody", "comparison"]}

    for _ in range(num_runs):
        t0 = time.time()
        audio, sr = load_audio(audio_path)
        timings["load"].append(time.time() - t0)

        t0 = time.time()
        pitch = extract_pitch(audio, sr)
        timings["pitch"].append(time.time() - t0)

        t0 = time.time()
        phonemes = align_phonemes(audio, sr, surah=1, ayah=1)
        timings["alignment"].append(time.time() - t0)

        t0 = time.time()
        tajweed_results = validate_tajweed(phonemes, audio)
        timings["tajweed"].append(time.time() - t0)

        t0 = time.time()
        prosody = analyze_prosody(audio, phonemes)
        timings["prosody"].append(time.time() - t0)

        t0 = time.time()
        comparison = compare(student, reference)
        timings["comparison"].append(time.time() - t0)

    return {k: np.percentile(v, [50, 95, 99]) for k, v in timings.items()}
```

### User Testing

**Alpha (N=10)**: Internal, qualitative feedback
**Beta (N=50-100)**: Public, quantitative metrics, A/B testing
**Validation Study (N=60-100)**: Pre/post measurements, academic publication

---

## PROGRESSIVE ROLLOUT

### PHASE 1: OFFLINE E2E (Months 1-6) ← START

**Goal**: 90% accuracy basic Tajweed, comprehensive prosody

**Deliverables**:
1. Preprocessing pipeline (M1)
2. SwiftF0 + RMVPE (M2)
3. Wav2Vec2-BERT <1% PER (M3)
4. Madd 99%, Ghunnah 85%, Qalqalah 80% (M4)
5. Voice quality + prosody (M5-M6)
6. Comparison engine (M7)
7. Feedback generation (M8)
8. Validation: 100 expert cases r > 0.7

**Outcome**: Production-ready desktop system

**Cost**: €1,000-2,000 (GPU + expert validation)

### PHASE 2: REAL-TIME (Months 7-12)

**Goal**: <500ms latency, streaming

**Deliverables**:
1. WebSocket streaming architecture
2. INT8 quantization (4× speedup, <2% accuracy loss)
3. ONNX Runtime (2-3× speedup)
4. GPU acceleration (A100/T4)
5. Reference caching (6,236 ayahs precomputed)
6. Redis cluster (<100ms lookup)
7. Load balancing (10+ concurrent users)
8. Monitoring (Prometheus + Grafana)

**Optimization Techniques**:
- **Quantization**: INT8 weights (4-bit activation)
- **Caching**: SHA256-based result cache (70%+ hit rate)
- **Batching**: Process multiple requests together
- **Model pruning**: Remove 20-30% weights

**Outcome**: Real-time feedback during live recitation

**Cost**: €500-1,000/month (GPU) + €5,000 optimization

### PHASE 3: MOBILE (Months 13-18)

**Goal**: On-device inference <300ms

**Deliverables**:
1. Model distillation (student <100MB)
2. INT8/INT4 quantization (TFLite/CoreML)
3. iOS app (CoreML + Neural Engine)
4. Android app (TFLite + NNAPI)
5. Hybrid architecture (on-device basic, server advanced)
6. Offline mode (basic Tajweed without internet)
7. Mobile SDK (React Native/Flutter)
8. App store approval

**Model Sizes**:
- Distilled Wav2Vec2: 50-80MB
- Pitch extraction: 0.4MB (SwiftF0)
- Tajweed validators: 10-20MB
- Total app: <200MB

**Outcome**: iOS + Android in production

**Cost**: €1,000-2,000 (distillation) + 3-6 months dev

---

## RISK MITIGATION

### Technical Risks

| Risk                     | Likelihood | Impact | Mitigation                                            |
| ------------------------ | ---------- | ------ | ----------------------------------------------------- |
| Low PER                  | Medium     | High   | Fine-tune domain data, MMS fallback                   |
| Ghunnah/Qalqalah <target | High       | Medium | Start rule-based, improve iteratively                 |
| Real-time >500ms         | Medium     | High   | GPU optimization, caching, quantization               |
| Model drift              | Low        | Medium | Continuous monitoring, quarterly retrain              |
| Poor user audio          | High       | Medium | Preprocessing, quality warnings, graceful degradation |

### Operational Risks

| Risk                    | Likelihood | Impact | Mitigation                                            |
| ----------------------- | ---------- | ------ | ----------------------------------------------------- |
| GPU cost exceeds budget | Medium     | High   | Caching (70%+ hit rate), spot instances, ARM Graviton |
| Scaling >1000 users     | Low        | High   | Horizontal scaling, load balancing, auto-scaling      |
| Data privacy            | Low        | High   | GDPR compliance, encryption, user consent             |
| Competitor parity       | Medium     | Low    | Focus phoneme-level accuracy (hard to replicate)      |

### User Adoption Risks

| Risk                   | Likelihood | Impact | Mitigation                                       |
| ---------------------- | ---------- | ------ | ------------------------------------------------ |
| Feedback too technical | Medium     | Medium | User proficiency levels, simplified language     |
| Lack of engagement     | Medium     | High   | Gamification, progress tracking, social features |
| Accuracy skepticism    | Low        | High   | Transparency, expert validation, publish metrics |

---

## FUTURE ENHANCEMENTS (Post-Phase 3)

### Advanced Features (Years 2-3)

1. **Multi-Reciter Support**: Train on multiple Qaris (Husary, Minshawi, Sudais), style transfer
2. **Adaptive Learning**: Personalized practice plans, spaced repetition, AI-generated exercises
3. **Social Features**: Peer comparison, teacher dashboards, community challenges
4. **Advanced Prosody**: Emotion recognition, contextual analysis, waqf optimization
5. **B2B Features**: Bulk management, curriculum integration, analytics reports

### Research Directions

1. **Zero-Shot Tajweed**: LLMs for rule explanation, few-shot learning
2. **Multimodal**: Video analysis (mouth movements), AR overlays
3. **Synthetic Recitation**: TTS for practice audio, style control

---

## ARCHITECTURE STRENGTHS

✅ **Modularity**: Each component independent, testable, swappable
✅ **SOTA Integration**: Latest research (Wav2Vec2-BERT, SwiftF0, OpenSMILE)
✅ **Progressive Rollout**: Start simple, scale to real-time and mobile
✅ **AI-Agent Friendly**: Clear interfaces, minimal context bleed
✅ **Validation-First**: Accuracy targets, expert validation, user studies
✅ **Production-Ready**: Edge cases, graceful degradation, monitoring

**Commitment**: This design is stable for 3 years. Focus on execution, not redesign.
