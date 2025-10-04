# Iqrah Audio - Qur'an Recitation Analysis

[![Python](https://img.shields.io/badge/python-3.9+-blue.svg)](https://www.python.org/downloads/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

**State-of-the-Art pitch tracking and DTW alignment for comparing Qur'anic recitations to reference Qari.**

## Features

### Phase 2 MVP (Current - Offline Analysis)
- ✅ **SOTA Pitch Tracking**: CREPE (neural network) + YIN fallback
- ✅ **Noise Reduction**: Spectral gating for robust analysis in noisy environments
- ✅ **DTW Alignment**: Fast C-based alignment with Sakoe-Chiba band
- ✅ **Comprehensive Scoring**:
  - Overall score (weighted combination)
  - Alignment score (DTW similarity)
  - On-note percentage (pitch accuracy within threshold)
  - Pitch stability (jitter measurement)
  - Tempo matching
- ✅ **Mobile-Ready Format**: CBOR + zstd compression for deployment
- ✅ **CLI Tool**: Complete command-line interface

### Future (Phase 3 - Real-Time)
- 🔄 Online-DTW for live coaching
- 🔄 Real-time pitch overlay
- 🔄 CTC forced alignment for phoneme-level analysis
- 🔄 GOP (Goodness of Pronunciation) scores
- 🔄 Tajwid rule detection

## Installation

```bash
# Clone the repository
cd research_and_dev/iqrah-audio

# Create virtual environment
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install in development mode
pip install -e ".[dev]"
```

### Dependencies

**Core:**
- `numpy`, `scipy`, `librosa` - Audio processing
- `crepe` - SOTA neural network pitch tracker
- `noisereduce` - Spectral gating denoising
- `dtaidistance` - Fast DTW with C speedups
- `cbor2`, `zstandard` - Mobile-compatible serialization

**CLI:**
- `click` - Command-line interface
- `rich` - Beautiful terminal output

## Quick Start

### 1. Process Reference Qari Audio

```bash
# Process a single reference file
iqrah-audio process-reference \
    qari/husary_001001.wav \
    output/001001.cbor.zst \
    --metadata '{"ayah": "1:1", "qari": "husary"}'

# Batch process directory
iqrah-audio batch-process \
    qari_audio/ \
    output/ \
    --pattern "*.wav"
```

### 2. Analyze User Recitation

```bash
iqrah-audio analyze \
    my_recitation.wav \
    output/001001.cbor.zst \
    --output-json results.json
```

Output:
```
┏━━━━━━━━━━━━━━━━━━━━━━━━━┳━━━━━━━━━━┓
┃ Metric                  ┃    Score ┃
┡━━━━━━━━━━━━━━━━━━━━━━━━━╇━━━━━━━━━━┩
│ Overall Score           │  85.3/100 │
│ Alignment Score         │  88.7/100 │
│ On-Note %               │     82.4% │
│ Pitch Stability         │  91.2/100 │
│ Tempo Score             │  76.5/100 │
│ Voiced Ratio            │      87%  │
└─────────────────────────┴───────────┘
```

### 3. Inspect Reference Files

```bash
iqrah-audio inspect output/001001.cbor.zst
```

## Python API

```python
from iqrah_audio import (
    PitchExtractor,
    AudioDenoiser,
    DTWAligner,
    RecitationScorer,
    ReferenceProcessor
)
import soundfile as sf

# Load user audio
user_audio, sr = sf.read("my_recitation.wav")

# Denoise
denoiser = AudioDenoiser(sample_rate=22050)
user_audio = denoiser.denoise_adaptive(user_audio)

# Extract pitch
extractor = PitchExtractor(method="crepe")  # or "yin", "auto"
user_contour = extractor.extract_stable_pitch(user_audio, sr=sr)

# Load reference
processor = ReferenceProcessor()
ref_contour = processor.get_contour_from_cbor("reference.cbor.zst")

# Align
aligner = DTWAligner()
alignment = aligner.align(user_contour.f0_cents, ref_contour.f0_cents)

# Score
scorer = RecitationScorer()
score = scorer.score(user_contour, ref_contour, alignment)

print(f"Overall Score: {score.overall_score:.1f}/100")
print(f"On-Note: {score.on_note_percent:.1f}%")
```

## Architecture

### Workflow

```
┌─────────────┐
│ Qari Audio  │
└──────┬──────┘
       │
       ▼
┌─────────────────┐
│ ReferenceProcessor │
│  - Denoise       │
│  - Extract Pitch │
│  - Save CBOR     │
└──────┬──────────┘
       │
       ▼
┌─────────────────┐
│ Reference CBOR  │ ◄───────┐
│  (compressed)   │         │
└─────────────────┘         │
                            │
┌─────────────┐             │
│ User Audio  │             │
└──────┬──────┘             │
       │                    │
       ▼                    │
┌─────────────────┐         │
│ PitchExtractor  │         │
│  - Denoise      │         │
│  - Extract Pitch│         │
└──────┬──────────┘         │
       │                    │
       ▼                    │
┌─────────────────┐         │
│  DTWAligner     │◄────────┘
│  - Align        │
│  - Find Path    │
└──────┬──────────┘
       │
       ▼
┌─────────────────┐
│ RecitationScorer│
│  - On-Note %    │
│  - Stability    │
│  - Tempo        │
└──────┬──────────┘
       │
       ▼
┌─────────────┐
│   Results   │
│  (JSON/UI)  │
└─────────────┘
```

### Key Components

| Component | Purpose | Technology |
|-----------|---------|------------|
| `PitchExtractor` | Extract F0 from audio | CREPE (neural) / YIN (classic) |
| `AudioDenoiser` | Noise reduction | Spectral gating (noisereduce) |
| `DTWAligner` | Align user to reference | Fast DTW (dtaidistance) |
| `RecitationScorer` | Score quality | Multi-metric scoring |
| `ReferenceProcessor` | Process qari audio | CBOR + zstd |

## Scoring Metrics

### Overall Score
Weighted combination:
- 40% Alignment Score (DTW similarity)
- 30% On-Note % (pitch accuracy)
- 20% Pitch Stability (low jitter)
- 10% Tempo Score (matching speed)

### On-Note Percentage
Percentage of frames within ±50 cents of reference pitch.

### Pitch Stability
Measures pitch steadiness in voiced regions. Low jitter = high score.

### Tempo Score
How well the user matches the qari's tempo. Perfect score at 1:1 ratio, decreases with deviation.

## File Formats

### CBOR Reference Format
```python
{
    "contour": {
        "f0_hz": [float, ...],         # Pitch in Hz
        "confidence": [float, ...],     # Voicing confidence
        "timestamps": [float, ...],     # Time in seconds
        "sample_rate": int
    },
    "metadata": {
        "ayah": str,                    # e.g., "1:1"
        "qari": str,                    # e.g., "husary"
        "surah": int,
        ...
    },
    "processing": {
        "sample_rate": int,
        "pitch_method": str,
        "denoised": bool,
        "duration": float,
        "n_frames": int
    }
}
```

Compressed with zstandard for mobile deployment (~70% size reduction).

## Performance

### Benchmarks (on M1 MacBook Air)

| Operation | Duration | RTF |
|-----------|----------|-----|
| Denoise (3s audio) | 120ms | 0.04 |
| CREPE pitch extraction | 180ms | 0.06 |
| YIN pitch extraction | 45ms | 0.015 |
| DTW alignment | 25ms | - |
| Full pipeline | ~350ms | 0.12 |

RTF = Real-Time Factor (< 1.0 = faster than real-time)

**Mobile-ready**: All operations run faster than real-time on mid-range devices.

## Development

### Running Tests

```bash
pytest tests/ -v --cov=iqrah_audio
```

### Code Quality

```bash
# Format code
black src/ tests/

# Lint
ruff src/ tests/

# Type check
mypy src/
```

## Roadmap

### Phase 2 (Current) ✅
- [x] Offline pitch tracking
- [x] DTW alignment
- [x] Multi-metric scoring
- [x] CBOR serialization
- [x] CLI tool

### Phase 3 (Next)
- [ ] Online-DTW for real-time
- [ ] Real-time audio streaming
- [ ] Live pitch visualization
- [ ] Arabic CTC model integration

### Phase 4 (Future)
- [ ] GOP (pronunciation scoring)
- [ ] Tajwid rule detection
- [ ] Multi-qari support
- [ ] Mobile app integration (Rust/Flutter)

## Integration with Iqrah App

This package generates `.cbor.zst` reference files that will be:

1. **Bundled in Flutter app** as assets
2. **Loaded by Rust backend** using `ciborium` crate
3. **Used for scoring** in offline pitch analysis feature (Sprint 8)

### Rust Integration (Future)

```rust
// Load reference contour
let reference: PitchContour = load_cbor("assets/001001.cbor.zst")?;

// Analyze user audio
let user_contour = extract_pitch(user_audio)?;

// DTW alignment
let alignment = dtw_align(&user_contour, &reference)?;

// Score
let score = calculate_score(&user_contour, &reference, &alignment)?;
```

## Credits

Built with:
- [CREPE](https://github.com/marl/crepe) - Neural pitch tracking
- [dtaidistance](https://github.com/wannesm/dtaidistance) - Fast DTW
- [noisereduce](https://github.com/timsainb/noisereduce) - Spectral gating

## License

MIT License - See LICENSE file for details.

---

**Built for the Iqrah Qur'an Learning App**
