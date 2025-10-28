"""
Tests for M4 Tier 2 Phase 2 Validators

Tests cover:
- Ghunnah formant analysis
- Qalqalah burst detection
- Tier 1 + Tier 2 integration
- Graceful fallback when libraries unavailable
"""

import pytest
import numpy as np
from dataclasses import dataclass

from iqrah.tajweed import GhunnahValidator, QalqalahValidator


# Mock AlignedPhoneme for testing
@dataclass
class MockPhoneme:
    phoneme: str
    start: float
    end: float
    sifa: dict = None


class TestGhunnahValidator:
    """Test Ghunnah validator with formant analysis."""

    def test_initialization_with_formants(self):
        """Test validator initializes with formant analysis enabled."""
        validator = GhunnahValidator(
            use_formants=True,
            formant_weight=0.3
        )

        # use_formants will be False if parselmouth is unavailable (graceful fallback)
        try:
            import parselmouth
            assert validator.use_formants == True
        except ImportError:
            assert validator.use_formants == False  # Graceful fallback

        assert validator.formant_weight == 0.3
        assert validator.confidence_threshold == 0.7

    def test_initialization_without_formants(self):
        """Test validator initializes without formant analysis."""
        validator = GhunnahValidator(use_formants=False)

        assert validator.use_formants == False

    def test_nasal_phoneme_detection(self):
        """Test detection of nasal phonemes."""
        validator = GhunnahValidator()

        # Nasal phonemes
        assert validator._is_nasal_phoneme(MockPhoneme('n', 0, 0.1))
        assert validator._is_nasal_phoneme(MockPhoneme('m', 0, 0.1))
        assert validator._is_nasal_phoneme(MockPhoneme('ن', 0, 0.1))
        assert validator._is_nasal_phoneme(MockPhoneme('م', 0, 0.1))

        # Non-nasal phonemes
        assert not validator._is_nasal_phoneme(MockPhoneme('b', 0, 0.1))
        assert not validator._is_nasal_phoneme(MockPhoneme('a', 0, 0.1))

    def test_baseline_confidence_extraction(self):
        """Test extraction of baseline confidence from sifat."""
        validator = GhunnahValidator()

        # High confidence sifat
        phoneme_high = MockPhoneme(
            'n', 0, 0.1,
            sifa={'ghonna': {'text': 'maghnoon', 'prob': 0.9}}
        )
        assert validator._get_baseline_confidence(phoneme_high) == 0.9

        # Low confidence sifat
        phoneme_low = MockPhoneme(
            'n', 0, 0.1,
            sifa={'ghonna': {'text': 'maghnoon', 'prob': 0.6}}
        )
        assert validator._get_baseline_confidence(phoneme_low) == 0.6

        # No sifat
        phoneme_none = MockPhoneme('n', 0, 0.1, sifa=None)
        assert validator._get_baseline_confidence(phoneme_none) == 0.5

    def test_validate_high_confidence_skips_formants(self):
        """Test that high confidence Tier 1 skips formant analysis."""
        validator = GhunnahValidator(use_formants=True)

        # High confidence nasal (>0.8), should be skipped
        phonemes = [
            MockPhoneme(
                'n', 0, 0.1,
                sifa={'ghonna': {'text': 'maghnoon', 'prob': 0.95}}
            )
        ]

        # No audio needed, should not raise error
        violations = validator.validate(phonemes, audio=None)

        # Should have no violations (high confidence)
        assert len(violations) == 0

    def test_validate_low_confidence_without_audio(self):
        """Test low confidence without audio falls back to baseline."""
        validator = GhunnahValidator(use_formants=True)

        # Low confidence nasal (<0.8)
        phonemes = [
            MockPhoneme(
                'n', 0, 0.1,
                sifa={'ghonna': {'text': 'maghnoon', 'prob': 0.6}}
            )
        ]

        # No audio, should use baseline only (below 0.7 threshold)
        violations = validator.validate(phonemes, audio=None)

        # Should have violation (low confidence, no formant enhancement)
        assert len(violations) == 1
        assert violations[0].rule == "Ghunnah"
        assert violations[0].tier == 2

    def test_formant_score_computation(self):
        """Test formant feature scoring logic."""
        validator = GhunnahValidator()

        # Strong Ghunnah features
        score_strong = validator._score_formants(
            f1_hz=400,        # Low F1 (nasalization)
            f2_hz=1500,       # F2-F1 coupling
            f3_hz=2500,       # F3 presence
            nasal_energy_db=-15  # High nasal energy
        )
        assert score_strong > 0.8

        # Weak Ghunnah features
        score_weak = validator._score_formants(
            f1_hz=700,        # High F1 (no nasalization)
            f2_hz=2200,       # No F2-F1 coupling
            f3_hz=1500,       # Low F3
            nasal_energy_db=-30  # Low nasal energy
        )
        assert score_weak < 0.7

    def test_severity_classification(self):
        """Test severity levels based on confidence."""
        validator = GhunnahValidator()

        assert validator._compute_severity(0.2) == "critical"
        assert validator._compute_severity(0.5) == "moderate"
        assert validator._compute_severity(0.65) == "minor"

    def test_feedback_generation(self):
        """Test feedback message generation."""
        validator = GhunnahValidator()

        feedback = validator._generate_feedback('ن', 0.4, "moderate")

        assert "Ghunnah" in feedback
        assert "ن" in feedback
        assert "nose" in feedback.lower()


class TestQalqalahValidator:
    """Test Qalqalah validator with burst detection."""

    def test_initialization_with_burst_detection(self):
        """Test validator initializes with burst detection enabled."""
        validator = QalqalahValidator(
            use_burst_detection=True,
            burst_weight=0.4
        )

        assert validator.use_burst_detection == True
        assert validator.burst_weight == 0.4
        assert validator.confidence_threshold == 0.6

    def test_initialization_without_burst_detection(self):
        """Test validator initializes without burst detection."""
        validator = QalqalahValidator(use_burst_detection=False)

        assert validator.use_burst_detection == False

    def test_qalqalah_letter_detection(self):
        """Test detection of Qalqalah letters."""
        validator = QalqalahValidator()

        # Qalqalah letters
        assert validator._is_qalqalah_letter(MockPhoneme('q', 0, 0.1))
        assert validator._is_qalqalah_letter(MockPhoneme('T', 0, 0.1))
        assert validator._is_qalqalah_letter(MockPhoneme('b', 0, 0.1))
        assert validator._is_qalqalah_letter(MockPhoneme('j', 0, 0.1))
        assert validator._is_qalqalah_letter(MockPhoneme('d', 0, 0.1))
        assert validator._is_qalqalah_letter(MockPhoneme('ق', 0, 0.1))

        # Non-Qalqalah letters
        assert not validator._is_qalqalah_letter(MockPhoneme('n', 0, 0.1))
        assert not validator._is_qalqalah_letter(MockPhoneme('a', 0, 0.1))

    def test_baseline_confidence_extraction(self):
        """Test extraction of baseline confidence from sifat."""
        validator = QalqalahValidator()

        # High confidence sifat
        phoneme_high = MockPhoneme(
            'q', 0, 0.1,
            sifa={'qalqla': {'text': 'qalqala', 'prob': 0.85}}
        )
        assert validator._get_baseline_confidence(phoneme_high) == 0.85

        # Low confidence sifat
        phoneme_low = MockPhoneme(
            'b', 0, 0.1,
            sifa={'qalqla': {'text': 'qalqala', 'prob': 0.5}}
        )
        assert validator._get_baseline_confidence(phoneme_low) == 0.5

        # No sifat
        phoneme_none = MockPhoneme('d', 0, 0.1, sifa=None)
        assert validator._get_baseline_confidence(phoneme_none) == 0.5

    def test_validate_high_confidence_skips_burst(self):
        """Test that high confidence Tier 1 skips burst detection."""
        validator = QalqalahValidator(use_burst_detection=True)

        # High confidence Qalqalah (>0.8)
        phonemes = [
            MockPhoneme(
                'q', 0, 0.1,
                sifa={'qalqla': {'text': 'qalqala', 'prob': 0.9}}
            )
        ]

        # No audio needed
        violations = validator.validate(phonemes, audio=None)

        # Should have no violations (high confidence)
        assert len(violations) == 0

    def test_validate_low_confidence_without_audio(self):
        """Test low confidence without audio falls back to baseline."""
        validator = QalqalahValidator(use_burst_detection=True)

        # Low confidence Qalqalah (<0.8, <0.6)
        phonemes = [
            MockPhoneme(
                'b', 0, 0.1,
                sifa={'qalqla': {'text': 'qalqala', 'prob': 0.4}}
            )
        ]

        # No audio, should use baseline only (below 0.6 threshold)
        violations = validator.validate(phonemes, audio=None)

        # Should have violation (low confidence)
        assert len(violations) == 1
        assert violations[0].rule == "Qalqalah"
        assert violations[0].tier == 2

    def test_burst_score_computation(self):
        """Test burst feature scoring logic."""
        validator = QalqalahValidator()

        # Strong burst features
        score_strong = validator._score_burst(
            zcr_mean=0.4,        # High ZCR (sharp transient)
            zcr_std=0.1,
            centroid_mean=2000,  # High centroid (brightness)
            rms_max=0.8,
            rms_std=0.15,        # High variability
            has_burst=True       # Burst detected
        )
        assert score_strong > 0.7

        # Weak burst features
        score_weak = validator._score_burst(
            zcr_mean=0.15,       # Low ZCR
            zcr_std=0.05,
            centroid_mean=1000,  # Low centroid
            rms_max=0.3,
            rms_std=0.005,       # Low variability
            has_burst=False      # No burst
        )
        assert score_weak < 0.5

    def test_severity_classification(self):
        """Test severity levels based on confidence."""
        validator = QalqalahValidator()

        assert validator._compute_severity(0.2) == "critical"
        assert validator._compute_severity(0.45) == "moderate"
        assert validator._compute_severity(0.55) == "minor"

    def test_feedback_generation(self):
        """Test feedback message generation."""
        validator = QalqalahValidator()

        feedback = validator._generate_feedback('ق', 0.35, "moderate")

        assert "Qalqalah" in feedback
        assert "ق" in feedback
        assert "burst" in feedback.lower()


class TestPhase2Integration:
    """Test Phase 2 validators integration."""

    def test_ghunnah_without_parselmouth(self):
        """Test Ghunnah validator gracefully handles missing parselmouth."""
        # This test verifies fallback behavior
        validator = GhunnahValidator(use_formants=True)

        # Even if use_formants=True, should not crash
        assert validator is not None

    def test_qalqalah_without_librosa(self):
        """Test Qalqalah validator gracefully handles missing librosa."""
        # This test verifies fallback behavior
        validator = QalqalahValidator(use_burst_detection=True)

        # Even if use_burst_detection=True, should not crash
        assert validator is not None

    def test_empty_phoneme_list(self):
        """Test validators handle empty phoneme lists."""
        ghunnah_validator = GhunnahValidator()
        qalqalah_validator = QalqalahValidator()

        ghunnah_violations = ghunnah_validator.validate([])
        qalqalah_violations = qalqalah_validator.validate([])

        assert len(ghunnah_violations) == 0
        assert len(qalqalah_violations) == 0

    def test_non_applicable_phonemes(self):
        """Test validators skip non-applicable phonemes."""
        ghunnah_validator = GhunnahValidator()
        qalqalah_validator = QalqalahValidator()

        # Phonemes that don't apply to these rules
        phonemes = [
            MockPhoneme('a', 0, 0.1),  # Vowel
            MockPhoneme('s', 0.1, 0.2),  # Non-nasal, non-qalqalah
        ]

        ghunnah_violations = ghunnah_validator.validate(phonemes)
        qalqalah_violations = qalqalah_validator.validate(phonemes)

        assert len(ghunnah_violations) == 0
        assert len(qalqalah_violations) == 0


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
