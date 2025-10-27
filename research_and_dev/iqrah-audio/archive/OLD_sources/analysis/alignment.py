"""
Offline DTW Alignment
====================

Full DTW alignment (no time pressure - can take 10 seconds!)
Much more accurate than online DTW.
"""

import numpy as np
from typing import List, Tuple, Dict
from dtaidistance import dtw
from scipy.interpolate import interp1d


def prepare_pitch_for_dtw(pitch_data: List[float], voiced: List[bool] = None) -> np.ndarray:
    """
    Prepare pitch data for DTW alignment.

    Args:
        pitch_data: F0 values in Hz
        voiced: Optional boolean mask

    Returns:
        Prepared pitch array (log-scaled, interpolated)
    """
    pitch = np.array(pitch_data)

    if voiced is None:
        voiced = pitch > 0
    else:
        voiced = np.array(voiced)

    # Interpolate unvoiced frames (DTW needs continuous signal)
    if np.any(voiced):
        valid_indices = np.where(voiced)[0]
        if len(valid_indices) > 1:
            # Interpolate unvoiced regions
            interp = interp1d(
                valid_indices,
                pitch[voiced],
                kind='linear',
                fill_value='extrapolate',
                bounds_error=False
            )
            pitch = interp(np.arange(len(pitch)))

    # Convert to log scale (semitones)
    pitch = np.log2(np.maximum(pitch, 1.0))

    # Mean-center (normalize absolute pitch)
    if np.any(voiced):
        pitch -= np.mean(pitch[voiced])

    return pitch


def align_sequences_dtw(
    user_pitch: List[float],
    ref_pitch: List[float],
    user_voiced: List[bool] = None,
    ref_voiced: List[bool] = None
) -> Dict:
    """
    Align user pitch to reference pitch using full DTW.

    Args:
        user_pitch: User's f0 values
        ref_pitch: Reference f0 values
        user_voiced: User's voicing flags
        ref_voiced: Reference voicing flags

    Returns:
        Dictionary with:
            - path: DTW alignment path [(user_idx, ref_idx), ...]
            - distance: DTW distance
            - user_to_ref: Mapping from user frame to reference frame
            - ref_to_user: Mapping from reference frame to user frame
    """

    # Prepare sequences
    user_seq = prepare_pitch_for_dtw(user_pitch, user_voiced)
    ref_seq = prepare_pitch_for_dtw(ref_pitch, ref_voiced)

    # Compute DTW
    print(f"Computing DTW alignment...")
    print(f"  User frames: {len(user_seq)}")
    print(f"  Reference frames: {len(ref_seq)}")

    distance = dtw.distance(user_seq, ref_seq)
    path = dtw.warping_path(user_seq, ref_seq)

    print(f"✓ DTW distance: {distance:.2f}")
    print(f"✓ Path length: {len(path)}")

    # Create mappings
    user_to_ref = {}
    ref_to_user = {}

    for user_idx, ref_idx in path:
        user_to_ref[user_idx] = ref_idx
        if ref_idx not in ref_to_user:
            ref_to_user[ref_idx] = user_idx

    return {
        'path': path,
        'distance': float(distance),
        'user_to_ref': user_to_ref,
        'ref_to_user': ref_to_user,
        'user_frames': len(user_seq),
        'ref_frames': len(ref_seq)
    }


def map_user_to_words(
    alignment: Dict,
    ref_segments: List[Dict],
    user_duration: float,
    ref_duration: float
) -> List[int]:
    """
    Map each user frame to a word index.

    Args:
        alignment: DTW alignment result
        ref_segments: Reference word segments
        user_duration: User audio duration
        ref_duration: Reference audio duration

    Returns:
        List of word indices (one per user frame)
    """

    user_to_ref = alignment['user_to_ref']
    user_frames = alignment['user_frames']
    ref_frames = alignment['ref_frames']

    # Frame rate (CREPE uses 10ms steps by default)
    user_frame_rate = user_frames / user_duration if user_duration > 0 else 100
    ref_frame_rate = ref_frames / ref_duration if ref_duration > 0 else 100

    word_alignment = []

    for user_frame in range(user_frames):
        # Get corresponding reference frame
        ref_frame = user_to_ref.get(user_frame, -1)

        if ref_frame < 0:
            word_alignment.append(-1)
            continue

        # Convert ref frame to time (ms)
        ref_time_ms = (ref_frame / ref_frame_rate) * 1000

        # Find word at this time
        word_idx = -1
        for i, seg in enumerate(ref_segments):
            if seg['start_ms'] <= ref_time_ms <= seg['end_ms']:
                word_idx = i
                break

        word_alignment.append(word_idx)

    return word_alignment


def calculate_tempo_ratio(alignment: Dict, user_duration: float, ref_duration: float) -> Dict:
    """
    Calculate tempo (speed) ratio from alignment.

    Args:
        alignment: DTW alignment result
        user_duration: User audio duration (seconds)
        ref_duration: Reference audio duration (seconds)

    Returns:
        Dictionary with tempo statistics
    """

    path = alignment['path']

    # Calculate local speed ratios
    local_speeds = []

    for i in range(1, len(path)):
        user_delta = path[i][0] - path[i-1][0]
        ref_delta = path[i][1] - path[i-1][1]

        if ref_delta > 0:
            speed = user_delta / ref_delta  # >1 = user slower, <1 = user faster
            local_speeds.append(speed)

    local_speeds = np.array(local_speeds)

    return {
        'global_ratio': user_duration / ref_duration if ref_duration > 0 else 1.0,
        'mean_ratio': float(np.mean(local_speeds)) if len(local_speeds) > 0 else 1.0,
        'std_ratio': float(np.std(local_speeds)) if len(local_speeds) > 0 else 0.0,
        'is_stable': float(np.std(local_speeds)) < 0.3 if len(local_speeds) > 0 else True
    }


if __name__ == "__main__":
    # Test alignment
    print("Testing DTW alignment...")

    # Create synthetic test sequences
    ref = np.sin(np.linspace(0, 4*np.pi, 100))
    user = np.sin(np.linspace(0, 4*np.pi, 120))  # 20% slower

    alignment = align_sequences_dtw(
        user.tolist(),
        ref.tolist()
    )

    print(f"\n✓ Alignment complete")
    print(f"  Distance: {alignment['distance']:.2f}")
    print(f"  Path length: {len(alignment['path'])}")

    tempo = calculate_tempo_ratio(alignment, 12.0, 10.0)
    print(f"\n📊 Tempo:")
    print(f"  Ratio: {tempo['mean_ratio']:.2f}x")
    print(f"  Stable: {tempo['is_stable']}")
