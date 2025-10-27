"""
Pitch Extraction for Offline Analysis
======================================

Extract pitch using SwiftF0 (42× faster, better accuracy!)
Falls back to CREPE if SwiftF0 unavailable.
"""

import numpy as np
from typing import Dict, List

# Try SwiftF0 first (preferred)
try:
    from .pitch_extractor_swiftf0 import extract_pitch_from_file as _extract_swiftf0
    from .pitch_extractor_swiftf0 import extract_pitch_from_url as _extract_url_swiftf0
    USE_SWIFTF0 = True
    print("✓ Using SwiftF0 for pitch extraction (42× faster, 91.80% accuracy)")
except ImportError:
    USE_SWIFTF0 = False
    print("⚠ SwiftF0 not available, falling back to CREPE")
    import crepe
    import librosa


def extract_pitch_from_file(audio_path: str, sr: int = 16000) -> Dict:
    """
    Extract pitch from audio file using best available method.

    Args:
        audio_path: Path to audio file
        sr: Target sample rate (16000 recommended)

    Returns:
        Dictionary with:
            - time: Time stamps (seconds)
            - f0_hz: Fundamental frequency (Hz)
            - confidence: Confidence scores
            - voiced: Boolean mask of voiced frames
    """

    if USE_SWIFTF0:
        return _extract_swiftf0(audio_path, sr)

    # Fallback: CREPE
    import librosa
    import crepe

    # Load audio
    audio, _ = librosa.load(audio_path, sr=sr, mono=True)

    # Extract pitch with CREPE (best quality settings)
    time, frequency, confidence, activation = crepe.predict(
        audio,
        sr,
        viterbi=True,      # Use Viterbi decoding for smoother pitch
        step_size=10,      # 10ms steps = high time resolution
        model_capacity='full'  # Best quality model
    )

    # Filter out unvoiced frames (low confidence)
    voiced = confidence > 0.5

    return {
        'time': time.tolist(),
        'f0_hz': frequency.tolist(),
        'confidence': confidence.tolist(),
        'voiced': voiced.tolist(),
        'sample_rate': sr,
        'duration': float(time[-1]) if len(time) > 0 else 0.0
    }


def extract_pitch_from_url(audio_url: str) -> Dict:
    """
    Download audio from URL and extract pitch.

    Args:
        audio_url: URL to audio file (e.g., Tarteel CDN)

    Returns:
        Pitch data dictionary
    """

    if USE_SWIFTF0:
        return _extract_url_swiftf0(audio_url)

    # Fallback: CREPE
    import urllib.request
    import tempfile
    from pathlib import Path

    # Download to temporary file
    temp_path = Path(tempfile.gettempdir()) / f"audio_{hash(audio_url)}.mp3"

    if not temp_path.exists():
        print(f"Downloading audio from: {audio_url}")
        urllib.request.urlretrieve(audio_url, temp_path)
    else:
        print(f"Using cached audio: {temp_path}")

    # Extract pitch
    return extract_pitch_from_file(str(temp_path))


def normalize_pitch_range(pitch_data: List[float]) -> List[float]:
    """
    Normalize pitch to compare melody shape (not absolute pitch).

    Converts to log scale and mean-centers.

    Args:
        pitch_data: List of f0 values in Hz

    Returns:
        Normalized pitch values
    """
    pitch = np.array(pitch_data)

    # Only use voiced frames
    voiced = pitch > 0

    if not np.any(voiced):
        return pitch.tolist()

    # Log scale (semitones)
    pitch_log = np.zeros_like(pitch)
    pitch_log[voiced] = np.log2(pitch[voiced])

    # Mean-center (removes absolute pitch difference)
    mean = np.mean(pitch_log[voiced])
    pitch_log[voiced] -= mean

    return pitch_log.tolist()


def calculate_pitch_stats(pitch_data: List[float]) -> Dict:
    """
    Calculate statistics about pitch.

    Args:
        pitch_data: List of f0 values in Hz

    Returns:
        Dictionary with statistics
    """
    pitch = np.array(pitch_data)
    voiced = pitch > 0

    if not np.any(voiced):
        return {
            'mean_hz': 0,
            'median_hz': 0,
            'min_hz': 0,
            'max_hz': 0,
            'range_hz': 0,
            'std_hz': 0,
            'voiced_fraction': 0.0
        }

    voiced_pitch = pitch[voiced]

    return {
        'mean_hz': float(np.mean(voiced_pitch)),
        'median_hz': float(np.median(voiced_pitch)),
        'min_hz': float(np.min(voiced_pitch)),
        'max_hz': float(np.max(voiced_pitch)),
        'range_hz': float(np.ptp(voiced_pitch)),
        'std_hz': float(np.std(voiced_pitch)),
        'voiced_fraction': float(np.sum(voiced) / len(pitch))
    }


if __name__ == "__main__":
    # Test with Al-Fatihah 1:1
    test_url = "https://audio-cdn.tarteel.ai/quran/husary/001001.mp3"

    print("Testing pitch extraction...")
    pitch_data = extract_pitch_from_url(test_url)

    print(f"\n✓ Extracted {len(pitch_data['time'])} frames")
    print(f"✓ Duration: {pitch_data['duration']:.2f} seconds")

    stats = calculate_pitch_stats(pitch_data['f0_hz'])
    print(f"\n📊 Pitch Statistics:")
    print(f"  Mean: {stats['mean_hz']:.1f} Hz")
    print(f"  Range: {stats['min_hz']:.1f} - {stats['max_hz']:.1f} Hz")
    print(f"  Voiced: {stats['voiced_fraction']*100:.1f}%")
