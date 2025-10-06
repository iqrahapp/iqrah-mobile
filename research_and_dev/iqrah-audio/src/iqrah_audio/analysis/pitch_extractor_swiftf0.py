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
from scipy.signal import medfilt, savgol_filter
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

    # STEP 1: Gate by confidence FIRST (before range filtering)
    # This removes low-confidence frames that could be octave errors or noise
    v = voiced.astype(bool) & (confidence >= 0.6)

    # STEP 2: Apply range clamping AFTER confidence gating
    # Quranic recitation: male 65-420 Hz, allow some margin (55-550 Hz)
    f0 = f0_hz.copy()
    f0[~v] = 0.0

    # Range filter on voiced frames
    rng = (f0 >= 55) & (f0 <= 550)
    f0[~rng] = 0.0
    v = v & rng

    # STEP 3: Light smoothing on voiced frames to reduce jitter
    idx = np.where(f0 > 0)[0]
    if idx.size > 0:
        f = f0[idx]
        # Median filter removes outliers (kernel must be odd)
        f = medfilt(f, kernel_size=3)
        # Savitzky-Golay smooths pitch contour (needs at least 7 points)
        if f.size >= 7:
            f = savgol_filter(f, window_length=7, polyorder=2)
        f0[idx] = f

    # Compute accurate duration from audio samples (not from time array)
    duration = len(audio) / actual_sr

    return {
        'time': time.tolist(),
        'f0_hz': f0.tolist(),
        'confidence': confidence.tolist(),
        'voiced': v.tolist(),
        'sample_rate': actual_sr,
        'duration': float(duration)
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
    voiced_f0 = f0_hz[(f0_hz > 55) & (f0_hz < 550) & ~np.isnan(f0_hz)]

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
