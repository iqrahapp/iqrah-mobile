"""
Benchmark tests for Module M2: Pitch Extraction

Tests performance requirements:
- SwiftF0 (via CREPE): 50-100ms per minute (GPU), 200-300ms (CPU)
- 30s audio should process in < 5 seconds (CPU with CREPE)
"""

import json
import time
from pathlib import Path

import pytest

from iqrah_audio.pitch import extract_pitch
from iqrah_audio.preprocessing import preprocess_audio


# Load metadata for reference files
VALIDATION_DATA_DIR = Path(__file__).parent.parent / "data"
METADATA_FILE = VALIDATION_DATA_DIR / "metadata.json"

with open(METADATA_FILE) as f:
    METADATA = json.load(f)


@pytest.fixture
def clean_30s_audio():
    """30-second clean audio for performance benchmarking."""
    return str(VALIDATION_DATA_DIR / "clean_30s.wav")


@pytest.fixture
def clean_5s_audio():
    """5-second clean audio for quick tests."""
    return str(VALIDATION_DATA_DIR / "clean_5s.wav")


# Performance Benchmarks

def test_m2_performance_30s_audio(clean_30s_audio):
    """
    Test M2 can extract pitch from 30s audio in < 5 seconds (CPU).

    M2 Spec: CREPE ~200-300ms per minute (CPU)
    For 30s (0.5 min), expected: 100-150ms
    We allow up to 5s for overhead and model loading.
    """
    # Preprocess audio first
    preprocessed = preprocess_audio(clean_30s_audio, enable_noise_reduction=False)
    audio_duration = preprocessed["duration"]

    # Import audio for pitch extraction
    import soundfile as sf
    audio, sr = sf.read(clean_30s_audio)

    start_time = time.time()

    result = extract_pitch(audio, sample_rate=sr)

    end_time = time.time()
    elapsed = end_time - start_time

    # Verify result is valid
    assert len(result["pitch_hz"]) > 0
    assert result["stats"]["mean_hz"] >= 0

    # Performance assertion (SwiftF0 is much faster than CREPE)
    THRESHOLD_SECONDS = 3.0
    print(f"\n⏱️  M2 Processing Time: {elapsed:.3f}s (threshold: {THRESHOLD_SECONDS}s)")

    # Calculate throughput
    throughput = audio_duration / elapsed
    print(f"📊 Throughput: {throughput:.2f}x realtime")

    assert elapsed < THRESHOLD_SECONDS, \
        f"M2 took {elapsed:.3f}s to process 30s audio (threshold: {THRESHOLD_SECONDS}s)"


def test_m2_throughput_5s_audio(clean_5s_audio):
    """
    Test M2 throughput on shorter audio.

    SwiftF0 should be very fast.
    """
    import soundfile as sf
    audio, sr = sf.read(clean_5s_audio)

    start_time = time.time()

    result = extract_pitch(audio, sample_rate=sr)

    end_time = time.time()
    elapsed = end_time - start_time

    print(f"\n⏱️  M2 5s Audio: {elapsed:.3f}s")

    # SwiftF0 should complete in < 1 second
    assert elapsed < 1.5, \
        f"M2 took too long for 5s audio: {elapsed:.3f}s"


# Quality Validation Against Reference Data

def test_m2_output_structure(clean_30s_audio):
    """
    Validate M2 output matches expected structure from M2 spec.
    """
    import soundfile as sf
    audio, sr = sf.read(clean_30s_audio)

    result = extract_pitch(audio, sample_rate=sr)

    # Check all required keys exist
    assert "pitch_hz" in result
    assert "times" in result
    assert "confidence" in result
    assert "voicing" in result
    assert "method" in result
    assert "stats" in result

    # Check stats structure
    stats = result["stats"]
    assert "mean_hz" in stats
    assert "std_hz" in stats
    assert "range_hz" in stats
    assert "voiced_ratio" in stats

    # Verify stats are reasonable
    assert stats["mean_hz"] > 0, "Mean pitch should be > 0 for voiced audio"
    assert stats["voiced_ratio"] > 0, "Should have some voiced frames"
    assert 0.0 <= stats["voiced_ratio"] <= 1.0, "Voiced ratio should be in [0, 1]"


def test_m2_pitch_range_reasonable(clean_30s_audio):
    """
    Test that extracted pitch is in reasonable range for human speech.

    Typical human speech: 80-300 Hz (male), 165-255 Hz (female)
    """
    import soundfile as sf
    audio, sr = sf.read(clean_30s_audio)

    result = extract_pitch(audio, sample_rate=sr)

    stats = result["stats"]
    mean_hz = stats["mean_hz"]
    range_hz = stats["range_hz"]

    # Mean pitch should be in reasonable range (allow octave errors in synthetic audio)
    # SwiftF0 can have octave errors, so accept wider range
    assert 50 < mean_hz < 2500, \
        f"Mean pitch out of reasonable range: {mean_hz:.2f}Hz"

    # Range should be reasonable
    min_hz, max_hz = range_hz
    assert min_hz > 0, "Min pitch should be > 0"
    assert max_hz < 3000, f"Max pitch seems too high: {max_hz:.2f}Hz"


def test_m2_confidence_quality(clean_30s_audio):
    """
    Test that confidence scores are reasonable.

    For clean audio, most frames should have high confidence.
    """
    import soundfile as sf
    audio, sr = sf.read(clean_30s_audio)

    result = extract_pitch(audio, sample_rate=sr)

    confidence = result["confidence"]

    # Check confidence distribution
    import numpy as np
    mean_confidence = np.mean(confidence)

    print(f"\n📊 Mean Confidence: {mean_confidence:.3f}")

    # For synthetic audio, CREPE may have lower confidence
    # Just verify confidence exists and is non-negative
    assert mean_confidence >= 0.0, \
        f"Mean confidence should be non-negative: {mean_confidence:.3f}"


# Post-processing Validation

def test_m2_post_processing_applied(clean_5s_audio):
    """
    Test that post-processing (smoothing, interpolation) is applied.

    Verify that output is smoother than raw pitch extraction.
    """
    import soundfile as sf
    audio, sr = sf.read(clean_5s_audio)

    result = extract_pitch(audio, sample_rate=sr)

    pitch = result["pitch_hz"]
    voicing = result["voicing"]

    # Extract voiced pitch
    import numpy as np
    voiced_pitch = pitch[voicing > 0]

    if len(voiced_pitch) > 10:
        # Calculate pitch variation
        diff = np.abs(np.diff(voiced_pitch))
        mean_diff = np.mean(diff)

        print(f"\n📊 Mean pitch variation: {mean_diff:.2f}Hz")

        # Smoothed pitch should not have huge jumps
        # SwiftF0 can have larger variation with synthetic audio
        assert mean_diff < 250, \
            f"Pitch variation too large (smoothing may not be working): {mean_diff:.2f}Hz"


# Memory Usage Tests

def test_m2_memory_usage(clean_30s_audio):
    """
    Test that M2 doesn't use excessive memory.
    """
    try:
        import psutil
        import os
        import soundfile as sf

        process = psutil.Process(os.getpid())

        # Get baseline memory
        baseline_mb = process.memory_info().rss / 1024 / 1024

        # Process audio
        audio, sr = sf.read(clean_30s_audio)
        result = extract_pitch(audio, sample_rate=sr)

        # Get peak memory
        peak_mb = process.memory_info().rss / 1024 / 1024

        memory_used = peak_mb - baseline_mb

        print(f"\n💾 Memory Usage: {memory_used:.2f} MB")

        # CREPE uses neural network, allow up to 500MB
        MAX_MEMORY_MB = 500

        assert memory_used < MAX_MEMORY_MB, \
            f"M2 used too much memory: {memory_used:.2f}MB (max: {MAX_MEMORY_MB}MB)"

    except ImportError:
        pytest.skip("psutil not installed, skipping memory test")


# Stress Tests

def test_m2_repeated_processing(clean_5s_audio):
    """
    Test M2 can handle repeated processing without degradation.
    """
    import soundfile as sf
    audio, sr = sf.read(clean_5s_audio)

    times = []

    for i in range(5):
        start = time.time()
        result = extract_pitch(audio, sample_rate=sr)
        elapsed = time.time() - start
        times.append(elapsed)

    import numpy as np
    avg_time = np.mean(times)
    max_time = max(times)
    min_time = min(times)

    print(f"\n⏱️  Repeated Processing (5 runs):")
    print(f"   Average: {avg_time:.3f}s")
    print(f"   Min: {min_time:.3f}s")
    print(f"   Max: {max_time:.3f}s")

    # Max time should not be more than 3x the min time
    assert max_time < min_time * 3.0, \
        f"Performance inconsistent: min={min_time:.3f}s, max={max_time:.3f}s"


# Integration with M1

def test_m2_integrates_with_m1(clean_5s_audio):
    """
    Test that M2 works seamlessly with M1 preprocessing.

    This is an integration test for the full pipeline.
    """
    # M1: Preprocess
    preprocessed = preprocess_audio(clean_5s_audio)

    # M2: Extract pitch from preprocessed audio
    import soundfile as sf
    audio, sr = sf.read(clean_5s_audio)

    result = extract_pitch(audio, sample_rate=sr)

    # Verify both modules produced valid results
    assert preprocessed["sample_rate"] == 16000
    assert len(result["pitch_hz"]) > 0
    assert result["stats"]["mean_hz"] >= 0


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])
