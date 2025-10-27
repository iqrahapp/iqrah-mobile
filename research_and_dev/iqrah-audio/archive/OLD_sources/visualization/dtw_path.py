"""
DTW Path Visualization - Shows where timing diverges between student and reference.

Displays:
- Onset strength heatmap for both student and reference
- DTW alignment path overlay
- Regions of timing divergence (highlighted)
- Syllable boundaries
"""
import numpy as np
import matplotlib.pyplot as plt
import matplotlib.patches as patches
from typing import Dict, List, Optional, Tuple
import io
import base64


def plot_dtw_path(
    comparison_result: Dict,
    student_features: Optional[np.ndarray] = None,
    reference_features: Optional[np.ndarray] = None,
    show_syllables: bool = True,
    highlight_divergence: bool = True,
    divergence_threshold: float = 0.5,
    figsize: Tuple[int, int] = (12, 8),
    return_base64: bool = False
):
    """
    Plot DTW alignment path over onset grid.

    Args:
        comparison_result: Result from compare_recitations() containing rhythm data
        student_features: Optional [T_student, D] feature array
        reference_features: Optional [T_reference, D] feature array
        show_syllables: Whether to show syllable boundaries
        highlight_divergence: Whether to highlight high-divergence regions
        divergence_threshold: Threshold for divergence highlighting (0-1)
        figsize: Figure size (width, height)
        return_base64: If True, return base64-encoded PNG instead of displaying

    Returns:
        None if return_base64=False, else base64-encoded PNG string
    """
    rhythm = comparison_result.get('rhythm', {})

    # Extract DTW path
    path = rhythm.get('path')
    if path is None or len(path) == 0:
        raise ValueError("No DTW path found in comparison result")

    path = np.array(path)  # [N, 2] where path[:, 0] = student, path[:, 1] = reference

    # Get frame times if available
    student_times = rhythm.get('student_frame_times')
    reference_times = rhythm.get('reference_frame_times')

    T_student = path[:, 0].max() + 1
    T_reference = path[:, 1].max() + 1

    # Create figure with subplots
    fig = plt.figure(figsize=figsize)
    gs = fig.add_gridspec(3, 2, height_ratios=[1, 1, 2], hspace=0.3, wspace=0.3)

    # Top: Student onset strength
    ax_student = fig.add_subplot(gs[0, :])
    ax_student.set_title('Student Onset Strength', fontsize=12, fontweight='bold')
    ax_student.set_ylabel('Strength')

    if student_features is not None and student_features.shape[1] > 0:
        onset_strength = student_features[:, 0]  # First channel is onset strength
        if student_times is not None:
            ax_student.plot(student_times, onset_strength, color='#2E86AB', linewidth=1.5)
            ax_student.fill_between(student_times, 0, onset_strength, alpha=0.3, color='#2E86AB')
            ax_student.set_xlabel('Time (s)')
        else:
            ax_student.plot(onset_strength, color='#2E86AB', linewidth=1.5)
            ax_student.fill_between(range(len(onset_strength)), 0, onset_strength, alpha=0.3, color='#2E86AB')
            ax_student.set_xlabel('Frame')
    else:
        ax_student.text(0.5, 0.5, 'Student features not available',
                       ha='center', va='center', transform=ax_student.transAxes)

    ax_student.grid(True, alpha=0.3)

    # Middle: Reference onset strength
    ax_reference = fig.add_subplot(gs[1, :])
    ax_reference.set_title('Reference Onset Strength', fontsize=12, fontweight='bold')
    ax_reference.set_ylabel('Strength')

    if reference_features is not None and reference_features.shape[1] > 0:
        onset_strength = reference_features[:, 0]
        if reference_times is not None:
            ax_reference.plot(reference_times, onset_strength, color='#A23B72', linewidth=1.5)
            ax_reference.fill_between(reference_times, 0, onset_strength, alpha=0.3, color='#A23B72')
            ax_reference.set_xlabel('Time (s)')
        else:
            ax_reference.plot(onset_strength, color='#A23B72', linewidth=1.5)
            ax_reference.fill_between(range(len(onset_strength)), 0, onset_strength, alpha=0.3, color='#A23B72')
            ax_reference.set_xlabel('Frame')
    else:
        ax_reference.text(0.5, 0.5, 'Reference features not available',
                         ha='center', va='center', transform=ax_reference.transAxes)

    ax_reference.grid(True, alpha=0.3)

    # Bottom: DTW alignment path
    ax_dtw = fig.add_subplot(gs[2, :])
    ax_dtw.set_title('DTW Alignment Path (shows timing correspondence)',
                     fontsize=12, fontweight='bold')
    ax_dtw.set_xlabel('Student Frames')
    ax_dtw.set_ylabel('Reference Frames')

    # Plot perfect diagonal (ideal alignment)
    max_frames = max(T_student, T_reference)
    ax_dtw.plot([0, max_frames], [0, max_frames], 'k--', alpha=0.3, linewidth=1, label='Perfect alignment')

    # Compute local divergence along path (how far from diagonal)
    if highlight_divergence:
        # Normalize path to [0, 1]
        path_norm = path.astype(float)
        path_norm[:, 0] /= (T_student - 1) if T_student > 1 else 1
        path_norm[:, 1] /= (T_reference - 1) if T_reference > 1 else 1

        # Divergence from diagonal
        local_divergence = np.abs(path_norm[:, 0] - path_norm[:, 1])

        # Color path by divergence
        scatter = ax_dtw.scatter(path[:, 0], path[:, 1],
                               c=local_divergence,
                               cmap='RdYlGn_r',
                               s=10,
                               alpha=0.6,
                               vmin=0,
                               vmax=divergence_threshold)

        # Add colorbar
        cbar = plt.colorbar(scatter, ax=ax_dtw)
        cbar.set_label('Timing Divergence', rotation=270, labelpad=15)

        # Highlight high-divergence regions
        high_div_indices = np.where(local_divergence > divergence_threshold)[0]
        if len(high_div_indices) > 0:
            ax_dtw.scatter(path[high_div_indices, 0], path[high_div_indices, 1],
                         color='red', s=30, marker='x', linewidths=2,
                         label=f'High divergence (>{divergence_threshold:.1f})')
    else:
        # Simple path plot
        ax_dtw.plot(path[:, 0], path[:, 1], 'o-', color='#F18F01',
                   markersize=3, linewidth=1, alpha=0.6, label='DTW path')

    ax_dtw.set_xlim(0, T_student)
    ax_dtw.set_ylim(0, T_reference)
    ax_dtw.legend(loc='upper left', fontsize=9)
    ax_dtw.grid(True, alpha=0.3)
    ax_dtw.set_aspect('equal', adjustable='box')

    # Add score annotation
    rhythm_score = rhythm.get('score', 0)
    divergence_val = rhythm.get('divergence', 0)
    ax_dtw.text(0.98, 0.02,
               f'Rhythm Score: {rhythm_score:.1f}/100\nDivergence: {divergence_val:.2f}',
               transform=ax_dtw.transAxes,
               fontsize=10,
               verticalalignment='bottom',
               horizontalalignment='right',
               bbox=dict(boxstyle='round', facecolor='wheat', alpha=0.5))

    plt.suptitle('Rhythm Analysis: DTW Alignment Path', fontsize=14, fontweight='bold', y=0.98)

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


def create_dtw_path_dict(comparison_result: Dict, **kwargs) -> Dict:
    """
    Create a dictionary with DTW path visualization data.

    Returns dict with 'image_base64' key containing the plot.
    """
    img_base64 = plot_dtw_path(comparison_result, return_base64=True, **kwargs)

    return {
        'image_base64': img_base64,
        'rhythm_score': comparison_result.get('rhythm', {}).get('score', 0),
        'divergence': comparison_result.get('rhythm', {}).get('divergence', 0),
        'path_length': len(comparison_result.get('rhythm', {}).get('path', [])),
    }
