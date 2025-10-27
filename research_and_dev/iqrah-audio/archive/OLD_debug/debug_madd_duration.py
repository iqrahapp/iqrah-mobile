import sys
from pathlib import Path
sys.path.insert(0, str(Path.cwd()))

from src.iqrah_audio.analysis.phoneme_wav2vec2_ctc import extract_phonemes_wav2vec2_ctc
from src.iqrah_audio.analysis.pitch_extractor_swiftf0 import extract_pitch_swiftf0
from src.iqrah_audio.analysis.segments_loader import get_word_segments_with_text
from src.iqrah_audio.analysis.phoneme_from_transliteration import load_transliteration_data
from src.iqrah_audio.analysis.statistics_analyzer import compute_full_statistics

audio = 'data/husary/surahs/001/01.mp3'
surah, ayah = 1, 1

word_segments = get_word_segments_with_text(surah, ayah)
trans_data = load_transliteration_data()
transliteration = trans_data.get(f'{surah}:{ayah}', '')
pitch_data = extract_pitch_swiftf0(audio)
phonemes = extract_phonemes_wav2vec2_ctc(audio, word_segments, transliteration, pitch_data, surah, ayah)
stats = compute_full_statistics(phonemes, pitch_data)

mean_count_sec = stats['count']['mean_count']
mean_count_ms = mean_count_sec * 1000

print('Husary Phonemes with Madd:')
for p in phonemes:
    rule = p.get('tajweed_rule')
    if rule and 'madda' in rule:
        dur_ms = p['duration'] * 1000
        actual_counts = p['duration'] / mean_count_sec
        print(f"  {p['phoneme']:10} {p['start']:.3f}s-{p['end']:.3f}s  dur={dur_ms:6.1f}ms  counts={actual_counts:.2f}  rule={rule}")

print(f"\nMean count: {mean_count_ms:.1f}ms ({mean_count_sec:.3f}s)")

print('\nExpected for madda_permissible: 2, 4, or 6 counts')
print(f"At {mean_count_ms:.1f}ms/count:")
print(f"  2 counts = {2*mean_count_ms:.1f}ms")
print(f"  4 counts = {4*mean_count_ms:.1f}ms")
print(f"  6 counts = {6*mean_count_ms:.1f}ms")

print('\nConclusion:')
for p in phonemes:
    rule = p.get('tajweed_rule')
    if rule and 'madda' in rule:
        actual_counts = p['duration'] / mean_count_sec

        if rule == 'madda_permissible':
            # Find closest valid count (2, 4, or 6)
            valid_counts = [2, 4, 6]
            closest = min(valid_counts, key=lambda x: abs(x - actual_counts))
            error = abs(actual_counts - closest)
            print(f"  '{p['phoneme']}' is {actual_counts:.1f} counts → closest valid: {closest} counts (error: {error:.1f})")
        elif rule == 'madda_normal':
            error = abs(actual_counts - 2)
            print(f"  '{p['phoneme']}' is {actual_counts:.1f} counts → expected: 2 counts (error: {error:.1f})")
