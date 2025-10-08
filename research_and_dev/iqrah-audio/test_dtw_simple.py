#!/usr/bin/env python3
"""
Simple DTW Test - Compare your recitation vs Husary
"""

import sys
sys.path.insert(0, 'src')

from pathlib import Path
import torchaudio
import numpy as np

# Test the critical DTW warping fix
def test_dtw_mapping():
    print("=" * 80)
    print("Testing DTW Feature-to-Time Mapping")
    print("=" * 80)

    # Simulate the problem:
    # Student: 6.33s normalized to 150 frames
    # Reference: 5.12s normalized to 150 frames

    student_duration = 6.328
    ref_duration = 5.12
    n_frames = 150

    student_frame_times = np.linspace(0, student_duration, n_frames)
    ref_frame_times = np.linspace(0, ref_duration, n_frames)

    print(f"\nStudent: {student_duration:.2f}s in {n_frames} frames")
    print(f"Reference: {ref_duration:.2f}s in {n_frames} frames")

    print(f"\nTime per frame:")
    print(f"  Student: {student_duration/n_frames:.4f}s/frame")
    print(f"  Reference: {ref_duration/n_frames:.4f}s/frame")

    # Simulate DTW path mapping frame 100 to frame 100
    frame_idx = 100

    print(f"\n❌ WRONG (old method) - assuming same frame = same time:")
    print(f"  Frame {frame_idx} → Student: {student_frame_times[frame_idx]:.3f}s")
    print(f"  Frame {frame_idx} → Reference: {ref_frame_times[frame_idx]:.3f}s")
    print(f"  Difference: {abs(student_frame_times[frame_idx] - ref_frame_times[frame_idx]):.3f}s")

    print(f"\n✅ CORRECT (new method) - using frame_times:")
    print(f"  1. Student pitch at 4.5s → Find in student_frame_times → frame {np.argmin(np.abs(student_frame_times - 4.5))}")
    print(f"  2. DTW maps student frame to reference frame (e.g., same index)")
    print(f"  3. Reference frame → Look up in reference_frame_times → {ref_frame_times[np.argmin(np.abs(student_frame_times - 4.5))]:.3f}s")

    print("\n" + "=" * 80)
    print("This is why alignment was wrong - different durations = different times!")
    print("=" * 80)

if __name__ == "__main__":
    test_dtw_mapping()

    print("\n\n" + "=" * 80)
    print("To test with actual audio files:")
    print("=" * 80)
    print("1. Start the server: conda run -n iqrah python app_qari_final.py")
    print("2. Go to: http://localhost:8006/compare/user")
    print("3. Upload: data/me/surahs/001/01.mp3")
    print("4. Select: Surah 1, Ayah 1")
    print("5. Check console for DTW frame_times debug logs")
    print("6. Verify audio plays immediately (no 18s delay)")
    print("7. Check if Melody/Rhythm scores are now 40+")
    print("=" * 80)
