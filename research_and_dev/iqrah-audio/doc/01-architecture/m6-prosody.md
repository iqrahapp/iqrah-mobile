# Module M6: Prosodic Analysis

[← Back to Overview](overview.md) | [↑ Navigation](../NAVIGATION.md)

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

---

**Next**: [Module M7: Comparison Engine](m7-comparison-engine/overview.md) | [← Back to Overview](overview.md)
