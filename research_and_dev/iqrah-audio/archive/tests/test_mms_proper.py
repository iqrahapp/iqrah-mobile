"""
Test Proper MMS-FA Pipeline with Word Segments
===============================================

Tests the final implementation using segments.json data.
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

from iqrah_audio.analysis.segments_loader import (
    get_ayah_segments,
    get_word_segments_with_text,
    download_audio
)
from iqrah_audio.analysis.pitch_extractor_swiftf0 import extract_pitch_swiftf0
from iqrah_audio.analysis.phoneme_mms_proper import extract_phonemes_mms_proper
from iqrah_audio.analysis.phoneme_from_transliteration import load_transliteration_data


def test_segments_loading():
    """Test that segments.json loads correctly."""
    print("\n" + "="*80)
    print("TEST 1: Segments Loading")
    print("="*80)

    # Get segment data
    seg_data = get_ayah_segments(1, 1)

    print(f"\n✓ Segment data for 1:1:")
    print(f"   Audio URL: {seg_data['audio_url']}")
    print(f"   Segments: {seg_data['segments']}")

    assert seg_data is not None, "No segment data found"
    assert 'audio_url' in seg_data, "Missing audio_url"
    assert 'segments' in seg_data, "Missing segments"
    assert len(seg_data['segments']) > 0, "No word segments"

    print(f"\n✅ PASS: Loaded {len(seg_data['segments'])} word segments")


def test_word_segments_with_text():
    """Test combining segments with Arabic text."""
    print("\n" + "="*80)
    print("TEST 2: Word Segments with Arabic Text")
    print("="*80)

    word_segs = get_word_segments_with_text(1, 1)

    print(f"\n✓ Word segments with text:")
    for seg in word_segs:
        print(f"   {seg['word_index']}: '{seg['text']}' "
              f"[{seg['start_ms']}-{seg['end_ms']}ms] ({seg['duration_ms']}ms)")

    assert len(word_segs) > 0, "No word segments"
    assert all('text' in s for s in word_segs), "Missing text"
    assert all('start_ms' in s for s in word_segs), "Missing start_ms"

    print(f"\n✅ PASS: {len(word_segs)} word segments with Arabic text")


def test_audio_download():
    """Test audio downloading and caching."""
    print("\n" + "="*80)
    print("TEST 3: Audio Download")
    print("="*80)

    seg_data = get_ayah_segments(1, 1)
    audio_url = seg_data['audio_url']

    print(f"\n✓ Downloading: {audio_url}")
    audio_path = download_audio(audio_url)

    print(f"   ✓ Cached at: {audio_path}")

    assert Path(audio_path).exists(), "Audio file not downloaded"

    print(f"\n✅ PASS: Audio downloaded and cached")


def test_mms_fa_proper_pipeline():
    """Test full MMS-FA pipeline with word segments."""
    print("\n" + "="*80)
    print("TEST 4: MMS-FA Proper Pipeline")
    print("="*80)

    # Get data
    seg_data = get_ayah_segments(1, 1)
    audio_path = download_audio(seg_data['audio_url'])
    word_segments = get_word_segments_with_text(1, 1)

    trans_data = load_transliteration_data()
    transliteration = trans_data['1:1']

    print(f"\n✓ Transliteration: {transliteration}")
    print(f"✓ Word segments: {len(word_segments)}")

    # Extract pitch
    print(f"\n1️⃣ Extracting pitch...")
    pitch_data = extract_pitch_swiftf0(audio_path)
    print(f"   ✓ Duration: {pitch_data['duration']:.2f}s")

    # Extract phonemes with MMS-FA
    print(f"\n2️⃣ Extracting phonemes with MMS-FA...")
    phonemes = extract_phonemes_mms_proper(
        audio_path=audio_path,
        word_segments=word_segments,
        transliteration=transliteration,
        pitch_data=pitch_data,
        device='cpu'
    )

    print(f"\n📊 Results:")
    print(f"   Phonemes: {len(phonemes)}")

    # Show first 10
    print(f"\n   First 10 phonemes:")
    print(f"   {'#':>3} {'Phoneme':15s} {'Start':>8s} {'End':>8s} {'Dur':>7s} {'Word':>5s}")
    print(f"   {'-'*60}")

    for i, p in enumerate(phonemes[:10]):
        print(f"   {i+1:3d} {p['phoneme']:15s} "
              f"{p['start']:8.3f} {p['end']:8.3f} "
              f"{p['duration']:7.3f} {p.get('word_index', -1):5d}")

    # Validation
    assert len(phonemes) > 0, "No phonemes extracted"

    # Check all have required fields
    for p in phonemes:
        assert 'phoneme' in p, "Missing phoneme"
        assert 'start' in p, "Missing start"
        assert 'end' in p, "Missing end"
        assert p['start'] < p['end'], "Invalid timing"

    # Check monotonic
    for i in range(len(phonemes) - 1):
        assert phonemes[i]['end'] <= phonemes[i+1]['start'] + 0.1, \
            f"Non-monotonic at {i}"

    # Check coverage
    total_duration = pitch_data['duration']
    phoneme_coverage = phonemes[-1]['end'] if phonemes else 0
    coverage_ratio = phoneme_coverage / total_duration

    print(f"\n   Coverage: {phoneme_coverage:.2f}s / {total_duration:.2f}s = {coverage_ratio:.1%}")

    # Should have good coverage (not necessarily 100% due to word windowing)
    assert coverage_ratio > 0.7, f"Low coverage: {coverage_ratio:.1%}"

    print(f"\n✅ PASS: MMS-FA pipeline works with word segments!")


if __name__ == "__main__":
    print("\n" + "="*80)
    print("MMS-FA PROPER PIPELINE TEST")
    print("="*80)
    print("\nTesting AI Report 2's approach with segments.json data")
    print("="*80)

    try:
        test_segments_loading()
        test_word_segments_with_text()
        test_audio_download()
        test_mms_fa_proper_pipeline()

        print("\n" + "="*80)
        print("✅ ALL TESTS PASSED!")
        print("="*80)
        print("\nMMS-FA pipeline is working correctly with:")
        print("  ✓ Word-level segments from segments.json")
        print("  ✓ Audio downloading and caching")
        print("  ✓ Proper phoneme alignment")
        print("  ✓ Pitch integration")
        print("\n" + "="*80 + "\n")

    except Exception as e:
        print(f"\n❌ TEST FAILED: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
