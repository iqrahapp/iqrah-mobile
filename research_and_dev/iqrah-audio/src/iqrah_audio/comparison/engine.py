"""
Comparison Engine - Main Orchestrator
======================================

Coordinates all comparison components and returns comprehensive assessment.
"""

from typing import Dict
from .features import extract_features, extract_tempo_ratio
from .rhythm import rhythm_score
from .melody import melody_score
from .duration import madd_score_tempo_adaptive
from .fusion import compute_overall_score, aggregate_feedback_notes, generate_improvement_suggestions


def compare_recitations(
    student_audio_path: str,
    reference_audio_path: str,
    student_phonemes: list,
    reference_phonemes: list,
    student_pitch: dict,
    reference_pitch: dict,
    student_stats: dict,
    reference_stats: dict
) -> Dict:
    """
    Compare student recitation against reference (Qari).

    Args:
        student_audio_path: Path to student audio
        reference_audio_path: Path to reference audio
        student_phonemes: Student phoneme list from Phase 1
        reference_phonemes: Reference phoneme list
        student_pitch: Student pitch data
        reference_pitch: Reference pitch data
        student_stats: Student statistics from Phase 1
        reference_stats: Reference statistics from Phase 1

    Returns:
        Comprehensive comparison dictionary with JSON structure from spec
    """
    # Extract features for both
    print("📊 Extracting comparison features...")

    student_features = extract_features(
        student_audio_path,
        student_phonemes,
        student_pitch,
        student_stats
    )

    reference_features = extract_features(
        reference_audio_path,
        reference_phonemes,
        reference_pitch,
        reference_stats
    )

    # Compute tempo ratio
    tempo_ratio = extract_tempo_ratio(student_features, reference_features)
    print(f"   Tempo ratio: {tempo_ratio:.2f} (1.0 = same pace)")

    # Component 1: Rhythm
    print("🎵 Analyzing rhythm...")
    rhythm_result = rhythm_score(student_features, reference_features)
    print(f"   Rhythm score: {rhythm_result['score']}/100")

    # Component 2: Melody
    print("🎼 Analyzing melody...")
    melody_result = melody_score(student_features, reference_features)
    print(f"   Melody score: {melody_result['score']}/100")
    print(f"   Pitch shift: {melody_result['pitch_shift_cents']:+.0f} cents")

    # Component 3: Duration (Madd)
    print("⏱️  Analyzing elongations...")
    duration_result = madd_score_tempo_adaptive(
        student_phonemes,
        reference_phonemes,
        student_features.mean_count,
        reference_features.mean_count,
        tempo_ratio
    )
    print(f"   Duration score: {duration_result['overall_accuracy']}/100")

    # Component 4: Pronunciation (TODO: implement SSL-GOP)
    # For now, skip pronunciation
    pronunciation_result = {
        'score': 0,
        'notes': ['Pronunciation assessment not yet implemented']
    }

    # Combine components
    component_scores = {
        'rhythm': rhythm_result,
        'melody': melody_result,
        'duration': duration_result,
        # 'pronunciation': pronunciation_result  # Skip for now
    }

    # Adjust weights since pronunciation is missing
    weights = {
        'rhythm': 0.40,
        'melody': 0.25,
        'duration': 0.35
    }

    # Compute overall score
    print("🎯 Computing overall score...")
    overall_result = compute_overall_score(component_scores, weights=weights)
    print(f"   Overall: {overall_result['overall']}/100 (confidence: {overall_result['confidence']})")

    # Aggregate feedback
    feedback_notes = aggregate_feedback_notes(component_scores)
    improvement_suggestions = generate_improvement_suggestions(
        component_scores,
        overall_result['top_issues']
    )

    # Build final result (matches JSON contract from spec)
    result = {
        'overall': overall_result['overall'],
        'confidence': overall_result['confidence'],

        'rhythm': {
            'score': rhythm_result['score'],
            'notes': rhythm_result['notes'],
            'path': rhythm_result['path'],
            'divergence': rhythm_result['divergence']
        },

        'melody': {
            'score': melody_result['score'],
            'pitch_shift_cents': melody_result['pitch_shift_cents'],
            'contour_similarity': melody_result['contour_similarity'],
            'notes': melody_result['notes']
        },

        'durations': {
            'overall': duration_result['overall_accuracy'],
            'by_type': duration_result['by_type'],
            'critical_issues': duration_result['critical_issues'],
            'notes': duration_result['notes']
        },

        # 'pronunciation': pronunciation_result,  # Skip for now

        'feedback': {
            'all_notes': feedback_notes,
            'suggestions': improvement_suggestions,
            'top_issues': overall_result['top_issues']
        },

        'metadata': {
            'tempo_ratio': tempo_ratio,
            'student_pace': student_features.tempo_estimate,
            'reference_pace': reference_features.tempo_estimate
        }
    }

    return result
