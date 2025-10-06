"""
SwiftF0 Pitch Extraction
========================

State-of-the-art pitch extraction using SwiftF0.

SwiftF0 advantages:
- 42× faster than CREPE
- 91.80% accuracy at 10dB SNR (vs CREPE's ~79%)
- Better noise robustness
- Smaller model (95,842 params vs 2M)
"""

import numpy as np
import librosa
from swift_f0 import SwiftF0
from typing import Dict, List
from pathlib import Path
import urllib.request
import tempfile

# Initialize SwiftF0 detector (for Quranic recitation: 65-400 Hz range)
_detector = None

def _get_detector():
    global _detector
    if _detector is None:
        _detector = SwiftF0(fmin=65, fmax=400, confidence_threshold=0.5)
    return _detector


def extract_pitch_swiftf0(
    audio_path: str,
    sr: int = 16000,
    hop_length: int = 160  # 10ms at 16kHz
) -> Dict:
    """
    Extract pitch using SwiftF0.

    Args:
        audio_path: Path to audio file
        sr: Sample rate (16000 recommended for SwiftF0)
        hop_length: Hop size in samples (160 = 10ms at 16kHz)

    Returns:
        Dictionary with:
            - time: Time array (seconds)
            - f0_hz: F0 values (Hz)
            - confidence: Confidence scores (0-1)
            - voiced: Boolean array (True = voiced)
            - sample_rate: Sample rate
            - duration: Audio duration (seconds)
    """
    # Load audio
    audio, actual_sr = librosa.load(audio_path, sr=sr, mono=True)

    # Use SwiftF0 detector
    detector = _get_detector()
    result = detector.detect_from_array(audio, actual_sr)

    # SwiftF0 returns PitchResult with: pitch_hz, timestamps, voicing, confidence
    f0_hz = np.array(result.pitch_hz)
    confidence = np.array(result.confidence)
    time = np.array(result.timestamps)
    voiced = np.array(result.voicing)

    # CRITICAL FIX: Filter out unrealistic values (SwiftF0 sometimes has initialization spikes)
    # Human voice range: 50-500 Hz (male: 80-250 Hz, female: 150-400 Hz)
    realistic_mask = (f0_hz >= 50) & (f0_hz <= 500)

    # For unrealistic frames, set to 0 (unvoiced)
    f0_hz_filtered = f0_hz.copy()
    f0_hz_filtered[~realistic_mask] = 0.0

    # Update voiced array
    voiced_filtered = voiced & realistic_mask

    return {
        'time': time.tolist(),
        'f0_hz': f0_hz_filtered.tolist(),  # Filtered!
        'confidence': confidence.tolist(),
        'voiced': voiced_filtered.tolist(),  # Updated!
        'sample_rate': actual_sr,
        'duration': float(time[-1]) if len(time) > 0 else 0.0
    }


def extract_pitch_from_file(audio_path: str, sr: int = 16000) -> Dict:
    """
    Extract pitch from audio file using SwiftF0.

    Args:
        audio_path: Path to audio file
        sr: Sample rate

    Returns:
        Pitch data dictionary
    """
    return extract_pitch_swiftf0(audio_path, sr)


def extract_pitch_from_url(audio_url: str, sr: int = 16000) -> Dict:
    """
    Download audio from URL and extract pitch using SwiftF0.
    Also handles file:// URLs for local files.

    Args:
        audio_url: URL to audio file (http://, https://, or file://)
        sr: Sample rate

    Returns:
        Pitch data dictionary
    """
    # Handle file:// URLs (for testing)
    if audio_url.startswith('file://'):
        local_path = audio_url[7:]  # Remove 'file://' prefix
        return extract_pitch_swiftf0(local_path, sr)

    # Handle HTTP/HTTPS URLs
    # Create cache path based on URL hash
    cache_dir = Path(tempfile.gettempdir())
    cache_path = cache_dir / f"audio_{hash(audio_url)}.mp3"

    # Download if not cached
    if not cache_path.exists():
        print(f"Downloading audio from: {audio_url}")
        urllib.request.urlretrieve(audio_url, cache_path)
    else:
        print(f"Using cached audio: {cache_path}")

    return extract_pitch_swiftf0(str(cache_path), sr)


def calculate_pitch_stats(f0_hz_list: List[float]) -> Dict:
    """
    Calculate pitch statistics from F0 array.

    Args:
        f0_hz_list: List of F0 values (Hz), with 0 or NaN for unvoiced

    Returns:
        Statistics dictionary with range_hz field
    """
    f0_hz = np.array(f0_hz_list)

    # Filter out unvoiced (0 or NaN) and unrealistic values
    voiced_f0 = f0_hz[(f0_hz > 50) & (f0_hz < 500) & ~np.isnan(f0_hz)]

    if len(voiced_f0) == 0:
        return {
            'mean_hz': 0.0,
            'median_hz': 0.0,
            'std_hz': 0.0,
            'min_hz': 0.0,
            'max_hz': 0.0,
            'range_hz': 0.0,
            'voiced_ratio': 0.0
        }

    return {
        'mean_hz': float(np.mean(voiced_f0)),
        'median_hz': float(np.median(voiced_f0)),
        'std_hz': float(np.std(voiced_f0)),
        'min_hz': float(np.min(voiced_f0)),
        'max_hz': float(np.max(voiced_f0)),
        'range_hz': float(np.max(voiced_f0) - np.min(voiced_f0)),
        'voiced_ratio': float(len(voiced_f0) / len(f0_hz))
    }
