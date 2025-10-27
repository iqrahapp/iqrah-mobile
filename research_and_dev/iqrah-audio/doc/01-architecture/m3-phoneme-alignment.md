# Module M3: Phoneme Recognition & Alignment

[← Back to Overview](overview.md) | [↑ Navigation](../NAVIGATION.md)

---

## M3: PHONEME RECOGNITION & ALIGNMENT

**Input**: Preprocessed audio (16kHz mono), Quranic reference text (with diacritics)
**Output**:
```python
{
    "phonemes": [
        {
            "phoneme": str,          # IPA or Buckwalter notation
            "start": float,          # Seconds
            "end": float,            # Seconds
            "confidence": float,     # Muaalem posterior probability
            "sifa": Sifa            # Tajweed properties from Muaalem
        }
    ],
    "words": [
        {
            "word": str,             # Arabic grapheme form
            "start": float,
            "end": float,
            "phonemes": list[int]    # Indices into phonemes array
        }
    ],
    "gate_result": {
        "passed": bool,
        "per": float,                # Phoneme Error Rate
        "confidence": float
    },
    "alignment_method": "ctc_phoneme_forced"
}
```

---

## 1. OVERVIEW

This module implements **phonetic-first Quranic recitation analysis** using the pre-trained `obadx/muaalem-model-v3_2` model, which outputs both phonemes and comprehensive Tajweed sifat (pronunciation attributes).

### Key Components

1. **Text Preprocessing**: Convert Quranic text to phonetic reference using `quran_phonetizer`
2. **ASR Inference**: Use Muaalem to predict phonemes + sifat from audio
3. **Content Verification**: Phonetic gatekeeper (PER-based) to verify correct Ayah
4. **Forced Alignment**: CTC-based phoneme-level alignment + word-level aggregation
5. **Sifat Extraction**: Baseline Tajweed attributes from Muaalem for M4

### Design Principles

- **No training required**: Use pre-trained Muaalem model as-is for MVP
- **Phonetic-first**: Analyze pronunciation at the phoneme level, not grapheme
- **Dual alignment**: Phoneme-level (primary) + word-level (visualization)
- **Baseline Tajweed**: Extract 10+ Tajweed rules from Muaalem sifat for free

---

## 2. ARCHITECTURE

### 2.1 Text Preprocessing (Phonetizer)

**Module**: [`src/iqrah/text/phonetizer.py`](../../src/iqrah/text/phonetizer.py)

**Purpose**: Convert Quranic text with diacritics to phonetic reference script required by Muaalem

**Interface**:
```python
from iqrah.text.phonetizer import phonetize_ayah

# Input: Quranic text with diacritics
text = "بِسْمِ اللَّهِ الرَّحْمَٰنِ الرَّحِيمِ"

# Output: QuranPhoneticScriptOutput
phonetic_ref = phonetize_ayah(text, remove_space=True)

# Structure:
# - phonetic_ref.text: str (phonetic script string)
# - phonetic_ref.units: list[PhoneticUnit] (phoneme sequence)
# - phonetic_ref.metadata: dict (word boundaries, positions)
```

**Output Schema**:
```python
@dataclass
class PhoneticUnit:
    phoneme: str          # IPA or Buckwalter notation
    position: int         # Character position in original text
    word_index: int       # Word index (for word-level aggregation)
    expected_sifa: dict   # Expected Tajweed properties (optional)

@dataclass
class QuranPhoneticScriptOutput:
    text: str                      # Full phonetic string (space-removed)
    units: list[PhoneticUnit]      # Phoneme sequence with metadata
    metadata: dict                 # word_boundaries, total_phonemes, etc.
```

**Implementation Notes**:
- Source: Extracted from `obadx/quran-muaalem` repository
- Integrates: Arabic phonetic rules, Quranic orthography conventions
- Compatible: Output format matches Muaalem's expected input

**Example**:
```python
phonetic_ref = phonetize_ayah("بِسْمِ اللَّهِ")
# phonetic_ref.text = "bismillaah"  (simplified example)
# phonetic_ref.units = [
#     PhoneticUnit(phoneme='b', position=0, word_index=0),
#     PhoneticUnit(phoneme='i', position=1, word_index=0),
#     ...
# ]
```

---

### 2.2 Muaalem Model Wrapper

**Module**: [`src/iqrah/asr/muaalem_wrapper.py`](../../src/iqrah/asr/muaalem_wrapper.py)

**Purpose**: Interface to `obadx/muaalem-model-v3_2` for phoneme recognition and sifat extraction

**Model Details**:
- **Architecture**: Wav2Vec2-BERT with CTC head + Tajweed classification heads
- **Pre-trained on**: Quranic recitation corpus
- **Outputs**: Phonemes (CTC logits) + Sifat (per-phoneme Tajweed properties)
- **Inference**: FP16/BFloat16 on CUDA, CPU fallback supported

**Interface**:
```python
from iqrah.asr.muaalem_wrapper import MuaalemASR

model = MuaalemASR(device="cuda", dtype=torch.bfloat16)

result = model.infer(
    audio=audio_array,           # (N,) float32 @ 16kHz
    phonetic_ref=phonetic_ref,    # QuranPhoneticScriptOutput
    return_ctc_logits=True        # For forced alignment
)

# Output: MuaalemInferenceOutput
# - phonemes: Unit (text, probs, ids)
# - sifat: list[Sifa] (tajweed properties per phoneme)
# - ctc_logits: torch.Tensor (T, V) - optional, for alignment
```

**Sifat Structure** (from Muaalem):
```python
@dataclass
class Sifa:
    """Tajweed properties predicted by Muaalem for a phoneme group"""
    phonemes_group: str                       # Phonemes this sifa applies to
    hams_or_jahr: Optional[SingleUnit]        # Whispered vs voiced
    shidda_or_rakhawa: Optional[SingleUnit]   # Tense vs lax
    tafkheem_or_tarqeeq: Optional[SingleUnit] # Emphatic vs plain
    itbaq: Optional[SingleUnit]               # Pharyngealized
    safeer: Optional[SingleUnit]              # Whistling
    qalqla: Optional[SingleUnit]              # Echo/bounce
    tikraar: Optional[SingleUnit]             # Trill
    tafashie: Optional[SingleUnit]            # Spreading
    istitala: Optional[SingleUnit]            # Elevation
    ghonna: Optional[SingleUnit]              # Nasalization

@dataclass
class SingleUnit:
    text: str      # e.g., "moqalqal", "maghnoon"
    prob: float    # Confidence score (0-1)
    idx: int       # Class ID
```

**Chunking Strategy** (for long audio):
```python
# Muaalem processes audio in chunks for memory efficiency
if duration > 20.0:  # seconds
    chunks = split_audio(audio, chunk_size=20.0, stride=0.4)
    results = [model.infer(chunk, phonetic_ref) for chunk in chunks]
    result = merge_chunk_results(results)
```

---

### 2.3 Forced Alignment

**Module**: [`src/iqrah/align/phoneme_aligner.py`](../../src/iqrah/align/phoneme_aligner.py)

**Purpose**: Extract precise timestamps for phonemes and words using CTC logits from Muaalem

#### 2.3.1 Phoneme-Level Alignment (Primary)

**Approach**: CTC Forced Aligner with Viterbi decoding

**Algorithm**:
1. Input: CTC logits (T, V) + phonetic reference sequence
2. Viterbi: Find best alignment path through CTC lattice
3. Handle blank tokens: CTC uses blank for repeated phonemes
4. Extract timestamps: Map aligned path to time via frame rate
5. Confidence: Mean CTC posterior probability along path

**Interface**:
```python
from iqrah.align.phoneme_aligner import PhonemeForcedAligner

aligner = PhonemeForcedAligner()

aligned = aligner.align(
    ctc_logits=muaalem_result.ctc_logits,  # (T, V) tensor
    phonetic_ref=phonetic_ref,              # Expected phoneme sequence
    audio_duration=len(audio) / 16000       # For timestamp calculation
)

# Output: AlignmentOutput
# - phonemes: list[AlignedPhoneme]
# - words: list[AlignedWord]
```

**Output Schema (Phoneme-Level)**:
```python
@dataclass
class AlignedPhoneme:
    phoneme: str           # Phoneme symbol
    start: float           # Start time (seconds)
    end: float             # End time (seconds)
    confidence: float      # Mean CTC posterior
    sifa: Sifa             # Tajweed properties from Muaalem
    word_index: int        # Which word this phoneme belongs to
```

**Quality Validation**:
```python
# Sanity checks (must pass for valid alignment)
assert mean_confidence >= 0.5, "Low confidence alignment"
assert all(20e-3 <= (p.end - p.start) <= 500e-3 for p in phonemes), "Invalid durations"
assert len(phonemes) within ±20% of len(phonetic_ref.units), "Length mismatch"
```

---

#### 2.3.2 Word-Level Alignment (Visualization)

**Approach**: Aggregate phoneme timestamps by word boundaries

**Algorithm**:
1. Group phonemes by `word_index` from phonetic_ref metadata
2. Word start = min(phoneme_starts)
3. Word end = max(phoneme_ends)
4. Word confidence = mean(phoneme_confidences)

**Interface**:
```python
# Word alignment is computed automatically during phoneme alignment
aligned = aligner.align(...)

for word in aligned.words:
    print(f"{word.word}: {word.start:.2f}s - {word.end:.2f}s")
```

**Output Schema (Word-Level)**:
```python
@dataclass
class AlignedWord:
    word: str                    # Arabic grapheme form
    start: float                 # Earliest phoneme start
    end: float                   # Latest phoneme end
    phonemes: list[AlignedPhoneme]  # Constituent phonemes
    confidence: float            # Mean of phoneme confidences
```

**Use Cases**:
- Visualization: Highlight words as user recites
- Navigation: Jump to specific word in playback
- Debugging: Compare word-level vs phoneme-level timing

---

### 2.4 ASR Gatekeeper (Content Verification)

**Module**: [`src/iqrah/compare/gate.py`](../../src/iqrah/compare/gate.py)

**Purpose**: Verify the user recited the correct Ayah before analyzing pronunciation quality

**Approach**: Phoneme Error Rate (PER) Comparison

**CRITICAL**: Muaalem outputs **phonemes only** (not graphemes), so we compare phonetic sequences directly:
- **Predicted phonemes**: From Muaalem's inference output
- **Expected phonemes**: From phonetic reference (phonetizer)
- **Metric**: Phoneme Error Rate (PER) via Levenshtein distance
- **Threshold**: PER < 0.05 (5%) for high confidence

**Why Phonetic Gate is Better**:
1. **More accurate**: Checks actual pronunciation, not just spelling
2. **No additional model**: No grapheme decoder needed
3. **Faster**: Direct string comparison, no CTC decoding
4. **Consistent**: Matches the phonetic-first architecture

**Interface**:
```python
from iqrah.compare.gate import PhoneticGatekeeper

gate = PhoneticGatekeeper(per_threshold=0.05)

gate_result = gate.check(
    predicted_phonemes=muaalem_result.phonemes.text,  # String of phonemes
    expected_phonemes=phonetic_ref.text               # From phonetizer
)

# Output: GateResult
# - passed: bool
# - per: float (Phoneme Error Rate)
# - errors: list[(expected, predicted, position)]
# - confidence: float (mean of phoneme probs from Muaalem)
```

**Output Schema**:
```python
@dataclass
class GateResult:
    passed: bool               # True if PER < threshold
    per: float                 # Phoneme Error Rate (0-1)
    errors: list[tuple]        # [(expected, predicted, position), ...]
    confidence: float          # Mean phoneme confidence from Muaalem
    threshold_used: float      # PER threshold applied
```

**Implementation** (Levenshtein Distance):
```python
def compute_per(predicted: str, expected: str) -> float:
    """
    Compute Phoneme Error Rate using Levenshtein distance.

    PER = (substitutions + deletions + insertions) / len(expected)
    """
    distance = levenshtein_distance(predicted, expected)
    return distance / len(expected) if len(expected) > 0 else 0.0
```

**Threshold Strategy**:
```python
PER_THRESHOLD_HIGH = 0.05    # ≤5%: High confidence, proceed with full analysis
PER_THRESHOLD_MEDIUM = 0.08  # ≤8%: Medium confidence, proceed with warning
PER_THRESHOLD_FAIL = 0.08    # >8%: Fail, report content mismatch

if gate_result.per <= PER_THRESHOLD_HIGH:
    confidence = "high"
elif gate_result.per <= PER_THRESHOLD_MEDIUM:
    confidence = "medium"  # Proceed with caution flag
else:
    confidence = "fail"    # Stop analysis, report errors
```

**Error Details**:
```python
# Extract specific phoneme errors for user feedback
for expected, predicted, pos in gate_result.errors:
    print(f"Position {pos}: Expected '{expected}', got '{predicted}'")
```

---

## 3. DATA FLOW

```
┌─────────────────────────────────────────────────────────────────┐
│ INPUT: Quranic Text (grapheme) + User Audio                    │
└─────────────────────────────────────────────────────────────────┘
                            ↓
        ┌───────────────────┴────────────────────┐
        │                                        │
        ↓                                        ↓
┌──────────────────┐                  ┌─────────────────┐
│  Text Phonetizer │                  │  Preprocessed   │
│  (M3.1)          │                  │  Audio (M1)     │
└──────────────────┘                  └─────────────────┘
        │                                        │
        ↓                                        │
phonetic_ref (expected phonemes)                 │
        │                                        │
        └────────────────┬───────────────────────┘
                         ↓
                ┌─────────────────┐
                │  Muaalem Model  │
                │  (M3.2)         │
                └─────────────────┘
                         │
        ┌────────────────┴────────────────┐
        ↓                                 ↓
predicted_phonemes + sifat        ctc_logits (T, V)
        │                                 │
        ↓                                 │
┌─────────────────────┐                  │
│ Phonetic Gatekeeper │                  │
│ (M3.4)              │                  │
└─────────────────────┘                  │
        │                                 │
        ↓                                 │
    passed/failed (PER < 5%)              │
        │                                 │
        └─────────┬───────────────────────┘
                  ↓ (if passed)
        ┌──────────────────────┐
        │  Phoneme Aligner     │
        │  (M3.3)              │
        └──────────────────────┘
                  │
        ┌─────────┴────────┐
        ↓                  ↓
aligned_phonemes      aligned_words
(with timestamps)     (aggregated)
        │
        └───────────→ [M4: Tajweed Validator]
```

---

## 4. TRAINING (NOT REQUIRED FOR MVP)

**MVP Strategy**: Use `muaalem-model-v3_2` as-is

- **Pre-trained**: Already trained on Quranic recitation corpus
- **Phoneme-level**: Outputs phonemes + sifat directly
- **No fine-tuning**: Sufficient accuracy for Phase 1 MVP
- **Cost savings**: Eliminates 8-10 weeks of training + €1,500 GPU costs

**Future Fine-Tuning** (Phase 2+):
- Collect edge cases from production usage (mispronunciations, accents)
- Fine-tune on 500-1000 labeled samples
- Target: PER <1% (from current ~2%)
- Adapt to diverse learner accents
- Improve sifat accuracy: 70-85% → 90%+

---

## 5. PERFORMANCE TARGETS

### Phase 1 (MVP - Pre-trained Muaalem)

- **PER (Phoneme Error Rate)**: <2% on expert reciters, <5% on learners
- **Alignment precision**: ±50ms for phonemes, ±100ms for words
- **Latency**: <3s for 10s audio (GPU), <10s (CPU)
- **Gatekeeper accuracy**: >98% (false accept <2%, false reject <2%)
- **Sifat baseline accuracy**: 70-85% per rule (comprehensive coverage)

### Phase 2 (Fine-tuned)

- **PER**: <1% on expert reciters, <3% on learners
- **Alignment precision**: ±30ms for phonemes
- **Latency**: <1s for 10s audio (optimized)
- **Sifat accuracy**: 85-95% per rule (with domain adaptation)

---

## 6. EVALUATION

### Metrics

1. **Phoneme Accuracy**: Compare predicted phonemes vs ground truth transcriptions
   - PER: Phoneme Error Rate (Levenshtein distance)
   - Phoneme-level precision, recall, F1

2. **Alignment Quality**: Compare predicted timestamps vs manual annotations
   - Mean Absolute Error (MAE): |predicted_time - ground_truth_time|
   - % within ±50ms: Fraction of phonemes aligned within 50ms tolerance
   - DTW distance: Compare alignment path to reference

3. **Sifat Accuracy**: Compare predicted Tajweed properties vs expert labels
   - Per-rule accuracy: Ghunnah, Qalqalah, Tafkhim, etc.
   - Overall sifat correlation: Spearman's ρ vs expert annotations

4. **Gate Quality**: Measure false accepts and false rejects
   - False Accept Rate: Wrong Ayah passes gate
   - False Reject Rate: Correct Ayah fails gate

### Test Sets

- **100 diverse Ayahs**: Different reciters, speeds, Surahs
- **50 edge cases**: Mispronunciations, heavy accents, hesitations
- **20 adversarial**: Wrong Ayahs for gatekeeper testing
- **Manual annotations**: 100 Ayahs with expert phoneme + timestamp labels

### Validation Procedure

```python
# Evaluate on test set
results = []
for audio, reference_text, ground_truth in test_set:
    # Run full M3 pipeline
    phonetic_ref = phonetize_ayah(reference_text)
    muaalem_result = model.infer(audio, phonetic_ref)
    gate_result = gate.check(muaalem_result.phonemes.text, phonetic_ref.text)
    aligned = aligner.align(muaalem_result.ctc_logits, phonetic_ref, len(audio) / 16000)

    # Compute metrics
    per = compute_per(muaalem_result.phonemes.text, ground_truth.phonemes)
    alignment_mae = compute_alignment_mae(aligned.phonemes, ground_truth.timestamps)
    sifat_accuracy = compute_sifat_accuracy(aligned.phonemes, ground_truth.sifat)

    results.append({
        "per": per,
        "alignment_mae": alignment_mae,
        "sifat_accuracy": sifat_accuracy,
        "gate_passed": gate_result.passed
    })

# Aggregate metrics
print(f"Mean PER: {np.mean([r['per'] for r in results]):.2%}")
print(f"Mean Alignment MAE: {np.mean([r['alignment_mae'] for r in results]):.1f}ms")
print(f"Mean Sifat Accuracy: {np.mean([r['sifat_accuracy'] for r in results]):.1%}")
```

---

## 7. IMPLEMENTATION TASKS

### T3.1: Text Phonetizer [AI Agent - HIGH PRIORITY]

**Description**: Extract and adapt quran_phonetizer from obadx/quran-muaalem repository

**Checklist**:
- [ ] Research obadx/quran-muaalem repo structure
- [ ] Extract phonetizer code (likely in `quran_phonetizer/` or similar)
- [ ] Create [`src/iqrah/text/phonetizer.py`](../../src/iqrah/text/phonetizer.py)
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

### T3.2: Muaalem Wrapper [AI Agent - HIGH PRIORITY]

**Description**: Create Python interface to obadx/muaalem-model-v3_2

**Checklist**:
- [ ] Load model from HuggingFace: `transformers.Wav2Vec2ForCTC.from_pretrained("obadx/muaalem-model-v3_2")`
- [ ] Create [`src/iqrah/asr/muaalem_wrapper.py`](../../src/iqrah/asr/muaalem_wrapper.py)
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

### T3.3: Phoneme Forced Aligner [AI Agent + HUMAN]

**Description**: Implement CTC Viterbi forced aligner for phoneme-level timestamps

**Checklist**:
- [ ] Create [`src/iqrah/align/phoneme_aligner.py`](../../src/iqrah/align/phoneme_aligner.py)
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

### T3.4: Word-Level Aggregation [AI Agent]

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

### T3.5: Phonetic Gatekeeper [AI Agent]

**Description**: Implement phoneme-level content verification using PER

**Checklist**:
- [ ] Create [`src/iqrah/compare/gate.py`](../../src/iqrah/compare/gate.py)
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

### T3.6: Integration Tests [AI Agent]

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

### T3.7: Performance Profiling [HUMAN]

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

**Total M3**: ~28 hours (AI: 23h, HUMAN: 5h)

---

## 8. RISKS & MITIGATIONS

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|-----------|
| Muaalem PER >5% on learners | Medium | High | Fine-tune in Phase 2; provide confidence flags |
| CTC alignment errors (±100ms) | Medium | Medium | Fallback to proportional slicing; validate against ground truth |
| Phonetizer bugs (wrong phoneme mapping) | Medium | High | Extensive unit tests; manual validation with expert |
| GPU memory overflow (>8GB) | Low | Medium | Use BFloat16; batch size = 1; chunking for long audio |
| Sifat schema changes in Muaalem updates | Low | Medium | Pin Muaalem version; maintain local copy |
| Phonetic gate false rejects | Low | High | Tune threshold; allow manual override in UI |

---

## 9. DEPENDENCIES

### External Libraries

```python
# Required for M3
transformers>=4.35.0    # Muaalem model loading
torch>=2.0.0            # Inference and CTC operations
torchaudio>=2.0.0       # Audio processing
soundfile>=0.12.1       # Audio I/O
numpy>=1.24.0           # Array operations
python-Levenshtein>=0.21.0  # Fast edit distance for PER
```

### Internal Modules

- [`iqrah.text.phonetizer`](../../src/iqrah/text/phonetizer.py) (NEW) - Text to phonetic conversion
- [`iqrah.text.arabic_norm`](../../src/iqrah/text/arabic_norm.py) (existing) - Text normalization
- [`iqrah.asr.muaalem_wrapper`](../../src/iqrah/asr/muaalem_wrapper.py) (NEW) - Model interface
- [`iqrah.align.phoneme_aligner`](../../src/iqrah/align/phoneme_aligner.py) (NEW) - CTC forced alignment
- [`iqrah.compare.gate`](../../src/iqrah/compare/gate.py) (NEW) - Content verification

### Module Dependencies (from other parts of system)

- **M1 (Preprocessing)** → Audio normalization (16kHz, mono)
- **M4 (Tajweed)** ← Aligned phonemes + sifat
- **M7 (Comparison)** ← Gate result + alignment quality

---

## 10. REFERENCES

### Models & Datasets

- **obadx/muaalem-model-v3_2**: https://huggingface.co/obadx/muaalem-model-v3_2
- **obadx/quran-muaalem**: https://github.com/obadx/quran-muaalem (phonetizer source)

### Academic Papers

- CTC forced alignment: Graves et al. (2006) - "Connectionist Temporal Classification"
- Goodness of Pronunciation: Witt & Young (2000) - "Phone-level pronunciation scoring"
- Arabic phonetics: IPA for Arabic consonants and vowels

### Related Documentation

- [M1: Preprocessing](m1-preprocessing.md) - Audio normalization
- [M4: Tajweed Validation](m4-tajweed.md) - Sifat interpretation
- [M7: Comparison Engine](m7-comparison-engine/comparison-methods.md) - Score aggregation

---

**Next**: [Module M4: Tajweed Validation](m4-tajweed.md) | [← Back to Overview](overview.md)
