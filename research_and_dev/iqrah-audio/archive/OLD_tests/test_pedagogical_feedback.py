"""
Test Pedagogical Feedback - Rich pronunciation guidance for non-Arabic speakers.

This demonstrates the dream feature: telling users exactly what mistakes they're making
with detailed articulation guidance.
"""
import sys
from pathlib import Path
sys.path.insert(0, str(Path.cwd()))

from src.iqrah_audio.comparison.pronunciation import score_pronunciation, ARABIC_CONFUSION_SETS
from src.iqrah_audio.analysis.phoneme_from_transliteration import load_transliteration_data

# User recitation
user_audio = 'static/temp/user_1_1_1759872988.webm'
reference_audio = 'data/husary/surahs/001/01.mp3'
surah, ayah = 1, 1

# Load transliteration
trans_data = load_transliteration_data()
transliteration = trans_data.get(f'{surah}:{ayah}', '')

print("="*70)
print("🎓 PEDAGOGICAL FEEDBACK SYSTEM")
print("   Pronunciation Guidance for Quran Recitation")
print("="*70)

print(f"\n📖 Surah {surah}, Ayah {ayah}")
print(f"   {transliteration}")

# Score pronunciation with reference normalization
print("\n🔍 Analyzing your pronunciation...")
result = score_pronunciation(user_audio, transliteration, reference_audio=reference_audio, device='cpu')

print(f"\n{'='*70}")
print(f"📊 YOUR PRONUNCIATION ASSESSMENT")
print(f"{'='*70}")

print(f"\n✅ Overall Score: {result.overall_score:.1f}/100")

# Severity breakdown
from collections import Counter
severity_counts = Counter(s['severity'] for s in result.phone_scores)
total_phones = len(result.phone_scores)

print(f"\n📊 Performance Breakdown:")
print(f"   ✅ Correct pronunciation:  {severity_counts.get('ok', 0):>3} / {total_phones} ({severity_counts.get('ok', 0)/total_phones*100:>5.1f}%)")
print(f"   ⚠️  Mild errors:           {severity_counts.get('mild', 0):>3} / {total_phones} ({severity_counts.get('mild', 0)/total_phones*100:>5.1f}%)")
print(f"   🚨 Severe errors:          {severity_counts.get('severe', 0):>3} / {total_phones} ({severity_counts.get('severe', 0)/total_phones*100:>5.1f}%)")

# Show detected confusions with rich feedback
if result.confusions:
    print(f"\n{'='*70}")
    print(f"🎯 SPECIFIC PRONUNCIATION ISSUES ({len(result.confusions)} detected)")
    print(f"{'='*70}")

    for i, conf in enumerate(result.confusions, 1):
        severity_emoji = "🚨" if conf['severity'] == 'severe' else "⚠️"
        print(f"\n{severity_emoji} Issue #{i} at {conf['position']:.2f}s ({conf['severity'].upper()})")
        print(f"   Expected sound: '{conf.get('target_char', '?')}' (Arabic: {conf.get('target_arabic', '?')})")
        print(f"   You produced:   '{conf.get('likely_produced', '?')}' (Arabic: {conf.get('likely_produced_arabic', '?')})")
        print(f"   GOP Score:      {conf['gop_score']:.2f} (lower = worse)")

        # Show detailed articulation guidance if available
        if conf.get('confusion_details'):
            details = conf['confusion_details']
            print(f"\n   📚 ARTICULATION GUIDANCE:")
            print(f"      {details.get('description', '')}")
            print(f"\n   💡 HOW TO FIX IT:")
            # Wrap the tip text nicely
            tip = details.get('tip', '')
            import textwrap
            wrapped_tip = textwrap.fill(tip, width=60, initial_indent='      ', subsequent_indent='      ')
            print(wrapped_tip)

else:
    print(f"\n✅ Excellent! No significant pronunciation confusions detected!")

# Show common Arabic pronunciation challenges
print(f"\n{'='*70}")
print(f"📖 COMMON ARABIC PRONUNCIATION CHALLENGES")
print(f"   (Reference Guide for Non-Arabic Speakers)")
print(f"{'='*70}")

# Show examples from each category
for category, sets in ARABIC_CONFUSION_SETS.items():
    print(f"\n🔹 {category.upper()} SOUNDS:")
    for conf_set in sets[:2]:  # Show first 2 from each category
        arabic_chars = ' vs '.join(conf_set['arabic'])
        print(f"\n   {arabic_chars} — {conf_set['description']}")
        print(f"   💡 {conf_set['tip']}")

print(f"\n{'='*70}")
print(f"🎯 NEXT STEPS")
print(f"{'='*70}")

if result.critical_errors:
    print(f"\n🚨 Priority: Fix {len(result.critical_errors)} critical errors first")
    for err in result.critical_errors[:3]:
        print(f"   • '{err['char']}' at {err['position']:.2f}s")
else:
    print(f"\n✅ Great job! Focus on perfecting the mild errors.")

print(f"\n💪 Practice Tips:")
print(f"   1. Record yourself repeatedly saying the problem sounds in isolation")
print(f"   2. Listen to the reference reciter for these specific sounds")
print(f"   3. Practice with a mirror to check tongue and mouth positions")
print(f"   4. Ask an Arabic speaker to verify your pronunciation")

print("\n" + "="*70)
print("✨ Keep practicing! May Allah make it easy for you. ✨")
print("="*70 + "\n")
