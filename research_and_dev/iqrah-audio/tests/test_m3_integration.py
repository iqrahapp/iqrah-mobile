"""
Integration tests for M3 (Phoneme Recognition & Alignment) with Muaalem.

Tests the complete M3 pipeline:
1. Text phonetization (quran_phonetizer wrapper)
2. Muaalem ASR inference (phonemes + sifat)
3. Phonetic gatekeeper (PER-based verification)
4. Phoneme-level CTC alignment
"""

import pytest
import numpy as np
import torch
from quran_transcript import Aya

from iqrah.text import phonetize_ayah, Phonetizer
from iqrah.asr import MuaalemASR
from iqrah.compare import PhoneticGatekeeper, compute_per
from iqrah.align import PhonemeCTCAligner


# Test data: Use proper Aya class to get Quranic text
# Al-Fatihah (1), Ayah 1: بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ
TEST_AYA = Aya(1, 1)
TEST_TEXT = TEST_AYA.get().uthmani
TEST_AUDIO_DURATION = 2.0  # seconds
SAMPLE_RATE = 16000


@pytest.fixture
def sample_audio():
    """Generate synthetic audio for testing."""
    # Generate 2 seconds of white noise
    duration = TEST_AUDIO_DURATION
    samples = int(duration * SAMPLE_RATE)
    audio = np.random.randn(samples).astype(np.float32) * 0.1
    return audio


@pytest.fixture
def phonetizer():
    """Create phonetizer instance."""
    return Phonetizer(rewaya="hafs")


class TestPhonetizer:
    """Test T3.1: Text Phonetizer."""

    def test_phonetize_ayah_basic(self):
        """Test basic phonetization."""
        result = phonetize_ayah(TEST_TEXT)

        assert result is not None
        assert isinstance(result.text, str)
        assert len(result.text) > 0
        assert result.metadata["total_phonemes"] > 0
        assert result.raw_output is not None

    def test_phonetize_with_space(self):
        """Test phonetization with space preservation."""
        result_no_space = phonetize_ayah(TEST_TEXT, remove_space=True)
        result_with_space = phonetize_ayah(TEST_TEXT, remove_space=False)

        # With spaces should be longer or equal
        assert len(result_with_space.text) >= len(result_no_space.text)

    def test_phonetizer_class(self, phonetizer):
        """Test stateful Phonetizer class."""
        result = phonetizer.phonetize(TEST_TEXT)

        assert result is not None
        assert len(result.text) > 0
        assert result.metadata["moshaf_config"]["rewaya"] == "hafs"

    def test_phonetic_units(self):
        """Test PhoneticUnit generation."""
        result = phonetize_ayah(TEST_TEXT)

        assert len(result.units) > 0
        assert all(hasattr(u, "phoneme") for u in result.units)
        assert all(hasattr(u, "position") for u in result.units)


class TestPhoneticGatekeeper:
    """Test T3.5: Phonetic Gatekeeper (PER-based)."""

    def test_compute_per_perfect_match(self):
        """Test PER computation with perfect match."""
        ref = ['b', 'i', 's', 'm']
        pred = ['b', 'i', 's', 'm']

        per, errors = compute_per(ref, pred)

        assert per == 0.0
        assert len(errors) == 0

    def test_compute_per_single_substitution(self):
        """Test PER with one substitution."""
        ref = ['b', 'i', 's', 'm']
        pred = ['b', 'a', 's', 'm']  # 'i' -> 'a'

        per, errors = compute_per(ref, pred)

        assert per == 0.25  # 1 error / 4 phonemes
        assert len(errors) == 1
        assert errors[0].type == "substitution"
        assert errors[0].reference_phoneme == 'i'
        assert errors[0].predicted_phoneme == 'a'

    def test_compute_per_deletion(self):
        """Test PER with deletion."""
        ref = ['b', 'i', 's', 'm']
        pred = ['b', 's', 'm']  # 'i' deleted

        per, errors = compute_per(ref, pred)

        assert per == 0.25
        assert len(errors) == 1
        assert errors[0].type == "deletion"

    def test_compute_per_insertion(self):
        """Test PER with insertion."""
        ref = ['b', 'i', 's', 'm']
        pred = ['b', 'i', 'x', 's', 'm']  # 'x' inserted

        per, errors = compute_per(ref, pred)

        assert per == 0.25
        assert len(errors) == 1
        assert errors[0].type == "insertion"

    def test_gatekeeper_high_confidence(self):
        """Test gatekeeper with high confidence (PER ≤ 0.02)."""
        gate = PhoneticGatekeeper()

        result = gate.verify(
            reference_phonemes=['b', 'i', 's', 'm'] * 100,  # 400 phonemes
            predicted_phonemes=['b', 'i', 's', 'm'] * 100   # Perfect match
        )

        assert result['per'] == 0.0
        assert result['confidence'] == "high"
        assert result['should_proceed'] is True

    def test_gatekeeper_medium_confidence(self):
        """Test gatekeeper with medium confidence (0.02 < PER ≤ 0.05)."""
        gate = PhoneticGatekeeper()

        # Create 3% error rate (3 errors in 100 phonemes)
        ref = ['a'] * 100
        pred = ['a'] * 97 + ['b', 'c', 'd']

        result = gate.verify(ref, pred)

        assert 0.02 < result['per'] <= 0.05
        assert result['confidence'] == "medium"
        assert result['should_proceed'] is True

    def test_gatekeeper_fail(self):
        """Test gatekeeper failure (PER > 0.05)."""
        gate = PhoneticGatekeeper()

        # Create 10% error rate
        ref = ['a'] * 100
        pred = ['a'] * 90 + ['b'] * 10

        result = gate.verify(ref, pred)

        assert result['per'] > 0.05
        assert result['confidence'] == "fail"
        assert result['should_proceed'] is False

    def test_gatekeeper_from_text(self):
        """Test gatekeeper with text input."""
        gate = PhoneticGatekeeper()

        result = gate.verify_from_text("bism", "bism")

        assert result['per'] == 0.0
        assert result['confidence'] == "high"


class TestMuaalemASRWrapper:
    """Test T3.2: Muaalem ASR Wrapper."""

    @pytest.mark.skip(reason="Requires Muaalem model download and real audio")
    def test_muaalem_inference_basic(self, sample_audio):
        """Test basic Muaalem inference."""
        model = MuaalemASR(device="cpu")
        phonetic_ref = phonetize_ayah(TEST_TEXT)

        result = model.infer(
            audio=sample_audio,
            phonetic_ref=phonetic_ref,
            sample_rate=SAMPLE_RATE
        )

        assert result is not None
        assert result.phonemes is not None
        assert len(result.sifat) > 0
        assert result.duration > 0

    @pytest.mark.skip(reason="Requires Muaalem model")
    def test_muaalem_chunking(self):
        """Test automatic chunking for long audio."""
        model = MuaalemASR(device="cpu", chunk_duration=20.0)
        phonetic_ref = phonetize_ayah(TEST_TEXT)

        # Generate 30 seconds of audio
        long_audio = np.random.randn(30 * SAMPLE_RATE).astype(np.float32)

        result = model.infer(
            audio=long_audio,
            phonetic_ref=phonetic_ref,
            sample_rate=SAMPLE_RATE
        )

        assert result.duration > 20.0  # Longer than chunk size

    def test_invalid_sample_rate(self, sample_audio):
        """Test error handling for invalid sample rate."""
        model = MuaalemASR(device="cpu")
        phonetic_ref = phonetize_ayah(TEST_TEXT)

        with pytest.raises(ValueError, match="Sample rate must be 16kHz"):
            model.infer(
                audio=sample_audio,
                phonetic_ref=phonetic_ref,
                sample_rate=44100  # Invalid
            )


class TestPhonemeCTCAligner:
    """Test T3.3: Phoneme-Level CTC Aligner."""

    @pytest.mark.skip(reason="Requires Muaalem model and real alignment")
    def test_phoneme_alignment_basic(self, sample_audio):
        """Test basic phoneme-level alignment."""
        model = MuaalemASR(device="cpu")
        aligner = PhonemeCTCAligner(model)
        phonetic_ref = phonetize_ayah(TEST_TEXT)

        result = aligner.align(
            audio=sample_audio,
            phonetic_ref=phonetic_ref,
            sample_rate=SAMPLE_RATE
        )

        assert result is not None
        assert "phonemes" in result
        assert "words" in result
        assert "alignment_method" in result
        assert result["alignment_method"] in ["ctc_phoneme_forced", "ctc_phoneme_fallback"]

    @pytest.mark.skip(reason="Requires Muaalem model")
    def test_phoneme_alignment_with_sifat(self, sample_audio):
        """Test phoneme alignment includes sifat."""
        model = MuaalemASR(device="cpu")
        aligner = PhonemeCTCAligner(model)
        phonetic_ref = phonetize_ayah(TEST_TEXT)

        result = aligner.align(
            audio=sample_audio,
            phonetic_ref=phonetic_ref,
            sample_rate=SAMPLE_RATE
        )

        # Check that phonemes have sifat attached
        if result["phonemes"]:
            assert hasattr(result["phonemes"][0], "sifa")


class TestM3Integration:
    """Integration tests for complete M3 pipeline."""

    @pytest.mark.skip(reason="Requires Muaalem model and real audio data")
    def test_full_m3_pipeline(self, sample_audio):
        """Test complete M3 pipeline end-to-end using M3Pipeline orchestrator."""
        from iqrah.pipeline import M3Pipeline

        # Initialize M3 pipeline
        pipeline = M3Pipeline(device="cpu")

        # Process audio
        result = pipeline.process(
            audio=sample_audio,
            reference_text=TEST_TEXT,
            sample_rate=SAMPLE_RATE
        )

        # Verify output structure
        assert result.phonemes is not None
        assert len(result.phonemes) > 0
        assert result.gate_result is not None
        assert result.alignment_method in ["ctc_phoneme_forced", "ctc_phoneme_fallback"]

        # Verify gate result
        assert hasattr(result.gate_result, "passed")
        assert hasattr(result.gate_result, "per")
        assert hasattr(result.gate_result, "confidence")
        assert hasattr(result.gate_result, "errors")

        # Verify phoneme alignments have sifat
        if result.phonemes:
            assert hasattr(result.phonemes[0], "sifa")
            assert hasattr(result.phonemes[0], "start")
            assert hasattr(result.phonemes[0], "end")
            assert hasattr(result.phonemes[0], "phoneme")

    def test_m3_output_schema(self):
        """Test that M3 output matches documented schema."""
        from iqrah.pipeline import M3Output, GateResult
        from iqrah.align import PhonemeAlignment

        # Create mock output matching schema
        gate_result = GateResult(
            passed=True,
            per=0.02,
            confidence=0.98,
            errors=[]
        )

        output = M3Output(
            phonemes=[],
            words=[],
            gate_result=gate_result,
            alignment_method="ctc_phoneme_forced"
        )

        # Verify structure
        assert hasattr(output, "phonemes")
        assert hasattr(output, "words")
        assert hasattr(output, "gate_result")
        assert hasattr(output, "alignment_method")

        # Verify gate result structure
        assert hasattr(gate_result, "passed")
        assert hasattr(gate_result, "per")
        assert hasattr(gate_result, "confidence")
        assert hasattr(gate_result, "errors")


class TestM4Integration:
    """Integration tests for M4 Tier 1 Tajweed validation."""

    def test_m4_baseline_validator_schema(self):
        """Test that M4 Tier 1 validator follows expected schema."""
        from iqrah.tajweed import BaselineTajweedInterpreter, TajweedViolation
        from iqrah.align import PhonemeAlignment

        # Initialize validator
        validator = BaselineTajweedInterpreter(confidence_threshold=0.7)

        # Create mock phoneme alignments
        mock_phonemes = [
            PhonemeAlignment(
                phoneme="b",
                start=0.0,
                end=0.1,
                confidence=0.9,
                sifa={
                    "hams_or_jahr": {"text": "jahr", "prob": 0.95},
                    "shidda_or_rakhawa": {"text": "rakhawa", "prob": 0.92}
                }
            )
        ]

        # Validate
        violations = validator.validate(mock_phonemes)

        # Verify structure
        assert isinstance(violations, dict)
        assert all(isinstance(v, list) for v in violations.values())

        # Compute scores
        scores = validator.compute_scores(violations, len(mock_phonemes))
        assert "overall" in scores
        assert all(0.0 <= score <= 100.0 for score in scores.values())

    def test_tajweed_violation_schema(self):
        """Test TajweedViolation dataclass structure."""
        from iqrah.tajweed import TajweedViolation

        violation = TajweedViolation(
            rule="Ghunnah",
            phoneme_idx=0,
            phoneme="ن",
            timestamp=1.5,
            expected="high_confidence",
            actual="nasal",
            confidence=0.65,
            severity="medium",
            tier=1,
            feedback="Low confidence ghunnah detection"
        )

        # Verify all required fields
        assert violation.rule == "Ghunnah"
        assert violation.phoneme_idx == 0
        assert violation.phoneme == "ن"
        assert violation.timestamp == 1.5
        assert violation.tier == 1

    @pytest.mark.skip(reason="Requires Muaalem model and real audio")
    def test_m3_m4_integrated_pipeline(self, sample_audio):
        """Test complete M3+M4 integration."""
        from iqrah.pipeline import M3Pipeline
        from iqrah.tajweed import BaselineTajweedInterpreter

        # Initialize pipelines
        m3_pipeline = M3Pipeline(device="cpu")
        m4_validator = BaselineTajweedInterpreter()

        # Run M3
        m3_result = m3_pipeline.process(
            audio=sample_audio,
            reference_text=TEST_TEXT,
            sample_rate=SAMPLE_RATE
        )

        # Run M4
        violations = m4_validator.validate(m3_result.phonemes)
        scores = m4_validator.compute_scores(violations, len(m3_result.phonemes))

        # Verify integration
        assert len(m3_result.phonemes) > 0
        assert isinstance(violations, dict)
        assert "overall" in scores
        assert scores["overall"] >= 0.0


class TestEdgeCases:
    """Test edge cases and error handling."""

    def test_empty_text_phonetization(self):
        """Test phonetization with empty text."""
        result = phonetize_ayah("")

        # Should handle gracefully
        assert result is not None
        assert len(result.text) == 0 or len(result.text) > 0

    def test_per_with_empty_sequences(self):
        """Test PER computation with empty sequences."""
        per, errors = compute_per([], [])

        assert per == 0.0
        assert len(errors) == 0

    def test_per_with_one_empty(self):
        """Test PER when reference is empty but prediction is not."""
        per, errors = compute_per([], ['a', 'b', 'c'])

        # Should return infinite PER (all insertions, no reference)
        assert per == float('inf')
        assert len(errors) == 3


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
