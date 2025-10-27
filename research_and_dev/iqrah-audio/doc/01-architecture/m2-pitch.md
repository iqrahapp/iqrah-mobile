# Module M2: Pitch Extraction

[← Back to Overview](overview.md) | [↑ Navigation](../NAVIGATION.md)

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

**Next**: [Module M3: Phoneme Alignment](m3-phoneme-alignment.md) | [← Back to Overview](overview.md)
