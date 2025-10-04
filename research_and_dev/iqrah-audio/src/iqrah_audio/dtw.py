"""
Dynamic Time Warping (DTW) Alignment Module
===========================================

Fast DTW implementation for aligning user recitation to reference.
Supports both offline (full) and online (streaming) alignment.
"""

import numpy as np
from dataclasses import dataclass
from typing import Optional, Tuple
from dtaidistance import dtw
from dtaidistance import dtw_ndim


@dataclass
class AlignmentResult:
    """
    DTW alignment result.

    Attributes:
        distance: DTW distance (lower = better alignment)
        path: Alignment path as list of (query_idx, reference_idx) pairs
        normalized_distance: Distance normalized by path length
        alignment_score: Similarity score [0, 1] (1 = perfect)
    """
    distance: float
    path: list[Tuple[int, int]]
    normalized_distance: float
    alignment_score: float

    @property
    def query_indices(self) -> np.ndarray:
        """Query indices from alignment path."""
        return np.array([p[0] for p in self.path])

    @property
    def reference_indices(self) -> np.ndarray:
        """Reference indices from alignment path."""
        return np.array([p[1] for p in self.path])

    def get_aligned_segments(self) -> list[Tuple[int, int, int, int]]:
        """
        Get aligned segments as (q_start, q_end, r_start, r_end).

        Returns segments where alignment is continuous.
        """
        segments = []
        if not self.path:
            return segments

        q_start, r_start = self.path[0]
        prev_q, prev_r = self.path[0]

        for i in range(1, len(self.path)):
            q, r = self.path[i]

            # Check for discontinuity
            if q != prev_q + 1 or r != prev_r + 1:
                # Save previous segment
                segments.append((q_start, prev_q, r_start, prev_r))
                q_start, r_start = q, r

            prev_q, prev_r = q, r

        # Add final segment
        segments.append((q_start, prev_q, r_start, prev_r))

        return segments


class DTWAligner:
    """
    DTW aligner for pitch contour comparison.

    Uses fast C implementation from dtaidistance library.
    """

    def __init__(
        self,
        window: Optional[int] = None,  # Sakoe-Chiba band (None = no constraint)
        psi: Optional[int] = None,  # Psi relaxation at borders
    ):
        """
        Initialize DTW aligner.

        Args:
            window: Sakoe-Chiba band width (constrains warping)
                   None = no constraint, 50 = ±50 frame window
            psi: Psi relaxation (allows partial matching at borders)
        """
        self.window = window
        self.psi = psi

    def align(
        self,
        query: np.ndarray,
        reference: np.ndarray,
        use_c: bool = True
    ) -> AlignmentResult:
        """
        Align query to reference using DTW.

        Args:
            query: Query sequence (user pitch contour)
            reference: Reference sequence (qari pitch contour)
            use_c: Use fast C implementation (recommended)

        Returns:
            AlignmentResult with distance, path, and score
        """
        # Ensure 1D arrays
        query = np.asarray(query, dtype=np.float64).flatten()
        reference = np.asarray(reference, dtype=np.float64).flatten()

        # Compute DTW distance with path
        distance, paths = dtw.warping_paths(
            query,
            reference,
            window=self.window,
            psi=self.psi,
            use_c=use_c
        )

        # Extract best path
        path = dtw.best_path(paths)

        # Normalize distance
        path_length = len(path)
        normalized_distance = distance / path_length if path_length > 0 else float('inf')

        # Convert to similarity score [0, 1]
        # Use exponential decay: score = exp(-k * normalized_distance)
        k = 0.1  # Tuning parameter (higher = stricter scoring)
        alignment_score = np.exp(-k * normalized_distance)

        return AlignmentResult(
            distance=float(distance),
            path=[(int(q), int(r)) for q, r in path],
            normalized_distance=float(normalized_distance),
            alignment_score=float(alignment_score)
        )

    def align_multivariate(
        self,
        query: np.ndarray,
        reference: np.ndarray,
        weights: Optional[np.ndarray] = None
    ) -> AlignmentResult:
        """
        Align multi-dimensional features (e.g., pitch + energy + timbre).

        Args:
            query: Query features, shape (n_frames, n_features)
            reference: Reference features, shape (m_frames, n_features)
            weights: Feature weights, shape (n_features,)

        Returns:
            AlignmentResult
        """
        query = np.asarray(query, dtype=np.float64)
        reference = np.asarray(reference, dtype=np.float64)

        if query.ndim == 1:
            query = query[:, np.newaxis]
        if reference.ndim == 1:
            reference = reference[:, np.newaxis]

        # Apply feature weights if provided
        if weights is not None:
            weights = np.asarray(weights, dtype=np.float64)
            query = query * weights
            reference = reference * weights

        # Compute DTW with ndim support
        distance = dtw_ndim.distance(
            query,
            reference,
            window=self.window,
            psi=self.psi
        )

        # Note: dtw_ndim doesn't return path, so we use distance only
        # For path, would need to fall back to 1D or implement custom
        path_length = max(len(query), len(reference))
        normalized_distance = distance / path_length

        k = 0.1
        alignment_score = np.exp(-k * normalized_distance)

        return AlignmentResult(
            distance=float(distance),
            path=[],  # Not available for multivariate
            normalized_distance=float(normalized_distance),
            alignment_score=float(alignment_score)
        )

    def find_best_window(
        self,
        query: np.ndarray,
        reference: np.ndarray,
        max_offset: int = 100
    ) -> Tuple[int, float]:
        """
        Find best alignment window within reference.

        Useful for finding where in a long reference the query matches best.

        Args:
            query: Query sequence
            reference: Long reference sequence
            max_offset: Maximum offset to search

        Returns:
            (best_offset, best_score) tuple
        """
        query_len = len(query)
        best_offset = 0
        best_score = 0.0

        for offset in range(0, len(reference) - query_len, max(1, max_offset // 10)):
            ref_window = reference[offset:offset + query_len + max_offset]

            if len(ref_window) < query_len:
                break

            result = self.align(query, ref_window)

            if result.alignment_score > best_score:
                best_score = result.alignment_score
                best_offset = offset

        return best_offset, best_score


class OnlineDTWAligner:
    """
    Online (streaming) DTW for real-time alignment.

    Maintains a sliding window and updates alignment incrementally.
    """

    def __init__(
        self,
        window_size: int = 300,  # ~3s at 100 Hz
        band_width: int = 50,  # Sakoe-Chiba band
        hop_size: int = 1  # Frames to advance per update
    ):
        """
        Initialize online DTW aligner.

        Args:
            window_size: Size of sliding window (frames)
            band_width: Sakoe-Chiba band width
            hop_size: Hop size for updates
        """
        self.window_size = window_size
        self.band_width = band_width
        self.hop_size = hop_size

        self.query_buffer = []
        self.reference_idx = 0
        self.last_result: Optional[AlignmentResult] = None

    def update(
        self,
        query_frame: float,
        reference: np.ndarray
    ) -> Optional[AlignmentResult]:
        """
        Update alignment with new query frame.

        Args:
            query_frame: New query pitch value
            reference: Full reference sequence

        Returns:
            AlignmentResult if window is full, None otherwise
        """
        self.query_buffer.append(query_frame)

        # Keep only window_size frames
        if len(self.query_buffer) > self.window_size:
            self.query_buffer.pop(0)

        # Only align if buffer is full
        if len(self.query_buffer) < self.window_size:
            return None

        # Define reference window around current position
        ref_start = max(0, self.reference_idx - self.band_width)
        ref_end = min(len(reference), self.reference_idx + self.window_size + self.band_width)

        ref_window = reference[ref_start:ref_end]

        # Align
        aligner = DTWAligner(window=self.band_width)
        result = aligner.align(
            np.array(self.query_buffer),
            ref_window
        )

        # Update reference index based on alignment
        if result.path:
            # Move reference index to end of aligned segment
            _, last_ref_idx = result.path[-1]
            self.reference_idx = ref_start + last_ref_idx

        self.last_result = result
        return result

    def reset(self):
        """Reset aligner state."""
        self.query_buffer = []
        self.reference_idx = 0
        self.last_result = None
