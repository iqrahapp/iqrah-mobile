"""
Qalqalah Validator - M4 Tier 2 Phase 2

Implements acoustic burst detection for Qalqalah (echo/bounce).

Qalqalah letters: ق، ط، ب، ج، د (with sukoon or waqf)

Approach:
- Use Tier 1 baseline (Muaalem sifat) as initial detection
- For low-confidence cases (prob < 0.8), perform burst detection
- Extract acoustic features: ZCR, spectral centroid, RMS envelope
- Detect sharp transients and energy spikes

Accuracy Target: 85%+ (enhanced from Tier 1's 75-80%)

References:
- doc/01-architecture/m4-tajweed.md Section 3.3
- Arabic phonetics: stop consonant bursts
"""

from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass
import numpy as np

from iqrah.tajweed.baseline_interpreter import TajweedViolation


@dataclass
class QalqalahBurstFeatures:
    """
    Burst features for Qalqalah analysis.

    Attributes:
        zcr_mean: Mean zero-crossing rate
        zcr_std: Std of zero-crossing rate
        centroid_mean: Mean spectral centroid (Hz)
        rms_max: Maximum RMS energy
        rms_std: Std of RMS envelope
        has_burst: Boolean burst detection
        burst_score: Overall burst score (0-1)
    """
    zcr_mean: float
    zcr_std: float
    centroid_mean: float
    rms_max: float
    rms_std: float
    has_burst: bool
    burst_score: float


class QalqalahValidator:
    """
    Enhanced Qalqalah validator using acoustic burst detection.

    Combines Tier 1 baseline (Muaalem sifat) with acoustic analysis
    for improved accuracy on low-confidence detections.

    Design:
    - High-confidence Tier 1 (>0.8): Trust baseline, skip burst detection
    - Low-confidence Tier 1 (<0.8): Enhance with acoustic analysis
    - Weighted combination: 0.6 × baseline + 0.4 × burst_score
    """

    # Qalqalah letters (Buckwalter notation + Arabic)
    QALQALAH_LETTERS = {'q', 'T', 'b', 'j', 'd', 'ق', 'ط', 'ب', 'ج', 'د'}

    # Burst detection thresholds
    ZCR_THRESHOLD = 0.3           # High ZCR indicates burst
    CENTROID_MIN_HZ = 1500.0      # Brightness from burst
    RMS_BURST_RATIO = 1.5         # RMS spike ratio (max / median)

    def __init__(
        self,
        use_burst_detection: bool = True,
        burst_weight: float = 0.4,
        confidence_threshold: float = 0.6,
        tier1_confidence_threshold: float = 0.8
    ):
        """
        Initialize enhanced Qalqalah validator.

        Args:
            use_burst_detection: Enable burst analysis (Phase 2 feature)
            burst_weight: Weight for burst score (0-1, default 0.4)
            confidence_threshold: Minimum confidence for violations (default 0.6)
            tier1_confidence_threshold: Tier 1 threshold for burst analysis (default 0.8)
        """
        self.use_burst_detection = use_burst_detection
        self.burst_weight = burst_weight
        self.confidence_threshold = confidence_threshold
        self.tier1_confidence_threshold = tier1_confidence_threshold

        # Check for librosa availability
        self.librosa_available = False
        if self.use_burst_detection:
            try:
                import librosa
                self.librosa_available = True
            except ImportError:
                print("Warning: librosa not available. Install with: pip install librosa")
                print("Falling back to Tier 1 baseline only.")
                self.use_burst_detection = False

    def validate(
        self,
        aligned_phonemes: List,
        audio: Optional[np.ndarray] = None,
        sample_rate: int = 16000
    ) -> List[TajweedViolation]:
        """
        Validate Qalqalah with enhanced burst detection.

        Args:
            aligned_phonemes: Aligned phonemes from M3 with sifat
            audio: Audio array (required for burst detection)
            sample_rate: Sample rate (default 16000)

        Returns:
            List of TajweedViolation objects for Qalqalah issues
        """
        violations = []

        for idx, phoneme in enumerate(aligned_phonemes):
            # Check if this is a Qalqalah letter
            if not self._is_qalqalah_letter(phoneme):
                continue

            # Get Tier 1 baseline confidence
            baseline_prob = self._get_baseline_confidence(phoneme)

            # If high confidence, trust Tier 1
            if baseline_prob > self.tier1_confidence_threshold:
                continue

            # Low confidence: enhance with burst detection
            combined_confidence = baseline_prob

            if self.use_burst_detection and audio is not None:
                try:
                    # Extract burst features
                    burst_features = self._extract_burst_features(
                        audio,
                        phoneme.start,
                        phoneme.end,
                        sample_rate
                    )

                    # Combine baseline + burst scores
                    combined_confidence = (
                        (1 - self.burst_weight) * baseline_prob +
                        self.burst_weight * burst_features.burst_score
                    )

                except Exception as e:
                    # Burst detection failed, use baseline only
                    print(f"Warning: Burst detection failed for phoneme {idx}: {e}")
                    combined_confidence = baseline_prob

            # Check for violation
            if combined_confidence < self.confidence_threshold:
                severity = self._compute_severity(combined_confidence)

                violations.append(TajweedViolation(
                    rule="Qalqalah",
                    phoneme_idx=idx,
                    phoneme=phoneme.phoneme,
                    timestamp=phoneme.start,
                    expected="Sharp burst with echo",
                    actual=f"Burst confidence: {combined_confidence:.2f}",
                    confidence=combined_confidence,
                    severity=severity,
                    tier=2,  # Tier 2 enhanced
                    feedback=self._generate_feedback(
                        phoneme.phoneme,
                        combined_confidence,
                        severity
                    )
                ))

        return violations

    def _is_qalqalah_letter(self, phoneme) -> bool:
        """Check if phoneme is a Qalqalah letter."""
        return phoneme.phoneme in self.QALQALAH_LETTERS

    def _get_baseline_confidence(self, phoneme) -> float:
        """Get Tier 1 baseline confidence from Muaalem sifat."""
        if not hasattr(phoneme, 'sifa') or phoneme.sifa is None:
            return 0.5  # Neutral if no sifat

        # Sifat can be dict or object
        if isinstance(phoneme.sifa, dict):
            qalqla = phoneme.sifa.get('qalqla')
            if qalqla is not None and isinstance(qalqla, dict) and 'prob' in qalqla:
                return float(qalqla['prob'])
        else:
            # Object with attributes
            if hasattr(phoneme.sifa, 'qalqla'):
                qalqla = phoneme.sifa.qalqla
                if qalqla is not None:
                    if isinstance(qalqla, dict) and 'prob' in qalqla:
                        return float(qalqla['prob'])
                    elif hasattr(qalqla, 'prob'):
                        return float(qalqla.prob)

        return 0.5  # Neutral if no qalqla info

    def _extract_burst_features(
        self,
        audio: np.ndarray,
        start: float,
        end: float,
        sample_rate: int
    ) -> QalqalahBurstFeatures:
        """
        Extract burst features for Qalqalah detection.

        Args:
            audio: Audio array
            start, end: Time boundaries (seconds)
            sample_rate: Sample rate

        Returns:
            QalqalahBurstFeatures with acoustic features and score
        """
        import librosa

        # Extract segment
        start_sample = int(start * sample_rate)
        end_sample = int(end * sample_rate)
        segment = audio[start_sample:end_sample]

        if len(segment) < 100:  # Too short
            return QalqalahBurstFeatures(0, 0, 0, 0, 0, False, 0.5)

        # Zero-crossing rate (high during burst)
        try:
            zcr = librosa.feature.zero_crossing_rate(segment)[0]
            zcr_mean = float(np.mean(zcr))
            zcr_std = float(np.std(zcr))
        except Exception as e:
            print(f"Warning: ZCR extraction failed: {e}")
            zcr_mean, zcr_std = 0.0, 0.0

        # Spectral centroid (brightness from burst)
        try:
            centroid = librosa.feature.spectral_centroid(y=segment, sr=sample_rate)[0]
            centroid_mean = float(np.mean(centroid))
        except Exception as e:
            print(f"Warning: Spectral centroid extraction failed: {e}")
            centroid_mean = 0.0

        # RMS energy envelope (spike detection)
        try:
            rms = librosa.feature.rms(y=segment)[0]
            rms_max = float(np.max(rms))
            rms_std = float(np.std(rms))
            rms_median = float(np.median(rms))

            # Burst detection: max RMS > 1.5× median
            has_burst = rms_max > self.RMS_BURST_RATIO * rms_median if rms_median > 0 else False

        except Exception as e:
            print(f"Warning: RMS extraction failed: {e}")
            rms_max, rms_std, has_burst = 0.0, 0.0, False

        # Compute burst score
        burst_score = self._score_burst(
            zcr_mean, zcr_std, centroid_mean, rms_max, rms_std, has_burst
        )

        return QalqalahBurstFeatures(
            zcr_mean=zcr_mean,
            zcr_std=zcr_std,
            centroid_mean=centroid_mean,
            rms_max=rms_max,
            rms_std=rms_std,
            has_burst=has_burst,
            burst_score=burst_score
        )

    def _score_burst(
        self,
        zcr_mean: float,
        zcr_std: float,
        centroid_mean: float,
        rms_max: float,
        rms_std: float,
        has_burst: bool
    ) -> float:
        """
        Score burst features for Qalqalah presence (0-1).

        Criteria:
        - High ZCR (>0.3) indicates sharp transient
        - High spectral centroid (>1500Hz) indicates brightness
        - Energy spike detected (has_burst)

        Args:
            zcr_mean, zcr_std: Zero-crossing rate statistics
            centroid_mean: Spectral centroid
            rms_max, rms_std: RMS envelope statistics
            has_burst: Boolean burst detection

        Returns:
            Score from 0 (no Qalqalah) to 1 (strong Qalqalah)
        """
        score = 0.3  # Low baseline (Qalqalah is subtle)

        # High ZCR indicates sharp transient
        if zcr_mean > self.ZCR_THRESHOLD:
            score += 0.3

        # High spectral centroid indicates brightness from burst
        if centroid_mean > self.CENTROID_MIN_HZ:
            score += 0.2

        # Energy spike detected
        if has_burst:
            score += 0.2

        # High RMS variability
        if rms_std > 0.01:
            score += 0.05

        return min(score, 1.0)

    def _compute_severity(self, confidence: float) -> str:
        """Determine severity based on confidence."""
        if confidence < 0.3:
            return "critical"
        elif confidence < 0.5:
            return "moderate"
        else:
            return "minor"

    def _generate_feedback(
        self,
        phoneme: str,
        confidence: float,
        severity: str
    ) -> str:
        """Generate user-friendly feedback message."""
        severity_text = {
            "critical": "Very weak",
            "moderate": "Weak",
            "minor": "Slightly weak"
        }.get(severity, "Weak")

        return (
            f"{severity_text} Qalqalah on '{phoneme}'. "
            f"Burst confidence: {confidence:.1%}. "
            f"Qalqalah requires short, explosive release with echo."
        )
