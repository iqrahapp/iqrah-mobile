"""
Recitation Statistics Analyzer
================================

Extracts statistical features for Quranic recitation analysis:
- Tempo (inter-syllable intervals)
- Pitch distribution (Gaussian Mixture Model)
- Count duration (fundamental time unit)
- Madd accuracy (elongation verification)
- Rhythm analysis
"""

import numpy as np
from scipy import stats
from scipy.stats import norm
from sklearn.mixture import GaussianMixture
from typing import List, Dict, Optional, Tuple


def analyze_tempo(phonemes: List[Dict]) -> Dict:
    """
    Analyze tempo from phoneme timings.

    Args:
        phonemes: List of phoneme dictionaries with 'start', 'end'

    Returns:
        Dict with tempo statistics
    """
    # Extract inter-syllable intervals (ISI)
    isi_list = []
    for i in range(len(phonemes) - 1):
        # Time from end of current to start of next
        interval = phonemes[i+1]['start'] - phonemes[i]['end']
        if interval > 0:  # Skip overlaps
            isi_list.append(interval)

    if not isi_list:
        return {
            'mean_isi': 0.0,
            'std_isi': 0.0,
            'stability_score': 0,
            'syllables_per_second': 0.0,
            'distribution': {'mean': 0.0, 'std': 0.0}
        }

    isi_array = np.array(isi_list)
    mean_isi = float(np.mean(isi_array))
    std_isi = float(np.std(isi_array))

    # Stability score: 100 * (1 - CV) where CV = std/mean
    cv = std_isi / mean_isi if mean_isi > 0 else 1.0
    stability_score = max(0, min(100, 100 * (1 - cv)))

    # Syllables per second
    total_duration = phonemes[-1]['end'] - phonemes[0]['start']
    syl_per_sec = len(phonemes) / total_duration if total_duration > 0 else 0.0

    return {
        'mean_isi': mean_isi,
        'std_isi': std_isi,
        'stability_score': round(stability_score, 1),
        'syllables_per_second': round(syl_per_sec, 2),
        'distribution': {
            'mean': mean_isi,
            'std': std_isi,
            'type': 'gaussian'
        },
        'isi_values': isi_list  # For visualization
    }


def analyze_pitch_distribution(pitch_data: Dict, phonemes: List[Dict]) -> Dict:
    """
    Analyze pitch distribution using Gaussian Mixture Model.

    Args:
        pitch_data: Pitch data with 'time', 'f0_hz'
        phonemes: Phonemes for filtering voiced segments

    Returns:
        Dict with pitch statistics and GMM components
    """
    # Extract voiced pitch values
    f0_array = np.array(pitch_data['f0_hz'])
    voiced_f0 = f0_array[f0_array > 0]

    if len(voiced_f0) < 10:
        return {
            'mean_pitch': 0.0,
            'std_pitch': 0.0,
            'range': [0, 0],
            'gmm_components': [],
            'stability_score': 0
        }

    mean_pitch = float(np.mean(voiced_f0))
    std_pitch = float(np.std(voiced_f0))
    pitch_min = float(np.min(voiced_f0))
    pitch_max = float(np.max(voiced_f0))

    # Fit GMM (try 2-3 components)
    best_gmm = None
    best_bic = float('inf')

    for n_components in [2, 3]:
        try:
            gmm = GaussianMixture(n_components=n_components, random_state=42)
            gmm.fit(voiced_f0.reshape(-1, 1))
            bic = gmm.bic(voiced_f0.reshape(-1, 1))

            if bic < best_bic:
                best_bic = bic
                best_gmm = gmm
        except:
            continue

    # Extract GMM components
    gmm_components = []
    if best_gmm is not None:
        for i in range(best_gmm.n_components):
            mean = float(best_gmm.means_[i][0])
            std = float(np.sqrt(best_gmm.covariances_[i][0][0]))
            weight = float(best_gmm.weights_[i])

            gmm_components.append({
                'mean': round(mean, 1),
                'std': round(std, 1),
                'weight': round(weight, 3)
            })

    # Pitch stability: inverse of coefficient of variation
    cv = std_pitch / mean_pitch if mean_pitch > 0 else 1.0
    stability_score = max(0, min(100, 100 * (1 - cv)))

    return {
        'mean_pitch': round(mean_pitch, 1),
        'std_pitch': round(std_pitch, 1),
        'range': [round(pitch_min, 1), round(pitch_max, 1)],
        'gmm_components': gmm_components,
        'stability_score': round(stability_score, 1),
        'pitch_values': voiced_f0.tolist()  # For visualization
    }


def analyze_count_duration(phonemes: List[Dict]) -> Dict:
    """
    Estimate count duration from baseline (non-elongated) phonemes.

    Args:
        phonemes: List with 'duration', 'tajweed_rule'

    Returns:
        Dict with count statistics
    """
    # Filter baseline phonemes (no Madd, no special rules)
    baseline_durations = []

    for p in phonemes:
        # Skip unvoiced
        if p.get('mean_pitch', 0) == 0:
            continue

        # Skip elongated phonemes
        tajweed = p.get('tajweed_rule', '')
        if tajweed and 'madd' in str(tajweed).lower():
            continue

        # Use short phonemes as baseline (likely 1 count)
        dur = p['duration']
        if 0.05 < dur < 0.4:  # Reasonable range
            baseline_durations.append(dur)

    if not baseline_durations:
        return {
            'mean_count': 0.0,
            'std_count': 0.0,
            'precision_score': 0,
            'distribution': {'mean': 0.0, 'std': 0.0}
        }

    dur_array = np.array(baseline_durations)
    mean_count = float(np.mean(dur_array))
    std_count = float(np.std(dur_array))

    # Precision score: 100 * (1 - CV)
    cv = std_count / mean_count if mean_count > 0 else 1.0
    precision_score = max(0, min(100, 100 * (1 - cv)))

    return {
        'mean_count': round(mean_count, 3),
        'std_count': round(std_count, 3),
        'precision_score': round(precision_score, 1),
        'distribution': {
            'mean': mean_count,
            'std': std_count,
            'type': 'gaussian'
        },
        'sample_count': len(baseline_durations)
    }


def analyze_madd_accuracy(phonemes: List[Dict], mean_count: float) -> Dict:
    """
    Analyze elongation (Madd) accuracy.

    Args:
        phonemes: List with 'duration', 'tajweed_rule'
        mean_count: Mean count duration

    Returns:
        Dict with Madd accuracy statistics
    """
    if mean_count <= 0:
        return {
            'overall_accuracy': 0,
            'by_type': {},
            'total_madds': 0
        }

    # Madd type to expected counts
    madd_rules = {
        'madda_normal': 2,
        'madda_permissible': 2,
        'madda_necessary': 6,
        'madda_obligatory_mottasel': 4,
        'madda_obligatory_monfasel': 4,
    }

    scores_by_type = {}
    all_scores = []

    for p in phonemes:
        tajweed = p.get('tajweed_rule', '')
        if not tajweed or tajweed not in madd_rules:
            continue

        expected_counts = madd_rules[tajweed]
        actual_counts = p['duration'] / mean_count

        # Score using Gaussian: exp(-error² / (2σ²))
        # σ = 0.3 counts (tolerance)
        error = abs(actual_counts - expected_counts)
        tolerance = 0.3
        score = 100 * np.exp(-(error ** 2) / (2 * tolerance ** 2))

        all_scores.append(score)

        # Group by expected count
        key = f"{expected_counts}_count"
        if key not in scores_by_type:
            scores_by_type[key] = {'scores': [], 'count': 0}

        scores_by_type[key]['scores'].append(score)
        scores_by_type[key]['count'] += 1

    # Calculate overall and per-type accuracy
    overall_accuracy = np.mean(all_scores) if all_scores else 0

    by_type_summary = {}
    for key, data in scores_by_type.items():
        by_type_summary[key] = {
            'accuracy': round(np.mean(data['scores']), 1),
            'count': data['count']
        }

    return {
        'overall_accuracy': round(overall_accuracy, 1),
        'by_type': by_type_summary,
        'total_madds': len(all_scores)
    }


def analyze_rhythm(phonemes: List[Dict]) -> Dict:
    """
    Analyze rhythmic patterns from phoneme onsets.

    Args:
        phonemes: List with 'start' times

    Returns:
        Dict with rhythm statistics
    """
    # Extract onset times
    onsets = [p['start'] for p in phonemes]

    # Calculate inter-onset intervals (IOI)
    ioi_list = []
    for i in range(len(onsets) - 1):
        ioi = onsets[i+1] - onsets[i]
        if ioi > 0:
            ioi_list.append(ioi)

    if not ioi_list:
        return {
            'onset_times': onsets,
            'interval_stability': 0.0,
            'mean_ioi': 0.0,
            'std_ioi': 0.0
        }

    ioi_array = np.array(ioi_list)
    mean_ioi = float(np.mean(ioi_array))
    std_ioi = float(np.std(ioi_array))

    # Stability: 1 - CV
    cv = std_ioi / mean_ioi if mean_ioi > 0 else 1.0
    stability = max(0, min(1, 1 - cv))

    return {
        'onset_times': onsets,
        'interval_stability': round(stability, 3),
        'mean_ioi': round(mean_ioi, 3),
        'std_ioi': round(std_ioi, 3),
        'ioi_values': ioi_list
    }


def compute_full_statistics(
    phonemes: List[Dict],
    pitch_data: Dict
) -> Dict:
    """
    Compute all statistics for a recitation.

    Args:
        phonemes: Phoneme segments
        pitch_data: Pitch data

    Returns:
        Complete statistics dictionary
    """
    # Tempo analysis
    tempo_stats = analyze_tempo(phonemes)

    # Pitch analysis
    pitch_stats = analyze_pitch_distribution(pitch_data, phonemes)

    # Count duration
    count_stats = analyze_count_duration(phonemes)

    # Madd accuracy
    madd_stats = analyze_madd_accuracy(phonemes, count_stats['mean_count'])

    # Rhythm analysis
    rhythm_stats = analyze_rhythm(phonemes)

    return {
        'tempo': tempo_stats,
        'pitch': pitch_stats,
        'count': count_stats,
        'madd': madd_stats,
        'rhythm': rhythm_stats,
        'metadata': {
            'total_phonemes': len(phonemes),
            'duration': phonemes[-1]['end'] - phonemes[0]['start'] if phonemes else 0
        }
    }
