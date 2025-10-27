"""
Test Simple Phoneme Extraction
===============================

Quick test to verify the simple phoneme extractor works correctly.
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

from iqrah_audio.analysis.pitch_extractor_swiftf0 import extract_pitch_swiftf0
from iqrah_audio.analysis.phoneme_simple import extract_phonemes_simple
from iqrah_audio.analysis.phoneme_from_transliteration import load_transliteration_data


def test_simple_phoneme_extraction():
    """Test simple phoneme extraction with real audio."""
    print("\n" + "="*80)
    print("TEST: Simple Phoneme Extraction")
    print("="*80)

    # Use real data
    audio_path = "data/husary/surahs/001/01.mp3"

    # Load transliteration
    trans_data = load_transliteration_data()
    transliteration = trans_data["1:1"]

    print(f"\n   Transliteration: {transliteration}")

    # Extract pitch
    print(f"\n1️⃣ Extracting pitch...")
    pitch_data = extract_pitch_swiftf0(audio_path)
    print(f"   ✓ Duration: {pitch_data['duration']:.2f}s")

    # Extract phonemes
    print(f"\n2️⃣ Extracting phonemes...")
    phonemes = extract_phonemes_simple(
        transliteration=transliteration,
        pitch_data=pitch_data
    )

    print(f"\n📊 Results:")
    print(f"   Phonemes: {len(phonemes)}")

    # Check coverage
    total_duration = pitch_data['duration']
    phoneme_coverage = phonemes[-1]['end'] if phonemes else 0
    coverage_ratio = phoneme_coverage / total_duration

    print(f"   Coverage: {phoneme_coverage:.2f}s / {total_duration:.2f}s = {coverage_ratio:.1%}")

    # Should have 100% coverage since we distribute evenly
    assert coverage_ratio > 0.95, f"Coverage too low: {coverage_ratio:.1%}"

    # Check timing
    for i, p in enumerate(phonemes):
        assert p['start'] < p['end'], f"Phoneme {i} has invalid timing"
        assert p['duration'] > 0, f"Phoneme {i} has zero duration"

    # Check monotonic
    for i in range(len(phonemes) - 1):
        assert phonemes[i]['end'] <= phonemes[i+1]['start'] + 0.01, \
            f"Non-monotonic at {i}"

    # Show first 10 phonemes
    print(f"\n   First 10 phonemes:")
    print(f"   {'#':>3} {'Phoneme':15s} {'Start':>8s} {'End':>8s} {'Dur':>7s} {'Pitch':>7s} {'Tajweed':12s}")
    print(f"   {'-'*70}")

    for i, p in enumerate(phonemes[:10]):
        tajweed = p['tajweed_rule'] or 'none'
        print(f"   {i+1:3d} {p['phoneme']:15s} "
              f"{p['start']:8.3f} {p['end']:8.3f} "
              f"{p['duration']:7.3f} {p['mean_pitch']:7.1f} {tajweed:12s}")

    print("\n✅ PASS: Simple phoneme extraction working correctly!")


if __name__ == "__main__":
    try:
        test_simple_phoneme_extraction()
        print("\n" + "="*80)
        print("✅ TEST PASSED!")
        print("="*80 + "\n")
    except Exception as e:
        print(f"\n❌ TEST FAILED: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
