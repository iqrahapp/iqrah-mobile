# Iqrah Audio Test Suite

Comprehensive test suite for the Iqrah Audio recitation analysis system with precision metrics and regression tracking.

## Test Structure

### Unit Tests

1. **test_pitch_swiftf0.py** - SwiftF0 Pitch Extraction
   - Basic pitch extraction validation
   - Accuracy tests across male/female voice ranges (80-400 Hz)
   - Harmonic confusion detection
   - Chirp pitch tracking
   - Silence detection
   - Noise robustness (10 dB SNR)
   - Temporal resolution verification
   - Confidence score validation
   - Precision benchmarks with regression tracking

2. **test_dtw_alignment.py** - DTW Alignment
   - Identical sequence alignment
   - Time-stretched sequence handling
   - Pitch variation tolerance
   - Pitch shift alignment
   - Insertion/deletion handling
   - Unvoiced frames handling
   - Edge cases (empty, single frame, very different lengths)
   - Bidirectional mapping consistency
   - Monotonicity verification
   - Performance benchmarks

3. **test_metrics.py** - Metrics Calculation
   - **Word-level pitch accuracy**:
     - Perfect pitch match
     - Small pitch variations
     - Large pitch errors
     - Missing words
     - Unvoiced words
     - DTW mapping usage
   - **Stability metrics**:
     - Stable pitch (low jitter)
     - Unstable pitch (high jitter)
     - Smooth variation (melodic)
     - Silence handling
     - Mixed voiced/unvoiced
   - **Complexity metrics**:
     - Simple melody detection
     - Complex melody detection
     - Single note identification
   - **Overall scoring**:
     - Perfect score validation
     - Poor score validation
     - Missing words penalty
     - Score bounds [0, 100]

### System Tests

4. **test_offline_pipeline.py** - End-to-End Pipeline
   - **End-to-end workflows**:
     - Identical audio analysis
     - Time-stretched audio (tempo variation)
     - Pitch-shifted audio (key transposition)
     - Melody variation
     - Noise robustness (15 dB SNR)
   - **Component integration**:
     - Pitch extraction → DTW → Metrics
     - Stability metrics calculation
     - Complexity metrics calculation
   - **Data flow validation**:
     - Pitch data structure
     - Alignment data structure
   - **Error handling**:
     - Invalid audio paths
     - Corrupted audio files
     - Empty segments
   - **Performance benchmarks**:
     - 3-second audio analysis <5s

## Running Tests

### Run All Tests
```bash
python tests/run_all_tests.py
```

### Run with Benchmarks
```bash
python tests/run_all_tests.py --bench
```

### Run Specific Test Module
```bash
python -m unittest tests.test_pitch_swiftf0
python -m unittest tests.test_dtw_alignment
python -m unittest tests.test_metrics
python -m unittest tests.test_offline_pipeline
```

### Run Specific Test Class
```bash
python -m unittest tests.test_pitch_swiftf0.TestSwiftF0PitchExtractor
python -m unittest tests.test_metrics.TestPitchAccuracyPerWord
```

### Run Specific Test Method
```bash
python -m unittest tests.test_pitch_swiftf0.TestSwiftF0PitchExtractor.test_pitch_accuracy_male_voice_range
```

## Test Results

Test results are saved to `test_results/` directory:

- `results_YYYYMMDD_HHMMSS.json` - Individual test run results
- `benchmarks_YYYYMMDD_HHMMSS.json` - Performance benchmarks
- `latest.json` - Most recent test results

### Result Structure

```json
{
  "timestamp": "2025-10-06T12:00:00",
  "total_tests": 38,
  "passed": 35,
  "failed": 2,
  "errors": 1,
  "skipped": 0,
  "elapsed_time": 6.789,
  "success": false
}
```

## Regression Tracking

The test runner automatically compares current results with previous runs:

- **Test count regression**: Warns if total tests decrease
- **Pass rate regression**: Warns if pass rate drops >5%
- **Performance regression**: Warns if execution time increases >20%

## Precision Metrics

### SwiftF0 Accuracy Benchmarks

Expected performance across Quranic recitation range (80-400 Hz):

- **Mean Error**: <15 Hz
- **Mean Error**: <50 cents
- **Max Error**: <30 Hz

### DTW Performance Benchmarks

Expected performance for typical audio lengths:

- **100 frames** (1s @ 10ms hop): <10ms
- **300 frames** (3s): <100ms
- **1000 frames** (10s): <1000ms

### End-to-End Pipeline Benchmarks

Expected performance:

- **1s audio**: <2s (2× realtime)
- **3s audio**: <5s (1.67× realtime)
- **5s audio**: <8s (1.6× realtime)

## Continuous Integration

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "Running tests..."
python tests/run_all_tests.py

if [ $? -ne 0 ]; then
    echo "❌ Tests failed. Commit aborted."
    exit 1
fi

echo "✅ All tests passed."
```

### GitHub Actions (Future)

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Set up Python
        uses: actions/setup-python@v2
        with:
          python-version: 3.11
      - name: Install dependencies
        run: |
          pip install -r requirements.txt
      - name: Run tests
        run: python tests/run_all_tests.py
```

## Known Issues

1. **SwiftF0 Installation**: Tests require `pip install swift-f0` (optional, falls back to CREPE)
2. **API Mismatch**: Some tests may need updating to match current `analyze_recitation()` API
3. **Test Data**: Tests use synthetic audio; consider adding real Quranic recitation samples

## Future Improvements

1. **Real Audio Test Data**: Add test fixtures with actual Quranic recitations
2. **Visual Regression**: Compare pitch visualization outputs
3. **Coverage Tracking**: Add code coverage reporting
4. **Property-Based Testing**: Use hypothesis for property-based tests
5. **Stress Testing**: Test with very long audio (1+ hour)
6. **Concurrency Testing**: Test thread safety for real-time pipeline

## Test Philosophy

- **Fast feedback**: Unit tests should run in <10s
- **Comprehensive coverage**: Test all critical paths
- **Regression safety**: Detect performance and accuracy regressions
- **CI-friendly**: Tests should be deterministic and reproducible
- **Real-world scenarios**: System tests use realistic audio patterns

## Updating Tests

When modifying the codebase:

1. **Update tests first** (TDD approach)
2. **Run full test suite** before committing
3. **Update benchmarks** if performance characteristics change
4. **Document new test cases** in this README

## Test Coverage Goals

- **Unit Tests**: >80% line coverage
- **System Tests**: All critical user workflows
- **Regression Tests**: All previously discovered bugs
- **Performance Tests**: All latency-critical operations
