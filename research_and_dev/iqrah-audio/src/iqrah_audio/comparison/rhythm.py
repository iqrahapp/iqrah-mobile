"""
Rhythm Comparison with Soft-DTW Divergence
===========================================

Implements tempo-invariant rhythm comparison using Soft-DTW divergence
with Sakoe-Chiba band constraint.

Based on: "Differentiable Divergences Between Time Series"
https://proceedings.mlr.press/v130/blondel21a/blondel21a.pdf
"""

import numpy as np
import torch
from typing import Tuple, Optional
from .features import FeaturePack, build_multi_feature_stack


def soft_dtw_divergence(
    x: np.ndarray,
    y: np.ndarray,
    gamma: float = 0.1,
    bandwidth: Optional[int] = None
) -> Tuple[float, np.ndarray]:
    """
    Compute Soft-DTW divergence between two sequences.

    Divergence = 2*SoftDTW(x,y) - SoftDTW(x,x) - SoftDTW(y,y)

    This gives a proper similarity measure (symmetric, unbiased).

    Args:
        x: Sequence 1 [T1, D]
        y: Sequence 2 [T2, D]
        gamma: Soft-min temperature (0.1-0.3 recommended)
        bandwidth: Sakoe-Chiba band width (e.g., 10-15% of max(T1,T2))

    Returns:
        divergence: Scalar divergence value
        path: Approximate alignment path (from soft-dtw)
    """
    # Convert to torch tensors
    x_t = torch.from_numpy(x).float()
    y_t = torch.from_numpy(y).float()

    # Compute pairwise distance matrix (squared Euclidean)
    cost_xy = pairwise_euclidean(x_t, y_t)
    cost_xx = pairwise_euclidean(x_t, x_t)
    cost_yy = pairwise_euclidean(y_t, y_t)

    # CRITICAL FIX: Apply Sakoe-Chiba band to ALL cost matrices
    # If we only band xy, then xx/yy stay artificially small → divergence explodes
    if bandwidth is not None:
        T1, T2 = x_t.size(0), y_t.size(0)
        cost_xy = apply_sakoe_chiba_band(cost_xy, bandwidth, T1, T2)
        cost_xx = apply_sakoe_chiba_band(cost_xx, bandwidth, T1, T1)
        cost_yy = apply_sakoe_chiba_band(cost_yy, bandwidth, T2, T2)

    # Compute Soft-DTW values
    sdtw_xy = soft_dtw_forward(cost_xy, gamma)
    sdtw_xx = soft_dtw_forward(cost_xx, gamma)
    sdtw_yy = soft_dtw_forward(cost_yy, gamma)

    # Divergence formula
    divergence = 2 * sdtw_xy - sdtw_xx - sdtw_yy

    # Normalize by path length to make scores comparable across different clip lengths
    path = extract_path_from_cost(cost_xy.cpu().numpy())
    path_len = max(len(path), 1)
    divergence_per_step = max(0.0, float(divergence)) / path_len

    # Diagnostic logging
    print(f"[DTW] T_student={len(x)}, T_reference={len(y)}, bandwidth={bandwidth} frames")
    print(f"[DTW] sdtw_xy={sdtw_xy:.3f}, sdtw_xx={sdtw_xx:.3f}, sdtw_yy={sdtw_yy:.3f}")
    print(f"[DTW] divergence={divergence:.3f}, path_len={path_len}, per_step={divergence_per_step:.4f}")
    print(f"[DTW] path coverage: {len(np.unique(path[:,1]))}/{len(y)} ref frames ({100*len(np.unique(path[:,1]))/len(y):.1f}%)")
    print(f"[DTW] path monotonic: {np.all(np.diff(path[:,0]) >= 0) and np.all(np.diff(path[:,1]) >= 0)}")

    return divergence_per_step, path


def pairwise_euclidean(x: torch.Tensor, y: torch.Tensor) -> torch.Tensor:
    """
    Compute pairwise SQUARED Euclidean distance matrix.

    CRITICAL FIX: Use squared distance (no sqrt) for better numerical stability
    and to avoid sqrt's non-linearity distorting the cost landscape.

    Args:
        x: [T1, D]
        y: [T2, D]

    Returns:
        cost: [T1, T2] - squared Euclidean distances
    """
    # ||x - y||^2 = ||x||^2 + ||y||^2 - 2<x,y>
    xx = (x ** 2).sum(dim=1, keepdim=True)  # [T1, 1]
    yy = (y ** 2).sum(dim=1, keepdim=True)  # [T2, 1]
    xy = torch.mm(x, y.t())                  # [T1, T2]

    cost = xx + yy.t() - 2 * xy
    # Return SQUARED distance (no sqrt)
    return torch.clamp(cost, min=0)


def soft_dtw_forward(cost: torch.Tensor, gamma: float) -> float:
    """
    Compute Soft-DTW value using forward pass.

    Args:
        cost: Cost matrix [T1, T2]
        gamma: Temperature

    Returns:
        Soft-DTW value (scalar)
    """
    T1, T2 = cost.shape
    device = cost.device

    # Initialize DP matrix with infinities
    D = torch.full((T1 + 1, T2 + 1), float('inf'), device=device)
    D[0, 0] = 0.0

    # Forward pass with soft-min
    for i in range(1, T1 + 1):
        for j in range(1, T2 + 1):
            # Skip if cost is infinite (outside band)
            if torch.isinf(cost[i - 1, j - 1]):
                continue

            # Soft-min of three predecessors
            r0 = D[i - 1, j]
            r1 = D[i, j - 1]
            r2 = D[i - 1, j - 1]

            # Soft-min: -gamma * log(sum(exp(-r/gamma)))
            rmin = soft_min_3(r0, r1, r2, gamma)

            D[i, j] = cost[i - 1, j - 1] + rmin

    return float(D[T1, T2])


def soft_min_3(a: float, b: float, c: float, gamma: float) -> float:
    """
    Compute soft-min of three values.

    soft_min(a,b,c) = -gamma * log(exp(-a/gamma) + exp(-b/gamma) + exp(-c/gamma))
    """
    # Numerically stable version using log-sum-exp trick
    m = min(a, b, c)
    return -gamma * torch.log(
        torch.exp(-(a - m) / gamma) +
        torch.exp(-(b - m) / gamma) +
        torch.exp(-(c - m) / gamma)
    ) + m


def apply_sakoe_chiba_band(cost: torch.Tensor, bandwidth: int, T1: int = None, T2: int = None) -> torch.Tensor:
    """
    Apply Sakoe-Chiba band constraint to cost matrix.

    Sets costs outside the band to infinity.

    Args:
        cost: Cost matrix [T1, T2]
        bandwidth: Band width (in frames)
        T1: Optional override for T1 dimension
        T2: Optional override for T2 dimension

    Returns:
        Banded cost matrix
    """
    if T1 is None or T2 is None:
        T1, T2 = cost.shape

    # Vectorized band application (much faster than loops)
    device = cost.device
    i = torch.arange(T1, device=device).unsqueeze(1)
    j = torch.arange(T2, device=device).unsqueeze(0)

    # Compute diagonal position for each (i,j)
    diag_pos = j - (T2 / float(T1)) * i

    # Create mask for points outside band
    band_mask = diag_pos.abs() > bandwidth

    # Clone and apply mask
    cost_banded = cost.clone()
    cost_banded[band_mask] = float('inf')

    return cost_banded


def extract_path_from_cost(cost: np.ndarray) -> np.ndarray:
    """
    Extract approximate alignment path using standard DTW.

    (Used for visualization, not the actual Soft-DTW path)

    Returns:
        path: Array of (i, j) indices
    """
    T1, T2 = cost.shape

    # Standard DTW
    D = np.full((T1 + 1, T2 + 1), float('inf'))
    D[0, 0] = 0.0

    for i in range(1, T1 + 1):
        for j in range(1, T2 + 1):
            D[i, j] = cost[i - 1, j - 1] + min(D[i - 1, j], D[i, j - 1], D[i - 1, j - 1])

    # Backtrack
    path = []
    i, j = T1, T2
    while i > 0 and j > 0:
        path.append((i - 1, j - 1))

        # Choose predecessor
        candidates = [
            (D[i - 1, j], i - 1, j),
            (D[i, j - 1], i, j - 1),
            (D[i - 1, j - 1], i - 1, j - 1)
        ]
        _, i, j = min(candidates)

    return np.array(path[::-1])


def rhythm_score(
    student: FeaturePack,
    reference: FeaturePack,
    gamma: float = 0.12,
    bandwidth_pct: float = 0.09
) -> dict:
    """
    Compute rhythm similarity score using Soft-DTW divergence.

    CRITICAL FIX: Tighter band (0.09 instead of 0.12) to prevent wild warping.
    Lower gamma (0.12) for sharper alignment.

    Args:
        student: Student feature pack
        reference: Reference (Qari) feature pack
        gamma: Soft-DTW temperature (0.10-0.15 recommended)
        bandwidth_pct: Sakoe-Chiba band as % of sequence length (0.08-0.10)

    Returns:
        Dictionary with:
            - score: 0-100 (100 = perfect match)
            - divergence: Raw Soft-DTW divergence
            - path: Alignment path for visualization
            - notes: List of feedback notes
    """
    # Build multi-feature stacks
    student_features = build_multi_feature_stack(student)
    ref_features = build_multi_feature_stack(reference)

    # Compute bandwidth
    max_len = max(len(student_features), len(ref_features))
    bandwidth = int(bandwidth_pct * max_len)

    # Compute Soft-DTW divergence
    divergence, path = soft_dtw_divergence(
        student_features,
        ref_features,
        gamma=gamma,
        bandwidth=bandwidth
    )

    # Convert divergence_per_step to score (0-100)
    # CRITICAL FIX: divergence is now normalized by path length
    # Much smaller scale needed: 1.6 instead of 60
    # With div_per_step ~ 0.87, score ~ 59 (expected range for good imitation)
    scale = 1.6  # Calibrated for per-step divergence (0.5-2.0 typical range)
    score = 100 * np.exp(-divergence / scale)
    score = max(0, min(100, score))

    print(f"[DEBUG rhythm] Divergence per step: {divergence:.4f}")
    print(f"[DEBUG rhythm] Rhythm score: {score:.1f}")
    print(f"[DEBUG rhythm] Student features: {len(student_features)} frames")
    print(f"[DEBUG rhythm] Reference features: {len(ref_features)} frames")

    # Generate feedback notes
    notes = []
    if score >= 90:
        notes.append("Excellent rhythm - timing matches reference very well")
    elif score >= 75:
        notes.append("Good rhythm - minor timing variations present")
    elif score >= 60:
        notes.append("Rhythm needs work - some timing inconsistencies")
    else:
        notes.append("Rhythm significantly differs from reference")

    # Analyze path for specific issues
    path_drift = analyze_path_drift(path, len(student_features), len(ref_features))
    if path_drift > 0.15:
        notes.append(f"Timing drifts {path_drift*100:.0f}% off alignment path")

    return {
        'score': round(score, 1),
        'divergence': round(divergence, 3),
        'path': path.tolist(),
        'notes': notes,
        'bandwidth_used': bandwidth,
        # Add feature metadata for proper time warping
        'student_frame_times': student.frame_times.tolist(),
        'reference_frame_times': reference.frame_times.tolist()
    }


def analyze_path_drift(path: np.ndarray, len_student: int, len_ref: int) -> float:
    """
    Analyze how much the alignment path drifts from the diagonal.

    Returns:
        Drift ratio (0 = perfect diagonal, 1 = maximum drift)
    """
    if len(path) == 0:
        return 0.0

    # Expected diagonal slope
    expected_slope = len_ref / len_student

    # Compute actual path deviations
    deviations = []
    for i, j in path:
        expected_j = i * expected_slope
        deviation = abs(j - expected_j) / len_ref
        deviations.append(deviation)

    return float(np.mean(deviations))
