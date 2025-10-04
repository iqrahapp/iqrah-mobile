"""
Enhanced Online DTW for Real-Time Alignment
============================================

Online DTW with anchor-based drift correction, confidence gating,
and smooth lead/lag estimates for real-time recitation coaching.
"""

import numpy as np
from typing import Optional, List, Tuple
from dataclasses import dataclass
from collections import deque

from ..dtw import DTWAligner, AlignmentResult
from .anchors import Anchor


@dataclass
class OnlineAlignmentState:
    """
    Real-time alignment state.

    Contains current alignment info and streaming metrics.
    """
    # Alignment
    reference_position: int  # Current position in reference
    lead_lag_ms: float      # Lead (+) or lag (-) in milliseconds
    confidence: float        # Alignment confidence [0, 1]

    # Drift tracking
    drift_estimate: float    # Estimated drift in frames
    drift_confidence: float  # Confidence in drift estimate

    # Status
    status: str             # "tracking", "lost", "acquiring", "anchored"
    frames_since_anchor: int # Frames since last anchor correction

    # Latest alignment
    last_alignment: Optional[AlignmentResult] = None


class EnhancedOnlineDTW:
    """
    Enhanced online DTW with anchors and confidence gating.

    Features:
    - Anchor-based drift correction
    - Confidence gating (freeze when uncertain)
    - Exponential smoothing for lead/lag
    - Adaptive window sizing
    - Status tracking (tracking/lost/acquiring)

    Target: <10ms per update
    """

    def __init__(
        self,
        window_size: int = 300,      # ~3s at 100 Hz
        band_width: int = 50,        # Sakoe-Chiba band
        confidence_threshold: float = 0.6,  # Min confidence to update
        anchor_search_radius: int = 20,     # Frames to search around anchors
        lead_lag_smoothing: float = 0.3,    # EMA alpha for lead/lag
        sample_rate: int = 22050,
        hop_length: int = 512,
    ):
        """
        Initialize enhanced online DTW.

        Args:
            window_size: Sliding window size (frames)
            band_width: Sakoe-Chiba band width
            confidence_threshold: Minimum confidence for updates
            anchor_search_radius: Search radius around anchor positions
            lead_lag_smoothing: Smoothing factor (0=no smooth, 1=no update)
            sample_rate: Audio sample rate
            hop_length: Hop length for frame rate
        """
        self.window_size = window_size
        self.band_width = band_width
        self.confidence_threshold = confidence_threshold
        self.anchor_search_radius = anchor_search_radius
        self.lead_lag_smoothing = lead_lag_smoothing
        self.sample_rate = sample_rate
        self.hop_length = hop_length

        # Frame duration in milliseconds
        self.frame_duration_ms = (hop_length / sample_rate) * 1000

        # Buffers
        self.query_buffer = deque(maxlen=window_size)
        self.confidence_buffer = deque(maxlen=window_size)

        # State
        self.state = OnlineAlignmentState(
            reference_position=0,
            lead_lag_ms=0.0,
            confidence=0.0,
            drift_estimate=0.0,
            drift_confidence=0.0,
            status="acquiring",
            frames_since_anchor=0,
        )

        # Anchors
        self.reference_anchors: List[Anchor] = []
        self.query_anchors: List[Anchor] = []
        self.last_anchor_correction = 0

        # Base aligner
        self.aligner = DTWAligner(window=band_width)

        # History for smoothing
        self.lead_lag_history = deque(maxlen=10)
        self.position_history = deque(maxlen=10)

    def set_reference_anchors(self, anchors: List[Anchor]):
        """
        Set reference anchors for drift correction.

        Args:
            anchors: List of anchors detected in reference
        """
        # Sort by timestamp
        self.reference_anchors = sorted(anchors, key=lambda a: a.timestamp)

    def update(
        self,
        query_frame: float,
        query_confidence: float,
        reference: np.ndarray,
        query_anchor: Optional[Anchor] = None,
    ) -> OnlineAlignmentState:
        """
        Update alignment with new query frame.

        Args:
            query_frame: New pitch value
            query_confidence: Voicing confidence for this frame
            reference: Full reference pitch sequence
            query_anchor: Anchor detected in query (if any)

        Returns:
            Updated alignment state
        """
        # Add to buffer
        self.query_buffer.append(query_frame)
        self.confidence_buffer.append(query_confidence)

        # Increment frames since anchor
        self.state.frames_since_anchor += 1

        # Check if buffer is full enough
        if len(self.query_buffer) < min(50, self.window_size // 2):
            self.state.status = "acquiring"
            self.state.confidence = 0.0
            return self.state

        # Anchor correction if detected
        if query_anchor is not None:
            self._apply_anchor_correction(query_anchor, reference)

        # Define reference window
        ref_start = max(0, self.state.reference_position - self.band_width)
        ref_end = min(
            len(reference),
            self.state.reference_position + self.window_size + self.band_width
        )

        ref_window = reference[ref_start:ref_end]

        if len(ref_window) < 10:  # Too short
            self.state.status = "lost"
            self.state.confidence = 0.0
            return self.state

        # Perform DTW alignment
        query_array = np.array(list(self.query_buffer))
        result = self.aligner.align(query_array, ref_window)

        self.state.last_alignment = result

        # Calculate confidence (combine DTW score with voicing confidence)
        dtw_confidence = result.alignment_score
        voice_confidence = np.mean(list(self.confidence_buffer))
        combined_confidence = (dtw_confidence + voice_confidence) / 2

        # Confidence gating: only update if confidence is high
        if combined_confidence < self.confidence_threshold:
            # Low confidence - freeze position
            self.state.status = "lost"
            self.state.confidence = combined_confidence
            return self.state

        # Update reference position from alignment
        if result.path:
            # Find where current query position maps to in reference
            query_current = len(self.query_buffer) - 1

            # Find closest match in path
            best_ref = ref_start
            min_dist = float('inf')

            for q_idx, r_idx in result.path:
                dist = abs(q_idx - query_current)
                if dist < min_dist:
                    min_dist = dist
                    best_ref = ref_start + r_idx

            # Update position with drift correction
            new_position = best_ref + self.state.drift_estimate

            # Smooth position updates
            if len(self.position_history) > 0:
                alpha = 0.3  # Smoothing factor
                new_position = (
                    alpha * new_position +
                    (1 - alpha) * self.position_history[-1]
                )

            self.position_history.append(new_position)
            self.state.reference_position = int(new_position)

        # Calculate lead/lag
        # Positive = ahead of reference, Negative = behind
        expected_position = len(self.query_buffer) + ref_start
        raw_lead_lag_frames = self.state.reference_position - expected_position
        raw_lead_lag_ms = raw_lead_lag_frames * self.frame_duration_ms

        # Smooth lead/lag with exponential moving average
        if len(self.lead_lag_history) > 0:
            smoothed_lead_lag = (
                self.lead_lag_smoothing * self.lead_lag_history[-1] +
                (1 - self.lead_lag_smoothing) * raw_lead_lag_ms
            )
        else:
            smoothed_lead_lag = raw_lead_lag_ms

        self.lead_lag_history.append(smoothed_lead_lag)
        self.state.lead_lag_ms = smoothed_lead_lag

        # Update drift estimate (slow-moving average of position error)
        position_error = raw_lead_lag_frames
        self.state.drift_estimate = 0.95 * self.state.drift_estimate + 0.05 * position_error

        # Update confidence and status
        self.state.confidence = combined_confidence

        if combined_confidence > 0.8:
            self.state.status = "tracking"
        elif combined_confidence > self.confidence_threshold:
            self.state.status = "tracking"
        else:
            self.state.status = "lost"

        return self.state

    def _apply_anchor_correction(
        self,
        query_anchor: Anchor,
        reference: np.ndarray,
    ):
        """
        Apply anchor-based drift correction.

        Finds matching anchor in reference and corrects position.

        Args:
            query_anchor: Detected anchor in query
            reference: Reference sequence
        """
        if not self.reference_anchors:
            return

        # Find closest reference anchor by type and position
        best_ref_anchor = None
        min_distance = float('inf')

        # Current query time estimate
        query_time = len(self.query_buffer) * self.frame_duration_ms / 1000.0

        # Expected reference time based on current position
        expected_ref_time = self.state.reference_position * self.frame_duration_ms / 1000.0

        for ref_anchor in self.reference_anchors:
            # Prefer same type
            type_match = (ref_anchor.anchor_type == query_anchor.anchor_type)

            # Check if near expected position
            time_diff = abs(ref_anchor.timestamp - expected_ref_time)

            if type_match and time_diff < min_distance:
                min_distance = time_diff
                best_ref_anchor = ref_anchor

        # Apply correction if found good match
        if best_ref_anchor and min_distance < 5.0:  # Within 5 seconds
            # Calculate position from anchor
            anchor_ref_frame = int(
                best_ref_anchor.timestamp * self.sample_rate / self.hop_length
            )

            # Correct position
            self.state.reference_position = anchor_ref_frame

            # Reset drift estimate
            self.state.drift_estimate = 0.0
            self.state.drift_confidence = best_ref_anchor.confidence

            # Mark as anchored
            self.state.status = "anchored"
            self.state.frames_since_anchor = 0
            self.last_anchor_correction = len(self.query_buffer)

    def reset(self):
        """Reset aligner state."""
        self.query_buffer.clear()
        self.confidence_buffer.clear()
        self.lead_lag_history.clear()
        self.position_history.clear()

        self.state = OnlineAlignmentState(
            reference_position=0,
            lead_lag_ms=0.0,
            confidence=0.0,
            drift_estimate=0.0,
            drift_confidence=0.0,
            status="acquiring",
            frames_since_anchor=0,
        )

    def get_hints(self) -> dict:
        """
        Get real-time coaching hints.

        Returns:
            Dictionary with feedback for UI
        """
        hints = {
            "lead_lag_ms": int(self.state.lead_lag_ms),
            "confidence": self.state.confidence,
            "status": self.state.status,
            "reference_position": self.state.reference_position,
            "drift_estimate": self.state.drift_estimate,
        }

        # Add status-specific hints
        if self.state.status == "tracking":
            if abs(self.state.lead_lag_ms) < 100:
                hints["message"] = "Good timing"
            elif self.state.lead_lag_ms > 100:
                hints["message"] = f"Slow down ({int(self.state.lead_lag_ms)}ms ahead)"
            else:
                hints["message"] = f"Speed up ({int(abs(self.state.lead_lag_ms))}ms behind)"

        elif self.state.status == "lost":
            hints["message"] = "Re-acquiring alignment..."

        elif self.state.status == "acquiring":
            hints["message"] = "Starting..."

        elif self.state.status == "anchored":
            hints["message"] = "Position corrected"

        return hints
