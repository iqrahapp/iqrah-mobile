# SwiftF0 Migration Plan

## Why SwiftF0?

SwiftF0 is the new state-of-the-art for pitch extraction, significantly outperforming CREPE:

### Performance Comparison
| Metric | SwiftF0 | CREPE |
|--------|---------|-------|
| Accuracy (10dB SNR) | 91.80% | ~79% |
| Average Accuracy | 90.2% | 85.3% |
| Speed | **42× faster** | Baseline |
| Model Size | 95,842 params | ~2M params |
| Noise Robustness | Excellent | Good |

### Key Advantages for Iqrah Audio
1. **Better Accuracy**: 5% higher average accuracy
2. **Noise Resilience**: Maintains 91.80% at 10dB SNR (noisy environments)
3. **Speed**: 42× faster enables real-time analysis possibilities
4. **Smaller Model**: Faster downloads, less memory

## Implementation Steps

### 1. Install SwiftF0
```bash
conda activate iqrah
pip install swiftf0
```

**Source**: https://github.com/lars76/swift-f0

### 2. Replace Pitch Extractor

Create new module: `src/iqrah_audio/analysis/pitch_extractor_swiftf0.py`

```python
import numpy as np
import librosa
from swiftf0 import swiftf0
from typing import Dict, List

def extract_pitch_swiftf0(
    audio_path: str,
    sr: int = 16000,
    hop_length: int = 160  # 10ms at 16kHz
) -> Dict:
    """
    Extract pitch using SwiftF0.

    Args:
        audio_path: Path to audio file
        sr: Sample rate (16000 recommended)
        hop_length: Hop size in samples (10ms = 160 samples at 16kHz)

    Returns:
        Dictionary with time, f0_hz, confidence, voiced arrays
    """
    # Load audio
    audio, _ = librosa.load(audio_path, sr=sr, mono=True)

    # SwiftF0 expects audio normalized to [-1, 1]
    audio = audio / np.max(np.abs(audio) + 1e-8)

    # Extract pitch
    f0_hz, confidence = swiftf0(audio, sr, hop_length=hop_length)

    # Create time array
    time = np.arange(len(f0_hz)) * hop_length / sr

    # Determine voiced frames (SwiftF0 outputs 0 for unvoiced)
    voiced = f0_hz > 0

    return {
        'time': time.tolist(),
        'f0_hz': f0_hz.tolist(),
        'confidence': confidence.tolist(),
        'voiced': voiced.tolist(),
        'sample_rate': sr,
        'duration': float(time[-1]) if len(time) > 0 else 0.0
    }
```

### 3. Update Backend

Modify `src/iqrah_audio/analysis/pitch_extractor.py`:

```python
# Add import
try:
    from .pitch_extractor_swiftf0 import extract_pitch_swiftf0
    USE_SWIFTF0 = True
except ImportError:
    USE_SWIFTF0 = False
    import crepe

def extract_pitch_from_file(audio_path: str, sr: int = 16000) -> Dict:
    """Extract pitch using best available method."""
    if USE_SWIFTF0:
        return extract_pitch_swiftf0(audio_path, sr)
    else:
        # Fallback to CREPE
        return extract_pitch_crepe(audio_path, sr)
```

### 4. Testing & Validation

Create test suite: `tests/test_pitch_comparison.py`

```python
def test_swiftf0_vs_crepe():
    """
    Compare SwiftF0 vs CREPE on sample ayahs.

    Metrics:
    - Extraction time
    - F0 accuracy (if ground truth available)
    - Consistency (correlation between methods)
    """
    pass

def test_noise_robustness():
    """
    Test with artificially added noise at different SNR levels.
    """
    pass
```

## Migration Timeline

### Phase 1 (Week 1): Setup & Testing
- Install SwiftF0
- Create new pitch extractor module
- Test on sample ayahs
- Compare results with CREPE
- Measure performance improvements

### Phase 2 (Week 2): Integration
- Update backend to use SwiftF0
- Add fallback to CREPE if SwiftF0 unavailable
- Update documentation
- Test full pipeline

### Phase 3 (Week 3): Validation
- Test with 100+ diverse ayahs
- Validate metrics accuracy
- User testing for quality
- Performance benchmarking

## Metrics System

### Accuracy Metrics to Track
1. **Pitch Estimation Accuracy**: Compare against ground truth (if available)
2. **Consistency**: Correlation between SwiftF0 and CREPE
3. **Speed**: Time per second of audio
4. **Noise Resilience**: Performance at different SNR levels

### Implementation

Create `src/iqrah_audio/evaluation/metrics.py`:

```python
def calculate_pitch_accuracy(
    predicted_f0: np.ndarray,
    ground_truth_f0: np.ndarray,
    tolerance_cents: float = 50.0
) -> Dict:
    """
    Calculate pitch estimation accuracy.

    Metrics:
    - Raw Pitch Accuracy (RPA): % of frames within tolerance
    - Raw Chroma Accuracy (RCA): % of frames within tolerance (octave invariant)
    - Voicing Decision Error (VDE): % of voiced/unvoiced mistakes
    """
    pass

def calculate_correlation(
    method1_f0: np.ndarray,
    method2_f0: np.ndarray
) -> float:
    """
    Calculate correlation between two pitch estimation methods.
    """
    pass

def benchmark_speed(
    audio_path: str,
    method: callable
) -> Dict:
    """
    Benchmark extraction speed.

    Returns:
    - total_time: Total extraction time
    - audio_duration: Audio duration
    - real_time_factor: duration / time (>1 = faster than real-time)
    """
    pass
```

## Expected Improvements

### Speed
- CREPE: ~2-3 seconds for 5-second ayah
- SwiftF0: ~0.05-0.1 seconds for 5-second ayah
- **Improvement: 20-60× faster**

### Accuracy
- CREPE: 85.3% average
- SwiftF0: 90.2% average
- **Improvement: +4.9% accuracy**

### Noise Robustness
- CREPE @ 10dB SNR: ~79%
- SwiftF0 @ 10dB SNR: 91.80%
- **Improvement: +12.8% at 10dB SNR**

## Resources

- SwiftF0 Paper: https://arxiv.org/abs/2408.15658
- SwiftF0 GitHub: https://github.com/lars76/swift-f0
- Benchmark Results: https://github.com/w-okada/pitch-estimator-benchmark

## Notes

- SwiftF0 is optimized for monophonic audio (perfect for Quran recitation)
- Trained on diverse datasets including speech and singing
- CPU-friendly (no GPU required)
- Compatible with real-time applications if needed in future
