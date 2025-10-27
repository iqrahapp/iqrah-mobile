# Module M4: Tajweed Validation

[← Back to Overview](overview.md) | [↑ Navigation](../NAVIGATION.md)

---

## M4: TAJWEED VALIDATION

**Input**: Aligned phonemes (from M3) with timestamps + sifat, phonetic reference, audio
**Output**:
```python
{
    "violations": [
        {
            "rule": str,              # "Ghunnah", "Madd", "Qalqalah", etc.
            "phoneme_idx": int,
            "timestamp": float,
            "expected": str,
            "actual": str,
            "severity": "critical" | "moderate" | "minor",
            "confidence": float,
            "tier": 1 | 2,            # Which tier detected this
            "feedback": str           # User-facing message
        }
    ],
    "scores_by_rule": {
        "ghunnah": float,            # 0-100
        "qalqalah": float,
        "madd": float,
        "tafkhim": float,
        # ... one score per rule
    },
    "overall_score": float,          # Weighted average
    "tier1_coverage": float,         # % rules covered by Tier 1
    "tier2_enhancements": int        # # violations caught by Tier 2
}
```

---

## 1. OVERVIEW

This module implements **two-tier Tajweed validation** that combines baseline sifat from Muaalem (Tier 1) with specialized phonetic analysis modules (Tier 2).

### Two-Tier Architecture (CRITICAL)

**Tier 1: Baseline Sifat (from Muaalem)**
- **Free**: Comes directly from Muaalem model output (no additional training)
- **Comprehensive**: Covers 10+ Tajweed rules from Day 1
- **Accuracy**: 70-85% per rule (sufficient for MVP)
- **Serves as**: (1) MVP feature, (2) Baseline for Tier 2 enhancement

**Tier 2: Specialized Modules (Pluggable)**
- **Advanced detection**: Per-rule acoustic analysis
- **Probabilistic**: Uses statistical models (Gaussian, SVM, MLP)
- **Accuracy**: 90-99% per rule (targeted improvement)
- **Examples**:
  - **Madd**: Probabilistic duration modeling (local + global distributions)
  - **Ghunnah**: Enhanced formant analysis + Tier 1 baseline
  - **Qalqalah**: Acoustic burst detection + Tier 1 baseline

### Design Principles

1. **Modularity**: Each Tajweed rule = independent module
2. **Plug-and-play**: Enable/disable modules without affecting others
3. **Baseline-first**: Always run Tier 1, optionally enhance with Tier 2
4. **Probabilistic**: Outputs are confidence distributions, not binary
5. **Graceful degradation**: If Tier 2 fails, fall back to Tier 1

### Why Two Tiers?

- **Fast MVP**: Deliver 10+ rules immediately using Muaalem's free sifat
- **Incremental enhancement**: Add Tier 2 modules one by one
- **Resource efficiency**: Only run expensive Tier 2 for low-confidence cases
- **Modular testing**: Test each rule independently

---

## 2. TIER 1: BASELINE SIFAT INTERPRETER

### 2.1 Purpose

Parse Muaalem's sifat output, compare against expected Tajweed rules from phonetic reference, and identify violations based on probability thresholds.

### 2.2 Implementation

**Module**: [`src/iqrah/tajweed/baseline_interpreter.py`](../../src/iqrah/tajweed/baseline_interpreter.py)

**Interface**:
```python
from iqrah.tajweed.baseline_interpreter import BaselineTajweedInterpreter

interpreter = BaselineTajweedInterpreter(
    confidence_threshold=0.7  # Require prob > 0.7 to accept prediction
)

violations = interpreter.validate(
    aligned_phonemes=aligned_phonemes,  # From M3, includes sifat
    phonetic_ref=phonetic_ref           # Expected rules
)

# Output: dict[str, list[Violation]]
# - "ghunnah": list[Violation]
# - "qalqalah": list[Violation]
# - "tafkhim": list[Violation]
# - ... (one list per sifat type)
```

**Logic**:
```python
def validate(self, aligned_phonemes, phonetic_ref):
    violations = defaultdict(list)

    for phoneme in aligned_phonemes:
        expected_sifat = get_expected_from_ref(phoneme, phonetic_ref)
        predicted_sifat = phoneme.sifa  # From Muaalem

        # Check Ghunnah rule
        if expected_sifat.ghonna == "maghnoon":
            if (predicted_sifat.ghonna.text != "maghnoon" or
                predicted_sifat.ghonna.prob < self.confidence_threshold):

                violations["ghunnah"].append(Violation(
                    rule="Ghunnah",
                    phoneme_idx=phoneme.idx,
                    timestamp=phoneme.start,
                    expected="maghnoon",
                    actual=predicted_sifat.ghonna.text,
                    confidence=predicted_sifat.ghonna.prob,
                    severity=self._compute_severity(predicted_sifat.ghonna.prob),
                    tier=1,
                    feedback=f"Nasal resonance expected at {phoneme.start:.2f}s"
                ))

        # Repeat for other sifat: qalqalah, tafkhim, itbaq, etc.

    return dict(violations)
```

**Severity Computation**:
```python
def _compute_severity(self, prob: float) -> str:
    """Determine severity based on confidence"""
    if prob < 0.3:
        return "critical"  # Very low confidence, likely wrong
    elif prob < 0.6:
        return "moderate"  # Borderline, review needed
    else:
        return "minor"     # Mostly correct, minor issue
```

### 2.3 Supported Rules (Tier 1)

| Rule | Sifat Property | Expected Accuracy | Description |
|------|----------------|-------------------|-------------|
| **Ghunnah** | `ghonna` | 70-85% | Nasalization (ن، م with specific rules) |
| **Qalqalah** | `qalqla` | 75-80% | Echo/bounce on ق، ط، ب، ج، د with sukoon |
| **Tafkhim/Tarqiq** | `tafkheem_or_tarqeeq` | 80-85% | Emphatic vs plain (e.g., ص vs س) |
| **Itbaq** | `itbaq` | 80% | Pharyngealized (ص، ض، ط، ظ) |
| **Safeer** | `safeer` | 85% | Whistling (ص، س، ز) |
| **Shidda/Rakhawa** | `shidda_or_rakhawa` | 80% | Tense vs lax consonants |
| **Hams/Jahr** | `hams_or_jahr` | 75% | Whispered vs voiced |

**Note**: Madd is **NOT** in Tier 1 baseline (Muaalem doesn't handle duration well), so we implement Madd in Tier 2 as a priority.

---

## 3. TIER 2: SPECIALIZED MODULES

### 3.1 Madd (Vowel Elongation) - PRIORITY 1

**Accuracy Target**: 95%+ (Phase 1), 99%+ (Phase 2)

**Challenge**: Estimating 1 harakat duration is context-dependent:
- Reciter's speed varies (slow vs fast recitation)
- Local tempo changes (emphatic sections, pauses)
- Personal style differences

**Approach**: Probabilistic duration modeling using Gaussian distributions

**Module**: [`src/iqrah/tajweed/madd_validator.py`](../../src/iqrah/tajweed/madd_validator.py)

---

#### 3.1.1 Duration Distribution Estimation

**Local Distribution** (recent recitation pace):

```python
def estimate_local_distribution(aligned_phonemes, window_seconds=10.0):
    """
    Estimate harakat duration from recent audio window.

    Args:
        aligned_phonemes: All aligned phonemes from recitation
        window_seconds: Time window to analyze (last N seconds)

    Returns:
        (mean_harakat_ms, std_harakat_ms)
    """
    # Get all short vowels in last N seconds
    short_vowels = [
        p for p in aligned_phonemes
        if p.phoneme in {'a', 'i', 'u'} and
           p.end > (max_time - window_seconds)
    ]

    durations = [(p.end - p.start) * 1000 for p in short_vowels]  # ms

    if len(durations) < 5:
        return 100.0, 20.0  # Fallback to default

    mean_ms = np.mean(durations)
    std_ms = np.std(durations)

    return mean_ms, std_ms
```

**Interpretation**:
- Low σ_local (<20ms): Stable pace (experienced reciter)
- High σ_local (>50ms): Inconsistent pace (beginner)

**Global Distribution** (overall Surah pace):

```python
def load_global_distribution(user_id, surah_id, db_session):
    """
    Load user's historical harakat distribution for this Surah.

    Args:
        user_id: User UUID
        surah_id: Surah number (1-114)
        db_session: Database session

    Returns:
        (mean_harakat_ms, std_harakat_ms, n_samples) or None if no data
    """
    result = db_session.query(UserMaddDistribution).filter_by(
        user_id=user_id,
        surah_id=surah_id
    ).first()

    if result:
        return result.mean_harakat_ms, result.std_harakat_ms, result.n_samples
    else:
        return None  # No historical data yet
```

**Gaussian Mixture** (future enhancement - Phase 2):
- Model: Multiple rhythms/tempos per page
- Capture: Slow sections (Madd 6), fast sections (Madd 2)
- Implementation: Scikit-learn `GaussianMixture`

---

#### 3.1.2 Madd Rule Validation

**Madd Types & Expected Durations**:

| Type | Arabic | Duration | Example |
|------|--------|----------|---------|
| **Madd Tabi'i** (Natural) | مد طبيعي | 1 harakat | ما |
| **Madd Permissible** (Jaiz) | مد جائز | 2-4 harakats | المآء |
| **Madd Necessary** (Lazim) | مد لازم | 6 harakats | الحآقّة |

**Validation Algorithm**:

```python
def validate_madd(phoneme, duration_ms, local_dist, global_dist=None):
    """
    Validate Madd elongation using probabilistic model.

    Args:
        phoneme: AlignedPhoneme with expected Madd rule
        duration_ms: Actual duration from audio
        local_dist: (mean, std) from recent window
        global_dist: (mean, std) from user history (optional)

    Returns:
        MaddViolation or None
    """
    # Get expected duration
    expected_harakats = get_madd_rule(phoneme)  # e.g., 6 for lazim
    expected_duration = expected_harakats * local_dist[0]  # mean

    # Tolerance based on local std (2-sigma rule)
    tolerance = 2 * local_dist[1]

    # Check violation
    deviation = abs(duration_ms - expected_duration)

    if deviation > tolerance:
        # Compute z-score
        z_score = (duration_ms - expected_duration) / local_dist[1]

        # Probability of observing this deviation (normal distribution)
        from scipy.stats import norm
        confidence = 1 - norm.cdf(abs(z_score))

        return MaddViolation(
            rule="Madd",
            subtype=f"{expected_harakats}_harakats",
            phoneme_idx=phoneme.idx,
            timestamp=phoneme.start,
            expected_duration=expected_duration,
            actual_duration=duration_ms,
            z_score=z_score,
            confidence=confidence,
            severity="critical" if abs(z_score) > 3 else "moderate",
            tier=2,
            feedback=self._generate_madd_feedback(phoneme, expected_harakats,
                                                    expected_duration, duration_ms)
        )

    return None  # Valid Madd
```

**Interface**:

```python
from iqrah.tajweed.madd_validator import MaddValidator

validator = MaddValidator()

# Update distributions from recent audio
validator.update_distributions(
    aligned_phonemes=aligned_phonemes,   # Last 10s or last waqf segment
    global_history=load_global_distribution(user_id, surah_id, db)
)

# Validate each Madd
violations = validator.validate(aligned_phonemes, phonetic_ref)

# Output: list[MaddViolation]
# - phoneme_idx, timestamp, expected_harakats, actual_duration
# - z_score, confidence, local_mean, local_std
```

**Storage Schema** (for global distributions):

```sql
CREATE TABLE user_madd_distributions (
    id SERIAL PRIMARY KEY,
    user_id UUID NOT NULL,
    surah_id INT NOT NULL,
    mean_harakat_ms FLOAT NOT NULL,     -- μ_global
    std_harakat_ms FLOAT NOT NULL,      -- σ_global
    n_samples INT NOT NULL,             -- # recitations used
    updated_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(user_id, surah_id)
);

CREATE INDEX idx_user_madd ON user_madd_distributions(user_id, surah_id);
```

---

### 3.2 Ghunnah (Nasalization) - PRIORITY 2

**Accuracy Target**: 70-85% (Tier 1 baseline) → 90%+ (Tier 2 enhanced)

**Baseline**: Muaalem's `ghonna` sifat (from Tier 1)

**Enhancement**: Formant analysis for low-confidence cases

**Module**: [`src/iqrah/tajweed/ghunnah_validator.py`](../../src/iqrah/tajweed/ghunnah_validator.py)

#### Approach

1. **Use Tier 1 baseline** as initial detection
2. **For low-confidence cases** (prob < 0.8), perform formant analysis:
   - Extract F1, F2, F3 from nasal segment
   - Check for nasal formant signature:
     - Low F1 (<500Hz)
     - F2-F1 coupling (energy transfer)
     - Elevated energy in 250-350Hz band
3. **Combine**: Weighted average of baseline prob + formant score

#### Formant Extraction

```python
import parselmouth

def extract_ghunnah_formants(audio, start, end, sr=16000):
    """
    Extract formant features for Ghunnah detection.

    Args:
        audio: Audio array
        start, end: Time boundaries (seconds)
        sr: Sample rate

    Returns:
        dict with f1_hz, f2_hz, f3_hz, nasal_energy_db
    """
    segment = audio[int(start*sr):int(end*sr)]
    sound = parselmouth.Sound(segment, sr)

    # Formants via Praat
    formants = sound.to_formant_burg(max_number_of_formants=3)
    midpoint = (start + end) / 2
    f1_hz = formants.get_value_at_time(1, midpoint)
    f2_hz = formants.get_value_at_time(2, midpoint)
    f3_hz = formants.get_value_at_time(3, midpoint)

    # Nasal energy (250-350Hz band)
    from scipy.signal import butter, filtfilt
    b, a = butter(4, [250, 350], btype='band', fs=sr)
    nasal_band = filtfilt(b, a, segment)
    nasal_energy_db = 10 * np.log10(np.mean(nasal_band**2) + 1e-10)

    return {
        "f1_hz": f1_hz,
        "f2_hz": f2_hz,
        "f3_hz": f3_hz,
        "nasal_energy_db": nasal_energy_db
    }
```

#### Combined Scoring

```python
class GhunnahValidator:
    def __init__(self, use_formants=True, formant_weight=0.3):
        self.use_formants = use_formants
        self.formant_weight = formant_weight

    def validate(self, aligned_phonemes, audio, sr=16000):
        violations = []

        for p in aligned_phonemes:
            if p.sifa.ghonna is None:
                continue  # Not a Ghunnah phoneme

            baseline_prob = p.sifa.ghonna.prob  # Tier 1 confidence

            # If high confidence, trust Tier 1
            if baseline_prob > 0.8:
                continue

            # Low confidence: enhance with formants
            if self.use_formants:
                features = extract_ghunnah_formants(audio, p.start, p.end, sr)
                formant_score = self._score_formants(features)

                # Weighted combination
                combined_confidence = (
                    (1 - self.formant_weight) * baseline_prob +
                    self.formant_weight * formant_score
                )
            else:
                combined_confidence = baseline_prob

            # Check for violation
            if combined_confidence < 0.7:
                violations.append(Violation(
                    rule="Ghunnah",
                    phoneme_idx=p.idx,
                    timestamp=p.start,
                    expected="Nasal resonance (ghunnah)",
                    actual=f"Confidence: {combined_confidence:.2f}",
                    confidence=combined_confidence,
                    severity="moderate" if combined_confidence < 0.5 else "minor",
                    tier=2,
                    feedback="Nasal resonance too weak. Hum through nose."
                ))

        return violations

    def _score_formants(self, features):
        """Score formant features for Ghunnah presence (0-1)"""
        score = 0.5  # Neutral

        # Low F1 indicates nasalization
        if features["f1_hz"] < 500:
            score += 0.2

        # Elevated nasal energy
        if features["nasal_energy_db"] > -20:
            score += 0.2

        # F2-F1 coupling (F2 pulled down toward F1)
        if 1000 < features["f2_hz"] < 1800:
            score += 0.1

        return min(score, 1.0)
```

---

### 3.3 Qalqalah (Echo/Bounce) - PRIORITY 3

**Accuracy Target**: 75-80% (Tier 1 baseline) → 85%+ (Tier 2 enhanced)

**Baseline**: Muaalem's `qalqla` sifat (from Tier 1)

**Enhancement**: Acoustic burst detection

**Module**: [`src/iqrah/tajweed/qalqalah_validator.py`](../../src/iqrah/tajweed/qalqalah_validator.py)

**Qalqalah Letters**: ق، ط، ب، ج، د (with sukoon)

#### Acoustic Correlates

- **Burst detection**: Sharp transient at consonant release
- **Zero-crossing rate (ZCR)**: High during burst
- **Spectral centroid**: Higher (brightness from burst)
- **Energy spike**: Localized RMS increase

#### Feature Extraction

```python
import librosa

def extract_qalqalah_features(audio, start, end, sr=16000):
    """Extract acoustic features for Qalqalah burst detection"""
    segment = audio[int(start*sr):int(end*sr)]

    # Zero-crossing rate
    zcr = librosa.feature.zero_crossing_rate(segment)[0]

    # Spectral centroid
    centroid = librosa.feature.spectral_centroid(y=segment, sr=sr)[0]

    # Energy envelope (RMS)
    rms = librosa.feature.rms(y=segment)[0]

    # Burst detection: max RMS > 1.5× median
    burst_idx = np.argmax(rms)
    has_burst = rms[burst_idx] > 1.5 * np.median(rms)

    return {
        "zcr_mean": np.mean(zcr),
        "zcr_std": np.std(zcr),
        "centroid_mean": np.mean(centroid),
        "rms_max": np.max(rms),
        "rms_std": np.std(rms),
        "has_burst": has_burst
    }
```

#### Validation (Tier 1 + Tier 2 Combination)

```python
class QalqalahValidator:
    QALQALAH_LETTERS = {'q', 'T', 'b', 'j', 'd'}  # Buckwalter notation

    def validate(self, aligned_phonemes, audio, sr=16000):
        violations = []

        for p in aligned_phonemes:
            if p.phoneme not in self.QALQALAH_LETTERS:
                continue

            baseline_prob = p.sifa.qalqla.prob if p.sifa.qalqla else 0.0

            # If high confidence, trust Tier 1
            if baseline_prob > 0.8:
                continue

            # Low confidence: enhance with burst detection
            features = extract_qalqalah_features(audio, p.start, p.end, sr)

            # Combined score
            if features["has_burst"] and features["zcr_mean"] > 0.3:
                burst_score = 0.8
            else:
                burst_score = 0.3

            combined_confidence = 0.6 * baseline_prob + 0.4 * burst_score

            # Check violation
            if combined_confidence < 0.6:
                violations.append(Violation(
                    rule="Qalqalah",
                    phoneme_idx=p.idx,
                    timestamp=p.start,
                    expected="Sharp burst with echo",
                    actual=f"Burst confidence: {combined_confidence:.2f}",
                    confidence=combined_confidence,
                    severity="moderate",
                    tier=2,
                    feedback="Qalqalah requires short, explosive release"
                ))

        return violations
```

---

### 3.4 Other Rules (Lower Priority)

- **Tafkhim/Tarqiq**: Tier 1 baseline sufficient (80-85%)
- **Idgham, Ikhfa**: Future (Phase 2+, requires complex phoneme merging analysis)

---

## 4. MODULE ORCHESTRATION

**Module**: [`src/iqrah/tajweed/orchestrator.py`](../../src/iqrah/tajweed/orchestrator.py)

**Purpose**: Coordinate Tier 1 + Tier 2 modules, merge violations, compute scores

### Interface

```python
from iqrah.tajweed.orchestrator import TajweedOrchestrator

orchestrator = TajweedOrchestrator(
    enable_baseline=True,
    enable_madd=True,
    enable_ghunnah=True,
    enable_qalqalah=False  # Can disable individual Tier 2 modules
)

result = orchestrator.validate(
    aligned_phonemes=aligned_phonemes,
    phonetic_ref=phonetic_ref,
    audio=audio,
    user_global_stats=load_user_stats(user_id, surah_id, db)  # For Madd
)

# Output: TajweedResult
# - violations: list[Violation] (all rules, sorted by timestamp)
# - scores_by_rule: dict[str, float] (per-rule accuracy 0-100)
# - overall_score: float (weighted average)
# - tier1_coverage: float (% rules from Tier 1)
# - tier2_enhancements: int (# violations caught by Tier 2)
```

### Orchestrator Logic

```python
class TajweedOrchestrator:
    def __init__(self, enable_baseline=True, enable_madd=True,
                 enable_ghunnah=True, enable_qalqalah=False):
        self.enable_baseline = enable_baseline
        self.enable_madd = enable_madd
        self.enable_ghunnah = enable_ghunnah
        self.enable_qalqalah = enable_qalqalah

        # Initialize validators
        self.baseline_interpreter = BaselineTajweedInterpreter()
        self.madd_validator = MaddValidator()
        self.ghunnah_validator = GhunnahValidator()
        self.qalqalah_validator = QalqalahValidator()

    def validate(self, aligned_phonemes, phonetic_ref, audio,
                 user_global_stats=None):
        violations = []

        # Tier 1: Baseline
        if self.enable_baseline:
            baseline_viol = self.baseline_interpreter.validate(
                aligned_phonemes, phonetic_ref
            )
            for rule, viol_list in baseline_viol.items():
                violations.extend(viol_list)

        # Tier 2: Specialized modules
        if self.enable_madd:
            self.madd_validator.update_distributions(
                aligned_phonemes, user_global_stats
            )
            madd_viol = self.madd_validator.validate(
                aligned_phonemes, phonetic_ref
            )
            violations.extend(madd_viol)

        if self.enable_ghunnah:
            ghunnah_viol = self.ghunnah_validator.validate(
                aligned_phonemes, audio
            )
            # Merge with baseline (override low-confidence Tier 1 predictions)
            violations = self._merge_ghunnah_violations(violations, ghunnah_viol)

        if self.enable_qalqalah:
            qalqalah_viol = self.qalqalah_validator.validate(
                aligned_phonemes, audio
            )
            violations = self._merge_qalqalah_violations(violations, qalqalah_viol)

        # Aggregate results
        return self._aggregate_results(violations, aligned_phonemes)

    def _aggregate_results(self, violations, aligned_phonemes):
        """Compute per-rule scores and overall score"""
        # Group violations by rule
        violations_by_rule = defaultdict(list)
        for v in violations:
            violations_by_rule[v.rule].append(v)

        # Compute per-rule scores
        scores_by_rule = {}
        for rule in ["Ghunnah", "Qalqalah", "Madd", "Tafkhim", "Itbaq"]:
            total = count_rule_instances(aligned_phonemes, rule)
            violations_count = len(violations_by_rule[rule])
            if total > 0:
                scores_by_rule[rule] = max(0, 100 * (1 - violations_count / total))
            else:
                scores_by_rule[rule] = None  # N/A

        # Compute overall score (weighted average)
        weights = {"Ghunnah": 0.25, "Qalqalah": 0.15, "Madd": 0.40,
                   "Tafkhim": 0.10, "Itbaq": 0.10}
        weighted_scores = [
            weights[rule] * score
            for rule, score in scores_by_rule.items()
            if score is not None
        ]
        overall_score = sum(weighted_scores) / sum(
            w for r, w in weights.items() if scores_by_rule[r] is not None
        )

        # Compute tier metrics
        tier1_violations = [v for v in violations if v.tier == 1]
        tier2_violations = [v for v in violations if v.tier == 2]

        return {
            "violations": sorted(violations, key=lambda v: v.timestamp),
            "scores_by_rule": scores_by_rule,
            "overall_score": overall_score,
            "tier1_coverage": len(tier1_violations) / len(violations) * 100,
            "tier2_enhancements": len(tier2_violations)
        }
```

---

## 5. DATA FLOW

```
┌──────────────────────────────────────────────────────────────┐
│ INPUT: Aligned Phonemes (M3) + Phonetic Ref + Audio + User Stats │
└──────────────────────────────────────────────────────────────┘
                            ↓
            ┌───────────────┴───────────────┐
            ↓                               ↓
  ┌──────────────────────┐       ┌──────────────────────┐
  │ TIER 1: Baseline     │       │ Phonemes with sifat  │
  │ Sifat Interpreter    │←──────│ (from Muaalem)       │
  └──────────────────────┘       └──────────────────────┘
            ↓
  Baseline Violations (Ghunnah, Qalqalah, Tafkhim, Itbaq, etc.)
            │
            ↓
  ┌─────────────────────────────────────────────────────────┐
  │ TIER 2: Specialized Modules (optional, per-rule)       │
  └─────────────────────────────────────────────────────────┘
            │
    ┌───────┼───────┐
    ↓       ↓       ↓
 ┌─────┐ ┌──────┐ ┌──────────┐
 │Madd │ │Ghunnah│ │Qalqalah│
 │Probab│ │Formant│ │  Burst │
 └─────┘ └──────┘ └──────────┘
    │       │       │
    └───────┼───────┘
            ↓
  Enhanced Violations (higher precision)
            │
            ↓
  ┌──────────────────────┐
  │ Orchestrator         │
  │ - Merge violations   │
  │ - Compute scores     │
  │ - Resolve conflicts  │
  └──────────────────────┘
            ↓
┌──────────────────────────────────────────────────┐
│ OUTPUT: TajweedResult                           │
│ - All violations (Tier 1 + Tier 2)             │
│ - Per-rule scores (0-100)                      │
│ - Overall Tajweed score                        │
│ - Tier metrics (coverage, enhancements)       │
└──────────────────────────────────────────────────┘
```

---

## 6. PERFORMANCE TARGETS

### Phase 1 (MVP - Tier 1 + Madd Tier 2)

| Rule | Tier | Target Accuracy |
|------|------|----------------|
| **Ghunnah** | Tier 1 only | 70-85% |
| **Qalqalah** | Tier 1 only | 75-80% |
| **Madd** | Tier 2 probabilistic | 95%+ |
| **Tafkhim/Tarqiq** | Tier 1 only | 80-85% |
| **Itbaq** | Tier 1 only | 80% |
| **Overall** | Combined | 85%+ |

### Phase 2 (Tier 1 + Enhanced Tier 2)

| Rule | Tier | Target Accuracy |
|------|------|----------------|
| **Ghunnah** | Tier 2 formants | 90%+ |
| **Qalqalah** | Tier 2 burst | 85%+ |
| **Madd** | Tier 2 refined | 99%+ |
| **Overall** | Combined | 93%+ |
| **Expert correlation** | Spearman's ρ | >0.75 |

---

## 7. IMPLEMENTATION TASKS

### T4.1: Baseline Interpreter [AI Agent - HIGH PRIORITY]

**Description**: Parse Muaalem sifat output and compare against expected rules

**Checklist**:
- [ ] Create [`src/iqrah/tajweed/baseline_interpreter.py`](../../src/iqrah/tajweed/baseline_interpreter.py)
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

### T4.2: Madd Validator - Duration Estimation [AI Agent + HUMAN]

**Description**: Implement local and global harakat distribution estimation

**Checklist**:
- [ ] Create [`src/iqrah/tajweed/madd_validator.py`](../../src/iqrah/tajweed/madd_validator.py)
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

### T4.3: Madd Validator - Rule Validation [AI Agent]

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

### T4.4: Database Schema for Global Stats [AI Agent]

**Description**: Store per-user, per-Surah harakat distributions

**Checklist**:
- [ ] Design table `user_madd_distributions` (SQL schema provided above)
- [ ] Implement ORM model (SQLAlchemy or similar)
- [ ] Create migration script (Alembic)
- [ ] Implement insert/update logic (upsert on conflict)
- [ ] Test query performance (index on user_id + surah_id)

**Schema** (see Section 3.1.2 above)

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

### T4.5: Tajweed Orchestrator [AI Agent]

**Description**: Coordinate Tier 1 + Tier 2 modules and merge violations

**Checklist**:
- [ ] Create [`src/iqrah/tajweed/orchestrator.py`](../../src/iqrah/tajweed/orchestrator.py)
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

### T4.6: Ghunnah Formant Analyzer [AI Agent - OPTIONAL FOR MVP]

**Description**: Extract formants for low-confidence Ghunnah enhancement

**Checklist**:
- [ ] Create [`src/iqrah/tajweed/ghunnah_validator.py`](../../src/iqrah/tajweed/ghunnah_validator.py)
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

### T4.7: E2E M4 Pipeline Test [AI Agent]

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

### T4.8: Expert Validation [HUMAN - HIGH PRIORITY]

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

**Total M4**: ~72 hours (AI: 48h, HUMAN: 24h)

---

## 8. EVALUATION

### Metrics

1. **Per-rule accuracy**: Precision, recall, F1 per Tajweed rule
   - True Positive: System detects violation, expert confirms
   - False Positive: System detects violation, expert rejects
   - False Negative: System misses violation, expert confirms

2. **Tier comparison**: Measure accuracy gain from Tier 2
   - Baseline (Tier 1 only) accuracy
   - Enhanced (Tier 1 + Tier 2) accuracy
   - Improvement: Δ = Enhanced - Baseline

3. **Expert correlation**: Spearman's ρ vs human raters
   - Compute overall Tajweed score per recitation
   - Compare system score vs expert score
   - Target: ρ > 0.75

### Test Sets

- **100 Ayahs** with expert annotations (all rules)
- **20 edge cases** per rule (difficult pronunciations)
- **Ablation study**: Tier 1 only vs Tier 1+2

### Evaluation Procedure

```python
# Load expert annotations
expert_labels = load_expert_annotations("data/expert_annotations.csv")

# Run system on same samples
system_results = []
for sample in expert_labels:
    audio = load_audio(sample.audio_path)
    aligned_phonemes = run_m3_pipeline(audio, sample.reference_text)
    tajweed_result = orchestrator.validate(aligned_phonemes, ...)
    system_results.append(tajweed_result)

# Compute per-rule metrics
for rule in ["Ghunnah", "Qalqalah", "Madd"]:
    tp, fp, fn = compare_rule(system_results, expert_labels, rule)
    precision = tp / (tp + fp)
    recall = tp / (tp + fn)
    f1 = 2 * precision * recall / (precision + recall)
    print(f"{rule}: P={precision:.2f}, R={recall:.2f}, F1={f1:.2f}")

# Compute correlation
system_scores = [r["overall_score"] for r in system_results]
expert_scores = [e.overall_score for e in expert_labels]
rho, p_value = spearmanr(system_scores, expert_scores)
print(f"Spearman's ρ = {rho:.3f} (p={p_value:.4f})")
```

---

## 9. CONFIGURATION

Users can configure Tier 1 and Tier 2 modules via YAML or JSON:

**Example** (`config/tajweed.yaml`):
```yaml
tajweed:
  tier1_enabled: true
  tier1_confidence_threshold: 0.7   # Prob threshold for Tier 1 violations

  tier2_modules:
    madd:
      enabled: true
      local_window_seconds: 10      # Recent audio window for pace estimation
      global_weight: 0.3            # Weight of global history vs local
      z_score_threshold: 2.0        # Sigma tolerance

    ghunnah:
      enabled: true                 # Set to false to use Tier 1 only
      formant_weight: 0.3           # Weight of formant score vs Tier 1
      confidence_threshold: 0.8     # Only run formants if Tier 1 < this

    qalqalah:
      enabled: false                # Not ready for MVP

  scoring:
    weights:
      ghunnah: 0.25
      qalqalah: 0.15
      madd: 0.40
      tafkhim: 0.10
      itbaq: 0.10
```

**Loading configuration**:
```python
import yaml

with open("config/tajweed.yaml") as f:
    config = yaml.safe_load(f)

orchestrator = TajweedOrchestrator(
    enable_baseline=config["tajweed"]["tier1_enabled"],
    enable_madd=config["tajweed"]["tier2_modules"]["madd"]["enabled"],
    enable_ghunnah=config["tajweed"]["tier2_modules"]["ghunnah"]["enabled"],
    enable_qalqalah=config["tajweed"]["tier2_modules"]["qalqalah"]["enabled"]
)
```

---

## 10. RISKS & MITIGATIONS

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|-----------|
| Tier 1 accuracy <70% for some rules | Medium | High | Fine-tune Muaalem in Phase 2; prioritize Tier 2 for low-accuracy rules |
| Madd distribution estimation fails (high variance) | Medium | Medium | Fallback to fixed rule-based durations (e.g., 1 harakat = 100ms) |
| Tier 2 modules too slow (>1s latency) | Low | Medium | Async processing; cache formant results; limit Tier 2 to low-confidence cases |
| Over-reliance on Tier 1 baseline | Low | Medium | Regular expert validation; continuous improvement based on user feedback |
| Configuration complexity | Low | Low | Provide sensible defaults; document each parameter with examples |

---

## 11. REFERENCES

### Models & Datasets

- **Muaalem sifat documentation**: obadx/quran-muaalem repository
- **Tajweed rules**: Classical Arabic Tajweed texts, expert consultations

### Academic Papers

- Madd duration research: [Quranic phonetics and duration studies]
- Formant analysis: Praat manual, parselmouth library documentation
- Qalqalah acoustics: [Arabic phonetics papers on stop consonant bursts]

### Tools & Libraries

- **Parselmouth**: Python interface to Praat (formant extraction)
  - `pip install praat-parselmouth`
  - Docs: https://parselmouth.readthedocs.io/

- **Librosa**: Audio feature extraction (ZCR, spectral centroid, RMS)
  - `pip install librosa`
  - Docs: https://librosa.org/

- **SciPy**: Gaussian distributions, signal processing (bandpass filters)
  - `from scipy.stats import norm`
  - `from scipy.signal import butter, filtfilt`

### Related Documentation

- [M3: Phoneme Recognition & Alignment](m3-phoneme-alignment.md) - Sifat source
- [M7: Comparison Engine](m7-comparison-engine/comparison-methods.md) - Score aggregation
- [Phase 1 Tasks](../03-tasks/phase1-offline.md) - Implementation timeline

---

**Next**: [Module M5: Voice Quality Analysis](m5-voice-quality.md) | [← Back to Overview](overview.md)
