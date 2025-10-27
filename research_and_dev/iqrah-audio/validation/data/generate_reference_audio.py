#!/usr/bin/env python3
"""
Generate reference audio files for validation and benchmarking.

This script creates synthetic audio files with known properties
to test each module's functionality and performance.
"""

import json
from pathlib import Path

import numpy as np
import soundfile as sf


def generate_speech_like_audio(duration: float, sample_rate: int = 16000) -> np.ndarray:
    """
    Generate speech-like audio with multiple formants and varying pitch.

    Args:
        duration: Duration in seconds
        sample_rate: Sample rate in Hz

    Returns:
        Audio array
    """
    t = np.linspace(0, duration, int(sample_rate * duration))

    # Varying fundamental frequency (simulates pitch variation in speech)
    f0 = 150 + 50 * np.sin(2 * np.pi * 2 * t)

    # First formant (vowel-like)
    f1 = 700 + 200 * np.sin(2 * np.pi * 3 * t)

    # Second formant
    f2 = 1220 + 300 * np.sin(2 * np.pi * 1.5 * t)

    # Third formant
    f3 = 2600

    # Combine formants
    audio = (0.4 * np.sin(2 * np.pi * f0 * t) +
             0.3 * np.sin(2 * np.pi * f1 * t) +
             0.2 * np.sin(2 * np.pi * f2 * t) +
             0.1 * np.sin(2 * np.pi * f3 * t))

    # Add amplitude envelope (speech-like rhythm)
    envelope = 0.3 + 0.7 * (0.5 + 0.5 * np.sin(2 * np.pi * 4 * t))
    audio = audio * envelope

    # Add small pauses to simulate word boundaries
    for i in range(int(duration)):
        pause_start = int((i + 0.8) * sample_rate)
        pause_end = int((i + 0.95) * sample_rate)
        if pause_end < len(audio):
            audio[pause_start:pause_end] *= np.linspace(1.0, 0.1, pause_end - pause_start)

    # Normalize
    audio = audio / np.abs(audio).max() * 0.9

    return audio


def main():
    """Generate all reference audio files."""
    output_dir = Path(__file__).parent
    output_dir.mkdir(exist_ok=True)

    metadata = {
        "files": {},
        "description": "Reference audio files for Iqrah Audio validation",
        "generated_with": "validation/data/generate_reference_audio.py",
    }

    print("Generating reference audio files...")

    # 1. Clean 30s audio for M1 benchmarking
    print("  - clean_30s.wav (for M1 performance benchmark)")
    audio_30s = generate_speech_like_audio(duration=30.0, sample_rate=16000)
    output_file = output_dir / "clean_30s.wav"
    sf.write(output_file, audio_30s, 16000)

    metadata["files"]["clean_30s.wav"] = {
        "duration": 30.0,
        "sample_rate": 16000,
        "purpose": "M1 performance benchmark",
        "expected_properties": {
            "sample_rate": 16000,
            "duration": 30.0,
            "quality_flag": "excellent",
            "min_snr_db": 15.0,
        },
    }

    # 2. Short clean audio for quick tests
    print("  - clean_5s.wav (for quick validation tests)")
    audio_5s = generate_speech_like_audio(duration=5.0, sample_rate=16000)
    output_file = output_dir / "clean_5s.wav"
    sf.write(output_file, audio_5s, 16000)

    metadata["files"]["clean_5s.wav"] = {
        "duration": 5.0,
        "sample_rate": 16000,
        "purpose": "Quick validation tests",
        "expected_properties": {
            "sample_rate": 16000,
            "duration": 5.0,
            "quality_flag": "excellent",
        },
    }

    # 3. Noisy audio for quality checks
    print("  - noisy_10s.wav (for quality metric validation)")
    audio_10s = generate_speech_like_audio(duration=10.0, sample_rate=16000)
    noise = 0.15 * np.random.randn(len(audio_10s))
    audio_noisy = audio_10s + noise
    audio_noisy = audio_noisy / np.abs(audio_noisy).max() * 0.9
    output_file = output_dir / "noisy_10s.wav"
    sf.write(output_file, audio_noisy, 16000)

    metadata["files"]["noisy_10s.wav"] = {
        "duration": 10.0,
        "sample_rate": 16000,
        "purpose": "Quality metric validation",
        "expected_properties": {
            "sample_rate": 16000,
            "duration": 10.0,
            "quality_flag": "good",
            "max_snr_db": 15.0,
        },
    }

    # 4. High sample rate audio for resampling tests
    print("  - high_sr_3s.wav (for resampling validation)")
    audio_3s_48k = generate_speech_like_audio(duration=3.0, sample_rate=48000)
    output_file = output_dir / "high_sr_3s.wav"
    sf.write(output_file, audio_3s_48k, 48000)

    metadata["files"]["high_sr_3s.wav"] = {
        "duration": 3.0,
        "sample_rate": 48000,
        "purpose": "Resampling validation",
        "expected_properties": {
            "sample_rate": 16000,  # After resampling
            "duration": 3.0,
        },
    }

    # Save metadata
    metadata_file = output_dir / "metadata.json"
    with open(metadata_file, "w") as f:
        json.dump(metadata, f, indent=2)

    print(f"\n✅ Generated {len(metadata['files'])} reference audio files")
    print(f"📊 Metadata saved to: {metadata_file}")


if __name__ == "__main__":
    main()
