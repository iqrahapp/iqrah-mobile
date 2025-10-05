#!/usr/bin/env python3
"""
Test V2 with real Husary audio (simplified, no full pipeline)
"""

import numpy as np
import soundfile as sf
from src.iqrah_audio.pitch import PitchExtractor
from src.iqrah_audio.streaming.online_dtw_v2 import TrueOnlineDTW

def test_real_audio():
    print("=" * 80)
    print("V2 REAL AUDIO TEST (Husary Al-Fatiha)")
    print("=" * 80)
    
    # Load audio
    audio_path = "data/husary/surahs/01.mp3"
    print(f"\n📖 Loading: {audio_path}")
    audio, sr = sf.read(audio_path)
    if len(audio.shape) > 1:
        audio = audio.mean(axis=1)
    audio = audio.astype(np.float32)
    
    duration = len(audio) / sr
    print(f"✓ Loaded: {duration:.2f}s @ {sr} Hz")
    
    # Extract pitch
    print("\n🎵 Extracting pitch...")
    extractor = PitchExtractor(method="yin", sample_rate=sr, hop_length=512)
    pitch_contour = extractor.extract_stable_pitch(audio)
    
    f0_hz = pitch_contour.f0_hz
    confidence = pitch_contour.confidence
    print(f"✓ Pitch extracted: {len(f0_hz)} frames")
    
    # Self-alignment test
    print("\n🔄 Running self-alignment test...")
    print("  (Using same audio as reference and query)")
    
    dtw = TrueOnlineDTW(f0_hz, sample_rate=sr, hop_length=512)
    
    # Seed with first 50 frames at position 0
    seed_len = 50
    dtw.seed(f0_hz[:seed_len], force_position=0)
    
    print(f"\n✓ Seeded at position {dtw.state.reference_position}")
    print(f"  Frames processed: {dtw.state.frames_processed}")
    
    # Process remaining frames
    print(f"\n▶ Processing {len(f0_hz) - seed_len} frames...")
    
    diagonal_count = 0
    total_frames = 0
    
    for i in range(seed_len, len(f0_hz)):
        state = dtw.update(f0_hz[i], confidence[i])
        
        # Check if diagonal (±1 frame tolerance for real audio)
        if abs(state.reference_position - i) <= 1:
            diagonal_count += 1
        total_frames += 1
        
        if i % 500 == 0:
            acc = (diagonal_count / total_frames) * 100
            print(f"  Frame {i}/{len(f0_hz)}: ref_pos={state.reference_position}, "
                  f"diff={state.reference_position - i:+d}, conf={state.confidence:.3f}, "
                  f"accuracy={acc:.1f}%")
    
    # Results
    print("\n" + "=" * 80)
    print("RESULTS")
    print("=" * 80)
    
    accuracy = (diagonal_count / total_frames) * 100
    final_pos = state.reference_position
    expected_pos = len(f0_hz) - 1
    
    print(f"\n📊 Tracking accuracy: {diagonal_count}/{total_frames} = {accuracy:.1f}%")
    print(f"  Final position: {final_pos}/{expected_pos}")
    print(f"  Position error: {final_pos - expected_pos:+d} frames")
    print(f"  Final confidence: {state.confidence:.3f}")
    print(f"  Lead/lag: {dtw.get_lead_lag_ms():+.1f}ms")
    
    if accuracy >= 95.0:
        print(f"\n✅ EXCELLENT: {accuracy:.1f}% accuracy")
        return True
    elif accuracy >= 85.0:
        print(f"\n✓ GOOD: {accuracy:.1f}% accuracy")
        return True
    elif accuracy >= 70.0:
        print(f"\n⚠ FAIR: {accuracy:.1f}% accuracy")
        return False
    else:
        print(f"\n❌ POOR: {accuracy:.1f}% accuracy")
        return False

if __name__ == "__main__":
    try:
        success = test_real_audio()
        exit(0 if success else 1)
    except Exception as e:
        print(f"\n❌ Error: {e}")
        import traceback
        traceback.print_exc()
        exit(1)
