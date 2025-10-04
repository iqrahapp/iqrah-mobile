"""
SOTA Pitch Extraction Pipeline
================================

Smart pitch extraction that automatically selects the best method
and applies post-processing for maximum accuracy.
"""

import numpy as np
from typing import Optional, Union
import warnings

from .pitch import PitchExtractor, PitchContour
from .octave import OctaveCorrector, calculate_octave_confidence

try:
    from .pitch_rmvpe import (
        TorchCrepeExtractor,
        RMVPEExtractor,
        EnsemblePitchExtractor,
        select_best_pitch_method,
    )
    ADVANCED_METHODS_AVAILABLE = True
except ImportError:
    ADVANCED_METHODS_AVAILABLE = False


class SmartPitchExtractor:
    """
    Smart pitch extractor with automatic method selection and post-processing.

    Features:
    - Auto-selects best method (YIN/CREPE/TorchCrepe/RMVPE/Ensemble)
    - Applies octave correction
    - Confidence calibration
    - Outlier removal
    """

    def __init__(
        self,
        sample_rate: int = 22050,
        hop_length: int = 512,
        method: str = "auto",  # "auto", "yin", "crepe", "torchcrepe", "rmvpe", "ensemble"
        fmin: float = 50.0,  # Quranic recitation range
        fmax: float = 1000.0,
        octave_correction: str = "hybrid",  # "none", "median", "hybrid"
        confidence_threshold: float = 0.3,  # Lower = more permissive
    ):
        """
        Initialize smart pitch extractor.

        Args:
            sample_rate: Audio sample rate
            hop_length: Hop length in samples
            method: Extraction method or "auto" for automatic selection
            fmin: Minimum frequency (Hz)
            fmax: Maximum frequency (Hz)
            octave_correction: Octave correction strategy
            confidence_threshold: Minimum confidence for voiced frames
        """
        self.sample_rate = sample_rate
        self.hop_length = hop_length
        self.method = method
        self.fmin = fmin
        self.fmax = fmax
        self.octave_correction = octave_correction
        self.confidence_threshold = confidence_threshold

        # Initialize octave corrector
        if octave_correction != "none":
            self.corrector = OctaveCorrector(
                strategy=octave_correction,
                confidence_threshold=confidence_threshold,
            )
        else:
            self.corrector = None

    def extract(
        self,
        audio: np.ndarray,
        sr: Optional[int] = None,
        reference_f0: Optional[np.ndarray] = None,
        chroma: Optional[np.ndarray] = None,
    ) -> PitchContour:
        """
        Extract pitch with smart method selection and post-processing.

        Args:
            audio: Audio signal
            sr: Sample rate (will resample if different)
            reference_f0: Reference F0 for octave correction (optional)
            chroma: Chroma features for octave correction (optional)

        Returns:
            Processed PitchContour
        """
        # Select method
        if self.method == "auto":
            method = select_best_pitch_method(audio, self.sample_rate)
            print(f"Auto-selected pitch method: {method}")
        else:
            method = self.method

        # Extract raw pitch
        contour = self._extract_raw(audio, sr, method)

        # Post-processing
        contour = self._post_process(
            contour,
            audio,
            reference_f0=reference_f0,
            chroma=chroma,
        )

        return contour

    def _extract_raw(
        self,
        audio: np.ndarray,
        sr: Optional[int],
        method: str,
    ) -> PitchContour:
        """Extract raw pitch using selected method."""
        if method == "yin" or method == "crepe":
            extractor = PitchExtractor(
                method=method,
                sample_rate=self.sample_rate,
                hop_length=self.hop_length,
            )
            return extractor.extract(audio, sr)

        elif method == "torchcrepe" and ADVANCED_METHODS_AVAILABLE:
            extractor = TorchCrepeExtractor(
                sample_rate=self.sample_rate,
                hop_length=self.hop_length,
                model="tiny",  # torchcrepe only supports "tiny" or "full"
                fmin=self.fmin,
                fmax=self.fmax,
            )
            return extractor.extract(audio, sr)

        elif method == "rmvpe" and ADVANCED_METHODS_AVAILABLE:
            extractor = RMVPEExtractor(
                sample_rate=self.sample_rate,
                hop_length=self.hop_length,
            )
            return extractor.extract(audio, sr)

        elif method == "ensemble" and ADVANCED_METHODS_AVAILABLE:
            extractor = EnsemblePitchExtractor(
                sample_rate=self.sample_rate,
                hop_length=self.hop_length,
                methods=["yin", "torchcrepe"],
                weights={"yin": 0.4, "torchcrepe": 0.6},
            )
            return extractor.extract(audio, sr, strategy="confidence_weighted")

        else:
            # Fallback to YIN
            warnings.warn(f"Method {method} not available, using YIN")
            extractor = PitchExtractor(method="yin", sample_rate=self.sample_rate)
            return extractor.extract(audio, sr)

    def _post_process(
        self,
        contour: PitchContour,
        audio: np.ndarray,
        reference_f0: Optional[np.ndarray] = None,
        chroma: Optional[np.ndarray] = None,
    ) -> PitchContour:
        """
        Post-process pitch contour.

        Steps:
        1. Remove outliers
        2. Octave correction
        3. Median filtering
        4. Confidence calibration
        """
        from scipy.signal import medfilt

        # 1. Remove gross outliers (>3 octaves from median)
        voiced_mask = contour.confidence > self.confidence_threshold
        if np.sum(voiced_mask) > 0:
            median_f0 = np.median(contour.f0_hz[voiced_mask])

            for i in range(len(contour.f0_hz)):
                if voiced_mask[i]:
                    # Check if more than 3 octaves away
                    ratio = contour.f0_hz[i] / median_f0
                    if ratio < 1/8 or ratio > 8:  # 3 octaves = 2^3 = 8
                        contour.f0_hz[i] = 0
                        contour.confidence[i] = 0

        # 2. Apply octave correction
        if self.corrector is not None:
            contour.f0_hz = self.corrector.correct(
                contour.f0_hz,
                contour.confidence,
                reference_f0_hz=reference_f0,
                chroma=chroma,
            )

        # 3. Median filtering on voiced regions
        voiced_mask = contour.confidence > self.confidence_threshold
        if np.sum(voiced_mask) > 5:
            f0_filtered = contour.f0_hz.copy()
            f0_filtered[voiced_mask] = medfilt(
                contour.f0_hz[voiced_mask],
                kernel_size=5
            )
            contour.f0_hz = f0_filtered

        # 4. Calibrate confidence (adjust based on octave confidence if chroma available)
        if chroma is not None:
            octave_conf = calculate_octave_confidence(
                contour.f0_hz,
                contour.confidence,
                chroma,
            )
            # Multiply confidences (both must be high)
            contour.confidence = contour.confidence * octave_conf

        return contour

    def extract_with_features(
        self,
        audio: np.ndarray,
        sr: Optional[int] = None,
    ) -> tuple[PitchContour, dict]:
        """
        Extract pitch and additional features for analysis.

        Returns:
            (contour, features_dict)
        """
        from .features import FeatureExtractor

        # Extract pitch
        contour = self.extract(audio, sr)

        # Extract features for post-processing
        feat_extractor = FeatureExtractor(
            sample_rate=self.sample_rate,
            hop_length=self.hop_length,
            extract_chroma=True,
            extract_energy=True,
        )

        # Re-extract with chroma for octave correction
        features = feat_extractor.extract_all(audio, contour)

        # Re-process with chroma
        contour_improved = self._post_process(
            contour,
            audio,
            chroma=features.chroma,
        )

        features_dict = {
            "chroma": features.chroma,
            "rms": features.rms,
            "mel_spec": features.mel_spec,
        }

        return contour_improved, features_dict


class AdaptivePitchExtractor:
    """
    Adaptive pitch extractor that learns from alignment.

    Uses feedback from DTW alignment to improve pitch tracking:
    - Detects systematic octave errors
    - Adjusts confidence thresholds
    - Tunes post-processing parameters
    """

    def __init__(
        self,
        sample_rate: int = 22050,
        hop_length: int = 512,
    ):
        """Initialize adaptive extractor."""
        self.sample_rate = sample_rate
        self.hop_length = hop_length

        # Smart extractor
        self.extractor = SmartPitchExtractor(
            sample_rate=sample_rate,
            hop_length=hop_length,
            method="auto",
        )

        # Adaptation state
        self.octave_bias = 0  # Learned octave bias
        self.confidence_adjustment = 1.0

    def extract_and_adapt(
        self,
        audio: np.ndarray,
        sr: Optional[int] = None,
        reference_contour: Optional[PitchContour] = None,
    ) -> PitchContour:
        """
        Extract pitch and adapt based on reference.

        Args:
            audio: Audio signal
            sr: Sample rate
            reference_contour: Reference pitch for adaptation

        Returns:
            Adapted pitch contour
        """
        # Extract raw
        contour = self.extractor.extract(audio, sr)

        # Adapt if reference is available
        if reference_contour is not None:
            self._adapt_from_reference(contour, reference_contour)

            # Re-extract with learned bias
            if abs(self.octave_bias) > 0:
                contour.f0_hz *= (2 ** self.octave_bias)

        return contour

    def _adapt_from_reference(
        self,
        user_contour: PitchContour,
        ref_contour: PitchContour,
    ):
        """Learn from reference alignment."""
        from .dtw import DTWAligner
        from .octave import detect_octave_errors

        # Align
        aligner = DTWAligner(window=50)
        alignment = aligner.align(
            user_contour.f0_cents,
            ref_contour.f0_cents,
        )

        # Check for systematic octave errors
        if alignment.path:
            errors_cents = []
            for u_idx, r_idx in alignment.path:
                if (user_contour.confidence[u_idx] > 0.5 and
                    ref_contour.confidence[r_idx] > 0.5):

                    error = (user_contour.f0_cents[u_idx] -
                            ref_contour.f0_cents[r_idx])
                    errors_cents.append(error)

            if errors_cents:
                median_error = np.median(errors_cents)

                # Check if systematic octave error (close to ±1200 cents)
                if 1000 < median_error < 1400:
                    self.octave_bias = -1  # Too high
                elif -1400 < median_error < -1000:
                    self.octave_bias = 1  # Too low

        # Detect octave error rate
        octave_errors = detect_octave_errors(
            user_contour.f0_cents,
            ref_contour.f0_cents,
        )

        error_rate = np.mean(octave_errors)

        # Adjust confidence threshold
        if error_rate > 0.1:  # >10% errors
            self.confidence_adjustment *= 0.9  # Be more conservative
        elif error_rate < 0.01:  # <1% errors
            self.confidence_adjustment *= 1.05  # Can be more permissive


def compare_pitch_methods(
    audio: np.ndarray,
    sample_rate: int = 22050,
    methods: list[str] = ["yin", "torchcrepe", "ensemble"],
    ground_truth_f0: Optional[np.ndarray] = None,
) -> dict:
    """
    Compare different pitch tracking methods.

    Args:
        audio: Audio signal
        sample_rate: Sample rate
        methods: Methods to compare
        ground_truth_f0: Ground truth F0 (optional, for accuracy calculation)

    Returns:
        Comparison results dict
    """
    import time

    results = {}

    for method in methods:
        extractor = SmartPitchExtractor(
            sample_rate=sample_rate,
            method=method,
        )

        # Time the extraction
        start = time.time()
        try:
            contour = extractor.extract(audio)
            elapsed = time.time() - start

            results[method] = {
                "success": True,
                "time_ms": elapsed * 1000,
                "rtf": elapsed / (len(audio) / sample_rate),
                "median_f0": np.median(contour.f0_hz[contour.confidence > 0.5])
                    if np.sum(contour.confidence > 0.5) > 0 else 0,
                "voiced_ratio": np.mean(contour.confidence > 0.5),
                "contour": contour,
            }

            # Calculate accuracy if ground truth available
            if ground_truth_f0 is not None:
                # Interpolate ground truth to match contour length
                gt_interp = np.interp(
                    contour.timestamps,
                    np.linspace(0, len(audio) / sample_rate, len(ground_truth_f0)),
                    ground_truth_f0,
                )

                # Calculate error (voiced frames only)
                voiced_mask = contour.confidence > 0.5
                if np.sum(voiced_mask) > 0:
                    errors_cents = np.abs(
                        1200 * np.log2(contour.f0_hz[voiced_mask] / gt_interp[voiced_mask])
                    )
                    results[method]["mae_cents"] = np.mean(errors_cents)
                    results[method]["median_error_cents"] = np.median(errors_cents)

        except Exception as e:
            results[method] = {
                "success": False,
                "error": str(e),
            }

    return results
