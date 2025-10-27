# M7: Gatekeeper Rationale

[← Back to M7 Overview](overview.md) | [↑ Navigation](../../NAVIGATION.md)

**Purpose**: Why we chose two-stage architecture (ASR + Forced Alignment) over end-to-end models

---

## Why Two-Stage Architecture vs. End-to-End?

End-to-end CTC/Attention models that perform transcription and alignment simultaneously are actively researched but not production-ready for high-stakes Quranic assessment. The two-stage architecture provides critical advantages:

## 1. Alignment Precision

**Evidence**: A June 2024 comparative study (Interspeech 2024) directly tested modern ASR methods (WhisperX, MMS) against traditional GMM-HMM forced aligners (Montreal Forced Aligner) on manually aligned datasets (TIMIT, Buckeye).

**Result**: MFA significantly outperformed both modern end-to-end systems for phoneme boundary detection at all tolerance levels.

**Impact for Iqrah**: Duration-based Tajweed rules (madd elongation, ghunnah nasalization) require phoneme boundaries accurate to within 20-50ms. End-to-end aligners currently cannot achieve this precision reliably.

## 2. Error Type Separation

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

## 3. Catastrophic Failure Prevention

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

## 4. Production Validation

**Industry Implementations**:
- **Microsoft Azure Pronunciation Assessment**: Uses two-stage pipeline (ASR transcription → phoneme-level scoring). Explicitly separates "completeness" score (content) from "accuracy" score (pronunciation).
- **Amazon Alexa English Learning**: RNN-Transducer predicts phonemes → Levenshtein alignment detects errors. Two-stage approach for reliability.
- **Tarteel AI**: World's leading Quranic recitation platform (millions of users) employs multi-stage architecture: ASR → error detection → feedback generation.

**Academic Consensus**: A 2025 ArXiv paper on Quranic pronunciation assessment (2509.00094) introduced a "Tasmeea algorithm" for transcript verification before pronunciation analysis—functionally identical to the ASR-Gatekeeper concept. They achieved 0.16% PER with Wav2Vec2-BERT.

## 5. Modularity and Future-Proofing

**Advantage**: Each module (ASR, Forced Aligner, Tajweed Validators) can be independently upgraded without redesigning the entire system.

**Example Upgrade Path**:
- **2025**: Use MFA (GMM-HMM) for highest accuracy
- **2026**: Replace with fast neural aligner when boundary accuracy improves
- **2027**: Integrate alignment-free GOP as auxiliary cross-check
- **2028**: Evaluate end-to-end models if they achieve production-grade reliability

The two-stage architecture allows incremental adoption of new research without breaking the system.

## 6. Research Trajectory

**Current State (2024-2025)**:
- Alignment-free GOP (Goodness of Pronunciation) methods using CTC are emerging in research papers (Interspeech 2024, NOCASA 2025 Challenge)
- These methods eliminate forced alignment but currently:
  - Do not provide explicit phoneme boundaries (needed for Tajweed visualization)
  - Have not been validated on Arabic/Quranic domain at scale
  - Work best as auxiliary features, not primary graders

**Recommendation**: Monitor alignment-free methods for Phase 3. Add as auxiliary cross-check (if GOP and FA disagree, flag as "uncertain"). Do not replace forced alignment in Phase 1-2.

## Trade-offs Acknowledged

| Aspect                            | Two-Stage                        | End-to-End                     |
| --------------------------------- | -------------------------------- | ------------------------------ |
| **Accuracy**                      | High (MFA alignment)             | Moderate (CTC implicit timing) |
| **Interpretability**              | Excellent (clear error taxonomy) | Limited (confidence scores)    |
| **Latency**                       | Higher (2 passes)                | Lower (1 pass)                 |
| **Computational Cost**            | Higher                           | Lower                          |
| **Catastrophic Error Prevention** | Excellent                        | Poor                           |
| **Modularity**                    | High                             | Low                            |

**Verdict**: For a high-stakes religious education application, accuracy and interpretability outweigh computational efficiency.

---

## **MVP GATEKEEPER: HYBRID WER/CER**

**Status**: Current implementation for the MVP release.

### The WER Instability Problem

Traditional ASR gatekeepers use **Word Error Rate (WER)** as the sole metric for content verification. However, WER has a critical flaw when applied to very short texts:

**Example**:
- Reference: "قل هو" (2 words, "Say: He is")
- User recites: "قل الله" (2 words, "Say: Allah")
- WER = 1 error / 2 words = **50%**

A 50% error rate suggests severe content mismatch, but the user correctly recited the first word and only substituted one word. This is statistically misleading for short Ayahs.

**Root Cause**: WER's denominator is word count. For texts with ≤3 words, a single error yields disproportionately high WER (33-100%), making threshold-based gating unreliable.

### The MVP Solution: Hybrid WER/CER

The MVP implements a **text-length-aware hybrid approach**:

```python
def select_error_metric(reference_text_normalized: List[str]) -> str:
    """
    Select appropriate error metric based on reference text length.

    Returns:
        "cer" if reference has ≤ 3 words, else "wer"
    """
    word_count = len(reference_text_normalized)
    return "cer" if word_count <= 3 else "wer"
```

**Logic**:
- **Reference ≤ 3 words** → Use **Character Error Rate (CER)**
  - CER = (character insertions + deletions + substitutions) / total reference characters
  - More stable for short texts, as character count is typically >>3 even for 1-2 words
- **Reference > 3 words** → Use **Word Error Rate (WER)**
  - WER = (word insertions + deletions + substitutions) / total reference words
  - Standard metric for longer texts, well-validated in ASR literature

### Thresholds (Unified)

Both WER and CER use **identical thresholds**:
- **Error Rate ≤ 0.05 (5%)** → **High Confidence** → Proceed with full analysis
- **Error Rate ≤ 0.08 (8%)** → **Medium Confidence** → Proceed with warning flag
- **Error Rate > 0.08 (8%)** → **Fail** → Stop analysis, report content mismatch

**Rationale**:
- 5% threshold: Allows for minor ASR noise (diacritics, alif variants) while catching real errors
- 8% threshold: Permissive boundary that captures learners with heavy accents or minor word skips
- >8%: High confidence of wrong verse or major content error

### Normalization Specification (Exact Implementation)

To prevent false positives from orthographic variations, **all text** (reference and ASR output) must be normalized before error rate calculation:

```python
import re

def normalize_arabic_text(text: str) -> str:
    """
    Normalize Arabic text for robust comparison.

    Operations (in order):
    1. Remove Arabic combining marks: [\u064B-\u0652\u0670\u06DC-\u06ED]
    2. Remove tatweel (kashida): \u0640
    3. Map all alif forms to plain alif: أ/إ/آ/ٱ → ا
    4. Map hamza carriers: ؤ → و, ئ → ي
    5. Drop standalone hamza: ء → (removed)
    6. Strip punctuation and collapse whitespace to single space
    """
    # Remove combining marks (diacritics, shadda, sukun, etc.)
    text = re.sub(r'[\u064B-\u0652\u0670\u06DC-\u06ED]', '', text)

    # Remove tatweel
    text = text.replace('\u0640', '')

    # Normalize alif forms
    alif_forms = {'أ': 'ا', 'إ': 'ا', 'آ': 'ا', 'ٱ': 'ا'}
    for old, new in alif_forms.items():
        text = text.replace(old, new)

    # Normalize hamza carriers
    text = text.replace('ؤ', 'و').replace('ئ', 'ي').replace('ء', '')

    # Strip punctuation and normalize whitespace
    text = re.sub(r'[^\w\s]', '', text, flags=re.UNICODE)
    text = re.sub(r'\s+', ' ', text).strip()

    return text
```

**Critical**: Normalization must be applied to **both** reference and hypothesis before computing WER/CER. Failure to normalize will cause false positives.

### Implementation with `rapidfuzz`

The MVP uses `rapidfuzz` for efficient Levenshtein distance calculation:

```python
from rapidfuzz.distance import Levenshtein

def compute_wer(reference_words: List[str], hypothesis_words: List[str]) -> float:
    """Compute Word Error Rate."""
    distance = Levenshtein.distance(reference_words, hypothesis_words)
    wer = distance / max(len(reference_words), 1)
    return wer

def compute_cer(reference_text: str, hypothesis_text: str) -> float:
    """Compute Character Error Rate."""
    distance = Levenshtein.distance(list(reference_text), list(hypothesis_text))
    cer = distance / max(len(reference_text), 1)
    return cer
```

### Output Schema

```python
{
    "error_rate": float,           # WER or CER (0.0 to 1.0+)
    "metric_type": "wer" | "cer",  # Which metric was used
    "confidence": "high" | "medium" | "fail",
    "should_proceed": bool,        # True if error_rate ≤ 0.08
    "reference_normalized": str,   # After normalization
    "hypothesis_normalized": str,  # After normalization
    "raw_transcript": str          # Original ASR output
}
```

### Example Scenarios

**Scenario 1: Short Ayah, Perfect Recitation**
- Reference: "قُلْ هُوَ" (2 words)
- User: "قل هو"
- Normalized reference: "قل هو" (7 chars including space)
- Normalized hypothesis: "قل هو" (7 chars)
- **CER = 0.0** → High Confidence → Proceed ✓

**Scenario 2: Short Ayah, One Word Error**
- Reference: "قُلْ هُوَ" (2 words)
- User: "قل الله"
- Normalized reference: "قل هو" (7 chars)
- Normalized hypothesis: "قل الله" (8 chars)
- **CER = 3 edits / 7 chars ≈ 0.43** (43%) → Fail → Stop ✗

**Scenario 3: Long Ayah, Minor ASR Noise**
- Reference: 20-word Ayah
- User: Perfect recitation, but ASR misrecognizes 1 word
- **WER = 1 / 20 = 0.05** (5%) → High Confidence → Proceed ✓

---

> **MVP Reason Note**: A hybrid WER/CER approach provides robust content verification across all Ayah lengths. WER is the industry standard for longer texts, but becomes statistically unstable on very short texts (≤3 words) where a single error produces misleading error rates (33-100%). CER provides more granular, reliable error measurement in these cases. The unified 5%/8% thresholds are conservative enough to catch real content errors while tolerating minor ASR noise from diacritics and orthographic variants.

---

## Alternative Considered: Hybrid Architecture

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

---
**Related**: [Orchestrator Implementation](orchestrator.md) | [← M7 Overview](overview.md)
