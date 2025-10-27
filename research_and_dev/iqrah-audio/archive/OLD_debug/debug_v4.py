"""Quick debug script for OLTW v4."""

import numpy as np
from src.iqrah_audio.streaming.online_dtw_v4 import OLTWAligner

# Create simple test signal
sr = 22050
duration = 2.0  # 2 seconds
t = np.linspace(0, duration, int(sr * duration))

# Sine wave at 200 Hz (typical pitch)
reference = 200.0 + 50.0 * np.sin(2 * np.pi * 2.0 * t)  # 200 Hz with 2Hz vibrato

# Sample to frames (hop_length=512)
hop = 512
n_frames = len(reference) // hop
reference_frames = reference[::hop][:n_frames]

print(f"Reference: {len(reference_frames)} frames")
print(f"Mean pitch: {np.mean(reference_frames):.1f} Hz")

# Test 1: Self-alignment with delta-pitch
print("\n" + "="*60)
print("TEST 1: Self-alignment with delta-pitch (default)")
print("="*60)

aligner_delta = OLTWAligner(
    reference=reference_frames,
    sample_rate=sr,
    hop_length=hop,
    force_seed_position=0,
    use_delta_pitch=True,  # Default
)

# Seed
aligner_delta.oltw.seed(reference_frames[:50], force_position=0)

# Process all available frames
for i in range(min(200, len(reference_frames))):
    state = aligner_delta.update(
        query_frame=reference_frames[i],
        query_confidence=1.0,
        reference=reference_frames,
    )

    if i % 20 == 0 and i > 0:
        pen_v, pen_h = aligner_delta.oltw._compute_transition_penalties()
        print(f"  Frame {i:3d}: pos={state.reference_position:3d} conf={state.confidence:.2f} "
              f"pen_v={pen_v:.2f} pen_h={pen_h:.2f}")

print(f"\nFinal: {state.reference_position}/{n_frames} = {100*state.reference_position/n_frames:.1f}%")

# Test 2: Self-alignment with raw pitch
print("\n" + "="*60)
print("TEST 2: Self-alignment with raw pitch")
print("="*60)

aligner_raw = OLTWAligner(
    reference=reference_frames,
    sample_rate=sr,
    hop_length=hop,
    force_seed_position=0,
    use_delta_pitch=False,  # Raw pitch
)

# Seed
aligner_raw.oltw.seed(reference_frames[:50], force_position=0)

# Process all available frames
for i in range(min(200, len(reference_frames))):
    state = aligner_raw.update(
        query_frame=reference_frames[i],
        query_confidence=1.0,
        reference=reference_frames,
    )

    if i % 20 == 0 and i > 0:
        pen_v, pen_h = aligner_raw.oltw._compute_transition_penalties()
        print(f"  Frame {i:3d}: pos={state.reference_position:3d} conf={state.confidence:.2f} "
              f"pen_v={pen_v:.2f} pen_h={pen_h:.2f}")

print(f"\nFinal: {state.reference_position}/{n_frames} = {100*state.reference_position/n_frames:.1f}%")
