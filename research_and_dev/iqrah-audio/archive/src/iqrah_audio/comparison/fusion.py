"""
Fusion Module - Overall Scoring & Explainability
=================================================

Combines rhythm, melody, duration, and pronunciation scores into overall assessment
with confidence estimation, top-issue identification, and actionable feedback.

Based on Phase-2 spec section 7.
"""

import numpy as np
from typing import Dict, Optional, List
from scipy import stats


# Cold-start weights (can be learned later via ordinal regression)
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

    # Compute confidence via bootstrap (if granular data available)
    confidence = estimate_confidence_bootstrap(component_scores, bootstrap_uncertainty)

    # Identify top issues with hierarchical feedback
    top_issues = identify_top_issues(component_scores, weights)

    return {
        'overall': round(overall, 1),
        'confidence': round(confidence, 2),
        'breakdown': breakdown,
        'top_issues': top_issues
    }


def estimate_confidence_bootstrap(
    component_scores: Dict,
    enabled: bool = False,
    n_bootstrap: int = 100
) -> float:
    """
    Estimate confidence via bootstrap over segments.

    If component scores contain per-segment/per-item data, resample and
    recompute to get confidence interval width.

    Args:
        component_scores: Component score dictionaries
        enabled: Whether to run bootstrap (computationally expensive)
        n_bootstrap: Number of bootstrap samples

    Returns:
        confidence: 0-1 (1 = high confidence, 0 = low confidence)
    """
    if not enabled:
        # Fast fallback: confidence based on score variance
        score_values = [cs.get('score', 0) for cs in component_scores.values()]
        score_std = np.std(score_values)
        # High variance = low confidence
        confidence = 1.0 - min(score_std / 100.0, 0.5)
        return confidence

    # Extract per-segment scores if available
    segment_data = {}

    # Duration: per-phoneme scores
    if 'duration' in component_scores and 'details' in component_scores['duration']:
        duration_items = [
            d.get('score', 100)
            for d in component_scores['duration']['details']
            if 'score' in d
        ]
        if duration_items:
            segment_data['duration'] = duration_items

    # Pronunciation: per-phone GOP scores
    if 'pronunciation' in component_scores and 'phone_scores' in component_scores['pronunciation']:
        pron_items = [
            100 * np.exp(s.get('gop_mean', -2) / 2.0)
            for s in component_scores['pronunciation']['phone_scores']
        ]
        if pron_items:
            segment_data['pronunciation'] = pron_items

    # If no granular data, return variance-based confidence
    if not segment_data:
        score_values = [cs.get('score', 0) for cs in component_scores.values()]
        score_std = np.std(score_values)
        confidence = 1.0 - min(score_std / 100.0, 0.5)
        return confidence

    # Bootstrap over segments
    bootstrap_overalls = []
    weights = DEFAULT_WEIGHTS.copy()

    for _ in range(n_bootstrap):
        # Resample each component's segments
        resampled_scores = {}

        for comp, score_dict in component_scores.items():
            if comp in segment_data:
                # Resample with replacement
                items = segment_data[comp]
                n_samples = max(1, int(len(items) * 0.8))
                resampled = np.random.choice(items, size=n_samples, replace=True)
                resampled_scores[comp] = {'score': float(np.mean(resampled))}
            else:
                # Use original score
                resampled_scores[comp] = {'score': score_dict.get('score', 0)}

        # Compute overall for this bootstrap sample
        overall = sum(
            weights.get(comp, 0) * scores['score']
            for comp, scores in resampled_scores.items()
        )
        bootstrap_overalls.append(overall)

    # Confidence from coefficient of variation
    mean_overall = np.mean(bootstrap_overalls)
    std_overall = np.std(bootstrap_overalls)
    cv = std_overall / (mean_overall + 1e-6)

    # Map CV to confidence: CV < 0.1 → confidence > 0.9
    confidence = float(np.clip(1.0 - cv / 0.2, 0.0, 1.0))

    return confidence


def identify_top_issues(
    component_scores: Dict,
    weights: Dict[str, float]
) -> List[Dict]:
    """
    Identify top 3 contributing issues for pedagogical feedback.

    Issues are ranked by:
    1. Severity (critical Tajweed/pronunciation > timing > style)
    2. Impact on overall score (weight × (100 - component_score))

    Returns:
        List of top 3 issues with actionable feedback
    """
    issues = []

    # Duration issues (critical - Tajweed rule violations)
    if 'duration' in component_scores:
        duration_data = component_scores['duration']
        if 'details' in duration_data:
            for d in duration_data['details']:
                if d.get('score', 100) < 70:  # Significant error
                    impact = weights['duration'] * (100 - d['score'])
                    issues.append({
                        'category': 'critical',
                        'component': 'duration',
                        'impact': impact,
                        'score': d['score'],
                        'message': f"Madd elongation error in '{d.get('phoneme', '?')}': "
                                   f"expected {d.get('expected_counts', 0):.1f} counts, "
                                   f"held {d.get('observed_counts', 0):.1f} counts",
                        'phoneme': d.get('phoneme', '?'),
                        'position': d.get('start', 0),
                        'tajweed_rule': d.get('tajweed_rule', 'madda')
                    })

    # Pronunciation issues (critical - mispronunciation)
    if 'pronunciation' in component_scores:
        pron_data = component_scores['pronunciation']
        if 'confusions' in pron_data:
            for conf in pron_data['confusions']:
                if conf.get('severity') in ['severe', 'mild']:
                    impact = weights['pronunciation'] * abs(conf.get('gop_score', 0)) * 20
                    issues.append({
                        'category': 'critical',
                        'component': 'pronunciation',
                        'impact': impact,
                        'score': 100 * np.exp(conf.get('gop_score', -2) / 2.0),
                        'message': f"Pronunciation error at {conf.get('position', 0):.2f}s: "
                                   f"'{conf.get('target_char', '?')}' → '{conf.get('likely_produced', '?')}'",
                        'confusion_type': conf.get('confusion_type'),
                        'severity': conf.get('severity', 'mild'),
                        'tajweed_feedback': get_tajweed_feedback_for_confusion(conf)
                    })

    # Rhythm issues (timing)
    rhythm_score = component_scores.get('rhythm', {}).get('score', 100)
    if rhythm_score < 80:
        impact = weights['rhythm'] * (100 - rhythm_score)
        rhythm_details = component_scores.get('rhythm', {})
        issues.append({
            'category': 'timing',
            'component': 'rhythm',
            'impact': impact,
            'score': rhythm_score,
            'message': f"Rhythm divergence: {rhythm_score:.1f}/100. "
                       f"Timing pattern differs from reference.",
            'divergence': rhythm_details.get('divergence', 0),
            'notes': rhythm_details.get('notes', [])
        })

    # Melody issues (style)
    melody_score = component_scores.get('melody', {}).get('score', 100)
    if melody_score < 70:
        impact = weights['melody'] * (100 - melody_score)
        melody_details = component_scores.get('melody', {})
        pitch_shift = melody_details.get('pitch_shift_cents', 0)
        issues.append({
            'category': 'style',
            'component': 'melody',
            'impact': impact,
            'score': melody_score,
            'message': f"Melody contour: {melody_score:.1f}/100. "
                       f"Pitch shift: {pitch_shift:+.0f} cents.",
            'pitch_shift': pitch_shift,
            'notes': melody_details.get('notes', [])
        })

    # Sort by impact (descending) and take top 3
    issues.sort(key=lambda x: x['impact'], reverse=True)

    return issues[:3]


def get_tajweed_feedback_for_confusion(confusion: Dict) -> str:
    """Get Tajweed-specific feedback for a pronunciation confusion."""
    # Get detailed confusion info if available
    confusion_details = confusion.get('confusion_details')

    if confusion_details:
        # Use rich feedback from confusion set
        description = confusion_details.get('description', '')
        tip = confusion_details.get('tip', '')

        # Get Arabic representations
        target_arabic = confusion.get('target_arabic', confusion.get('target_char', ''))
        produced_arabic = confusion.get('likely_produced_arabic', confusion.get('likely_produced', ''))

        feedback = f"**{description}**\n"
        feedback += f"You said '{produced_arabic}' but should say '{target_arabic}'\n"
        feedback += f"💡 {tip}"
        return feedback
    else:
        # Fallback to simple feedback
        target = confusion.get('target_arabic', confusion.get('target_char', ''))
        produced = confusion.get('likely_produced_arabic', confusion.get('likely_produced', ''))
        return f"Pronunciation error: produced '{produced}' instead of '{target}'. Review articulation rules."


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
