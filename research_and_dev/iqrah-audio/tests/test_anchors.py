#!/usr/bin/env python3
"""
Test Anchor Detection
=====================

Test anchor detection on real Quranic audio (Husary Al-Fatiha).
"""

import numpy as np
import soundfile as sf
from pathlib import Path

print("=" * 80)
print("ANCHOR DETECTION TEST - Husary Al-Fatiha")
print("=" * 80)

# Load Husary audio
audio_path = Path("media/husary/01.mp3")

if not audio_path.exists():
    print(f"\n✗ Audio file not found: {audio_path}")
    print("  Please ensure media/husary/01.mp3exists")
    exit(1)

print(f"\nLoading audio: {audio_path}")
audio, sr = sf.read(str(audio_path))

# Convert to mono if stereo
if len(audio.shape) > 1:
    audio = audio.mean(axis=1)

audio = audio.astype(np.float32)
duration = len(audio) / sr

print(f"✓ Loaded: {duration:.2f}s @ {sr} Hz")

# Extract features for anchor detection
print("\nExtracting features...")

from iqrah_audio import PitchExtractor
from iqrah_audio.features import FeatureExtractor

# Extract pitch
pitch_ext = PitchExtractor(method="yin", sample_rate=sr)
pitch = pitch_ext.extract_stable_pitch(audio)

print(f"✓ Pitch extracted: {len(pitch.f0_hz)} frames")

# Extract multi-dimensional features
feat_ext = FeatureExtractor(
    sample_rate=sr,
    extract_chroma=False,  # Not needed for anchors
    extract_energy=True,
    extract_spectral=True,
)

features = feat_ext.extract_all(audio, pitch)

print(f"✓ Features extracted")
print(f"  RMS energy: {features.rms.shape}")
print(f"  Spectral centroid: {features.spectral_centroid.shape}")
print(f"  Spectral flatness: {features.spectral_flatness.shape}")

# Detect anchors
print("\n" + "=" * 80)
print("DETECTING ANCHORS")
print("=" * 80)

from iqrah_audio.streaming import AnchorDetector

detector = AnchorDetector(
    sample_rate=sr,
    silence_threshold_db=-40.0,
    silence_min_duration_s=0.2,
    long_note_min_duration_s=0.5,
)

# Detect each type
print("\n1. Silence Detection")
print("-" * 80)
silence_anchors = detector.detect_silence(
    features.rms,
    pitch.timestamps,
)
print(f"Found {len(silence_anchors)} silence anchors:")
for i, anchor in enumerate(silence_anchors[:10]):  # Show first 10
    print(f"  {i+1}. {anchor}")
if len(silence_anchors) > 10:
    print(f"  ... and {len(silence_anchors) - 10} more")

print("\n2. Plosive Detection")
print("-" * 80)
plosive_anchors = detector.detect_plosives(
    features.spectral_flatness,
    features.rms,
    pitch.timestamps,
)
print(f"Found {len(plosive_anchors)} plosive anchors:")
for i, anchor in enumerate(plosive_anchors[:10]):
    print(f"  {i+1}. {anchor}")
if len(plosive_anchors) > 10:
    print(f"  ... and {len(plosive_anchors) - 10} more")

print("\n3. Long Note Detection")
print("-" * 80)
long_note_anchors = detector.detect_long_notes(
    pitch.f0_hz,
    pitch.confidence,
    pitch.timestamps,
)
print(f"Found {len(long_note_anchors)} long note anchors:")
for i, anchor in enumerate(long_note_anchors[:10]):
    print(f"  {i+1}. {anchor}")
if len(long_note_anchors) > 10:
    print(f"  ... and {len(long_note_anchors) - 10} more")

print("\n4. All Anchors Combined")
print("-" * 80)
all_anchors = detector.detect_all(
    pitch.f0_hz,
    pitch.confidence,
    features.rms,
    features.spectral_flatness,
    pitch.timestamps,
)
print(f"Total anchors: {len(all_anchors)}")

# Count by type
silence_count = sum(1 for a in all_anchors if a.anchor_type == "silence")
plosive_count = sum(1 for a in all_anchors if a.anchor_type == "plosive")
long_note_count = sum(1 for a in all_anchors if a.anchor_type == "long_note")

print(f"  Silence: {silence_count}")
print(f"  Plosives: {plosive_count}")
print(f"  Long notes: {long_note_count}")

print("\nAll anchors (sorted by time):")
for i, anchor in enumerate(all_anchors[:20]):  # Show first 20
    print(f"  {i+1}. {anchor}")
if len(all_anchors) > 20:
    print(f"  ... and {len(all_anchors) - 20} more")

# Filter anchors
print("\n5. Filtered Anchors (conf >= 0.7)")
print("-" * 80)
filtered = detector.filter_anchors(all_anchors, min_confidence=0.7)
print(f"High-confidence anchors: {len(filtered)}")
for i, anchor in enumerate(filtered[:15]):
    print(f"  {i+1}. {anchor}")

# Statistics
print("\n" + "=" * 80)
print("STATISTICS")
print("=" * 80)

avg_confidence = np.mean([a.confidence for a in all_anchors])
avg_spacing = np.mean(np.diff([a.timestamp for a in all_anchors])) if len(all_anchors) > 1 else 0

print(f"\nTotal duration: {duration:.2f}s")
print(f"Total anchors: {len(all_anchors)}")
print(f"Anchor density: {len(all_anchors) / duration:.2f} anchors/second")
print(f"Average confidence: {avg_confidence:.2f}")
print(f"Average spacing: {avg_spacing:.2f}s")

print("\nConfidence distribution:")
confidences = [a.confidence for a in all_anchors]
print(f"  Min: {np.min(confidences):.2f}")
print(f"  25%: {np.percentile(confidences, 25):.2f}")
print(f"  50%: {np.percentile(confidences, 50):.2f}")
print(f"  75%: {np.percentile(confidences, 75):.2f}")
print(f"  Max: {np.max(confidences):.2f}")

# Success criteria
print("\n" + "=" * 80)
print("VALIDATION")
print("=" * 80)

checks = []

# Check 1: Found anchors
if len(all_anchors) > 0:
    print("✓ Found anchors")
    checks.append(True)
else:
    print("✗ No anchors found")
    checks.append(False)

# Check 2: Reasonable density (expect ~1-5 anchors per second)
density = len(all_anchors) / duration
if 0.5 < density < 10:
    print(f"✓ Anchor density reasonable: {density:.2f}/s")
    checks.append(True)
else:
    print(f"⚠ Anchor density unusual: {density:.2f}/s")
    checks.append(False)

# Check 3: Found multiple types
types_found = len(set(a.anchor_type for a in all_anchors))
if types_found >= 2:
    print(f"✓ Found {types_found} anchor types")
    checks.append(True)
else:
    print(f"⚠ Only found {types_found} anchor type(s)")
    checks.append(False)

# Check 4: Average confidence reasonable
if avg_confidence > 0.3:
    print(f"✓ Average confidence good: {avg_confidence:.2f}")
    checks.append(True)
else:
    print(f"⚠ Low average confidence: {avg_confidence:.2f}")
    checks.append(False)

if all(checks):
    print("\n✅ All validation checks PASSED")
else:
    print(f"\n⚠ {sum(checks)}/{len(checks)} validation checks passed")

print("\n" + "=" * 80)
print("Test completed!")
print("=" * 80)
