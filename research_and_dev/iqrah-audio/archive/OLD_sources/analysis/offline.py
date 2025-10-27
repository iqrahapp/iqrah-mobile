"""
Offline Recitation Analysis
===========================

Main orchestrator for offline analysis.
No time pressure - perfect accuracy!
"""

from typing import Dict
from pathlib import Path

from .pitch_extractor import extract_pitch_from_file, extract_pitch_from_url, calculate_pitch_stats
from .alignment import align_sequences_dtw, map_user_to_words, calculate_tempo_ratio
from .metrics import (
    calculate_pitch_accuracy_per_word,
    calculate_stability,
    calculate_complexity,
    calculate_overall_score
)
from .phoneme_alignment_fixed import analyze_word_phonemes
from .tajweed_parser import load_tajweed_data, get_word_tajweed


# Export for testing
def calculate_dtw_alignment(user_pitch, ref_pitch):
    """
    Calculate DTW alignment between user and reference pitch sequences.

    Convenience wrapper for testing.

    Args:
        user_pitch: List of user pitch values (Hz)
        ref_pitch: List of reference pitch values (Hz)

    Returns:
        Dictionary with 'distance', 'user_to_ref', 'ref_to_user'
    """
    return align_sequences_dtw(user_pitch, ref_pitch)


def analyze_recitation(
    user_audio_path: str,
    ref_audio_url: str,
    ref_segments: list,
    ref_words: list
) -> Dict:
    """
    Complete offline analysis of user's recitation.

    Args:
        user_audio_path: Path to user's recording
        ref_audio_url: URL to reference audio (Qari)
        ref_segments: List of word segments with start_ms, end_ms
        ref_words: List of Arabic words

    Returns:
        Complete analysis dictionary
    """

    print("=" * 60)
    print("OFFLINE RECITATION ANALYSIS")
    print("=" * 60)

    # Step 1: Extract pitches
    print("\n1️⃣  Extracting user pitch...")
    user_pitch_data = extract_pitch_from_file(user_audio_path)
    user_stats = calculate_pitch_stats(user_pitch_data['f0_hz'])
    print(f"   ✓ User: {user_pitch_data['duration']:.2f}s, "
          f"{user_stats['mean_hz']:.1f} Hz mean")

    print("\n2️⃣  Extracting reference pitch...")
    ref_pitch_data = extract_pitch_from_url(ref_audio_url)
    ref_stats = calculate_pitch_stats(ref_pitch_data['f0_hz'])
    print(f"   ✓ Reference: {ref_pitch_data['duration']:.2f}s, "
          f"{ref_stats['mean_hz']:.1f} Hz mean")

    # Step 2: Align sequences
    print("\n3️⃣  Aligning sequences with DTW...")
    alignment = align_sequences_dtw(
        user_pitch_data['f0_hz'],
        ref_pitch_data['f0_hz'],
        user_pitch_data['voiced'],
        ref_pitch_data['voiced']
    )
    print(f"   ✓ Alignment complete (distance: {alignment['distance']:.2f})")

    # Step 3: Map to words
    print("\n4️⃣  Mapping to words...")
    word_alignment = map_user_to_words(
        alignment,
        ref_segments,
        user_pitch_data['duration'],
        ref_pitch_data['duration']
    )
    print(f"   ✓ Mapped {len(word_alignment)} frames to {len(ref_words)} words")

    # Step 4: Calculate metrics
    print("\n5️⃣  Calculating metrics...")

    # 4.1: Word-level accuracy
    word_scores = calculate_pitch_accuracy_per_word(
        user_pitch_data['f0_hz'],
        ref_pitch_data['f0_hz'],
        word_alignment,
        len(ref_words),
        alignment['user_to_ref']  # Pass DTW mapping
    )
    print(f"   ✓ Word accuracy calculated")

    # 4.2: Tempo
    tempo = calculate_tempo_ratio(
        alignment,
        user_pitch_data['duration'],
        ref_pitch_data['duration']
    )
    print(f"   ✓ Tempo ratio: {tempo['mean_ratio']:.2f}x")

    # 4.3: Stability
    stability = calculate_stability(user_pitch_data['f0_hz'])
    print(f"   ✓ Voice stability: {stability['status']}")

    # 4.4: Complexity
    complexity = calculate_complexity(user_pitch_data['f0_hz'])
    print(f"   ✓ Melody complexity: {complexity['status']}")

    # 4.5: Overall score
    overall_score = calculate_overall_score(word_scores, tempo, stability, complexity)
    print(f"   ✓ Overall score: {overall_score}/100")

    # Step 5: Phoneme segmentation with Tajweed rules
    print("\n6️⃣  Segmenting phonemes with Tajweed...")
    user_phoneme_segments = []
    ref_phoneme_segments = []

    try:
        # Load Tajweed data
        tajweed_data = load_tajweed_data()
        print(f"   ✓ Loaded {len(tajweed_data)} Tajweed-annotated words")

        # Process each word segment
        for i, seg in enumerate(ref_segments):
            # Get Tajweed-annotated word
            surah = seg.get('surah', 1)
            ayah = seg.get('ayah', 1)
            word_num = seg.get('word', i + 1)

            taj_data = get_word_tajweed(surah, ayah, word_num, tajweed_data)

            if taj_data:
                word_text = taj_data['clean_text']
                tajweed_rules = taj_data.get('rules', [])

                # Analyze phonemes for USER audio
                user_word_phonemes = analyze_word_phonemes(
                    audio_path=user_audio_path,
                    word_text=word_text,
                    word_start_ms=seg['start_ms'],
                    word_end_ms=seg['end_ms'],
                    pitch_data=user_pitch_data,
                    tajweed_rules=tajweed_rules
                )
                user_phoneme_segments.extend(user_word_phonemes)

                # CRITICAL: Also analyze phonemes for REFERENCE (Qari) audio!
                # Download reference audio to temp file for phoneme analysis
                import tempfile
                import urllib.request
                from pathlib import Path

                temp_ref = Path(tempfile.gettempdir()) / f"ref_audio_{hash(ref_audio_url)}.mp3"
                if not temp_ref.exists():
                    urllib.request.urlretrieve(ref_audio_url, temp_ref)

                ref_word_phonemes = analyze_word_phonemes(
                    audio_path=str(temp_ref),
                    word_text=word_text,
                    word_start_ms=seg['start_ms'],
                    word_end_ms=seg['end_ms'],
                    pitch_data=ref_pitch_data,
                    tajweed_rules=tajweed_rules
                )
                ref_phoneme_segments.extend(ref_word_phonemes)

            else:
                # Fallback: use raw text from ref_words
                if i < len(ref_words):
                    word_text = ref_words[i]

                    # User phonemes
                    user_word_phonemes = analyze_word_phonemes(
                        audio_path=user_audio_path,
                        word_text=word_text,
                        word_start_ms=seg['start_ms'],
                        word_end_ms=seg['end_ms'],
                        pitch_data=user_pitch_data
                    )
                    user_phoneme_segments.extend(user_word_phonemes)

                    # Reference phonemes
                    import tempfile
                    import urllib.request
                    from pathlib import Path

                    temp_ref = Path(tempfile.gettempdir()) / f"ref_audio_{hash(ref_audio_url)}.mp3"
                    if not temp_ref.exists():
                        urllib.request.urlretrieve(ref_audio_url, temp_ref)

                    ref_word_phonemes = analyze_word_phonemes(
                        audio_path=str(temp_ref),
                        word_text=word_text,
                        word_start_ms=seg['start_ms'],
                        word_end_ms=seg['end_ms'],
                        pitch_data=ref_pitch_data
                    )
                    ref_phoneme_segments.extend(ref_word_phonemes)

        print(f"   ✓ User phonemes: {len(user_phoneme_segments)}")
        print(f"   ✓ Reference phonemes: {len(ref_phoneme_segments)}")
    except Exception as e:
        print(f"   ⚠ Phoneme segmentation failed: {e}")
        import traceback
        traceback.print_exc()
        user_phoneme_segments = []
        ref_phoneme_segments = []

    print("\n" + "=" * 60)
    print(f"✅ ANALYSIS COMPLETE - Score: {overall_score}/100")
    if user_phoneme_segments or ref_phoneme_segments:
        print(f"   📝 User: {len(user_phoneme_segments)} phonemes, Ref: {len(ref_phoneme_segments)} phonemes")
    print("=" * 60)

    # Return comprehensive results
    return {
        'user_pitch': user_pitch_data,
        'ref_pitch': ref_pitch_data,
        'user_stats': user_stats,
        'ref_stats': ref_stats,
        'alignment': {
            'path': alignment['path'][:1000],  # Limit size for JSON
            'distance': alignment['distance'],
            'user_to_ref': {str(k): v for k, v in list(alignment['user_to_ref'].items())[:1000]},
        },
        'word_alignment': word_alignment,
        'word_scores': word_scores,
        'user_phoneme_segments': user_phoneme_segments,  # NEW: User phoneme-level data
        'ref_phoneme_segments': ref_phoneme_segments,    # NEW: Reference phoneme-level data
        'metrics': {
            'overall_score': overall_score,
            'pitch_accuracy': {
                'per_word': word_scores,
                'mean_error': float(sum(w['error_cents'] for w in word_scores if w['status'] != 'missing') / max(1, len([w for w in word_scores if w['status'] != 'missing']))),
                'good_words': len([w for w in word_scores if w['status'] == 'good']),
                'warning_words': len([w for w in word_scores if w['status'] == 'warning']),
                'error_words': len([w for w in word_scores if w['status'] == 'error']),
            },
            'tempo': tempo,
            'stability': stability,
            'complexity': complexity,
            'pitch_range': {
                'user_semitones_vs_ref': float(12 * (user_stats['mean_hz'] / ref_stats['mean_hz'] - 1)) if ref_stats['mean_hz'] > 0 else 0,
                'user_range_hz': user_stats['range_hz'],
                'ref_range_hz': ref_stats['range_hz'],
            }
        }
    }


if __name__ == "__main__":
    # Test full analysis
    from src.iqrah_audio.core.segments_loader import SegmentsLoader

    print("Testing full offline analysis...")

    loader = SegmentsLoader()
    ayah = loader.get_ayah(1, 1)  # Al-Fatihah 1:1

    # For testing, we'll use the reference audio as "user" audio
    # In reality, this would be user's recording
    import urllib.request
    import tempfile

    temp_user = Path(tempfile.gettempdir()) / "test_user.mp3"
    urllib.request.urlretrieve(ayah.audio_url, temp_user)

    # Analyze
    result = analyze_recitation(
        user_audio_path=str(temp_user),
        ref_audio_url=ayah.audio_url,
        ref_segments=[
            {'start_ms': seg.start_ms, 'end_ms': seg.end_ms}
            for seg in ayah.segments
        ],
        ref_words=ayah.words
    )

    print(f"\n📊 Results:")
    print(f"  Overall Score: {result['metrics']['overall_score']}/100")
    print(f"  Good Words: {result['metrics']['pitch_accuracy']['good_words']}/{len(ayah.words)}")
    print(f"  Tempo: {result['metrics']['tempo']['mean_ratio']:.2f}x")
