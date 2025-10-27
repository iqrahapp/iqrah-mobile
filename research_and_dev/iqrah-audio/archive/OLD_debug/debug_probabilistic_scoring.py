import sys
from pathlib import Path
import numpy as np
sys.path.insert(0, str(Path.cwd()))

from src.iqrah_audio.analysis.phoneme_wav2vec2_ctc import extract_phonemes_wav2vec2_ctc
from src.iqrah_audio.analysis.pitch_extractor_swiftf0 import extract_pitch_swiftf0
from src.iqrah_audio.analysis.segments_loader import get_word_segments_with_text
from src.iqrah_audio.analysis.phoneme_from_transliteration import load_transliteration_data
from src.iqrah_audio.analysis.statistics_analyzer import compute_full_statistics

# Load both recitations
student_audio = 'data/me/surahs/001/01.mp3'
reference_audio = 'data/husary/surahs/001/01.mp3'

word_segments = get_word_segments_with_text(1, 1)
trans_data = load_transliteration_data()
transliteration = trans_data.get('1:1', '')

# Student
student_pitch = extract_pitch_swiftf0(student_audio)
student_phonemes = extract_phonemes_wav2vec2_ctc(student_audio, word_segments, transliteration, student_pitch, 1, 1)
student_stats = compute_full_statistics(student_phonemes, student_pitch)

# Reference
reference_pitch = extract_pitch_swiftf0(reference_audio)
reference_phonemes = extract_phonemes_wav2vec2_ctc(reference_audio, word_segments, transliteration, reference_pitch, 1, 1)
reference_stats = compute_full_statistics(reference_phonemes, reference_pitch)

# Get count parameters
student_mean_count = student_stats['count']['mean_count']
student_std_count = student_stats['count']['std_count']
reference_mean_count = reference_stats['count']['mean_count']
reference_std_count = reference_stats['count']['std_count']

print(f"Student mean count: {student_mean_count:.3f}s (std={student_std_count:.3f}s, CV={student_std_count/student_mean_count:.1%})")
print(f"Reference mean count: {reference_mean_count:.3f}s (std={reference_std_count:.3f}s, CV={reference_std_count/reference_mean_count:.1%})")
print()

# Analyze each Madd phoneme
print("Student Madd phonemes:")
for sp in student_phonemes:
    if sp.get('tajweed_rule') and 'madda' in sp['tajweed_rule']:
        # Find reference
        ref_match = None
        for rp in reference_phonemes:
            if rp.get('tajweed_rule') == sp['tajweed_rule']:
                if ref_match is None or abs(rp['start'] - sp['start']) < abs(ref_match['start'] - sp['start']):
                    ref_match = rp

        if ref_match:
            # Determine expected counts
            ref_actual_counts = ref_match['duration'] / reference_mean_count
            if sp['tajweed_rule'] == 'madda_permissible':
                valid_counts = [2, 4, 6]
                expected_counts = min(valid_counts, key=lambda x: abs(x - ref_actual_counts))
            else:
                expected_counts = 2  # madda_normal

            # Calculate expected duration distribution
            estimated_cv = 0.18
            student_std_count_est = student_mean_count * estimated_cv
            expected_duration_mean = expected_counts * student_mean_count
            expected_duration_std = np.sqrt(expected_counts) * student_std_count_est

            # Observed
            observed_duration = sp['duration']

            # Z-score
            z_score = (observed_duration - expected_duration_mean) / expected_duration_std

            # Score (using Laplace)
            score = 100 * np.exp(-abs(z_score) / 3.0)

            print(f"\n  {sp['phoneme']:10} ({sp['tajweed_rule']})")
            print(f"    Expected: {expected_counts} counts = {expected_duration_mean*1000:.0f}ms ± {expected_duration_std*1000:.0f}ms")
            print(f"    Observed: {observed_duration*1000:.0f}ms ({observed_duration/student_mean_count:.2f} counts)")
            print(f"    Z-score: {z_score:.2f}σ")
            print(f"    Score: {score:.1f}/100")
            print(f"    Reference: {ref_match['duration']*1000:.0f}ms ({ref_actual_counts:.2f} counts)")
