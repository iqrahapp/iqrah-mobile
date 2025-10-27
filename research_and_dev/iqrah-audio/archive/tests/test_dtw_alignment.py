#!/usr/bin/env python3
"""
Unit Tests for DTW Alignment
=============================

Tests DTW alignment for correctness, edge cases, and performance.
"""

import unittest
import numpy as np
import sys
from pathlib import Path

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

from iqrah_audio.analysis.offline import calculate_dtw_alignment


class TestDTWAlignment(unittest.TestCase):
    """Unit tests for DTW alignment"""

    def test_identical_sequences(self):
        """Test DTW on identical pitch sequences"""
        pitch1 = [100, 110, 120, 130, 140, 150]
        pitch2 = [100, 110, 120, 130, 140, 150]

        result = calculate_dtw_alignment(pitch1, pitch2)

        # Should have perfect alignment
        self.assertIn('distance', result)
        self.assertIn('user_to_ref', result)
        self.assertIn('ref_to_user', result)

        # Distance should be very small (near zero)
        self.assertLess(result['distance'], 1.0, "Identical sequences should have near-zero distance")

        # Mapping should be 1:1
        self.assertEqual(len(result['user_to_ref']), len(pitch1))
        for i in range(len(pitch1)):
            self.assertEqual(result['user_to_ref'][i], i, f"Frame {i} should map to itself")

    def test_time_stretched_sequence(self):
        """Test DTW on time-stretched sequence (same pitch, different speed)"""
        # Reference: normal speed
        ref_pitch = [100, 110, 120, 130, 140, 150]

        # User: 2x slower (each pitch repeated twice)
        user_pitch = [100, 100, 110, 110, 120, 120, 130, 130, 140, 140, 150, 150]

        result = calculate_dtw_alignment(user_pitch, ref_pitch)

        # Should align successfully
        self.assertIn('user_to_ref', result)

        # Each user frame should map to correct reference frame
        # Frames 0,1 → 0; Frames 2,3 → 1; etc.
        for i, ref_idx in result['user_to_ref'].items():
            expected_ref = i // 2
            self.assertEqual(
                ref_idx,
                expected_ref,
                f"User frame {i} should map to ref frame {expected_ref}, got {ref_idx}"
            )

    def test_pitch_variation_tolerance(self):
        """Test DTW handles small pitch variations"""
        # Reference pitch
        ref_pitch = [100, 110, 120, 130, 140, 150]

        # User pitch with small variations (±5 Hz)
        user_pitch = [102, 108, 122, 128, 142, 148]

        result = calculate_dtw_alignment(user_pitch, ref_pitch)

        # Should align successfully despite variations
        self.assertIn('user_to_ref', result)
        self.assertEqual(len(result['user_to_ref']), len(user_pitch))

        # Distance should be reasonable (sum of differences ~30)
        self.assertLess(result['distance'], 100, "Small pitch variations should have low distance")

    def test_pitch_shift_alignment(self):
        """Test DTW aligns sequences with constant pitch shift"""
        # Reference pitch contour
        ref_pitch = [100, 110, 120, 110, 100]

        # User: same contour but shifted +20 Hz
        user_pitch = [120, 130, 140, 130, 120]

        result = calculate_dtw_alignment(user_pitch, ref_pitch)

        # Should align correctly (DTW finds temporal alignment)
        self.assertIn('user_to_ref', result)

        # Mapping should be 1:1 since they have same temporal structure
        for i in range(len(user_pitch)):
            self.assertEqual(
                result['user_to_ref'][i],
                i,
                f"Pitch-shifted sequence should have 1:1 alignment"
            )

    def test_insertion_handling(self):
        """Test DTW handles inserted frames"""
        # Reference
        ref_pitch = [100, 120, 140]

        # User: extra frame inserted in middle
        user_pitch = [100, 110, 120, 140]

        result = calculate_dtw_alignment(user_pitch, ref_pitch)

        # Should align successfully
        self.assertIn('user_to_ref', result)
        self.assertEqual(len(result['user_to_ref']), len(user_pitch))

        # First frame should map to first ref
        self.assertEqual(result['user_to_ref'][0], 0)

        # Last frame should map to last ref
        self.assertEqual(result['user_to_ref'][3], 2)

    def test_deletion_handling(self):
        """Test DTW handles deleted frames"""
        # Reference
        ref_pitch = [100, 110, 120, 130, 140]

        # User: skipped middle frame
        user_pitch = [100, 110, 130, 140]

        result = calculate_dtw_alignment(user_pitch, ref_pitch)

        # Should align successfully
        self.assertIn('user_to_ref', result)
        self.assertEqual(len(result['user_to_ref']), len(user_pitch))

    def test_unvoiced_frames_handling(self):
        """Test DTW handles unvoiced frames (zero pitch)"""
        # Reference with unvoiced frames
        ref_pitch = [100, 110, 0, 0, 120, 130]

        # User with different unvoiced pattern
        user_pitch = [100, 0, 110, 120, 0, 130]

        result = calculate_dtw_alignment(user_pitch, ref_pitch)

        # Should handle zeros gracefully
        self.assertIn('user_to_ref', result)
        self.assertIn('distance', result)

        # Should not crash or produce NaN
        self.assertFalse(np.isnan(result['distance']), "Distance should not be NaN")

    def test_empty_sequence_handling(self):
        """Test DTW handles edge case of empty sequences"""
        # Empty user pitch
        user_pitch = []
        ref_pitch = [100, 110, 120]

        # Should handle gracefully (may return empty mapping or raise)
        try:
            result = calculate_dtw_alignment(user_pitch, ref_pitch)
            # If it doesn't raise, check result is valid
            self.assertIn('user_to_ref', result)
        except (ValueError, IndexError):
            # Expected: DTW can't align empty sequence
            pass

    def test_single_frame_alignment(self):
        """Test DTW on single frame sequences"""
        user_pitch = [100]
        ref_pitch = [105]

        result = calculate_dtw_alignment(user_pitch, ref_pitch)

        # Should align single frame
        self.assertIn('user_to_ref', result)
        self.assertEqual(len(result['user_to_ref']), 1)
        self.assertEqual(result['user_to_ref'][0], 0)

    def test_very_different_lengths(self):
        """Test DTW handles very different sequence lengths"""
        # Short reference
        ref_pitch = [100, 110, 120]

        # Much longer user sequence (3x)
        user_pitch = [100] * 3 + [110] * 3 + [120] * 3

        result = calculate_dtw_alignment(user_pitch, ref_pitch)

        # Should align successfully
        self.assertIn('user_to_ref', result)
        self.assertEqual(len(result['user_to_ref']), len(user_pitch))

        # Each group should map to corresponding reference frame
        for i in range(3):
            self.assertEqual(result['user_to_ref'][i], 0, "First 3 frames should map to ref[0]")
        for i in range(3, 6):
            self.assertEqual(result['user_to_ref'][i], 1, "Middle 3 frames should map to ref[1]")
        for i in range(6, 9):
            self.assertEqual(result['user_to_ref'][i], 2, "Last 3 frames should map to ref[2]")

    def test_bidirectional_mapping(self):
        """Test that user_to_ref and ref_to_user are consistent"""
        ref_pitch = [100, 110, 120, 130]
        user_pitch = [100, 105, 110, 115, 120, 125, 130]

        result = calculate_dtw_alignment(user_pitch, ref_pitch)

        # Check both mappings exist
        self.assertIn('user_to_ref', result)
        self.assertIn('ref_to_user', result)

        # Verify ref_to_user is inverse mapping
        # For each ref frame, check that mapped user frames map back
        for ref_idx, user_val in result['ref_to_user'].items():
            user_indices = user_val if isinstance(user_val, list) else [user_val]
            for user_idx in user_indices:
                self.assertEqual(
                    result['user_to_ref'][user_idx],
                    ref_idx,
                    f"Bidirectional mapping inconsistent: ref[{ref_idx}] → user[{user_idx}] "
                    f"but user[{user_idx}] → ref[{result['user_to_ref'][user_idx]}]"
                )

    def test_alignment_monotonicity(self):
        """Test that DTW alignment is monotonically increasing"""
        ref_pitch = [100, 110, 120, 130, 140, 150]
        user_pitch = [100, 105, 110, 115, 120, 125, 130, 135, 140, 145, 150]

        result = calculate_dtw_alignment(user_pitch, ref_pitch)

        # DTW path should be monotonically increasing
        prev_ref = -1
        for i in range(len(user_pitch)):
            curr_ref = result['user_to_ref'][i]
            self.assertGreaterEqual(
                curr_ref,
                prev_ref,
                f"DTW alignment should be monotonic: frame {i} maps to {curr_ref} "
                f"but previous frame mapped to {prev_ref}"
            )
            prev_ref = curr_ref


class TestDTWPerformance(unittest.TestCase):
    """Performance tests for DTW alignment"""

    def test_alignment_speed(self):
        """Test DTW completes in reasonable time for typical audio"""
        import time

        # Typical case: 3 seconds of audio at 10ms hop = 300 frames
        ref_pitch = list(np.random.uniform(100, 300, 300))
        user_pitch = list(np.random.uniform(100, 300, 350))  # Slightly longer

        start = time.perf_counter()
        result = calculate_dtw_alignment(user_pitch, ref_pitch)
        elapsed = time.perf_counter() - start

        # Should complete in <1 second for 300 frames
        self.assertLess(
            elapsed,
            1.0,
            f"DTW took {elapsed:.3f}s for 300 frames (should be <1s)"
        )

    def test_long_sequence_performance(self):
        """Test DTW on longer sequences (10 seconds)"""
        import time

        # 10 seconds at 10ms hop = 1000 frames
        ref_pitch = list(np.random.uniform(100, 300, 1000))
        user_pitch = list(np.random.uniform(100, 300, 1200))

        start = time.perf_counter()
        result = calculate_dtw_alignment(user_pitch, ref_pitch)
        elapsed = time.perf_counter() - start

        # Should complete in reasonable time
        self.assertLess(
            elapsed,
            5.0,
            f"DTW took {elapsed:.3f}s for 1000 frames (should be <5s)"
        )

        # Check result is valid
        self.assertIn('user_to_ref', result)
        self.assertEqual(len(result['user_to_ref']), len(user_pitch))


class TestDTWAccuracy(unittest.TestCase):
    """Accuracy tests for DTW alignment quality"""

    def test_alignment_accuracy_on_perfect_match(self):
        """Test alignment accuracy when sequences match perfectly"""
        # Generate test pitch contour
        pitch = [100, 110, 120, 130, 140, 130, 120, 110, 100]

        result = calculate_dtw_alignment(pitch, pitch)

        # Perfect match should have 1:1 alignment
        accuracy = sum(1 for i, j in result['user_to_ref'].items() if i == j) / len(pitch)

        self.assertGreaterEqual(
            accuracy,
            0.95,
            f"Perfect match should have >95% 1:1 alignment, got {accuracy*100:.1f}%"
        )

    def test_alignment_accuracy_with_noise(self):
        """Test alignment robustness to noise"""
        # Reference pitch
        ref_pitch = [100, 110, 120, 130, 140, 130, 120, 110, 100]

        # User pitch with 5% noise
        user_pitch = [p + np.random.normal(0, 5) for p in ref_pitch]

        result = calculate_dtw_alignment(user_pitch, ref_pitch)

        # Should still achieve good alignment
        accuracy = sum(1 for i, j in result['user_to_ref'].items() if i == j) / len(ref_pitch)

        self.assertGreaterEqual(
            accuracy,
            0.6,
            f"Noisy match should have >60% 1:1 alignment, got {accuracy*100:.1f}%"
        )

    def test_dtw_cost_metric(self):
        """Test that DTW distance correlates with alignment quality"""
        ref_pitch = [100, 110, 120, 130, 140]

        # Perfect match
        user_perfect = [100, 110, 120, 130, 140]

        # Good match (small variations)
        user_good = [102, 108, 122, 128, 142]

        # Poor match (large variations)
        user_poor = [120, 130, 100, 150, 110]

        dist_perfect = calculate_dtw_alignment(user_perfect, ref_pitch)['distance']
        dist_good = calculate_dtw_alignment(user_good, ref_pitch)['distance']
        dist_poor = calculate_dtw_alignment(user_poor, ref_pitch)['distance']

        # Distances should correlate with quality
        self.assertLess(dist_perfect, dist_good, "Perfect match should have lower distance than good match")
        self.assertLess(dist_good, dist_poor, "Good match should have lower distance than poor match")


if __name__ == '__main__':
    # Run tests with verbose output
    unittest.main(verbosity=2)
