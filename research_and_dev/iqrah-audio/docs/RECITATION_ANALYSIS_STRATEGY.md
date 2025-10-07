# Quranic Recitation Analysis & Comparison Strategy

## Overview
This document outlines the comprehensive statistical analysis system for comparing user recitations against expert Qari recitations.

## Core Concept: Tempo Normalization
**Key Insight**: A user can recite faster/slower than the Qari but maintain PERFECT rhythm if they respect the counting system. All comparisons must be tempo-normalized.

---

## Phase 1: Statistical Feature Extraction (Current Phase)

### 1.1 Tempo Analysis
**Metric**: Inter-syllable intervals (ISI)

**Implementation**:
```python
def analyze_tempo(phonemes: List[Dict]) -> Dict:
    """
    Extract tempo statistics from phoneme timings.

    Returns:
        - mean_isi: Mean inter-syllable interval (seconds)
        - std_isi: Standard deviation (stability measure)
        - tempo_distribution: Probability distribution (Gaussian)
        - tempo_stability_score: 1 - (std/mean) normalized to 0-100
    """
```

**Interpretation**:
- **Low std**: Stable tempo (expert)
- **High std**: Unstable tempo (beginner)
- **Mean ISI**: Base pace (e.g., 0.2s = 5 syllables/sec)

---

### 1.2 Pitch Distribution Analysis
**Metric**: Gaussian Mixture Model (GMM) of pitch values

**Implementation**:
```python
def analyze_pitch_distribution(pitch_data: Dict) -> Dict:
    """
    Model pitch as Gaussian Mixture (2-4 components).

    Returns:
        - mean_pitch: Overall mean frequency (Hz)
        - std_pitch: Overall standard deviation
        - gmm_components: List of (mean, std, weight) tuples
        - pitch_range: (min, max) Hz
        - pitch_stability: Based on GMM component variances
    """
```

**Why GMM?**
- Quranic recitation has multiple "pitch levels" (modes)
- Each level corresponds to a Gaussian component
- Better than single Gaussian for multi-modal distributions

---

### 1.3 Count Duration Analysis
**Metric**: Fundamental time unit (1 count = N seconds)

**Implementation**:
```python
def analyze_count_duration(phonemes: List[Dict], tajweed_rules: List[str]) -> Dict:
    """
    Estimate count duration from non-elongated phonemes.

    Strategy:
        1. Filter phonemes with no Madd (baseline = 1 count)
        2. Extract durations
        3. Fit Gaussian: N(μ, σ²)
        4. μ = mean count duration
        5. σ = count consistency (low = expert)

    Returns:
        - mean_count: Mean duration (seconds)
        - std_count: Standard deviation
        - count_distribution: Gaussian parameters
        - count_precision_score: Based on CV (coefficient of variation)
    """
```

**Key Insight**: Short, consistent phonemes ≈ 1 count. Use these as baseline.

---

### 1.4 Elongation (Madd) Analysis
**Metric**: Actual vs. expected count ratios

**Implementation**:
```python
def analyze_madd_accuracy(phonemes: List[Dict], mean_count: float) -> Dict:
    """
    Verify elongation accuracy using count distributions.

    For each Madd phoneme:
        1. Get expected counts (2, 4, or 6 based on Tajweed rule)
        2. Calculate actual_counts = duration / mean_count
        3. Compute match_score using normal distribution:
           P(actual | N(expected, σ²))

    Returns:
        - madd_accuracy: Percentage of correct elongations
        - per_madd_scores: Individual phoneme scores
        - madd_distribution: Actual vs expected histograms
    """
```

**Scoring Formula**:
```
score = 100 × exp(-((actual - expected)² / (2 × tolerance²)))
tolerance = 0.25 counts (expert) to 0.5 counts (beginner)
```

---

### 1.5 Rhythm Analysis (Tempo-Normalized)
**Metric**: Phoneme onset timing patterns

**Implementation**:
```python
def analyze_rhythm(phonemes: List[Dict], tempo_normalization: float) -> Dict:
    """
    Compare phoneme timing after tempo normalization.

    Steps:
        1. Normalize all times: t_norm = t_actual / tempo_ratio
        2. Extract onset times: [t₁, t₂, ..., tₙ]
        3. Compute interval sequence: [Δt₁, Δt₂, ..., Δtₙ₋₁]
        4. Compare sequences using DTW (Dynamic Time Warping)

    Returns:
        - rhythm_similarity: DTW distance (0 = perfect match)
        - onset_correlation: Pearson correlation of normalized onsets
        - rhythm_stability: Variance of normalized intervals
    """
```

**Why DTW?**: Handles local timing variations while preserving overall pattern.

---

## Phase 2: Comparison Metrics (User vs. Qari)

### 2.1 Tempo Comparison
```python
def compare_tempo(user_stats: Dict, qari_stats: Dict) -> Dict:
    """
    Compare tempo characteristics.

    Metrics:
        - pace_ratio: user_mean_isi / qari_mean_isi
          * < 1: Faster than Qari
          * > 1: Slower than Qari

        - stability_diff: user_std - qari_std
          * Positive: Less stable
          * Negative: More stable (rare for beginners)

        - tempo_match_score: Based on distribution overlap (KL divergence)
    """
```

---

### 2.2 Pitch Comparison
```python
def compare_pitch(user_stats: Dict, qari_stats: Dict) -> Dict:
    """
    Compare pitch distributions.

    Metrics:
        - pitch_shift: Median difference (Hz)
          * User might recite in different key (ok!)

        - normalized_gmm_distance: Compare GMM shapes after normalizing mean
          * Measures melodic contour similarity

        - pitch_variance_ratio: user_std / qari_std
          * > 1: More variation (unstable)
          * < 1: Less variation (monotone?)

        - melodic_correlation: Pitch trajectory correlation
    """
```

---

### 2.3 Count Accuracy Comparison
```python
def compare_count_precision(user_stats: Dict, qari_stats: Dict) -> Dict:
    """
    Compare count consistency.

    Metrics:
        - count_precision_ratio: user_std / qari_std
          * > 1: Less precise
          * ≈ 1: Similar precision

        - count_duration_match: Overlap of count distributions
    """
```

---

### 2.4 Madd Accuracy Comparison
```python
def compare_madd_accuracy(user_madd: Dict, qari_madd: Dict) -> Dict:
    """
    Compare elongation accuracy.

    Metrics:
        - madd_score_diff: user_accuracy - qari_accuracy
          * Qari should be ~95-100%
          * User might be 60-90%

        - per_type_accuracy: Breakdown by Madd type (2/4/6 counts)

        - elongation_consistency: Variance across Madd instances
    """
```

---

### 2.5 Rhythm Similarity (Tempo-Normalized)
```python
def compare_rhythm(user_phonemes: List, qari_phonemes: List,
                   tempo_ratio: float) -> Dict:
    """
    Compare rhythmic patterns after tempo normalization.

    THIS IS THE KEY METRIC for determining if user maintains
    proper elongations and rhythm despite different pace.

    Metrics:
        - dtw_distance: After tempo normalization
          * 0: Perfect rhythm match
          * > 0: Deviations from ideal rhythm

        - onset_timing_accuracy: % of phonemes within tolerance

        - rhythm_consistency_score: Overall match (0-100)
    """
```

---

## Phase 3: UI Visualization

### 3.1 Statistics Display Panel
```html
<div class="stats-panel">
  <!-- Tempo -->
  <div class="stat-card">
    <h3>Tempo</h3>
    <div class="value">4.8 syllables/sec</div>
    <div class="stability">Stability: 92/100 ⭐</div>
    <canvas id="tempoDistribution"></canvas>
  </div>

  <!-- Pitch -->
  <div class="stat-card">
    <h3>Pitch Distribution</h3>
    <div class="value">Mean: 245 Hz</div>
    <div class="value">Range: 180-280 Hz</div>
    <canvas id="pitchGMM"></canvas>
  </div>

  <!-- Count -->
  <div class="stat-card">
    <h3>Count Duration</h3>
    <div class="value">0.18s ± 0.02s</div>
    <div class="precision">Precision: 95/100 ⭐</div>
    <canvas id="countDistribution"></canvas>
  </div>

  <!-- Madd -->
  <div class="stat-card">
    <h3>Elongation Accuracy</h3>
    <div class="value">98% correct</div>
    <canvas id="maddAccuracy"></canvas>
  </div>
</div>
```

### 3.2 Comparison View (User vs. Qari)
```html
<div class="comparison-panel">
  <div class="metric">
    <h4>Tempo</h4>
    <div class="comparison-bar">
      <div class="qari-bar">Qari: 5.0 syl/s</div>
      <div class="user-bar">You: 4.5 syl/s (10% slower)</div>
    </div>
    <div class="score">Match: 85/100</div>
  </div>

  <div class="metric">
    <h4>Rhythm (Tempo-Normalized)</h4>
    <div class="score">98/100 ⭐ Excellent!</div>
    <p>Your rhythm perfectly matches the Qari despite slower pace.</p>
  </div>

  <div class="metric">
    <h4>Elongations</h4>
    <div class="score">75/100 ⚠️ Needs work</div>
    <ul>
      <li>2-count Madd: 85% ✓</li>
      <li>4-count Madd: 60% ⚠️ (too short)</li>
      <li>6-count Madd: 70% ⚠️</li>
    </ul>
  </div>
</div>
```

---

## Phase 4: Implementation Order

### Step 1: Basic Statistics (30 min)
- [x] Tempo analysis (ISI mean/std)
- [ ] Pitch distribution (mean/std/range)
- [ ] Count duration estimation

### Step 2: Advanced Statistics (45 min)
- [ ] GMM fitting for pitch
- [ ] Madd accuracy scoring
- [ ] Rhythm analysis (DTW)

### Step 3: Backend API (20 min)
- [ ] Add `/api/analyze_statistics/{surah}/{ayah}` endpoint
- [ ] Return JSON with all stats
- [ ] Cache results

### Step 4: Frontend Visualization (40 min)
- [ ] Statistics display panel
- [ ] Chart.js integration for distributions
- [ ] Score cards with visual indicators

### Step 5: Comparison System (60 min)
- [ ] User recording upload
- [ ] User analysis pipeline
- [ ] Comparison computation
- [ ] Comparison UI

---

## Key Formulas

### Tempo Stability Score
```
stability = 100 × (1 - min(std_isi / mean_isi, 1))
```

### Count Precision Score
```
precision = 100 × (1 - min(CV, 1))
where CV = std_count / mean_count
```

### Madd Accuracy Score
```
For each madd_phoneme:
    expected_counts = tajweed_rule_mapping[rule]
    actual_counts = duration / mean_count
    error = |actual_counts - expected_counts|
    score_i = 100 × exp(-(error² / (2 × tolerance²)))

madd_accuracy = mean(score_i for all madd)
```

### Rhythm Similarity Score
```
dtw_normalized = dtw_distance / max(len(user), len(qari))
rhythm_score = 100 × (1 - min(dtw_normalized, 1))
```

### Overall Recitation Score
```
overall_score = weighted_average([
    tempo_stability * 0.15,
    pitch_consistency * 0.10,
    count_precision * 0.20,
    madd_accuracy * 0.30,
    rhythm_similarity * 0.25
])
```

---

## Next Steps After Implementation

1. **Validation**: Test on multiple Qaris to establish baseline ranges
2. **Calibration**: Adjust tolerance thresholds based on real data
3. **Feedback System**: Generate actionable feedback ("Your 4-count Madds are 0.5 counts too short")
4. **Progress Tracking**: Store user stats over time to show improvement
5. **Leaderboard**: Compare users (anonymized) to motivate practice

---

## Technical Dependencies

- **scipy**: GMM fitting, statistical distributions
- **dtaidistance**: DTW implementation
- **Chart.js**: Frontend visualization
- **numpy**: Numerical computations

---

## Files to Create/Modify

### New Files:
1. `src/iqrah_audio/analysis/statistics_analyzer.py` - Core stats
2. `src/iqrah_audio/analysis/comparison_engine.py` - Comparison logic
3. `src/iqrah_audio/analysis/score_calculator.py` - Scoring formulas
4. `static/js/statistics_viz.js` - Frontend charts

### Modified Files:
1. `app_qari_final.py` - Add statistics endpoint
2. `static/qari_final.html` - Add statistics panel

---

## Expected Output Format

```json
{
  "tempo": {
    "mean_isi": 0.185,
    "std_isi": 0.023,
    "stability_score": 92,
    "distribution": {"mean": 0.185, "std": 0.023}
  },
  "pitch": {
    "mean_pitch": 245.3,
    "std_pitch": 28.5,
    "range": [180, 285],
    "gmm_components": [
      {"mean": 230, "std": 15, "weight": 0.4},
      {"mean": 260, "std": 12, "weight": 0.6}
    ]
  },
  "count": {
    "mean_duration": 0.18,
    "std_duration": 0.02,
    "precision_score": 95,
    "distribution": {"mean": 0.18, "std": 0.02}
  },
  "madd": {
    "overall_accuracy": 98,
    "by_type": {
      "2_count": {"accuracy": 97, "count": 5},
      "4_count": {"accuracy": 100, "count": 3},
      "6_count": {"accuracy": 95, "count": 2}
    }
  },
  "rhythm": {
    "onset_times": [0.0, 0.2, 0.4, ...],
    "interval_stability": 0.94
  }
}
```
