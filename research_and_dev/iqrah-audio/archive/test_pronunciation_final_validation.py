"""
Final Validation - Pronunciation Scoring System
================================================

Demonstrates all key features:
1. Phoneme-by-phoneme GOP delta comparison
2. Perfect recitation validation (100/100)
3. User recitation with specific feedback
4. Integration with comparison engine
"""
import sys
from pathlib import Path
sys.path.insert(0, str(Path.cwd()))

from src.iqrah_audio.comparison.pronunciation import score_pronunciation
from src.iqrah_audio.analysis.phoneme_from_transliteration import load_transliteration_data

print("="*80)
print("🎯 PRONUNCIATION SCORING - FINAL VALIDATION")
print("="*80)

# Load transliteration
trans_data = load_transliteration_data()
transliteration = trans_data.get('1:1', '')

print(f"\n📖 Surah 1, Ayah 1: {transliteration}")

# Test 1: Perfect recitation (Husary vs Husary)
print("\n" + "-"*80)
print("TEST 1: Perfect Recitation Validation")
print("-"*80)

husary_audio = 'data/husary/surahs/001/01.mp3'
print("\n🔍 Comparing Husary to himself (should score 100/100)...")

husary_result = score_pronunciation(
    husary_audio,
    transliteration,
    reference_audio=husary_audio,
    device='cpu'
)

print(f"\n✅ RESULT: {husary_result.overall_score:.1f}/100")
print(f"   • Correct: {sum(1 for s in husary_result.phone_scores if s.get('severity')=='ok')}/{len(husary_result.phone_scores)}")
print(f"   • Mild errors: {sum(1 for s in husary_result.phone_scores if s.get('severity')=='mild')}")
print(f"   • Severe errors: {sum(1 for s in husary_result.phone_scores if s.get('severity')=='severe')}")
print(f"   • Confusions: {len(husary_result.confusions)}")

if husary_result.overall_score >= 99.0:
    print("\n   ✅ PASS: Perfect recitation scores near 100/100")
else:
    print(f"\n   ❌ FAIL: Expected ~100, got {husary_result.overall_score:.1f}")

# Test 2: User recitation with errors
print("\n" + "-"*80)
print("TEST 2: User Recitation with Error Detection")
print("-"*80)

user_audio = 'static/temp/user_1_1_1759872988.webm'
print("\n🔍 Comparing user recitation to Husary...")

user_result = score_pronunciation(
    user_audio,
    transliteration,
    reference_audio=husary_audio,
    device='cpu'
)

print(f"\n✅ RESULT: {user_result.overall_score:.1f}/100")
print(f"   • Correct: {sum(1 for s in user_result.phone_scores if s.get('severity')=='ok')}/{len(user_result.phone_scores)}")
print(f"   • Mild errors: {sum(1 for s in user_result.phone_scores if s.get('severity')=='mild')}")
print(f"   • Severe errors: {sum(1 for s in user_result.phone_scores if s.get('severity')=='severe')}")
print(f"   • Confusions: {len(user_result.confusions)}")

# Show detected issues
if user_result.confusions:
    print(f"\n📋 Detected Issues:")
    for i, conf in enumerate(user_result.confusions[:3], 1):
        severity_label = conf['severity'].upper()
        print(f"   {i}. [{severity_label}] at {conf['position']:.2f}s: "
              f"'{conf['target_char']}' → '{conf['likely_produced']}' "
              f"(GOP: {conf['gop_score']:.2f})")

# Test 3: Score separation
print("\n" + "-"*80)
print("TEST 3: Discriminative Power")
print("-"*80)

score_diff = husary_result.overall_score - user_result.overall_score
print(f"\n📊 Score Separation: {score_diff:.1f} points")
print(f"   • Perfect (Husary): {husary_result.overall_score:.1f}/100")
print(f"   • Good (User): {user_result.overall_score:.1f}/100")

if score_diff >= 5.0:
    print(f"\n   ✅ PASS: Good separation ({score_diff:.1f} points)")
else:
    print(f"\n   ⚠️  WARNING: Low separation ({score_diff:.1f} points)")

# Test 4: Pedagogical feedback quality
print("\n" + "-"*80)
print("TEST 4: Pedagogical Feedback Quality")
print("-"*80)

print("\n🔍 Checking feedback richness...")

feedback_quality_score = 0

# Check if we have detailed confusion info
if user_result.confusions:
    has_positions = all('position' in c for c in user_result.confusions)
    has_targets = all('target_char' in c for c in user_result.confusions)
    has_produced = all('likely_produced' in c for c in user_result.confusions)
    has_severity = all('severity' in c for c in user_result.confusions)

    if has_positions:
        feedback_quality_score += 1
        print("   ✅ Timestamps provided")
    if has_targets:
        feedback_quality_score += 1
        print("   ✅ Target phonemes identified")
    if has_produced:
        feedback_quality_score += 1
        print("   ✅ Likely produced alternatives detected")
    if has_severity:
        feedback_quality_score += 1
        print("   ✅ Severity classification working")

    print(f"\n📊 Feedback Quality Score: {feedback_quality_score}/4")

    if feedback_quality_score >= 3:
        print("\n   ✅ PASS: Rich pedagogical feedback available")
    else:
        print("\n   ❌ FAIL: Insufficient feedback detail")
else:
    print("   ⚠️  No confusions to check feedback quality")

# Summary
print("\n" + "="*80)
print("📊 VALIDATION SUMMARY")
print("="*80)

tests_passed = 0
total_tests = 4

# Test 1: Perfect recitation
if husary_result.overall_score >= 99.0:
    tests_passed += 1
    print("\n✅ TEST 1 PASSED: Perfect recitation scores 100/100")
else:
    print(f"\n❌ TEST 1 FAILED: Expected 100, got {husary_result.overall_score:.1f}")

# Test 2: User recitation detects errors
if user_result.overall_score < husary_result.overall_score and len(user_result.confusions) > 0:
    tests_passed += 1
    print("✅ TEST 2 PASSED: User recitation detects errors correctly")
else:
    print("❌ TEST 2 FAILED: Error detection not working")

# Test 3: Score separation
if score_diff >= 5.0:
    tests_passed += 1
    print(f"✅ TEST 3 PASSED: Good score separation ({score_diff:.1f} points)")
else:
    print(f"❌ TEST 3 FAILED: Insufficient separation ({score_diff:.1f} points)")

# Test 4: Pedagogical feedback
if feedback_quality_score >= 3:
    tests_passed += 1
    print("✅ TEST 4 PASSED: Rich pedagogical feedback available")
else:
    print("❌ TEST 4 FAILED: Insufficient feedback detail")

print(f"\n" + "="*80)
print(f"🎯 FINAL RESULT: {tests_passed}/{total_tests} tests passed")
print("="*80)

if tests_passed == total_tests:
    print("\n🎉 ALL TESTS PASSED! Pronunciation scoring system is working correctly.")
else:
    print(f"\n⚠️  {total_tests - tests_passed} test(s) failed. Review implementation.")

print("\n" + "="*80 + "\n")
