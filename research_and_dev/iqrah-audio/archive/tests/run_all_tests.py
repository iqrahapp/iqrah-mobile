#!/usr/bin/env python3
"""
Test Runner with Benchmarks and Regression Tracking
===================================================

Runs all tests and tracks precision metrics over time.
"""

import unittest
import sys
import time
import json
from pathlib import Path
from datetime import datetime

# Test discovery
TEST_DIR = Path(__file__).parent
PROJECT_ROOT = TEST_DIR.parent


def run_test_suite(verbosity=2):
    """Run all unit and system tests"""

    print("=" * 80)
    print("IQRAH AUDIO TEST SUITE")
    print("=" * 80)
    print(f"Started: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"Test directory: {TEST_DIR}")
    print("=" * 80)

    # Discover and load tests
    loader = unittest.TestLoader()
    start_dir = str(TEST_DIR)

    # Load test modules
    test_modules = [
        'test_pitch_swiftf0',
        'test_dtw_alignment',
        'test_metrics',
        'test_offline_pipeline'
    ]

    suite = unittest.TestSuite()

    print("\n📦 Loading test modules...")
    for module_name in test_modules:
        try:
            tests = loader.loadTestsFromName(module_name)
            test_count = tests.countTestCases()
            suite.addTests(tests)
            print(f"  ✓ {module_name}: {test_count} tests")
        except Exception as e:
            print(f"  ✗ {module_name}: Failed to load - {e}")

    total_tests = suite.countTestCases()
    print(f"\n📊 Total tests loaded: {total_tests}")

    # Run tests
    print("\n" + "=" * 80)
    print("RUNNING TESTS")
    print("=" * 80 + "\n")

    runner = unittest.TextTestRunner(verbosity=verbosity)
    start_time = time.time()
    result = runner.run(suite)
    elapsed_time = time.time() - start_time

    # Print summary
    print("\n" + "=" * 80)
    print("TEST SUMMARY")
    print("=" * 80)
    print(f"\nRan {result.testsRun} tests in {elapsed_time:.3f}s")
    print(f"\n  ✓ Passed:  {result.testsRun - len(result.failures) - len(result.errors)}")
    print(f"  ✗ Failed:  {len(result.failures)}")
    print(f"  ⚠ Errors:  {len(result.errors)}")
    print(f"  ⊘ Skipped: {len(result.skipped)}")

    # Show failures
    if result.failures:
        print("\n" + "=" * 80)
        print("FAILURES")
        print("=" * 80)
        for test, traceback in result.failures:
            print(f"\n✗ {test}")
            print(traceback)

    # Show errors
    if result.errors:
        print("\n" + "=" * 80)
        print("ERRORS")
        print("=" * 80)
        for test, traceback in result.errors:
            print(f"\n⚠ {test}")
            print(traceback)

    # Overall status
    print("\n" + "=" * 80)
    if result.wasSuccessful():
        print("✅ ALL TESTS PASSED")
    else:
        print("❌ SOME TESTS FAILED")
    print("=" * 80)

    # Save results
    save_test_results(result, elapsed_time)

    return result


def save_test_results(result, elapsed_time):
    """Save test results for regression tracking"""

    results_dir = PROJECT_ROOT / "test_results"
    results_dir.mkdir(exist_ok=True)

    timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
    results_file = results_dir / f"results_{timestamp}.json"

    data = {
        'timestamp': datetime.now().isoformat(),
        'total_tests': result.testsRun,
        'passed': result.testsRun - len(result.failures) - len(result.errors),
        'failed': len(result.failures),
        'errors': len(result.errors),
        'skipped': len(result.skipped),
        'elapsed_time': elapsed_time,
        'success': result.wasSuccessful()
    }

    with open(results_file, 'w') as f:
        json.dump(data, f, indent=2)

    print(f"\n💾 Results saved to: {results_file}")

    # Update latest results
    latest_file = results_dir / "latest.json"
    with open(latest_file, 'w') as f:
        json.dump(data, f, indent=2)


def run_benchmarks():
    """Run performance benchmarks"""

    print("\n" + "=" * 80)
    print("PERFORMANCE BENCHMARKS")
    print("=" * 80)

    benchmarks = []

    # Benchmark 1: SwiftF0 pitch extraction
    try:
        from test_pitch_swiftf0 import TestSwiftF0Precision
        import tempfile
        import soundfile as sf
        import numpy as np

        print("\n📊 Benchmark 1: SwiftF0 Pitch Extraction Speed")

        test = TestSwiftF0Precision()
        sr = 16000
        durations = [1.0, 3.0, 5.0, 10.0]  # seconds

        for duration in durations:
            # Generate test audio
            t = np.linspace(0, duration, int(sr * duration))
            audio = np.sin(2 * np.pi * 200 * t).astype(np.float32)

            with tempfile.NamedTemporaryFile(suffix='.wav', delete=False) as f:
                sf.write(f.name, audio, sr)

                # Benchmark
                from iqrah_audio.analysis.pitch_extractor import extract_pitch_from_file
                start = time.perf_counter()
                result = extract_pitch_from_file(f.name, sr=sr)
                elapsed = time.perf_counter() - start

                realtime_factor = elapsed / duration
                print(f"  {duration:4.1f}s audio → {elapsed:6.3f}s ({realtime_factor:.3f}× realtime)")

                benchmarks.append({
                    'name': f'pitch_extraction_{duration}s',
                    'duration': duration,
                    'elapsed': elapsed,
                    'realtime_factor': realtime_factor
                })

                Path(f.name).unlink()

    except Exception as e:
        print(f"  ⚠ Benchmark 1 failed: {e}")

    # Benchmark 2: DTW alignment
    try:
        print("\n📊 Benchmark 2: DTW Alignment Speed")

        from iqrah_audio.analysis.offline import calculate_dtw_alignment
        import numpy as np

        sequence_lengths = [100, 300, 500, 1000]

        for length in sequence_lengths:
            # Generate random pitch sequences
            user_pitch = list(np.random.uniform(100, 300, length))
            ref_pitch = list(np.random.uniform(100, 300, length))

            # Benchmark
            start = time.perf_counter()
            result = calculate_dtw_alignment(user_pitch, ref_pitch)
            elapsed = time.perf_counter() - start

            print(f"  {length:4d} frames → {elapsed*1000:6.2f}ms")

            benchmarks.append({
                'name': f'dtw_alignment_{length}frames',
                'frames': length,
                'elapsed': elapsed
            })

    except Exception as e:
        print(f"  ⚠ Benchmark 2 failed: {e}")

    # Benchmark 3: End-to-end pipeline
    try:
        print("\n📊 Benchmark 3: End-to-End Pipeline")

        import tempfile
        import soundfile as sf
        import numpy as np
        from iqrah_audio.analysis.offline import analyze_recitation

        durations = [1.0, 3.0, 5.0]

        for duration in durations:
            sr = 16000
            t = np.linspace(0, duration, int(sr * duration))
            audio = np.sin(2 * np.pi * 200 * t).astype(np.float32)

            temp_dir = tempfile.mkdtemp()
            ref_path = Path(temp_dir) / "ref.wav"
            user_path = Path(temp_dir) / "user.wav"

            sf.write(str(ref_path), audio, sr)
            sf.write(str(user_path), audio, sr)

            # Create segments
            num_words = int(duration * 2)  # 2 words per second
            segments = [
                {
                    'word': f'word{i}',
                    'start_ms': int(i * (duration * 1000 / num_words)),
                    'end_ms': int((i + 1) * (duration * 1000 / num_words)),
                    'word_idx': i
                }
                for i in range(num_words)
            ]

            # Benchmark
            start = time.perf_counter()
            result = analyze_recitation(
                user_audio_path=str(user_path),
                reference_audio_path=str(ref_path),
                segments=segments
            )
            elapsed = time.perf_counter() - start

            realtime_factor = elapsed / duration
            print(f"  {duration:4.1f}s audio → {elapsed:6.3f}s ({realtime_factor:.3f}× realtime, {num_words} words)")

            benchmarks.append({
                'name': f'pipeline_{duration}s',
                'duration': duration,
                'elapsed': elapsed,
                'realtime_factor': realtime_factor,
                'num_words': num_words
            })

            # Cleanup
            ref_path.unlink()
            user_path.unlink()
            Path(temp_dir).rmdir()

    except Exception as e:
        print(f"  ⚠ Benchmark 3 failed: {e}")

    # Save benchmarks
    if benchmarks:
        results_dir = PROJECT_ROOT / "test_results"
        results_dir.mkdir(exist_ok=True)

        timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
        benchmark_file = results_dir / f"benchmarks_{timestamp}.json"

        data = {
            'timestamp': datetime.now().isoformat(),
            'benchmarks': benchmarks
        }

        with open(benchmark_file, 'w') as f:
            json.dump(data, f, indent=2)

        print(f"\n💾 Benchmarks saved to: {benchmark_file}")

    print("\n" + "=" * 80)


def check_regressions():
    """Check for performance regressions"""

    print("\n" + "=" * 80)
    print("REGRESSION CHECK")
    print("=" * 80)

    results_dir = PROJECT_ROOT / "test_results"
    if not results_dir.exists():
        print("\n⚠ No previous test results found")
        return

    # Load latest results
    latest_file = results_dir / "latest.json"
    if not latest_file.exists():
        print("\n⚠ No latest.json found")
        return

    with open(latest_file) as f:
        latest = json.load(f)

    # Load all previous results
    previous_results = sorted(results_dir.glob("results_*.json"))
    if len(previous_results) < 2:
        print("\n⚠ Not enough historical data for regression check")
        return

    # Load second-to-last result
    with open(previous_results[-2]) as f:
        previous = json.load(f)

    print(f"\nComparing:")
    print(f"  Current:  {latest['timestamp']}")
    print(f"  Previous: {previous['timestamp']}")

    # Check test count regression
    if latest['total_tests'] < previous['total_tests']:
        print(f"\n⚠ REGRESSION: Test count decreased from {previous['total_tests']} to {latest['total_tests']}")

    # Check pass rate regression
    prev_pass_rate = previous['passed'] / previous['total_tests'] * 100 if previous['total_tests'] > 0 else 0
    curr_pass_rate = latest['passed'] / latest['total_tests'] * 100 if latest['total_tests'] > 0 else 0

    print(f"\nPass Rate:")
    print(f"  Previous: {prev_pass_rate:.1f}%")
    print(f"  Current:  {curr_pass_rate:.1f}%")

    if curr_pass_rate < prev_pass_rate - 5:  # 5% threshold
        print(f"  ⚠ REGRESSION: Pass rate decreased by {prev_pass_rate - curr_pass_rate:.1f}%")

    # Check performance regression
    if 'elapsed_time' in previous and 'elapsed_time' in latest:
        prev_time = previous['elapsed_time']
        curr_time = latest['elapsed_time']
        time_diff_pct = (curr_time - prev_time) / prev_time * 100

        print(f"\nExecution Time:")
        print(f"  Previous: {prev_time:.3f}s")
        print(f"  Current:  {curr_time:.3f}s")
        print(f"  Change:   {time_diff_pct:+.1f}%")

        if time_diff_pct > 20:  # 20% slower threshold
            print(f"  ⚠ REGRESSION: Execution time increased by {time_diff_pct:.1f}%")

    print("\n" + "=" * 80)


if __name__ == '__main__':
    # Add project to path
    sys.path.insert(0, str(PROJECT_ROOT / "src"))
    sys.path.insert(0, str(TEST_DIR))

    # Parse args
    verbosity = 2
    run_bench = '--bench' in sys.argv or '-b' in sys.argv

    # Run tests
    result = run_test_suite(verbosity=verbosity)

    # Run benchmarks if requested
    if run_bench:
        run_benchmarks()

    # Check regressions
    check_regressions()

    # Exit with appropriate code
    sys.exit(0 if result.wasSuccessful() else 1)
