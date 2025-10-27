"""
Test Visualization System - Complete end-to-end test of all visualizations.

Tests:
1. DTW path visualization
2. Melody contour visualization
3. Duration bars visualization
4. Pronunciation timeline visualization
5. Interactive HTML viewer generation
"""
import sys
from pathlib import Path
sys.path.insert(0, str(Path.cwd()))

from src.iqrah_audio.analysis.phoneme_wav2vec2_ctc import extract_phonemes_wav2vec2_ctc
from src.iqrah_audio.analysis.pitch_extractor_swiftf0 import extract_pitch_swiftf0
from src.iqrah_audio.analysis.segments_loader import get_word_segments_with_text
from src.iqrah_audio.analysis.phoneme_from_transliteration import load_transliteration_data
from src.iqrah_audio.analysis.statistics_analyzer import compute_full_statistics
from src.iqrah_audio.comparison import compare_recitations

# Import visualization modules
from src.iqrah_audio.visualization.dtw_path import plot_dtw_path, create_dtw_path_dict
from src.iqrah_audio.visualization.melody_contour import plot_melody_contour, create_melody_contour_dict
from src.iqrah_audio.visualization.duration_bars import plot_duration_bars, create_duration_bars_dict
from src.iqrah_audio.visualization.pronunciation_timeline import plot_pronunciation_timeline, create_pronunciation_timeline_dict
from src.iqrah_audio.visualization.html_viewer import create_interactive_viewer

print("="*80)
print("🎨 VISUALIZATION SYSTEM - END-TO-END TEST")
print("="*80)

# Test data
user_audio = 'static/temp/user_1_1_1759872988.webm'
reference_audio = 'data/husary/surahs/001/01.mp3'
surah, ayah = 1, 1

# Load transliteration
trans_data = load_transliteration_data()
transliteration = trans_data.get(f'{surah}:{ayah}', '')

print(f"\n📖 Surah {surah}, Ayah {ayah}")
print(f"   {transliteration}")
print(f"\n🔍 Running full comparison...")

# Extract features for both
print("\n📊 Extracting student features...")
student_segments = get_word_segments_with_text(surah, ayah)
student_pitch = extract_pitch_swiftf0(user_audio)
student_phonemes = extract_phonemes_wav2vec2_ctc(
    user_audio, student_segments, transliteration, student_pitch, surah, ayah
)
student_stats = compute_full_statistics(student_phonemes, student_pitch)

print("📊 Extracting reference features...")
reference_segments = get_word_segments_with_text(surah, ayah)
reference_pitch = extract_pitch_swiftf0(reference_audio)
reference_phonemes = extract_phonemes_wav2vec2_ctc(
    reference_audio, reference_segments, transliteration, reference_pitch, surah, ayah
)
reference_stats = compute_full_statistics(reference_phonemes, reference_pitch)

# Run comparison
print("\n🔄 Running comparison with all components...")
comparison = compare_recitations(
    user_audio,
    reference_audio,
    student_phonemes,
    reference_phonemes,
    student_pitch,
    reference_pitch,
    student_stats,
    reference_stats,
    transliteration=transliteration,
    include_pronunciation=True
)

print(f"\n✅ Comparison complete:")
print(f"   Overall: {comparison['overall']:.1f}/100 (confidence: {comparison['confidence']:.2f})")
print(f"   Rhythm: {comparison['rhythm']['score']:.1f}/100")
print(f"   Melody: {comparison['melody']['score']:.1f}/100")
print(f"   Duration: {comparison['durations']['overall']:.1f}/100")
print(f"   Pronunciation: {comparison['pronunciation']['score']:.1f}/100")

# Test 1: DTW Path Visualization
print("\n" + "-"*80)
print("TEST 1: DTW Path Visualization")
print("-"*80)

try:
    # We need to extract features for visualization
    from src.iqrah_audio.comparison.features import extract_features

    print("\n🎨 Generating DTW path visualization...")
    student_feat_pack = extract_features(user_audio, student_phonemes, student_pitch, student_stats)
    reference_feat_pack = extract_features(reference_audio, reference_phonemes, reference_pitch, reference_stats)

    # Stack features from FeaturePack into numpy array [T, D]
    import numpy as np
    student_features = np.stack([
        student_feat_pack.onset_strength,
        student_feat_pack.syll_onset_mask,
        student_feat_pack.norm_time,
        student_feat_pack.df0
    ], axis=1)

    reference_features = np.stack([
        reference_feat_pack.onset_strength,
        reference_feat_pack.syll_onset_mask,
        reference_feat_pack.norm_time,
        reference_feat_pack.df0
    ], axis=1)

    dtw_dict = create_dtw_path_dict(
        comparison,
        student_features=student_features,
        reference_features=reference_features
    )

    print(f"✅ DTW visualization generated")
    print(f"   Rhythm score: {dtw_dict['rhythm_score']:.1f}/100")
    print(f"   Divergence: {dtw_dict['divergence']:.2f}")
    print(f"   Path length: {dtw_dict['path_length']} points")
    print(f"   Image size: {len(dtw_dict['image_base64'])} bytes")

    rhythm_viz_base64 = dtw_dict['image_base64']

except Exception as e:
    print(f"❌ DTW visualization failed: {e}")
    rhythm_viz_base64 = None

# Test 2: Melody Contour Visualization
print("\n" + "-"*80)
print("TEST 2: Melody Contour Visualization")
print("-"*80)

try:
    print("\n🎨 Generating melody contour visualization...")

    melody_dict = create_melody_contour_dict(
        comparison,
        student_pitch=student_pitch,
        reference_pitch=reference_pitch
    )

    print(f"✅ Melody visualization generated")
    print(f"   Melody score: {melody_dict['melody_score']:.1f}/100")
    print(f"   Pitch shift: {melody_dict['pitch_shift_cents']:.0f} cents")
    print(f"   Contour similarity: {melody_dict['contour_similarity']:.1f}%")
    print(f"   Image size: {len(melody_dict['image_base64'])} bytes")

    melody_viz_base64 = melody_dict['image_base64']

except Exception as e:
    print(f"❌ Melody visualization failed: {e}")
    melody_viz_base64 = None

# Test 3: Duration Bars Visualization
print("\n" + "-"*80)
print("TEST 3: Duration Bars Visualization")
print("-"*80)

try:
    print("\n🎨 Generating duration bars visualization...")

    duration_dict = create_duration_bars_dict(comparison)

    print(f"✅ Duration visualization generated")
    print(f"   Overall score: {duration_dict['overall_score']:.1f}/100")
    print(f"   Madd events: {duration_dict['num_events']}")
    print(f"   By type: {duration_dict['by_type']}")
    print(f"   Image size: {len(duration_dict['image_base64'])} bytes")

    duration_viz_base64 = duration_dict['image_base64']

except Exception as e:
    print(f"❌ Duration visualization failed: {e}")
    duration_viz_base64 = None

# Test 4: Pronunciation Timeline Visualization
print("\n" + "-"*80)
print("TEST 4: Pronunciation Timeline Visualization")
print("-"*80)

try:
    print("\n🎨 Generating pronunciation timeline visualization...")

    pronunciation_dict = create_pronunciation_timeline_dict(comparison)

    print(f"✅ Pronunciation visualization generated")
    print(f"   Pronunciation score: {pronunciation_dict['pronunciation_score']:.1f}/100")
    print(f"   Confusions: {pronunciation_dict['num_confusions']}")
    print(f"   Critical errors: {pronunciation_dict['num_critical']}")
    print(f"   Image size: {len(pronunciation_dict['image_base64'])} bytes")

    pronunciation_viz_base64 = pronunciation_dict['image_base64']

except Exception as e:
    print(f"❌ Pronunciation visualization failed: {e}")
    pronunciation_viz_base64 = None

# Test 5: Interactive HTML Viewer
print("\n" + "-"*80)
print("TEST 5: Interactive HTML Viewer")
print("-"*80)

try:
    print("\n🌐 Generating interactive HTML viewer...")

    output_path = f"output/visualization_test_{surah}_{ayah}.html"

    html = create_interactive_viewer(
        comparison,
        surah=surah,
        ayah=ayah,
        transliteration=transliteration,
        output_path=output_path,
        rhythm_viz_base64=rhythm_viz_base64,
        melody_viz_base64=melody_viz_base64,
        duration_viz_base64=duration_viz_base64,
        pronunciation_viz_base64=pronunciation_viz_base64
    )

    print(f"✅ HTML viewer generated")
    print(f"   Output: {output_path}")
    print(f"   HTML size: {len(html)} bytes")
    print(f"   Contains {len(comparison.get('top_issues', []))} top issues")

except Exception as e:
    print(f"❌ HTML viewer generation failed: {e}")
    import traceback
    traceback.print_exc()

# Summary
print("\n" + "="*80)
print("📊 VISUALIZATION SYSTEM TEST SUMMARY")
print("="*80)

tests_passed = 0
total_tests = 5

if rhythm_viz_base64:
    tests_passed += 1
    print("✅ TEST 1 PASSED: DTW path visualization")
else:
    print("❌ TEST 1 FAILED: DTW path visualization")

if melody_viz_base64:
    tests_passed += 1
    print("✅ TEST 2 PASSED: Melody contour visualization")
else:
    print("❌ TEST 2 FAILED: Melody contour visualization")

if duration_viz_base64:
    tests_passed += 1
    print("✅ TEST 3 PASSED: Duration bars visualization")
else:
    print("❌ TEST 3 FAILED: Duration bars visualization")

if pronunciation_viz_base64:
    tests_passed += 1
    print("✅ TEST 4 PASSED: Pronunciation timeline visualization")
else:
    print("❌ TEST 4 FAILED: Pronunciation timeline visualization")

if Path(output_path).exists():
    tests_passed += 1
    print("✅ TEST 5 PASSED: Interactive HTML viewer")
else:
    print("❌ TEST 5 FAILED: Interactive HTML viewer")

print(f"\n" + "="*80)
print(f"🎯 FINAL RESULT: {tests_passed}/{total_tests} tests passed")
print("="*80)

if tests_passed == total_tests:
    print(f"\n🎉 ALL TESTS PASSED! Visualization system is working correctly.")
    print(f"\n📂 View the interactive report at: {output_path}")
else:
    print(f"\n⚠️  {total_tests - tests_passed} test(s) failed. Review implementation.")

print("\n" + "="*80 + "\n")
