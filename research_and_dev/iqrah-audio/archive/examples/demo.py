#!/usr/bin/env python3
"""
Demo script for Iqrah Audio analysis.

Generates synthetic audio for demonstration without requiring real files.
"""

import numpy as np
import soundfile as sf
from pathlib import Path
import json

from iqrah_audio import (
    PitchExtractor,
    AudioDenoiser,
    DTWAligner,
    RecitationScorer,
    ReferenceProcessor,
)


def generate_quran_like_audio(
    base_freq=200.0,
    duration=3.0,
    sample_rate=22050,
    add_noise=False
):
    """
    Generate synthetic audio that mimics Qur'anic recitation patterns.

    Uses a simple melodic pattern with pitch variation.
    """
    t = np.linspace(0, duration, int(sample_rate * duration))

    # Create melodic pattern (simplified maqam)
    # Base frequency with some melodic movement
    freq_pattern = base_freq * (
        1.0 +
        0.05 * np.sin(2 * np.pi * 0.5 * t) +  # Slow vibrato
        0.1 * np.sin(2 * np.pi * 2.0 * t)     # Melodic movement
    )

    # Generate audio
    audio = np.sin(2 * np.pi * freq_pattern * t)

    # Add amplitude envelope (fade in/out)
    envelope = np.ones_like(t)
    fade_samples = int(0.1 * sample_rate)
    envelope[:fade_samples] = np.linspace(0, 1, fade_samples)
    envelope[-fade_samples:] = np.linspace(1, 0, fade_samples)

    audio *= envelope

    # Add noise if requested
    if add_noise:
        noise = np.random.normal(0, 0.05, len(audio))
        audio += noise

    return audio.astype(np.float32)


def main():
    """Run complete demo workflow."""
    print("=" * 60)
    print("Iqrah Audio - Demo Script")
    print("=" * 60)

    # Create output directory
    output_dir = Path("demo_output")
    output_dir.mkdir(exist_ok=True)

    # 1. Generate reference audio (Qari)
    print("\n[1/6] Generating reference qari audio...")
    ref_audio = generate_quran_like_audio(
        base_freq=200.0,
        duration=3.0,
        add_noise=False  # Clean qari recording
    )
    ref_path = output_dir / "reference_qari.wav"
    sf.write(ref_path, ref_audio, 22050)
    print(f"   Saved: {ref_path}")

    # 2. Process reference to CBOR
    print("\n[2/6] Processing reference to CBOR...")
    processor = ReferenceProcessor(
        sample_rate=22050,
        pitch_method="yin",  # Use YIN for demo (faster than CREPE)
        denoise=False
    )

    ref_data = processor.process_audio_file(
        ref_path,
        metadata={
            "ayah": "1:1",
            "qari": "demo",
            "description": "Synthetic reference for demo"
        }
    )

    cbor_path = output_dir / "reference.cbor.zst"
    processor.save_cbor(ref_data, cbor_path, compress=True)
    print(f"   Saved: {cbor_path}")
    print(f"   Duration: {ref_data['processing']['duration']:.2f}s")
    print(f"   Frames: {ref_data['processing']['n_frames']}")

    # 3. Generate user audio (slightly off-pitch + noise)
    print("\n[3/6] Generating user recitation...")
    user_audio = generate_quran_like_audio(
        base_freq=205.0,  # Slightly higher pitch
        duration=3.2,     # Slightly slower
        add_noise=True    # Noisy recording
    )
    user_path = output_dir / "user_recitation.wav"
    sf.write(user_path, user_audio, 22050)
    print(f"   Saved: {user_path}")

    # 4. Denoise user audio
    print("\n[4/6] Denoising user audio...")
    denoiser = AudioDenoiser(sample_rate=22050)
    user_audio_clean = denoiser.denoise_adaptive(user_audio)

    snr = denoiser.estimate_snr(user_audio, user_audio_clean)
    print(f"   SNR improvement: {snr:.1f} dB")

    # 5. Extract pitch contours
    print("\n[5/6] Extracting pitch contours...")
    extractor = PitchExtractor(sample_rate=22050, method="yin")

    user_contour = extractor.extract_stable_pitch(user_audio_clean, sr=22050)
    ref_contour = processor.get_contour_from_cbor(cbor_path)

    print(f"   User contour: {len(user_contour.f0_hz)} frames")
    print(f"   Reference contour: {len(ref_contour.f0_hz)} frames")

    # 6. Align and score
    print("\n[6/6] Aligning and scoring...")
    aligner = DTWAligner()
    alignment = aligner.align(user_contour.f0_cents, ref_contour.f0_cents)

    scorer = RecitationScorer()
    score = scorer.score(user_contour, ref_contour, alignment)

    # Display results
    print("\n" + "=" * 60)
    print("RECITATION ANALYSIS RESULTS")
    print("=" * 60)

    print(f"\n  Overall Score:     {score.overall_score:.1f}/100")
    print(f"  Alignment Score:   {score.alignment_score:.1f}/100")
    print(f"  On-Note %:         {score.on_note_percent:.1f}%")
    print(f"  Pitch Stability:   {score.pitch_stability:.1f}/100")
    print(f"  Tempo Score:       {score.tempo_score:.1f}/100")
    print(f"  Voiced Ratio:      {score.voiced_ratio:.1%}")

    print("\nDetailed Metrics:")
    for key, value in score.metrics.items():
        if isinstance(value, float):
            print(f"  {key}: {value:.2f}")
        else:
            print(f"  {key}: {value}")

    # Save results to JSON
    results_path = output_dir / "results.json"
    with open(results_path, 'w') as f:
        json.dump(score.to_dict(), f, indent=2)

    print(f"\n✓ Results saved to: {results_path}")

    # Summary
    print("\n" + "=" * 60)
    print("DEMO COMPLETE!")
    print("=" * 60)
    print(f"\nGenerated files in: {output_dir.absolute()}")
    print("  - reference_qari.wav (clean qari audio)")
    print("  - reference.cbor.zst (compressed pitch contour)")
    print("  - user_recitation.wav (noisy user audio)")
    print("  - results.json (analysis results)")

    print("\nNext steps:")
    print("  1. Try with real audio: iqrah-audio analyze user.wav ref.cbor.zst")
    print("  2. Process qari directory: iqrah-audio batch-process qari/ output/")
    print("  3. View reference: iqrah-audio inspect reference.cbor.zst")

    print("\n" + "=" * 60)


if __name__ == "__main__":
    main()
