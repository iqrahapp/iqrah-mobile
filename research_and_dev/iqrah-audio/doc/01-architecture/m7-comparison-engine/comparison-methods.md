# M7: Comparison Methods

[← Back to M7 Overview](overview.md) | [↑ Navigation](../../NAVIGATION.md)

**Purpose**: Detailed scoring algorithms for tajweed, prosody, pronunciation, and voice quality

---

## M7.1: Tajweed Comparison

**Input**: Aligned phonemes with **sifat (Tajweed properties) from Muaalem model output**

**Two-Tier Architecture**:
- **Tier 1 Baseline**: Uses sifat directly from Muaalem (10+ rules, 70-85% accuracy, free)
- **Tier 2 Specialized**: Pluggable modules for enhanced detection (Madd 95%+, Ghunnah 90%+, Qalqalah 85%+)

**Code**:
```python
def _compare_tajweed(self, student_phonemes, ref_phonemes, audio):
    """Run all validators (Tier 1 baseline + enabled Tier 2 modules)

    student_phonemes contains sifat from Muaalem:
    - p["sifat"]["ghonna"]: Ghunnah flag
    - p["sifat"]["qalqla"]: Qalqalah flag
    - p["sifat"]["tafkheem"]: Tafkheem flag
    - p["sifat"]["madd"]: Madd flag
    - etc. (10+ rules)
    """
    results = {}
    violations = []

    # Madd (Tier 2 specialized: probabilistic duration modeling)
    madd_violations = self.validators["madd"].validate(student_phonemes)
    num_madd = sum(1 for p in student_phonemes if p.get("sifat", {}).get("madd", False))
    results["madd"] = 100 - (len(madd_violations) / num_madd * 100) if num_madd > 0 else 100
    violations.extend(madd_violations)

    # Ghunnah (Tier 1 baseline or Tier 2 formant analysis if enabled)
    ghunnah_violations = self.validators["ghunnah"].validate(student_phonemes, audio)
    num_ghunnah = sum(1 for p in student_phonemes if p.get("sifat", {}).get("ghonna", False))
    results["ghunnah"] = 100 - (len(ghunnah_violations) / num_ghunnah * 100) if num_ghunnah > 0 else 100
    violations.extend(ghunnah_violations)

    # Qalqalah (Tier 1 baseline or Tier 2 burst detection if enabled)
    qalqalah_violations = self.validators["qalqalah"].validate(student_phonemes, audio)
    num_qalqalah = sum(1 for p in student_phonemes if p.get("sifat", {}).get("qalqla", False))
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

**Note**: Tier 1 baseline validators use sifat flags directly from Muaalem for comprehensive coverage (10+ rules). Tier 2 specialized modules provide enhanced accuracy for critical rules (Madd, Ghunnah, Qalqalah) using acoustic analysis.

## M7.2: Prosody Comparison

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

## M7.3: Pronunciation Comparison

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

## M7.4: Voice Quality Comparison

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
**Related**: [Orchestrator Implementation](orchestrator.md) | [← M7 Overview](overview.md)
