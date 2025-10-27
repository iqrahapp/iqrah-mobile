"""
RMVPE Pitch Tracker Integration
=================================

RMVPE (Robust Multi-Period Variational Pitch Estimation) is SOTA for vocals.
Better than CREPE for singing/recitation with:
- Higher accuracy on breathy/noisy vocals
- Better octave error resistance
- Lower latency (smaller model)

Installation:
    pip install torch torchcrepe
    # Download RMVPE weights: https://github.com/yxlllc/RMVPE

Model info:
    - Size: ~9 MB (vs CREPE tiny: ~5 MB, full: 250 MB)
    - Latency: ~50ms for 3s audio (vs CREPE: ~180ms)
    - Accuracy: Better than CREPE on vocals
"""

import numpy as np
from pathlib import Path
from typing import Optional, Tuple
import warnings

from .pitch import PitchContour

# Try to import RMVPE dependencies
try:
    import torch
    import torch.nn as nn
    import torch.nn.functional as F
    TORCH_AVAILABLE = True
except ImportError:
    TORCH_AVAILABLE = False
    warnings.warn("PyTorch not available. Install with: pip install torch")

try:
    import torchcrepe
    TORCHCREPE_AVAILABLE = True
except ImportError:
    TORCHCREPE_AVAILABLE = False


class TorchCrepeExtractor:
    """
    Fast CREPE implementation using torchcrepe.

    Much faster than the original CREPE (uses GPU if available).
    Good middle ground between YIN and RMVPE.
    """

    def __init__(
        self,
        sample_rate: int = 22050,
        hop_length: int = 512,
        model: str = "tiny",  # "tiny" or "full" (torchcrepe only supports these two)
        device: Optional[str] = None,
        fmin: float = 50.0,
        fmax: float = 1000.0,
    ):
        """
        Initialize TorchCrepe extractor.

        Args:
            sample_rate: Audio sample rate
            hop_length: Hop length in samples
            model: Model size
            device: "cuda", "cpu", or None (auto-detect)
            fmin: Minimum frequency (Hz)
            fmax: Maximum frequency (Hz)
        """
        if not TORCH_AVAILABLE or not TORCHCREPE_AVAILABLE:
            raise ImportError(
                "torchcrepe requires PyTorch. Install with: "
                "pip install torch torchcrepe"
            )

        self.sample_rate = sample_rate
        self.hop_length = hop_length
        self.model = model
        self.fmin = fmin
        self.fmax = fmax

        # Auto-detect device
        if device is None:
            self.device = "cuda" if torch.cuda.is_available() else "cpu"
        else:
            self.device = device

    def extract(self, audio: np.ndarray, sr: Optional[int] = None) -> PitchContour:
        """
        Extract pitch using torchcrepe.

        Args:
            audio: Audio signal (1D numpy array)
            sr: Sample rate (will resample if different)

        Returns:
            PitchContour
        """
        # Resample if needed
        if sr is not None and sr != self.sample_rate:
            import resampy
            audio = resampy.resample(audio, sr, self.sample_rate)

        # Convert to torch tensor
        audio_torch = torch.from_numpy(audio).float().unsqueeze(0).to(self.device)

        # Extract pitch
        hop_length_ms = int(1000 * self.hop_length / self.sample_rate)

        with torch.no_grad():
            pitch, periodicity = torchcrepe.predict(
                audio_torch,
                self.sample_rate,
                hop_length=hop_length_ms,
                fmin=self.fmin,
                fmax=self.fmax,
                model=self.model,
                return_periodicity=True,
                device=self.device,
                batch_size=1,
            )

        # Convert back to numpy
        f0_hz = pitch.squeeze().cpu().numpy()
        confidence = periodicity.squeeze().cpu().numpy()

        # Generate timestamps
        n_frames = len(f0_hz)
        timestamps = np.arange(n_frames) * (self.hop_length / self.sample_rate)

        return PitchContour(
            f0_hz=f0_hz.astype(np.float32),
            confidence=confidence.astype(np.float32),
            timestamps=timestamps.astype(np.float32),
            sample_rate=self.sample_rate,
        )


class RMVPEExtractor:
    """
    RMVPE pitch extractor - SOTA for vocals.

    NOTE: This is a placeholder for RMVPE integration.
    Full RMVPE requires downloading weights from:
    https://github.com/yxlllc/RMVPE

    For now, we use TorchCrepe as a faster alternative to original CREPE.
    """

    def __init__(
        self,
        sample_rate: int = 22050,
        hop_length: int = 512,
        device: Optional[str] = None,
        model_path: Optional[Path] = None,
    ):
        """
        Initialize RMVPE extractor.

        Args:
            sample_rate: Audio sample rate
            hop_length: Hop length in samples
            device: "cuda", "cpu", or None
            model_path: Path to RMVPE weights (if available)
        """
        self.sample_rate = sample_rate
        self.hop_length = hop_length
        self.device = device or ("cuda" if torch.cuda.is_available() else "cpu")
        self.model_path = model_path

        # For now, fall back to TorchCrepe
        if model_path is None or not model_path.exists():
            warnings.warn(
                "RMVPE weights not found. Using TorchCrepe instead. "
                "For true RMVPE, download weights from: "
                "https://github.com/yxlllc/RMVPE"
            )
            self.backend = TorchCrepeExtractor(
                sample_rate=sample_rate,
                hop_length=hop_length,
                model="tiny",  # torchcrepe only supports "tiny" or "full"
                device=self.device,
            )
        else:
            # TODO: Load actual RMVPE model
            # self.model = self._load_rmvpe_model(model_path)
            raise NotImplementedError(
                "Full RMVPE loading not yet implemented. "
                "Using TorchCrepe as backend."
            )

    def extract(self, audio: np.ndarray, sr: Optional[int] = None) -> PitchContour:
        """
        Extract pitch using RMVPE (or TorchCrepe backend).

        Args:
            audio: Audio signal
            sr: Sample rate

        Returns:
            PitchContour
        """
        return self.backend.extract(audio, sr)


class EnsemblePitchExtractor:
    """
    Ensemble pitch extractor using multiple methods.

    Combines predictions from multiple pitch trackers for robustness.
    Strategy: Use weighted voting or median filtering across methods.
    """

    def __init__(
        self,
        sample_rate: int = 22050,
        hop_length: int = 512,
        methods: list[str] = ["yin", "torchcrepe"],
        weights: Optional[dict] = None,
    ):
        """
        Initialize ensemble extractor.

        Args:
            sample_rate: Audio sample rate
            hop_length: Hop length
            methods: List of methods to use ["yin", "crepe", "torchcrepe", "rmvpe"]
            weights: Method weights for voting (default: equal)
        """
        from .pitch import PitchExtractor

        self.sample_rate = sample_rate
        self.hop_length = hop_length
        self.methods = methods
        self.weights = weights or {method: 1.0 for method in methods}

        # Initialize extractors
        self.extractors = {}

        for method in methods:
            if method == "yin":
                self.extractors["yin"] = PitchExtractor(
                    method="yin",
                    sample_rate=sample_rate,
                    hop_length=hop_length,
                )
            elif method == "crepe":
                self.extractors["crepe"] = PitchExtractor(
                    method="crepe",
                    sample_rate=sample_rate,
                    hop_length=hop_length,
                    crepe_model="tiny",
                )
            elif method == "torchcrepe":
                if TORCHCREPE_AVAILABLE:
                    self.extractors["torchcrepe"] = TorchCrepeExtractor(
                        sample_rate=sample_rate,
                        hop_length=hop_length,
                        model="tiny",  # torchcrepe only supports "tiny" or "full"
                    )
            elif method == "rmvpe":
                if TORCH_AVAILABLE:
                    self.extractors["rmvpe"] = RMVPEExtractor(
                        sample_rate=sample_rate,
                        hop_length=hop_length,
                    )

    def extract(
        self,
        audio: np.ndarray,
        sr: Optional[int] = None,
        strategy: str = "confidence_weighted",  # "median", "confidence_weighted", "mean"
    ) -> PitchContour:
        """
        Extract pitch using ensemble of methods.

        Args:
            audio: Audio signal
            sr: Sample rate
            strategy: Combination strategy

        Returns:
            Combined PitchContour
        """
        # Extract from all methods
        contours = {}
        for method, extractor in self.extractors.items():
            try:
                contours[method] = extractor.extract(audio, sr)
            except Exception as e:
                warnings.warn(f"Method {method} failed: {e}")

        if not contours:
            raise RuntimeError("All pitch extraction methods failed")

        # Ensure all contours have same length
        min_len = min(len(c.f0_hz) for c in contours.values())
        for method in contours:
            contours[method].f0_hz = contours[method].f0_hz[:min_len]
            contours[method].confidence = contours[method].confidence[:min_len]
            contours[method].timestamps = contours[method].timestamps[:min_len]

        # Combine using strategy
        if strategy == "median":
            f0_combined = self._median_combine(contours)
            conf_combined = self._median_combine(
                {m: c.confidence for m, c in contours.items()}
            )

        elif strategy == "confidence_weighted":
            f0_combined, conf_combined = self._confidence_weighted_combine(contours)

        elif strategy == "mean":
            f0_combined = self._mean_combine(contours)
            conf_combined = self._mean_combine(
                {m: c.confidence for m, c in contours.items()}
            )

        else:
            raise ValueError(f"Unknown strategy: {strategy}")

        # Use timestamps from first method
        timestamps = next(iter(contours.values())).timestamps

        return PitchContour(
            f0_hz=f0_combined.astype(np.float32),
            confidence=conf_combined.astype(np.float32),
            timestamps=timestamps.astype(np.float32),
            sample_rate=self.sample_rate,
        )

    def _median_combine(self, contours: dict) -> np.ndarray:
        """Combine using median."""
        f0_stack = np.stack([c.f0_hz for c in contours.values()])
        # Replace zeros (unvoiced) with NaN for median calculation
        f0_stack[f0_stack == 0] = np.nan
        return np.nan_to_num(np.nanmedian(f0_stack, axis=0), nan=0.0)

    def _mean_combine(self, contours: dict) -> np.ndarray:
        """Combine using mean."""
        f0_stack = np.stack([c.f0_hz for c in contours.values()])
        f0_stack[f0_stack == 0] = np.nan
        return np.nan_to_num(np.nanmean(f0_stack, axis=0), nan=0.0)

    def _confidence_weighted_combine(
        self,
        contours: dict
    ) -> Tuple[np.ndarray, np.ndarray]:
        """
        Combine using confidence weighting.

        Each method's contribution is weighted by its confidence.
        Combines all predictions, then the combined confidence reflects agreement.
        """
        n_frames = len(next(iter(contours.values())).f0_hz)
        f0_combined = np.zeros(n_frames)
        conf_combined = np.zeros(n_frames)

        for i in range(n_frames):
            weighted_f0 = 0.0
            total_weight = 0.0
            confidence_sum = 0.0

            for method, contour in contours.items():
                conf = contour.confidence[i]
                f0 = contour.f0_hz[i]
                method_weight = self.weights.get(method, 1.0)

                # Include all frames with f0 > 0, even if confidence is low
                # The combined confidence will reflect the overall reliability
                if f0 > 0:
                    weight = conf * method_weight
                    weighted_f0 += f0 * weight
                    total_weight += weight
                    confidence_sum += conf * method_weight

            if total_weight > 0:
                f0_combined[i] = weighted_f0 / total_weight
                # Combined confidence: normalize by total method weights
                total_method_weight = sum(self.weights.get(m, 1.0) for m in contours.keys())
                conf_combined[i] = min(1.0, confidence_sum / total_method_weight)

        return f0_combined, conf_combined


def select_best_pitch_method(
    audio: np.ndarray,
    sample_rate: int = 22050,
    auto_detect: bool = True,
) -> str:
    """
    Auto-select best pitch tracking method for the audio.

    Heuristics:
    - Clean audio + GPU available → torchcrepe
    - Noisy audio → ensemble
    - CPU only → yin
    - Default → yin (most compatible)

    Args:
        audio: Audio signal
        sample_rate: Sample rate
        auto_detect: Enable auto-detection

    Returns:
        Method name: "yin", "crepe", "torchcrepe", "rmvpe", or "ensemble"
    """
    if not auto_detect:
        return "yin"

    # Check GPU availability
    has_gpu = TORCH_AVAILABLE and torch.cuda.is_available()

    # Estimate SNR (very rough)
    rms = np.sqrt(np.mean(audio ** 2))
    noise_floor = np.sqrt(np.mean(audio[:1000] ** 2))  # First 1000 samples
    snr_rough = 20 * np.log10(rms / (noise_floor + 1e-8))

    # Decision logic
    if has_gpu and TORCHCREPE_AVAILABLE:
        if snr_rough > 20:  # Clean audio
            return "torchcrepe"
        else:  # Noisy audio
            return "ensemble"
    elif TORCHCREPE_AVAILABLE and snr_rough > 30:
        # Very clean audio, torchcrepe even on CPU
        return "torchcrepe"
    else:
        # Fall back to YIN (fast, reliable)
        return "yin"
