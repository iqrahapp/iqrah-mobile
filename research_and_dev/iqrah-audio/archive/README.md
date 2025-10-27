# Iqrah Audio - Quranic Recitation Analysis

[![Python](https://img.shields.io/badge/python-3.9+-blue.svg)](https://www.python.org/downloads/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

**Phoneme-level Quranic recitation analysis with Tajweed mapping and multi-dimensional comparison.**

---

## Quick Start

### Installation

```bash
# Clone repository
cd research_and_dev/iqrah-audio

# Create virtual environment (recommended)
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install dependencies
pip install -r requirements.txt
```

**Note:** First-time installation downloads ML models (~2GB). See [INSTALLATION.md](INSTALLATION.md) for detailed setup instructions and troubleshooting.

### Run Web Application

```bash
# Start FastAPI server
python app_qari_final.py

# Open browser
http://localhost:8004/
```

### Features
- **Analyze Qari recitations** with pitch visualization and Tajweed colors
- **Compare user audio** against Husary reference
- **Multi-dimensional scoring**: rhythm, melody, duration, pronunciation

---

## What This System Does

### 1. Recitation Analysis
- **Pitch extraction**: SwiftF0 (fast) or CREPE (accurate)
- **Phoneme alignment**: Wav2Vec2 CTC with word-level windowing
- **Tajweed mapping**: Madd, ghunnah, and other rules from authoritative data
- **Statistics**: Tempo, pitch stability, madd accuracy, duration analysis

### 2. Recitation Comparison
- **Rhythm**: Tempo-invariant comparison using Soft-DTW (Sakoe-Chiba band)
- **Melody**: Key-invariant comparison using ΔF0 (pitch deltas)
- **Duration**: Probabilistic madd scoring (Natural, Connected, Separated, Required)
- **Pronunciation**: Goodness of Pronunciation (GOP) using Wav2Vec2 phoneme quality
- **Overall Score**: Weighted fusion with explainability (0-100 scale)

### 3. Web Interface
- Interactive pitch visualization with real-time cursor
- Arabic text with Tajweed colors (right-to-left)
- Upload user audio for comparison against Husary
- Visualizations: DTW path, pitch contours, spectrograms

---

## Project Structure

```
iqrah-audio/
├── app_qari_final.py              # Main FastAPI application
├── README.md                      # This file
├── ARCHITECTURE.md                # Technical architecture
├── PROJECT_SUMMARY.md             # Comprehensive project overview
│
├── static/                        # Web interface
│   ├── qari_final.html           # Main analysis page
│   └── compare_user.html         # User comparison page
│
├── src/iqrah_audio/              # Source code
│   ├── analysis/                 # Analysis module
│   │   ├── pitch_extractor_swiftf0.py      # Fast pitch extraction
│   │   ├── pitch_extractor_crepe.py        # Accurate pitch extraction
│   │   ├── phoneme_wav2vec2_ctc.py         # PRIMARY phoneme alignment
│   │   ├── tajweed_loader.py               # Tajweed markup
│   │   ├── tajweed_mapper.py               # Phoneme-to-rule mapping
│   │   ├── segments_loader.py              # Word timestamps (6,236 ayahs)
│   │   └── statistics_analyzer.py          # Comprehensive statistics
│   │
│   └── comparison/               # Comparison module
│       ├── engine.py             # Main orchestrator
│       ├── rhythm.py             # Soft-DTW rhythm comparison
│       ├── melody.py             # ΔF0 melody comparison
│       ├── duration.py           # Madd duration scoring
│       ├── pronunciation.py      # SSL-GOP pronunciation scoring
│       ├── fusion.py             # Weighted fusion & explainability
│       ├── features.py           # Feature extraction
│       └── visualization.py      # Comparison visualizations
│
├── data/                         # Data files
│   ├── husary-ayah-segments.json # Word timestamps for all 6,236 ayahs
│   ├── qpc-hafs-tajweed.json     # Arabic text with Tajweed markup
│   ├── quran-phoneme-tajweed.json # Phoneme-level Tajweed rules
│   ├── quran-transliteration-simple.json # Transliteration data
│   └── audio_cache/              # Downloaded Husary audio
│
└── doc/                      # Research documentation
    └── final-target-pipeline-draft.md # State-of-the-art analysis
```

---

## API Endpoints

### Main Application: `app_qari_final.py` (Port 8004)

1. **`GET /`** - Main analysis page ([qari_final.html](static/qari_final.html))

2. **`GET /compare`** - User comparison page ([compare_user.html](static/compare_user.html))

3. **`GET /api/analyze/{surah}/{ayah}?pitch_extractor=swiftf0`**
   - Analyzes Husary recitation
   - Returns: `{pitch, phonemes, arabic_words, statistics, audio_url}`

4. **`GET /audio/{surah}/{ayah}`** - Serves cached audio

5. **`POST /api/compare`**
   - Compares two recitations (surah:ayah pairs)
   - Returns: `{overall, rhythm, melody, duration, pronunciation, feedback}`

6. **`POST /api/compare/visualize`**
   - Same as `/api/compare` with base64 visualizations

7. **`POST /api/compare/user`**
   - Upload user audio, compare against Husary reference
   - Form data: `audio` (file), `surah`, `ayah`, `pitch_extractor`

---

## Python API Usage

### Single Recitation Analysis

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
from src.iqrah_audio.analysis.phoneme_from_transliteration import load_transliteration_data

# Load ayah data
surah, ayah = 1, 1
seg_data = get_ayah_segments(surah, ayah)
audio_path = download_audio(seg_data['audio_url'])

# Extract pitch
pitch_data = extract_pitch_swiftf0(audio_path)

# Extract phonemes with alignment
word_segments = get_word_segments_with_text(surah, ayah)
trans_data = load_transliteration_data()
transliteration = trans_data.get(f'{surah}:{ayah}', '')

phonemes = extract_phonemes_wav2vec2_ctc(
    audio_path=audio_path,
    word_segments=word_segments,
    transliteration=transliteration,
    pitch_data=pitch_data,
    surah=surah,
    ayah=ayah
)

# Compute statistics
statistics = compute_full_statistics(phonemes, pitch_data)

print(f"Tempo: {statistics['tempo']['syllables_per_second']:.2f} syl/s")
print(f"Mean Pitch: {statistics['pitch']['mean_pitch']:.1f} Hz")
print(f"Madd Accuracy: {statistics['madd']['overall_accuracy']:.1f}%")
```

### Recitation Comparison

```python
from src.iqrah_audio.comparison import compare_recitations

# Analyze both recitations first (see above)
# Then compare:
comparison = compare_recitations(
    student_audio_path="student_1_1.mp3",
    reference_audio_path="husary_1_1.mp3",
    student_phonemes=student_phonemes,
    reference_phonemes=reference_phonemes,
    student_pitch=student_pitch,
    reference_pitch=reference_pitch,
    student_stats=student_stats,
    reference_stats=reference_stats
)

print(f"Overall Score: {comparison['overall']}/100")
print(f"  Rhythm: {comparison['rhythm']['score']}/100")
print(f"  Melody: {comparison['melody']['score']}/100")
print(f"  Duration: {comparison['durations']['overall']}/100")
print(f"  Pronunciation: {comparison['pronunciation']['score']}/100")

# Get detailed feedback
for note in comparison['feedback']['all_notes']:
    print(f"  • {note}")
```

---

## Technology Stack

### Core Technologies
- **Audio Processing**: librosa, soundfile, scipy
- **ML Models**:
  - Wav2Vec2 (facebook/mms-1b-all) - Phoneme alignment
  - CREPE (torchcrepe) - Accurate pitch extraction
  - SwiftF0 - Fast pitch extraction (42× faster than CREPE)
- **Web Framework**: FastAPI, Uvicorn
- **Visualization**: Matplotlib (backend), Plotly.js (frontend)

### Algorithms
- **Phoneme Alignment**: CTC forced alignment with word-level windowing
- **Rhythm Comparison**: Soft-DTW with Sakoe-Chiba band (band_radius=0.1)
- **Melody Comparison**: ΔF0 (key-invariant) warped by rhythm DTW path
- **Duration Scoring**: Probabilistic madd scoring with expected duration ± tolerance
- **Pronunciation**: SSL-GOP using Wav2Vec2 phoneme posteriors

---

## Data Sources

### Audio Data
- **Source**: Everyayah.com (Husary recitations)
- **Coverage**: All 6,236 Quranic ayahs
- **Format**: MP3, 44.1kHz
- **Caching**: Automatic download to `data/audio_cache/`

### Tajweed Data
- **Word timestamps**: `data/husary-ayah-segments.json` (6,236 ayahs)
- **Tajweed markup**: `data/qpc-hafs-tajweed.json` (Arabic text with colors)
- **Phoneme rules**: `data/quran-phoneme-tajweed.json` (phoneme-level mappings)
- **Transliteration**: `data/quran-transliteration-simple.json`

---

## Documentation

- **[ARCHITECTURE.md](ARCHITECTURE.md)** - Technical architecture, component details, API reference
- **[PROJECT_SUMMARY.md](PROJECT_SUMMARY.md)** - Comprehensive overview for AI-assisted research
- **[doc/final-target-pipeline-draft.md](doc/final-target-pipeline-draft.md)** - State-of-the-art analysis and research foundation

---

## Project Status

### ✅ Completed (Production-Ready)
- Phoneme-level analysis with Tajweed mapping
- Multi-dimensional comparison engine (rhythm, melody, duration, pronunciation)
- Web interface with visualization
- Support for all 6,236 Quranic ayahs
- User audio upload and comparison

### 🚧 Known Limitations
- No real-time streaming analysis (static file processing only)
- Limited Tajweed rule coverage (madd works well, complex rules need refinement)
- No mobile deployment (models not quantized)
- Single recitation style (Hafs only, via Husary)

### 📋 Future Roadmap
- **Ghunnah detection** using formant analysis (target: 85%+ accuracy)
- **Qalqalah detection** using burst analysis
- **Complex rules** (idghaam, ikhfaa, iqlaab) with acoustic validation
- **Mobile deployment** (INT8 quantization, <100MB model size)
- **Real-time streaming** (<500ms latency for live feedback)
- **Fine-tuned models** (task-adaptive pretraining on Quranic data)

See [PROJECT_SUMMARY.md](PROJECT_SUMMARY.md) for detailed research directions.

---

## Performance

### Current Benchmarks (Desktop CPU)
- **Single recitation analysis**: ~5-10 seconds
- **Comparison (4 components)**: ~5-10 seconds
- **Pitch extraction (SwiftF0)**: ~0.5s (42× faster than CREPE)
- **Phoneme alignment (Wav2Vec2)**: ~1-2s

### Target Performance (with GPU)
- **Total pipeline**: ~1-2s (5-10× speedup)
- **Real-time streaming**: <500ms latency (future goal)

---

## Testing

### Validation Tests (3 files kept)
- `test_user_recitation.py` - Test user vs Husary comparison
- `test_visualization_system.py` - Test visualization generation
- `test_pronunciation_final_validation.py` - Test GOP scoring

```bash
# Run validation tests
python test_user_recitation.py
python test_visualization_system.py
python test_pronunciation_final_validation.py
```

---

## Archive

Historical drafts and experiments are preserved in `archive/`:
- `archive/OLD_apps/` - Previous app versions (app.py, app_qari.py, etc.)
- `archive/OLD_sources/` - Unused/replaced source files (60% of original codebase)
- `archive/OLD_tests/` - Obsolete test scripts
- `archive/OLD_html/` - Previous HTML interface versions
- `archive/OLD_docs/` - Draft documentation

**These files are kept for reference only and are not part of the current production system.**

---

## Credits

**Research Foundation:**
- Meta AI - Wav2Vec2 & MMS models
- Google Research - CREPE pitch tracker
- Montreal Forced Aligner - Alignment concepts
- Everyayah.com - Husary recitation audio

**Key Papers:**
- AraS2P (2025) - Phoneme-aware Wav2Vec2-BERT (0.16% PER)
- Automatic Pronunciation Error Detection (2025) - Quran Phonetic Script
- Madd Detection (2024) - Rule-based duration algorithm (99.87% accuracy)
- Ghunnah Detection (2020) - Formant analysis + MLP (71-85% accuracy)

See [doc/final-target-pipeline-draft.md](doc/final-target-pipeline-draft.md) for comprehensive research analysis.

---

## License

MIT License - See LICENSE file for details.

---

**Built for the Iqrah Quran Learning App**
**Version:** 1.0 (Static Analysis Prototype)
**Last Updated:** 2025-10-23
