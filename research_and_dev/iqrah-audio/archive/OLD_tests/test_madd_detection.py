"""Test Madd detection with Tajweed mapper."""

import sys
from pathlib import Path

# Add src to path
sys.path.insert(0, str(Path(__file__).parent))

from src.iqrah_audio.analysis.tajweed_mapper import TajweedMapper
from src.iqrah_audio.analysis.segments_loader import get_word_segments_with_text

def test_madd_detection():
    """Test Madd detection on Surah Al-Fatiha."""
    print("\n" + "="*70)
    print("Testing Madd Detection for Surah Al-Fatiha (001:01)")
    print("="*70)

    # Initialize Tajweed mapper
    mapper = TajweedMapper()

    # Test on first ayah
    surah, ayah = 1, 1

    # Get word segments
    word_segments = get_word_segments_with_text(surah, ayah)
    print(f"\n✓ Loaded {len(word_segments)} word segments")

    # Check each word for Tajweed rules
    print(f"\n{'Word':<4} {'Text':<20} {'Tajweed Rules'}")
    print("-" * 70)

    for i, word_seg in enumerate(word_segments):
        word_idx = i + 1  # 1-indexed
        tajweed_segments = mapper.get_word_tajweed_rules(surah, ayah, word_idx)

        # Extract text
        text = ''.join(seg['text'] for seg in tajweed_segments)

        # Find Madd rules
        madd_rules = [seg for seg in tajweed_segments if seg['rule'] and 'madda' in seg['rule']]

        if madd_rules:
            rules_str = ', '.join([f"{r['text']}({r['rule']})" for r in madd_rules])
            print(f"{word_idx:<4} {text:<20} {rules_str}")
        else:
            print(f"{word_idx:<4} {text:<20} (no Madd)")

    # Test phoneme-to-tajweed mapping
    print("\n" + "="*70)
    print("Testing Phoneme → Tajweed Rule Mapping")
    print("="*70)

    # Simulate a phoneme in the middle of word 3 (الرَّحْمَـٰنِ - has Madd)
    if len(word_segments) >= 3:
        word_idx = 2  # 0-indexed (word 3 is index 2)
        word_seg = word_segments[word_idx]

        # Simulate phoneme in the middle
        word_start_ms = word_seg['start_ms']
        word_end_ms = word_seg['end_ms']
        word_mid_ms = (word_start_ms + word_end_ms) / 2

        phoneme_start = word_mid_ms / 1000
        phoneme_end = (word_mid_ms + 100) / 1000

        tajweed_rule = mapper.map_phoneme_to_tajweed(
            phoneme_start=phoneme_start,
            phoneme_end=phoneme_end,
            word_idx=word_idx,
            word_segments=word_segments,
            surah=surah,
            ayah=ayah
        )

        print(f"\nWord 3 (الرَّحْمَـٰنِ):")
        print(f"  Phoneme time: {phoneme_start:.3f}s - {phoneme_end:.3f}s")
        print(f"  Detected Tajweed rule: {tajweed_rule}")

        # Show all segments for this word
        tajweed_segments = mapper.get_word_tajweed_rules(surah, ayah, word_idx + 1)
        print(f"  Word segments:")
        for seg in tajweed_segments:
            print(f"    '{seg['text']}' → {seg['rule']} ({seg['category']})")

    print("\n" + "="*70)
    print("✓ Test complete!")
    print("="*70 + "\n")

if __name__ == "__main__":
    test_madd_detection()
