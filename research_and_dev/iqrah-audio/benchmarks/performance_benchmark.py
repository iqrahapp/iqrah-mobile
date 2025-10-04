"""
Performance Benchmarking Suite
================================

Measure RTF, latency, memory usage, and accuracy.
"""

import numpy as np
import time
import tracemalloc
from pathlib import Path
from dataclasses import dataclass
from typing import Dict, List
import json

from iqrah_audio import (
    PitchExtractor,
    AudioDenoiser,
    DTWAligner,
    RecitationScorer,
)


@dataclass
class BenchmarkResult:
    """Benchmark result metrics."""
    operation: str
    duration_s: float
    audio_duration_s: float
    rtf: float  # Real-time factor
    memory_mb: float
    extra_metrics: Dict = None

    def to_dict(self):
        return {
            "operation": self.operation,
            "duration_s": self.duration_s,
            "audio_duration_s": self.audio_duration_s,
            "rtf": self.rtf,
            "memory_mb": self.memory_mb,
            **(self.extra_metrics or {}),
        }


class PerformanceBenchmark:
    """Comprehensive performance benchmarking."""

    def __init__(self, sample_rate: int = 22050):
        self.sample_rate = sample_rate
        self.results: List[BenchmarkResult] = []

    def generate_test_audio(
        self,
        duration: float = 3.0,
        frequency: float = 220.0,
        noise_level: float = 0.0
    ) -> np.ndarray:
        """Generate test audio signal."""
        n_samples = int(self.sample_rate * duration)
        t = np.linspace(0, duration, n_samples)

        # Fundamental + harmonics (more realistic)
        signal = np.sin(2 * np.pi * frequency * t)
        signal += 0.5 * np.sin(2 * np.pi * 2 * frequency * t)  # 2nd harmonic
        signal += 0.3 * np.sin(2 * np.pi * 3 * frequency * t)  # 3rd harmonic

        if noise_level > 0:
            noise = np.random.normal(0, noise_level, n_samples)
            signal += noise

        return signal.astype(np.float32)

    def benchmark_operation(
        self,
        operation_name: str,
        func,
        audio: np.ndarray,
        *args,
        **kwargs
    ) -> BenchmarkResult:
        """Benchmark a single operation."""
        audio_duration = len(audio) / self.sample_rate

        # Start memory tracking
        tracemalloc.start()

        # Time the operation
        start = time.perf_counter()
        result = func(audio, *args, **kwargs)
        end = time.perf_counter()

        # Memory usage
        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()

        duration = end - start
        rtf = duration / audio_duration if audio_duration > 0 else float('inf')
        memory_mb = peak / 1024 / 1024

        bench_result = BenchmarkResult(
            operation=operation_name,
            duration_s=duration,
            audio_duration_s=audio_duration,
            rtf=rtf,
            memory_mb=memory_mb,
        )

        self.results.append(bench_result)
        return bench_result

    def run_pitch_benchmark(self, audio: np.ndarray):
        """Benchmark pitch extraction methods."""
        print("\n=== Pitch Extraction Benchmark ===")

        # YIN
        extractor_yin = PitchExtractor(method="yin", sample_rate=self.sample_rate)
        result = self.benchmark_operation(
            "Pitch: YIN",
            extractor_yin.extract,
            audio
        )
        print(f"YIN: {result.duration_s*1000:.1f}ms, RTF={result.rtf:.3f}, "
              f"Memory={result.memory_mb:.1f}MB")

        # CREPE (if available)
        try:
            extractor_crepe = PitchExtractor(
                method="crepe",
                sample_rate=self.sample_rate,
                crepe_model="tiny"
            )
            result = self.benchmark_operation(
                "Pitch: CREPE-tiny",
                extractor_crepe.extract,
                audio
            )
            print(f"CREPE-tiny: {result.duration_s*1000:.1f}ms, RTF={result.rtf:.3f}, "
                  f"Memory={result.memory_mb:.1f}MB")
        except Exception as e:
            print(f"CREPE not available: {e}")

        # Stable pitch (with filtering)
        result = self.benchmark_operation(
            "Pitch: YIN + Median Filter",
            extractor_yin.extract_stable_pitch,
            audio
        )
        print(f"YIN+Filter: {result.duration_s*1000:.1f}ms, RTF={result.rtf:.3f}, "
              f"Memory={result.memory_mb:.1f}MB")

    def run_denoise_benchmark(self, audio: np.ndarray):
        """Benchmark denoising."""
        print("\n=== Denoising Benchmark ===")

        denoiser = AudioDenoiser(sample_rate=self.sample_rate)
        result = self.benchmark_operation(
            "Denoise: Spectral Gate",
            denoiser.denoise,
            audio
        )
        print(f"Spectral Gate: {result.duration_s*1000:.1f}ms, RTF={result.rtf:.3f}, "
              f"Memory={result.memory_mb:.1f}MB")

    def run_dtw_benchmark(self):
        """Benchmark DTW alignment."""
        print("\n=== DTW Alignment Benchmark ===")

        # Generate test sequences
        seq_lengths = [50, 100, 200, 500, 1000]
        aligner = DTWAligner()

        for length in seq_lengths:
            # Create sequences
            reference = np.random.randn(length).astype(np.float32)
            query = reference + np.random.randn(length) * 0.1  # Slightly noisy

            # Benchmark
            start = time.perf_counter()
            result = aligner.align(query, reference)
            end = time.perf_counter()

            duration = end - start
            print(f"DTW ({length} frames): {duration*1000:.1f}ms")

    def run_full_pipeline_benchmark(self, audio: np.ndarray):
        """Benchmark full pipeline."""
        print("\n=== Full Pipeline Benchmark ===")

        tracemalloc.start()
        start = time.perf_counter()

        # Full pipeline
        denoiser = AudioDenoiser(sample_rate=self.sample_rate)
        extractor = PitchExtractor(method="yin", sample_rate=self.sample_rate)
        aligner = DTWAligner()
        scorer = RecitationScorer()

        # Process
        denoised = denoiser.denoise(audio)
        user_contour = extractor.extract_stable_pitch(denoised)

        # Create reference (copy with slight variation)
        ref_contour = user_contour  # Simplified for benchmark

        alignment = aligner.align(user_contour.f0_cents, ref_contour.f0_cents)
        score = scorer.score(user_contour, ref_contour, alignment)

        end = time.perf_counter()
        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()

        audio_duration = len(audio) / self.sample_rate
        duration = end - start
        rtf = duration / audio_duration
        memory_mb = peak / 1024 / 1024

        result = BenchmarkResult(
            operation="Full Pipeline (denoise+pitch+DTW+score)",
            duration_s=duration,
            audio_duration_s=audio_duration,
            rtf=rtf,
            memory_mb=memory_mb,
            extra_metrics={
                "score": score.overall_score,
            }
        )
        self.results.append(result)

        print(f"Full Pipeline: {duration*1000:.1f}ms, RTF={rtf:.3f}, "
              f"Memory={memory_mb:.1f}MB")
        print(f"Score: {score.overall_score:.1f}/100")

    def run_all_benchmarks(
        self,
        durations: List[float] = [1.0, 3.0, 5.0, 10.0]
    ):
        """Run comprehensive benchmark suite."""
        print("=" * 60)
        print("IQRAH AUDIO - Performance Benchmark")
        print("=" * 60)

        for duration in durations:
            print(f"\n{'='*60}")
            print(f"Testing with {duration}s audio")
            print(f"{'='*60}")

            # Clean audio
            audio_clean = self.generate_test_audio(duration=duration, noise_level=0.0)
            self.run_pitch_benchmark(audio_clean)
            self.run_denoise_benchmark(audio_clean)
            self.run_full_pipeline_benchmark(audio_clean)

            # Noisy audio
            if duration == 3.0:  # Only test once
                print(f"\n{'='*60}")
                print("Testing with noisy audio (SNR ~10dB)")
                print(f"{'='*60}")
                audio_noisy = self.generate_test_audio(
                    duration=duration,
                    noise_level=0.1
                )
                self.run_denoise_benchmark(audio_noisy)
                self.run_full_pipeline_benchmark(audio_noisy)

        self.run_dtw_benchmark()

    def save_results(self, output_path: Path):
        """Save benchmark results to JSON."""
        results_dict = {
            "benchmark_date": time.strftime("%Y-%m-%d %H:%M:%S"),
            "sample_rate": self.sample_rate,
            "results": [r.to_dict() for r in self.results],
        }

        with open(output_path, 'w') as f:
            json.dump(results_dict, f, indent=2)

        print(f"\nResults saved to {output_path}")

    def print_summary(self):
        """Print summary statistics."""
        print("\n" + "=" * 60)
        print("SUMMARY")
        print("=" * 60)

        # Group by operation
        ops = {}
        for result in self.results:
            op = result.operation
            if op not in ops:
                ops[op] = []
            ops[op].append(result)

        for op, results in ops.items():
            avg_rtf = np.mean([r.rtf for r in results])
            avg_time = np.mean([r.duration_s for r in results]) * 1000
            avg_mem = np.mean([r.memory_mb for r in results])

            print(f"\n{op}:")
            print(f"  Avg Time: {avg_time:.1f}ms")
            print(f"  Avg RTF: {avg_rtf:.3f}")
            print(f"  Avg Memory: {avg_mem:.1f}MB")


def main():
    """Run benchmarks."""
    benchmark = PerformanceBenchmark(sample_rate=22050)

    # Run all benchmarks
    benchmark.run_all_benchmarks(durations=[1.0, 3.0, 5.0])

    # Print summary
    benchmark.print_summary()

    # Save results
    output_dir = Path(__file__).parent / "results"
    output_dir.mkdir(exist_ok=True)
    benchmark.save_results(output_dir / "performance_benchmark.json")


if __name__ == "__main__":
    main()
