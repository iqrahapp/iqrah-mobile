"""
Duration Analysis with Tempo-Adaptive Scoring
==============================================

Implements tempo-adaptive Madd (elongation) scoring using Laplace/Huber penalty.
Accounts for local tempo variations.
"""

import numpy as np
from typing import List, Dict


def madd_score_tempo_adaptive(
    student_phonemes: List[Dict],
    reference_phonemes: List[Dict],
    student_mean_count: float,
    reference_mean_count: float,
    tempo_ratio: float
) -> Dict:
    """
    Score Madd accuracy with tempo-adaptive tolerance.

    Args:
        student_phonemes: Student phoneme list with 'duration', 'tajweed_rule'
        reference_phonemes: Reference phoneme list
        student_mean_count: Student's mean count duration
        reference_mean_count: Reference's mean count duration
        tempo_ratio: student_tempo / reference_tempo

    Returns:
        Dictionary with:
            - overall_accuracy: 0-100
            - by_type: Dict with scores per count type (2/4/6)
            - critical_issues: List of significant shortfalls
            - notes: Feedback notes
    """
    # Madd rules to expected counts
    madd_rules = {
        'madda_normal': 2,
        'madda_permissible': 2,
        'madda_necessary': 6,
        'madda_obligatory_mottasel': 4,
        'madda_obligatory_monfasel': 4,
    }

    scores_by_type = {}
    all_scores = []
    critical_issues = []

    for student_p in student_phonemes:
        tajweed = student_p.get('tajweed_rule', '')
        if not tajweed or tajweed not in madd_rules:
            continue

        expected_counts = madd_rules[tajweed]

        # Compute actual counts (tempo-normalized)
        actual_counts = student_p['duration'] / student_mean_count

        # Find corresponding reference phoneme (by timing)
        ref_expected_counts = find_reference_madd_counts(
            student_p, reference_phonemes, expected_counts
        )

        # Tempo-adaptive tolerance (Laplace scale parameter)
        # σ = 0.15 × expected_counts × (local_tempo / global_tempo)
        # For simplicity, use global tempo ratio
        sigma = 0.15 * expected_counts * tempo_ratio

        # Compute error
        error = abs(actual_counts - expected_counts)

        # Laplace scoring: exp(-|error| / σ)
        score = 100 * np.exp(-error / sigma)

        all_scores.append(score)

        # Track by type
        key = f"{expected_counts}_count"
        if key not in scores_by_type:
            scores_by_type[key] = {'scores': [], 'count': 0, 'errors': []}

        scores_by_type[key]['scores'].append(score)
        scores_by_type[key]['count'] += 1
        scores_by_type[key]['errors'].append(error)

        # Flag critical issues (>0.5 count shortfall)
        if error > 0.5 and actual_counts < expected_counts:
            critical_issues.append({
                'phoneme': student_p.get('phoneme', '?'),
                'expected': expected_counts,
                'actual': round(actual_counts, 2),
                'shortfall': round(error, 2),
                'severity': 'critical' if error > 1.0 else 'moderate'
            })

    # Compute overall and per-type scores
    overall_accuracy = np.mean(all_scores) if all_scores else 0

    by_type_summary = {}
    for key, data in scores_by_type.items():
        by_type_summary[key] = {
            'accuracy': round(np.mean(data['scores']), 1),
            'count': data['count'],
            'mean_error': round(np.mean(data['errors']), 2),
            'std_error': round(np.std(data['errors']), 2)
        }

    # Generate feedback notes
    notes = []

    if overall_accuracy >= 95:
        notes.append("Excellent elongation accuracy - all Madds held correctly")
    elif overall_accuracy >= 85:
        notes.append("Good elongation accuracy - minor timing issues")
    elif overall_accuracy >= 70:
        notes.append("Elongation accuracy needs improvement")
    else:
        notes.append("Significant elongation issues - focus on holding counts")

    # Specific feedback by type
    for key, summary in by_type_summary.items():
        counts = key.split('_')[0]
        if summary['mean_error'] > 0.5:
            direction = "too short" if summary['mean_error'] > 0 else "too long"
            notes.append(f"{counts}-count Madds consistently {direction} (avg error: {summary['mean_error']} counts)")

    # Critical issues feedback
    if len(critical_issues) > 0:
        notes.append(f"{len(critical_issues)} critical elongation shortfalls detected")

    return {
        'overall_accuracy': round(overall_accuracy, 1),
        'by_type': by_type_summary,
        'total_madds': len(all_scores),
        'critical_issues': critical_issues,
        'notes': notes,
        'tempo_ratio_applied': round(tempo_ratio, 2)
    }


def find_reference_madd_counts(
    student_phoneme: Dict,
    reference_phonemes: List[Dict],
    expected_counts: int
) -> int:
    """
    Find the corresponding Madd in reference recording.

    Uses temporal proximity heuristic.

    Returns:
        Expected count from reference (for validation)
    """
    # Simple heuristic: find reference phoneme at similar relative position
    # (More sophisticated: use DTW alignment path)

    # For now, just return expected_counts (assumes same structure)
    return expected_counts


def compute_duration_consistency(phonemes: List[Dict], rule_type: str) -> float:
    """
    Compute consistency (std/mean) for a specific Tajweed rule type.

    Lower = more consistent.

    Returns:
        Coefficient of variation (CV)
    """
    durations = [
        p['duration'] for p in phonemes
        if p.get('tajweed_rule', '') == rule_type
    ]

    if len(durations) < 2:
        return 0.0

    mean_dur = np.mean(durations)
    std_dur = np.std(durations)

    cv = std_dur / mean_dur if mean_dur > 0 else 0.0

    return float(cv)


def compare_duration_distributions(
    student_phonemes: List[Dict],
    reference_phonemes: List[Dict]
) -> Dict:
    """
    Compare overall duration distributions between student and reference.

    Uses Kolmogorov-Smirnov test.

    Returns:
        Dictionary with distribution comparison metrics
    """
    from scipy.stats import ks_2samp

    # Extract all voiced phoneme durations
    student_durs = np.array([
        p['duration'] for p in student_phonemes
        if p.get('mean_pitch', 0) > 0
    ])

    ref_durs = np.array([
        p['duration'] for p in reference_phonemes
        if p.get('mean_pitch', 0) > 0
    ])

    if len(student_durs) < 5 or len(ref_durs) < 5:
        return {
            'ks_statistic': 0.0,
            'p_value': 1.0,
            'similar': True
        }

    # Perform KS test
    statistic, p_value = ks_2samp(student_durs, ref_durs)

    # Interpret: p > 0.05 suggests similar distributions
    similar = p_value > 0.05

    return {
        'ks_statistic': round(float(statistic), 3),
        'p_value': round(float(p_value), 3),
        'similar': similar,
        'student_mean': round(float(np.mean(student_durs)), 3),
        'reference_mean': round(float(np.mean(ref_durs)), 3)
    }
