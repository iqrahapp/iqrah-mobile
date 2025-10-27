"""Analyze GOP reliability by phoneme duration."""
import sys
from pathlib import Path
sys.path.insert(0, str(Path.cwd()))

import numpy as np
from src.iqrah_audio.comparison.pronunciation import extract_emissions_and_alignment, compute_gop_scores
from src.iqrah_audio.analysis.phoneme_from_transliteration import load_transliteration_data

husary_audio = 'data/husary/surahs/001/01.mp3'
surah, ayah = 1, 1

trans_data = load_transliteration_data()
transliteration = trans_data.get(f'{surah}:{ayah}', '')

print("="*70)
print("GOP RELIABILITY BY PHONEME DURATION")
print("="*70)

# Extract and compute GOP
emissions, char_spans, labels = extract_emissions_and_alignment(husary_audio, transliteration)
gop_scores = compute_gop_scores(emissions, char_spans, labels)

# Group by duration
short = [s for s in gop_scores if s['duration'] < 0.03]  # < 30ms
medium = [s for s in gop_scores if 0.03 <= s['duration'] < 0.06]  # 30-60ms
long = [s for s in gop_scores if s['duration'] >= 0.06]  # >= 60ms

print(f"\n📊 GOP Statistics by Duration:")
print("-"*70)

for name, group in [("Short (<30ms)", short), ("Medium (30-60ms)", medium), ("Long (>=60ms)", long)]:
    if len(group) == 0:
        continue

    gop_vals = [s['gop_mean'] for s in group]
    print(f"\n{name}: {len(group)} phonemes")
    print(f"  Mean GOP:   {np.mean(gop_vals):>6.2f}")
    print(f"  Std GOP:    {np.std(gop_vals):>6.2f}")
    print(f"  Median GOP: {np.median(gop_vals):>6.2f}")
    print(f"  Range:      [{np.min(gop_vals):>6.2f}, {np.max(gop_vals):>6.2f}]")

# Check if longer phonemes have lower variance
print("\n" + "="*70)
print("INSIGHT")
print("="*70)

if len(short) > 0 and len(long) > 0:
    short_std = np.std([s['gop_mean'] for s in short])
    long_std = np.std([s['gop_mean'] for s in long])

    print(f"\nVariance comparison:")
    print(f"  Short phonemes: std = {short_std:.2f}")
    print(f"  Long phonemes:  std = {long_std:.2f}")

    if short_std > long_std * 1.5:
        print(f"\n  ✅ Longer phonemes have lower variance!")
        print(f"     → Can weight longer phonemes more heavily in scoring")
    else:
        print(f"\n  ⚠️  Duration doesn't significantly affect variance")
        print(f"     → Need different approach")

print("\n" + "="*70)
