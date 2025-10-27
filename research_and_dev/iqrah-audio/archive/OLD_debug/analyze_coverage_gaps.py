"""Analyze where the missing coverage is in the original alignment."""
import sys
from pathlib import Path
import numpy as np
sys.path.insert(0, str(Path.cwd()))

from src.iqrah_audio.analysis.phoneme_wav2vec2_ctc import extract_phonemes_wav2vec2_ctc
from src.iqrah_audio.analysis.pitch_extractor_swiftf0 import extract_pitch_swiftf0
from src.iqrah_audio.analysis.segments_loader import get_word_segments_with_text
from src.iqrah_audio.analysis.phoneme_from_transliteration import load_transliteration_data

audio = 'data/husary/surahs/001/01.mp3'
surah, ayah = 1, 1

word_segments = get_word_segments_with_text(surah, ayah)
trans_data = load_transliteration_data()
transliteration = trans_data.get(f'{surah}:{ayah}', '')
pitch_data = extract_pitch_swiftf0(audio)

phonemes = extract_phonemes_wav2vec2_ctc(audio, word_segments, transliteration, pitch_data, surah, ayah)

print("="*70)
print("COVERAGE GAP ANALYSIS")
print("="*70)

total_duration = pitch_data['duration']
print(f"\nTotal audio duration: {total_duration:.3f}s ({total_duration*1000:.0f}ms)")

# Phoneme coverage
phoneme_dur = sum(p['duration'] for p in phonemes)
print(f"Phoneme coverage: {phoneme_dur:.3f}s ({phoneme_dur/total_duration*100:.1f}%)")
print(f"Missing coverage: {total_duration - phoneme_dur:.3f}s ({(1 - phoneme_dur/total_duration)*100:.1f}%)")

# Find gaps between phonemes
print(f"\n\nGAPS BETWEEN PHONEMES:")
print(f"{'Location':<30} {'Duration':<15} {'% of total'}")
print("-"*70)

# Gap before first phoneme
if len(phonemes) > 0:
    gap_start = phonemes[0]['start']
    if gap_start > 0.01:
        print(f"Start of audio → phoneme 1    {gap_start*1000:8.0f}ms       {gap_start/total_duration*100:5.1f}%")

# Gaps between phonemes
for i in range(len(phonemes) - 1):
    gap = phonemes[i+1]['start'] - phonemes[i]['end']
    if gap > 0.01:  # Only show gaps > 10ms
        print(f"Phoneme {i+1} → Phoneme {i+2}       {gap*1000:8.0f}ms       {gap/total_duration*100:5.1f}%")

# Gap after last phoneme
if len(phonemes) > 0:
    gap_end = total_duration - phonemes[-1]['end']
    if gap_end > 0.01:
        print(f"Phoneme {len(phonemes)} → end of audio      {gap_end*1000:8.0f}ms       {gap_end/total_duration*100:5.1f}%")

# Word segments vs phoneme coverage
print(f"\n\nWORD SEGMENT COVERAGE:")
print(f"{'Word':<15} {'Segment':<20} {'Phonemes':<20} {'Gap %'}")
print("-"*70)

for i, ws in enumerate(word_segments):
    w_start = ws['start_ms'] / 1000
    w_end = ws['end_ms'] / 1000
    w_dur = w_end - w_start

    # Find phonemes for this word
    word_phonemes = [p for p in phonemes if p.get('word_index') == i]

    if not word_phonemes:
        print(f"{ws['text']:<15} {w_start:.2f}-{w_end:.2f}s       NO PHONEMES")
        continue

    p_start = word_phonemes[0]['start']
    p_end = word_phonemes[-1]['end']
    p_dur = sum(p['duration'] for p in word_phonemes)

    gap_before = p_start - w_start
    gap_after = w_end - p_end

    gap_pct = (gap_before + gap_after) / w_dur * 100 if w_dur > 0 else 0

    print(f"{ws['text']:<15} {w_start:.2f}-{w_end:.2f}s ({w_dur*1000:4.0f}ms)  {p_start:.2f}-{p_end:.2f}s ({p_dur*1000:4.0f}ms)  {gap_pct:5.1f}%")

print("\n" + "="*70)
