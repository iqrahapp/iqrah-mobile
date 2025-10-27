"""
Melody Contour Visualization - Shows pitch contour comparison.

Displays:
- ΔF0 contour for student and reference (overlay)
- Key shift annotation
- Contour similarity heatmap
- Voiced/unvoiced regions
"""
import numpy as np
import matplotlib.pyplot as plt
from typing import Dict, Optional, Tuple
import io
import base64


def plot_melody_contour(
    comparison_result: Dict,
    student_pitch: Optional[Dict] = None,
    reference_pitch: Optional[Dict] = None,
    show_key_shift: bool = True,
    show_similarity: bool = True,
    figsize: Tuple[int, int] = (12, 6),
    return_base64: bool = False
):
    """
    Plot ΔF0 melody contour comparison.

    Args:
        comparison_result: Result from compare_recitations() containing melody data
        student_pitch: Optional student pitch analysis dict
        reference_pitch: Optional reference pitch analysis dict
        show_key_shift: Whether to annotate key shift
        show_similarity: Whether to show similarity score
        figsize: Figure size (width, height)
        return_base64: If True, return base64-encoded PNG instead of displaying

    Returns:
        None if return_base64=False, else base64-encoded PNG string
    """
    melody = comparison_result.get('melody', {})

    # Extract melody data
    melody_score = melody.get('score', 0)
    pitch_shift = melody.get('pitch_shift_cents', 0)
    contour_similarity = melody.get('contour_similarity', 0)

    # Try to get ΔF0 from pitch analysis
    student_delta_f0 = None
    reference_delta_f0 = None
    student_times = None
    reference_times = None

    if student_pitch:
        student_delta_f0 = student_pitch.get('delta_f0')
        student_times = student_pitch.get('times')

    if reference_pitch:
        reference_delta_f0 = reference_pitch.get('delta_f0')
        reference_times = reference_pitch.get('times')

    # Create figure
    fig, axes = plt.subplots(2, 1, figsize=figsize, sharex=False)

    # Top: Overlaid ΔF0 contours
    ax_overlay = axes[0]
    ax_overlay.set_title('Melody Contour Comparison (ΔF0)', fontsize=12, fontweight='bold')
    ax_overlay.set_ylabel('ΔF0 (semitones)')
    ax_overlay.axhline(y=0, color='k', linestyle='--', alpha=0.3, linewidth=1)

    if student_delta_f0 is not None:
        # Filter out NaN/inf
        valid_mask = np.isfinite(student_delta_f0)
        delta_f0_clean = student_delta_f0[valid_mask]

        if student_times is not None:
            times_clean = student_times[valid_mask]
            ax_overlay.plot(times_clean, delta_f0_clean,
                          color='#2E86AB', linewidth=2, alpha=0.7, label='Student')
            ax_overlay.set_xlabel('Time (s)')
        else:
            ax_overlay.plot(delta_f0_clean,
                          color='#2E86AB', linewidth=2, alpha=0.7, label='Student')
            ax_overlay.set_xlabel('Frame')
    else:
        ax_overlay.text(0.5, 0.5, 'Student ΔF0 not available',
                       ha='center', va='center', transform=ax_overlay.transAxes,
                       fontsize=10, color='gray')

    if reference_delta_f0 is not None:
        # Filter out NaN/inf
        valid_mask = np.isfinite(reference_delta_f0)
        delta_f0_clean = reference_delta_f0[valid_mask]

        if reference_times is not None:
            times_clean = reference_times[valid_mask]
            ax_overlay.plot(times_clean, delta_f0_clean,
                          color='#A23B72', linewidth=2, alpha=0.7, label='Reference')
        else:
            ax_overlay.plot(delta_f0_clean,
                          color='#A23B72', linewidth=2, alpha=0.7, label='Reference')
    else:
        if student_delta_f0 is None:  # Only show if both are missing
            ax_overlay.text(0.5, 0.4, 'Reference ΔF0 not available',
                           ha='center', va='center', transform=ax_overlay.transAxes,
                           fontsize=10, color='gray')

    ax_overlay.legend(loc='upper right', fontsize=9)
    ax_overlay.grid(True, alpha=0.3)

    # Add key shift annotation
    if show_key_shift and pitch_shift != 0:
        shift_dir = '+' if pitch_shift > 0 else ''
        ax_overlay.text(0.02, 0.98,
                       f'Key Shift: {shift_dir}{pitch_shift:.0f} cents\n({shift_dir}{pitch_shift/100:.2f} semitones)',
                       transform=ax_overlay.transAxes,
                       fontsize=9,
                       verticalalignment='top',
                       bbox=dict(boxstyle='round', facecolor='lightyellow', alpha=0.7))

    # Bottom: Difference plot (student - reference)
    ax_diff = axes[1]
    ax_diff.set_title('Contour Difference (Student - Reference)', fontsize=12, fontweight='bold')
    ax_diff.set_ylabel('ΔF0 Difference (semitones)')
    ax_diff.axhline(y=0, color='k', linestyle='--', alpha=0.3, linewidth=1)

    if student_delta_f0 is not None and reference_delta_f0 is not None:
        # Align lengths (simple approach: use shorter length)
        min_len = min(len(student_delta_f0), len(reference_delta_f0))
        student_trimmed = student_delta_f0[:min_len]
        reference_trimmed = reference_delta_f0[:min_len]

        # Compute difference
        diff = student_trimmed - reference_trimmed

        # Filter out NaN/inf
        valid_mask = np.isfinite(diff)
        diff_clean = diff[valid_mask]

        if student_times is not None:
            times_clean = student_times[:min_len][valid_mask]
            ax_diff.fill_between(times_clean, 0, diff_clean,
                               where=(diff_clean >= 0),
                               color='#2E86AB', alpha=0.4, label='Student higher')
            ax_diff.fill_between(times_clean, 0, diff_clean,
                               where=(diff_clean < 0),
                               color='#A23B72', alpha=0.4, label='Reference higher')
            ax_diff.plot(times_clean, diff_clean, color='#F18F01', linewidth=1, alpha=0.8)
            ax_diff.set_xlabel('Time (s)')
        else:
            frames = np.arange(len(diff_clean))
            ax_diff.fill_between(frames, 0, diff_clean,
                               where=(diff_clean >= 0),
                               color='#2E86AB', alpha=0.4, label='Student higher')
            ax_diff.fill_between(frames, 0, diff_clean,
                               where=(diff_clean < 0),
                               color='#A23B72', alpha=0.4, label='Reference higher')
            ax_diff.plot(frames, diff_clean, color='#F18F01', linewidth=1, alpha=0.8)
            ax_diff.set_xlabel('Frame')

        ax_diff.legend(loc='upper right', fontsize=9)
    else:
        ax_diff.text(0.5, 0.5, 'Cannot compute difference (missing data)',
                    ha='center', va='center', transform=ax_diff.transAxes,
                    fontsize=10, color='gray')

    ax_diff.grid(True, alpha=0.3)

    # Add score annotation
    if show_similarity:
        score_text = f'Melody Score: {melody_score:.1f}/100\n'
        score_text += f'Contour Similarity: {contour_similarity:.1f}%'

        ax_diff.text(0.98, 0.98,
                    score_text,
                    transform=ax_diff.transAxes,
                    fontsize=10,
                    verticalalignment='top',
                    horizontalalignment='right',
                    bbox=dict(boxstyle='round', facecolor='wheat', alpha=0.5))

    plt.suptitle('Melody Analysis: Pitch Contour Comparison', fontsize=14, fontweight='bold', y=0.98)

    if return_base64:
        # Save to base64
        buf = io.BytesIO()
        plt.savefig(buf, format='png', dpi=150, bbox_inches='tight')
        buf.seek(0)
        img_base64 = base64.b64encode(buf.read()).decode('utf-8')
        plt.close(fig)
        return img_base64
    else:
        plt.tight_layout()
        return fig


def create_melody_contour_dict(comparison_result: Dict, **kwargs) -> Dict:
    """
    Create a dictionary with melody contour visualization data.

    Returns dict with 'image_base64' key containing the plot.
    """
    img_base64 = plot_melody_contour(comparison_result, return_base64=True, **kwargs)

    melody = comparison_result.get('melody', {})

    return {
        'image_base64': img_base64,
        'melody_score': melody.get('score', 0),
        'pitch_shift_cents': melody.get('pitch_shift_cents', 0),
        'contour_similarity': melody.get('contour_similarity', 0),
    }
