"""Test pronunciation scoring with SSL-GOP."""
import sys
from pathlib import Path
sys.path.insert(0, str(Path.cwd()))

from src.iqrah_audio.comparison.pronunciation import score_pronunciation
from src.iqrah_audio.analysis.phoneme_from_transliteration import load_transliteration_data

# User recitation
user_audio = 'static/temp/user_1_1_1759872988.webm'
surah, ayah = 1, 1

# Load transliteration
trans_data = load_transliteration_data()
transliteration = trans_data.get(f'{surah}:{ayah}', '')

print("="*70)
print("PRONUNCIATION QUALITY ASSESSMENT (SSL-GOP)")
print("="*70)
print(f"\nAudio: {user_audio}")
print(f"Transliteration: {transliteration}")

# Score pronunciation
print("\n📊 Computing pronunciation scores...")
result = score_pronunciation(user_audio, transliteration, device='cpu')

print(f"\n{'='*70}")
print(f"RESULTS")
print(f"{'='*70}")

print(f"\n✅ Overall Pronunciation Score: {result.overall_score:.1f}/100")

print(f"\n📝 Per-Phone Scores ({len(result.phone_scores)} phones):")
print(f"{'Phone':<8} {'Start':<8} {'Duration':<10} {'GOP':<8} {'Severity'}")
print("-"*70)
for s in result.phone_scores[:15]:  # Show first 15
    print(f"{s['char']:<8} {s['start']:>6.2f}s  {s['duration']*1000:>6.0f}ms    "
          f"{s['gop_mean']:>6.2f}  {s['severity']}")

if len(result.phone_scores) > 15:
    print(f"... and {len(result.phone_scores) - 15} more phones")

# Show confusions
if result.confusions:
    print(f"\n⚠️  Detected Confusions ({len(result.confusions)}):")
    print("-"*70)
    for conf in result.confusions:
        print(f"\nPosition: {conf['position']:.2f}s")
        print(f"  Expected: '{conf['target_char']}' → Likely produced: '{conf['likely_produced']}'")
        print(f"  GOP Score: {conf['gop_score']:.2f}")
        print(f"  Severity: {conf['severity']}")
        if conf.get('confusion_type'):
            print(f"  Type: {conf['confusion_type']}")
else:
    print(f"\n✅ No significant pronunciation confusions detected!")

# Show critical errors
if result.critical_errors:
    print(f"\n🚨 Critical Errors ({len(result.critical_errors)}):")
    print("-"*70)
    for err in result.critical_errors:
        print(f"  '{err['char']}' at {err['position']:.2f}s: GOP={err['gop']:.2f} ({err['severity']})")
else:
    print(f"\n✅ No critical pronunciation errors!")

# Distribution of GOP scores
import numpy as np
gop_values = [s['gop_mean'] for s in result.phone_scores]
print(f"\n📊 GOP Score Distribution:")
print(f"  Mean: {np.mean(gop_values):.2f}")
print(f"  Std:  {np.std(gop_values):.2f}")
print(f"  Min:  {np.min(gop_values):.2f}")
print(f"  Max:  {np.max(gop_values):.2f}")

# Severity breakdown
from collections import Counter
severity_counts = Counter(s['severity'] for s in result.phone_scores)
print(f"\n📊 Severity Breakdown:")
for severity, count in severity_counts.most_common():
    pct = count / len(result.phone_scores) * 100
    print(f"  {severity:>8}: {count:>3} ({pct:>5.1f}%)")

print("\n" + "="*70)
