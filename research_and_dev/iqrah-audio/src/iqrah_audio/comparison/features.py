"""
Feature Extraction for Comparison Engine
=========================================

Extracts normalized features for tempo-invariant rhythm and key-invariant melody comparison.
Based on SOTA report recommendations.
"""

import numpy as np
import torch
import torchaudio
import librosa
from dataclasses import dataclass
from typing import Optional, Tuple
from scipy.signal import savgol_filter


@dataclass
class FeaturePack:
    """
    Standardized feature pack for comparison.

    All features are aligned to the same time grid with ~50-80 Hz frame rate,
    then resampled to L≈150 for DTW efficiency.
    """
    onset_strength: np.ndarray      # [T] z-scored per clip
    syll_onset_mask: np.ndarray     # [T] {0,1}
    norm_time: np.ndarray           # [T] in [0,1]
    f0_semitones: np.ndarray        # [T], NaN on unvoiced
    df0: np.ndarray                 # [T], z-norm per phrase
    hpcp: Optional[np.ndarray]      # [T,12] chroma (optional)
    frame_times: np.ndarray         # [T]

    # Metadata
    duration: float
    tempo_estimate: float           # syllables per second
    mean_count: float               # from baseline phonemes


def extract_features(
    audio_path: str,
    phonemes: list,
    pitch_data: dict,
    statistics: dict,
    target_length: int = 200
) -> FeaturePack:
    """
    Extract comparison features from analysis results.

    Args:
        audio_path: Path to audio file
        phonemes: List of phoneme dictionaries from Phase 1
        pitch_data: Pitch data from Phase 1 (SwiftF0/CREPE)
        statistics: Statistics from Phase 1
        target_length: Target sequence length for DTW (default 150)

    Returns:
        FeaturePack with normalized features
    """
    # Load audio
    waveform, sr = torchaudio.load(audio_path)
    if waveform.size(0) > 1:
        waveform = waveform.mean(dim=0, keepdim=True)

    duration = waveform.size(1) / sr

    # Extract onset strength (using librosa)
    y = waveform.numpy()[0]
    onset_env = librosa.onset.onset_strength(y=y, sr=sr)
    onset_times = librosa.frames_to_time(np.arange(len(onset_env)), sr=sr)

    # Create syllable onset mask from phonemes
    syll_onsets = np.array([p['start'] for p in phonemes if p.get('mean_pitch', 0) > 0])

    # Create common time grid (50 Hz)
    frame_rate = 50
    n_frames = int(duration * frame_rate)
    frame_times = np.linspace(0, duration, n_frames)

    # Interpolate onset strength to common grid
    onset_strength_interp = np.interp(frame_times, onset_times, onset_env)
    onset_strength_zscore = (onset_strength_interp - np.mean(onset_strength_interp)) / (np.std(onset_strength_interp) + 1e-8)

    # Create syllable onset mask
    syll_onset_mask = np.zeros(n_frames)
    for onset in syll_onsets:
        idx = np.argmin(np.abs(frame_times - onset))
        if idx < n_frames:
            syll_onset_mask[idx] = 1.0

    # Normalized time
    norm_time = frame_times / duration

    # Extract F0 in semitones (relative to 55 Hz = A1)
    pitch_times = np.array(pitch_data['time'])
    pitch_f0 = np.array(pitch_data['f0_hz'])

    # Interpolate to common grid
    f0_interp = np.interp(frame_times, pitch_times, pitch_f0)

    # Convert to semitones (with NaN for unvoiced)
    # Handle zeros to avoid log(0) warning
    f0_safe = np.where(f0_interp > 0, f0_interp, 1.0)  # Replace 0 with 1 temporarily
    f0_semitones = np.where(
        f0_interp > 0,
        12 * np.log2(f0_safe / 55.0),
        np.nan
    )

    # Compute ΔF0 (first difference) with smoothing
    df0_raw = np.diff(f0_semitones, prepend=f0_semitones[0])

    # Replace NaNs with 0 for smoothing
    df0_clean = np.nan_to_num(df0_raw, nan=0.0)

    # Smooth with Savitzky-Golay filter
    if len(df0_clean) > 11:
        df0_smooth = savgol_filter(df0_clean, 11, 3)
    else:
        df0_smooth = df0_clean

    # Z-normalize per phrase (entire clip for now)
    df0_znorm = (df0_smooth - np.mean(df0_smooth)) / (np.std(df0_smooth) + 1e-8)

    # Resample all features to target length for DTW efficiency
    if n_frames != target_length:
        indices = np.linspace(0, n_frames - 1, target_length).astype(int)
        onset_strength_zscore = onset_strength_zscore[indices]
        syll_onset_mask = syll_onset_mask[indices]
        norm_time = norm_time[indices]
        f0_semitones = f0_semitones[indices]
        df0_znorm = df0_znorm[indices]
        frame_times = frame_times[indices]

    return FeaturePack(
        onset_strength=onset_strength_zscore,
        syll_onset_mask=syll_onset_mask,
        norm_time=norm_time,
        f0_semitones=f0_semitones,
        df0=df0_znorm,
        hpcp=None,  # TODO: Add HPCP/chroma extraction
        frame_times=frame_times,
        duration=duration,
        tempo_estimate=statistics['tempo']['syllables_per_second'],
        mean_count=statistics['count']['mean_count']
    )


def build_multi_feature_stack(feat: FeaturePack) -> np.ndarray:
    """
    Build the multi-feature stack for rhythm alignment.

    CRITICAL FIX: Make onset-dominant for rhythm (not ΔF0-dominant).
    Weights: onset=1.0, syllable_mask=0.3, norm_time=0.2
    ΔF0 removed - use only for melody comparison, not rhythm warping.

    Args:
        feat: FeaturePack

    Returns:
        Feature matrix [T, 3] - onset-led rhythm features
    """
    # Already z-scored in extract_features
    onset = feat.onset_strength

    # Scale binary mask to have small influence
    mask = feat.syll_onset_mask.astype(float) * 0.5

    # Normalize time to ~unit variance (range [0,1] -> std ~0.29)
    tnorm = (feat.norm_time - 0.5) / 0.29

    # Weight features: onset dominates
    feature_stack = np.stack([
        1.0 * onset,    # Primary: onset envelope
        0.3 * mask,     # Secondary: syllable markers
        0.2 * tnorm     # Tertiary: time progression (prevent wild warping)
    ], axis=1)

    return feature_stack


def estimate_pitch_shift(student: FeaturePack, reference: FeaturePack) -> float:
    """
    Estimate pitch shift in cents between student and reference.

    Uses median difference of voiced semitones.

    Returns:
        Pitch shift in cents (100 cents = 1 semitone)
    """
    # Get voiced frames
    student_voiced = student.f0_semitones[~np.isnan(student.f0_semitones)]
    ref_voiced = reference.f0_semitones[~np.isnan(reference.f0_semitones)]

    if len(student_voiced) == 0 or len(ref_voiced) == 0:
        return 0.0

    # Compute median pitch shift
    median_student = np.median(student_voiced)
    median_ref = np.median(ref_voiced)

    semitone_shift = median_student - median_ref
    cents_shift = semitone_shift * 100

    return float(cents_shift)


def extract_tempo_ratio(student: FeaturePack, reference: FeaturePack) -> float:
    """
    Estimate tempo ratio: student_tempo / reference_tempo.

    Returns:
        Tempo ratio (1.0 = same, >1 = student slower, <1 = student faster)
    """
    if reference.tempo_estimate == 0:
        return 1.0

    return student.tempo_estimate / reference.tempo_estimate
