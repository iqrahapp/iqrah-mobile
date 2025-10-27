[↑ Navigation](../NAVIGATION.md)

# Phase 1: Offline E2E Tasks (Weeks 1-18)

**Purpose**: Concrete, AI-agent-assignable tasks for the offline end-to-end pipeline
**Duration**: 4 months (18 weeks) - ACCELERATED from 6 months
**Status**: Ready to Start

---

## OVERVIEW

Phase 1 delivers a **fully functional offline Tajweed analysis system** that compares user recitations against Quranic reference audio using comprehensive phoneme + Tajweed validation.

### Key Architectural Shift (2025-10-27)

**Discovery**: The `obadx/muaalem-model-v3_2` model outputs **phonemes + comprehensive Tajweed sifat**, eliminating the need to train custom models from scratch.

**Impact**:
- **Timeline**: 6 months → 4 months (33% faster)
- **Effort**: ~400 hours → ~300 hours (25% reduction)
- **Training eliminated**: No Wav2Vec2 training required for MVP
- **Tajweed coverage**: 10+ rules from Day 1 (via Muaalem baseline)

### Two-Tier Tajweed Architecture

- **Tier 1**: Baseline sifat from Muaalem (70-85% accuracy, comprehensive)
- **Tier 2**: Specialized modules (Madd probabilistic 95%+, Ghunnah formants 90%+)

---

## TASK ASSIGNMENT CONVENTIONS

### Roles
- **AI Agent**: Claude Code, Sonnet 4.5, or similar
- **HUMAN**: Software engineer with domain expertise
- **PRIORITY**: HIGH = critical path, MEDIUM = parallelizable, LOW = optional

### Task Format
```markdown
#### T3.1: Task Name [AI Agent - HIGH PRIORITY]
**Description**: One-sentence summary

**Checklist**:
- [ ] Concrete action item 1
- [ ] Concrete action item 2

**Test**:
\```python
# Test code
\```

**Dependencies**: TaskID or None
**Estimate**: Xh (AI: Xh, HUMAN: Yh if mixed)
**Assigned**: AI Agent / HUMAN
```

---

## 1. M1: PREPROCESSING (Week 1)

### T1.1: Audio Loader [AI Agent - HIGH PRIORITY]
**Description**: Load and validate multi-format audio files

**Checklist**:
- [ ] Support MP3, WAV, WebM, M4A, FLAC
- [ ] Validate file format headers
- [ ] Extract metadata (duration, sample rate, channels)
- [ ] Convert stereo to mono (average channels)
- [ ] Handle corrupted files gracefully

**Test**:
```python
def test_audio_loader():
    audio_data = load_audio("test.wav")
    assert audio_data["audio"].shape[0] > 0
    assert audio_data["sample_rate"] == 16000
    assert audio_data["audio"].ndim == 1  # Mono
```

**Dependencies**: None
**Estimate**: 4 hours
**Assigned**: AI Agent

---

### T1.2: Resampling & Normalization [AI Agent - HIGH PRIORITY]
**Description**: Resample to 16kHz and normalize amplitude

**Checklist**:
- [ ] Resample to 16kHz using librosa
- [ ] Normalize amplitude to [-1, 1] range
- [ ] Apply RMS normalization
- [ ] Test on various sample rates

**Test**:
```python
def test_normalization():
    audio, sr = librosa.load("test_44100.wav", sr=16000)
    normalized = normalize_audio(audio)
    assert abs(normalized).max() <= 1.0
    assert -1.0 <= normalized.min()
```

**Dependencies**: T1.1
**Estimate**: 3 hours
**Assigned**: AI Agent

---

**Total M1**: ~10 hours (Week 1)

---

## 2. M2: PITCH EXTRACTION (Week 1-2)

### T2.1: SwiftF0 Integration [AI Agent - HIGH PRIORITY]
**Description**: Integrate SwiftF0 pitch extractor

**Checklist**:
- [ ] Install SwiftF0 model
- [ ] Create wrapper interface
- [ ] Extract F0 contour (frame-level)
- [ ] Handle unvoiced regions (F0 = 0)
- [ ] Test on Quranic audio samples

**Test**:
```python
def test_swiftf0():
    audio = load_audio("test.wav")
    f0, voiced = extract_pitch_swiftf0(audio["audio"], audio["sample_rate"])
    assert len(f0) > 0
    assert len(voiced) == len(f0)
```

**Dependencies**: T1.2
**Estimate**: 6 hours
**Assigned**: AI Agent

---

**Total M2**: ~10 hours (Week 1-2)

---

## 3. M3: PHONEME RECOGNITION & ALIGNMENT (Week 2-4)

**Critical Change**: M3 now uses **pre-trained Muaalem model** (no training required)

### Sprint 1: Foundation (Week 2)

#### T3.1: Text Phonetizer [AI Agent - HIGH PRIORITY]
**Description**: Extract and adapt quran_phonetizer from obadx/quran-muaalem repository

**Checklist**:
- [ ] Research obadx/quran-muaalem repo structure
- [ ] Extract phonetizer code (likely in `quran_phonetizer/` or similar)
- [ ] Create `src/iqrah/text/phonetizer.py`
- [ ] Implement `phonetize_ayah(text) -> QuranPhoneticScriptOutput`
- [ ] Test on sample Ayahs: "بِسْمِ اللَّهِ الرَّحْمَٰنِ الرَّحِيمِ"
- [ ] Verify output format compatible with Muaalem input
- [ ] Add comprehensive docstrings and type hints

**Test**:
```python
def test_phonetizer():
    phonetic_ref = phonetize_ayah("بِسْمِ اللَّهِ")
    assert phonetic_ref.text is not None
    assert len(phonetic_ref.units) > 0
    assert all(hasattr(u, 'phoneme') for u in phonetic_ref.units)
```

**Dependencies**: None
**Estimate**: 4 hours (includes research + extraction)
**Assigned**: AI Agent (Claude Code / Sonnet 4.5)

---

#### T3.2: Muaalem Wrapper [AI Agent - HIGH PRIORITY]
**Description**: Create Python interface to obadx/muaalem-model-v3_2

**Checklist**:
- [ ] Load model from HuggingFace: `transformers.Wav2Vec2ForCTC.from_pretrained("obadx/muaalem-model-v3_2")`
- [ ] Create `src/iqrah/asr/muaalem_wrapper.py`
- [ ] Implement `MuaalemASR` class with `infer()` method
- [ ] Handle chunking for audio >20 seconds
- [ ] Expose CTC logits (set `return_ctc_logits=True`)
- [ ] Parse sifat output (phonemes + Tajweed properties)
- [ ] Test inference on sample audio
- [ ] Verify sifat structure matches Muaalem's schema

**Test**:
```python
def test_muaalem_inference():
    model = MuaalemASR(device="cpu")
    audio = np.random.randn(16000 * 5).astype(np.float32)  # 5 seconds
    phonetic_ref = phonetize_ayah("بِسْمِ اللَّهِ")

    result = model.infer(audio, phonetic_ref, return_ctc_logits=True)

    assert result.phonemes is not None
    assert result.sifat is not None
    assert len(result.sifat) <= len(result.phonemes.ids)
    assert result.ctc_logits.shape[1] == model.vocab_size
```

**Dependencies**: T3.1 (phonetizer)
**Estimate**: 6 hours
**Assigned**: AI Agent

---

### Sprint 2: Alignment (Week 2-3)

#### T3.3: Phoneme Forced Aligner [AI Agent + HUMAN]
**Description**: Implement CTC Viterbi forced aligner for phoneme-level timestamps

**Checklist**:
- [ ] Create `src/iqrah/align/phoneme_aligner.py`
- [ ] Implement Viterbi decoding with blank transitions
- [ ] Handle CTC blank tokens correctly (for repeated phonemes)
- [ ] Extract timestamps: frame_index × frame_duration
- [ ] Compute confidence: mean CTC posterior along path
- [ ] Handle edge cases: empty paths → fallback to proportional slicing
- [ ] **HUMAN**: Validate algorithm correctness (check against known alignments)
- [ ] Test on 10 sample audios with manual annotations

**Test**:
```python
def test_phoneme_alignment():
    aligner = PhonemeForcedAligner()
    ctc_logits = torch.randn(100, 50)  # (T=100 frames, V=50 vocab)
    phonetic_ref = phonetize_ayah("بِسْمِ")

    aligned = aligner.align(ctc_logits, phonetic_ref, audio_duration=5.0)

    assert len(aligned.phonemes) > 0
    assert all(p.start < p.end for p in aligned.phonemes)
    assert all(0.02 <= (p.end - p.start) <= 0.5 for p in aligned.phonemes)

    # HUMAN: Manually verify timestamps match audio playback
```

**Dependencies**: T3.1, T3.2
**Estimate**: 8 hours (AI: 6h skeleton + tests, HUMAN: 2h algorithm validation)
**Assigned**: AI Agent + HUMAN

---

#### T3.4: Word-Level Aggregation [AI Agent]
**Description**: Aggregate phoneme timestamps into word-level segments

**Checklist**:
- [ ] Parse word boundaries from phonetic_ref.metadata
- [ ] Group phonemes by `word_index`
- [ ] Compute word start: min(phoneme_starts)
- [ ] Compute word end: max(phoneme_ends)
- [ ] Compute word confidence: mean(phoneme_confidences)
- [ ] Attach phoneme list to each word
- [ ] Test output structure with assertions

**Test**:
```python
def test_word_aggregation():
    aligned = aligner.align(...)  # From T3.3

    assert len(aligned.words) > 0
    assert all(hasattr(w, 'phonemes') for w in aligned.words)
    assert all(w.start <= w.end for w in aligned.words)
    assert all(len(w.phonemes) > 0 for w in aligned.words)
```

**Dependencies**: T3.3
**Estimate**: 3 hours
**Assigned**: AI Agent

---

### Sprint 3: Gatekeeper (Week 3-4)

#### T3.5: Phonetic Gatekeeper [AI Agent]
**Description**: Implement phoneme-level content verification using PER (Phoneme Error Rate)

**Checklist**:
- [ ] Create `src/iqrah/compare/gate.py`
- [ ] Implement Levenshtein distance for phoneme strings
- [ ] Implement `compute_per(predicted, expected) -> float`
- [ ] Implement `PhoneticGatekeeper` class with `check()` method
- [ ] Apply threshold: PER < 0.05 for high confidence
- [ ] Extract error details: (expected, predicted, position) tuples
- [ ] Return `GateResult` with passed/failed + metrics
- [ ] Test on correct and incorrect recitations

**Test**:
```python
def test_phonetic_gate():
    gate = PhoneticGatekeeper(per_threshold=0.05)

    # Test correct Ayah (should pass)
    predicted = "bismillaahirrahmaanirraheem"
    expected = "bismillaahirrahmaanirraheem"
    result = gate.check(predicted, expected)
    assert result.passed == True
    assert result.per == 0.0

    # Test wrong Ayah (should fail)
    predicted = "alhamdulilaahi"  # Wrong Ayah
    expected = "bismillaahirrahmaanirraheem"
    result = gate.check(predicted, expected)
    assert result.passed == False
    assert result.per > 0.50
```

**Dependencies**: T3.2 (needs Muaalem phoneme output format)
**Estimate**: 3 hours
**Assigned**: AI Agent

---

### Sprint 4: Integration & Testing (Week 4)

#### T3.6: E2E M3 Pipeline Test [AI Agent]
**Description**: Test complete M3 pipeline end-to-end

**Checklist**:
- [ ] Load sample Quranic audio + reference text
- [ ] Run full pipeline: Phonetize → Muaalem → Gate → Align
- [ ] Test on 20 diverse samples (different reciters, speeds)
- [ ] Measure metrics: PER, alignment MAE, gate accuracy
- [ ] Log results to JSON file for analysis
- [ ] Verify output formats match M3 specification
- [ ] Check for edge cases: very short Ayahs, long Ayahs, silences

**Test**:
```python
def test_m3_end_to_end():
    # Load test data
    audio, sr = librosa.load("test_data/fatiha_verse1.wav", sr=16000)
    reference_text = "بِسْمِ اللَّهِ الرَّحْمَٰنِ الرَّحِيمِ"

    # Run M3 pipeline
    phonetic_ref = phonetize_ayah(reference_text)
    muaalem_result = model.infer(audio, phonetic_ref, return_ctc_logits=True)
    gate_result = gate.check(muaalem_result.phonemes.text, phonetic_ref.text)

    assert gate_result.passed == True

    aligned = aligner.align(muaalem_result.ctc_logits, phonetic_ref, len(audio) / sr)

    assert len(aligned.phonemes) > 0
    assert len(aligned.words) > 0
    assert all(hasattr(p, 'sifa') for p in aligned.phonemes)
```

**Dependencies**: All T3.x tasks
**Estimate**: 4 hours
**Assigned**: AI Agent

---

#### T3.7: Performance Profiling [HUMAN]
**Description**: Measure latency and memory usage of M3 components

**Checklist**:
- [ ] Profile each component: phonetizer, muaalem inference, alignment, gate
- [ ] Measure: P50, P95, P99 latencies
- [ ] Check GPU memory usage (VRAM)
- [ ] Identify bottlenecks (likely: Muaalem inference, alignment)
- [ ] **HUMAN**: Analyze results, document findings in `doc/performance/m3-profile.md`
- [ ] Recommend optimizations if needed

**Tools**:
```python
import time
import cProfile

# Example profiling
start = time.perf_counter()
result = model.infer(audio, phonetic_ref)
latency = time.perf_counter() - start
print(f"Muaalem inference: {latency:.2f}s")

# Memory profiling
import torch
print(f"GPU memory: {torch.cuda.max_memory_allocated() / 1e9:.2f} GB")
```

**Dependencies**: T3.6 (needs working pipeline)
**Estimate**: 4 hours
**Assigned**: HUMAN

---

**Total M3**: ~28 hours (AI: 23h, HUMAN: 5h) - Week 2-4

**Key Savings**: Eliminated ~50 hours of Wav2Vec2 training tasks

---

## 4. M4: TAJWEED VALIDATION (Week 5-7)

**Critical Change**: M4 now uses **two-tier architecture** (Baseline sifat + Specialized modules)

### Sprint 1: Baseline Interpreter (Week 5)

#### T4.1: Sifat Parser & Baseline Interpreter [AI Agent - HIGH PRIORITY]
**Description**: Parse Muaalem sifat output and compare against expected rules

**Checklist**:
- [ ] Create `src/iqrah/tajweed/baseline_interpreter.py`
- [ ] Parse sifat structure from Muaalem (Sifa dataclass)
- [ ] Load expected rules from phonetic reference metadata
- [ ] Compare predicted vs expected for each sifat property
- [ ] Generate violations for mismatches (prob < threshold)
- [ ] Apply confidence threshold (default 0.7)
- [ ] Support all sifat properties: ghonna, qalqla, tafkheem, itbaq, safeer, etc.

**Test**:
```python
def test_baseline_interpreter():
    interpreter = BaselineTajweedInterpreter()
    violations = interpreter.validate(aligned_phonemes, phonetic_ref)

    assert isinstance(violations, dict)
    assert "ghunnah" in violations
    assert all(isinstance(v, Violation) for v_list in violations.values() for v in v_list)
```

**Dependencies**: M3 complete (needs aligned phonemes with sifat)
**Estimate**: 8 hours
**Assigned**: AI Agent

---

### Sprint 2: Madd Probabilistic Validator (Week 5-6)

#### T4.2: Madd Validator - Duration Estimation [AI Agent + HUMAN]
**Description**: Implement local and global harakat distribution estimation

**Checklist**:
- [ ] Create `src/iqrah/tajweed/madd_validator.py`
- [ ] Implement `estimate_local_distribution(aligned_phonemes, window_seconds)`
- [ ] Extract short vowel durations from aligned phonemes
- [ ] Compute mean and std (Gaussian parameters)
- [ ] Implement `load_global_distribution(user_id, surah_id, db)`
- [ ] **HUMAN**: Validate distribution estimation logic with sample data

**Test**:
```python
def test_duration_estimation():
    validator = MaddValidator()
    validator.update_distributions(aligned_phonemes, global_stats=None)

    assert validator.local_mean > 0
    assert validator.local_std > 0
    assert 50 < validator.local_mean < 200  # Reasonable harakat duration
```

**Dependencies**: T4.1, M3 complete
**Estimate**: 10 hours (AI: 7h, HUMAN: 3h)
**Assigned**: AI Agent + HUMAN

---

#### T4.3: Madd Validator - Rule Validation Logic [AI Agent]
**Description**: Validate Madd duration using Gaussian model

**Checklist**:
- [ ] Identify Madd phonemes from phonetic reference
- [ ] Look up expected duration (harakats × local_mean)
- [ ] Compute tolerance (2 × local_std)
- [ ] Generate violations for out-of-tolerance durations
- [ ] Compute z-score and confidence (scipy.stats.norm)
- [ ] Generate user-facing feedback messages

**Test**:
```python
def test_madd_validation():
    validator = MaddValidator()
    validator.update_distributions(aligned_phonemes, None)

    violations = validator.validate(aligned_phonemes, phonetic_ref)

    assert all(hasattr(v, 'z_score') for v in violations)
    assert all(hasattr(v, 'expected_duration') for v in violations)
```

**Dependencies**: T4.2
**Estimate**: 6 hours
**Assigned**: AI Agent

---

#### T4.4: Database Schema for Global Stats [AI Agent]
**Description**: Store per-user, per-Surah harakat distributions

**Checklist**:
- [ ] Design table `user_madd_distributions` (SQL schema below)
- [ ] Implement ORM model (SQLAlchemy or similar)
- [ ] Create migration script (Alembic)
- [ ] Implement insert/update logic (upsert on conflict)
- [ ] Test query performance (index on user_id + surah_id)

**Schema**:
```sql
CREATE TABLE user_madd_distributions (
    id SERIAL PRIMARY KEY,
    user_id UUID NOT NULL,
    surah_id INT NOT NULL,
    mean_harakat_ms FLOAT NOT NULL,
    std_harakat_ms FLOAT NOT NULL,
    n_samples INT NOT NULL,
    updated_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(user_id, surah_id)
);

CREATE INDEX idx_user_madd ON user_madd_distributions(user_id, surah_id);
```

**Test**:
```python
def test_madd_distribution_storage():
    # Insert
    save_global_distribution(user_id, surah_id, mean=100, std=20, n=10, db)

    # Query
    result = load_global_distribution(user_id, surah_id, db)
    assert result == (100.0, 20.0, 10)

    # Update
    save_global_distribution(user_id, surah_id, mean=105, std=18, n=15, db)
    result = load_global_distribution(user_id, surah_id, db)
    assert result == (105.0, 18.0, 15)
```

**Dependencies**: T4.3
**Estimate**: 4 hours
**Assigned**: AI Agent

---

### Sprint 3: Orchestrator (Week 6)

#### T4.5: Tajweed Orchestrator [AI Agent]
**Description**: Coordinate Tier 1 + Tier 2 modules and merge violations

**Checklist**:
- [ ] Create `src/iqrah/tajweed/orchestrator.py`
- [ ] Integrate baseline interpreter (Tier 1)
- [ ] Integrate Madd validator (Tier 2)
- [ ] Merge violations (avoid duplicates, resolve conflicts)
- [ ] Compute per-rule scores: 100 × (1 - violations / total)
- [ ] Compute overall score (weighted average)
- [ ] Make modules configurable (enable/disable flags)

**Test**:
```python
def test_orchestrator():
    orch = TajweedOrchestrator(
        enable_baseline=True,
        enable_madd=True,
        enable_ghunnah=False
    )

    result = orch.validate(aligned_phonemes, phonetic_ref, audio, user_stats)

    assert "violations" in result
    assert "scores_by_rule" in result
    assert "overall_score" in result
    assert 0 <= result["overall_score"] <= 100
```

**Dependencies**: T4.1, T4.3
**Estimate**: 6 hours
**Assigned**: AI Agent

---

### Sprint 4: Ghunnah Enhanced (Week 7) - OPTIONAL FOR MVP

#### T4.6: Ghunnah Formant Analyzer [AI Agent - OPTIONAL]
**Description**: Extract formants for low-confidence Ghunnah enhancement

**Checklist**:
- [ ] Create `src/iqrah/tajweed/ghunnah_validator.py`
- [ ] Install Parselmouth: `pip install praat-parselmouth`
- [ ] Implement `extract_ghunnah_formants(audio, start, end, sr)`
- [ ] Extract F1, F2, F3 using Praat's Burg algorithm
- [ ] Extract nasal energy (250-350Hz band filter)
- [ ] Implement formant scoring logic
- [ ] Combine with Tier 1 baseline (weighted average)
- [ ] Test on nasal phonemes (ن، م)

**Test**:
```python
def test_ghunnah_formants():
    validator = GhunnahValidator(use_formants=True, formant_weight=0.3)
    violations = validator.validate(aligned_phonemes, audio, sr=16000)

    assert all(hasattr(v, 'combined_confidence') for v in violations)
    assert all(0 <= v.combined_confidence <= 1 for v in violations)
```

**Dependencies**: T4.1
**Estimate**: 10 hours
**Assigned**: AI Agent

---

### Sprint 5: Integration & Validation (Week 7)

#### T4.7: E2E M4 Pipeline Test [AI Agent]
**Description**: Test complete Tajweed validation end-to-end

**Checklist**:
- [ ] Load 50 diverse samples (different reciters, Surahs)
- [ ] Run full M4 pipeline: Baseline + Madd (+ Ghunnah if enabled)
- [ ] Measure per-rule accuracy (Ghunnah, Qalqalah, Madd, etc.)
- [ ] Compare Tier 1 only vs Tier 1+2
- [ ] Log false positives and false negatives
- [ ] Generate confusion matrices per rule

**Test**:
```python
def test_m4_end_to_end():
    result = m4_pipeline(aligned_phonemes, phonetic_ref, audio, user_stats)

    assert "violations" in result
    assert len(result["violations"]) >= 0
    assert "scores_by_rule" in result
    assert result["overall_score"] >= 0
```

**Dependencies**: T4.5
**Estimate**: 8 hours
**Assigned**: AI Agent

---

#### T4.8: Expert Validation [HUMAN - HIGH PRIORITY]
**Description**: Validate system output against expert annotations

**Checklist**:
- [ ] **HUMAN**: Manually annotate 100 samples (Ghunnah, Qalqalah, Madd)
- [ ] Run system on same 100 samples
- [ ] Compare system output vs expert labels
- [ ] Compute precision, recall, F1 per rule
- [ ] Compute Spearman's ρ (overall score vs expert score)
- [ ] **HUMAN**: Analyze errors, document failure modes
- [ ] **HUMAN**: Create error taxonomy (common mispredictions)

**Deliverables**:
- Annotated dataset: `data/expert_annotations.csv`
- Evaluation report: `doc/validation/m4-expert-validation.md`
- Error analysis: `doc/validation/m4-error-taxonomy.md`

**Dependencies**: T4.7
**Estimate**: 20 hours (HUMAN: 16h annotations, 4h analysis)
**Assigned**: HUMAN + Expert Annotators

---

**Total M4**: ~72 hours (AI: 48h, HUMAN: 24h) - Week 5-7

**Key Savings**: Eliminated ~20 hours of custom classifier training tasks

---

## 5. M5: VOICE QUALITY ANALYSIS (Week 8-9)

### T5.1: OpenSMILE Integration [AI Agent]
**Description**: Integrate OpenSMILE for voice quality features

**Checklist**:
- [ ] Install opensmile Python wrapper
- [ ] Extract ComParE feature set (6373 features)
- [ ] Select relevant features: jitter, shimmer, HNR
- [ ] Create wrapper interface
- [ ] Test on diverse audio samples

**Test**:
```python
def test_voice_quality():
    features = extract_voice_quality(audio)
    assert "jitter" in features
    assert "shimmer" in features
    assert "hnr" in features
    assert 0 <= features["hnr"] <= 40  # Reasonable HNR range
```

**Dependencies**: T1.2
**Estimate**: 6 hours
**Assigned**: AI Agent

---

**Total M5**: ~12 hours (Week 8-9)

---

## 6. M6: PROSODY ANALYSIS (Week 9-10)

### T6.1: Rhythm Validator [AI Agent]
**Description**: Implement rhythm deviation detection using DTW

**Checklist**:
- [ ] Implement DTW alignment between user and reference F0
- [ ] Compute rhythm deviation score
- [ ] Detect rushed/dragged segments
- [ ] Generate feedback with timestamps

**Test**:
```python
def test_rhythm_validator():
    result = validate_rhythm(user_f0, ref_f0, user_times, ref_times)
    assert "deviation_score" in result
    assert "violations" in result
```

**Dependencies**: T2.1 (pitch extraction)
**Estimate**: 10 hours
**Assigned**: AI Agent

---

**Total M6**: ~16 hours (Week 9-10)

---

## 7. M7: COMPARISON ENGINE (Week 11-12)

### T7.1: Score Aggregation [AI Agent]
**Description**: Aggregate component scores into overall score

**Checklist**:
- [ ] Collect scores from M3, M4, M5, M6
- [ ] Apply weights (configurable)
- [ ] Compute overall score (0-100)
- [ ] Generate detailed breakdown
- [ ] Test edge cases (missing components)

**Test**:
```python
def test_score_aggregation():
    scores = {
        "pronunciation": 85,
        "tajweed": 78,
        "voice_quality": 92,
        "prosody": 88
    }
    overall = aggregate_scores(scores)
    assert 0 <= overall <= 100
```

**Dependencies**: M3, M4, M5, M6 complete
**Estimate**: 8 hours
**Assigned**: AI Agent

---

**Total M7**: ~16 hours (Week 11-12)

---

## 8. M8: FEEDBACK GENERATION (Week 13-14)

### T8.1: Feedback Templates [AI Agent]
**Description**: Create structured feedback templates per violation type

**Checklist**:
- [ ] Design template format (JSON/YAML)
- [ ] Create templates for each Tajweed rule
- [ ] Include severity, timestamp, suggestion
- [ ] Support i18n (Arabic, English)
- [ ] Test template rendering

**Test**:
```python
def test_feedback_generation():
    violation = {
        "rule": "Madd",
        "expected": 6,
        "actual": 4,
        "timestamp": 2.5
    }
    feedback = generate_feedback(violation)
    assert "Madd" in feedback
    assert "2.5" in feedback
```

**Dependencies**: M4, M6 complete
**Estimate**: 10 hours
**Assigned**: AI Agent

---

**Total M8**: ~16 hours (Week 13-14)

---

## 9. INTEGRATION & TESTING (Week 15-16)

### T9.1: E2E Pipeline Integration [AI Agent + HUMAN]
**Description**: Integrate all modules into single pipeline

**Checklist**:
- [ ] Create master pipeline controller
- [ ] Wire M1 → M2 → M3 → M4 → M5 → M6 → M7 → M8
- [ ] Handle errors gracefully
- [ ] Log intermediate results
- [ ] **HUMAN**: Test on 100 diverse samples
- [ ] Fix integration bugs

**Test**:
```python
def test_full_pipeline():
    result = run_full_pipeline(audio_path, reference_text)
    assert "overall_score" in result
    assert "violations" in result
    assert "feedback" in result
```

**Dependencies**: All modules complete
**Estimate**: 20 hours (AI: 12h, HUMAN: 8h)
**Assigned**: AI Agent + HUMAN

---

**Total Integration**: ~20 hours (Week 15-16)

---

## 10. VALIDATION & POLISH (Week 17-18)

### T10.1: Expert Validation Study [HUMAN]
**Description**: Validate system against expert ratings

**Checklist**:
- [ ] **HUMAN**: Recruit 3 Tajweed experts
- [ ] Prepare 100 test samples
- [ ] Collect expert ratings (independent)
- [ ] Run system on same samples
- [ ] Compute correlation (Spearman's ρ)
- [ ] Target: ρ > 0.75
- [ ] Document findings

**Dependencies**: T9.1
**Estimate**: 40 hours (HUMAN: expert time + analysis)
**Assigned**: HUMAN

---

### T10.2: Demo Polish [AI Agent + HUMAN]
**Description**: Create polished demo interface

**Checklist**:
- [ ] Build web UI (Gradio or Streamlit)
- [ ] Upload audio or record
- [ ] Display results (scores + violations)
- [ ] Visualize pitch + alignment
- [ ] **HUMAN**: UX review and refinement

**Dependencies**: T9.1
**Estimate**: 16 hours (AI: 10h, HUMAN: 6h)
**Assigned**: AI Agent + HUMAN

---

**Total Validation & Polish**: ~56 hours (Week 17-18)

---

## TIMELINE SUMMARY

| Week | Module | Tasks | Hours | Cumulative |
|------|--------|-------|-------|------------|
| 1 | M1, M2 | Audio preprocessing, pitch | 20 | 20 |
| 2-4 | M3 | Phoneme alignment (Muaalem) | 28 | 48 |
| 5-7 | M4 | Tajweed (two-tier) | 72 | 120 |
| 8-9 | M5 | Voice quality | 12 | 132 |
| 9-10 | M6 | Prosody | 16 | 148 |
| 11-12 | M7 | Comparison engine | 16 | 164 |
| 13-14 | M8 | Feedback generation | 16 | 180 |
| 15-16 | Integration | E2E pipeline | 20 | 200 |
| 17-18 | Validation | Expert study + demo | 56 | 256 |

**Total**: ~256 hours (AI: ~180h, HUMAN: ~76h)

**Key Improvement**: Reduced from ~400 hours (original estimate) by eliminating training tasks

---

## CRITICAL PATH

The following tasks are on the critical path and cannot be parallelized:

1. **M1.1-1.2**: Audio preprocessing → 7 hours (Week 1)
2. **M3.1-3.6**: Muaalem integration + alignment → 24 hours (Week 2-4)
3. **M4.1-4.5**: Tajweed baseline + Madd → 40 hours (Week 5-6)
4. **M7.1**: Score aggregation → 8 hours (Week 11)
5. **M9.1**: E2E integration → 20 hours (Week 15-16)
6. **M10.1**: Expert validation → 40 hours (Week 17-18)

**Total Critical Path**: ~139 hours across 18 weeks

**Buffer**: ~117 hours for bug fixes, edge cases, and refinement

---

## PARALLEL WORKSTREAMS

Tasks that can run in parallel (assign to multiple AI agents or humans):

| Stream | Tasks | Duration | Can Start |
|--------|-------|----------|-----------|
| **Stream 1** | M1, M2, M3 | Week 1-4 | Immediately |
| **Stream 2** | M5 (Voice quality) | Week 8-9 | After M1 |
| **Stream 3** | M6 (Prosody) | Week 9-10 | After M2 |
| **Stream 4** | M8 (Feedback) | Week 13-14 | After M4 |
| **Stream 5** | T4.6 (Ghunnah formants) | Week 7 | After M4.1 |

**Speedup Potential**: With 3-4 agents working in parallel, Phase 1 can complete in 12-14 weeks instead of 18 weeks.

---

## MILESTONES

### Milestone 1: Core Pipeline (Week 4)
- M1-M3 functional
- Phoneme alignment working
- Gatekeeper validates content

**Success Criteria**:
- PER < 5% on test set
- Gate accuracy > 95%

---

### Milestone 2: Tajweed MVP (Week 7)
- M4 Tier 1 + Madd Tier 2 operational
- 10+ Tajweed rules covered (baseline)
- Madd 95%+ accuracy

**Success Criteria**:
- Baseline Tajweed 75%+ accuracy
- Madd 95%+ accuracy
- No crashes on 100 test samples

---

### Milestone 3: Full Pipeline (Week 14)
- All modules integrated (M1-M8)
- E2E pipeline working
- Feedback generation complete

**Success Criteria**:
- Full pipeline latency < 30s per Ayah
- All component scores computed
- Feedback messages generated

---

### Milestone 4: Validated MVP (Week 18)
- Expert validation complete
- Spearman's ρ > 0.75
- Demo polished and ready

**Success Criteria**:
- Expert correlation > 0.75
- Demo deployed and accessible
- Documentation complete

---

## RISK MITIGATION

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Muaalem model accuracy < expected | High | Fine-tune in Phase 2; provide confidence flags |
| Alignment quality poor | High | Fallback to proportional slicing; manual validation |
| Expert validation low correlation | Medium | Iterate on scoring weights; add more rules |
| Integration bugs | Medium | Extensive unit + integration tests; CI/CD pipeline |
| Timeline slippage | Low | Buffer time built in; can skip optional tasks (T4.6) |

---

## RESOURCE ALLOCATION

### Solo Developer (18 Weeks)

**Week 1-4**: M1-M3 (Preprocessing + Alignment)
**Week 5-7**: M4 (Tajweed two-tier)
**Week 8-10**: M5-M6 (Voice quality + Prosody)
**Week 11-14**: M7-M8 (Comparison + Feedback)
**Week 15-16**: Integration
**Week 17-18**: Validation + Demo

---

### With AI Agents (12-14 Weeks)

Assign multiple agents in parallel:

- **Agent 1**: M1, M2, M3 (Week 1-4)
- **Agent 2**: M5 (Week 8-9), M8 (Week 13-14)
- **Agent 3**: M6 (Week 9-10), T4.6 (Week 7)
- **HUMAN**: M4 validation, integration, expert study

**Speedup**: 33% faster with parallelization

---

## NAVIGATION

**Related Documents**:
- [M3: Phoneme Alignment](../01-architecture/m3-phoneme-alignment.md)
- [M4: Tajweed Validation](../01-architecture/m4-tajweed.md)
- [Implementation Guide](../02-implementation/guide.md)

---

**Status**: Ready to Execute | **Last Updated**: 2025-10-27
