"""
True Online DTW (OLTW) - Maximum Precision Implementation
==========================================================

This is a refactored OLTW implementation optimized for:
- Maximum alignment precision with delta-pitch features
- Stability under tempo variation with adaptive bands
- Robustness in silence/unvoiced regions
- Real-time O(W) performance with optimizations

Key Improvements over v2:
1. Cents + Δpitch domain for transposition invariance
2. Huber loss for robust local distance
3. Adaptive Sakoe-Chiba window (symmetric ↔ forward-biased)
4. Freeze-on-silence (no drift during unvoiced frames)
5. EMA confidence smoothing
6. Moderate off-diagonal penalty (~1.2)
7. float32 arrays + finite values for performance
8. Correct cumulative cost (path cost, not accumulated)

References:
- Dixon, S. (2005). "On-Line Time Warping"
- Sakoe & Chiba (1978). "Dynamic programming algorithm optimization"
- Huber, P. J. (1964). "Robust Estimation"
"""

import numpy as np
from typing import Optional, Tuple, List, Deque
from dataclasses import dataclass
from collections import deque

# Import for compatibility
from .online_dtw import OnlineAlignmentState


@dataclass
class OLTWState:
    """State for Online Time Warping."""
    reference_position: int  # Current best match position in reference
    cumulative_cost: float  # Current path cost (not accumulated!)
    path: List[Tuple[int, int]]  # Alignment path (query_idx, ref_idx)
    confidence: float  # Alignment confidence [0, 1]
    is_tracking: bool  # Whether actively tracking
    drift_estimate: float = 0.0  # Estimated drift in frames
    frames_processed: int = 0  # Number of query frames processed
    tempo_estimate: float = 1.0  # Estimated tempo ratio vs reference


class TrueOnlineDTW:
    """
    Maximum-precision True Online DTW with delta-pitch features.

    Performance & Complexity:
    - Time: O(W) per frame where W = adaptive window size (~120-160)
    - Memory: O(N) where N = reference length (2 float32 columns)
    - Latency: <1ms per frame typical

    Domain: Cents + Δpitch for transposition invariance
    Distance: Huber loss for robustness to noise
    Window: Adaptive (symmetric when uncertain, forward-biased when stable)
    Silence: Freeze position (no drift)
    """

    def __init__(
        self,
        reference: np.ndarray,
        sample_rate: int = 22050,
        hop_length: int = 512,
        base_window_size: int = 150,  # Base Sakoe-Chiba window
        off_diagonal_penalty: float = 2.5,  # Strong diagonal preference
    ):
        """
        Initialize True Online DTW with maximum precision settings.

        Args:
            reference: Reference pitch sequence (f0 in Hz)
            sample_rate: Audio sample rate
            hop_length: Hop length in samples
            base_window_size: Base Sakoe-Chiba window size (adaptive)
            off_diagonal_penalty: Penalty for non-diagonal moves [1.0-1.6]
        """
        self.sample_rate = sample_rate
        self.hop_length = hop_length
        self.base_window_size = base_window_size
        self.off_diagonal_penalty = off_diagonal_penalty

        # Frame duration for timing
        self.frame_duration_ms = (hop_length / sample_rate) * 1000

        # Convert reference to delta-pitch in cents domain
        self.reference_cents = self._to_cents(reference)
        self.reference_feat = self._to_delta_pitch(self.reference_cents)
        self.n_reference = len(self.reference_feat)

        # OLTW columns (use float32 for performance)
        self.cost_column = np.full(self.n_reference, 1e12, dtype=np.float32)
        self.prev_column = np.full(self.n_reference, 1e12, dtype=np.float32)

        # State
        self.state = OLTWState(
            reference_position=0,
            cumulative_cost=0.0,
            path=[],
            confidence=0.0,
            is_tracking=False,
        )

        # Query history for delta-pitch computation
        self.query_history = deque(maxlen=100)

        # Reference position history for adaptive window
        self._last_refs = deque(maxlen=15)

        # EMA confidence tracker
        self._ema_conf = None

        print(f"✓ TrueOnlineDTW (v3) initialized: {self.n_reference} reference frames")
        print(f"  Domain: Δpitch in cents")
        print(f"  Window: adaptive {base_window_size}±50")
        print(f"  Penalty: {off_diagonal_penalty}")

    def _to_cents(self, f0_hz: np.ndarray) -> np.ndarray:
        """Convert pitch from Hz to cents (log scale)."""
        # Reference: A0 = 55 Hz
        # Cents = 1200 * log2(f / 55)
        f0_safe = np.maximum(f0_hz, 1e-3)
        return 1200.0 * np.log2(f0_safe / 55.0)

    def _to_delta_pitch(self, cents: np.ndarray) -> np.ndarray:
        """
        Convert pitch contour to delta-pitch (first difference).

        Δpitch is transposition-invariant and captures melodic contour.
        """
        if len(cents) < 2:
            return np.array([0.0], dtype=np.float32)

        delta = np.diff(cents).astype(np.float32)
        # Pad to maintain length
        return np.concatenate([[0.0], delta])

    def _huber_distance(self, x: float, delta: float = 10.0) -> float:
        """
        Huber loss for robust distance calculation.

        Tolerates small deviations (quadratic) but limits large jumps (linear).

        Args:
            x: Raw difference
            delta: Threshold for switching quadratic→linear
        """
        ax = abs(x)
        if ax <= delta:
            return 0.5 * ax * ax
        else:
            return delta * (ax - 0.5 * delta)

    def _confidence_weight(self, query_confidence: float) -> float:
        """
        Map query confidence [0,1] to distance weight.

        Returns weight in [0.6, 1.0] - reduces but doesn't eliminate
        low-confidence frames.
        """
        c = np.clip(query_confidence, 0.0, 1.0)
        return 0.6 + 0.4 * c

    def seed(self, initial_query: np.ndarray, force_position: Optional[int] = None) -> int:
        """
        Seed alignment using subsequence search in delta-pitch space.

        Args:
            initial_query: Initial pitch frames (Hz)
            force_position: Force seed at exact position (for self-alignment)

        Returns:
            Best starting index in reference
        """
        if len(initial_query) < 10:
            print("⚠ Warning: Very short seeding query")

        # Populate query history
        self.query_history.extend(initial_query.tolist())

        # Convert to delta-pitch features
        query_cents = self._to_cents(initial_query)
        query_feat = self._to_delta_pitch(query_cents)
        query_len = len(query_feat)

        if force_position is not None:
            best_idx = force_position
            best_dist = 0.0
            print(f"✓ Forced seed at reference position {best_idx}")
        else:
            # Subsequence search in delta-pitch space
            best_dist = np.inf
            best_idx = 0

            for i in range(self.n_reference - query_len):
                ref_window = self.reference_feat[i:i + query_len]
                # Euclidean distance in delta-pitch space
                dist = np.sqrt(np.sum((query_feat - ref_window) ** 2))

                if dist < best_dist:
                    best_dist = dist
                    best_idx = i

            print(f"✓ Seeded at reference position {best_idx} (distance: {best_dist:.3f})")

        # Initialize cost columns
        self.prev_column[:] = 1e12
        self.cost_column[:] = 1e12

        # Seed position has zero cost
        self.prev_column[best_idx] = 0.0
        self.cost_column[best_idx] = 0.0

        # Set state
        self.state.reference_position = best_idx
        self.state.is_tracking = True
        self.state.path = [(0, best_idx)]
        self._last_refs.append(best_idx)

        return best_idx

    def update(self, query_frame: float, query_confidence: float = 1.0) -> OLTWState:
        """
        Update alignment with new query frame (maximum precision).

        Args:
            query_frame: New pitch value (Hz)
            query_confidence: Voicing confidence [0, 1]

        Returns:
            Updated OLTWState
        """
        if not self.state.is_tracking:
            raise RuntimeError("OLTW not seeded. Call seed() first.")

        # === FREEZE ON SILENCE/UNVOICED ===
        # Never drift during silence - this is critical for precision
        if query_confidence < 0.1:
            self.state.frames_processed += 1
            # Gentle confidence decay
            if self._ema_conf is not None:
                self._ema_conf *= 0.98
                self.state.confidence = float(np.clip(self._ema_conf, 0.0, 1.0))
            # Add to path but don't advance reference
            self.state.path.append((self.state.frames_processed - 1, self.state.reference_position))
            return self.state

        # === COMPUTE QUERY DELTA-PITCH FEATURE ===
        self.query_history.append(query_frame)

        # Need at least 2 frames for delta
        if len(self.query_history) < 2:
            query_feat = 0.0
        else:
            # Get last 2 frames, convert to cents, compute delta
            recent = list(self.query_history)[-2:]
            cents = self._to_cents(np.array(recent))
            query_feat = float(cents[1] - cents[0])

        # Confidence weighting
        conf_weight = self._confidence_weight(query_confidence)

        # === ADAPTIVE SAKOE-CHIBA WINDOW ===
        center = self.state.reference_position

        # Calculate tempo estimate from recent history
        if len(self._last_refs) > 1:
            drefs = np.diff(list(self._last_refs))
            tempo = np.median(drefs) if len(drefs) > 0 else 1.0
        else:
            tempo = 1.0

        self.state.tempo_estimate = float(tempo)

        # Adaptive window sizing
        W = self.base_window_size

        if self.state.confidence < 0.6:
            # Symmetric during acquisition/uncertainty
            back, fwd = W // 2, W - W // 2
        elif tempo < 0.9:
            # User slower - bias backward
            back, fwd = int(0.6 * W), W - int(0.6 * W)
        elif tempo > 1.1:
            # User faster - bias forward
            back, fwd = int(0.3 * W), W - int(0.3 * W)
        else:
            # Stable tempo - slight forward bias
            back, fwd = int(0.4 * W), W - int(0.4 * W)

        window_start = max(0, center - back)
        window_end = min(self.n_reference, center + fwd)

        # === SWAP COLUMNS ===
        self.prev_column, self.cost_column = self.cost_column, self.prev_column
        self.cost_column[:] = 1e12

        # === DTW UPDATE LOOP WITH HUBER DISTANCE ===
        INF = 1e12
        OFF = self.off_diagonal_penalty

        for j in range(window_start, window_end):
            # Local distance with Huber loss
            raw_diff = query_feat - self.reference_feat[j]
            local_dist = self._huber_distance(raw_diff, delta=10.0)

            # Apply confidence weighting
            local_dist /= conf_weight

            # DTW recurrence with moderate off-diagonal penalty
            cost_diag = self.prev_column[j - 1] if j > window_start else INF
            cost_vert = self.prev_column[j] + OFF
            cost_horiz = self.cost_column[j - 1] + OFF if j > window_start else INF

            self.cost_column[j] = local_dist + min(cost_diag, cost_vert, cost_horiz)

        # === FIND BEST POSITION ===
        valid_costs = self.cost_column[window_start:window_end]

        if len(valid_costs) == 0 or np.all(valid_costs >= 1e11):
            # Lost tracking
            self.state.is_tracking = False
            self.state.confidence = 0.0
            return self.state

        best_local_idx = np.argmin(valid_costs)
        best_ref_idx = window_start + best_local_idx
        best_cost = float(valid_costs[best_local_idx])

        # === UPDATE STATE ===
        self.state.frames_processed += 1
        self.state.reference_position = best_ref_idx

        # Cumulative cost IS the current path cost (not accumulated!)
        self.state.cumulative_cost = best_cost

        # === CONFIDENCE WITH EMA SMOOTHING ===
        # Use local match quality only
        raw_diff = query_feat - self.reference_feat[best_ref_idx]
        local_match_quality = abs(raw_diff)

        # Instantaneous confidence
        conf_local = 1.0 / (1.0 + local_match_quality / 10.0)  # Scale by 10 for cents

        # EMA smoothing (α=0.15)
        if self._ema_conf is None:
            self._ema_conf = conf_local
        else:
            self._ema_conf = 0.85 * self._ema_conf + 0.15 * conf_local

        self.state.confidence = float(np.clip(self._ema_conf, 0.0, 1.0))

        # === UPDATE PATH AND HISTORY ===
        query_idx = self.state.frames_processed - 1
        self.state.path.append((query_idx, best_ref_idx))

        # Keep path manageable
        if len(self.state.path) > 1000:
            self.state.path = self.state.path[-1000:]

        # Update reference history for tempo estimation
        self._last_refs.append(best_ref_idx)

        # Drift estimate
        expected_ref_pos = query_idx
        self.state.drift_estimate = float(best_ref_idx - expected_ref_pos)

        return self.state

    def get_lead_lag_ms(self) -> float:
        """Calculate lead/lag in milliseconds."""
        if not self.state.path:
            return 0.0

        query_idx, ref_idx = self.state.path[-1]
        lag_frames = ref_idx - query_idx
        return lag_frames * self.frame_duration_ms

    def reset(self):
        """Reset alignment state."""
        self.cost_column[:] = 1e12
        self.prev_column[:] = 1e12
        self.query_history.clear()
        self._last_refs.clear()
        self._ema_conf = None

        self.state = OLTWState(
            reference_position=0,
            cumulative_cost=0.0,
            path=[],
            confidence=0.0,
            is_tracking=False,
        )

        print("✓ OLTW (v3) reset")


class OLTWAligner:
    """
    High-level wrapper for maximum-precision OLTW.

    Compatible with existing pipeline infrastructure.
    """

    def __init__(
        self,
        reference: np.ndarray,
        sample_rate: int = 22050,
        hop_length: int = 512,
        seed_buffer_frames: int = 50,
        force_seed_position: Optional[int] = None,
    ):
        """Initialize OLTW aligner (v3 - maximum precision)."""
        self.oltw = TrueOnlineDTW(
            reference=reference,
            sample_rate=sample_rate,
            hop_length=hop_length,
            base_window_size=150,  # Optimal for precision-speed tradeoff
            off_diagonal_penalty=2.5,  # Strong diagonal preference
        )

        self.seed_buffer_frames = seed_buffer_frames
        self.seed_buffer = []
        self.is_seeded = False
        self.force_seed_position = force_seed_position
        self.total_frames = 0

        # Compatibility state
        self.state = OnlineAlignmentState(
            reference_position=0,
            lead_lag_ms=0.0,
            confidence=0.0,
            drift_estimate=0.0,
            drift_confidence=0.0,
            status="acquiring",
            frames_since_anchor=0,
        )

    def update(
        self,
        query_frame: float,
        query_confidence: float,
        reference: Optional[np.ndarray] = None,
        query_anchor: Optional[object] = None,
    ) -> OnlineAlignmentState:
        """Update alignment (pipeline-compatible interface)."""
        self.total_frames += 1

        # Seeding phase
        if not self.is_seeded:
            self.seed_buffer.append(query_frame)

            if len(self.seed_buffer) >= self.seed_buffer_frames:
                seed_array = np.array(self.seed_buffer)
                self.oltw.seed(seed_array, force_position=self.force_seed_position)
                self.is_seeded = True
                self.seed_buffer = []

            self.state = OnlineAlignmentState(
                reference_position=0,
                lead_lag_ms=0.0,
                confidence=0.0,
                drift_estimate=0.0,
                drift_confidence=0.0,
                status="acquiring",
                frames_since_anchor=self.total_frames,
            )
            return self.state

        # Normal update
        oltw_state = self.oltw.update(query_frame, query_confidence)

        # Convert to pipeline state
        status = "tracking" if oltw_state.is_tracking else "lost"
        lead_lag_ms = self.oltw.get_lead_lag_ms()

        self.state = OnlineAlignmentState(
            reference_position=oltw_state.reference_position,
            lead_lag_ms=lead_lag_ms,
            confidence=oltw_state.confidence,
            drift_estimate=float(oltw_state.drift_estimate),
            drift_confidence=oltw_state.confidence,
            status=status,
            frames_since_anchor=0,
        )
        return self.state

    def reset(self):
        """Reset for new recitation."""
        self.oltw.reset()
        self.seed_buffer = []
        self.is_seeded = False
        self.total_frames = 0
