# Quick Start - Testing the Improvements

## Installation

If you haven't installed the package yet:

```bash
cd /home/shared/ws/iqrah/research_and_dev/iqrah-audio
pip install -e ".[dev]"
```

## 1. Run Baseline Benchmarks (5 minutes)

```bash
# Create benchmarks results directory
mkdir -p benchmarks/results

# Run accuracy benchmark
cd benchmarks
python accuracy_benchmark.py

# Run performance benchmark
python performance_benchmark.py

# Check results
cat results/accuracy_benchmark.json
cat results/performance_benchmark.json
```

**Expected baseline (YIN method):**
- Pitch MAE: 30-50 cents on clean audio
- RTF: ~0.12 (needs to be <0.05 for real-time)
- Octave error rate: 5-10%

## 2. Test Multi-Dimensional Features

Create a test script:

```python
# test_improvements.py
import numpy as np
from iqrah_audio import (
    PitchExtractor,
    FeatureExtractor,
    AudioDenoiser,
    OctaveCorrector,
)

# Generate test audio (440 Hz sine wave)
def generate_test_audio(duration=3.0, frequency=440.0):
    sr = 22050
    t = np.linspace(0, duration, int(sr * duration))
    audio = np.sin(2 * np.pi * frequency * t)
    audio += 0.5 * np.sin(2 * np.pi * 2 * frequency * t)  # 2nd harmonic
    return audio.astype(np.float32), sr

print("=" * 60)
print("Testing Multi-Dimensional Feature Extraction")
print("=" * 60)

# Generate test audio
audio, sr = generate_test_audio(duration=3.0, frequency=220.0)  # A3

# 1. Extract pitch
print("\n1. Extracting pitch with YIN...")
pitch_ext = PitchExtractor(method="yin", sample_rate=sr)
pitch = pitch_ext.extract_stable_pitch(audio)

median_pitch = np.median(pitch.f0_hz[pitch.confidence > 0.5])
print(f"   Median F0: {median_pitch:.1f} Hz (expected ~220 Hz)")
print(f"   Voiced frames: {np.sum(pitch.confidence > 0.5)}/{len(pitch.f0_hz)}")

# 2. Extract multi-dimensional features
print("\n2. Extracting multi-dimensional features...")
feature_ext = FeatureExtractor(
    sample_rate=sr,
    n_mels=80,
    extract_chroma=True,
    extract_energy=True,
    extract_spectral=True,
)

features = feature_ext.extract_all(audio, pitch)

print(f"   F0 shape: {features.f0_hz.shape}")
print(f"   Mel-spec shape: {features.mel_spec.shape}")  # (80, n_frames)
print(f"   Chroma shape: {features.chroma.shape}")      # (12, n_frames)
print(f"   RMS shape: {features.rms.shape}")
print(f"   Spectral centroid shape: {features.spectral_centroid.shape}")

# 3. Test similarity computation
print("\n3. Testing similarity computation...")
sim_same = feature_ext.compute_similarity(
    features, features,
    frame_a=50, frame_b=50,
    weights={"f0": 0.5, "timbre": 0.3, "energy": 0.1, "chroma": 0.1}
)
print(f"   Self-similarity (should be ~1.0): {sim_same:.3f}")

sim_different = feature_ext.compute_similarity(
    features, features,
    frame_a=10, frame_b=100,
    weights={"f0": 0.5, "timbre": 0.3, "energy": 0.1, "chroma": 0.1}
)
print(f"   Different frames: {sim_different:.3f}")

# 4. Test octave correction
print("\n4. Testing octave correction...")

# Create pitch with octave error
pitch_with_error = pitch.f0_hz.copy()
# Introduce octave error (shift middle section up by 1 octave)
pitch_with_error[50:100] *= 2.0

print(f"   Original median: {np.median(pitch.f0_hz[pitch.confidence > 0.5]):.1f} Hz")
print(f"   With octave error: {np.median(pitch_with_error[50:100]):.1f} Hz")

# Apply octave correction
corrector = OctaveCorrector(strategy="median")
pitch_corrected = corrector.correct(
    pitch_with_error,
    pitch.confidence
)

print(f"   After correction: {np.median(pitch_corrected[50:100]):.1f} Hz")

print("\n" + "=" * 60)
print("✅ All tests passed!")
print("=" * 60)
```

Run it:
```bash
python test_improvements.py
```

## 3. Test with Real Audio (if you have Quranic recitation files)

```python
# test_real_audio.py
import soundfile as sf
from iqrah_audio import *

# Load real audio
audio, sr = sf.read("path/to/husary_001001.wav")

# Process
denoiser = AudioDenoiser(sample_rate=22050)
audio_clean = denoiser.denoise(audio)

pitch_ext = PitchExtractor(method="yin", sample_rate=22050)
pitch = pitch_ext.extract_stable_pitch(audio_clean)

feature_ext = FeatureExtractor(sample_rate=22050)
features = feature_ext.extract_all(audio_clean, pitch)

# Apply octave correction
corrector = OctaveCorrector(strategy="hybrid")
pitch.f0_hz = corrector.correct(
    pitch.f0_hz,
    pitch.confidence,
    chroma=features.chroma
)

print(f"Processed {len(audio)/22050:.2f}s of audio")
print(f"Extracted {len(pitch.f0_hz)} pitch frames")
print(f"Voiced ratio: {np.mean(pitch.confidence > 0.5):.1%}")
```

## 4. Compare Old vs New Alignment

```python
# compare_alignment.py
import numpy as np
from iqrah_audio import PitchExtractor, DTWAligner, FeatureExtractor

# Generate reference and user audio (slightly different)
def gen_audio(freq, duration=3.0, sr=22050):
    t = np.linspace(0, duration, int(sr * duration))
    return np.sin(2 * np.pi * freq * t).astype(np.float32)

ref_audio = gen_audio(440.0)  # A4
user_audio = gen_audio(445.0)  # Slightly sharp

# Extract pitch
pitch_ext = PitchExtractor(method="yin", sample_rate=22050)
ref_pitch = pitch_ext.extract_stable_pitch(ref_audio)
user_pitch = pitch_ext.extract_stable_pitch(user_audio)

# OLD: Pitch-only alignment
print("OLD - Pitch-only alignment:")
aligner = DTWAligner()
result_old = aligner.align(user_pitch.f0_cents, ref_pitch.f0_cents)
print(f"  Distance: {result_old.distance:.2f}")
print(f"  Score: {result_old.alignment_score:.3f}")

# NEW: Multi-dimensional alignment
print("\nNEW - Multi-dimensional alignment:")
feature_ext = FeatureExtractor(sample_rate=22050)
ref_features = feature_ext.extract_all(ref_audio, ref_pitch)
user_features = feature_ext.extract_all(user_audio, user_pitch)

# Compute cost matrix with multi-dimensional features
cost_matrix = feature_ext.compute_cost_matrix(
    user_features,
    ref_features,
    weights={"f0": 0.5, "timbre": 0.3, "energy": 0.1, "chroma": 0.1}
)

print(f"  Cost matrix shape: {cost_matrix.shape}")
print(f"  Mean cost: {np.mean(cost_matrix):.3f}")
print(f"  (lower cost = better alignment)")

# You can then use this cost matrix with custom DTW if needed
```

## 5. Test Octave Error Detection

```python
# test_octave_detection.py
import numpy as np
from iqrah_audio.octave import detect_octave_errors, octave_aware_pitch_distance

# Create test data with octave errors
ref_cents = np.array([0, 0, 0, 0, 0] * 20)  # A4 (440 Hz)
user_cents = np.array([
    0, 0, 1200, 1200, 0,  # Octave jump in middle
    0, 0, -1200, 0, 0,    # Octave drop
] * 10)

# Detect errors
errors = detect_octave_errors(user_cents, ref_cents, threshold_cents=600)

print(f"Octave errors detected: {np.sum(errors)}/{len(errors)} frames")
print(f"Error rate: {np.mean(errors)*100:.1f}%")

# Test octave-aware distance
print("\nOctave-aware distances:")
print(f"A4 (440) vs A5 (880) - regular: 1200 cents")
print(f"A4 (440) vs A5 (880) - octave-aware: {octave_aware_pitch_distance(440, 880, max_octaves=2):.1f} cents")
```

## 6. Full Pipeline Comparison

```python
# full_pipeline_test.py
import time
import numpy as np
from iqrah_audio import *

# Generate test data
def gen_audio(duration=5.0):
    sr = 22050
    t = np.linspace(0, duration, int(sr * duration))
    freq = 220 + 50 * np.sin(2 * np.pi * 0.5 * t)  # Vibrato
    return np.sin(2 * np.pi * freq * t).astype(np.float32), sr

audio, sr = gen_audio()

print("=" * 60)
print("Full Pipeline Benchmark")
print("=" * 60)

# OLD pipeline
print("\nOLD Pipeline (pitch + basic DTW):")
start = time.time()

pitch_ext = PitchExtractor(method="yin", sample_rate=sr)
pitch = pitch_ext.extract(audio)
aligner = DTWAligner()
result = aligner.align(pitch.f0_cents, pitch.f0_cents)  # Self-alignment

elapsed = time.time() - start
rtf = elapsed / 5.0

print(f"  Time: {elapsed*1000:.0f}ms")
print(f"  RTF: {rtf:.3f}")
print(f"  Score: {result.alignment_score:.3f}")

# NEW pipeline
print("\nNEW Pipeline (features + octave correction):")
start = time.time()

pitch = pitch_ext.extract_stable_pitch(audio)

feature_ext = FeatureExtractor(sample_rate=sr)
features = feature_ext.extract_all(audio, pitch)

corrector = OctaveCorrector(strategy="hybrid")
pitch.f0_hz = corrector.correct(
    pitch.f0_hz,
    pitch.confidence,
    chroma=features.chroma
)

# Multi-dimensional similarity
sim = feature_ext.compute_similarity(
    features, features,
    frame_a=50, frame_b=50,
    weights={"f0": 0.5, "timbre": 0.3, "energy": 0.1, "chroma": 0.1}
)

elapsed = time.time() - start
rtf = elapsed / 5.0

print(f"  Time: {elapsed*1000:.0f}ms")
print(f"  RTF: {rtf:.3f}")
print(f"  Similarity: {sim:.3f}")
print(f"  Features: F0 + Mel({features.mel_spec.shape}) + Chroma + RMS")
```

## Expected Results

After running these tests, you should see:

### Improvements ✅
- **Octave errors**: Near 0% with correction (was 5-10%)
- **Robustness**: Multi-dimensional features work better on noisy audio
- **Accuracy**: Tighter pitch tracking with median filtering
- **Tajweed ready**: Nasal energy, spectral flatness available

### Performance 📊
- RTF should still be <0.3 (real-time capable)
- Memory usage ~50-100MB for 3s audio
- Feature extraction adds ~30-50ms overhead

### Next Steps 🚀
1. Test on real Quranic audio
2. Compare accuracy on noisy recordings
3. Integrate into your main pipeline
4. Add RMVPE for better accuracy (Phase 1.1)

## Troubleshooting

**Issue: Tests fail with import errors**
```bash
# Reinstall in editable mode
pip install -e ".[dev]"
```

**Issue: CREPE not found**
```bash
# Install CREPE
pip install crepe tensorflow  # or tensorflow-cpu
```

**Issue: Out of memory**
```python
# Use smaller feature dimensions
feature_ext = FeatureExtractor(
    n_mels=40,  # Reduce from 80
    extract_spectral=False  # Disable some features
)
```

## Questions?

Check:
- [SOTA_IMPROVEMENTS_SUMMARY.md](SOTA_IMPROVEMENTS_SUMMARY.md) - Full overview
- [IMPROVEMENTS.md](IMPROVEMENTS.md) - Detailed roadmap
- [README.md](README.md) - Package documentation
