"""
Fusion Module - Overall Scoring
================================

Combines rhythm, melody, duration, and pronunciation scores into overall assessment.
"""

import numpy as np
from typing import Dict, Optional


# Cold-start weights (can be learned later)
DEFAULT_WEIGHTS = {
    'rhythm': 0.30,
    'melody': 0.20,
    'duration': 0.30,
    'pronunciation': 0.20
}


def compute_overall_score(
    component_scores: Dict,
    weights: Optional[Dict] = None,
    bootstrap_uncertainty: bool = False
) -> Dict:
    """
    Compute overall recitation score from components.

    Args:
        component_scores: Dictionary with keys 'rhythm', 'melody', 'duration', 'pronunciation'
        weights: Custom weights (defaults to cold-start if None)
        bootstrap_uncertainty: Whether to compute confidence intervals

    Returns:
        Dictionary with:
            - overall: Overall score 0-100
            - confidence: Confidence level 0-1
            - breakdown: Component scores
            - top_issues: Top 3 areas needing improvement
    """
    if weights is None:
        weights = DEFAULT_WEIGHTS.copy()

    # Normalize weights to sum to 1
    total_weight = sum(weights.values())
    weights = {k: v / total_weight for k, v in weights.items()}

    # Compute weighted average
    overall = 0.0
    breakdown = {}

    for component, weight in weights.items():
        if component in component_scores:
            score = component_scores[component].get('score', 0)
            overall += weight * score
            breakdown[component] = {
                'score': score,
                'weight': weight,
                'contribution': weight * score
            }

    # Compute confidence (simple version: based on score variance)
    score_values = [cs.get('score', 0) for cs in component_scores.values()]
    score_std = np.std(score_values)

    # High variance = low confidence
    confidence = 1.0 - min(score_std / 100.0, 0.5)

    # Identify top issues (lowest scoring components)
    sorted_components = sorted(
        breakdown.items(),
        key=lambda x: x[1]['score']
    )

    top_issues = []
    for comp, details in sorted_components[:3]:
        if details['score'] < 85:
            top_issues.append({
                'component': comp,
                'score': details['score'],
                'priority': get_priority_label(details['score'])
            })

    return {
        'overall': round(overall, 1),
        'confidence': round(confidence, 2),
        'breakdown': breakdown,
        'top_issues': top_issues
    }


def get_priority_label(score: float) -> str:
    """Get priority label based on score."""
    if score < 60:
        return 'critical'
    elif score < 75:
        return 'high'
    else:
        return 'medium'


def aggregate_feedback_notes(component_scores: Dict) -> list:
    """
    Aggregate all feedback notes from components into hierarchical list.

    Prioritizes: Critical (tajweed/duration) → Timing (rhythm) → Style (melody)

    Returns:
        List of feedback strings, ordered by priority
    """
    all_notes = []

    # Critical: duration issues
    if 'duration' in component_scores:
        duration_notes = component_scores['duration'].get('notes', [])
        critical_issues = component_scores['duration'].get('critical_issues', [])

        if critical_issues:
            all_notes.append({
                'priority': 1,
                'category': 'Elongation (Critical)',
                'text': f"{len(critical_issues)} critical Madd shortfalls"
            })

        for note in duration_notes:
            all_notes.append({
                'priority': 1 if 'critical' in note.lower() else 2,
                'category': 'Elongation',
                'text': note
            })

    # Timing: rhythm
    if 'rhythm' in component_scores:
        rhythm_notes = component_scores['rhythm'].get('notes', [])
        for note in rhythm_notes:
            all_notes.append({
                'priority': 2,
                'category': 'Rhythm',
                'text': note
            })

    # Style: melody
    if 'melody' in component_scores:
        melody_notes = component_scores['melody'].get('notes', [])
        for note in melody_notes:
            all_notes.append({
                'priority': 3,
                'category': 'Melody',
                'text': note
            })

    # Pronunciation (if available)
    if 'pronunciation' in component_scores:
        pron_notes = component_scores['pronunciation'].get('notes', [])
        for note in pron_notes:
            all_notes.append({
                'priority': 1,
                'category': 'Pronunciation',
                'text': note
            })

    # Sort by priority
    all_notes.sort(key=lambda x: x['priority'])

    return all_notes


def generate_improvement_suggestions(
    component_scores: Dict,
    top_issues: list
) -> list:
    """
    Generate actionable improvement suggestions based on component scores.

    Returns:
        List of suggestion strings
    """
    suggestions = []

    for issue in top_issues:
        component = issue['component']
        score = issue['score']

        if component == 'rhythm' and score < 75:
            suggestions.append("Practice with a metronome to improve timing consistency")
            suggestions.append("Focus on maintaining steady pace throughout the ayah")

        elif component == 'melody' and score < 75:
            suggestions.append("Listen repeatedly to reference and try to match pitch contour")
            suggestions.append("Practice pitch transitions between words")

        elif component == 'duration' and score < 75:
            # Get specific Madd issues
            if 'duration' in component_scores:
                critical = component_scores['duration'].get('critical_issues', [])
                if critical:
                    counts = set(c['expected'] for c in critical)
                    for count in counts:
                        suggestions.append(f"Practice holding {count}-count Madds longer - use a timer")

        elif component == 'pronunciation' and score < 75:
            suggestions.append("Review pronunciation rules for problematic phonemes")
            suggestions.append("Practice difficult sounds in isolation before full recitation")

    return suggestions
