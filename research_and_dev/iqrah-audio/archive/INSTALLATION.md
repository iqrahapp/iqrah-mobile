# Installation Guide

## Quick Start (Recommended)

```bash
# 1. Install dependencies
pip install -r requirements.txt

# 2. Run the application
python app_qari_final.py

# 3. Open browser
http://localhost:8004/
```

---

## Detailed Installation

### Prerequisites

- Python 3.9 or higher
- pip package manager
- ~5GB disk space for models and data

### Step 1: Install Core Dependencies

```bash
# Update pip
pip install --upgrade pip

# Install requirements
pip install -r requirements.txt
```

### Step 2: Verify Installation

```bash
# Test imports
python -c "import torch; import torchaudio; import transformers; print('✓ Core dependencies OK')"
python -c "import librosa; import numpy; import scipy; print('✓ Audio processing OK')"
python -c "import fastapi; import uvicorn; print('✓ Web framework OK')"
```

### Step 3: Optional - Install SwiftF0 (Faster Pitch Extraction)

SwiftF0 is 42× faster than CREPE but may not be available on all platforms.

```bash
# Try to install swift-f0 (optional)
pip install swift-f0
```

If `swift-f0` fails to install, the app will automatically fall back to CREPE (which is slower but works on all platforms).

---

## Dependency Overview

### Required Dependencies

| Package | Purpose | Size |
|---------|---------|------|
| torch | PyTorch for ML models | ~800MB |
| torchaudio | Audio processing for PyTorch | ~10MB |
| transformers | Wav2Vec2 for phoneme alignment | ~100MB |
| torchcrepe | CREPE pitch extraction | ~5MB |
| librosa | Audio analysis | ~50MB |
| fastapi | Web framework | ~10MB |
| uvicorn | ASGI server | ~5MB |

### Optional Dependencies

| Package | Purpose | Fallback |
|---------|---------|----------|
| swift-f0 | Fast pitch extraction (42× faster) | CREPE (slower but accurate) |

---

## Troubleshooting

### Issue: `ModuleNotFoundError: No module named 'torch'`

**Solution:**
```bash
pip install torch torchaudio --index-url https://download.pytorch.org/whl/cpu
```

Or with CUDA support (for GPU acceleration):
```bash
pip install torch torchaudio --index-url https://download.pytorch.org/whl/cu118
```

### Issue: `ModuleNotFoundError: No module named 'transformers'`

**Solution:**
```bash
pip install transformers
```

### Issue: `ModuleNotFoundError: No module named 'torchcrepe'`

**Solution:**
```bash
pip install torchcrepe
```

### Issue: `swift-f0` installation fails

**This is OK!** The app will automatically use CREPE as a fallback. You'll see this warning:
```
⚠️  WARNING: swift-f0 not installed
   Using CREPE as fallback (slower but accurate)
```

No action needed - the app will work fine with CREPE.

### Issue: Models downloading on first run

**This is normal!** The first time you run the app, it will download:
- Wav2Vec2 model (~1.5GB)
- CREPE model (~30MB)

These are cached locally and won't be downloaded again.

---

## Platform-Specific Notes

### Linux

All dependencies should install cleanly with pip.

### macOS

PyTorch CPU version is recommended unless you have an M1/M2 Mac:

```bash
# For M1/M2 Mac with MPS support
pip install torch torchaudio

# For Intel Mac
pip install torch torchaudio --index-url https://download.pytorch.org/whl/cpu
```

### Windows

Use the CPU version of PyTorch:

```bash
pip install torch torchaudio --index-url https://download.pytorch.org/whl/cpu
```

Or with CUDA (if you have an NVIDIA GPU):
```bash
pip install torch torchaudio --index-url https://download.pytorch.org/whl/cu118
```

---

## Verifying the Installation

### Test the Application

```bash
# Start the server
python app_qari_final.py
```

You should see:
```
======================================================================
🎯 Qari Tajweed Analysis - FINAL PROPER VERSION
======================================================================

Implementing AI Report 2's approach:
  ✅ Word-level segments (6,236 ayahs)
  ✅ MMS-FA with windowing
  ✅ Proper phoneme alignment
  ✅ Audio downloading
  ✅ Real-time cursor (dot on pitch)
  ✅ Arabic words + Tajweed colors
  ✅ RTL X-axis

⚠️  WARNING: swift-f0 not installed (optional)
   Using CREPE as fallback (slower but accurate)

Starting on http://0.0.0.0:8004
======================================================================

INFO:     Started server process
INFO:     Waiting for application startup.
INFO:     Application startup complete.
INFO:     Uvicorn running on http://0.0.0.0:8004
```

### Test the Web Interface

1. Open http://localhost:8004/ in your browser
2. Select "Surah 1, Ayah 1"
3. Click "Analyze"
4. You should see pitch visualization and phoneme segmentation

---

## Minimal Installation (For Testing Only)

If you want to test without installing all dependencies:

```bash
# Minimal requirements (web framework only)
pip install fastapi uvicorn python-multipart

# Add audio processing
pip install numpy scipy librosa soundfile

# Add ML models (required for analysis)
pip install torch torchaudio transformers torchcrepe scikit-learn
```

Note: This won't include optional dependencies like swift-f0.

---

## Development Installation

For development with testing and linting tools:

```bash
# Install with development dependencies
pip install -e ".[dev]"

# Or manually:
pip install pytest pytest-cov black ruff mypy
```

---

## Docker Installation (Advanced)

If you prefer Docker:

```bash
# Build the container
docker build -t iqrah-audio .

# Run the container
docker run -p 8004:8004 iqrah-audio
```

(Note: Dockerfile not included yet, but can be added if needed)

---

## Getting Help

If you encounter issues:

1. Check this guide for common solutions
2. Verify Python version: `python --version` (should be 3.9+)
3. Verify pip version: `pip --version` (should be 20.0+)
4. Try creating a fresh virtual environment:
   ```bash
   python -m venv venv
   source venv/bin/activate  # On Windows: venv\Scripts\activate
   pip install -r requirements.txt
   ```

For persistent issues, please check the GitHub issues or create a new one with:
- Your Python version
- Your OS/platform
- The full error message
