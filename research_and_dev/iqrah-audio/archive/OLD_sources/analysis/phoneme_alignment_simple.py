"""
Simple phoneme alignment improvement: boundary snapping only.

Instead of windowed CTC (which fails due to inaccurate word boundaries),
we keep the original full-audio CTC alignment and just snap boundaries
to acoustic minima for better accuracy.
"""

import torch
import torchaudio
import numpy as np
import librosa
from typing import List, Dict
from .phoneme_wav2vec2_ctc import extract_phonemes_wav2vec2_ctc


def extract_phonemes_simple_improved(
    audio_path: str,
    word_segments: List[Dict],
    transliteration: str,
    pitch_data: Dict,
    surah: int,
    ayah: int
) -> List[Dict]:
    """
    Extract phonemes with boundary snapping improvement.

    Steps:
    1. Use original CTC alignment (full audio, good coverage)
    2. Compute acoustic features (RMS, spectral flux, voicing)
    3. Snap phoneme boundaries to acoustic minima
    4. Keep all other processing the same
    """
    print(f"\n🎯 Simple Improved Phoneme Alignment")
    print(f"   Strategy: Full-audio CTC + boundary snapping")

    # Step 1: Original CTC alignment
    print(f"\n1️⃣ Running full-audio CTC alignment...")
    phonemes = extract_phonemes_wav2vec2_ctc(
        audio_path, word_segments, transliteration, pitch_data, surah, ayah
    )
    print(f"   ✓ Extracted {len(phonemes)} phonemes (coverage: {sum(p['duration'] for p in phonemes) / pitch_data['duration'] * 100:.1f}%)")

    # Step 2: Compute acoustic features
    print(f"\n2️⃣ Computing acoustic features...")
    y, sr = librosa.load(audio_path, sr=16000)

    # RMS energy (frame_length=512, hop_length=160 → 100 Hz)
    frame_length = 512
    hop_length = 160
    rms = librosa.feature.rms(y=y, frame_length=frame_length, hop_length=hop_length)[0]

    # Spectral flux
    spec = np.abs(librosa.stft(y, n_fft=frame_length, hop_length=hop_length))
    spec_flux = np.sqrt(np.sum(np.diff(spec, axis=1)**2, axis=0))
    spec_flux = np.concatenate([[0], spec_flux])  # Pad to match length

    # Voicing (from pitch data)
    times_rms = librosa.frames_to_time(np.arange(len(rms)), sr=sr, hop_length=hop_length)
    voicing = np.interp(times_rms, pitch_data['time'], (np.array(pitch_data['f0_hz']) > 0).astype(float))

    print(f"   ✓ RMS, spectral flux, voicing computed ({len(times_rms)} frames)")

    # Step 3: Snap boundaries
    print(f"\n3️⃣ Snapping boundaries to acoustic minima...")
    phonemes = snap_boundaries_to_minima(phonemes, rms, spec_flux, voicing, times_rms)
    print(f"   ✓ Boundaries snapped")

    print(f"\n✅ {len(phonemes)} phonemes ready")
    return phonemes


def compute_salience(
    rms: np.ndarray,
    spec_flux: np.ndarray,
    voicing: np.ndarray,
    alpha: float = 0.6,
    beta: float = 0.3,
    gamma: float = 0.1
) -> np.ndarray:
    """
    Compute acoustic salience for boundary detection.

    salience(t) = α·RMS_z(t) + β·SpecFlux_z(t) + γ·(1-voicing(t))

    Lower salience = better boundary location (low energy, low flux, unvoiced).
    """
    def z_score(a):
        return (a - np.mean(a)) / (np.std(a) + 1e-8)

    rms_z = z_score(rms)
    flux_z = z_score(spec_flux)

    salience = alpha * rms_z + beta * flux_z + gamma * (1 - voicing)
    return salience


def snap_boundaries_to_minima(
    phonemes: List[Dict],
    rms: np.ndarray,
    spec_flux: np.ndarray,
    voicing: np.ndarray,
    times: np.ndarray,
    win_ms: int = 50
) -> List[Dict]:
    """
    Snap phoneme boundaries to local acoustic minima.

    For each boundary between phonemes, search within ±win_ms
    for the point with lowest acoustic salience.
    """
    salience = compute_salience(rms, spec_flux, voicing)

    # Window size in frames
    dt = times[1] - times[0] if len(times) > 1 else 0.01
    W = int((win_ms / 1000) / dt)

    # Snap internal boundaries (not first start or last end)
    for i in range(1, len(phonemes)):
        # Boundary time
        t = phonemes[i]['start']

        # Find closest frame
        k = np.argmin(np.abs(times - t))

        # Window around boundary
        s = max(0, k - W)
        e = min(len(times), k + W + 1)

        # Find local minimum in salience
        sal_window = salience[s:e]
        if len(sal_window) == 0:
            continue

        kk = s + np.argmin(sal_window)
        t_new = times[kk]

        # Update boundary (end of previous phoneme, start of current)
        phonemes[i-1]['end'] = t_new
        phonemes[i-1]['duration'] = phonemes[i-1]['end'] - phonemes[i-1]['start']

        phonemes[i]['start'] = t_new
        phonemes[i]['duration'] = phonemes[i]['end'] - phonemes[i]['start']

    return phonemes
