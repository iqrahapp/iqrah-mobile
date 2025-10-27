"""Test GOP with reference normalization - Husary should get ~100."""
import sys
from pathlib import Path
sys.path.insert(0, str(Path.cwd()))

from src.iqrah_audio.comparison.pronunciation import score_pronunciation
from src.iqrah_audio.analysis.phoneme_from_transliteration import load_transliteration_data
import numpy as np
from collections import Counter

husary_audio = 'data/husary/surahs/001/01.mp3'
user_audio = 'static/temp/user_1_1_1759872988.webm'
surah, ayah = 1, 1

trans_data = load_transliteration_data()
transliteration = trans_data.get(f'{surah}:{ayah}', '')

print("="*70)
print("GOP WITH REFERENCE NORMALIZATION")
print("="*70)

# Test 1: Husary vs Husary (self-comparison, should be ~100)
print("\n📊 Test 1: Husary vs Husary (Reference Normalization)")
print("-"*70)
husary_result = score_pronunciation(
    husary_audio, transliteration,
    reference_audio=husary_audio,  # Self-reference
    device='cpu'
)

print(f"\n✅ Overall Score: {husary_result.overall_score:.1f}/100")
print(f"   Expected: ~100/100 (perfect self-match)")

gop_values = [s['gop_mean'] for s in husary_result.phone_scores]
print(f"\n   GOP Distribution:")
print(f"   Mean: {np.mean(gop_values):>6.2f}")
print(f"   Std:  {np.std(gop_values):>6.2f}")
print(f"   Min:  {np.min(gop_values):>6.2f}")
print(f"   Max:  {np.max(gop_values):>6.2f}")

severity_counts = Counter(s['severity'] for s in husary_result.phone_scores)
total = len(husary_result.phone_scores)
print(f"\n   Severity: ok={severity_counts.get('ok',0)}/{total}, " +
      f"mild={severity_counts.get('mild',0)}/{total}, " +
      f"severe={severity_counts.get('severe',0)}/{total}")
print(f"   Confusions: {len(husary_result.confusions)}")
print(f"   Critical Errors: {len(husary_result.critical_errors)}")

# Test 2: User vs Husary (should show meaningful difference)
print("\n\n📊 Test 2: User vs Husary (Reference Normalization)")
print("-"*70)
user_result = score_pronunciation(
    user_audio, transliteration,
    reference_audio=husary_audio,  # Husary as reference
    device='cpu'
)

print(f"\n✅ Overall Score: {user_result.overall_score:.1f}/100")
print(f"   Expected: < 100 (user has pronunciation errors)")

gop_values = [s['gop_mean'] for s in user_result.phone_scores]
print(f"\n   GOP Distribution:")
print(f"   Mean: {np.mean(gop_values):>6.2f}")
print(f"   Std:  {np.std(gop_values):>6.2f}")
print(f"   Min:  {np.min(gop_values):>6.2f}")
print(f"   Max:  {np.max(gop_values):>6.2f}")

severity_counts = Counter(s['severity'] for s in user_result.phone_scores)
total = len(user_result.phone_scores)
print(f"\n   Severity: ok={severity_counts.get('ok',0)}/{total}, " +
      f"mild={severity_counts.get('mild',0)}/{total}, " +
      f"severe={severity_counts.get('severe',0)}/{total}")
print(f"   Confusions: {len(user_result.confusions)}")
print(f"   Critical Errors: {len(user_result.critical_errors)}")

# Show top confusions
if user_result.confusions:
    print(f"\n   Top Confusions:")
    for conf in user_result.confusions[:3]:
        print(f"   • {conf['position']:.2f}s: '{conf['target_char']}' → '{conf['likely_produced']}' (GOP={conf['gop_score']:.2f})")

print("\n" + "="*70)
print("VALIDATION")
print("="*70)

if husary_result.overall_score >= 95:
    print("\n✅ PASS: Husary gets excellent score with normalization!")
    print(f"   Husary: {husary_result.overall_score:.1f}/100")
elif husary_result.overall_score >= 85:
    print("\n⚠️  ACCEPTABLE: Husary score is good")
    print(f"   Husary: {husary_result.overall_score:.1f}/100")
else:
    print("\n❌ NEEDS IMPROVEMENT: Husary score still too low")
    print(f"   Husary: {husary_result.overall_score:.1f}/100")

print(f"\n✅ Score Separation: {husary_result.overall_score - user_result.overall_score:.1f} points")
print(f"   (Husary {husary_result.overall_score:.1f} vs User {user_result.overall_score:.1f})")
print("   Good separation indicates discriminative power!")

print("\n" + "="*70)
