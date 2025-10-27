"""Test that Madd scoring works end-to-end in comparison."""

import sys
from pathlib import Path

# Add src to path
sys.path.insert(0, str(Path(__file__).parent))

from src.iqrah_audio.analysis.phoneme_wav2vec2_ctc import extract_phonemes_wav2vec2_ctc
from src.iqrah_audio.analysis.pitch_extractor_swiftf0 import extract_pitch_swiftf0
from src.iqrah_audio.analysis.segments_loader import get_word_segments_with_text
from src.iqrah_audio.analysis.phoneme_from_transliteration import load_transliteration_data
from src.iqrah_audio.analysis.statistics_analyzer import compute_full_statistics
from src.iqrah_audio.comparison import compare_recitations

def test_madd_scoring():
    """Test that Madd scoring is included in comparison results."""
    print("\n" + "="*70)
    print("Testing End-to-End Madd Scoring in Comparison")
    print("="*70)

    # Test with two recitations of the same ayah
    student_audio = "data/me/surahs/001/01.mp3"
    reference_audio = "data/husary/surahs/001/01.mp3"
    surah, ayah = 1, 1

    # Load shared data
    print("\n1️⃣ Loading transliteration and word segments...")
    word_segments = get_word_segments_with_text(surah, ayah)
    trans_data = load_transliteration_data()
    transliteration = trans_data.get(f"{surah}:{ayah}", "")
    print(f"   ✓ Transliteration: {transliteration}")

    # Process student
    print("\n2️⃣ Analyzing student recitation...")
    student_pitch = extract_pitch_swiftf0(student_audio)
    student_phonemes = extract_phonemes_wav2vec2_ctc(
        audio_path=student_audio,
        word_segments=word_segments,
        transliteration=transliteration,
        pitch_data=student_pitch,
        surah=surah,
        ayah=ayah
    )
    student_stats = compute_full_statistics(student_phonemes, student_pitch)
    print(f"   ✓ {len(student_phonemes)} phonemes extracted")

    # Count Madd in student
    student_madd = [p for p in student_phonemes if p.get('tajweed_rule') and 'madda' in p['tajweed_rule']]
    print(f"   ✓ {len(student_madd)} phonemes with Madd rules")

    # Process reference
    print("\n3️⃣ Analyzing reference recitation...")
    reference_pitch = extract_pitch_swiftf0(reference_audio)
    reference_phonemes = extract_phonemes_wav2vec2_ctc(
        audio_path=reference_audio,
        word_segments=word_segments,
        transliteration=transliteration,
        pitch_data=reference_pitch,
        surah=surah,
        ayah=ayah
    )
    reference_stats = compute_full_statistics(reference_phonemes, reference_pitch)
    print(f"   ✓ {len(reference_phonemes)} phonemes extracted")

    # Count Madd in reference
    reference_madd = [p for p in reference_phonemes if p.get('tajweed_rule') and 'madda' in p['tajweed_rule']]
    print(f"   ✓ {len(reference_madd)} phonemes with Madd rules")

    # Run comparison
    print("\n4️⃣ Running comparison with Madd scoring...")
    comparison = compare_recitations(
        student_audio_path=student_audio,
        reference_audio_path=reference_audio,
        student_phonemes=student_phonemes,
        reference_phonemes=reference_phonemes,
        student_pitch=student_pitch,
        reference_pitch=reference_pitch,
        student_stats=student_stats,
        reference_stats=reference_stats
    )

    # Check results
    print("\n" + "="*70)
    print("Comparison Results:")
    print("="*70)

    print(f"\n📊 Overall Score: {comparison['overall']}")
    print(f"   Confidence: {comparison['confidence']}")
    print(f"\n🎵 Rhythm Score: {comparison['rhythm']['score']}")
    print(f"🎼 Melody Score: {comparison['melody']['score']}")

    # Check if duration/Madd scoring exists
    if 'durations' in comparison:
        duration = comparison['durations']
        print(f"⏱️  Duration/Madd Score: {duration.get('overall', 'N/A')}")

        if 'by_type' in duration:
            print(f"\n   Madd by type:")
            for madd_type, data in duration['by_type'].items():
                print(f"     {madd_type}: avg={data.get('average', 'N/A')}, count={data.get('count', 0)}")

        if 'critical_issues' in duration and duration['critical_issues']:
            print(f"\n   Critical issues: {len(duration['critical_issues'])}")
            for issue in duration['critical_issues'][:3]:
                print(f"     - Phoneme '{issue['phoneme']}': expected {issue['expected']} counts, got {issue['actual']} ({issue['severity']})")

        if 'notes' in duration and duration['notes']:
            print(f"\n   Notes:")
            for note in duration['notes'][:3]:
                print(f"     - {note}")
    else:
        print("⚠️  WARNING: Duration/Madd scoring NOT found in comparison results!")

    print("\n" + "="*70)

    # Verify Madd scoring is present
    assert 'durations' in comparison, "Duration/Madd scoring missing from comparison!"
    assert 'overall' in comparison['durations'], "Duration overall score missing!"

    print("\n✓ SUCCESS: Madd scoring is working end-to-end!")
    print("="*70 + "\n")

if __name__ == "__main__":
    test_madd_scoring()
