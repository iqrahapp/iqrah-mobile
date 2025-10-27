"""Analyze frame-level GOP to understand variance."""
import sys
from pathlib import Path
sys.path.insert(0, str(Path.cwd()))

import torch
import numpy as np
import matplotlib.pyplot as plt
from src.iqrah_audio.comparison.pronunciation import extract_emissions_and_alignment, compute_gop_scores
from src.iqrah_audio.analysis.phoneme_from_transliteration import load_transliteration_data

husary_audio = 'data/husary/surahs/001/01.mp3'
user_audio = 'static/temp/user_1_1_1759872988.webm'
surah, ayah = 1, 1

trans_data = load_transliteration_data()
transliteration = trans_data.get(f'{surah}:{ayah}', '')

print("="*70)
print("FRAME-LEVEL GOP ANALYSIS")
print("="*70)

# Extract for Husary
print("\n📊 Analyzing Husary (perfect recitation)...")
emissions_h, char_spans_h, labels = extract_emissions_and_alignment(husary_audio, transliteration)

# Look at GOP per frame for a few phonemes
print(f"\nAnalyzing frame-level GOP for first 5 phonemes:")
print("-"*70)

for i, span in enumerate(char_spans_h[:5]):
    frame_start = span['frame_start']
    frame_end = span['frame_end']
    token_id = span['token_id']
    char = span['char']

    # Extract emissions for this span
    span_emissions = emissions_h[frame_start:frame_end, :]

    if span_emissions.size(0) == 0:
        continue

    # GOP per frame
    target_logits = span_emissions[:, token_id]
    mask = torch.ones(span_emissions.size(1), dtype=torch.bool)
    mask[token_id] = False
    other_logits = span_emissions[:, mask]
    max_other_logits = other_logits.max(dim=1).values
    gop_per_frame = target_logits - max_other_logits

    gop_values = gop_per_frame.numpy()

    print(f"\nPhone '{char}' ({frame_end - frame_start} frames):")
    print(f"  Mean GOP:   {np.mean(gop_values):>6.2f}")
    print(f"  Std GOP:    {np.std(gop_values):>6.2f}")
    print(f"  Min GOP:    {np.min(gop_values):>6.2f}")
    print(f"  Max GOP:    {np.max(gop_values):>6.2f}")
    print(f"  Median GOP: {np.median(gop_values):>6.2f}")
    print(f"  IQR:        {np.percentile(gop_values, 75) - np.percentile(gop_values, 25):>6.2f}")

    # Check for outliers
    q1, q3 = np.percentile(gop_values, [25, 75])
    iqr = q3 - q1
    outliers = np.sum((gop_values < q1 - 1.5*iqr) | (gop_values > q3 + 1.5*iqr))
    print(f"  Outliers:   {outliers} / {len(gop_values)} ({outliers/len(gop_values)*100:.1f}%)")

print("\n" + "="*70)
print("KEY INSIGHTS")
print("="*70)

# Compute overall statistics
all_gop_values = []
all_std_values = []

for span in char_spans_h:
    frame_start = span['frame_start']
    frame_end = span['frame_end']
    token_id = span['token_id']

    span_emissions = emissions_h[frame_start:frame_end, :]
    if span_emissions.size(0) == 0:
        continue

    target_logits = span_emissions[:, token_id]
    mask = torch.ones(span_emissions.size(1), dtype=torch.bool)
    mask[token_id] = False
    other_logits = span_emissions[:, mask]
    max_other_logits = other_logits.max(dim=1).values
    gop_per_frame = target_logits - max_other_logits

    all_gop_values.extend(gop_per_frame.numpy().tolist())
    all_std_values.append(np.std(gop_per_frame.numpy()))

print(f"\nOverall frame-level GOP statistics:")
print(f"  Mean:   {np.mean(all_gop_values):>6.2f}")
print(f"  Std:    {np.std(all_gop_values):>6.2f}")
print(f"  Median: {np.median(all_gop_values):>6.2f}")

print(f"\nWithin-phoneme variance:")
print(f"  Mean std across phonemes: {np.mean(all_std_values):>6.2f}")
print(f"  Max std:                  {np.max(all_std_values):>6.2f}")

print("\n💡 RECOMMENDATIONS:")
print("   1. Use MEDIAN instead of MEAN to reduce outlier impact")
print("   2. Use IQR-based filtering to remove alignment noise")
print("   3. Require consistent poor performance across frames for 'severe'")

print("\n" + "="*70)
