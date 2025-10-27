"""
Pronunciation Timeline Visualization - Shows phoneme-level errors with highlights.

Displays:
- Timeline of phonemes with GOP scores
- Error highlights (mild vs severe)
- Confusion details with Arabic letters
- Articulation tips for specific errors
"""
import numpy as np
import matplotlib.pyplot as plt
from typing import Dict, List, Optional, Tuple
import io
import base64


def plot_pronunciation_timeline(
    comparison_result: Dict,
    show_confusions: bool = True,
    show_tips: bool = True,
    max_confusions: int = 10,
    figsize: Tuple[int, int] = (14, 8),
    return_base64: bool = False
):
    """
    Plot pronunciation timeline with error highlights.

    Args:
        comparison_result: Result from compare_recitations() containing pronunciation data
        show_confusions: Whether to show detailed confusion annotations
        show_tips: Whether to show articulation tips
        max_confusions: Maximum number of confusions to annotate
        figsize: Figure size (width, height)
        return_base64: If True, return base64-encoded PNG instead of displaying

    Returns:
        None if return_base64=False, else base64-encoded PNG string
    """
    pronunciation = comparison_result.get('pronunciation', {})

    # Extract pronunciation data
    overall_score = pronunciation.get('score', 0)
    phone_scores = pronunciation.get('phone_scores', [])
    confusions = pronunciation.get('confusions', [])
    critical_errors = pronunciation.get('critical_errors', [])

    if not phone_scores:
        # No pronunciation data available
        fig, ax = plt.subplots(figsize=(8, 6))
        ax.text(0.5, 0.5, 'No pronunciation data available',
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
    n_subplots = 2 if show_confusions and confusions else 1
    fig, axes = plt.subplots(n_subplots, 1, figsize=figsize,
                            gridspec_kw={'height_ratios': [2, 1] if n_subplots == 2 else [1]})

    if n_subplots == 1:
        axes = [axes]

    # Top: GOP scores timeline
    ax_timeline = axes[0]
    ax_timeline.set_title('Pronunciation Quality Timeline (GOP Scores)', fontsize=12, fontweight='bold')
    ax_timeline.set_xlabel('Phoneme Index')
    ax_timeline.set_ylabel('GOP Score (higher = better)')

    # Extract GOP data
    phoneme_indices = []
    gop_scores = []
    severities = []
    phoneme_chars = []
    positions = []

    for i, phone in enumerate(phone_scores):
        phoneme_indices.append(i)
        gop_scores.append(phone.get('gop_mean', 0))
        severities.append(phone.get('severity', 'ok'))
        phoneme_chars.append(phone.get('char', '?'))
        positions.append(phone.get('start', 0))

    # Plot GOP scores with color-coding by severity
    severity_colors = {
        'ok': '#2ECC71',      # Green
        'mild': '#F39C12',    # Orange
        'severe': '#E74C3C'   # Red
    }

    for severity_type in ['ok', 'mild', 'severe']:
        mask = [s == severity_type for s in severities]
        indices = [idx for idx, m in zip(phoneme_indices, mask) if m]
        scores = [score for score, m in zip(gop_scores, mask) if m]

        if indices:
            ax_timeline.scatter(indices, scores, c=severity_colors[severity_type],
                              label=severity_type.capitalize(), s=60, alpha=0.7,
                              edgecolors='black', linewidths=0.5)

    # Connect with line
    ax_timeline.plot(phoneme_indices, gop_scores, color='gray', linewidth=1, alpha=0.3, zorder=0)

    # Add threshold lines
    ax_timeline.axhline(y=-2.0, color='orange', linestyle='--', alpha=0.5, linewidth=1,
                       label='Mild threshold (-2.0)')
    ax_timeline.axhline(y=-4.0, color='red', linestyle='--', alpha=0.5, linewidth=1,
                       label='Severe threshold (-4.0)')
    ax_timeline.axhline(y=0, color='gray', linestyle='-', alpha=0.3, linewidth=1)

    # Annotate critical phonemes
    for i, (idx, char, sev, gop) in enumerate(zip(phoneme_indices, phoneme_chars, severities, gop_scores)):
        if sev in ['mild', 'severe'] and i < max_confusions:
            # Add phoneme label
            ax_timeline.annotate(char,
                               xy=(idx, gop),
                               xytext=(0, -15 if gop > 0 else 15),
                               textcoords='offset points',
                               ha='center',
                               fontsize=8,
                               bbox=dict(boxstyle='round,pad=0.3', facecolor='yellow', alpha=0.7),
                               arrowprops=dict(arrowstyle='->', color='black', lw=0.5))

    ax_timeline.legend(loc='lower left', fontsize=9)
    ax_timeline.grid(True, alpha=0.3)

    # Add score annotation
    ax_timeline.text(0.98, 0.98,
                    f'Pronunciation Score: {overall_score:.1f}/100\n'
                    f'Confusions: {len(confusions)} detected\n'
                    f'Critical: {len(critical_errors)}',
                    transform=ax_timeline.transAxes,
                    fontsize=10,
                    verticalalignment='top',
                    horizontalalignment='right',
                    bbox=dict(boxstyle='round', facecolor='wheat', alpha=0.5))

    # Bottom: Confusion details table
    if show_confusions and confusions:
        ax_confusions = axes[1]
        ax_confusions.axis('off')
        ax_confusions.set_title('Detected Confusions (Top Errors)', fontsize=11, fontweight='bold', pad=10)

        # Prepare table data
        table_data = []
        headers = ['Time', 'Expected', 'Produced', 'Severity', 'GOP']

        for i, conf in enumerate(confusions[:max_confusions]):
            time_str = f"{conf.get('position', 0):.2f}s"
            target = conf.get('target_char', '?')
            target_ar = conf.get('target_arabic', '')
            produced = conf.get('likely_produced', '?')
            produced_ar = conf.get('likely_produced_arabic', '')
            severity = conf.get('severity', 'ok').upper()
            gop = f"{conf.get('gop_score', 0):.2f}"

            # Format with Arabic letters
            expected_str = f"{target}\n({target_ar})" if target_ar else target
            produced_str = f"{produced}\n({produced_ar})" if produced_ar else produced

            # Add severity emoji
            severity_emoji = {
                'OK': '✅',
                'MILD': '⚠️',
                'SEVERE': '🚨'
            }
            severity_str = f"{severity_emoji.get(severity, '')} {severity}"

            table_data.append([time_str, expected_str, produced_str, severity_str, gop])

        # Create table
        if table_data:
            cell_colors = []
            for row in table_data:
                severity = row[3]
                if '🚨' in severity:
                    row_color = ['#FFCCCC'] * 5  # Light red
                elif '⚠️' in severity:
                    row_color = ['#FFE6CC'] * 5  # Light orange
                else:
                    row_color = ['#E6FFE6'] * 5  # Light green
                cell_colors.append(row_color)

            table = ax_confusions.table(cellText=table_data,
                                       colLabels=headers,
                                       cellLoc='center',
                                       loc='center',
                                       cellColours=cell_colors,
                                       colColours=['#CCCCCC'] * 5)

            table.auto_set_font_size(False)
            table.set_fontsize(9)
            table.scale(1, 2)

            # Style header
            for i in range(len(headers)):
                table[(0, i)].set_facecolor('#4472C4')
                table[(0, i)].set_text_props(weight='bold', color='white')

        # Add tips if requested
        if show_tips and confusions:
            tips_text = "💡 ARTICULATION TIPS:\n"
            shown_tips = set()

            for conf in confusions[:3]:  # Show tips for top 3 confusions
                details = conf.get('confusion_details', {})
                tip = details.get('tip', '')
                if tip and tip not in shown_tips:
                    tips_text += f"• {tip[:100]}...\n" if len(tip) > 100 else f"• {tip}\n"
                    shown_tips.add(tip)

            if shown_tips:
                ax_confusions.text(0.5, -0.1,
                                 tips_text,
                                 transform=ax_confusions.transAxes,
                                 fontsize=8,
                                 verticalalignment='top',
                                 horizontalalignment='center',
                                 bbox=dict(boxstyle='round', facecolor='lightyellow', alpha=0.8),
                                 wrap=True)

    plt.suptitle('Pronunciation Analysis: Phoneme-Level Assessment', fontsize=14, fontweight='bold', y=0.98)

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


def create_pronunciation_timeline_dict(comparison_result: Dict, **kwargs) -> Dict:
    """
    Create a dictionary with pronunciation timeline visualization data.

    Returns dict with 'image_base64' key containing the plot.
    """
    img_base64 = plot_pronunciation_timeline(comparison_result, return_base64=True, **kwargs)

    pronunciation = comparison_result.get('pronunciation', {})

    return {
        'image_base64': img_base64,
        'pronunciation_score': pronunciation.get('score', 0),
        'num_confusions': len(pronunciation.get('confusions', [])),
        'num_critical': len(pronunciation.get('critical_errors', [])),
    }
