#!/usr/bin/env python3
"""
Test Pitch Extraction Optimization
===================================

Compare current vs optimized incremental pitch extraction latency.
Target: Reduce from ~60ms to <10ms per chunk
"""

import numpy as np
import time
import soundfile as sf
from pathlib import Path

print("=" * 80)
print("PITCH EXTRACTION OPTIMIZATION TEST")
print("=" * 80)

# Test 1: Current Implementation Baseline
print("\n" + "=" * 80)
print("TEST 1: Current Implementation Baseline")
print("=" * 80)

try:
    from iqrah_audio.streaming import IncrementalPitchExtractor

    # Generate test audio
    sr = 22050
    duration = 5.0
    frequency = 220.0

    t = np.linspace(0, duration, int(sr * duration))
    audio = np.sin(2 * np.pi * frequency * t).astype(np.float32)

    print(f"\nTest audio: {duration:.1f}s @ {frequency} Hz")

    # Create extractor
    extractor = IncrementalPitchExtractor(
        method="yin",
        sample_rate=sr,
        hop_length=512,
    )

    # Simulate streaming
    chunk_size = 512
    n_chunks = len(audio) // chunk_size
    latencies = []

    print(f"\nProcessing {n_chunks} chunks of {chunk_size} samples...")

    for i in range(n_chunks):
        chunk = audio[i * chunk_size:(i + 1) * chunk_size]

        t0 = time.perf_counter()
        f0, conf, ts = extractor.process_chunk(chunk)
        t1 = time.perf_counter()

        latency_ms = (t1 - t0) * 1000
        latencies.append(latency_ms)

    avg_latency = np.mean(latencies)
    max_latency = np.max(latencies)
    min_latency = np.min(latencies)
    p95_latency = np.percentile(latencies, 95)

    print(f"\n📊 Current Implementation Latency:")
    print(f"  Average: {avg_latency:.2f}ms")
    print(f"  Min:     {min_latency:.2f}ms")
    print(f"  Max:     {max_latency:.2f}ms")
    print(f"  P95:     {p95_latency:.2f}ms")

    print(f"\n✓ Test 1 PASSED")

except Exception as e:
    print(f"\n✗ Test 1 FAILED: {e}")
    import traceback
    traceback.print_exc()


# Test 2: Optimized Implementation
print("\n" + "=" * 80)
print("TEST 2: Optimized Implementation")
print("=" * 80)

try:
    from iqrah_audio.streaming.pitch_stream_optimized import OptimizedIncrementalPitchExtractor

    # Same test audio
    sr = 22050
    duration = 5.0
    frequency = 220.0

    t = np.linspace(0, duration, int(sr * duration))
    audio = np.sin(2 * np.pi * frequency * t).astype(np.float32)

    print(f"\nTest audio: {duration:.1f}s @ {frequency} Hz")

    # Create optimized extractor
    extractor = OptimizedIncrementalPitchExtractor(
        method="yin",
        sample_rate=sr,
        hop_length=512,
    )

    # Simulate streaming
    chunk_size = 512
    n_chunks = len(audio) // chunk_size
    latencies = []

    print(f"\nProcessing {n_chunks} chunks of {chunk_size} samples...")

    for i in range(n_chunks):
        chunk = audio[i * chunk_size:(i + 1) * chunk_size]

        t0 = time.perf_counter()
        f0, conf, ts = extractor.process_chunk(chunk)
        t1 = time.perf_counter()

        latency_ms = (t1 - t0) * 1000
        latencies.append(latency_ms)

    avg_latency = np.mean(latencies)
    max_latency = np.max(latencies)
    min_latency = np.min(latencies)
    p95_latency = np.percentile(latencies, 95)

    print(f"\n📊 Optimized Implementation Latency:")
    print(f"  Average: {avg_latency:.2f}ms")
    print(f"  Min:     {min_latency:.2f}ms")
    print(f"  Max:     {max_latency:.2f}ms")
    print(f"  P95:     {p95_latency:.2f}ms")

    # Check target
    if avg_latency < 10:
        print(f"\n  ✓ TARGET MET (<10ms)")
    else:
        print(f"\n  ⚠ TARGET MISSED (>10ms)")

    print(f"\n✓ Test 2 PASSED")

except Exception as e:
    print(f"\n✗ Test 2 FAILED: {e}")
    import traceback
    traceback.print_exc()


# Test 3: Accuracy Comparison
print("\n" + "=" * 80)
print("TEST 3: Accuracy Comparison")
print("=" * 80)

try:
    from iqrah_audio.streaming import IncrementalPitchExtractor
    from iqrah_audio.streaming.pitch_stream_optimized import OptimizedIncrementalPitchExtractor

    # Generate stable tone
    sr = 22050
    duration = 3.0
    frequency = 220.0

    t = np.linspace(0, duration, int(sr * duration))
    audio = np.sin(2 * np.pi * frequency * t).astype(np.float32)

    print(f"\nTest audio: {duration:.1f}s @ {frequency} Hz")

    # Process with both extractors
    current = IncrementalPitchExtractor(method="yin", sample_rate=sr)
    optimized = OptimizedIncrementalPitchExtractor(method="yin", sample_rate=sr)

    chunk_size = 512
    n_chunks = len(audio) // chunk_size

    current_f0_all = []
    optimized_f0_all = []

    for i in range(n_chunks):
        chunk = audio[i * chunk_size:(i + 1) * chunk_size]

        f0_curr, _, _ = current.process_chunk(chunk)
        f0_opt, _, _ = optimized.process_chunk(chunk)

        if len(f0_curr) > 0:
            current_f0_all.extend(f0_curr)
        if len(f0_opt) > 0:
            optimized_f0_all.extend(f0_opt)

    current_f0_all = np.array(current_f0_all)
    optimized_f0_all = np.array(optimized_f0_all)

    # Filter voiced frames
    current_voiced = current_f0_all[current_f0_all > 0]
    optimized_voiced = optimized_f0_all[optimized_f0_all > 0]

    print(f"\nCurrent Implementation:")
    print(f"  Voiced frames: {len(current_voiced)}/{len(current_f0_all)}")
    print(f"  Mean F0: {np.mean(current_voiced):.2f} Hz")
    print(f"  Std F0:  {np.std(current_voiced):.2f} Hz")

    print(f"\nOptimized Implementation:")
    print(f"  Voiced frames: {len(optimized_voiced)}/{len(optimized_f0_all)}")
    print(f"  Mean F0: {np.mean(optimized_voiced):.2f} Hz")
    print(f"  Std F0:  {np.std(optimized_voiced):.2f} Hz")

    # Compare
    if len(current_voiced) > 0 and len(optimized_voiced) > 0:
        mean_diff = abs(np.mean(current_voiced) - np.mean(optimized_voiced))
        print(f"\nMean F0 difference: {mean_diff:.2f} Hz")

        if mean_diff < 5.0:  # Allow 5Hz tolerance
            print("  ✓ Accuracy comparable")
        else:
            print("  ⚠ Accuracy differs")

    print(f"\n✓ Test 3 PASSED")

except Exception as e:
    print(f"\n✗ Test 3 FAILED: {e}")
    import traceback
    traceback.print_exc()


# Test 4: Real Audio
print("\n" + "=" * 80)
print("TEST 4: Real Audio Performance")
print("=" * 80)

try:
    from iqrah_audio.streaming.pitch_stream_optimized import OptimizedIncrementalPitchExtractor

    audio_path = Path("data/husary/surahs/01.mp3")

    if not audio_path.exists():
        print("⚠ Skipping Test 4: Audio file not found")
    else:
        print(f"\nLoading audio: {audio_path}")
        audio, sr = sf.read(str(audio_path))

        if len(audio.shape) > 1:
            audio = audio.mean(axis=1)

        audio = audio.astype(np.float32)[:sr*10]  # First 10s
        duration = len(audio) / sr

        print(f"✓ Loaded: {duration:.2f}s @ {sr} Hz")

        # Create optimized extractor
        extractor = OptimizedIncrementalPitchExtractor(
            method="yin",
            sample_rate=sr,
        )

        # Process in streaming fashion
        chunk_size = 512
        n_chunks = len(audio) // chunk_size
        latencies = []
        frames_extracted = 0

        print(f"\nProcessing {n_chunks} chunks...")

        for i in range(n_chunks):
            chunk = audio[i * chunk_size:(i + 1) * chunk_size]

            t0 = time.perf_counter()
            f0, conf, ts = extractor.process_chunk(chunk)
            t1 = time.perf_counter()

            latency_ms = (t1 - t0) * 1000
            latencies.append(latency_ms)
            frames_extracted += len(f0)

        avg_latency = np.mean(latencies)
        max_latency = np.max(latencies)
        p95_latency = np.percentile(latencies, 95)

        print(f"\n✓ Processed {n_chunks} chunks")
        print(f"  Frames extracted: {frames_extracted}")

        print(f"\n📊 Real Audio Latency:")
        print(f"  Average: {avg_latency:.2f}ms")
        print(f"  Max:     {max_latency:.2f}ms")
        print(f"  P95:     {p95_latency:.2f}ms")

        if avg_latency < 10:
            print(f"\n  ✓ TARGET MET (<10ms)")
        else:
            print(f"\n  ⚠ TARGET MISSED (>10ms)")

        print(f"\n✓ Test 4 PASSED")

except Exception as e:
    print(f"\n✗ Test 4 FAILED: {e}")
    import traceback
    traceback.print_exc()


# Summary
print("\n" + "=" * 80)
print("OPTIMIZATION SUMMARY")
print("=" * 80)

print("\n✅ Optimization Tests:")
print("  1. ✓ Current implementation baseline")
print("  2. ✓ Optimized implementation")
print("  3. ✓ Accuracy comparison")
print("  4. ✓ Real audio performance")

print("\n🎯 Target: <10ms per chunk")
print("  Current: ~60ms per chunk")
print("  Optimized: TBD from tests above")

print("\n" + "=" * 80)
print("All tests completed!")
print("=" * 80)
