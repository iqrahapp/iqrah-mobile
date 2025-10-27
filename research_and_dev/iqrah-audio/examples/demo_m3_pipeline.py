"""
Demo: M3 Pipeline with Real Muaalem Model

This script demonstrates the complete M3 pipeline using the Muaalem model.
The model will be automatically downloaded from HuggingFace on first run.

Usage:
    python examples/demo_m3_pipeline.py
"""

import sys
import json
from pathlib import Path
import numpy as np
from quran_transcript import Aya

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

from iqrah.pipeline import M3Pipeline

# Use real audio instead of synthetic
# AUDIO_PATH = Path(__file__).parent.parent / "data/me/surahs/001/01.mp3"
AUDIO_PATH = Path(__file__).parent.parent / "data/me/surahs/001/01-mistake.mp3"


def load_audio_file(path: Path, sample_rate: int = 16000) -> np.ndarray:
    """Load MP3 audio and resample to the given sample rate."""
    if not Path(path).exists():
        raise FileNotFoundError(f"Audio file not found: {path}")
    try:
        import librosa
    except ImportError as e:
        raise RuntimeError(
            "librosa is required to load MP3 audio. Install with: pip install librosa"
        ) from e
    audio, _ = librosa.load(str(path), sr=sample_rate, mono=True)
    return audio.astype(np.float32)


def demo_basic_pipeline():
    """Demo: Basic M3 pipeline with synthetic audio."""
    print("=" * 80)
    print("DEMO 1: Basic M3 Pipeline")
    print("=" * 80)

    # Get reference text from Quran
    # Al-Fatihah (1), Ayah 1: بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ
    aya = Aya(1, 1)
    reference_text = aya.get().uthmani
    print(f"\nReference text: {reference_text}")

    # Load real audio
    print(f"\nLoading audio from {AUDIO_PATH} ...")
    audio = load_audio_file(AUDIO_PATH, sample_rate=16000)
    print(f"Audio shape: {audio.shape}, dtype: {audio.dtype}")

    # Initialize M3 pipeline
    print("\nInitializing M3 Pipeline...")
    print("(Muaalem model will download automatically if not cached)")
    pipeline = M3Pipeline(device="cpu")  # Use CPU for demo

    # Process audio
    print("\nProcessing audio through M3 pipeline...")
    print("-" * 80)

    try:
        # Try with gate enabled first
        result = pipeline.process(audio=audio, reference_text=reference_text, sample_rate=16000)
    except RuntimeError as e:
        # Gate failed - show error details and continue with skip_gate
        print(f"\n⚠️  {e}")
        print("\n" + "=" * 80)
        print("GATE FAILED - Processing anyway with skip_gate=True to show details...")
        print("=" * 80)
        result = pipeline.process(audio=audio, reference_text=reference_text, sample_rate=16000, skip_gate=True)

        print("\n" + "=" * 80)
        print("M3 PIPELINE RESULTS")
        print("=" * 80)

        # Gate result
        print(f"\n[Gate Result]")
        print(f"  Passed: {result.gate_result.passed}")
        print(f"  PER: {result.gate_result.per:.2%}")
        print(f"  Confidence: {result.gate_result.confidence:.2%}")
        print(f"  Errors: {len(result.gate_result.errors)}")

        # Show error details if any
        if result.gate_result.errors:
            print("\n  Error Details:")
            for i, error in enumerate(result.gate_result.errors[:10]):  # Show first 10
                err_type = error.get('type', 'unknown')
                pos = error.get('position', -1)
                ref = error.get('reference_phoneme', '?')
                pred = error.get('predicted_phoneme', '?')
                print(f"    {i+1}. {err_type.upper()} at position {pos}: expected '{ref}', got '{pred}'")

        # Phoneme results
        print(f"\n[Phonemes]")
        print(f"  Total: {len(result.phonemes)}")
        if result.phonemes:
            print(f"  First 5 phonemes:")
            for i, p in enumerate(result.phonemes[:5]):
                sifa_count = len(p.sifa) if p.sifa else 0
                print(
                    f"    {i+1}. '{p.phoneme}' @ {p.start:.2f}-{p.end:.2f}s "
                    f"(conf={p.confidence:.2f}, sifat={sifa_count})"
                )

        # Word results
        print(f"\n[Words]")
        print(f"  Total: {len(result.words)}")
        if result.words:
            print(f"  First 3 words:")
            for i, w in enumerate(result.words[:3]):
                print(
                    f"    {i+1}. '{w.word}' @ {w.start:.2f}-{w.end:.2f}s "
                    f"(phonemes: {len(w.phonemes)})"
                )

        # Alignment method
        print(f"\n[Alignment]")
        print(f"  Method: {result.alignment_method}")

        # Statistics
        print(f"\n[Statistics]")
        stats = pipeline.get_statistics(result)
        for key, value in stats.items():
            print(f"  {key}: {value}")

        # Save results
        output_file = Path(__file__).parent / "demo_m3_output.json"
        with open(output_file, "w", encoding="utf-8") as f:
            json.dump(result.to_dict(), f, ensure_ascii=False, indent=2, default=str)
        print(f"\nResults saved to: {output_file}")

    except Exception as e:
        print(f"\nERROR: {e}")
        import traceback

        traceback.print_exc()


def demo_sifat_extraction():
    """Demo: Focus on Tajweed sifat extraction."""
    print("\n\n" + "=" * 80)
    print("DEMO 2: Tajweed Sifat Extraction")
    print("=" * 80)

    # Short ayah for focused demo
    aya = Aya(1, 1)
    reference_text = aya.get().uthmani
    print(f"\nReference text: {reference_text}")

    # Load real audio
    audio = load_audio_file(AUDIO_PATH, sample_rate=16000)

    # Initialize pipeline
    pipeline = M3Pipeline(device="cpu")

    # Process (skip gate for this demo to focus on sifat)
    print("\nProcessing...")
    try:
        result = pipeline.process(audio=audio, reference_text=reference_text, sample_rate=16000)
    except RuntimeError:
        print("\n⚠️  Gate failed (expected with mistake audio)")
        print("Continuing with skip_gate=True to show sifat...\n")
        result = pipeline.process(audio=audio, reference_text=reference_text, sample_rate=16000, skip_gate=True)

    # Focus on sifat
    print("\n" + "=" * 80)
    print("TAJWEED SIFAT (First 3 Phonemes)")
    print("=" * 80)

    for i, phoneme in enumerate(result.phonemes[:3]):
        print(f"\n[Phoneme {i+1}]: '{phoneme.phoneme}'")
        print(f"  Time: {phoneme.start:.2f}s - {phoneme.end:.2f}s")
        print(f"  Confidence: {phoneme.confidence:.2%}")

        if phoneme.sifa:
            print(f"  Sifat ({len(phoneme.sifa)} properties):")
            for prop, value in phoneme.sifa.items():
                if value:
                    print(f"    - {prop}: {value['text']} (prob={value['prob']:.2f})")
        else:
            print(f"  Sifat: None")


def main():
    """Run all demos."""
    print("\n" + "#" * 80)
    print("# M3 PIPELINE DEMONSTRATION")
    print("# Module: Phoneme Recognition & Alignment with Muaalem")
    print("#" * 80)

    try:
        # Demo 1: Basic pipeline
        demo_basic_pipeline()

        # Demo 2: Sifat extraction
        demo_sifat_extraction()

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
