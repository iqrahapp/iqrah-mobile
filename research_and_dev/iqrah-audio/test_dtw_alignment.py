#!/usr/bin/env python3
"""
Test DTW Alignment with Real Audio Files
=========================================

Debug script to test DTW alignment directly with:
- Student: data/me/surahs/001/01.mp3
- Reference: data/husary/surahs/001/01.mp3

This simulates the full API flow to debug DTW alignment and scoring.
"""

import sys
sys.path.insert(0, 'src')

import asyncio
from pathlib import Path
import json

# Import from the actual app
sys.path.insert(0, '.')
from app_qari_final import analyze_qari_from_file

async def main():
    print("=" * 80)
    print("DTW Alignment Test - Direct Audio Comparison")
    print("=" * 80)

    student_path = "data/me/surahs/001/01.mp3"
    reference_path = "data/husary/surahs/001/01.mp3"

    if not Path(student_path).exists():
        print(f"ERROR: Student audio not found: {student_path}")
        return

    if not Path(reference_path).exists():
        print(f"ERROR: Reference audio not found: {reference_path}")
        return

    print(f"\nStudent: {student_path}")
    print(f"Reference: {reference_path}")
    print()

    # Analyze both using the same pipeline as the API
    print("1️⃣ Analyzing student recitation...")
    student_result = await analyze_qari_from_file(student_path, surah=1, ayah=1)

    print("\n2️⃣ Analyzing reference (Husary)...")
    reference_result = await analyze_qari_from_file(reference_path, surah=1, ayah=1)

    # Now run comparison
    print("\n3️⃣ Running comparison engine...")
    from iqrah_audio.comparison.engine import compare_recitations

    result = compare_recitations(
        student_audio_path=student_path,
        reference_audio_path=reference_path,
        student_phonemes=student_result['phonemes'],
        reference_phonemes=reference_result['phonemes'],
        student_pitch=student_result['pitch'],
        reference_pitch=reference_result['pitch'],
        student_stats=student_result['statistics'],
        reference_stats=reference_result['statistics']
    )

    print("\n" + "=" * 80)
    print("COMPARISON RESULTS")
    print("=" * 80)

    print(f"\n🎯 Overall Score: {result['overall']:.1f}/100 (confidence: {result['confidence']})")
    print(f"\n🎵 Rhythm Score: {result['rhythm']['score']:.1f}/100")
    print(f"   Divergence: {result['rhythm']['divergence']:.3f}")
    print(f"   DTW path length: {len(result['rhythm']['path'])}")

    print(f"\n🎼 Melody Score: {result['melody']['score']:.1f}/100")
    print(f"   Pitch shift: {result['melody']['pitch_shift_cents']:+.1f} cents")
    print(f"   Contour similarity: {result['melody']['contour_similarity']:.3f}")

    print(f"\n⏱️  Duration Score: {result['durations']['overall']:.1f}/100")

    # Debug: Check if frame_times are being returned
    print(f"\n🔍 Debug Info:")
    print(f"   Student frame_times: {len(result['rhythm'].get('student_frame_times', []))} frames")
    print(f"   Reference frame_times: {len(result['rhythm'].get('reference_frame_times', []))} frames")

    # Check tempo ratio
    print(f"\n   Tempo ratio: {result['metadata']['tempo_ratio']:.3f}")
    print(f"   Student duration: {result['metadata']['student_duration']:.2f}s")
    print(f"   Reference duration: {result['metadata']['reference_duration']:.2f}s")

    # Print feedback
    print(f"\n💬 Feedback:")
    for note in result['feedback']['all_notes'][:5]:  # First 5 notes
        if isinstance(note, dict):
            print(f"   [{note.get('category', 'Note')}] {note.get('text', note)}")
        else:
            print(f"   {note}")

    # Save detailed results to JSON for inspection
    output_file = "dtw_alignment_test_results.json"
    with open(output_file, 'w', encoding='utf-8') as f:
        json.dump(result, f, indent=2, ensure_ascii=False)
    print(f"\n📄 Full results saved to: {output_file}")

    print("\n" + "=" * 80)

if __name__ == "__main__":
    asyncio.run(main())
