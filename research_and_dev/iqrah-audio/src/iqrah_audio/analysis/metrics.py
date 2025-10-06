"""
Recitation Metrics
=================

Calculate comprehensive metrics for recitation analysis.
"""

import numpy as np
from typing import List, Dict
from scipy import stats


def calculate_pitch_accuracy_per_word(
    user_pitch: List[float],
    ref_pitch: List[float],
    word_alignment: List[int],
    num_words: int,
    user_to_ref: Dict[int, int] = None
) -> List[Dict]:
    """
    Calculate pitch accuracy for each word.

    Args:
        user_pitch: User's f0 values
        ref_pitch: Reference f0 values
        word_alignment: Word index for each user frame
        num_words: Total number of words
        user_to_ref: DTW mapping from user frames to reference frames

    Returns:
        List of word accuracy dictionaries
    """

    user_pitch = np.array(user_pitch)
    ref_pitch = np.array(ref_pitch)
    word_alignment = np.array(word_alignment)

    word_scores = []

    for word_idx in range(num_words):
        # Get frames for this word
        word_mask = word_alignment == word_idx
        user_frames = np.where(word_mask)[0]

        if len(user_frames) == 0:
            word_scores.append({
                'word_idx': word_idx,
                'error_cents': 999.0,
                'status': 'missing',
                'confidence': 0.0
            })
            continue

        # Get corresponding ref frames via DTW mapping
        user_word_pitch = user_pitch[user_frames]

        if user_to_ref is not None:
            # Use DTW mapping
            ref_word_pitch = []
            for user_idx in user_frames:
                ref_idx = user_to_ref.get(int(user_idx), None)
                if ref_idx is not None and ref_idx < len(ref_pitch):
                    ref_word_pitch.append(ref_pitch[ref_idx])
                else:
                    ref_word_pitch.append(0.0)
            ref_word_pitch = np.array(ref_word_pitch)
        else:
            # Fallback: assume same indices (won't work well but won't crash)
            max_idx = min(len(user_frames), len(ref_pitch))
            ref_word_pitch = ref_pitch[user_frames[:max_idx]] if max_idx > 0 else np.array([0.0])

        # Remove zeros
        valid = (user_word_pitch > 0) & (ref_word_pitch > 0)
        if not np.any(valid):
            word_scores.append({
                'word_idx': word_idx,
                'error_cents': 999.0,
                'status': 'unvoiced',
                'confidence': 0.0
            })
            continue

        user_valid = user_word_pitch[valid]
        ref_valid = ref_word_pitch[valid]

        # Normalize to compare melody (not absolute pitch)
        user_norm = user_valid / np.mean(user_valid)
        ref_norm = ref_valid / np.mean(ref_valid)

        # Calculate error in cents
        ratio = user_norm / ref_norm
        error_cents = np.abs(1200 * np.log2(ratio))
        mean_error = np.mean(error_cents)

        # Determine status
        if mean_error < 30:
            status = 'good'
        elif mean_error < 60:
            status = 'warning'
        else:
            status = 'error'

        word_scores.append({
            'word_idx': word_idx,
            'error_cents': float(mean_error),
            'status': status,
            'confidence': float(1.0 - min(mean_error / 100, 1.0))
        })

    return word_scores


def calculate_stability(pitch: List[float]) -> Dict:
    """
    Calculate pitch stability (voice quality).

    High local variation = shaky voice.

    Args:
        pitch: F0 values

    Returns:
        Dictionary with stability metrics
    """

    pitch = np.array(pitch)
    voiced = pitch > 0

    if not np.any(voiced):
        return {
            'jitter_hz': 0.0,
            'jitter_percent': 0.0,
            'stability_score': 0.0,
            'status': 'unknown'
        }

    voiced_pitch = pitch[voiced]

    # Calculate jitter (local pitch variation)
    pitch_deltas = np.diff(voiced_pitch)
    jitter_hz = np.sqrt(np.mean(pitch_deltas**2))
    jitter_percent = (jitter_hz / np.mean(voiced_pitch)) * 100

    # Stability score (0-1, higher is better)
    stability_score = max(0, 1.0 - (jitter_percent / 5.0))  # 5% jitter = 0 score

    status = 'stable' if jitter_percent < 2.0 else 'unstable'

    return {
        'jitter_hz': float(jitter_hz),
        'jitter_percent': float(jitter_percent),
        'stability_score': float(stability_score),
        'status': status
    }


def calculate_complexity(pitch: List[float]) -> Dict:
    """
    Calculate melody complexity.

    Good: 1-3 distinct pitch peaks (structured melody)
    Bad: Flat distribution (random pitches)

    Args:
        pitch: F0 values

    Returns:
        Dictionary with complexity metrics
    """

    pitch = np.array(pitch)
    voiced = pitch > 0

    if not np.any(voiced):
        return {
            'num_peaks': 0,
            'entropy': 0.0,
            'complexity_score': 0.0,
            'status': 'unknown'
        }

    voiced_pitch = pitch[voiced]

    # Histogram of pitches
    hist, bins = np.histogram(voiced_pitch, bins=30, density=True)

    # Find peaks in distribution
    from scipy.signal import find_peaks
    peaks, _ = find_peaks(hist, prominence=np.max(hist)*0.1)

    # Calculate entropy (measure of randomness)
    prob = hist / np.sum(hist)
    prob = prob[prob > 0]  # Remove zeros
    entropy = -np.sum(prob * np.log(prob))

    # Complexity score (0-1, lower is better)
    # Ideal: 1-3 peaks, low entropy
    peak_score = 1.0 - min(abs(len(peaks) - 2) / 5.0, 1.0)  # Optimal at 2 peaks
    entropy_score = 1.0 - min(entropy / 3.0, 1.0)  # Lower entropy is better

    complexity_score = (peak_score + entropy_score) / 2

    if len(peaks) <= 3 and entropy < 2.0:
        status = 'simple'  # Good!
    else:
        status = 'complex'  # Too many different pitches

    return {
        'num_peaks': int(len(peaks)),
        'entropy': float(entropy),
        'complexity_score': float(complexity_score),
        'status': status
    }


def calculate_overall_score(
    word_scores: List[Dict],
    tempo: Dict,
    stability: Dict,
    complexity: Dict
) -> int:
    """
    Calculate overall score (0-100).

    Args:
        word_scores: Per-word accuracy scores
        tempo: Tempo metrics
        stability: Stability metrics
        complexity: Complexity metrics

    Returns:
        Overall score (0-100)
    """

    # Word accuracy score (0-50 points)
    word_confidences = [w['confidence'] for w in word_scores if w['status'] != 'missing']
    word_score = np.mean(word_confidences) * 50 if word_confidences else 0

    # Tempo score (0-15 points)
    tempo_ratio = tempo.get('mean_ratio', 1.0)
    tempo_deviation = abs(tempo_ratio - 1.0)
    tempo_score = max(0, (1.0 - tempo_deviation) * 15)

    # Stability score (0-20 points)
    stability_score = stability.get('stability_score', 0.0) * 20

    # Complexity score (0-15 points)
    complexity_score = complexity.get('complexity_score', 0.0) * 15

    overall = word_score + tempo_score + stability_score + complexity_score

    return int(min(max(overall, 0), 100))


if __name__ == "__main__":
    # Test metrics
    print("Testing metrics calculation...")

    # Synthetic data
    pitch = np.sin(np.linspace(0, 4*np.pi, 100)) * 50 + 200
    pitch_noisy = pitch + np.random.normal(0, 2, len(pitch))

    stability = calculate_stability(pitch.tolist())
    print(f"\n✓ Stability (clean): {stability['jitter_percent']:.2f}%")

    stability_noisy = calculate_stability(pitch_noisy.tolist())
    print(f"✓ Stability (noisy): {stability_noisy['jitter_percent']:.2f}%")

    complexity = calculate_complexity(pitch.tolist())
    print(f"\n✓ Complexity: {complexity['num_peaks']} peaks, entropy={complexity['entropy']:.2f}")
