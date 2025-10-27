# Prosody Algorithms Implementation

[↑ Navigation](../../NAVIGATION.md) | [← Technical Details](../infrastructure.md)

**Purpose**: Complete implementations of prosody analysis algorithms (Fujisaki, Declination, Tilt)

**Audience**: AI agents implementing M6 (Prosody Analysis)

**Includes**:
- Fujisaki Model Decomposition (F0 → phrase + accent components)
- Declination Modeling (baseline F0 drop over time)
- Tilt Parametrization (contour shape classification)

---

## 1.1: Complete Fujisaki Model Implementation

```python
from scipy.optimize import minimize

def fit_fujisaki(f0_contour, times):
    """
    Decompose F0 into phrase + accent components

    F0(t) = F_b * exp(A_p(t) + Σ A_a(t))
    where:
    - F_b = baseline frequency
    - A_p = phrase component (slow modulation)
    - A_a = accent components (rapid modulations)

    Returns: phrase_commands, accent_commands
    """
    def fujisaki_model(t, phrase_params, accent_params, Fb):
        """
        Compute F0 at time t given parameters

        phrase_params: list of (Ap, T1p, Tp) tuples
        accent_params: list of (Aa, T1a, Ta) tuples
        Fb: baseline frequency (Hz)
        """
        # Phrase component: low-pass filtered (slow changes)
        # Formula: Ap * (1 - (1 + (t-T1p)/Tp) * exp(-(t-T1p)/Tp)) if t >= T1p else 0
        Fp = sum(
            Ap * (1 - (1 + (t - T1p) / Tp) * np.exp(-(t - T1p) / Tp))
            for Ap, T1p, Tp in phrase_params
            if t >= T1p
        )

        # Accent component: high-pass filtered (rapid changes)
        # Formula: Aa * (1 - (1 + (t-T1a)/Ta) * exp(-(t-T1a)/Ta)) if t >= T1a else 0
        Fa = sum(
            Aa * (1 - (1 + (t - T1a) / Ta) * np.exp(-(t - T1a) / Ta))
            for Aa, T1a, Ta in accent_params
            if t >= T1a
        )

        # Total F0 in log domain, then exponentiate
        return Fb * np.exp(Fp + Fa)

    def objective_function(params):
        """
        Minimize squared error between observed and predicted F0

        params structure:
        - params[0]: Fb (baseline)
        - params[1:4]: (Ap, T1p, Tp) for single phrase command
        - params[4:]: pairs of (Aa, T1a) for accent commands (Ta is fixed)
        """
        Fb = params[0]
        phrase_params = [(params[1], params[2], params[3])]  # Single phrase

        # Extract accent parameters (pairs)
        accent_params = []
        Ta = 0.04  # Fixed accent time constant (40ms typical)
        for i in range(4, len(params), 2):
            if i + 1 < len(params):
                accent_params.append((params[i], params[i+1], Ta))

        # Compute predicted F0 for all time points
        f0_pred = np.array([
            fujisaki_model(t, phrase_params, accent_params, Fb)
            for t in times
        ])

        # Mean squared error
        mse = np.sum((f0_contour - f0_pred) ** 2)
        return mse

    # Initial parameter guess
    # Fb: mean of F0 contour
    # Phrase: amplitude=0.1, onset=0s, duration=total_duration
    # Accents: 3-5 accents spaced evenly, amplitude=0.05
    duration = times[-1]
    num_accents = 4
    accent_spacing = duration / (num_accents + 1)

    initial_params = [
        np.mean(f0_contour),  # Fb
        0.1, 0.0, duration,   # Phrase: (Ap, T1p, Tp)
    ]
    for i in range(1, num_accents + 1):
        initial_params.extend([0.05, i * accent_spacing])  # (Aa, T1a)

    # Bounds: Fb > 0, amplitudes in [-1, 1], times in [0, duration]
    bounds = [
        (50, 500),           # Fb: 50-500 Hz
        (-1, 1), (0, duration), (0.1, duration),  # Phrase
    ]
    for _ in range(num_accents):
        bounds.extend([(-1, 1), (0, duration)])  # Accent (Aa, T1a)

    # Optimize using L-BFGS-B
    result = minimize(
        objective_function,
        initial_params,
        method='L-BFGS-B',
        bounds=bounds
    )

    # Extract optimized parameters
    Fb_opt = result.x[0]
    phrase_opt = [(result.x[1], result.x[2], result.x[3])]

    accents_opt = []
    Ta = 0.04
    for i in range(4, len(result.x), 2):
        if i + 1 < len(result.x):
            accents_opt.append({
                "amplitude": result.x[i],
                "time": result.x[i+1],
                "time_constant": Ta
            })

    return {
        "baseline_hz": Fb_opt,
        "phrase_commands": [
            {
                "amplitude": phrase_opt[0][0],
                "onset": phrase_opt[0][1],
                "duration": phrase_opt[0][2]
            }
        ],
        "accent_commands": accents_opt,
        "mse": result.fun
    }
```

**Latency**: 100-200ms per ayah (scipy.optimize is slow)

---

## 1.2: Complete Declination Modeling

```python
def fit_declination(f0_contour, times):
    """
    Model baseline F0 drop over utterance
    Typical: 10-15% decrease from start to end

    Extracts F0 minima (baseline) and fits exponential decay
    """
    from scipy.signal import argrelextrema
    from scipy.optimize import curve_fit

    # Step 1: Extract F0 minima (approximate baseline)
    minima_idx = argrelextrema(f0_contour, np.less, order=5)[0]

    # If too few minima, use bottom 20% of F0 values
    if len(minima_idx) < 5:
        threshold = np.percentile(f0_contour, 20)
        minima_idx = np.where(f0_contour <= threshold)[0]

    f0_minima = f0_contour[minima_idx]
    t_minima = times[minima_idx]

    # Step 2: Fit exponential decay
    # Model: f0(t) = a * exp(-b*t) + c
    # where:
    # - a: initial amplitude
    # - b: decay rate (Hz/second)
    # - c: asymptotic floor
    def exp_decay(t, a, b, c):
        return a * np.exp(-b * t) + c

    try:
        popt, pcov = curve_fit(
            exp_decay,
            t_minima,
            f0_minima,
            p0=[f0_minima[0] - f0_minima[-1], 0.1, f0_minima[-1]],  # Initial guess
            maxfev=5000
        )

        # Compute R-squared
        predicted = exp_decay(t_minima, *popt)
        ss_res = np.sum((f0_minima - predicted) ** 2)
        ss_tot = np.sum((f0_minima - np.mean(f0_minima)) ** 2)
        r_squared = 1 - (ss_res / ss_tot) if ss_tot > 0 else 0

        # Slope in Hz/second
        slope_hz_per_sec = -popt[0] * popt[1]

        # Step 3: Remove declination trend from entire F0 contour
        baseline = exp_decay(times, *popt)
        f0_normalized = f0_contour - baseline + np.mean(baseline)

        return {
            "slope": slope_hz_per_sec,
            "r_squared": r_squared,
            "decay_params": {
                "a": popt[0],
                "b": popt[1],
                "c": popt[2]
            },
            "f0_normalized": f0_normalized,
            "baseline": baseline
        }
    except RuntimeError:
        # If fitting fails, return None
        return {
            "slope": 0.0,
            "r_squared": 0.0,
            "decay_params": None,
            "f0_normalized": f0_contour,
            "baseline": np.full_like(f0_contour, np.mean(f0_contour))
        }
```

---

## 1.3: Complete Tilt Parametrization

```python
def extract_tilt_features(f0_contour):
    """
    Classify contour shapes: rising, falling, level, convex, concave

    Tilt parameters quantify contour asymmetry:
    - tilt_amp: Amplitude asymmetry (rise vs fall magnitude)
    - tilt_dur: Duration asymmetry (rise vs fall duration)
    """
    # Step 1: Find peak (maximum F0)
    peak_idx = np.argmax(f0_contour)

    # Step 2: Rise phase (start to peak)
    if peak_idx > 0:
        rise_amp = f0_contour[peak_idx] - f0_contour[0]
        rise_dur = peak_idx
    else:
        rise_amp = 0
        rise_dur = 1

    # Step 3: Fall phase (peak to end)
    if peak_idx < len(f0_contour) - 1:
        fall_amp = f0_contour[peak_idx] - f0_contour[-1]
        fall_dur = len(f0_contour) - peak_idx
    else:
        fall_amp = 0
        fall_dur = 1

    # Step 4: Compute tilt parameters
    # tilt_amp: positive = more rise, negative = more fall
    tilt_amp = (rise_amp - fall_amp) / (rise_amp + fall_amp + 1e-6)

    # tilt_dur: positive = longer rise, negative = longer fall
    tilt_dur = (rise_dur - fall_dur) / (rise_dur + fall_dur + 1e-6)

    # Step 5: Classify shape based on tilt parameters
    if abs(tilt_amp) < 0.2:
        shape = "level"
    elif tilt_amp > 0.5:
        shape = "rising"
    elif tilt_amp < -0.5:
        shape = "falling"
    elif rise_dur < fall_dur:
        shape = "convex"  # Quick rise, slow fall
    else:
        shape = "concave"  # Slow rise, quick fall

    # Step 6: Additional contour features
    # Convexity: second derivative
    if len(f0_contour) > 2:
        second_derivative = np.diff(np.diff(f0_contour))
        convexity = np.mean(second_derivative)
    else:
        convexity = 0.0

    return {
        "tilt_amp": tilt_amp,
        "tilt_dur": tilt_dur,
        "shape": shape,
        "rise_amplitude": rise_amp,
        "fall_amplitude": fall_amp,
        "rise_duration": rise_dur,
        "fall_duration": fall_dur,
        "peak_position": peak_idx / len(f0_contour),  # Normalized 0-1
        "convexity": convexity
    }
```

---
**Related**: [Maqam Classifier](maqam.md) | [Architecture M6](../../01-architecture/m6-prosody.md)
