"""
Live Feedback System
====================

Real-time coaching feedback generator with rate limiting and smoothing.

Generates actionable hints for UI display at 10-20 Hz.
"""

import time
import numpy as np
from typing import Optional, Dict, List
from dataclasses import dataclass, asdict
from collections import deque

from .online_dtw import OnlineAlignmentState


@dataclass
class RealtimeHints:
    """
    Real-time coaching hints for UI.

    Attributes:
        timestamp: Unix timestamp
        lead_lag_ms: Lead (+) or lag (-) in milliseconds
        pitch_error_cents: Current pitch error in cents
        on_note: Whether pitch is within acceptable range
        confidence: Overall confidence [0, 1]
        status: Current status (good, warning, error, acquiring)
        message: Human-readable coaching message
        visual_cue: Suggested visual feedback (color/icon)
        current_pitch_hz: Current user pitch in Hz (for visualization)
    """
    timestamp: float
    lead_lag_ms: int
    pitch_error_cents: float
    on_note: bool
    confidence: float
    status: str  # "good", "warning", "error", "acquiring"
    message: str
    visual_cue: str  # "green", "yellow", "red", "gray"
    current_pitch_hz: float = 0.0  # Current user pitch for visualization

    # Optional detailed info
    reference_position: Optional[int] = None
    drift_estimate: Optional[float] = None
    frames_processed: Optional[int] = None

    def to_dict(self) -> dict:
        """Convert to dictionary for JSON serialization."""
        return asdict(self)


class LiveFeedback:
    """
    Generate real-time coaching feedback.

    Features:
    - Rate limiting (10-20 Hz max)
    - Exponential smoothing for stable feedback
    - Priority-based message generation
    - Status determination
    - Visual cue suggestions

    Target: Actionable feedback every 50-100ms
    """

    def __init__(
        self,
        update_rate_hz: float = 15.0,  # Target update rate
        on_note_threshold_cents: float = 50.0,  # ±50 cents = acceptable
        warning_threshold_cents: float = 100.0,  # ±100 cents = warning
        smoothing_alpha: float = 0.3,  # EMA smoothing (0=no smooth, 1=no update)
        lead_lag_threshold_ms: float = 200.0,  # Lead/lag warning threshold
    ):
        """
        Initialize live feedback generator.

        Args:
            update_rate_hz: Maximum update rate (10-20 Hz recommended)
            on_note_threshold_cents: Pitch error threshold for "on note"
            warning_threshold_cents: Pitch error threshold for warning
            smoothing_alpha: Exponential smoothing factor
            lead_lag_threshold_ms: Lead/lag threshold for warnings
        """
        self.update_rate_hz = update_rate_hz
        self.min_update_interval = 1.0 / update_rate_hz
        self.on_note_threshold_cents = on_note_threshold_cents
        self.warning_threshold_cents = warning_threshold_cents
        self.smoothing_alpha = smoothing_alpha
        self.lead_lag_threshold_ms = lead_lag_threshold_ms

        # State
        self.last_update_time = 0.0
        self.frames_processed = 0

        # Smoothing buffers
        self.pitch_error_history = deque(maxlen=5)
        self.lead_lag_history = deque(maxlen=5)
        self.confidence_history = deque(maxlen=5)

        # Last hints (for comparison)
        self.last_hints: Optional[RealtimeHints] = None

    def generate_hints(
        self,
        alignment_state: OnlineAlignmentState,
        current_pitch_hz: float,
        current_confidence: float,
        reference_pitch_hz: np.ndarray,
    ) -> Optional[RealtimeHints]:
        """
        Generate real-time coaching hints.

        Args:
            alignment_state: Current alignment state from OnlineDTW
            current_pitch_hz: Current user pitch in Hz
            current_confidence: Current voicing confidence
            reference_pitch_hz: Reference pitch contour

        Returns:
            RealtimeHints if update is needed, None if rate-limited
        """
        current_time = time.time()

        # Rate limiting: check if enough time has passed
        time_since_last = current_time - self.last_update_time
        if time_since_last < self.min_update_interval:
            return None  # Skip this update

        self.last_update_time = current_time
        self.frames_processed += 1

        # Calculate pitch error
        pitch_error_cents = self._calculate_pitch_error(
            current_pitch_hz,
            reference_pitch_hz,
            alignment_state.reference_position,
        )

        # Smooth pitch error
        self.pitch_error_history.append(pitch_error_cents)
        smoothed_pitch_error = self._smooth_value(
            pitch_error_cents,
            list(self.pitch_error_history)
        )

        # Smooth lead/lag
        self.lead_lag_history.append(alignment_state.lead_lag_ms)
        smoothed_lead_lag = self._smooth_value(
            alignment_state.lead_lag_ms,
            list(self.lead_lag_history)
        )

        # Smooth confidence
        combined_confidence = (alignment_state.confidence + current_confidence) / 2
        self.confidence_history.append(combined_confidence)
        smoothed_confidence = self._smooth_value(
            combined_confidence,
            list(self.confidence_history)
        )

        # Determine if on note
        on_note = abs(smoothed_pitch_error) < self.on_note_threshold_cents

        # Determine status and visual cue
        status, visual_cue = self._determine_status(
            alignment_state=alignment_state,
            pitch_error=smoothed_pitch_error,
            lead_lag=smoothed_lead_lag,
            confidence=smoothed_confidence,
            on_note=on_note,
        )

        # Generate coaching message
        message = self._generate_message(
            status=status,
            alignment_state=alignment_state,
            pitch_error=smoothed_pitch_error,
            lead_lag=smoothed_lead_lag,
            on_note=on_note,
        )

        # Create hints
        hints = RealtimeHints(
            timestamp=current_time,
            lead_lag_ms=int(smoothed_lead_lag),
            pitch_error_cents=smoothed_pitch_error,
            on_note=on_note,
            confidence=smoothed_confidence,
            status=status,
            message=message,
            visual_cue=visual_cue,
            current_pitch_hz=current_pitch_hz,  # Include current pitch for visualization
            reference_position=alignment_state.reference_position,
            drift_estimate=alignment_state.drift_estimate,
            frames_processed=self.frames_processed,
        )

        self.last_hints = hints
        return hints

    def _calculate_pitch_error(
        self,
        user_pitch_hz: float,
        reference_pitch_hz: np.ndarray,
        reference_position: int,
    ) -> float:
        """
        Calculate pitch error in cents.

        Args:
            user_pitch_hz: Current user pitch
            reference_pitch_hz: Reference pitch contour
            reference_position: Current position in reference

        Returns:
            Pitch error in cents
        """
        # Bounds check
        if reference_position < 0 or reference_position >= len(reference_pitch_hz):
            return 0.0

        ref_pitch = reference_pitch_hz[reference_position]

        # Check for unvoiced
        if user_pitch_hz <= 0 or ref_pitch <= 0:
            return 0.0

        # Calculate error in cents
        error_cents = 1200 * np.log2(user_pitch_hz / ref_pitch)

        return float(error_cents)

    def _smooth_value(self, current_value: float, history: List[float]) -> float:
        """
        Apply exponential moving average smoothing.

        Args:
            current_value: Current value
            history: Historical values

        Returns:
            Smoothed value
        """
        if len(history) <= 1:
            return current_value

        # Exponential moving average
        prev_smoothed = history[-2] if len(history) >= 2 else current_value
        smoothed = (
            self.smoothing_alpha * prev_smoothed +
            (1 - self.smoothing_alpha) * current_value
        )

        return smoothed

    def _determine_status(
        self,
        alignment_state: OnlineAlignmentState,
        pitch_error: float,
        lead_lag: float,
        confidence: float,
        on_note: bool,
    ) -> tuple[str, str]:
        """
        Determine overall status and visual cue.

        Args:
            alignment_state: Alignment state
            pitch_error: Smoothed pitch error
            lead_lag: Smoothed lead/lag
            confidence: Smoothed confidence
            on_note: Whether pitch is acceptable

        Returns:
            (status, visual_cue) tuple
        """
        # Priority 1: Low confidence / lost tracking
        if confidence < 0.3 or alignment_state.status == "lost":
            return "error", "red"

        # Priority 2: Acquiring
        if alignment_state.status == "acquiring":
            return "acquiring", "gray"

        # Priority 3: Large pitch error
        if abs(pitch_error) > self.warning_threshold_cents:
            return "warning", "yellow"

        # Priority 4: Large lead/lag
        if abs(lead_lag) > self.lead_lag_threshold_ms:
            return "warning", "yellow"

        # Priority 5: Not on note
        if not on_note:
            return "warning", "yellow"

        # All good!
        return "good", "green"

    def _generate_message(
        self,
        status: str,
        alignment_state: OnlineAlignmentState,
        pitch_error: float,
        lead_lag: float,
        on_note: bool,
    ) -> str:
        """
        Generate human-readable coaching message.

        Args:
            status: Overall status
            alignment_state: Alignment state
            pitch_error: Smoothed pitch error
            lead_lag: Smoothed lead/lag
            on_note: Whether pitch is acceptable

        Returns:
            Coaching message
        """
        # Error states
        if status == "error":
            return "Lost tracking - please continue reciting"

        if status == "acquiring":
            return "Starting analysis..."

        # Warning states - prioritize most important issue
        if status == "warning":
            # Check pitch first
            if abs(pitch_error) > self.warning_threshold_cents:
                if pitch_error > 0:
                    return f"Pitch too high ({int(abs(pitch_error))} cents)"
                else:
                    return f"Pitch too low ({int(abs(pitch_error))} cents)"

            # Then timing
            if abs(lead_lag) > self.lead_lag_threshold_ms:
                if lead_lag > 0:
                    return f"Slow down ({int(abs(lead_lag))}ms ahead)"
                else:
                    return f"Speed up ({int(abs(lead_lag))}ms behind)"

            # Gentle nudge for off-note
            if not on_note:
                if abs(pitch_error) < 10:
                    return "Good pitch"
                elif pitch_error > 0:
                    return "Slightly high"
                else:
                    return "Slightly low"

        # Good state - provide encouragement or fine-tuning
        if status == "good":
            # Perfect
            if abs(pitch_error) < 10 and abs(lead_lag) < 50:
                return "Excellent!"

            # Very good
            if abs(pitch_error) < 25 and abs(lead_lag) < 100:
                return "Very good"

            # Good
            return "Good"

        return "Keep going"

    def reset(self):
        """Reset feedback state."""
        self.last_update_time = 0.0
        self.frames_processed = 0
        self.pitch_error_history.clear()
        self.lead_lag_history.clear()
        self.confidence_history.clear()
        self.last_hints = None

    def get_update_rate(self) -> float:
        """
        Get actual update rate.

        Returns:
            Updates per second
        """
        if self.last_update_time == 0:
            return 0.0

        elapsed = time.time() - self.last_update_time
        if elapsed > 0:
            return 1.0 / elapsed

        return 0.0
