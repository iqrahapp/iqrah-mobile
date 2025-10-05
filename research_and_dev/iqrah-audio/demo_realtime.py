#!/usr/bin/env python3
"""
Real-Time Quranic Recitation Analysis Demo
===========================================

Demonstrates the complete real-time streaming pipeline with live feedback.

This demo shows:
1. Loading reference recitation (Husary Al-Fatiha)
2. Processing user recitation in real-time chunks
3. Generating live coaching feedback
4. Displaying performance metrics

Usage:
    # Simulate streaming from audio file
    python demo_realtime.py

    # With custom reference
    python demo_realtime.py --reference path/to/reference.mp3

    # Adjust chunk size for different latencies
    python demo_realtime.py --chunk-size 1024

Performance:
    - Latency: ~3-7ms per chunk
    - Update rate: 15 Hz feedback
    - Supports real-time audio input
"""

import argparse
import numpy as np
import soundfile as sf
import time
from pathlib import Path
from typing import Optional

# Import the complete streaming pipeline
from iqrah_audio.streaming import (
    RealtimePipeline,
    PipelineConfig,
    RealtimeHints,
)


class RealtimeDemo:
    """
    Real-time recitation analysis demo.

    Simulates live audio streaming and provides visual feedback.
    """

    def __init__(
        self,
        reference_audio_path: str,
        chunk_size: int = 512,
        update_rate_hz: float = 15.0,
        verbose: bool = True,
    ):
        """
        Initialize demo.

        Args:
            reference_audio_path: Path to reference recitation
            chunk_size: Audio chunk size in samples
            update_rate_hz: Feedback update rate (Hz)
            verbose: Print detailed output
        """
        self.chunk_size = chunk_size
        self.verbose = verbose

        # Load reference audio
        print("=" * 80)
        print("REAL-TIME QURANIC RECITATION ANALYSIS DEMO")
        print("=" * 80)
        print(f"\n📖 Loading reference: {reference_audio_path}")

        audio, sr = sf.read(reference_audio_path)
        if len(audio.shape) > 1:
            audio = audio.mean(axis=1)
        self.reference_audio = audio.astype(np.float32)
        self.sample_rate = sr

        duration = len(self.reference_audio) / sr
        print(f"✓ Reference loaded: {duration:.2f}s @ {sr} Hz")

        # Create pipeline configuration
        config = PipelineConfig(
            sample_rate=sr,
            hop_length=512,
            buffer_size_s=3.0,
            pitch_method="yin",
            dtw_window_size=300,
            dtw_band_width=50,
            confidence_threshold=0.6,
            update_rate_hz=update_rate_hz,
            on_note_threshold_cents=50.0,
            smoothing_alpha=0.3,
            enable_anchors=True,
            anchor_min_confidence=0.7,
        )

        # Initialize pipeline
        print("\n🔧 Initializing real-time pipeline...")
        self.pipeline = RealtimePipeline(
            reference_audio=self.reference_audio,
            config=config,
            on_hints_callback=self._on_hints_callback if verbose else None,
        )

        print(f"✓ Pipeline ready")
        print(f"  Configuration:")
        print(f"    - Chunk size: {chunk_size} samples (~{chunk_size/sr*1000:.1f}ms)")
        print(f"    - Update rate: {update_rate_hz} Hz")
        print(f"    - Anchors: {len(self.pipeline.reference_anchors)}")

        # Statistics
        self.total_hints = 0
        self.status_counts = {"good": 0, "warning": 0, "error": 0, "acquiring": 0}
        self.chunk_latencies = []

    def _on_hints_callback(self, hints: RealtimeHints):
        """Callback for real-time hints (called by pipeline)."""
        self.total_hints += 1
        self.status_counts[hints.status] += 1

        # Display hint with visual cue
        icon = {
            "green": "✓",
            "yellow": "⚠",
            "red": "✗",
            "gray": "○",
        }.get(hints.visual_cue, "·")

        print(f"  [{hints.visual_cue:6s}] {icon} {hints.message}")

    def process_audio_file(self, user_audio_path: str):
        """
        Process audio file in streaming fashion.

        Args:
            user_audio_path: Path to user recitation audio
        """
        print("\n" + "=" * 80)
        print("PROCESSING USER RECITATION")
        print("=" * 80)
        print(f"\n🎤 Loading user audio: {user_audio_path}")

        # Load user audio
        audio, sr = sf.read(user_audio_path)
        if len(audio.shape) > 1:
            audio = audio.mean(axis=1)
        user_audio = audio.astype(np.float32)

        if sr != self.sample_rate:
            print(f"⚠ Warning: Sample rate mismatch ({sr} vs {self.sample_rate})")

        duration = len(user_audio) / sr
        print(f"✓ User audio loaded: {duration:.2f}s @ {sr} Hz")

        # Process in chunks
        n_chunks = len(user_audio) // self.chunk_size
        print(f"\n▶ Processing {n_chunks} chunks...")
        print("-" * 80)

        start_time = time.time()

        for i in range(n_chunks):
            chunk = user_audio[i * self.chunk_size:(i + 1) * self.chunk_size]

            # Process chunk
            t0 = time.perf_counter()
            hints = self.pipeline.process_chunk(chunk)
            t1 = time.perf_counter()

            latency_ms = (t1 - t0) * 1000
            self.chunk_latencies.append(latency_ms)

            # Show progress every 50 chunks
            if self.verbose and i > 0 and i % 50 == 0:
                elapsed = time.time() - start_time
                progress = (i / n_chunks) * 100
                print(f"\n  Progress: {progress:.1f}% ({i}/{n_chunks}) | "
                      f"Elapsed: {elapsed:.1f}s | Latency: {latency_ms:.2f}ms")

        total_time = time.time() - start_time
        print("-" * 80)
        print(f"✓ Processing complete: {total_time:.2f}s")

        # Display results
        self._display_results()

    def process_self_alignment(self):
        """
        Process reference audio against itself (for testing).

        This simulates perfect recitation to validate the pipeline.
        """
        print("\n" + "=" * 80)
        print("SELF-ALIGNMENT TEST (Reference vs Reference)")
        print("=" * 80)
        print("\n🔄 Processing reference audio against itself...")
        print("   (This simulates perfect recitation)")

        # Reinitialize pipeline with forced seed at position 0
        print("\n🔧 Reinitializing pipeline for self-alignment...")
        config = PipelineConfig(
            sample_rate=self.sample_rate,
            hop_length=512,
            buffer_size_s=3.0,
            pitch_method="yin",
            dtw_window_size=300,
            dtw_band_width=50,
            confidence_threshold=0.6,
            update_rate_hz=15.0,
            on_note_threshold_cents=50.0,
            smoothing_alpha=0.3,
            enable_anchors=True,
            anchor_min_confidence=0.7,
            oltw_force_seed_at_start=True,  # Force seed at position 0 for self-alignment
        )

        self.pipeline = RealtimePipeline(
            reference_audio=self.reference_audio,
            config=config,
            on_hints_callback=self._on_hints_callback if self.verbose else None,
        )
        print("✓ Pipeline ready (forced seed at position 0)")

        # Process in chunks
        n_chunks = len(self.reference_audio) // self.chunk_size
        print(f"\n▶ Processing {n_chunks} chunks...")
        print("-" * 80)

        start_time = time.time()

        for i in range(n_chunks):
            chunk = self.reference_audio[i * self.chunk_size:(i + 1) * self.chunk_size]

            # Process chunk
            t0 = time.perf_counter()
            hints = self.pipeline.process_chunk(chunk)
            t1 = time.perf_counter()

            latency_ms = (t1 - t0) * 1000
            self.chunk_latencies.append(latency_ms)

            # Show progress every 50 chunks
            if self.verbose and i > 0 and i % 50 == 0:
                elapsed = time.time() - start_time
                progress = (i / n_chunks) * 100
                print(f"\n  Progress: {progress:.1f}% ({i}/{n_chunks}) | "
                      f"Elapsed: {elapsed:.1f}s | Latency: {latency_ms:.2f}ms")

        total_time = time.time() - start_time
        print("-" * 80)
        print(f"✓ Processing complete: {total_time:.2f}s")

        # Display results
        self._display_results()

    def _display_results(self):
        """Display comprehensive results."""
        print("\n" + "=" * 80)
        print("RESULTS")
        print("=" * 80)

        # Get pipeline stats
        stats = self.pipeline.get_stats()
        alignment_state = self.pipeline.get_alignment_state()

        # Performance metrics
        print("\n📊 Performance Metrics:")
        print(f"  Total frames processed: {stats.total_frames_processed}")
        print(f"  Total hints generated: {self.total_hints}")
        print(f"  Audio duration: {stats.total_audio_duration_s:.2f}s")

        print("\n⏱  Latency Breakdown:")
        print(f"  Pitch extraction: {stats.pitch_latency_ms:.2f}ms")
        print(f"  Anchor detection: {stats.anchor_latency_ms:.2f}ms")
        print(f"  DTW alignment:    {stats.dtw_latency_ms:.2f}ms")
        print(f"  Feedback gen:     {stats.feedback_latency_ms:.2f}ms")
        print(f"  ─────────────────────────────")
        print(f"  TOTAL:            {stats.total_latency_ms:.2f}ms")

        if len(self.chunk_latencies) > 0:
            avg_chunk = np.mean(self.chunk_latencies)
            max_chunk = np.max(self.chunk_latencies)
            p95_chunk = np.percentile(self.chunk_latencies, 95)

            print(f"\n  Per-chunk latency:")
            print(f"    Average: {avg_chunk:.2f}ms")
            print(f"    Max:     {max_chunk:.2f}ms")
            print(f"    P95:     {p95_chunk:.2f}ms")

            if avg_chunk < 10:
                print(f"    ✓ EXCELLENT (<10ms)")
            elif avg_chunk < 50:
                print(f"    ✓ GOOD (<50ms)")
            elif avg_chunk < 100:
                print(f"    ✓ ACCEPTABLE (<100ms)")
            else:
                print(f"    ⚠ HIGH LATENCY (>100ms)")

        # Feedback quality
        print("\n🎯 Feedback Quality:")
        if self.total_hints > 0:
            for status, count in sorted(self.status_counts.items()):
                if count > 0:
                    pct = count / self.total_hints * 100
                    icon = {
                        "good": "✓",
                        "warning": "⚠",
                        "error": "✗",
                        "acquiring": "○",
                    }.get(status, "·")
                    print(f"  {icon} {status:10s}: {count:4d} ({pct:5.1f}%)")

            # Accuracy estimate
            good_pct = self.status_counts["good"] / self.total_hints * 100
            if good_pct > 80:
                print(f"\n  Overall: EXCELLENT ({good_pct:.1f}% good)")
            elif good_pct > 50:
                print(f"\n  Overall: GOOD ({good_pct:.1f}% good)")
            elif good_pct > 20:
                print(f"\n  Overall: FAIR ({good_pct:.1f}% good)")
            else:
                print(f"\n  Overall: NEEDS IMPROVEMENT ({good_pct:.1f}% good)")
        else:
            print("  (No feedback generated)")

        # Alignment state
        print("\n🎵 Final Alignment State:")
        print(f"  Reference position: {alignment_state.reference_position}")
        print(f"  Lead/lag: {alignment_state.lead_lag_ms:+.1f}ms")
        print(f"  Confidence: {alignment_state.confidence:.2f}")
        print(f"  Status: {alignment_state.status}")
        print(f"  Drift estimate: {alignment_state.drift_estimate:.2f}")

        print("\n" + "=" * 80)


def main():
    """Main demo entry point."""
    parser = argparse.ArgumentParser(
        description="Real-time Quranic recitation analysis demo"
    )
    parser.add_argument(
        "--reference",
        type=str,
        default="data/husary/surahs/01.mp3",
        help="Path to reference recitation (default: Husary Al-Fatiha)",
    )
    parser.add_argument(
        "--user",
        type=str,
        default=None,
        help="Path to user recitation (default: use reference for self-test)",
    )
    parser.add_argument(
        "--chunk-size",
        type=int,
        default=512,
        help="Audio chunk size in samples (default: 512)",
    )
    parser.add_argument(
        "--update-rate",
        type=float,
        default=15.0,
        help="Feedback update rate in Hz (default: 15.0)",
    )
    parser.add_argument(
        "--quiet",
        action="store_true",
        help="Suppress detailed output",
    )

    args = parser.parse_args()

    # Check if reference exists
    ref_path = Path(args.reference)
    if not ref_path.exists():
        print(f"❌ Error: Reference audio not found: {args.reference}")
        print(f"\nPlease provide a valid reference audio file.")
        print(f"Example: python demo_realtime.py --reference path/to/surah.mp3")
        return 1

    # Create demo
    try:
        demo = RealtimeDemo(
            reference_audio_path=str(ref_path),
            chunk_size=args.chunk_size,
            update_rate_hz=args.update_rate,
            verbose=not args.quiet,
        )

        # Process user audio or self-test
        if args.user:
            user_path = Path(args.user)
            if not user_path.exists():
                print(f"❌ Error: User audio not found: {args.user}")
                return 1
            demo.process_audio_file(str(user_path))
        else:
            print("\n💡 No user audio provided - running self-alignment test")
            demo.process_self_alignment()

        print("\n✅ Demo complete!")
        return 0

    except Exception as e:
        print(f"\n❌ Error: {e}")
        import traceback
        traceback.print_exc()
        return 1


if __name__ == "__main__":
    exit(main())
