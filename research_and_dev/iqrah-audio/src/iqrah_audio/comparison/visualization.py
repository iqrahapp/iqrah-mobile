"""
Visualization Module for Comparison Results
============================================

Generates visual representations of comparison results including:
- DTW alignment path overlays
- Pitch contour comparisons
- Rhythm onset strength plots
- Spectrogram with alignment markers
"""

import numpy as np
import matplotlib
matplotlib.use('Agg')  # Non-interactive backend for server use
import matplotlib.pyplot as plt
from matplotlib.patches import Rectangle
import io
import base64
from typing import Optional, Tuple, Dict
import librosa
import librosa.display


def plot_dtw_path(
    student_features: np.ndarray,
    reference_features: np.ndarray,
    path: np.ndarray,
    title: str = "DTW Alignment Path",
    figsize: Tuple[int, int] = (10, 8)
) -> str:
    """
    Plot DTW alignment path on cost matrix.

    Args:
        student_features: Student feature sequence [T1, D]
        reference_features: Reference feature sequence [T2, D]
        path: Alignment path [(i, j), ...]
        title: Plot title
        figsize: Figure size (width, height)

    Returns:
        Base64-encoded PNG image
    """
    fig, ax = plt.subplots(figsize=figsize)

    # Compute cost matrix (Euclidean distance)
    T1, T2 = len(student_features), len(reference_features)
    cost_matrix = np.zeros((T1, T2))
    for i in range(T1):
        for j in range(T2):
            cost_matrix[i, j] = np.linalg.norm(student_features[i] - reference_features[j])

    # Plot cost matrix
    im = ax.imshow(cost_matrix.T, origin='lower', aspect='auto', cmap='viridis')
    plt.colorbar(im, ax=ax, label='Distance')

    # Plot DTW path
    if len(path) > 0:
        path_i = path[:, 0]
        path_j = path[:, 1]
        ax.plot(path_i, path_j, 'r-', linewidth=2, label='Alignment Path')
        ax.plot(path_i, path_j, 'ro', markersize=3)

    ax.set_xlabel('Student Time (frames)', fontsize=12)
    ax.set_ylabel('Reference Time (frames)', fontsize=12)
    ax.set_title(title, fontsize=14, fontweight='bold')
    ax.legend()
    ax.grid(True, alpha=0.3)

    # Convert to base64
    buf = io.BytesIO()
    plt.tight_layout()
    plt.savefig(buf, format='png', dpi=100, bbox_inches='tight')
    plt.close(fig)
    buf.seek(0)
    img_base64 = base64.b64encode(buf.read()).decode('utf-8')

    return f"data:image/png;base64,{img_base64}"


def plot_pitch_comparison(
    student_pitch: Dict,
    reference_pitch: Dict,
    student_time: np.ndarray,
    reference_time: np.ndarray,
    path: Optional[np.ndarray] = None,
    pitch_shift_cents: float = 0,
    figsize: Tuple[int, int] = (14, 6),
    student_frame_times: Optional[np.ndarray] = None,
    reference_frame_times: Optional[np.ndarray] = None
) -> str:
    """
    Plot pitch contour comparison with alignment.

    Args:
        student_pitch: Student pitch data {'f0_hz': array, 'confidence': array, 'time': array}
        reference_pitch: Reference pitch data
        student_time: Student time grid (unused, using pitch data's own time)
        reference_time: Reference time grid (unused, using pitch data's own time)
        path: DTW alignment path (optional)
        pitch_shift_cents: Key difference in cents
        figsize: Figure size

    Returns:
        Base64-encoded PNG image
    """
    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=figsize, sharex=False)

    # Convert to semitones for better visualization
    def hz_to_semitones(f0_hz):
        f0_hz = np.array(f0_hz)  # Ensure numpy array
        valid = f0_hz > 0
        semitones = np.full_like(f0_hz, np.nan, dtype=float)
        semitones[valid] = 12 * np.log2(f0_hz[valid] / 55.0)  # A1 = 55 Hz reference
        return semitones

    student_semitones = hz_to_semitones(student_pitch['f0_hz'])
    reference_semitones = hz_to_semitones(reference_pitch['f0_hz'])

    # Use pitch data's own time arrays
    student_time_original = np.array(student_pitch['time'])
    reference_time = np.array(reference_pitch['time'])

    # Debug: print durations
    print(f"[DEBUG viz] Student duration: {student_time_original[-1]:.2f}s, Reference duration: {reference_time[-1]:.2f}s")

    # Apply DTW warping to student time if path and frame_times provided
    if path is not None and len(path) > 0 and student_frame_times is not None and reference_frame_times is not None:
        print(f"[DEBUG viz] Applying DTW warping with path length {len(path)}")
        print(f"[DEBUG viz] Feature frame_times: student {len(student_frame_times)}, ref {len(reference_frame_times)}")

        student_time_warped = np.zeros_like(student_time_original)
        student_frame_times_np = np.array(student_frame_times)
        reference_frame_times_np = np.array(reference_frame_times)

        # CORRECT MAPPING:
        # 1. Student pitch time -> Student feature frame index (via frame_times)
        # 2. Student feature frame -> Reference feature frame (via DTW path)
        # 3. Reference feature frame -> Reference pitch time (via frame_times)

        for i, pitch_time in enumerate(student_time_original):
            # Step 1: Find closest student feature frame for this pitch time
            student_feat_idx = np.argmin(np.abs(student_frame_times_np - pitch_time))

            # Step 2: Find corresponding reference feature frame in DTW path
            # Path is list of (student_feat_idx, ref_feat_idx) pairs
            best_j = 0
            min_dist = abs(path[0][0] - student_feat_idx)
            for j in range(1, len(path)):
                dist = abs(path[j][0] - student_feat_idx)
                if dist < min_dist:
                    min_dist = dist
                    best_j = j
                elif dist > min_dist:
                    break

            ref_feat_idx = path[best_j][1]

            # Step 3: Get reference time for this feature frame
            if ref_feat_idx < len(reference_frame_times_np):
                ref_time = reference_frame_times_np[ref_feat_idx]
            else:
                ref_time = reference_frame_times_np[-1]

            student_time_warped[i] = ref_time

        student_time = student_time_warped
        print(f"[DEBUG viz] Warped student time: {student_time[0]:.2f}s to {student_time[-1]:.2f}s")
        print(f"[DEBUG viz] Original student time: {student_time_original[0]:.2f}s to {student_time_original[-1]:.2f}s")
        print(f"[DEBUG viz] Reference time: {reference_time[0]:.2f}s to {reference_time[-1]:.2f}s")
    else:
        student_time = student_time_original
        print(f"[DEBUG viz] No DTW path, using original student time")

    # Top plot: Pitch contours (student time is now warped!)
    ax1.plot(student_time, student_semitones, 'b-', linewidth=2, label='Student', alpha=0.7)
    ax1.plot(reference_time, reference_semitones, 'r-', linewidth=2, label='Reference', alpha=0.7)

    # Set x-axis to cover both durations
    max_time = max(student_time[-1] if len(student_time) > 0 else 0,
                   reference_time[-1] if len(reference_time) > 0 else 0)
    ax1.set_xlim(0, max_time)

    # Add alignment markers if path provided
    if path is not None and len(path) > 0:
        # Sample path points for clarity (every 10th point)
        for i in range(0, len(path), 10):
            idx_student, idx_reference = path[i]
            if idx_student < len(student_time) and idx_reference < len(reference_time):
                t_student = student_time[idx_student]
                t_reference = reference_time[idx_reference]
                y_student = student_semitones[idx_student]
                y_reference = reference_semitones[idx_reference]
                if not np.isnan(y_student) and not np.isnan(y_reference):
                    ax1.plot([t_student, t_reference], [y_student, y_reference],
                            'g--', alpha=0.3, linewidth=0.5)

    ax1.set_ylabel('Pitch (semitones)', fontsize=12)
    ax1.set_title(f'Pitch Contour Comparison (Key Shift: {pitch_shift_cents:+.0f} cents)',
                  fontsize=14, fontweight='bold')
    ax1.legend(loc='upper right')
    ax1.grid(True, alpha=0.3)

    # Bottom plot: ΔF0 (first difference) - melodic contour
    student_df0 = np.diff(student_semitones)
    reference_df0 = np.diff(reference_semitones)

    # Use warped student_time (already applied above if DTW path provided)
    ax2.plot(student_time[1:], student_df0, 'b-', linewidth=1.5, label='Student ΔF0', alpha=0.7)
    ax2.plot(reference_time[1:], reference_df0, 'r-', linewidth=1.5, label='Reference ΔF0', alpha=0.7)
    ax2.set_xlim(0, max_time)  # Match x-axis range with top plot
    ax2.axhline(0, color='k', linestyle='--', alpha=0.3)

    ax2.set_xlabel('Time (seconds)', fontsize=12)
    ax2.set_ylabel('ΔF0 (semitones/frame)', fontsize=12)
    ax2.set_title('Melodic Contour (Key-Invariant)', fontsize=12, fontweight='bold')
    ax2.legend(loc='upper right')
    ax2.grid(True, alpha=0.3)

    # Convert to base64
    buf = io.BytesIO()
    plt.tight_layout()
    plt.savefig(buf, format='png', dpi=100, bbox_inches='tight')
    plt.close(fig)
    buf.seek(0)
    img_base64 = base64.b64encode(buf.read()).decode('utf-8')

    return f"data:image/png;base64,{img_base64}"


def plot_rhythm_comparison(
    student_onset,
    reference_onset,
    student_time,
    reference_time,
    path: Optional[np.ndarray] = None,
    tempo_ratio: float = 1.0,
    figsize: Tuple[int, int] = (14, 8)
) -> str:
    """
    Plot rhythm onset strength comparison with alignment.

    Args:
        student_onset: Student onset strength
        reference_onset: Reference onset strength
        student_time: Student time grid
        reference_time: Reference time grid
        path: DTW alignment path (optional)
        tempo_ratio: Tempo ratio (student/reference)
        figsize: Figure size

    Returns:
        Base64-encoded PNG image
    """
    # Ensure numpy arrays
    student_onset = np.array(student_onset)
    reference_onset = np.array(reference_onset)
    student_time = np.array(student_time)
    reference_time = np.array(reference_time)

    fig, (ax1, ax2, ax3) = plt.subplots(3, 1, figsize=figsize, sharex=False)

    # Top: Student onset
    ax1.fill_between(student_time, 0, student_onset, color='blue', alpha=0.6)
    ax1.plot(student_time, student_onset, 'b-', linewidth=1)
    ax1.set_ylabel('Onset Strength', fontsize=11)
    ax1.set_title('Student Rhythm', fontsize=12, fontweight='bold')
    ax1.grid(True, alpha=0.3)

    # Middle: Reference onset
    ax2.fill_between(reference_time, 0, reference_onset, color='red', alpha=0.6)
    ax2.plot(reference_time, reference_onset, 'r-', linewidth=1)
    ax2.set_ylabel('Onset Strength', fontsize=11)
    ax2.set_title('Reference Rhythm', fontsize=12, fontweight='bold')
    ax2.grid(True, alpha=0.3)

    # Bottom: Aligned comparison
    # Warp student time using DTW path
    if path is not None and len(path) > 0:
        warped_time = np.zeros(len(reference_time))
        warped_onset = np.zeros(len(reference_time))

        for i, (idx_student, idx_reference) in enumerate(path):
            if idx_reference < len(reference_time) and idx_student < len(student_time):
                warped_time[idx_reference] = student_time[idx_student]
                warped_onset[idx_reference] = student_onset[idx_student]

        ax3.plot(reference_time, reference_onset, 'r-', linewidth=2, label='Reference', alpha=0.7)
        ax3.plot(reference_time, warped_onset, 'b-', linewidth=2, label='Student (aligned)', alpha=0.7)
    else:
        ax3.plot(reference_time, reference_onset, 'r-', linewidth=2, label='Reference', alpha=0.7)
        ax3.plot(student_time, student_onset, 'b-', linewidth=2, label='Student', alpha=0.7)

    ax3.set_xlabel('Time (seconds)', fontsize=12)
    ax3.set_ylabel('Onset Strength', fontsize=11)
    ax3.set_title(f'Aligned Comparison (Tempo Ratio: {tempo_ratio:.2f})',
                  fontsize=12, fontweight='bold')
    ax3.legend(loc='upper right')
    ax3.grid(True, alpha=0.3)

    # Convert to base64
    buf = io.BytesIO()
    plt.tight_layout()
    plt.savefig(buf, format='png', dpi=100, bbox_inches='tight')
    plt.close(fig)
    buf.seek(0)
    img_base64 = base64.b64encode(buf.read()).decode('utf-8')

    return f"data:image/png;base64,{img_base64}"


def plot_spectrogram_with_alignment(
    audio_path: str,
    phonemes: list,
    pitch_data: Dict,
    figsize: Tuple[int, int] = (14, 8)
) -> str:
    """
    Plot spectrogram with phoneme boundaries and pitch overlay.

    Args:
        audio_path: Path to audio file
        phonemes: List of phoneme dicts with timing info
        pitch_data: Pitch data dict
        figsize: Figure size

    Returns:
        Base64-encoded PNG image
    """
    # Load audio
    y, sr = librosa.load(audio_path, sr=22050)

    # Compute spectrogram
    D = librosa.amplitude_to_db(np.abs(librosa.stft(y)), ref=np.max)

    fig, ax = plt.subplots(figsize=figsize)

    # Plot spectrogram
    img = librosa.display.specshow(D, sr=sr, x_axis='time', y_axis='hz', ax=ax, cmap='magma')
    plt.colorbar(img, ax=ax, format='%+2.0f dB')

    # Overlay phoneme boundaries
    for phoneme in phonemes:
        start = phoneme['start']
        end = phoneme['end']
        duration = end - start

        # Draw boundary line
        ax.axvline(start, color='cyan', linestyle='--', linewidth=1, alpha=0.6)

        # Add phoneme label
        label = phoneme.get('phoneme', '')
        if label:
            ax.text(start + duration/2, sr * 0.45, label,
                   color='white', fontsize=8, ha='center', va='center',
                   bbox=dict(boxstyle='round,pad=0.3', facecolor='black', alpha=0.6))

    # Overlay pitch contour
    f0_hz = np.array(pitch_data['f0_hz'])
    time_pitch = np.array(pitch_data['time'])
    valid = f0_hz > 0
    ax.plot(time_pitch[valid], f0_hz[valid], 'lime', linewidth=2, label='Pitch (F0)', alpha=0.8)

    ax.set_title('Spectrogram with Phoneme Alignment and Pitch Overlay',
                 fontsize=14, fontweight='bold')
    ax.set_xlabel('Time (seconds)', fontsize=12)
    ax.set_ylabel('Frequency (Hz)', fontsize=12)
    ax.legend(loc='upper right')
    ax.set_ylim(0, 1000)  # Focus on vocal range

    # Convert to base64
    buf = io.BytesIO()
    plt.tight_layout()
    plt.savefig(buf, format='png', dpi=100, bbox_inches='tight')
    plt.close(fig)
    buf.seek(0)
    img_base64 = base64.b64encode(buf.read()).decode('utf-8')

    return f"data:image/png;base64,{img_base64}"


def generate_comparison_visualizations(
    comparison_result: Dict,
    student_audio_path: str,
    reference_audio_path: str,
    student_features,
    reference_features,
    student_phonemes: list,
    reference_phonemes: list,
    student_pitch: Dict,
    reference_pitch: Dict
) -> Dict[str, str]:
    """
    Generate all comparison visualizations.

    Args:
        comparison_result: Result from compare_recitations()
        student_audio_path: Path to student audio
        reference_audio_path: Path to reference audio
        student_features: Student FeaturePack
        reference_features: Reference FeaturePack
        student_phonemes: Student phoneme list
        reference_phonemes: Reference phoneme list
        student_pitch: Student pitch data
        reference_pitch: Reference pitch data

    Returns:
        Dictionary of visualization names to base64 images
    """
    from .features import build_multi_feature_stack

    visualizations = {}

    # 1. DTW Path on Rhythm Features
    student_rhythm_features = build_multi_feature_stack(student_features)
    reference_rhythm_features = build_multi_feature_stack(reference_features)
    rhythm_path = np.array(comparison_result['rhythm'].get('path', []))

    if len(rhythm_path) > 0:
        visualizations['dtw_path'] = plot_dtw_path(
            student_rhythm_features,
            reference_rhythm_features,
            rhythm_path,
            title="Rhythm DTW Alignment Path"
        )

    # 2. Pitch Comparison
    student_frame_times = comparison_result['rhythm'].get('student_frame_times')
    reference_frame_times = comparison_result['rhythm'].get('reference_frame_times')

    visualizations['pitch_comparison'] = plot_pitch_comparison(
        student_pitch,
        reference_pitch,
        student_features.frame_times,
        reference_features.frame_times,
        path=rhythm_path if len(rhythm_path) > 0 else None,
        pitch_shift_cents=comparison_result['melody']['pitch_shift_cents'],
        student_frame_times=np.array(student_frame_times) if student_frame_times else None,
        reference_frame_times=np.array(reference_frame_times) if reference_frame_times else None
    )

    # 3. Rhythm Comparison
    visualizations['rhythm_comparison'] = plot_rhythm_comparison(
        student_features.onset_strength,
        reference_features.onset_strength,
        student_features.frame_times,
        reference_features.frame_times,
        path=rhythm_path if len(rhythm_path) > 0 else None,
        tempo_ratio=comparison_result['metadata']['tempo_ratio']
    )

    # 4. Student Spectrogram
    visualizations['student_spectrogram'] = plot_spectrogram_with_alignment(
        student_audio_path,
        student_phonemes,
        student_pitch
    )

    # 5. Reference Spectrogram
    visualizations['reference_spectrogram'] = plot_spectrogram_with_alignment(
        reference_audio_path,
        reference_phonemes,
        reference_pitch
    )

    return visualizations
