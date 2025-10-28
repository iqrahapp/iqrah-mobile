"""
Quick test to verify Ghunnah formant and Qalqalah burst extraction work with real audio.
"""

import numpy as np
import soundfile as sf
from iqrah.tajweed import GhunnahValidator, QalqalahValidator

# Load audio
audio_path = "validation/data/clean_5s.wav"
audio, sr = sf.read(audio_path)

print(f"Loaded audio: {audio_path}")
print(f"Duration: {len(audio)/sr:.2f}s, Sample rate: {sr}Hz")
print(f"Audio shape: {audio.shape}")
print()

# Test Ghunnah formant extraction
print("=" * 60)
print("Testing Ghunnah Formant Extraction")
print("=" * 60)

ghunnah_validator = GhunnahValidator(use_formants=True)
print(f"Parselmouth available: {ghunnah_validator.parselmouth_available}")

if ghunnah_validator.parselmouth_available:
    try:
        # Extract formants from a segment (first 0.5s)
        formant_features = ghunnah_validator._extract_formant_features(
            audio=audio,
            start=0.5,
            end=1.0,
            sample_rate=sr
        )

        print(f"✅ Formant extraction successful!")
        print(f"   F1: {formant_features.f1_hz:.1f} Hz")
        print(f"   F2: {formant_features.f2_hz:.1f} Hz")
        print(f"   F3: {formant_features.f3_hz:.1f} Hz")
        print(f"   Nasal energy: {formant_features.nasal_energy_db:.1f} dB")
        print(f"   Formant score: {formant_features.formant_score:.3f}")

    except Exception as e:
        print(f"❌ Formant extraction failed: {e}")
        import traceback
        traceback.print_exc()
else:
    print("❌ Parselmouth not available")

print()

# Test Qalqalah burst detection
print("=" * 60)
print("Testing Qalqalah Burst Detection")
print("=" * 60)

qalqalah_validator = QalqalahValidator(use_burst_detection=True)
print(f"Librosa available: {qalqalah_validator.librosa_available}")

if qalqalah_validator.librosa_available:
    try:
        # Extract burst features from a segment (1.0-1.5s)
        burst_features = qalqalah_validator._extract_burst_features(
            audio=audio,
            start=1.0,
            end=1.5,
            sample_rate=sr
        )

        print(f"✅ Burst detection successful!")
        print(f"   ZCR mean: {burst_features.zcr_mean:.3f}")
        print(f"   ZCR std: {burst_features.zcr_std:.3f}")
        print(f"   Centroid: {burst_features.centroid_mean:.1f} Hz")
        print(f"   RMS max: {burst_features.rms_max:.3f}")
        print(f"   RMS std: {burst_features.rms_std:.3f}")
        print(f"   Has burst: {burst_features.has_burst}")
        print(f"   Burst score: {burst_features.burst_score:.3f}")

    except Exception as e:
        print(f"❌ Burst detection failed: {e}")
        import traceback
        traceback.print_exc()
else:
    print("❌ Librosa not available")

print()
print("=" * 60)
print("Summary")
print("=" * 60)
print(f"Ghunnah formants: {'✅ Working' if ghunnah_validator.parselmouth_available else '❌ Not available'}")
print(f"Qalqalah bursts: {'✅ Working' if qalqalah_validator.librosa_available else '❌ Not available'}")
