"""
Enhanced Ghunnah Validator - M4 Tier 2 Phase 2

Implements formant analysis for Ghunnah (nasalization) detection.

Approach:
- Use Tier 1 baseline (Muaalem sifat) as initial detection
- For low-confidence cases (prob < 0.8), perform formant analysis
- Extract F1, F2, F3 and nasal energy (250-350Hz band)
- Combine baseline + formant scores with weighted average

Accuracy Target: 90%+ (enhanced from Tier 1's 70-85%)

References:
- doc/01-architecture/m4-tajweed.md Section 3.2
- Praat formant analysis for Arabic nasals
"""

from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass
import numpy as np

from iqrah.tajweed.baseline_interpreter import TajweedViolation


@dataclass
class GhunnahFormantFeatures:
    """
    Formant features for Ghunnah analysis.

    Attributes:
        f1_hz: First formant frequency (Hz)
        f2_hz: Second formant frequency (Hz)
        f3_hz: Third formant frequency (Hz)
        nasal_energy_db: Energy in nasal band (250-350Hz)
        formant_score: Overall formant score (0-1)
    """
    f1_hz: float
    f2_hz: float
    f3_hz: float
    nasal_energy_db: float
    formant_score: float


class GhunnahValidator:
    """
    Enhanced Ghunnah validator using formant analysis.

    Combines Tier 1 baseline (Muaalem sifat) with acoustic formant analysis
    for improved accuracy on low-confidence detections.

    Design:
    - High-confidence Tier 1 (>0.8): Trust baseline, skip formants
    - Low-confidence Tier 1 (<0.8): Enhance with formant analysis
    - Weighted combination: (1-w) × baseline + w × formant_score
    """

    # Nasal phonemes that should have Ghunnah
    NASAL_PHONEMES = {'n', 'm', 'ن', 'م', 'ں', 'N'}

    # Formant thresholds for Ghunnah detection
    F1_THRESHOLD_HZ = 500.0      # Low F1 indicates nasalization
    F2_MIN_HZ = 1000.0           # F2-F1 coupling range
    F2_MAX_HZ = 1800.0
    NASAL_ENERGY_THRESHOLD_DB = -20.0  # Elevated nasal band energy

    def __init__(
        self,
        use_formants: bool = True,
        formant_weight: float = 0.3,
        confidence_threshold: float = 0.7,
        tier1_confidence_threshold: float = 0.8
    ):
        """
        Initialize enhanced Ghunnah validator.

        Args:
            use_formants: Enable formant analysis (Phase 2 feature)
            formant_weight: Weight for formant score (0-1, default 0.3)
            confidence_threshold: Minimum confidence for violations (default 0.7)
            tier1_confidence_threshold: Tier 1 threshold for formant analysis (default 0.8)
        """
        self.use_formants = use_formants
        self.formant_weight = formant_weight
        self.confidence_threshold = confidence_threshold
        self.tier1_confidence_threshold = tier1_confidence_threshold

        # Check for parselmouth availability
        self.parselmouth_available = False
        if self.use_formants:
            try:
                import parselmouth
                self.parselmouth_available = True
            except ImportError:
                print("Warning: parselmouth not available. Install with: pip install praat-parselmouth")
                print("Falling back to Tier 1 baseline only.")
                self.use_formants = False

    def validate(
        self,
        aligned_phonemes: List,
        audio: Optional[np.ndarray] = None,
        sample_rate: int = 16000
    ) -> List[TajweedViolation]:
        """
        Validate Ghunnah with enhanced formant analysis.

        Args:
            aligned_phonemes: Aligned phonemes from M3 with sifat
            audio: Audio array (required for formant analysis)
            sample_rate: Sample rate (default 16000)

        Returns:
            List of TajweedViolation objects for Ghunnah issues
        """
        violations = []

        for idx, phoneme in enumerate(aligned_phonemes):
            # Check if this is a nasal phoneme
            if not self._is_nasal_phoneme(phoneme):
                continue

            # Get Tier 1 baseline confidence
            baseline_prob = self._get_baseline_confidence(phoneme)

            # If high confidence, trust Tier 1
            if baseline_prob > self.tier1_confidence_threshold:
                continue

            # Low confidence: enhance with formants
            combined_confidence = baseline_prob

            if self.use_formants and audio is not None:
                try:
                    # Extract formant features
                    formant_features = self._extract_formant_features(
                        audio,
                        phoneme.start,
                        phoneme.end,
                        sample_rate
                    )

                    # Combine baseline + formant scores
                    combined_confidence = (
                        (1 - self.formant_weight) * baseline_prob +
                        self.formant_weight * formant_features.formant_score
                    )

                except Exception as e:
                    # Formant extraction failed, use baseline only
                    print(f"Warning: Formant extraction failed for phoneme {idx}: {e}")
                    combined_confidence = baseline_prob

            # Check for violation
            if combined_confidence < self.confidence_threshold:
                severity = self._compute_severity(combined_confidence)

                violations.append(TajweedViolation(
                    rule="Ghunnah",
                    phoneme_idx=idx,
                    phoneme=phoneme.phoneme,
                    timestamp=phoneme.start,
                    expected="Nasal resonance (ghunnah)",
                    actual=f"Confidence: {combined_confidence:.2f}",
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

    def _is_nasal_phoneme(self, phoneme) -> bool:
        """Check if phoneme is a nasal that should have Ghunnah."""
        return phoneme.phoneme in self.NASAL_PHONEMES

    def _get_baseline_confidence(self, phoneme) -> float:
        """Get Tier 1 baseline confidence from Muaalem sifat."""
        if not hasattr(phoneme, 'sifa') or phoneme.sifa is None:
            return 0.5  # Neutral if no sifat

        # Sifat can be dict or object
        if isinstance(phoneme.sifa, dict):
            ghonna = phoneme.sifa.get('ghonna')
            if ghonna is not None:
                if isinstance(ghonna, dict) and 'prob' in ghonna:
                    return float(ghonna['prob'])
        else:
            # Object with attributes
            if hasattr(phoneme.sifa, 'ghonna'):
                ghonna = phoneme.sifa.ghonna
                if ghonna is not None:
                    if isinstance(ghonna, dict) and 'prob' in ghonna:
                        return float(ghonna['prob'])
                    elif hasattr(ghonna, 'prob'):
                        return float(ghonna.prob)

        return 0.5  # Neutral if no ghonna info

    def _extract_formant_features(
        self,
        audio: np.ndarray,
        start: float,
        end: float,
        sample_rate: int
    ) -> GhunnahFormantFeatures:
        """
        Extract formant features for Ghunnah detection.

        Args:
            audio: Audio array
            start, end: Time boundaries (seconds)
            sample_rate: Sample rate

        Returns:
            GhunnahFormantFeatures with F1, F2, F3, nasal energy, and score
        """
        import parselmouth
        from scipy.signal import butter, filtfilt

        # Extract segment
        start_sample = int(start * sample_rate)
        end_sample = int(end * sample_rate)
        segment = audio[start_sample:end_sample]

        if len(segment) < 100:  # Too short
            return GhunnahFormantFeatures(0, 0, 0, -50.0, 0.5)

        # Create Praat Sound object
        try:
            sound = parselmouth.Sound(segment, sampling_frequency=sample_rate)
        except Exception as e:
            print(f"Warning: Failed to create Praat Sound: {e}")
            return GhunnahFormantFeatures(0, 0, 0, -50.0, 0.5)

        # Extract formants using Praat's Burg algorithm
        try:
            formants = sound.to_formant_burg(
                max_number_of_formants=3,
                maximum_formant=5500.0
            )

            # Get formant values at midpoint
            midpoint = (end - start) / 2.0
            f1_hz = formants.get_value_at_time(1, midpoint)
            f2_hz = formants.get_value_at_time(2, midpoint)
            f3_hz = formants.get_value_at_time(3, midpoint)

            # Handle NaN values
            f1_hz = float(f1_hz) if not np.isnan(f1_hz) else 0.0
            f2_hz = float(f2_hz) if not np.isnan(f2_hz) else 0.0
            f3_hz = float(f3_hz) if not np.isnan(f3_hz) else 0.0

        except Exception as e:
            print(f"Warning: Formant extraction failed: {e}")
            f1_hz, f2_hz, f3_hz = 0.0, 0.0, 0.0

        # Extract nasal energy (250-350Hz band)
        try:
            b, a = butter(4, [250, 350], btype='band', fs=sample_rate)
            nasal_band = filtfilt(b, a, segment)
            nasal_energy_db = 10 * np.log10(np.mean(nasal_band**2) + 1e-10)
        except Exception as e:
            print(f"Warning: Nasal energy extraction failed: {e}")
            nasal_energy_db = -50.0

        # Compute formant score
        formant_score = self._score_formants(f1_hz, f2_hz, f3_hz, nasal_energy_db)

        return GhunnahFormantFeatures(
            f1_hz=f1_hz,
            f2_hz=f2_hz,
            f3_hz=f3_hz,
            nasal_energy_db=nasal_energy_db,
            formant_score=formant_score
        )

    def _score_formants(
        self,
        f1_hz: float,
        f2_hz: float,
        f3_hz: float,
        nasal_energy_db: float
    ) -> float:
        """
        Score formant features for Ghunnah presence (0-1).

        Criteria:
        - Low F1 (<500Hz) indicates nasalization
        - F2-F1 coupling (F2 in 1000-1800Hz range)
        - Elevated nasal energy (>-20dB)

        Args:
            f1_hz, f2_hz, f3_hz: Formant frequencies
            nasal_energy_db: Nasal band energy

        Returns:
            Score from 0 (no Ghunnah) to 1 (strong Ghunnah)
        """
        score = 0.5  # Neutral baseline

        # Low F1 indicates nasalization
        if f1_hz > 0 and f1_hz < self.F1_THRESHOLD_HZ:
            score += 0.2

        # F2-F1 coupling (F2 pulled down toward F1)
        if f2_hz > 0 and self.F2_MIN_HZ < f2_hz < self.F2_MAX_HZ:
            score += 0.1

        # Elevated nasal energy
        if nasal_energy_db > self.NASAL_ENERGY_THRESHOLD_DB:
            score += 0.2

        # F3 presence (optional)
        if f3_hz > 2000:
            score += 0.05

        return min(score, 1.0)

    def _compute_severity(self, confidence: float) -> str:
        """Determine severity based on confidence."""
        if confidence < 0.3:
            return "critical"
        elif confidence < 0.6:
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
            f"{severity_text} Ghunnah on '{phoneme}'. "
            f"Nasal resonance too low (confidence: {confidence:.1%}). "
            f"Hum through the nose with mouth closed."
        )
