"""
Analyze phoneme alignment errors to understand what needs improvement.
"""
import sys
from pathlib import Path
import numpy as np
import matplotlib.pyplot as plt
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

# Extract twice to see variance
print("Extracting phonemes (run 1)...")
phonemes1 = extract_phonemes_wav2vec2_ctc(audio, word_segments, transliteration, pitch_data, surah, ayah)

print("\nExtracting phonemes (run 2)...")
phonemes2 = extract_phonemes_wav2vec2_ctc(audio, word_segments, transliteration, pitch_data, surah, ayah)

print("\n" + "="*70)
print("PHONEME ALIGNMENT ANALYSIS")
print("="*70)

# 1. Check determinism
print("\n1. DETERMINISM CHECK:")
if len(phonemes1) != len(phonemes2):
    print(f"   ❌ Different phoneme counts: {len(phonemes1)} vs {len(phonemes2)}")
else:
    start_diffs = [abs(p1['start'] - p2['start']) for p1, p2 in zip(phonemes1, phonemes2)]
    end_diffs = [abs(p1['end'] - p2['end']) for p1, p2 in zip(phonemes1, phonemes2)]

    print(f"   Phoneme count: {len(phonemes1)}")
    print(f"   Start time variance: mean={np.mean(start_diffs)*1000:.2f}ms, max={np.max(start_diffs)*1000:.2f}ms")
    print(f"   End time variance: mean={np.mean(end_diffs)*1000:.2f}ms, max={np.max(end_diffs)*1000:.2f}ms")

    if np.max(start_diffs) < 0.001:
        print("   ✅ Extraction is deterministic")
    else:
        print("   ⚠️  Extraction has some variance")

# 2. Word boundary alignment
print("\n2. WORD BOUNDARY ALIGNMENT:")
print(f"   {'Word':<15} {'Segment Time':<20} {'First Phoneme':<20} {'Last Phoneme':<20} {'Gap Before':<12} {'Gap After'}")
print("   " + "-"*120)

for i, ws in enumerate(word_segments):
    word_start_ms = ws['start_ms']
    word_end_ms = ws['end_ms']

    # Find phonemes in this word
    word_phonemes = [p for p in phonemes1 if p.get('word_index') == i]

    if not word_phonemes:
        print(f"   {ws['text']:<15} {word_start_ms:6.0f}-{word_end_ms:6.0f}ms   NO PHONEMES FOUND!")
        continue

    first_p = word_phonemes[0]
    last_p = word_phonemes[-1]

    gap_before = (first_p['start'] * 1000 - word_start_ms)
    gap_after = (word_end_ms - last_p['end'] * 1000)

    print(f"   {ws['text']:<15} {word_start_ms:6.0f}-{word_end_ms:6.0f}ms   "
          f"{first_p['start']*1000:6.0f}ms           {last_p['end']*1000:6.0f}ms           "
          f"{gap_before:+6.0f}ms      {gap_after:+6.0f}ms")

# 3. Duration distribution
print("\n3. DURATION DISTRIBUTION:")
durations = [p['duration'] * 1000 for p in phonemes1]
print(f"   Mean: {np.mean(durations):.0f}ms")
print(f"   Std:  {np.std(durations):.0f}ms")
print(f"   Min:  {np.min(durations):.0f}ms (phoneme: '{phonemes1[np.argmin(durations)]['phoneme']}')")
print(f"   Max:  {np.max(durations):.0f}ms (phoneme: '{phonemes1[np.argmax(durations)]['phoneme']}')")

# Find suspiciously short/long phonemes
short_threshold = 50  # ms
long_threshold = 500  # ms

short_phonemes = [(p['phoneme'], p['duration']*1000, p['start'], p['end'])
                  for p in phonemes1 if p['duration']*1000 < short_threshold]
long_phonemes = [(p['phoneme'], p['duration']*1000, p['start'], p['end'])
                 for p in phonemes1 if p['duration']*1000 > long_threshold]

if short_phonemes:
    print(f"\n   ⚠️  {len(short_phonemes)} suspiciously SHORT phonemes (<{short_threshold}ms):")
    for ph, dur, start, end in short_phonemes[:5]:
        print(f"      '{ph}': {dur:.0f}ms @ {start:.3f}s-{end:.3f}s")

if long_phonemes:
    print(f"\n   ⚠️  {len(long_phonemes)} suspiciously LONG phonemes (>{long_threshold}ms):")
    for ph, dur, start, end in long_phonemes[:5]:
        print(f"      '{ph}': {dur:.0f}ms @ {start:.3f}s-{end:.3f}s")

# 4. Energy analysis at boundaries
print("\n4. ENERGY ANALYSIS:")
print("   Analyzing if phoneme boundaries align with energy minima...")

import librosa
y, sr = librosa.load(audio, sr=16000)
# Compute RMS energy
hop_length = 160  # 10ms at 16kHz
rms = librosa.feature.rms(y=y, hop_length=hop_length)[0]
times = librosa.frames_to_time(np.arange(len(rms)), sr=sr, hop_length=hop_length)

boundary_energies = []
for i in range(1, len(phonemes1)):
    boundary_time = phonemes1[i]['start']
    # Find closest energy frame
    idx = np.argmin(np.abs(times - boundary_time))
    energy = rms[idx]

    # Compare to local context (±50ms)
    context_start = max(0, idx - 5)
    context_end = min(len(rms), idx + 5)
    local_min = np.min(rms[context_start:context_end])
    local_max = np.max(rms[context_start:context_end])

    # Normalized position: 0=local_min, 1=local_max
    if local_max > local_min:
        normalized_pos = (energy - local_min) / (local_max - local_min)
    else:
        normalized_pos = 0.5

    boundary_energies.append(normalized_pos)

mean_boundary_energy = np.mean(boundary_energies)
print(f"   Mean boundary energy (0=min, 1=max): {mean_boundary_energy:.2f}")
if mean_boundary_energy < 0.3:
    print("   ✅ Boundaries align well with energy minima")
elif mean_boundary_energy < 0.5:
    print("   ⚠️  Boundaries somewhat align with energy minima")
else:
    print("   ❌ Boundaries do NOT align with energy minima")

# 5. Coverage analysis
print("\n5. COVERAGE ANALYSIS:")
total_audio_duration = pitch_data['duration']
total_phoneme_duration = sum(p['duration'] for p in phonemes1)
coverage = (total_phoneme_duration / total_audio_duration) * 100

print(f"   Total audio: {total_audio_duration:.2f}s")
print(f"   Total phonemes: {total_phoneme_duration:.2f}s")
print(f"   Coverage: {coverage:.1f}%")

if coverage < 85:
    print("   ⚠️  Low coverage - significant gaps between phonemes")
elif coverage > 110:
    print("   ⚠️  Overlap - phonemes are overlapping")
else:
    print("   ✅ Good coverage")

# 6. Tajweed rule assignment
print("\n6. TAJWEED RULE ASSIGNMENT:")
tajweed_counts = {}
for p in phonemes1:
    rule = p.get('tajweed_rule', 'None')
    tajweed_counts[rule] = tajweed_counts.get(rule, 0) + 1

print(f"   Phonemes with Tajweed rules: {sum(v for k, v in tajweed_counts.items() if k and k != 'None')}/{len(phonemes1)}")
for rule, count in sorted(tajweed_counts.items(), key=lambda x: (x[0] is None, x[0])):
    if rule != 'None':
        print(f"      {rule}: {count}")

print("\n" + "="*70)
print("SUMMARY OF ISSUES")
print("="*70)

issues = []
if np.max(start_diffs) > 0.001:
    issues.append("• Non-deterministic extraction")
if len(short_phonemes) > 2:
    issues.append(f"• {len(short_phonemes)} very short phonemes (<{short_threshold}ms)")
if len(long_phonemes) > 1:
    issues.append(f"• {len(long_phonemes)} very long phonemes (>{long_threshold}ms)")
if mean_boundary_energy > 0.5:
    issues.append("• Boundaries don't align with energy minima")
if coverage < 85 or coverage > 110:
    issues.append(f"• Coverage issue: {coverage:.1f}%")

if issues:
    for issue in issues:
        print(issue)
else:
    print("✅ No major issues detected!")

print("\n")
