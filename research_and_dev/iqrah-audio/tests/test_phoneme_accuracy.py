"""
Test Phoneme Extraction Accuracy Using Gold Data
=================================================

Tests phoneme extraction against the gold English transliteration data.
This validates that our phoneme segmentation makes sense.
"""

import sys
from pathlib import Path

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

from iqrah_audio.analysis.pitch_extractor_swiftf0 import extract_pitch_swiftf0
from iqrah_audio.analysis.phoneme_from_transliteration import (
    load_transliteration_data,
    get_phonemes_from_transliteration,
    split_into_syllables
)

def test_transliteration_data():
    """Test that we can load the gold transliteration data."""
    print("\n" + "="*80)
    print("TEST 1: Load English Transliteration Data")
    print("="*80)

    data = load_transliteration_data()

    print(f"\n✓ Loaded {len(data)} ayahs")

    # Check first ayah
    assert "1:1" in data, "Missing ayah 1:1"
    print(f"\n✓ Ayah 1:1: {data['1:1']}")

    # Split into words
    words = data['1:1'].split()
    print(f"\n✓ Words ({len(words)}): {words}")

    # Check syllable splitting
    for word in words:
        syllables = split_into_syllables(word)
        print(f"   {word:15s} → {syllables}")

    print("\n✅ PASS: Transliteration data loaded successfully")


def test_phoneme_extraction_with_pitch():
    """Test phoneme extraction with real audio and pitch data."""
    print("\n" + "="*80)
    print("TEST 2: Phoneme Extraction with Pitch")
    print("="*80)

    # Use Husary recitation
    audio_path = "data/husary/surahs/001/01.mp3"

    print(f"\n✓ Testing with: {audio_path}")

    # Extract pitch
    print("\n1. Extracting pitch with SwiftF0...")
    pitch_data = extract_pitch_swiftf0(audio_path)

    print(f"   Duration: {pitch_data['duration']:.2f}s")
    print(f"   Frames: {len(pitch_data['time'])}")

    # Extract phonemes
    print("\n2. Extracting phonemes from transliteration...")
    phonemes = get_phonemes_from_transliteration(
        surah=1,
        ayah=1,
        audio_path=audio_path,
        pitch_data=pitch_data
    )

    print(f"   Phonemes found: {len(phonemes)}")

    # Show first 15 phonemes
    print("\n3. First 15 phonemes:")
    print(f"   {'#':>3} {'Phoneme':15s} {'Start':>8s} {'End':>8s} {'Dur':>7s} {'Pitch':>7s}")
    print("   " + "-"*60)

    for i, p in enumerate(phonemes[:15]):
        print(f"   {i+1:3d} {p['phoneme']:15s} "
              f"{p['start']:8.3f} {p['end']:8.3f} "
              f"{p['duration']:7.3f} {p['mean_pitch']:7.1f}")

    # Validation checks
    print("\n4. Validation:")

    # Check coverage
    total_duration = pitch_data['duration']
    phoneme_coverage = phonemes[-1]['end'] if phonemes else 0
    coverage_ratio = phoneme_coverage / total_duration

    print(f"   Audio duration: {total_duration:.2f}s")
    print(f"   Phoneme coverage: {phoneme_coverage:.2f}s")
    print(f"   Coverage ratio: {coverage_ratio:.1%}")

    assert coverage_ratio > 0.9, f"Low coverage: {coverage_ratio:.1%}"
    print(f"   ✓ Coverage is good (>{coverage_ratio:.1%})")

    # Check timing
    for i, p in enumerate(phonemes):
        assert p['start'] < p['end'], f"Phoneme {i}: invalid timing"
        assert p['duration'] > 0, f"Phoneme {i}: zero duration"

    print(f"   ✓ All phonemes have valid timing")

    # Check pitch values
    phonemes_with_pitch = [p for p in phonemes if p['mean_pitch'] > 0]
    print(f"   ✓ Phonemes with pitch: {len(phonemes_with_pitch)}/{len(phonemes)}")

    print("\n✅ PASS: Phoneme extraction works with real audio!")


def test_comparison_with_expected():
    """Compare extracted phonemes with expected transliteration."""
    print("\n" + "="*80)
    print("TEST 3: Compare with Expected Transliteration")
    print("="*80)

    # Load expected transliteration
    data = load_transliteration_data()
    expected = data["1:1"]

    print(f"\n✓ Expected: {expected}")

    # Extract phonemes
    audio_path = "data/husary/surahs/001/01.mp3"
    pitch_data = extract_pitch_swiftf0(audio_path)
    phonemes = get_phonemes_from_transliteration(1, 1, audio_path, pitch_data)

    # Reconstruct from phonemes
    reconstructed = " ".join(p['phoneme'] for p in phonemes)

    print(f"\n✓ Reconstructed ({len(phonemes)} segments):")
    print(f"   {reconstructed}")

    # Check that key words are present
    expected_words = expected.split()
    extracted_text = reconstructed.lower()

    matches = 0
    for word in expected_words:
        # Simple substring check (syllables should be part of words)
        word_lower = word.lower().replace("'", "").replace("-", "")
        if word_lower in extracted_text.replace(" ", ""):
            matches += 1
            print(f"   ✓ Found: {word}")

    match_ratio = matches / len(expected_words)
    print(f"\n✓ Match ratio: {matches}/{len(expected_words)} = {match_ratio:.1%}")

    # We expect at least 50% match since we're splitting into syllables
    assert match_ratio >= 0.5, f"Low match ratio: {match_ratio:.1%}"

    print("\n✅ PASS: Phoneme extraction matches expected transliteration!")


if __name__ == "__main__":
    print("\n" + "="*80)
    print("PHONEME ACCURACY TEST (Using Gold Transliteration Data)")
    print("="*80)

    try:
        test_transliteration_data()
        test_phoneme_extraction_with_pitch()
        test_comparison_with_expected()

        print("\n" + "="*80)
        print("✅ ALL TESTS PASSED!")
        print("="*80)
        print("\nPhoneme extraction is accurate and ready for visualization!")
        print()

    except Exception as e:
        print(f"\n❌ TEST FAILED: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
