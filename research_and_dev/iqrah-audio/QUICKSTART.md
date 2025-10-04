# Iqrah Audio - Quick Start Guide

Get up and running in 5 minutes!

## Step 1: Install

```bash
cd research_and_dev/iqrah-audio

# Create virtual environment
python3 -m venv venv
source venv/bin/activate  # Windows: venv\Scripts\activate

# Install package
pip install -e ".[dev]"
```

## Step 2: Run Demo

```bash
# Run the demo script (uses synthetic audio)
python examples/demo.py
```

This will:
1. Generate synthetic qari audio
2. Process it to CBOR format
3. Generate synthetic user audio (with pitch errors)
4. Analyze and score the recitation

Output:
```
RECITATION ANALYSIS RESULTS
============================================================

  Overall Score:     85.3/100
  Alignment Score:   88.7/100
  On-Note %:         82.4%
  Pitch Stability:   91.2/100
  Tempo Score:       76.5/100
  Voiced Ratio:      87%
```

## Step 3: Use CLI (with real audio)

### Process reference qari audio:

```bash
iqrah-audio process-reference \
    path/to/qari_audio.wav \
    output/reference.cbor.zst \
    --metadata '{"ayah": "1:1", "qari": "husary"}'
```

### Analyze your recitation:

```bash
iqrah-audio analyze \
    my_recitation.wav \
    output/reference.cbor.zst \
    --output-json results.json
```

### Batch process directory:

```bash
iqrah-audio batch-process \
    qari_audio_directory/ \
    output/ \
    --pattern "*.wav"
```

## Step 4: Use Python API

```python
from iqrah_audio import (
    PitchExtractor,
    DTWAligner,
    RecitationScorer,
    ReferenceProcessor
)
import soundfile as sf

# Load audio
user_audio, sr = sf.read("my_recitation.wav")

# Extract pitch
extractor = PitchExtractor(method="yin")  # or "crepe"
user_contour = extractor.extract_stable_pitch(user_audio, sr=sr)

# Load reference
processor = ReferenceProcessor()
ref_contour = processor.get_contour_from_cbor("reference.cbor.zst")

# Align & Score
aligner = DTWAligner()
scorer = RecitationScorer()

alignment = aligner.align(user_contour.f0_cents, ref_contour.f0_cents)
score = scorer.score(user_contour, ref_contour, alignment)

print(f"Score: {score.overall_score:.1f}/100")
```

## Step 5: Run Tests

```bash
# Run test suite
pytest tests/ -v

# With coverage
pytest tests/ -v --cov=iqrah_audio --cov-report=html

# View coverage report
open htmlcov/index.html
```

## Features Overview

### Available Methods

| Method | Speed | Accuracy | Best For |
|--------|-------|----------|----------|
| YIN | ⚡⚡⚡ Fast | Good | Real-time, testing |
| CREPE | ⚡ Slower | Excellent | Offline, noisy audio |

### Scoring Metrics

- **Overall Score** (0-100): Weighted combination of all metrics
- **Alignment Score**: DTW similarity to reference
- **On-Note %**: Percentage within ±50 cents
- **Pitch Stability**: Steadiness measurement
- **Tempo Score**: Speed matching

### File Formats

**Input**: `.wav`, `.mp3`, `.flac` (any format supported by `soundfile`)
**Output**: `.cbor.zst` (compressed binary format for mobile)

## Common Issues

### Issue: CREPE not found
```bash
pip install crepe
```

### Issue: "No module named 'iqrah_audio'"
```bash
# Make sure you're in the right directory
cd research_and_dev/iqrah-audio

# Reinstall in editable mode
pip install -e .
```

### Issue: Audio file not loading
```bash
# Install audio codecs
pip install soundfile librosa

# On Ubuntu/Debian:
sudo apt-get install libsndfile1
```

## Next Steps

1. **Get Real Qari Audio**
   - Download from Tarteel, EveryAyah, or other sources
   - Process with `batch-process` command

2. **Integrate with Iqrah App**
   - Use `.cbor.zst` files as app assets
   - Load in Rust backend using `ciborium` crate
   - See Sprint 8 plan in main docs

3. **Customize Scoring**
   - Adjust `on_note_threshold_cents` for stricter/looser matching
   - Modify weights in `RecitationScorer.score()` method
   - Add custom metrics in `scorer.py`

## Performance Tips

- Use YIN for real-time (15ms latency)
- Use CREPE for offline/noisy (180ms, but more accurate)
- Apply denoising for recordings in noisy environments
- Compress references with zstd (70% size reduction)

## Help & Support

```bash
# CLI help
iqrah-audio --help
iqrah-audio analyze --help

# View reference file
iqrah-audio inspect reference.cbor.zst
```

---

**Ready to build SOTA recitation analysis! 🎯**
