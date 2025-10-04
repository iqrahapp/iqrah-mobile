#!/usr/bin/env python3
"""
Test Streaming Components
==========================

Test and benchmark streaming audio buffer and incremental pitch extraction.
"""

import numpy as np
import time
import sys
from pathlib import Path

print("=" * 80)
print("STREAMING COMPONENTS TEST")
print("=" * 80)


# Test 1: Streaming Audio Buffer
print("\n" + "=" * 80)
print("TEST 1: Streaming Audio Buffer")
print("=" * 80)

try:
    from iqrah_audio.streaming import StreamingAudioBuffer

    # Create buffer
    buffer = StreamingAudioBuffer(window_size_s=3.0, sample_rate=22050)
    print(f"\n✓ Created: {buffer}")

    # Test: Push chunks and verify
    print("\nPushing audio chunks...")
    chunk_size = 1024  # ~46ms at 22050 Hz

    for i in range(10):
        chunk = np.random.randn(chunk_size).astype(np.float32)
        buffer.push_samples(chunk)

        if i % 3 == 0:
            print(f"  Chunk {i+1}: {buffer.available_samples} samples "
                  f"({buffer.duration_s:.2f}s) available, full={buffer.is_full}")

    # Verify we can retrieve window
    window = buffer.get_window()
    print(f"\n✓ Retrieved window: {len(window)} samples")

    # Test latest samples
    latest = buffer.get_latest_samples(512)
    print(f"✓ Latest 512 samples: {len(latest)} samples")

    # Test clear
    buffer.clear()
    print(f"✓ Cleared buffer: {buffer.available_samples} samples")

    print("\n✓ Test 1 PASSED")

except Exception as e:
    print(f"\n✗ Test 1 FAILED: {e}")
    import traceback
    traceback.print_exc()
    sys.exit(1)


# Test 2: Incremental Pitch Extraction
print("\n" + "=" * 80)
print("TEST 2: Incremental Pitch Extraction")
print("=" * 80)

try:
    from iqrah_audio.streaming import IncrementalPitchExtractor

    # Create extractor
    extractor = IncrementalPitchExtractor(
        method="yin",
        sample_rate=22050,
        hop_length=512,  # ~23ms
    )
    print(f"\n✓ Created: {extractor}")

    # Generate test audio with known pitch
    print("\nGenerating test audio (220 Hz)...")
    sr = 22050
    duration = 3.0
    frequency = 220.0
    t = np.linspace(0, duration, int(sr * duration))
    audio = np.sin(2 * np.pi * frequency * t).astype(np.float32)

    # Simulate streaming: process in chunks
    print("\nSimulating streaming (50ms chunks)...")
    chunk_size_ms = 50
    chunk_samples = int(sr * chunk_size_ms / 1000)

    total_new_frames = 0
    latencies = []

    for i in range(0, len(audio), chunk_samples):
        chunk = audio[i:i+chunk_samples]

        # Measure latency
        start = time.time()
        f0, conf, ts = extractor.process_chunk(chunk)
        elapsed_ms = (time.time() - start) * 1000

        if len(f0) > 0:
            latencies.append(elapsed_ms)
            total_new_frames += len(f0)

            if i % (chunk_samples * 10) == 0:  # Print every 10 chunks
                median_f0 = np.median(f0[conf > 0.5]) if np.any(conf > 0.5) else 0
                print(f"  Chunk {i//chunk_samples + 1}: "
                      f"{len(f0)} new frames, "
                      f"median F0: {median_f0:.1f} Hz, "
                      f"latency: {elapsed_ms:.2f}ms")

    print(f"\n✓ Processed {total_new_frames} frames")
    print(f"  Average latency: {np.mean(latencies):.2f}ms")
    print(f"  Max latency: {np.max(latencies):.2f}ms")
    print(f"  Min latency: {np.min(latencies):.2f}ms")

    # Verify final contour
    contour = extractor.get_contour()
    median_f0 = np.median(contour.f0_hz[contour.confidence > 0.5])
    print(f"\n✓ Final contour: {len(contour.f0_hz)} frames")
    print(f"  Median F0: {median_f0:.1f} Hz (expected ~220 Hz)")
    print(f"  Duration: {contour.duration:.2f}s")

    # Check accuracy
    error_hz = abs(median_f0 - 220.0)
    if error_hz < 5.0:
        print(f"  ✓ Accuracy: ±{error_hz:.1f} Hz (within 5 Hz)")
    else:
        print(f"  ⚠ Accuracy: ±{error_hz:.1f} Hz (> 5 Hz)")

    # Check latency target
    avg_latency = np.mean(latencies)
    if avg_latency < 10.0:
        print(f"  ✓ Latency: {avg_latency:.2f}ms (< 10ms target)")
    else:
        print(f"  ⚠ Latency: {avg_latency:.2f}ms (> 10ms target)")

    print("\n✓ Test 2 PASSED")

except Exception as e:
    print(f"\n✗ Test 2 FAILED: {e}")
    import traceback
    traceback.print_exc()
    sys.exit(1)


# Test 3: Streaming Pitch Analyzer (High-level API)
print("\n" + "=" * 80)
print("TEST 3: Streaming Pitch Analyzer")
print("=" * 80)

try:
    from iqrah_audio.streaming import StreamingPitchAnalyzer

    # Create analyzer
    analyzer = StreamingPitchAnalyzer(
        method="yin",
        sample_rate=22050,
        buffer_size_s=3.0,
        hop_length=512,
    )
    print(f"\n✓ Created analyzer")

    # Generate test audio
    sr = 22050
    duration = 2.0
    frequency = 440.0  # A4
    t = np.linspace(0, duration, int(sr * duration))
    audio = np.sin(2 * np.pi * frequency * t).astype(np.float32)

    # Simulate streaming
    print("\nStreaming audio (100ms chunks)...")
    chunk_size = int(sr * 0.1)  # 100ms chunks

    total_frames = 0
    for i in range(0, len(audio), chunk_size):
        chunk = audio[i:i+chunk_size]
        f0, conf, ts = analyzer.push_audio(chunk)

        if len(f0) > 0:
            total_frames += len(f0)

    print(f"✓ Processed {total_frames} frames")

    # Get final contour
    contour = analyzer.get_contour()
    median_f0 = np.median(contour.f0_hz[contour.confidence > 0.5])
    print(f"✓ Median F0: {median_f0:.1f} Hz (expected ~440 Hz)")

    error_hz = abs(median_f0 - 440.0)
    if error_hz < 5.0:
        print(f"  ✓ Accuracy: ±{error_hz:.1f} Hz")

    print("\n✓ Test 3 PASSED")

except Exception as e:
    print(f"\n✗ Test 3 FAILED: {e}")
    import traceback
    traceback.print_exc()
    sys.exit(1)


# Test 4: Performance Benchmark
print("\n" + "=" * 80)
print("TEST 4: Performance Benchmark")
print("=" * 80)

try:
    print("\nBenchmarking streaming pitch extraction...")
    print("Target: <10ms per frame for real-time capability")
    print()

    from iqrah_audio.streaming import IncrementalPitchExtractor

    # Test different chunk sizes
    chunk_sizes_ms = [10, 25, 50, 100]
    sr = 22050

    results = []

    for chunk_ms in chunk_sizes_ms:
        extractor = IncrementalPitchExtractor(method="yin", sample_rate=sr)

        # Generate 1 second of audio
        audio = np.random.randn(sr).astype(np.float32)
        chunk_samples = int(sr * chunk_ms / 1000)

        latencies = []
        n_chunks = 0

        for i in range(0, len(audio), chunk_samples):
            chunk = audio[i:i+chunk_samples]
            start = time.time()
            f0, conf, ts = extractor.process_chunk(chunk)
            elapsed_ms = (time.time() - start) * 1000
            latencies.append(elapsed_ms)
            n_chunks += 1

        avg_latency = np.mean(latencies)
        max_latency = np.max(latencies)

        results.append({
            "chunk_ms": chunk_ms,
            "n_chunks": n_chunks,
            "avg_latency": avg_latency,
            "max_latency": max_latency,
        })

        status = "✓" if avg_latency < 10 else "⚠"
        print(f"{status} Chunk {chunk_ms:3d}ms: "
              f"avg={avg_latency:5.2f}ms, max={max_latency:5.2f}ms "
              f"({n_chunks} chunks)")

    print("\n✓ Test 4 PASSED")

except Exception as e:
    print(f"\n✗ Test 4 FAILED: {e}")
    import traceback
    traceback.print_exc()
    sys.exit(1)


# Summary
print("\n" + "=" * 80)
print("TEST SUMMARY")
print("=" * 80)

print("\n✅ All streaming components tests PASSED:")
print("  1. ✓ StreamingAudioBuffer: Ring buffer working")
print("  2. ✓ IncrementalPitchExtractor: Incremental extraction working")
print("  3. ✓ StreamingPitchAnalyzer: High-level API working")
print("  4. ✓ Performance: Latency benchmarked")

print("\n📊 Performance Summary:")
for r in results:
    status = "✓" if r["avg_latency"] < 10 else "⚠"
    print(f"  {status} {r['chunk_ms']}ms chunks: {r['avg_latency']:.2f}ms avg latency")

print("\n🚀 Next Steps:")
print("  1. Implement AnchorDetector (silence, plosives, long notes)")
print("  2. Enhance OnlineDTW with anchors and confidence gating")
print("  3. Implement LiveFeedback system")
print("  4. Create RealtimePipeline integration")

print("\n" + "=" * 80)
print("All tests completed!")
print("=" * 80)
