"""
Madd Duration Bars Visualization - Shows expected vs actual elongations.

Displays:
- Bar chart comparing expected vs held counts for each Madd type (2/4/6)
- Variance bands showing acceptable range
- Critical shortfalls highlighted
- Individual Madd events timeline
"""
import numpy as np
import matplotlib.pyplot as plt
from typing import Dict, List, Optional, Tuple
import io
import base64


def plot_duration_bars(
    comparison_result: Dict,
    show_variance_bands: bool = True,
    show_timeline: bool = True,
    figsize: Tuple[int, int] = (12, 8),
    return_base64: bool = False
):
    """
    Plot Madd duration comparison bars.

    Args:
        comparison_result: Result from compare_recitations() containing duration data
        show_variance_bands: Whether to show acceptable variance bands
        show_timeline: Whether to show timeline of individual Madd events
        figsize: Figure size (width, height)
        return_base64: If True, return base64-encoded PNG instead of displaying

    Returns:
        None if return_base64=False, else base64-encoded PNG string
    """
    durations = comparison_result.get('durations', {})

    # Extract duration data
    overall_score = durations.get('overall', 0)
    by_type = durations.get('by_type', {})
    events = durations.get('events', [])

    if not by_type and not events:
        # No duration data available
        fig, ax = plt.subplots(figsize=(8, 6))
        ax.text(0.5, 0.5, 'No Madd duration data available',
               ha='center', va='center', fontsize=12, color='gray')
        ax.axis('off')

        if return_base64:
            buf = io.BytesIO()
            plt.savefig(buf, format='png', dpi=150, bbox_inches='tight')
            buf.seek(0)
            img_base64 = base64.b64encode(buf.read()).decode('utf-8')
            plt.close(fig)
            return img_base64
        return fig

    # Create figure with subplots
    n_subplots = 2 if show_timeline and events else 1
    fig, axes = plt.subplots(n_subplots, 1, figsize=figsize,
                            gridspec_kw={'height_ratios': [2, 1] if n_subplots == 2 else [1]})

    if n_subplots == 1:
        axes = [axes]

    # Top: Bar chart by Madd type
    ax_bars = axes[0]
    ax_bars.set_title('Madd Duration Comparison (Expected vs Actual)', fontsize=12, fontweight='bold')
    ax_bars.set_ylabel('Duration (counts)')

    # Prepare data for bar chart
    madd_types = []
    expected_values = []
    actual_values = []
    scores = []
    critical_flags = []

    for madd_type in ['2', '4', '6']:
        if madd_type in by_type:
            data = by_type[madd_type]
            madd_types.append(f'Madd {madd_type}')
            expected_values.append(data.get('expected', 0))
            actual_values.append(data.get('actual', 0))
            scores.append(data.get('score', 0))
            critical_flags.append(data.get('critical', False))

    if not madd_types:
        ax_bars.text(0.5, 0.5, 'No Madd events detected',
                    ha='center', va='center', transform=ax_bars.transAxes,
                    fontsize=10, color='gray')
    else:
        x = np.arange(len(madd_types))
        width = 0.35

        # Plot bars
        bars_expected = ax_bars.bar(x - width/2, expected_values, width,
                                    label='Expected (Reference)', color='#A23B72', alpha=0.7)
        bars_actual = ax_bars.bar(x + width/2, actual_values, width,
                                  label='Actual (Student)', color='#2E86AB', alpha=0.7)

        # Highlight critical shortfalls
        for i, (is_critical, score) in enumerate(zip(critical_flags, scores)):
            if is_critical:
                # Add red border to actual bar
                bars_actual[i].set_edgecolor('red')
                bars_actual[i].set_linewidth(3)

                # Add warning marker
                ax_bars.text(x[i] + width/2, actual_values[i] + 0.1,
                           '⚠️', ha='center', fontsize=16)

        # Add variance bands if requested
        if show_variance_bands:
            for i, expected in enumerate(expected_values):
                # Acceptable variance: ±15% of expected (from spec: σ = 0.15 × expected)
                variance = 0.15 * expected
                lower = max(0, expected - variance)
                upper = expected + variance

                # Draw band
                ax_bars.fill_between([x[i] - width*1.5, x[i] + width*1.5],
                                    lower, upper,
                                    alpha=0.15, color='green',
                                    label='Acceptable range' if i == 0 else '')

        # Add value labels on bars
        for i, (exp, act) in enumerate(zip(expected_values, actual_values)):
            ax_bars.text(x[i] - width/2, exp + 0.05, f'{exp:.1f}',
                        ha='center', va='bottom', fontsize=9)
            ax_bars.text(x[i] + width/2, act + 0.05, f'{act:.1f}',
                        ha='center', va='bottom', fontsize=9)

        ax_bars.set_xticks(x)
        ax_bars.set_xticklabels(madd_types)
        ax_bars.legend(loc='upper left', fontsize=9)
        ax_bars.grid(True, alpha=0.3, axis='y')

        # Add score annotation
        ax_bars.text(0.98, 0.98,
                    f'Duration Score: {overall_score:.1f}/100',
                    transform=ax_bars.transAxes,
                    fontsize=10,
                    verticalalignment='top',
                    horizontalalignment='right',
                    bbox=dict(boxstyle='round', facecolor='wheat', alpha=0.5))

    # Bottom: Timeline of individual Madd events
    if show_timeline and events:
        ax_timeline = axes[1]
        ax_timeline.set_title('Individual Madd Events Timeline', fontsize=11, fontweight='bold')
        ax_timeline.set_xlabel('Time (s)')
        ax_timeline.set_ylabel('Madd Type')

        # Extract event data
        event_times = []
        event_types = []
        event_expected = []
        event_actual = []
        event_critical = []

        for event in events:
            event_times.append(event.get('time', 0))
            event_types.append(event.get('type', '?'))
            event_expected.append(event.get('expected', 0))
            event_actual.append(event.get('actual', 0))
            event_critical.append(event.get('critical', False))

        # Create categorical y-axis
        unique_types = sorted(set(event_types), key=lambda x: int(x) if x.isdigit() else 0)
        type_to_y = {t: i for i, t in enumerate(unique_types)}

        # Plot events
        for time, typ, exp, act, crit in zip(event_times, event_types, event_expected,
                                             event_actual, event_critical):
            y = type_to_y[typ]

            # Color based on accuracy
            ratio = act / exp if exp > 0 else 0
            if ratio >= 0.85 and ratio <= 1.15:  # Within 15%
                color = 'green'
                marker = 'o'
            elif crit:
                color = 'red'
                marker = 'x'
            else:
                color = 'orange'
                marker = 's'

            ax_timeline.scatter(time, y, c=color, marker=marker, s=100, alpha=0.7, edgecolors='black')

            # Add label with counts
            ax_timeline.text(time, y + 0.15, f'{act:.1f}/{exp:.1f}',
                           ha='center', fontsize=7, bbox=dict(boxstyle='round', facecolor='white', alpha=0.7))

        ax_timeline.set_yticks(range(len(unique_types)))
        ax_timeline.set_yticklabels([f'Madd {t}' for t in unique_types])
        ax_timeline.grid(True, alpha=0.3, axis='x')

        # Add legend
        from matplotlib.lines import Line2D
        legend_elements = [
            Line2D([0], [0], marker='o', color='w', markerfacecolor='green', markersize=8,
                  label='Accurate (±15%)', markeredgecolor='black'),
            Line2D([0], [0], marker='s', color='w', markerfacecolor='orange', markersize=8,
                  label='Minor error', markeredgecolor='black'),
            Line2D([0], [0], marker='x', color='w', markerfacecolor='red', markersize=8,
                  label='Critical (>0.5 count off)', markeredgecolor='red', markeredgewidth=2)
        ]
        ax_timeline.legend(handles=legend_elements, loc='upper right', fontsize=8)

    plt.suptitle('Duration Analysis: Madd Elongations', fontsize=14, fontweight='bold', y=0.98)

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


def create_duration_bars_dict(comparison_result: Dict, **kwargs) -> Dict:
    """
    Create a dictionary with duration bars visualization data.

    Returns dict with 'image_base64' key containing the plot.
    """
    img_base64 = plot_duration_bars(comparison_result, return_base64=True, **kwargs)

    durations = comparison_result.get('durations', {})

    return {
        'image_base64': img_base64,
        'overall_score': durations.get('overall', 0),
        'by_type': durations.get('by_type', {}),
        'num_events': len(durations.get('events', [])),
    }
