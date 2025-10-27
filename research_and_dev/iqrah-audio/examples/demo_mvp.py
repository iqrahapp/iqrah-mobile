#!/usr/bin/env python3
"""
Demo script for Iqrah Audio MVP pipeline.

This demonstrates the complete flow:
1. Load audio
2. Compare to reference text
3. Get structured output with quality scores
"""

import sys
from pathlib import Path

# Add src to path for development imports
sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

import numpy as np
from iqrah.pipeline.compare_engine import ComparisonEngine


def demo_basic_usage():
    """Basic usage example with synthetic audio."""
    print("=" * 60)
    print("Iqrah Audio MVP Demo")
    print("=" * 60)

    # Initialize engine (loads ASR model)
    print("\n1. Initializing comparison engine...")
    engine = ComparisonEngine(
        asr_model_name="obadx/muaalem-model-v3_2",
        use_fp16=True
    )
    print("   ✓ Engine initialized")

    # Create synthetic audio (in practice, load from file)
    print("\n2. Loading audio...")
    sample_rate = 16000
    duration = 3.0  # seconds
    audio = np.random.randn(int(duration * sample_rate)).astype(np.float32)
    audio = audio / np.max(np.abs(audio))  # Normalize
    print(f"   ✓ Audio loaded: {duration}s at {sample_rate}Hz")

    # Reference text (Fatiha, first verse)
    reference_text = "بِسْمِ اللَّهِ الرَّحْمَٰنِ الرَّحِيمِ"
    print(f"\n3. Reference text: {reference_text}")

    # Run comparison
    print("\n4. Running comparison pipeline...")
    result = engine.compare(audio, reference_text, sample_rate)

    # Display results
    print("\n" + "=" * 60)
    print("RESULTS")
    print("=" * 60)

    print(f"\nStatus: {result['status']}")

    if result['status'] == 'content_error':
        print("\n⚠️  Content verification failed!")
        gate = result['content_verification']
        print(f"   Error rate: {gate['error_rate']:.2%}")
        print(f"   Confidence: {gate['confidence']}")
        print(f"   Message: {result['metadata']['message']}")
    else:
        print("\n✓ Content verification passed")

        # Content verification details
        gate = result['content_verification']
        print(f"\n   Error rate: {gate['error_rate']:.2%}")
        print(f"   Confidence: {gate['confidence']}")
        print(f"   Metric: {gate['metric_type'].upper()}")

        # Quality score
        print(f"\nOverall Quality Score: {result['quality_score']:.1f}/100")

        # Alignment details
        if result['tokens']:
            print(f"\nToken-level alignment:")
            for i, token in enumerate(result['tokens'][:5], 1):
                gop = token.get('gop_score', 0.0)
                print(f"   {i}. '{token['token']}' "
                      f"[{token['start']:.3f}s - {token['end']:.3f}s] "
                      f"conf={token['confidence']:.2f} "
                      f"llr={gop:.2f}")

            if len(result['tokens']) > 5:
                print(f"   ... and {len(result['tokens']) - 5} more tokens")

        # Tajweed results
        tajweed = result['tajweed']
        print(f"\nTajweed Validation:")
        print(f"   Overall score: {tajweed['overall_score']:.1f}/100")
        print(f"   Madd violations: {len(tajweed['madd_violations'])}")
        print(f"   Shadda violations: {len(tajweed['shadda_violations'])}")
        print(f"   Waqf violations: {len(tajweed['waqf_violations'])}")

        # Show first violation if any
        all_violations = (tajweed['madd_violations'] +
                         tajweed['shadda_violations'] +
                         tajweed['waqf_violations'])
        if all_violations:
            print(f"\n   First violation example:")
            v = all_violations[0]
            print(f"   - {v['message']}")
            print(f"     Severity: {v['severity']}")

    print("\n" + "=" * 60)


def demo_with_real_audio(audio_path: str, reference_text: str):
    """
    Demo with real audio file.

    Args:
        audio_path: Path to audio file (WAV, MP3, etc.)
        reference_text: Expected Quranic text with diacritics
    """
    import librosa

    print("=" * 60)
    print("Iqrah Audio MVP Demo - Real Audio")
    print("=" * 60)

    # Load audio
    print(f"\n1. Loading audio from: {audio_path}")
    audio, sr = librosa.load(audio_path, sr=16000, mono=True)
    print(f"   ✓ Loaded: {len(audio)/sr:.2f}s at {sr}Hz")

    # Initialize engine
    print("\n2. Initializing comparison engine...")
    engine = ComparisonEngine(use_fp16=True)
    print("   ✓ Engine initialized")

    # Run comparison
    print(f"\n3. Reference: {reference_text}")
    print("\n4. Running comparison pipeline...")
    result = engine.compare(audio, reference_text, sr)

    # Display results (same as above)
    print("\n" + "=" * 60)
    print("RESULTS")
    print("=" * 60)
    print(f"\nStatus: {result['status']}")
    print(f"Quality Score: {result.get('quality_score', 0):.1f}/100")

    if result['status'] == 'success':
        print(f"Tokens analyzed: {len(result['tokens'])}")
        print(f"Tajweed score: {result['tajweed']['overall_score']:.1f}/100")

    return result


if __name__ == "__main__":
    import sys

    if len(sys.argv) > 2:
        # Usage: python demo_mvp.py audio.wav "بِسْمِ اللَّهِ"
        audio_path = sys.argv[1]
        reference_text = sys.argv[2]
        demo_with_real_audio(audio_path, reference_text)
    else:
        # Demo with synthetic audio
        demo_basic_usage()

        print("\n\nTo test with real audio:")
        print('  python demo_mvp.py audio.wav "بِسْمِ اللَّهِ الرَّحْمَٰنِ الرَّحِيمِ"')
