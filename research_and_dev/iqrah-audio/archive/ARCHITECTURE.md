# Iqrah Audio - System Architecture

**Version:** 1.0 (2025-10-23)
**Status:** Production-ready static analysis system

---

## Table of Contents

1. [System Overview](#system-overview)
2. [Core Components](#core-components)
3. [Analysis Pipeline](#analysis-pipeline)
4. [Comparison Engine](#comparison-engine)
5. [Data Sources](#data-sources)
6. [API Endpoints](#api-endpoints)
7. [Technology Stack](#technology-stack)
8. [Future Roadmap](#future-roadmap)

---

## System Overview

Iqrah Audio is a Quranic recitation analysis and comparison system designed to provide:

- **Phoneme-level recitation analysis** with Tajweed rule detection
- **Multi-dimensional comparison** between student and reference recitations
- **Visual feedback** through interactive web interfaces
- **Pronunciation scoring** using Goodness of Pronunciation (GOP) metrics

### Key Features

✅ **Analysis Features:**
- Pitch extraction (SwiftF0 & CREPE)
- Phoneme alignment (Wav2Vec2 CTC & MMS-FA)
- Tajweed rule mapping
- Statistical analysis (tempo, rhythm, count, madd)

✅ **Comparison Features:**
- Rhythm comparison (Soft-DTW with Sakoe-Chiba band)
- Melody comparison (Key-invariant ΔF0)
- Duration analysis (Probabilistic madd scoring)
- Pronunciation assessment (SSL-GOP)
- Weighted fusion scoring

✅ **Visualization:**
- Real-time playback cursor
- Arabic text with Tajweed colors (RTL)
- Pitch contour overlay
- Phoneme segmentation
- Comparison visualizations (DTW path, spectrograms, etc.)

---

## Core Components

### 1. Analysis Module (`src/iqrah_audio/analysis/`)

#### **Pitch Extraction**
- [`pitch_extractor_swiftf0.py`](src/iqrah_audio/analysis/pitch_extractor_swiftf0.py)
  - Primary pitch extractor (42× faster than CREPE)
  - Uses SwiftF0 model for real-time performance
  - Returns: `{pitch_hz, times, confidence, duration}`

- [`pitch_extractor_crepe.py`](src/iqrah_audio/analysis/pitch_extractor_crepe.py)
  - High-accuracy pitch extraction using CREPE neural network
  - Two modes: `extract_pitch_crepe_fast()`, `extract_pitch_crepe_accurate()`
  - Trade-off: Slower but more accurate for challenging recordings

#### **Phoneme Alignment**
- [`phoneme_wav2vec2_ctc.py`](src/iqrah_audio/analysis/phoneme_wav2vec2_ctc.py) ⭐ **PRIMARY**
  - Uses Wav2Vec2 with CTC forced alignment
  - Model: `facebook/mms-1b-all` (MMS: Massively Multilingual Speech)
  - Windowed alignment within word boundaries
  - Integrates with Tajweed mapper for rule detection
  - Returns: `[{phoneme, start, end, word_num, confidence, tajweed_rule, ...}]`

- [`phoneme_mms_proper.py`](src/iqrah_audio/analysis/phoneme_mms_proper.py)
  - Alternative alignment using MMS Forced Aligner
  - Falls back to simple alignment if needed
  - Used for validation and robustness

- [`phoneme_simple.py`](src/iqrah_audio/analysis/phoneme_simple.py)
  - Fallback: Proportional distribution of phonemes within word segments
  - Based on gold transliteration data

#### **Tajweed Integration**
- [`tajweed_loader.py`](src/iqrah_audio/analysis/tajweed_loader.py)
  - Loads Arabic text with Tajweed markup from `qpc-hafs-tajweed.json`
  - Parses HTML-style Tajweed classes (madd, ghunnah, etc.)
  - Returns color-coded segments for visualization

- [`tajweed_mapper.py`](src/iqrah_audio/analysis/tajweed_mapper.py)
  - Maps phonemes to Tajweed rules using authoritative data
  - Sources: `quran-phoneme-tajweed.json`, `quran-transliteration-tajweed.json`
  - Identifies madd types, ghunnah, qalqalah, etc.

#### **Data Loading**
- [`segments_loader.py`](src/iqrah_audio/analysis/segments_loader.py)
  - Loads word-level timestamp data from `husary-ayah-segments.json`
  - Downloads and caches Husary recitation audio
  - Supports all 6,236 Quranic ayahs

- [`phoneme_from_transliteration.py`](src/iqrah_audio/analysis/phoneme_from_transliteration.py)
  - Loads gold transliteration data from `quran-transliteration-simple.json`
  - Used as reference for alignment

#### **Statistics**
- [`statistics_analyzer.py`](src/iqrah_audio/analysis/statistics_analyzer.py)
  - Computes comprehensive statistics:
    - **Tempo:** syllables/second, stability score
    - **Pitch:** mean, std, range, contour smoothness
    - **Count:** duration per word, precision score
    - **Madd:** duration accuracy, type detection
    - **Rhythm:** onset strength, syllable masks

---

### 2. Comparison Module (`src/iqrah_audio/comparison/`)

#### **Comparison Engine**
- [`engine.py`](src/iqrah_audio/comparison/engine.py)
  - Main orchestrator for multi-dimensional comparison
  - Calls individual component scorers
  - Function: `compare_recitations(student_audio, reference_audio, ...)`
  - Returns: Overall score + component breakdowns

#### **Component Scorers**

1. **Rhythm** ([`rhythm.py`](src/iqrah_audio/comparison/rhythm.py))
   - Tempo-invariant comparison using Soft-DTW
   - Sakoe-Chiba band constraint (band_radius=0.1)
   - Onset-dominant features for better alignment
   - Score: 0-100 (based on DTW divergence normalized by path length)

2. **Melody** ([`melody.py`](src/iqrah_audio/comparison/melody.py))
   - Key-invariant comparison using ΔF0 (pitch deltas)
   - Warps melody using rhythm DTW path
   - Tighter band (0.05) for per-step divergence
   - Score: 0-100 (average per-step similarity)

3. **Duration** ([`duration.py`](src/iqrah_audio/comparison/duration.py))
   - Phoneme-level duration comparison
   - Probabilistic madd scoring (expected duration ± tolerance)
   - Madd types: Natural (2h), Connected (4-5h), Separated (3-5h), Required (6h)
   - Score: Weighted average of phoneme duration matches

4. **Pronunciation** ([`pronunciation.py`](src/iqrah_audio/comparison/pronunciation.py))
   - Goodness of Pronunciation (GOP) using Wav2Vec2
   - Compares student GOP vs reference GOP (delta scoring)
   - Phoneme-by-phoneme feedback
   - Score: Based on GOP delta distribution

#### **Fusion & Features**
- [`fusion.py`](src/iqrah_audio/comparison/fusion.py)
  - Weighted combination of component scores
  - Explainability: Textual feedback per component
  - Default weights: `{rhythm: 0.3, melody: 0.3, duration: 0.2, pronunciation: 0.2}`

- [`features.py`](src/iqrah_audio/comparison/features.py)
  - Extract normalized feature vectors (FeaturePack)
  - Includes: pitch_hz, onset_strength, spectral_flux, rms_energy, syllable_mask, voiced_mask
  - Multi-feature stacking for DTW alignment

#### **Visualization**
- [`visualization.py`](src/iqrah_audio/comparison/visualization.py)
  - Generates base64-encoded PNG images for web display
  - Visualizations:
    - DTW path with alignment
    - Pitch contour comparison
    - Rhythm alignment
    - Spectrograms
  - Returns: `{dtw_path, pitch_comparison, rhythm_alignment, ...}`

---

## Analysis Pipeline

### Single Recitation Analysis Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                     INPUT: Audio File (MP3/WAV/WebM)            │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
         ┌───────────────────────────────────────┐
         │   1. Load Word Segments & Metadata    │
         │      - segments_loader.py             │
         │      - tajweed_loader.py              │
         │      - transliteration data           │
         └──────────────┬────────────────────────┘
                        │
                        ▼
         ┌───────────────────────────────────────┐
         │   2. Extract Pitch Contour            │
         │      - SwiftF0 (default, fast)        │
         │      - CREPE (optional, accurate)     │
         └──────────────┬────────────────────────┘
                        │
                        ▼
         ┌───────────────────────────────────────┐
         │   3. Phoneme Alignment                │
         │      - Wav2Vec2 CTC (primary)         │
         │      - Windowed by word segments      │
         │      - Tajweed rule mapping           │
         └──────────────┬────────────────────────┘
                        │
                        ▼
         ┌───────────────────────────────────────┐
         │   4. Compute Statistics               │
         │      - Tempo, pitch, rhythm           │
         │      - Madd accuracy                  │
         │      - Duration per phoneme           │
         └──────────────┬────────────────────────┘
                        │
                        ▼
┌────────────────────────────────────────────────────────────────┐
│  OUTPUT: {pitch, phonemes, arabic_words, statistics}           │
└────────────────────────────────────────────────────────────────┘
```

### Comparison Pipeline Flow

```
┌────────────────────────────────────────────────────────────────┐
│           INPUTS: Student Audio + Reference Audio              │
└──────────────┬──────────────────────────┬──────────────────────┘
               │                          │
               ▼                          ▼
    ┌──────────────────┐      ┌──────────────────┐
    │  Analyze Student │      │ Analyze Reference│
    │   (Pipeline #1)  │      │   (Pipeline #2)  │
    └────────┬─────────┘      └─────────┬────────┘
             │                          │
             └────────────┬─────────────┘
                          │
                          ▼
              ┌────────────────────────┐
              │  Extract Features      │
              │  - FeaturePack x2      │
              └───────────┬────────────┘
                          │
                          ▼
      ┌──────────────────────────────────────────┐
      │     Run Component Comparisons            │
      │  ┌────────────────────────────────────┐  │
      │  │  1. Rhythm (Soft-DTW)              │  │
      │  │  2. Melody (ΔF0 + warping)         │  │
      │  │  3. Duration (Phoneme-level)       │  │
      │  │  4. Pronunciation (GOP delta)      │  │
      │  └────────────────────────────────────┘  │
      └───────────────────┬──────────────────────┘
                          │
                          ▼
              ┌────────────────────────┐
              │   Fusion & Scoring     │
              │   - Weighted average   │
              │   - Explainability     │
              └───────────┬────────────┘
                          │
                          ▼
              ┌────────────────────────┐
              │  Generate Visualizations│
              │   (Optional)            │
              └───────────┬────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│  OUTPUT: {overall: X/100, rhythm: {...}, melody: {...}, ...}   │
└─────────────────────────────────────────────────────────────────┘
```

---

## Comparison Engine

### Scoring Components

| Component         | Weight | Algorithm                       | Key Features                             |
| ----------------- | ------ | ------------------------------- | ---------------------------------------- |
| **Rhythm**        | 30%    | Soft-DTW (Sakoe-Chiba band=0.1) | Tempo-invariant, onset-dominant features |
| **Melody**        | 30%    | ΔF0 warped by rhythm path       | Key-invariant, per-step divergence       |
| **Duration**      | 20%    | Probabilistic madd scoring      | Expected duration ± tolerance            |
| **Pronunciation** | 20%    | SSL-GOP delta (Wav2Vec2)        | Phoneme-by-phoneme quality               |

### Scoring Formula

```
Overall Score =
    0.3 × Rhythm Score +
    0.3 × Melody Score +
    0.2 × Duration Score +
    0.2 × Pronunciation Score
```

**Score Interpretation:**
- **90-100:** Excellent (Qari-level)
- **75-89:** Very Good
- **60-74:** Good
- **40-59:** Needs Improvement
- **0-39:** Significant Issues

---

## Data Sources

### 1. Audio Data
- **Source:** Everyayah.com (Husary recitations)
- **Format:** MP3, 44.1kHz
- **Coverage:** All 6,236 ayahs (114 surahs)
- **Storage:** `data/husary/surahs/{surah}/{ayah}.mp3`
- **Caching:** Automatic download and cache in `data/audio_cache/`

### 2. Word Segmentation
- **File:** `data/husary-ayah-segments.json`
- **Format:** `{surah:ayah: {audio_url, start, end, words: [{text, start, end}]}}`
- **Coverage:** 6,236 ayahs with word-level timestamps

### 3. Tajweed Markup
- **File:** `data/qpc-hafs-tajweed.json`
- **Format:** Arabic text with HTML-style Tajweed classes
- **Classes:** `madd_munfasil`, `madd_muttasil`, `ghunnah`, `idgham`, etc.

### 4. Phoneme-Level Tajweed
- **File:** `data/quran-phoneme-tajweed.json`
- **Format:** Phoneme-to-Tajweed rule mapping
- **Coverage:** All ayahs with phoneme-level rule labels

### 5. Transliteration
- **File:** `data/quran-transliteration-simple.json`
- **Format:** `{surah:ayah: "transliteration"}`
- **System:** Simplified Arabic transliteration (e.g., "bismi-llaahi")

---

## API Endpoints

### Main Application: `app_qari_final.py`

**Server:** FastAPI on port 8004
**Frontend:** Static HTML files in `static/`

#### **Endpoints:**

1. **`GET /`**
   - Serves main analysis page ([qari_final.html](static/qari_final.html))
   - Features: Qari recitation analysis, pitch visualization, Tajweed display

2. **`GET /compare`**
   - Serves user comparison page ([compare_user.html](static/compare_user.html))
   - Features: Upload user audio, compare vs Husary

3. **`GET /api/analyze/{surah}/{ayah}?pitch_extractor=swiftf0`**
   - Analyzes Husary recitation for given ayah
   - Returns: `{pitch, phonemes, arabic_words, statistics, audio_url}`

4. **`GET /audio/{surah}/{ayah}`**
   - Serves cached audio file with range request support

5. **`POST /api/compare`**
   - Compares two recitations (surah:ayah pairs)
   - Body: `{student_surah, student_ayah, reference_surah, reference_ayah, pitch_extractor}`
   - Returns: `{overall, rhythm, melody, duration, pronunciation, ...}`

6. **`POST /api/compare/visualize`**
   - Same as `/api/compare` but includes base64 visualizations
   - Returns: `{comparison, visualizations: {dtw_path, pitch_comparison, ...}}`

7. **`POST /api/compare/user`**
   - Compares user-uploaded audio vs Husary reference (same ayah)
   - Form data: `audio` (file), `surah`, `ayah`, `pitch_extractor`
   - Returns: Full comparison with visualizations

---

## Technology Stack

### Backend
- **Framework:** FastAPI 0.100+
- **Python:** 3.9+
- **Audio Processing:** librosa, soundfile, scipy
- **ML Models:**
  - Wav2Vec2 (facebook/mms-1b-all)
  - CREPE (torchcrepe)
  - SwiftF0 (swift-f0)
- **Alignment:** ctc-forced-aligner, Montreal Forced Aligner concepts

### Frontend
- **HTML/CSS/JavaScript** (Vanilla JS, no framework)
- **Visualization:** Plotly.js for interactive charts
- **Audio:** HTML5 Audio API

### Data
- **Format:** JSON
- **Size:** ~100MB (includes audio cache)

### Deployment
- **Server:** Uvicorn ASGI server
- **Port:** 8004 (default)
- **CORS:** Enabled for local development

---

## Future Roadmap

Based on [doc/final-target-pipeline-draft.md](doc/final-target-pipeline-draft.md):

### Phase 1: Production Refinement (Current)
✅ **Completed:**
- Phoneme-level analysis with Tajweed mapping
- Multi-dimensional comparison engine
- Web-based visualization
- GOP-based pronunciation scoring

⏳ **In Progress:**
- Fine-tune madd duration thresholds
- Expand Tajweed rule coverage (qalqalah, idghaam, ikhfaa)
- Improve alignment accuracy for non-Hafs recitations

### Phase 2: Advanced Features (Next 3-6 months)
- **Mobile deployment** (quantized models, on-device inference)
- **Real-time analysis** (WebSocket streaming, ~500ms latency)
- **Pedagogical feedback** (actionable corrections, not just scores)
- **Multi-reciter support** (different qira'at styles)

### Phase 3: Production Scaling (6-12 months)
- **Fine-tuned models** on Quranic data (Wav2Vec2-BERT)
- **Progressive rule rollout** (madd → ghunnah → complex rules)
- **B2B features** (Islamic schools, teacher dashboards)
- **Cost optimization** (self-hosted inference, caching)

---

## Project Goals

### Primary Objective
Build a **phoneme-level Tajweed correction system** that:
1. Achieves **90%+ accuracy** on basic rules (madd, ghunnah)
2. Provides **sub-500ms latency** for mobile deployment
3. Delivers **actionable feedback** for serious students

### Market Differentiation
- **vs Tarteel.ai:** We offer phoneme-level pronunciation correction (they do word-level mistake detection)
- **vs TajweedMate:** Validated accuracy with state-of-the-art models
- **Target Users:** Serious students, Islamic schools, teachers (premium tier)

### Technical Milestones
- [x] Phoneme-level alignment (Wav2Vec2 CTC)
- [x] Tajweed rule mapping
- [x] Multi-dimensional comparison (rhythm, melody, duration, pronunciation)
- [x] Web visualization
- [ ] Mobile deployment (quantized models)
- [ ] Real-time streaming analysis
- [ ] Fine-tuned Quranic models (Wav2Vec2-BERT)

---
