"""Test GOP scoring on Husary's recitation - should be 100/100."""
import sys
from pathlib import Path
sys.path.insert(0, str(Path.cwd()))

from src.iqrah_audio.comparison.pronunciation import score_pronunciation
from src.iqrah_audio.analysis.phoneme_from_transliteration import load_transliteration_data

# Husary's perfect recitation
husary_audio = 'data/husary/surahs/001/01.mp3'
surah, ayah = 1, 1

# Load transliteration
trans_data = load_transliteration_data()
transliteration = trans_data.get(f'{surah}:{ayah}', '')

print("="*70)
print("GOP VALIDATION: Husary's Perfect Recitation")
print("="*70)
print(f"\nAudio: {husary_audio}")
print(f"Transliteration: {transliteration}")

# Score pronunciation
print("\n🔍 Computing GOP scores...")
result = score_pronunciation(husary_audio, transliteration, device='cpu')

print(f"\n{'='*70}")
print(f"RESULTS")
print(f"{'='*70}")

print(f"\n✅ Overall Pronunciation Score: {result.overall_score:.1f}/100")
print(f"   Expected: ~100/100 (Husary's perfect recitation)")

# Distribution of GOP scores
import numpy as np
gop_values = [s['gop_mean'] for s in result.phone_scores]
print(f"\n📊 GOP Score Distribution:")
print(f"  Mean: {np.mean(gop_values):>6.2f} (higher = better, 0 = perfect)")
print(f"  Std:  {np.std(gop_values):>6.2f}")
print(f"  Min:  {np.min(gop_values):>6.2f}")
print(f"  Max:  {np.max(gop_values):>6.2f}")

# Severity breakdown
from collections import Counter
severity_counts = Counter(s['severity'] for s in result.phone_scores)
total_phones = len(result.phone_scores)

print(f"\n📊 Severity Breakdown:")
for severity in ['ok', 'mild', 'severe']:
    count = severity_counts.get(severity, 0)
    pct = count / total_phones * 100
    print(f"  {severity:>8}: {count:>3} / {total_phones} ({pct:>5.1f}%)")

# Show per-phone scores
print(f"\n📝 Per-Phone Scores (first 15):")
print(f"{'Phone':<8} {'Start':<8} {'Duration':<10} {'GOP':<8} {'Severity'}")
print("-"*70)
for s in result.phone_scores[:15]:
    print(f"{s['char']:<8} {s['start']:>6.2f}s  {s['duration']*1000:>6.0f}ms    "
          f"{s['gop_mean']:>6.2f}  {s['severity']}")

# Check for confusions (should be 0)
print(f"\n⚠️  Detected Confusions: {len(result.confusions)}")
if result.confusions:
    print("   ⚠️  WARNING: Husary should have 0 confusions!")
    for conf in result.confusions[:3]:
        print(f"   • {conf['position']:.2f}s: '{conf['target_char']}' → '{conf['likely_produced']}' (GOP={conf['gop_score']:.2f})")

# Check for critical errors (should be 0)
print(f"\n🚨 Critical Errors: {len(result.critical_errors)}")
if result.critical_errors:
    print("   ⚠️  WARNING: Husary should have 0 critical errors!")
    for err in result.critical_errors[:3]:
        print(f"   • '{err['char']}' at {err['position']:.2f}s: GOP={err['gop']:.2f}")

print("\n" + "="*70)
print("ANALYSIS")
print("="*70)

if result.overall_score >= 95:
    print("\n✅ PASS: GOP scoring working correctly!")
    print(f"   Husary's score: {result.overall_score:.1f}/100 (excellent)")
elif result.overall_score >= 85:
    print("\n⚠️  ACCEPTABLE: GOP score is good but could be better")
    print(f"   Husary's score: {result.overall_score:.1f}/100")
    print("   This might be due to audio quality or alignment precision")
else:
    print("\n❌ FAIL: GOP score too low for perfect recitation")
    print(f"   Husary's score: {result.overall_score:.1f}/100")
    print("   Expected: ~100/100")
    print("\n   Possible issues:")
    print("   - GOP threshold calibration")
    print("   - CTC alignment errors")
    print("   - Score mapping function needs adjustment")

print("\n" + "="*70)
