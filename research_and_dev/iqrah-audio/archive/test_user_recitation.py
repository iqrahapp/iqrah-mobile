"""Test user recitation vs Husary with current alignment."""
import sys
from pathlib import Path
sys.path.insert(0, str(Path.cwd()))

from src.iqrah_audio.analysis.phoneme_wav2vec2_ctc import extract_phonemes_wav2vec2_ctc
from src.iqrah_audio.analysis.pitch_extractor_swiftf0 import extract_pitch_swiftf0
from src.iqrah_audio.analysis.segments_loader import get_word_segments_with_text
from src.iqrah_audio.analysis.phoneme_from_transliteration import load_transliteration_data
from src.iqrah_audio.analysis.statistics_analyzer import compute_full_statistics
from src.iqrah_audio.comparison import compare_recitations

# User recitation
user_audio = 'static/temp/user_1_1_1759872988.webm'
ref_audio = 'data/husary/surahs/001/01.mp3'
surah, ayah = 1, 1

word_segments = get_word_segments_with_text(surah, ayah)
trans_data = load_transliteration_data()
transliteration = trans_data.get(f'{surah}:{ayah}', '')

print("="*70)
print("USER vs HUSARY COMPARISON")
print("="*70)

# Extract features for user
print("\n\nUSER RECITATION:")
user_pitch = extract_pitch_swiftf0(user_audio)
user_phonemes = extract_phonemes_wav2vec2_ctc(user_audio, word_segments, transliteration, user_pitch, surah, ayah)
user_stats = compute_full_statistics(user_phonemes, user_pitch)

print(f"Duration: {user_pitch['duration']:.2f}s")
print(f"Phonemes: {len(user_phonemes)}")
print(f"Coverage: {sum(p['duration'] for p in user_phonemes) / user_pitch['duration'] * 100:.1f}%")

# Extract features for reference
print("\n\nREFERENCE (HUSARY):")
ref_pitch = extract_pitch_swiftf0(ref_audio)
ref_phonemes = extract_phonemes_wav2vec2_ctc(ref_audio, word_segments, transliteration, ref_pitch, surah, ayah)
ref_stats = compute_full_statistics(ref_phonemes, ref_pitch)

print(f"Duration: {ref_pitch['duration']:.2f}s")
print(f"Phonemes: {len(ref_phonemes)}")
print(f"Coverage: {sum(p['duration'] for p in ref_phonemes) / ref_pitch['duration'] * 100:.1f}%")

# Compare
print("\n\n" + "="*70)
print("COMPARISON RESULTS")
print("="*70)

comparison = compare_recitations(
    user_audio, ref_audio,
    user_phonemes, ref_phonemes,
    user_pitch, ref_pitch,
    user_stats, ref_stats
)

print(f"\nOverall: {comparison['overall']:.1f}/100 (confidence: {comparison['confidence']:.2f})")
print(f"Rhythm: {comparison['rhythm']['score']:.1f}/100")
print(f"Melody: {comparison['melody']['score']:.1f}/100")
print(f"Duration: {comparison['durations']['overall']:.1f}/100")

if 'durations' in comparison and 'details' in comparison['durations']:
    details = comparison['durations']['details']
    print(f"\nDuration Details:")
    for d in details:
        if d['score'] < 80:  # Show low-scoring ones
            print(f"  {d['phoneme']}: {d['score']:.1f}/100 (expected {d['expected_counts']:.1f} counts, got {d['observed_counts']:.1f})")

print("\n" + "="*70)
