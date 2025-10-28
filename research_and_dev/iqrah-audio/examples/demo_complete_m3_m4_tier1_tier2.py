"""
Demo: Complete M3+M4 Pipeline (Tier 1 + Tier 2 Integrated)

This is the ULTIMATE demo showing the full Iqrah Audio pipeline:

1. M3: Phoneme Recognition & Alignment (PER gatekeeper + CTC)
2. M4 Tier 1: Baseline Tajweed (10+ rules from Muaalem sifat)
3. M4 Tier 2: Specialized Validators (Madd duration modeling)
4. Orchestrator: Integrated scoring and violation management

Usage:
    python examples/demo_complete_m3_m4_tier1_tier2.py
"""

import sys
from pathlib import Path
import numpy as np
from quran_transcript import Aya
import json

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

from iqrah.pipeline import M3Pipeline
from iqrah.tajweed import TajweedOrchestrator


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


def print_header(title: str, char: str = "="):
    """Print formatted header."""
    print("\n" + char * 80)
    print(title)
    print(char * 80)


def analyze_complete_pipeline(audio_path: Path, reference_text: str, label: str):
    """Run complete M3+M4 Tier1+Tier2 pipeline."""

    print_header(f"ANALYZING: {audio_path.name} ({label})")

    # Load audio
    print("\n[Step 1] Loading audio...")
    audio = load_audio_file(audio_path)
    print(f"  ✓ Duration: {len(audio)/16000:.2f}s")

    # Run M3 Pipeline
    print("\n[Step 2] Running M3 Pipeline...")
    m3_pipeline = M3Pipeline(device="cpu")

    try:
        m3_result = m3_pipeline.process(
            audio=audio,
            reference_text=reference_text,
            sample_rate=16000
        )
        print(f"  ✓ M3 Complete: {len(m3_result.phonemes)} phonemes aligned")
        print(f"  ✓ Content Gate: PASSED (PER = {m3_result.gate_result.per:.2%})")
    except RuntimeError as e:
        print(f"  ⚠ Content Gate: FAILED ({e})")
        print(f"  ⚠ Continuing with skip_gate=True...")
        m3_result = m3_pipeline.process(
            audio=audio,
            reference_text=reference_text,
            sample_rate=16000,
            skip_gate=True
        )
        print(f"  ✓ M3 Complete: {len(m3_result.phonemes)} phonemes aligned")

    # Run M4 with Orchestrator (Tier 1 + Tier 2)
    print("\n[Step 3] Running M4 Tajweed Orchestrator (Tier 1 + Tier 2)...")

    orchestrator = TajweedOrchestrator(
        enable_baseline=True,      # Tier 1: 10+ rules from sifat
        enable_madd=True,          # Tier 2: Madd duration modeling
        enable_ghunnah_formants=False,  # Tier 2 Phase 2 (not yet implemented)
        enable_qalqalah_bursts=False,   # Tier 2 Phase 2 (not yet implemented)
        confidence_threshold=0.7
    )

    print(f"  Enabled Modules: {', '.join(orchestrator.get_enabled_modules())}")

    tajweed_result = orchestrator.validate(
        aligned_phonemes=m3_result.phonemes,
        audio=audio
    )

    print(f"  ✓ Orchestrator Complete")

    # Display Results
    print_header("M3 CONTENT VERIFICATION")
    print(f"  PER (Phoneme Error Rate): {m3_result.gate_result.per:.2%}")
    print(f"  Gate Status: {'✅ PASSED' if m3_result.gate_result.passed else '❌ FAILED'}")
    print(f"  Confidence: {m3_result.gate_result.confidence:.1%}")
    print(f"  Phoneme Errors: {len(m3_result.gate_result.errors)}")

    if m3_result.gate_result.errors:
        print(f"\n  Error Details:")
        for i, error in enumerate(m3_result.gate_result.errors[:5], 1):
            print(f"    {i}. {error['type'].upper()} at position {error['position']}")
            if error['type'] == 'substitution':
                print(f"       Expected: '{error['reference_phoneme']}', Got: '{error['predicted_phoneme']}'")

    print_header("M4 TAJWEED VALIDATION (TIER 1 + TIER 2)")

    print(f"\n  Overall Tajweed Score: {tajweed_result.overall_score:.1f}%")
    print(f"  Total Violations: {len(tajweed_result.violations)}")
    print(f"  Total Phonemes: {tajweed_result.total_phonemes}")

    print(f"\n  Tier Breakdown:")
    print(f"    Tier 1 Coverage:      {tajweed_result.tier1_coverage:.1f}%")
    print(f"    Tier 2 Enhancements:  {tajweed_result.tier2_enhancements} violations")

    print(f"\n  Per-Rule Scores:")
    for rule_name, score in sorted(tajweed_result.scores_by_rule.items()):
        if score is not None:
            status = "✓" if score >= 90 else "⚠" if score >= 70 else "✗"
            print(f"    {status} {rule_name:20s} {score:6.1f}%")

    # Violations Detail
    if tajweed_result.violations:
        print(f"\n  Top Violations (by severity):")

        # Group by tier and severity
        tier1_violations = [v for v in tajweed_result.violations if v.get('tier', 1) == 1]
        tier2_violations = [v for v in tajweed_result.violations if v.get('tier', 1) == 2]

        if tier2_violations:
            print(f"\n    Tier 2 Violations ({len(tier2_violations)}):")
            for i, v in enumerate(tier2_violations[:3], 1):
                print(f"\n      {i}. [{v['severity'].upper()}] {v['rule']}")
                print(f"         Phoneme: '{v['phoneme']}' at {v['timestamp']:.2f}s")
                if 'expected_duration' in v:
                    print(f"         Expected: {v['expected_duration']:.0f}ms, Actual: {v['actual_duration']:.0f}ms")
                    print(f"         Z-Score: {v.get('z_score', 0):.2f}σ")
                print(f"         {v.get('feedback', '')}")

        if tier1_violations:
            print(f"\n    Tier 1 Violations ({len(tier1_violations)}):")
            for i, v in enumerate(tier1_violations[:3], 1):
                print(f"\n      {i}. [{v['severity'].upper()}] {v['rule']}")
                print(f"         Phoneme: '{v['phoneme']}' at {v['timestamp']:.2f}s")
                print(f"         Confidence: {v.get('confidence', 0):.1%}")
                print(f"         {v.get('feedback', '')}")

        if len(tajweed_result.violations) > 6:
            print(f"\n    ... and {len(tajweed_result.violations) - 6} more violations")

    else:
        print(f"\n  ✅ PERFECT: No Tajweed violations detected!")

    # Summary Card
    print_header("SUMMARY")
    gate_status = "✅ PASSED" if m3_result.gate_result.passed else "❌ FAILED"
    tajweed_status = "✅ EXCELLENT" if tajweed_result.overall_score >= 90 else \
                    "⚠ GOOD" if tajweed_result.overall_score >= 70 else \
                    "✗ NEEDS IMPROVEMENT"

    print(f"  Audio: {label}")
    print(f"  Duration: {len(audio)/16000:.2f}s")
    print(f"  Phonemes: {tajweed_result.total_phonemes}")
    print(f"  ")
    print(f"  M3 Content:    {m3_result.gate_result.per:6.2%} {gate_status}")
    print(f"  M4 Tajweed:    {tajweed_result.overall_score:6.1f}% {tajweed_status}")
    print(f"  ")
    print(f"  Enabled Modules: {', '.join(tajweed_result.enabled_modules)}")
    print(f"  Total Violations: {len(tajweed_result.violations)}")
    print(f"    • Tier 1: {len([v for v in tajweed_result.violations if v.get('tier', 1) == 1])}")
    print(f"    • Tier 2: {tajweed_result.tier2_enhancements}")

    return {
        "label": label,
        "per": m3_result.gate_result.per,
        "gate_passed": m3_result.gate_result.passed,
        "overall_score": tajweed_result.overall_score,
        "total_violations": len(tajweed_result.violations),
        "tier1_violations": len([v for v in tajweed_result.violations if v.get('tier', 1) == 1]),
        "tier2_violations": tajweed_result.tier2_enhancements,
        "scores_by_rule": tajweed_result.scores_by_rule
    }


def main():
    """Run complete M3+M4 Tier 1+Tier 2 pipeline demo."""
    print("\n" + "#" * 80)
    print("# COMPLETE M3+M4 PIPELINE DEMO")
    print("# Tier 1 (Baseline) + Tier 2 (Specialized) Integrated")
    print("#" * 80)

    print("\n📋 Pipeline Components:")
    print("  1. M3: Phoneme Recognition & Alignment")
    print("     • Phonetizer (quran_transcript)")
    print("     • Muaalem ASR (phonemes + sifat)")
    print("     • PER Gatekeeper (content verification)")
    print("     • CTC Aligner (timestamps)")
    print("")
    print("  2. M4 Tier 1: Baseline Sifat Interpreter")
    print("     • Ghunnah, Qalqalah, Tafkhim, Itbaq, etc.")
    print("     • 10+ rules from Muaalem sifat")
    print("     • 70-85% accuracy per rule")
    print("")
    print("  3. M4 Tier 2: Specialized Validators")
    print("     ✓ Madd Duration Modeling (95%+ target)")
    print("     ⏳ Ghunnah Formants (Phase 2)")
    print("     ⏳ Qalqalah Bursts (Phase 2)")

    # Get reference text
    aya = Aya(1, 1)
    reference_text = aya.get().uthmani
    print(f"\n📖 Reference Text (Al-Fatihah 1:1):")
    print(f"  {reference_text}")

    # Test cases
    audio_dir = Path(__file__).parent.parent / "data/me/surahs/001"
    test_cases = [
        (audio_dir / "01.mp3", "CORRECT RECITATION"),
        (audio_dir / "01-mistake.mp3", "WITH INTENTIONAL MISTAKES"),
    ]

    results = []

    try:
        for audio_path, label in test_cases:
            if not audio_path.exists():
                print(f"\n⚠ Skipping {label}: File not found")
                continue

            result = analyze_complete_pipeline(
                audio_path=audio_path,
                reference_text=reference_text,
                label=label
            )
            results.append(result)

        # Comparison
        if len(results) >= 2:
            print_header("COMPARISON REPORT", char="#")

            print(f"\n{'Metric':<30} {'Correct':<20} {'With Mistakes':<20} {'Delta':<15}")
            print("=" * 85)

            correct = results[0]
            mistake = results[1]

            print(f"{'M3 PER (%)':<30} {correct['per']*100:<20.2f} {mistake['per']*100:<20.2f} {(mistake['per']-correct['per'])*100:+.2f}")
            print(f"{'M3 Gate':<30} {str(correct['gate_passed']):<20} {str(mistake['gate_passed']):<20} {'—':<15}")
            print(f"{'M4 Overall Score (%)':<30} {correct['overall_score']:<20.1f} {mistake['overall_score']:<20.1f} {mistake['overall_score']-correct['overall_score']:+.1f}")
            print(f"{'Total Violations':<30} {correct['total_violations']:<20} {mistake['total_violations']:<20} {mistake['total_violations']-correct['total_violations']:+d}")
            print(f"{'Tier 1 Violations':<30} {correct['tier1_violations']:<20} {mistake['tier1_violations']:<20} {mistake['tier1_violations']-correct['tier1_violations']:+d}")
            print(f"{'Tier 2 Violations':<30} {correct['tier2_violations']:<20} {mistake['tier2_violations']:<20} {mistake['tier2_violations']-correct['tier2_violations']:+d}")

        # Final Summary
        print_header("DEMO COMPLETE", char="#")

        print("\n✅ Successfully Demonstrated:")
        print("  • Complete M3+M4 Pipeline Integration")
        print("  • Tier 1 Baseline (10+ rules from sifat)")
        print("  • Tier 2 Madd Validator (duration modeling)")
        print("  • Orchestrator (unified scoring & reporting)")
        print("  • Real-world validation (correct + mistake audio)")

        print("\n📊 Architecture Highlights:")
        print("  • Modular: Enable/disable validators independently")
        print("  • Baseline-first: Tier 1 always runs, Tier 2 enhances")
        print("  • Graceful degradation: Tier 2 failures don't affect Tier 1")
        print("  • Comprehensive: Content (M3) + Tajweed (M4)")

        print("\n🎯 Status:")
        print("  • M3 Pipeline: ✅ Complete")
        print("  • M4 Tier 1: ✅ Complete (10+ rules)")
        print("  • M4 Tier 2 Madd: ✅ Complete (95%+ target)")
        print("  • M4 Tier 2 Ghunnah: ⏳ Phase 2")
        print("  • M4 Tier 2 Qalqalah: ⏳ Phase 2")
        print("  • Orchestrator: ✅ Complete")

        print("\n🚀 Next Steps:")
        print("  • M4 Tier 2 Ghunnah: Formant analysis for nasal sounds")
        print("  • M4 Tier 2 Qalqalah: Burst detection for echoing sounds")
        print("  • Full surah testing: Multi-ayah validation")
        print("  • Expert validation: Compare with human raters")
        print("  • Performance optimization: GPU acceleration, batching")

    except KeyboardInterrupt:
        print("\n\n⚠ Demo interrupted by user.")
    except Exception as e:
        print(f"\n\n❌ FATAL ERROR: {e}")
        import traceback
        traceback.print_exc()


if __name__ == "__main__":
    main()
