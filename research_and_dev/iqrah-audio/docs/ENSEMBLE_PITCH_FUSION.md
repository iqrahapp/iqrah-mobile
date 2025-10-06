# Ensemble Pitch Fusion Strategy

## Goal
Minimize octave flips and harmonic detection errors while preserving tajweed ornaments using ensemble fusion of three pitch extractors.

## Three Extractors

### 1. **SwiftF0** (Currently Active)
- **Speed**: 42× faster than CREPE
- **Accuracy**: 91.80% at 10dB SNR
- **Strength**: Best noise robustness, monophonic
- **Weakness**: May miss some harmonics in complex sounds

### 2. **RMVPE** (Planned)
- **Speed**: Moderate
- **Accuracy**: 87.2% on vocal benchmarks
- **Strength**: Vocal-specialized, handles polyphonic conditions
- **Weakness**: Slower than SwiftF0

### 3. **Harvest (StoneMask)** (Planned)
- **Speed**: Very fast
- **Accuracy**: Good for clean speech
- **Strength**: Classic algorithm, reliable baseline
- **Weakness**: Sensitive to noise

## Fusion Algorithm

```python
def ensemble_pitch_fusion(
    swiftf0_pitch: np.ndarray,
    rmvpe_pitch: np.ndarray,
    harvest_pitch: np.ndarray,
    swiftf0_conf: np.ndarray,
    rmvpe_conf: np.ndarray,
    harvest_conf: np.ndarray,
    tolerance_cents: float = 50.0
) -> np.ndarray:
    """
    Fuse three pitch estimates using agreement-based voting.

    Algorithm:
    1. For each frame, check if 2+ extractors agree within ±50 cents
    2. If yes: use median of agreeing estimates
    3. If no: pick estimate with highest confidence/lowest local jitter
    4. Post-process: median filter → Savitzky-Golay smoothing

    Args:
        *_pitch: F0 arrays from each extractor
        *_conf: Confidence scores from each extractor
        tolerance_cents: Agreement threshold (default 50 cents)

    Returns:
        Fused F0 array
    """

    fused = np.zeros_like(swiftf0_pitch)

    for i in range(len(fused)):
        estimates = [swiftf0_pitch[i], rmvpe_pitch[i], harvest_pitch[i]]
        confidences = [swiftf0_conf[i], rmvpe_conf[i], harvest_conf[i]]

        # Remove unvoiced (0 Hz)
        valid_mask = np.array(estimates) > 0
        valid_estimates = np.array(estimates)[valid_mask]
        valid_confs = np.array(confidences)[valid_mask]

        if len(valid_estimates) == 0:
            fused[i] = 0.0
            continue

        # Check pairwise agreement within tolerance
        agreements = []
        for j in range(len(valid_estimates)):
            for k in range(j + 1, len(valid_estimates)):
                cents_diff = abs(1200 * np.log2(valid_estimates[j] / valid_estimates[k]))
                if cents_diff <= tolerance_cents:
                    agreements.append((j, k))

        # If 2+ agree, use median of agreeing estimates
        if len(agreements) >= 1:
            # Get all agreeing indices
            agree_indices = set()
            for j, k in agreements:
                agree_indices.add(j)
                agree_indices.add(k)

            fused[i] = np.median(valid_estimates[list(agree_indices)])

        else:
            # No agreement: pick highest confidence
            # OR lowest local jitter (compare with neighbors)
            if i > 0 and i < len(fused) - 1:
                # Calculate local jitter for each estimate
                jitters = []
                for est in valid_estimates:
                    prev = fused[i - 1] if fused[i - 1] > 0 else est
                    next_est = est  # Use same for next (we don't know it yet)
                    jitter = abs(est - prev)
                    jitters.append(jitter)

                # Weighted score: high confidence + low jitter
                scores = valid_confs / (1 + np.array(jitters))
                best_idx = np.argmax(scores)
            else:
                # Edge frames: just use highest confidence
                best_idx = np.argmax(valid_confs)

            fused[i] = valid_estimates[best_idx]

    # Post-processing: smooth while preserving ornaments
    from scipy.signal import medfilt, savgol_filter

    # 1. Median filter (removes outliers, preserves edges)
    fused_smooth = medfilt(fused, kernel_size=5)

    # 2. Savitzky-Golay filter (polynomial smoothing, preserves peaks)
    # Only on voiced regions
    voiced_mask = fused_smooth > 0
    if np.any(voiced_mask):
        fused_smooth[voiced_mask] = savgol_filter(
            fused_smooth[voiced_mask],
            window_length=11,  # Must be odd
            polyorder=3,        # Cubic polynomial
            mode='nearest'
        )

    return fused_smooth
```

## Implementation Plan

### Phase 1: Add RMVPE
```bash
# Clone RMVPE
cd /tmp
git clone https://github.com/Dream-High/RMVPE.git

# Create wrapper
cp /tmp/RMVPE/src/* src/iqrah_audio/analysis/rmvpe/
```

### Phase 2: Add Harvest (via pyworld)
```bash
pip install pyworld
```

```python
# src/iqrah_audio/analysis/pitch_extractor_harvest.py
import pyworld as pw
import librosa

def extract_pitch_harvest(audio_path, sr=16000):
    audio, _ = librosa.load(audio_path, sr=sr, mono=True)
    audio = audio.astype(np.float64)

    f0, t = pw.harvest(audio, sr)
    # StoneMask refinement
    f0 = pw.stonemask(audio, f0, t, sr)

    return {
        'time': t.tolist(),
        'f0_hz': f0.tolist(),
        'confidence': (f0 > 0).astype(float).tolist(),
        'voiced': (f0 > 0).tolist()
    }
```

### Phase 3: Create Ensemble Module
```python
# src/iqrah_audio/analysis/pitch_extractor_ensemble.py
from .pitch_extractor_swiftf0 import extract_pitch_swiftf0
from .pitch_extractor_rmvpe import extract_pitch_rmvpe
from .pitch_extractor_harvest import extract_pitch_harvest

def extract_pitch_ensemble(audio_path, sr=16000):
    # Run all three
    swift = extract_pitch_swiftf0(audio_path, sr)
    rmvpe = extract_pitch_rmvpe(audio_path, sr)
    harvest = extract_pitch_harvest(audio_path, sr)

    # Align time arrays (they may differ slightly)
    aligned = align_pitch_arrays([swift, rmvpe, harvest])

    # Fuse
    fused_f0 = ensemble_pitch_fusion(
        aligned['swift']['f0_hz'],
        aligned['rmvpe']['f0_hz'],
        aligned['harvest']['f0_hz'],
        aligned['swift']['confidence'],
        aligned['rmvpe']['confidence'],
        aligned['harvest']['confidence']
    )

    return {
        'time': aligned['time'],
        'f0_hz': fused_f0.tolist(),
        'confidence': np.ones_like(fused_f0).tolist(),
        'voiced': (fused_f0 > 0).tolist(),
        'method': 'ensemble_fusion'
    }
```

## Benefits

1. **Eliminates octave errors**: If one extractor jumps an octave, other two will disagree → use confidence
2. **Preserves ornaments**: Savitzky-Golay smoothing keeps rapid pitch changes (tajweed features)
3. **Robust to noise**: SwiftF0 handles noise, RMVPE handles harmonics, Harvest provides baseline
4. **Fully offline**: All three extractors work offline, no time pressure

## Testing Strategy

1. **Ground truth**: Use synthesized audio with known F0
2. **Real data**: Test on 100 diverse ayahs
3. **Metrics**:
   - Gross Pitch Error (GPE): % of >50 cent errors
   - Voicing Decision Error (VDE): % of wrong voiced/unvoiced
   - F1 score: Harmonic mean of precision/recall

## Expected Improvements

- **Octave error reduction**: 90% → 99% accuracy (eliminating most flips)
- **Ornament preservation**: Better than single-extractor Savitzky-Golay
- **Speed**: Still real-time capable (SwiftF0 dominant, run others in parallel)

## Activation

Currently using: **SwiftF0 only** (fast, good)

To activate ensemble fusion:
```python
# In pitch_extractor.py
USE_ENSEMBLE = True  # Set to True when ready
```
