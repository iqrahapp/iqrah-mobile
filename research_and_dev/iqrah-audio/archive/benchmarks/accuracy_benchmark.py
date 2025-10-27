"""
Accuracy Benchmarking Suite
============================

Test pitch tracking accuracy, alignment quality, and scoring reliability.
"""

import numpy as np
from pathlib import Path
from dataclasses import dataclass
from typing import List, Tuple
import json

from iqrah_audio import PitchExtractor, DTWAligner, RecitationScorer, PitchContour


@dataclass
class AccuracyResult:
    """Accuracy test result."""
    test_name: str
    pitch_mae_cents: float  # Mean absolute error
    pitch_rmse_cents: float  # Root mean squared error
    octave_error_rate: float  # Percentage of octave errors
    on_note_percent: float  # Percentage within ±50 cents
    voicing_accuracy: float  # Voiced/unvoiced classification
    extra_metrics: dict = None

    def to_dict(self):
        return {
            "test_name": self.test_name,
            "pitch_mae_cents": self.pitch_mae_cents,
            "pitch_rmse_cents": self.pitch_rmse_cents,
            "octave_error_rate": self.octave_error_rate,
            "on_note_percent": self.on_note_percent,
            "voicing_accuracy": self.voicing_accuracy,
            **(self.extra_metrics or {}),
        }


class AccuracyBenchmark:
    """Test pitch tracking and alignment accuracy."""

    def __init__(self, sample_rate: int = 22050):
        self.sample_rate = sample_rate
        self.results: List[AccuracyResult] = []

    def generate_ground_truth_audio(
        self,
        f0_hz: np.ndarray,
        voiced: np.ndarray,
        duration: float = 3.0,
        noise_level: float = 0.0,
        harmonics: bool = True,
    ) -> Tuple[np.ndarray, np.ndarray]:
        """
        Generate audio with known ground truth F0.

        Args:
            f0_hz: Ground truth F0 trajectory
            voiced: Voicing flags
            duration: Duration in seconds
            noise_level: Noise level
            harmonics: Add harmonics for realism

        Returns:
            (audio, ground_truth_f0_hz)
        """
        n_samples = int(self.sample_rate * duration)
        t = np.linspace(0, duration, n_samples)

        # Interpolate F0 to sample rate
        f0_times = np.linspace(0, duration, len(f0_hz))
        f0_interpolated = np.interp(t, f0_times, f0_hz)
        voiced_interpolated = np.interp(t, f0_times, voiced.astype(float)) > 0.5

        # Generate audio
        audio = np.zeros(n_samples)
        phase = 0.0

        for i in range(n_samples):
            if voiced_interpolated[i]:
                freq = f0_interpolated[i]

                # Fundamental
                audio[i] = np.sin(phase)

                if harmonics:
                    # Add harmonics
                    audio[i] += 0.5 * np.sin(2 * phase)  # 2nd
                    audio[i] += 0.3 * np.sin(3 * phase)  # 3rd
                    audio[i] += 0.2 * np.sin(4 * phase)  # 4th

                # Update phase
                phase += 2 * np.pi * freq / self.sample_rate
                phase = phase % (2 * np.pi)

        # Add noise
        if noise_level > 0:
            noise = np.random.normal(0, noise_level, n_samples)
            audio += noise

        # Normalize
        if np.max(np.abs(audio)) > 0:
            audio /= np.max(np.abs(audio))

        return audio.astype(np.float32), f0_hz

    def calculate_pitch_errors(
        self,
        estimated: PitchContour,
        ground_truth_f0_hz: np.ndarray,
        ground_truth_voiced: np.ndarray,
    ) -> AccuracyResult:
        """
        Calculate pitch tracking errors.

        Args:
            estimated: Estimated pitch contour
            ground_truth_f0_hz: Ground truth F0
            ground_truth_voiced: Ground truth voicing

        Returns:
            AccuracyResult
        """
        # Interpolate ground truth to match estimated timestamps
        gt_times = np.linspace(0, estimated.duration, len(ground_truth_f0_hz))
        gt_f0_interp = np.interp(
            estimated.timestamps,
            gt_times,
            ground_truth_f0_hz
        )
        gt_voiced_interp = np.interp(
            estimated.timestamps,
            gt_times,
            ground_truth_voiced.astype(float)
        ) > 0.5

        # Convert to cents
        with np.errstate(divide='ignore', invalid='ignore'):
            gt_cents = 1200 * np.log2(gt_f0_interp / 440.0)
            gt_cents[gt_f0_interp == 0] = 0

        est_cents = estimated.f0_cents

        # Voiced frames only
        voiced_mask = (estimated.confidence > 0.5) & gt_voiced_interp & (gt_f0_interp > 0)

        if np.sum(voiced_mask) == 0:
            return AccuracyResult(
                test_name="Unknown",
                pitch_mae_cents=float('inf'),
                pitch_rmse_cents=float('inf'),
                octave_error_rate=1.0,
                on_note_percent=0.0,
                voicing_accuracy=0.0,
            )

        # Pitch errors (voiced frames only)
        errors_cents = est_cents[voiced_mask] - gt_cents[voiced_mask]

        # Detect octave errors (error > 600 cents = half octave)
        octave_errors = np.abs(errors_cents) > 600
        octave_error_rate = np.mean(octave_errors)

        # MAE and RMSE
        mae_cents = np.mean(np.abs(errors_cents))
        rmse_cents = np.sqrt(np.mean(errors_cents ** 2))

        # On-note percentage (within ±50 cents)
        on_note = np.abs(errors_cents) <= 50
        on_note_percent = np.mean(on_note) * 100

        # Voicing accuracy
        est_voiced = estimated.confidence > 0.5
        voicing_correct = (est_voiced == gt_voiced_interp)
        voicing_accuracy = np.mean(voicing_correct)

        return AccuracyResult(
            test_name="Unknown",
            pitch_mae_cents=float(mae_cents),
            pitch_rmse_cents=float(rmse_cents),
            octave_error_rate=float(octave_error_rate),
            on_note_percent=float(on_note_percent),
            voicing_accuracy=float(voicing_accuracy),
            extra_metrics={
                "n_voiced_frames": int(np.sum(voiced_mask)),
                "median_error_cents": float(np.median(np.abs(errors_cents))),
            }
        )

    def test_constant_pitch(self, method: str = "yin"):
        """Test on constant pitch (easiest case)."""
        print(f"\n=== Test: Constant Pitch ({method.upper()}) ===")

        # Ground truth: constant A4 (440 Hz)
        n_frames = 150
        gt_f0 = np.full(n_frames, 440.0)
        gt_voiced = np.ones(n_frames, dtype=bool)

        audio, _ = self.generate_ground_truth_audio(gt_f0, gt_voiced, duration=3.0)

        # Extract pitch
        extractor = PitchExtractor(method=method, sample_rate=self.sample_rate)
        estimated = extractor.extract_stable_pitch(audio)

        # Calculate errors
        result = self.calculate_pitch_errors(estimated, gt_f0, gt_voiced)
        result.test_name = f"Constant Pitch ({method})"

        self.results.append(result)

        print(f"MAE: {result.pitch_mae_cents:.1f} cents")
        print(f"RMSE: {result.pitch_rmse_cents:.1f} cents")
        print(f"On-Note %: {result.on_note_percent:.1f}%")
        print(f"Octave Error Rate: {result.octave_error_rate*100:.1f}%")
        print(f"Voicing Accuracy: {result.voicing_accuracy*100:.1f}%")

    def test_vibrato(self, method: str = "yin"):
        """Test on pitch with vibrato."""
        print(f"\n=== Test: Vibrato ({method.upper()}) ===")

        # Ground truth: A4 with 6 Hz vibrato, ±30 cents
        n_frames = 150
        t_frames = np.linspace(0, 3.0, n_frames)

        # Base F0 with sinusoidal vibrato
        vibrato_hz = 6.0  # 6 Hz vibrato rate
        vibrato_depth_cents = 30.0

        base_f0 = 440.0
        vibrato_cents = vibrato_depth_cents * np.sin(2 * np.pi * vibrato_hz * t_frames)

        # Convert cents back to Hz
        gt_f0 = base_f0 * 2 ** (vibrato_cents / 1200)
        gt_voiced = np.ones(n_frames, dtype=bool)

        audio, _ = self.generate_ground_truth_audio(gt_f0, gt_voiced, duration=3.0)

        # Extract pitch
        extractor = PitchExtractor(method=method, sample_rate=self.sample_rate)
        estimated = extractor.extract_stable_pitch(audio, median_filter_size=3)

        # Calculate errors
        result = self.calculate_pitch_errors(estimated, gt_f0, gt_voiced)
        result.test_name = f"Vibrato ({method})"

        self.results.append(result)

        print(f"MAE: {result.pitch_mae_cents:.1f} cents")
        print(f"RMSE: {result.pitch_rmse_cents:.1f} cents")
        print(f"On-Note %: {result.on_note_percent:.1f}%")

    def test_octave_jumps(self, method: str = "yin"):
        """Test on octave jumps."""
        print(f"\n=== Test: Octave Jumps ({method.upper()}) ===")

        # Ground truth: jumps between A3, A4, A5
        n_frames = 150
        gt_f0 = np.zeros(n_frames)

        # A3 (220 Hz) for first third
        gt_f0[:50] = 220.0
        # A4 (440 Hz) for middle third
        gt_f0[50:100] = 440.0
        # A5 (880 Hz) for last third
        gt_f0[100:] = 880.0

        gt_voiced = np.ones(n_frames, dtype=bool)

        audio, _ = self.generate_ground_truth_audio(gt_f0, gt_voiced, duration=3.0)

        # Extract pitch
        extractor = PitchExtractor(method=method, sample_rate=self.sample_rate)
        estimated = extractor.extract_stable_pitch(audio)

        # Calculate errors
        result = self.calculate_pitch_errors(estimated, gt_f0, gt_voiced)
        result.test_name = f"Octave Jumps ({method})"

        self.results.append(result)

        print(f"MAE: {result.pitch_mae_cents:.1f} cents")
        print(f"Octave Error Rate: {result.octave_error_rate*100:.1f}%")

    def test_noisy_audio(self, method: str = "yin", snr_db: float = 10.0):
        """Test on noisy audio."""
        print(f"\n=== Test: Noisy Audio ({method.upper()}, SNR={snr_db}dB) ===")

        # Ground truth: constant pitch
        n_frames = 150
        gt_f0 = np.full(n_frames, 440.0)
        gt_voiced = np.ones(n_frames, dtype=bool)

        # Calculate noise level for target SNR
        signal_power = 1.0  # After normalization
        noise_power = signal_power / (10 ** (snr_db / 10))
        noise_level = np.sqrt(noise_power)

        audio, _ = self.generate_ground_truth_audio(
            gt_f0,
            gt_voiced,
            duration=3.0,
            noise_level=noise_level
        )

        # Extract pitch
        extractor = PitchExtractor(method=method, sample_rate=self.sample_rate)
        estimated = extractor.extract_stable_pitch(audio)

        # Calculate errors
        result = self.calculate_pitch_errors(estimated, gt_f0, gt_voiced)
        result.test_name = f"Noisy ({method}, SNR={snr_db}dB)"

        self.results.append(result)

        print(f"MAE: {result.pitch_mae_cents:.1f} cents")
        print(f"Voicing Accuracy: {result.voicing_accuracy*100:.1f}%")

    def run_all_tests(self):
        """Run comprehensive accuracy tests."""
        print("=" * 60)
        print("IQRAH AUDIO - Accuracy Benchmark")
        print("=" * 60)

        methods = ["yin"]

        # Try to test CREPE if available
        try:
            from iqrah_audio.pitch import CREPE_AVAILABLE
            if CREPE_AVAILABLE:
                methods.append("crepe")
        except:
            pass

        for method in methods:
            print(f"\n{'='*60}")
            print(f"Testing {method.upper()}")
            print(f"{'='*60}")

            self.test_constant_pitch(method)
            self.test_vibrato(method)
            self.test_octave_jumps(method)
            self.test_noisy_audio(method, snr_db=20)
            self.test_noisy_audio(method, snr_db=10)
            self.test_noisy_audio(method, snr_db=5)

    def save_results(self, output_path: Path):
        """Save results to JSON."""
        results_dict = {
            "results": [r.to_dict() for r in self.results],
        }

        with open(output_path, 'w') as f:
            json.dump(results_dict, f, indent=2)

        print(f"\nResults saved to {output_path}")

    def print_summary(self):
        """Print summary table."""
        print("\n" + "=" * 60)
        print("SUMMARY")
        print("=" * 60)

        print(f"\n{'Test':<30} {'MAE (cents)':<15} {'Octave Err %':<15} {'On-Note %':<15}")
        print("-" * 75)

        for result in self.results:
            print(f"{result.test_name:<30} "
                  f"{result.pitch_mae_cents:<15.1f} "
                  f"{result.octave_error_rate*100:<15.1f} "
                  f"{result.on_note_percent:<15.1f}")


def main():
    """Run accuracy benchmarks."""
    benchmark = AccuracyBenchmark(sample_rate=22050)
    benchmark.run_all_tests()
    benchmark.print_summary()

    # Save results
    output_dir = Path(__file__).parent / "results"
    output_dir.mkdir(exist_ok=True)
    benchmark.save_results(output_dir / "accuracy_benchmark.json")


if __name__ == "__main__":
    main()
