#!/usr/bin/env python3
"""
Simple test for V2 improvements without full demo infrastructure
"""

import numpy as np
from src.iqrah_audio.streaming.online_dtw_v2 import TrueOnlineDTW

def test_self_alignment():
    """Test self-alignment with synthetic data"""
    print("=" * 80)
    print("V2 SELF-ALIGNMENT TEST (Synthetic Data)")
    print("=" * 80)
    
    # Create synthetic reference: 500 frames with pattern
    ref_pattern = np.concatenate([
        np.full(100, 100.0),  # 100 frames at 100 Hz
        np.full(100, 200.0),  # 100 frames at 200 Hz
        np.full(100, 150.0),  # 100 frames at 150 Hz
        np.full(100, 250.0),  # 100 frames at 250 Hz
        np.full(100, 175.0),  # 100 frames at 175 Hz
    ])
    
    # Query is same as reference (perfect self-alignment)
    query_pattern = ref_pattern.copy()
    
    print(f"\n📊 Test setup:")
    print(f"  Reference frames: {len(ref_pattern)}")
    print(f"  Query frames: {len(query_pattern)}")
    print(f"  Expected: 100% tracking (500/500)")
    
    # Create DTW
    dtw = TrueOnlineDTW(ref_pattern)
    
    # Seed with first 50 frames at position 0 (self-alignment)
    seed_length = 50
    dtw.seed(query_pattern[:seed_length], force_position=0)
    
    print(f"\n🔧 After seeding:")
    print(f"  Reference position: {dtw.state.reference_position}")
    print(f"  Frames processed: {dtw.state.frames_processed}")
    print(f"  Expected position: {seed_length - 1}")
    
    # Process remaining frames
    print(f"\n▶ Processing {len(query_pattern) - seed_length} frames...")
    
    diagonal_count = 0
    total_frames = 0
    
    for i in range(seed_length, len(query_pattern)):
        state = dtw.update(query_pattern[i], query_confidence=1.0)
        
        # Check if diagonal (ref_pos == query_idx)
        if state.reference_position == i:
            diagonal_count += 1
        total_frames += 1
        
        # Print every 100 frames
        if i % 100 == 0:
            accuracy = (diagonal_count / total_frames) * 100 if total_frames > 0 else 0
            print(f"  Frame {i}: ref_pos={state.reference_position}, "
                  f"expected={i}, diff={state.reference_position - i:+d}, "
                  f"conf={state.confidence:.3f}, accuracy={accuracy:.1f}%")
    
    # Final results
    print("\n" + "=" * 80)
    print("RESULTS")
    print("=" * 80)
    
    final_accuracy = (diagonal_count / total_frames) * 100
    final_pos = state.reference_position
    expected_pos = len(query_pattern) - 1
    
    print(f"\n📊 Tracking accuracy: {diagonal_count}/{total_frames} = {final_accuracy:.1f}%")
    print(f"  Final position: {final_pos}")
    print(f"  Expected position: {expected_pos}")
    print(f"  Position error: {final_pos - expected_pos:+d} frames")
    print(f"  Final confidence: {state.confidence:.3f}")
    print(f"  Drift estimate: {state.drift_estimate:.1f}")
    
    if final_accuracy >= 95.0:
        print(f"\n✅ EXCELLENT: {final_accuracy:.1f}% accuracy (≥95%)")
    elif final_accuracy >= 85.0:
        print(f"\n✓ GOOD: {final_accuracy:.1f}% accuracy (≥85%)")
    elif final_accuracy >= 70.0:
        print(f"\n⚠ FAIR: {final_accuracy:.1f}% accuracy (≥70%)")
    else:
        print(f"\n❌ POOR: {final_accuracy:.1f}% accuracy (<70%)")
    
    print("\n" + "=" * 80)
    
    return final_accuracy >= 90.0  # Success if ≥90%


if __name__ == "__main__":
    success = test_self_alignment()
    exit(0 if success else 1)
