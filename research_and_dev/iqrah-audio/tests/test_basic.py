"""
Basic tests for iqrah-audio package.

Tests core functionality with synthetic audio.
"""

import numpy as np
import pytest
from pathlib import Path
import tempfile

from iqrah_audio import (
    PitchExtractor,
    PitchContour,
    AudioDenoiser,
    DTWAligner,
    RecitationScorer,
    ReferenceProcessor,
)


def generate_sine_wave(frequency=440.0, duration=1.0, sample_rate=22050, noise_level=0.0):
    """Generate sine wave for testing."""
    t = np.linspace(0, duration, int(sample_rate * duration))
    signal = np.sin(2 * np.pi * frequency * t)

    if noise_level > 0:
        noise = np.random.normal(0, noise_level, len(signal))
        signal += noise

    return signal.astype(np.float32)


class TestPitchExtractor:
    def test_extract_yin(self):
        """Test YIN pitch extraction on known frequency."""
        audio = generate_sine_wave(frequency=440.0, duration=1.0)

        extractor = PitchExtractor(method="yin", sample_rate=22050)
        contour = extractor.extract(audio)

        assert isinstance(contour, PitchContour)
        assert len(contour.f0_hz) > 0
        assert contour.sample_rate == 22050

        # Check that detected pitch is close to 440 Hz
        voiced_f0 = contour.f0_hz[contour.confidence > 0.5]
        if len(voiced_f0) > 0:
            mean_f0 = np.median(voiced_f0)
            assert 400 < mean_f0 < 480, f"Expected ~440 Hz, got {mean_f0:.1f} Hz"

    def test_stability_filtering(self):
        """Test pitch stability filtering."""
        audio = generate_sine_wave(frequency=440.0, duration=1.0)

        extractor = PitchExtractor(method="yin", sample_rate=22050)
        contour = extractor.extract_stable_pitch(audio, median_filter_size=5)

        assert isinstance(contour, PitchContour)

    def test_pitch_contour_conversion(self):
        """Test pitch conversions (Hz -> cents -> semitones)."""
        contour = PitchContour(
            f0_hz=np.array([440.0, 880.0, 220.0, 0.0]),
            confidence=np.array([0.9, 0.9, 0.9, 0.1]),
            timestamps=np.array([0.0, 0.5, 1.0, 1.5]),
            sample_rate=22050
        )

        # Test conversions
        cents = contour.f0_cents
        assert cents[0] == pytest.approx(0.0, abs=1)  # 440 Hz = A4 = 0 cents
        assert cents[1] == pytest.approx(1200.0, abs=1)  # 880 Hz = A5 = +1200 cents
        assert cents[2] == pytest.approx(-1200.0, abs=1)  # 220 Hz = A3 = -1200 cents
        assert cents[3] == 0.0  # Unvoiced

        semitones = contour.f0_semitones
        assert semitones[0] == pytest.approx(0.0, abs=0.1)
        assert semitones[1] == pytest.approx(12.0, abs=0.1)
        assert semitones[2] == pytest.approx(-12.0, abs=0.1)

    def test_voiced_segments(self):
        """Test voiced segment extraction."""
        contour = PitchContour(
            f0_hz=np.array([440.0, 440.0, 0.0, 0.0, 440.0, 440.0]),
            confidence=np.array([0.9, 0.9, 0.1, 0.1, 0.9, 0.9]),
            timestamps=np.array([0.0, 0.1, 0.2, 0.3, 0.4, 0.5]),
            sample_rate=22050
        )

        segments = contour.get_voiced_segments(confidence_threshold=0.5)

        assert len(segments) == 2
        assert segments[0] == pytest.approx((0.0, 0.1), abs=0.01)
        assert segments[1] == pytest.approx((0.4, 0.5), abs=0.01)


class TestAudioDenoiser:
    def test_denoise(self):
        """Test basic denoising."""
        clean_audio = generate_sine_wave(frequency=440.0, duration=1.0)
        noisy_audio = generate_sine_wave(frequency=440.0, duration=1.0, noise_level=0.1)

        denoiser = AudioDenoiser(sample_rate=22050)
        denoised = denoiser.denoise(noisy_audio)

        assert len(denoised) == len(noisy_audio)
        assert denoised.dtype == np.float32

    def test_snr_estimation(self):
        """Test SNR estimation."""
        clean_audio = generate_sine_wave(frequency=440.0, duration=1.0)
        noisy_audio = generate_sine_wave(frequency=440.0, duration=1.0, noise_level=0.1)

        denoiser = AudioDenoiser(sample_rate=22050)
        denoised = denoiser.denoise(noisy_audio)

        snr = denoiser.estimate_snr(noisy_audio, denoised)
        assert snr > 0  # Should have positive SNR improvement


class TestDTWAligner:
    def test_perfect_alignment(self):
        """Test DTW with identical sequences."""
        sequence = np.array([1.0, 2.0, 3.0, 2.0, 1.0])

        aligner = DTWAligner()
        result = aligner.align(sequence, sequence)

        assert result.distance == pytest.approx(0.0, abs=1e-6)
        assert result.alignment_score == pytest.approx(1.0, abs=0.01)
        assert len(result.path) > 0

    def test_shifted_alignment(self):
        """Test DTW with shifted sequence."""
        reference = np.array([0.0, 1.0, 2.0, 3.0, 2.0, 1.0, 0.0])
        query = np.array([1.0, 2.0, 3.0, 2.0, 1.0])

        aligner = DTWAligner()
        result = aligner.align(query, reference)

        # Should find good alignment despite shift
        assert result.alignment_score > 0.8
        assert len(result.path) > 0

    def test_find_best_window(self):
        """Test finding best alignment window in long reference."""
        # Create reference with pattern at offset 100
        reference = np.random.randn(500)
        pattern = np.array([1.0, 2.0, 3.0, 2.0, 1.0])
        reference[100:105] = pattern

        aligner = DTWAligner()
        best_offset, best_score = aligner.find_best_window(pattern, reference, max_offset=200)

        # Should find the pattern near offset 100
        assert 80 < best_offset < 120
        assert best_score > 0.8


class TestRecitationScorer:
    def test_scoring(self):
        """Test recitation scoring with synthetic contours."""
        # Create reference contour
        ref_f0 = np.array([440.0] * 50)  # Constant A4
        reference = PitchContour(
            f0_hz=ref_f0,
            confidence=np.ones(50) * 0.9,
            timestamps=np.linspace(0, 1, 50),
            sample_rate=22050
        )

        # Create user contour (slightly off-pitch)
        user_f0 = np.array([445.0] * 50)  # ~20 cents sharp
        user = PitchContour(
            f0_hz=user_f0,
            confidence=np.ones(50) * 0.9,
            timestamps=np.linspace(0, 1, 50),
            sample_rate=22050
        )

        scorer = RecitationScorer(on_note_threshold_cents=50.0)
        score = scorer.score(user, reference)

        # Should get good scores (within threshold)
        assert score.overall_score > 70
        assert score.on_note_percent > 90  # Within 50 cents threshold
        assert 0 <= score.alignment_score <= 100
        assert 0 <= score.pitch_stability <= 100
        assert 0 <= score.tempo_score <= 100

    def test_poor_alignment(self):
        """Test scoring with poor alignment."""
        reference = PitchContour(
            f0_hz=np.array([440.0] * 50),
            confidence=np.ones(50) * 0.9,
            timestamps=np.linspace(0, 1, 50),
            sample_rate=22050
        )

        # User is way off-pitch
        user = PitchContour(
            f0_hz=np.array([300.0] * 50),  # Much lower
            confidence=np.ones(50) * 0.9,
            timestamps=np.linspace(0, 1, 50),
            sample_rate=22050
        )

        scorer = RecitationScorer(on_note_threshold_cents=50.0)
        score = scorer.score(user, reference)

        # Should get low on-note percentage
        assert score.on_note_percent < 50


class TestReferenceProcessor:
    def test_cbor_serialization(self):
        """Test CBOR save/load round-trip."""
        # Create synthetic reference data
        contour = PitchContour(
            f0_hz=np.array([440.0, 445.0, 450.0]),
            confidence=np.array([0.9, 0.9, 0.9]),
            timestamps=np.array([0.0, 0.1, 0.2]),
            sample_rate=22050
        )

        ref_data = {
            "contour": contour.to_dict(),
            "metadata": {"ayah": "1:1", "qari": "test"},
            "processing": {"sample_rate": 22050, "pitch_method": "yin"}
        }

        # Save and load
        processor = ReferenceProcessor()

        with tempfile.NamedTemporaryFile(suffix=".cbor.zst", delete=False) as f:
            temp_path = Path(f.name)

        try:
            processor.save_cbor(ref_data, temp_path, compress=True)
            loaded_data = processor.load_cbor(temp_path)

            # Verify
            assert loaded_data["metadata"] == ref_data["metadata"]
            assert loaded_data["processing"] == ref_data["processing"]

            loaded_contour = PitchContour.from_dict(loaded_data["contour"])
            np.testing.assert_array_almost_equal(loaded_contour.f0_hz, contour.f0_hz)
            np.testing.assert_array_almost_equal(loaded_contour.confidence, contour.confidence)
            np.testing.assert_array_almost_equal(loaded_contour.timestamps, contour.timestamps)

        finally:
            temp_path.unlink()


class TestOnlineDTW:
    def test_online_alignment(self):
        """Test online DTW aligner."""
        from iqrah_audio.dtw import OnlineDTWAligner

        reference = np.array([1.0, 2.0, 3.0, 2.0, 1.0, 0.0] * 10)  # Repeating pattern

        aligner = OnlineDTWAligner(window_size=10, band_width=5)

        # Feed frames incrementally
        for i in range(30):
            query_frame = reference[i] + np.random.normal(0, 0.1)  # Add noise
            result = aligner.update(query_frame, reference)

            # Should start returning results after buffer fills
            if i >= 10:
                assert result is not None
                assert result.alignment_score > 0
