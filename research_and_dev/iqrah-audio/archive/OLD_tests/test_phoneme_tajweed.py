"""Test phoneme extraction with Tajweed annotation."""

import sys
from pathlib import Path

# Add src to path
sys.path.insert(0, str(Path(__file__).parent))

from src.iqrah_audio.analysis.phoneme_wav2vec2_ctc import extract_phonemes_wav2vec2_ctc
from src.iqrah_audio.analysis.pitch_extractor_swiftf0 import extract_pitch_swiftf0
from src.iqrah_audio.analysis.segments_loader import get_word_segments_with_text
from src.iqrah_audio.analysis.phoneme_from_transliteration import load_transliteration_data

def test_phoneme_tajweed():
    """Test that phonemes are annotated with Tajweed rules."""
    print("\n" + "="*70)
    print("Testing Phoneme Extraction with Tajweed Annotation")
    print("="*70)

    # Test files
    audio_path = "data/husary/surahs/001/01.mp3"
    surah, ayah = 1, 1

    # Load data
    print("\n1️⃣ Loading data...")
    word_segments = get_word_segments_with_text(surah, ayah)
    trans_data = load_transliteration_data()
    transliteration = trans_data.get(f"{surah}:{ayah}", "")
    print(f"   ✓ Transliteration: {transliteration}")

    # Extract pitch
    print("\n2️⃣ Extracting pitch...")
    pitch_data = extract_pitch_swiftf0(audio_path)
    print(f"   ✓ Duration: {pitch_data['duration']:.2f}s")

    # Extract phonemes with Tajweed
    print("\n3️⃣ Extracting phonemes with Tajweed...")
    phonemes = extract_phonemes_wav2vec2_ctc(
        audio_path=audio_path,
        word_segments=word_segments,
        transliteration=transliteration,
        pitch_data=pitch_data,
        surah=surah,
        ayah=ayah
    )
    print(f"   ✓ Extracted {len(phonemes)} phonemes")

    # Check for Madd rules
    print("\n" + "="*70)
    print("Phonemes with Madd Rules:")
    print("="*70)
    print(f"{'Phoneme':<15} {'Time':<20} {'Duration':<10} {'Tajweed Rule'}")
    print("-" * 70)

    madd_count = 0
    for p in phonemes:
        rule = p.get('tajweed_rule')
        if rule and 'madda' in str(rule):
            time_str = f"{p['start']:.3f}-{p['end']:.3f}s"
            dur_str = f"{p['duration']*1000:.0f}ms"
            print(f"{p['phoneme']:<15} {time_str:<20} {dur_str:<10} {rule}")
            madd_count += 1

    print(f"\n✓ Found {madd_count} phonemes with Madd rules")

    # Show all phonemes with any Tajweed rule
    print("\n" + "="*70)
    print("All Phonemes with Tajweed Rules:")
    print("="*70)
    print(f"{'Phoneme':<15} {'Time':<20} {'Tajweed Rule'}")
    print("-" * 70)

    tajweed_count = 0
    for p in phonemes:
        rule = p.get('tajweed_rule')
        if rule:
            time_str = f"{p['start']:.3f}-{p['end']:.3f}s"
            print(f"{p['phoneme']:<15} {time_str:<20} {rule}")
            tajweed_count += 1

    print(f"\n✓ Found {tajweed_count} phonemes with Tajweed rules (out of {len(phonemes)} total)")

    if tajweed_count == 0:
        print("\n⚠️  WARNING: No Tajweed rules detected! This suggests the mapping is not working.")
    else:
        print(f"\n✓ SUCCESS: Tajweed rules are being mapped to phonemes!")

    print("\n" + "="*70 + "\n")

if __name__ == "__main__":
    test_phoneme_tajweed()
