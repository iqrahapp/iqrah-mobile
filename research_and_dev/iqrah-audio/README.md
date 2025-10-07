# Iqrah Audio - Qur'an Recitation Analysis

[![Python](https://img.shields.io/badge/python-3.9+-blue.svg)](https://www.python.org/downloads/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

**State-of-the-Art pitch tracking and real-time streaming analysis for comparing Qur'anic recitations to reference Qari.**

## ⚡ Quick Start

### Web UI (Recommended)
```bash
# Install web dependencies
pip install -r requirements-web.txt

# Start web server
python app.py

# Open browser
open http://localhost:8000
```

### Command Line Demo
```bash
# Try the real-time demo (self-test mode)
python demo_realtime.py

# Analyze user recitation
python demo_realtime.py --user path/to/recitation.mp3
```

See [UI_GUIDE.md](UI_GUIDE.md) and [DEMO_GUIDE.md](DEMO_GUIDE.md) for detailed usage.

## 🎯 Performance Highlights

- **Ultra-Low Latency**: 3-7ms end-to-end processing
- **Real-Time Capable**: <100ms target achieved (30x better)
- **High Accuracy**: 86.1/100 SOTA score
- **Production Ready**: Optimized for live coaching applications

## Features

### ✅ Phase 1: Phoneme-Level Analysis (Complete)
- **Wav2Vec2 CTC Alignment**: Forced phoneme alignment with word boundaries
- **Advanced Pitch Tracking**: SwiftF0 (ONNX) + CREPE fallback
- **Comprehensive Statistics**:
  - Tempo (syllables/second)
  - Mean pitch (Hz)
  - Phoneme duration (mean count)
  - Detailed per-phoneme timing
- **Tajweed Integration**: 83,668 words with Tajweed rules
- **Mobile-Ready Format**: CBOR + zstd compression

### ✅ Phase 2: Comparison Engine (Complete)
- **Rhythm Analysis**: Soft-DTW divergence for tempo-invariant comparison (0-100)
- **Melody Analysis**: ΔF0 contour matching for key-invariant melody (0-100)
- **Duration Analysis**: Tempo-adaptive Madd (elongation) scoring with Laplace (0-100)
- **Overall Score**: Weighted combination (Rhythm 40%, Melody 25%, Duration 35%)
- **REST API**: `/api/compare` endpoint for HTTP comparison
- **Comprehensive Feedback**: Detailed notes and improvement suggestions
- **Test Coverage**:
  - Self-comparison: 100/100 ✅
  - Different ayahs: 40-45/100 ✅

### ✅ Real-Time Streaming (Complete)
- **Real-Time Pipeline**: <10ms latency per chunk
- **Incremental Pitch Extraction**: Optimized vectorized YIN (3-5ms)
- **Online DTW**: Streaming alignment with drift correction (1-2ms)
- **Anchor Detection**: Silence, plosives, long notes
- **Live Feedback**: 15 Hz coaching hints with visual cues
- **Performance Monitoring**: Comprehensive latency tracking

### ✅ Web UI (Complete)
- **FastAPI Backend**: REST API + WebSocket streaming
- **Real-Time Visualization**: Live waveform and feedback display
- **Interactive Interface**: Microphone capture, file upload, visual cues
- **Performance Dashboard**: Latency, confidence, frame tracking
- **Production Ready**: Docker, Nginx, multi-worker support

### 📋 Future Enhancements (Phase 2.5+)
- SSL-GOP pronunciation scoring
- RMVPE pitch extraction (more robust than CREPE)
- HPCP/chroma fallback for melody
- FAISS ANN for multi-reference comparison
- DTW path visualization
- Real-time streaming comparison
- Progress tracking across sessions

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

### Phase 1: Analysis API
```python
from src.iqrah_audio.analysis import (
    extract_pitch_swiftf0,
    extract_phonemes_wav2vec2_ctc,
    compute_full_statistics
)
from src.iqrah_audio.analysis.segments_loader import (
    get_ayah_segments,
    download_audio,
    get_word_segments_with_text
)

# Load ayah audio
surah, ayah = 1, 1
seg_data = get_ayah_segments(surah, ayah)
audio_path = download_audio(seg_data['audio_url'])

# Extract pitch
pitch_data = extract_pitch_swiftf0(audio_path)

# Extract phonemes with alignment
word_segments = get_word_segments_with_text(surah, ayah)
phonemes = extract_phonemes_wav2vec2_ctc(
    audio_path=audio_path,
    word_segments=word_segments,
    transliteration="BismillaahirRahmaanirRaheem",
    pitch_data=pitch_data,
    surah=surah,
    ayah=ayah
)

# Compute statistics
statistics = compute_full_statistics(phonemes, pitch_data)

print(f"Tempo: {statistics['tempo']:.2f} syl/s")
print(f"Mean Pitch: {statistics['mean_pitch']:.1f} Hz")
print(f"Mean Count: {statistics['mean_count']:.3f}s")
```

### Phase 2: Comparison API
```python
from src.iqrah_audio.comparison import compare_recitations

# Compare student vs reference
comparison = compare_recitations(
    student_audio_path="student_1_1.mp3",
    reference_audio_path="husary_1_1.mp3",
    student_phonemes=student_phonemes,
    reference_phonemes=reference_phonemes,
    student_pitch=student_pitch_data,
    reference_pitch=reference_pitch_data,
    student_stats=student_stats,
    reference_stats=reference_stats
)

print(f"Overall Score: {comparison['overall']:.1f}/100")
print(f"Rhythm: {comparison['rhythm']['score']:.1f}/100")
print(f"Melody: {comparison['melody']['score']:.1f}/100")
print(f"Duration: {comparison['durations']['overall']:.1f}/100")

# Get feedback
for note in comparison['feedback']['all_notes']:
    print(f"  • {note}")
```

See [docs/comparison-api.md](docs/comparison-api.md) for complete API documentation.

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

### ✅ Phase 1: Phoneme-Level Analysis (Complete)
- [x] Wav2Vec2 CTC forced alignment
- [x] SwiftF0/CREPE pitch extraction
- [x] Comprehensive statistics (tempo, pitch, duration)
- [x] Tajweed rule integration
- [x] CBOR serialization

### ✅ Phase 2: Comparison Engine (Complete)
- [x] Soft-DTW rhythm analysis
- [x] ΔF0 melody analysis
- [x] Tempo-adaptive Madd scoring
- [x] Component fusion with feedback
- [x] REST API integration
- [x] Comprehensive testing

### 🔄 Phase 2.5: Enhancement (Planned)
- [ ] SSL-GOP pronunciation scoring
- [ ] RMVPE pitch extraction
- [ ] HPCP/chroma melody fallback
- [ ] DTW path visualization
- [ ] Streaming comparison

### Phase 3: Production (Next)
- [ ] FAISS ANN for multi-reference
- [ ] Real-time streaming comparison
- [ ] Progress tracking dashboard
- [ ] Mobile SDK (Rust/Flutter)
- [ ] Cloud deployment

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
