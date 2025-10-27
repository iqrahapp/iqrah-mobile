"""Test improved phoneme alignment."""
import sys
from pathlib import Path
import numpy as np
sys.path.insert(0, str(Path.cwd()))

from src.iqrah_audio.analysis.phoneme_alignment_improved import extract_phonemes_improved
from src.iqrah_audio.analysis.phoneme_wav2vec2_ctc import extract_phonemes_wav2vec2_ctc
from src.iqrah_audio.analysis.pitch_extractor_swiftf0 import extract_pitch_swiftf0
from src.iqrah_audio.analysis.segments_loader import get_word_segments_with_text
from src.iqrah_audio.analysis.phoneme_from_transliteration import load_transliteration_data
from src.iqrah_audio.analysis.statistics_analyzer import compute_full_statistics
from src.iqrah_audio.comparison import compare_recitations

audio = 'data/husary/surahs/001/01.mp3'
surah, ayah = 1, 1

word_segments = get_word_segments_with_text(surah, ayah)
trans_data = load_transliteration_data()
transliteration = trans_data.get(f'{surah}:{ayah}', '')
pitch_data = extract_pitch_swiftf0(audio)

print("="*70)
print("TESTING IMPROVED ALIGNMENT")
print("="*70)

# Old alignment
print("\n\nOLD ALIGNMENT:")
old_phonemes = extract_phonemes_wav2vec2_ctc(audio, word_segments, transliteration, pitch_data, surah, ayah)

# New alignment
print("\n\nNEW IMPROVED ALIGNMENT:")
new_phonemes = extract_phonemes_improved(audio, word_segments, transliteration, pitch_data, surah, ayah)

# Compare
print("\n\n" + "="*70)
print("COMPARISON")
print("="*70)

print(f"\nPhoneme count: OLD={len(old_phonemes)}, NEW={len(new_phonemes)}")

# Coverage
total_duration = pitch_data['duration']
old_coverage = sum(p['duration'] for p in old_phonemes) / total_duration * 100
new_coverage = sum(p['duration'] for p in new_phonemes) / total_duration * 100

print(f"\nCoverage: OLD={old_coverage:.1f}%, NEW={new_coverage:.1f}%")

# Duration distribution
old_durs = [p['duration'] * 1000 for p in old_phonemes]
new_durs = [p['duration'] * 1000 for p in new_phonemes]

print(f"\nDuration stats:")
print(f"  OLD: mean={np.mean(old_durs):.0f}ms, std={np.std(old_durs):.0f}ms, max={np.max(old_durs):.0f}ms")
print(f"  NEW: mean={np.mean(new_durs):.0f}ms, std={np.std(new_durs):.0f}ms, max={np.max(new_durs):.0f}ms")

# Word boundary alignment
print(f"\nWord boundary alignment:")
print(f"  {'Word':<15} {'Segment':<20} {'OLD Coverage':<15} {'NEW Coverage'}")
print("  " + "-"*70)

for i, ws in enumerate(word_segments):
    w_start = ws['start_ms'] / 1000
    w_end = ws['end_ms'] / 1000
    w_dur = w_end - w_start

    old_word_phons = [p for p in old_phonemes if p.get('word_index') == i]
    new_word_phons = [p for p in new_phonemes if p.get('word_index') == i]

    old_word_dur = sum(p['duration'] for p in old_word_phons)
    new_word_dur = sum(p['duration'] for p in new_word_phons)

    old_cov = (old_word_dur / w_dur * 100) if w_dur > 0 else 0
    new_cov = (new_word_dur / w_dur * 100) if w_dur > 0 else 0

    print(f"  {ws['text']:<15} {w_start:.2f}-{w_end:.2f}s    {old_cov:5.1f}%          {new_cov:5.1f}%")

# Test self-comparison score
print(f"\n\n" + "="*70)
print("SELF-COMPARISON SCORE (Husary vs Husary)")
print("="*70)

# OLD
old_stats = compute_full_statistics(old_phonemes, pitch_data)
old_comparison = compare_recitations(audio, audio, old_phonemes, old_phonemes, pitch_data, pitch_data, old_stats, old_stats)

# NEW
new_stats = compute_full_statistics(new_phonemes, pitch_data)
new_comparison = compare_recitations(audio, audio, new_phonemes, new_phonemes, pitch_data, pitch_data, new_stats, new_stats)

print(f"\nOLD Alignment:")
print(f"  Overall: {old_comparison['overall']:.1f}/100")
print(f"  Duration: {old_comparison['durations']['overall']:.1f}/100")

print(f"\nNEW Improved Alignment:")
print(f"  Overall: {new_comparison['overall']:.1f}/100")
print(f"  Duration: {new_comparison['durations']['overall']:.1f}/100")

improvement = new_comparison['durations']['overall'] - old_comparison['durations']['overall']
print(f"\nImprovement: {improvement:+.1f} points")

print("\n" + "="*70)
