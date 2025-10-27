"""
Demo: M4 Tier 1 Baseline Tajweed Validation

This script demonstrates Tier 1 Tajweed validation using sifat from Muaalem.
Tests with both correct recitation and intentional mistakes.

Usage:
    python examples/demo_m4_tier1.py
"""

import sys
from pathlib import Path
import numpy as np
from quran_transcript import Aya

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

from iqrah.pipeline import M3Pipeline
from iqrah.tajweed import BaselineTajweedInterpreter


def load_audio_file(path: Path, sample_rate: int = 16000) -> np.ndarray:
    """Load MP3 audio and resample."""
    if not Path(path).exists():
        raise FileNotFoundError(f"Audio file not found: {path}")
    try:
        import librosa
    except ImportError as e:
        raise RuntimeError("librosa required: pip install librosa") from e

    audio, _ = librosa.load(str(path), sr=sample_rate, mono=True)
    return audio.astype(np.float32)


def demo_tier1_validation():
    """Demo: Tier 1 baseline Tajweed validation."""
    print("=" * 80)
    print("M4 TIER 1: BASELINE TAJWEED VALIDATION DEMO")
    print("=" * 80)

    # Test with correct recitation first
    audio_correct = Path(__file__).parent.parent / "data/me/surahs/001/01.mp3"
    audio_mistake = Path(__file__).parent.parent / "data/me/surahs/001/01-mistake.mp3"

    # Choose which audio to test
    test_audio = audio_mistake if audio_mistake.exists() else audio_correct
    audio_label = "WITH MISTAKES" if test_audio == audio_mistake else "CORRECT"

    print(f"\nTesting with: {test_audio.name} ({audio_label})")
    print("-" * 80)

    # Get reference text
    aya = Aya(1, 1)
    reference_text = aya.get().uthmani
    print(f"Reference: {reference_text}")

    # Load audio
    print(f"\nLoading audio...")
    audio = load_audio_file(test_audio)
    print(f"Audio duration: {len(audio)/16000:.2f}s")

    # Run M3 pipeline
    print("\n[Step 1] Running M3 Pipeline...")
    m3_pipeline = M3Pipeline(device="cpu")

    try:
        m3_result = m3_pipeline.process(
            audio=audio,
            reference_text=reference_text,
            sample_rate=16000
        )
    except RuntimeError as e:
        print(f"  Gate failed (expected for mistake audio): {e}")
        print("  Continuing with skip_gate=True...")
        m3_result = m3_pipeline.process(
            audio=audio,
            reference_text=reference_text,
            sample_rate=16000,
            skip_gate=True
        )

    print(f"  ✓ M3 Complete: {len(m3_result.phonemes)} phonemes with sifat")
    print(f"  PER: {m3_result.gate_result.per:.2%}")

    # Run M4 Tier 1 validation
    print("\n[Step 2] Running M4 Tier 1 Baseline Validation...")
    interpreter = BaselineTajweedInterpreter(
        confidence_threshold=0.7,
        enable_all_rules=True
    )

    violations = interpreter.validate(aligned_phonemes=m3_result.phonemes)

    print(f"  ✓ M4 Tier 1 Complete")

    # Display results
    print("\n" + "=" * 80)
    print("TAJWEED VALIDATION RESULTS")
    print("=" * 80)

    total_violations = sum(len(v) for v in violations.values())
    print(f"\nTotal Violations: {total_violations}")

    if total_violations == 0:
        print("  ✓ No Tajweed violations detected!")
        print("  All 10+ rules passed baseline validation.")
    else:
        print(f"\nViolations by Rule:")
        for rule_name, rule_violations in sorted(violations.items()):
            if rule_violations:
                print(f"\n  [{rule_name}]: {len(rule_violations)} violation(s)")
                for i, v in enumerate(rule_violations[:5]):  # Show first 5
                    print(f"    {i+1}. Phoneme '{v.phoneme}' @ {v.timestamp:.2f}s")
                    print(f"       Expected: {v.expected}, Actual: {v.actual}")
                    print(f"       Confidence: {v.confidence:.0%}, Severity: {v.severity}")
                    print(f"       {v.feedback}")

                if len(rule_violations) > 5:
                    print(f"    ... and {len(rule_violations) - 5} more")

    # Compute scores
    print("\n" + "=" * 80)
    print("TAJWEED SCORES (Per Rule)")
    print("=" * 80)

    scores = interpreter.compute_scores(violations, len(m3_result.phonemes))

    for rule_name, score in sorted(scores.items()):
        if rule_name != "overall":
            status = "✓" if score >= 90 else "⚠" if score >= 70 else "✗"
            print(f"  {status} {rule_name:20s} {score:6.1f}%")

    print("-" * 80)
    print(f"  {'OVERALL':20s} {scores['overall']:6.1f}%")

    # Summary
    print("\n" + "=" * 80)
    print("SUMMARY")
    print("=" * 80)
    print(f"Audio: {audio_label}")
    print(f"M3 PER: {m3_result.gate_result.per:.2%}")
    print(f"M4 Tier 1 Overall Score: {scores['overall']:.1f}%")
    print(f"Total Violations: {total_violations}")
    print(f"Rules Validated: {len([r for r in scores if r != 'overall'])}")


def demo_confidence_levels():
    """Demo: Show how confidence threshold affects violations."""
    print("\n\n" + "=" * 80)
    print("CONFIDENCE THRESHOLD COMPARISON")
    print("=" * 80)

    # Load audio
    audio_path = Path(__file__).parent.parent / "data/me/surahs/001/01.mp3"
    audio = load_audio_file(audio_path)

    # Get reference
    aya = Aya(1, 1)
    reference_text = aya.get().uthmani

    # Run M3
    print("\nRunning M3 pipeline...")
    m3_pipeline = M3Pipeline(device="cpu")
    m3_result = m3_pipeline.process(
        audio=audio,
        reference_text=reference_text,
        sample_rate=16000,
        skip_gate=True
    )

    # Test different thresholds
    thresholds = [0.5, 0.7, 0.9]

    print(f"\n{'Threshold':<12} {'Total Violations':<20} {'Overall Score':<15}")
    print("-" * 50)

    for thresh in thresholds:
        interpreter = BaselineTajweedInterpreter(confidence_threshold=thresh)
        violations = interpreter.validate(m3_result.phonemes)
        scores = interpreter.compute_scores(violations, len(m3_result.phonemes))

        total_viols = sum(len(v) for v in violations.values())
        print(f"{thresh:<12.1f} {total_viols:<20} {scores['overall']:<15.1f}%")

    print("\nNote: Higher threshold = more strict = more violations detected")


def main():
    """Run all demos."""
    print("\n" + "#" * 80)
    print("# M4 TIER 1: BASELINE TAJWEED VALIDATION")
    print("# Using Muaalem Sifat for 10+ Rules")
    print("#" * 80)

    try:
        # Demo 1: Baseline validation
        demo_tier1_validation()

        # Demo 2: Confidence comparison
        demo_confidence_levels()

        print("\n" + "=" * 80)
        print("ALL DEMOS COMPLETED SUCCESSFULLY!")
        print("=" * 80)

    except KeyboardInterrupt:
        print("\n\nDemo interrupted by user.")
    except Exception as e:
        print(f"\n\nFATAL ERROR: {e}")
        import traceback
        traceback.print_exc()


if __name__ == "__main__":
    main()
