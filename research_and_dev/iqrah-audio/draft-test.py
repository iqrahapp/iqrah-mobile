#!/usr/bin/env python3
"""
Test script for the Real-Time Quran Recitation Analysis Pipeline
================================================================

This script tests individual components without requiring live audio input.
Useful for verifying the installation and basic functionality.

Usage:
    python test_pipeline.py
"""

import numpy as np
import matplotlib.pyplot as plt
import time
import sys
import os

# Add the main pipeline module
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

try:
    from realtime_quran_pipeline import (
        ReferenceGenerator, 
        PitchTracker, 
        DTWAligner, 
        PerformanceMonitor
    )
    print("✓ Successfully imported pipeline components")
except ImportError as e:
    print(f"✗ Failed to import pipeline components: {e}")
    sys.exit(1)


def generate_test_audio(frequency=440.0, duration=2.0, sample_rate=16000, noise_level=0.1):
    """Generate synthetic audio for testing"""
    t = np.linspace(0, duration, int(sample_rate * duration))
    
    # Create a sine wave with some frequency modulation
    freq_mod = frequency * (1 + 0.1 * np.sin(2 * np.pi * t * 0.5))
    signal = np.sin(2 * np.pi * freq_mod * t)
    
    # Add some noise
    noise = np.random.normal(0, noise_level, len(signal))
    signal += noise
    
    # Apply envelope to make it more realistic
    envelope = np.exp(-t * 0.5)  # Decay envelope
    signal *= envelope
    
    return signal.astype(np.float32)


def test_reference_generation():
    """Test reference signal generation"""
    print("\n1. Testing Reference Generation...")
    
    try:
        # Test both reference types
        ref_short = ReferenceGenerator.create_test_reference(duration=5.0)
        ref_long = ReferenceGenerator.create_infinite_demo_reference(duration=15.0)
        
        print(f"   ✓ Short reference: {len(ref_short['time'])} points, {ref_short['time'][-1]:.1f}s")
        print(f"   ✓ Long reference: {len(ref_long['time'])} points, {ref_long['time'][-1]:.1f}s")
        print(f"   ✓ Frequency range: {np.min(ref_long['frequency']):.1f} - {np.max(ref_long['frequency']):.1f} Hz")
        print(f"   ✓ Semitone range: {np.min(ref_long['semitones']):.1f} - {np.max(ref_long['semitones']):.1f}")
        
        return ref_long
        
    except Exception as e:
        print(f"   ✗ Reference generation failed: {e}")
        return None


def test_pitch_tracking():
    """Test pitch tracking"""
    print("\n2. Testing Pitch Tracking...")
    
    try:
        tracker = PitchTracker(sample_rate=16000, hop_length=512)
        
        # Test with known frequencies
        test_frequencies = [220.0, 440.0, 880.0]  # A3, A4, A5
        results = []
        
        for freq in test_frequencies:
            audio = generate_test_audio(frequency=freq, duration=1.0)
            detected_freq, confidence = tracker.extract_pitch(audio)
            
            error = abs(detected_freq - freq) if detected_freq > 0 else freq
            error_percent = (error / freq) * 100
            
            results.append({
                'target': freq,
                'detected': detected_freq,
                'error_percent': error_percent,
                'confidence': confidence
            })
            
            status = "✓" if error_percent < 10 else "⚠"  # Less than 10% error is good
            print(f"   {status} {freq}Hz -> {detected_freq:.1f}Hz (error: {error_percent:.1f}%, confidence: {confidence:.2f})")
        
        # Overall accuracy
        avg_error = np.mean([r['error_percent'] for r in results])
        print(f"   → Average error: {avg_error:.1f}%")
        
        return results
        
    except Exception as e:
        print(f"   ✗ Pitch tracking failed: {e}")
        return None


def test_dtw_alignment():
    """Test DTW alignment"""
    print("\n3. Testing DTW Alignment...")
    
    try:
        aligner = DTWAligner()
        
        # Create test sequences
        reference = np.array([0, 2, 4, 2, 0, -2, -4, -2, 0])  # Semitone pattern
        
        # Test perfect alignment
        query1 = reference.copy()
        pos1, score1, path1 = aligner.align(query1, reference)
        
        # Test shifted alignment  
        query2 = np.roll(reference, 3)  # Shift by 3 positions
        pos2, score2, path2 = aligner.align(query2, reference)
        
        # Test noisy alignment
        query3 = reference + np.random.normal(0, 0.5, len(reference))
        pos3, score3, path3 = aligner.align(query3, reference)
        
        print(f"   ✓ Perfect match - Position: {pos1}, Score: {score1:.3f}")
        print(f"   ✓ Shifted match - Position: {pos2}, Score: {score2:.3f}")
        print(f"   ✓ Noisy match - Position: {pos3}, Score: {score3:.3f}")
        
        return True
        
    except Exception as e:
        print(f"   ✗ DTW alignment failed: {e}")
        return False


def test_performance_monitoring():
    """Test performance monitoring"""
    print("\n4. Testing Performance Monitoring...")
    
    try:
        monitor = PerformanceMonitor(window_size=10)
        
        # Simulate processing with random delays
        for i in range(15):
            monitor.start_timing()
            time.sleep(np.random.uniform(0.01, 0.05))  # 10-50ms delays
            latency = monitor.end_timing()
        
        stats = monitor.get_stats()
        
        print(f"   ✓ Average latency: {stats['avg_latency']:.1f}ms")
        print(f"   ✓ Max latency: {stats['max_latency']:.1f}ms") 
        print(f"   ✓ Min latency: {stats['min_latency']:.1f}ms")
        print(f"   ✓ Frames processed: {stats['frames_processed']}")
        
        return stats
        
    except Exception as e:
        print(f"   ✗ Performance monitoring failed: {e}")
        return None


def test_integration():
    """Test integration of all components"""
    print("\n5. Testing Component Integration...")
    
    try:
        # Initialize components
        tracker = PitchTracker()
        aligner = DTWAligner()
        monitor = PerformanceMonitor()
        reference = ReferenceGenerator.create_infinite_demo_reference(duration=10.0)
        
        # Simulate real-time processing
        print("   → Simulating 30 frames of processing...")
        
        pitch_buffer = []
        score_buffer = []
        
        for frame in range(30):
            monitor.start_timing()
            
            # Generate frame of audio (simulating microphone input)
            # Follow the C major scale pattern
            t = frame * 0.1
            cycle_time = t % 12.0
            note_index = int(cycle_time / 1.5)
            scale_freqs = [261.63, 293.66, 329.63, 349.23, 392.00, 440.00, 493.88, 523.25]
            freq = scale_freqs[min(note_index, len(scale_freqs)-1)]
            
            audio_frame = generate_test_audio(
                frequency=freq, 
                duration=0.1, 
                sample_rate=16000
            )
            
            # Process frame
            pitch_hz, confidence = tracker.extract_pitch(audio_frame)
            
            if pitch_hz > 0:
                pitch_semitones = 12 * np.log2(pitch_hz / 440.0)
                pitch_buffer.append(pitch_semitones)
                
                # Align with reference
                if len(pitch_buffer) >= 5:
                    recent_pitches = pitch_buffer[-5:]
                    pos, score, path = aligner.align(recent_pitches, reference['semitones'])
                    score_percent = max(0, (1.0 - score) * 100)
                    score_buffer.append(score_percent)
            
            latency = monitor.end_timing()
        
        # Results
        avg_score = np.mean(score_buffer) if score_buffer else 0
        stats = monitor.get_stats()
        
        print(f"   ✓ Processed {len(pitch_buffer)} valid pitch frames")
        print(f"   ✓ Average alignment score: {avg_score:.1f}%")
        print(f"   ✓ Average processing latency: {stats['avg_latency']:.1f}ms")
        
        # Check if latency is reasonable for real-time (< 100ms)
        if stats['avg_latency'] < 100:
            print("   ✓ Latency suitable for real-time processing")
        else:
            print("   ⚠ High latency - may not be suitable for real-time")
        
        return True
        
    except Exception as e:
        print(f"   ✗ Integration test failed: {e}")
        import traceback
        traceback.print_exc()
        return False


def create_test_visualization():
    """Create a test visualization"""
    print("\n6. Creating Test Visualization...")
    
    try:
        # Generate test data using the new infinite reference
        reference = ReferenceGenerator.create_infinite_demo_reference(duration=15.0)
        
        # Simulate some live pitch data
        time_points = np.linspace(0, 12, 120)  # 12 seconds of data
        live_pitches = []
        
        for t in time_points:
            # Simulate following the reference with some error and delay
            ref_idx = int(t * len(reference['time']) / 15)  # Scale to reference length
            if ref_idx < len(reference['semitones']):
                target = reference['semitones'][ref_idx]
                noise = np.random.normal(0, 0.5)
                delay_error = np.sin(2 * np.pi * 0.1 * t) * 2  # Some systematic error
                live_pitches.append(target + noise + delay_error)
            else:
                live_pitches.append(0)
        
        # Create plot showing one complete cycle
        plt.figure(figsize=(15, 8))
        
        # Show first 12 seconds (one complete pattern)
        ref_mask = reference['time'] <= 12
        ref_times = reference['time'][ref_mask] 
        ref_semitones = reference['semitones'][ref_mask]
        
        plt.plot(ref_times, ref_semitones, 'b-', linewidth=3, label='Reference (C Major Scale)', alpha=0.8)
        plt.plot(time_points, live_pitches, 'r-', linewidth=2, label='Simulated Live Input', alpha=0.9)
        
        # Add note labels
        note_names = ['C4', 'D4', 'E4', 'F4', 'G4', 'A4', 'B4', 'C5']
        for i, note in enumerate(note_names):
            t = i * 1.5 + 0.75  # Center of each note
            if t <= 12:
                plt.annotate(note, (t, 8), ha='center', va='bottom', fontsize=10, 
                           bbox=dict(boxstyle="round,pad=0.3", facecolor="lightblue", alpha=0.7))
        
        plt.xlabel('Time (seconds)')
        plt.ylabel('Pitch (Semitones from A4)')
        plt.title('Test Visualization: C Major Scale Reference vs Simulated Live Input')
        plt.grid(True, alpha=0.3)
        plt.legend()
        
        # Add pattern information
        plt.text(0.02, 0.98, 
                'Pattern: C-D-E-F-G-A-B-C (repeats every 12 seconds)\n'
                'Each note lasts 1.5 seconds with smooth transitions', 
                transform=plt.gca().transAxes,
                verticalalignment='top',
                bbox=dict(boxstyle="round,pad=0.3", facecolor="yellow", alpha=0.7))
        
        plt.tight_layout()
        
        # Save the plot
        plt.savefig('test_visualization.png', dpi=150, bbox_inches='tight')
        print("   ✓ Test visualization saved as 'test_visualization.png'")
        
        # Show plot (comment out if running headless)
        # plt.show()
        
        return True
        
    except Exception as e:
        print(f"   ✗ Visualization test failed: {e}")
        return False


def main():
    """Main test function"""
    print("Real-Time Quran Pipeline Component Tests")
    print("=" * 50)
    
    # Run all tests
    tests = [
        ("Reference Generation", test_reference_generation),
        ("Pitch Tracking", test_pitch_tracking),
        ("DTW Alignment", test_dtw_alignment),
        ("Performance Monitoring", test_performance_monitoring),
        ("Integration", test_integration),
        ("Visualization", create_test_visualization)
    ]
    
    results = {}
    
    for test_name, test_func in tests:
        try:
            result = test_func()
            results[test_name] = result is not None and result is not False
        except Exception as e:
            print(f"   ✗ {test_name} crashed: {e}")
            results[test_name] = False
    
    # Summary
    print("\n" + "=" * 50)
    print("TEST SUMMARY")
    print("=" * 50)
    
    passed = sum(results.values())
    total = len(results)
    
    for test_name, passed_test in results.items():
        status = "✓ PASS" if passed_test else "✗ FAIL"
        print(f"{status} {test_name}")
    
    print(f"\nResults: {passed}/{total} tests passed")
    
    if passed == total:
        print("\n🎉 All tests passed! The pipeline is ready to use.")
        print("Run 'python realtime_quran_pipeline.py' to start the real-time demo.")
    else:
        print(f"\n⚠️  {total - passed} tests failed. Check the error messages above.")
        print("Some features may not work correctly in the real-time pipeline.")
    
    return passed == total


if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)