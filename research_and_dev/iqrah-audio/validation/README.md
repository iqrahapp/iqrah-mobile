# Validation & Benchmarking Harness

This directory contains the validation and benchmarking infrastructure for Iqrah Audio.

## Overview

The validation harness provides:
- **Unit test orchestration** via pytest
- **Performance benchmarks** for each module
- **Reference data** for regression testing
- **CI/CD integration** via `run_validation.py`

## Directory Structure

```
validation/
├── run_validation.py          # Main validation script for CI/CD
├── benchmarks/                 # Performance benchmark tests
│   └── test_m1_benchmark.py    # M1 preprocessing benchmarks
├── data/                       # Reference audio and expected outputs
│   ├── generate_reference_audio.py
│   ├── metadata.json
│   └── *.wav                   # Reference audio files
└── results.json                # Latest validation results (generated)
```

## Usage

### Run All Tests and Benchmarks

```bash
python validation/run_validation.py
```

### Run Specific Test Suites

```bash
# Unit tests only
python validation/run_validation.py --unit

# Benchmarks only
python validation/run_validation.py --benchmark

# Specific module
python validation/run_validation.py --module m1

# Verbose output
python validation/run_validation.py --verbose
```

### CI/CD Integration

The validation script is designed for CI/CD environments:

```bash
# In GitHub Actions, CircleCI, etc.
python validation/run_validation.py --save-results validation/results.json
```

Exit codes:
- `0`: All tests passed
- Non-zero: At least one test failed

## Reference Audio Files

Reference audio files are generated using `validation/data/generate_reference_audio.py`:

- **clean_30s.wav**: 30-second clean speech-like audio for M1 performance benchmarks
- **clean_5s.wav**: 5-second audio for quick validation tests
- **noisy_10s.wav**: 10-second noisy audio for quality metric validation
- **high_sr_3s.wav**: 48kHz audio for resampling tests

### Regenerate Reference Audio

```bash
python validation/data/generate_reference_audio.py
```

## Benchmark Thresholds

### Module M1: Audio Preprocessing

Per [M1 spec](../doc/01-architecture/m1-preprocessing.md):
- **Offline latency**: 200-500ms per minute of audio
- **30s audio benchmark**: < 2 seconds (without noise reduction)
- **Throughput**: > 1x realtime
- **Memory**: < 100 MB for 30s audio

## Adding New Benchmarks

To add benchmarks for a new module:

1. Create `validation/benchmarks/test_mN_benchmark.py`
2. Use reference data from `validation/data/`
3. Follow the pattern in `test_m1_benchmark.py`
4. Add performance thresholds based on module spec

Example:

```python
def test_m2_pitch_extraction_performance(clean_30s_audio):
    """Test M2 can extract pitch from 30s audio in < 3 seconds."""
    start_time = time.time()

    result = extract_pitch(clean_30s_audio)

    elapsed = time.time() - start_time

    assert elapsed < 3.0, \
        f"M2 took {elapsed:.3f}s (threshold: 3.0s)"
```

## Results Output

The validation script saves results to `validation/results.json`:

```json
{
  "timestamp": 1698765432.0,
  "tests": {
    "exit_code": 0,
    "status": "passed"
  },
  "benchmarks": {
    "exit_code": 0,
    "status": "passed"
  },
  "summary": {}
}
```

## Best Practices

1. **Keep benchmarks fast**: Aim for < 30s total benchmark runtime
2. **Use reference data**: Don't generate audio in tests (use pre-generated)
3. **Set realistic thresholds**: Based on module specs in `doc/01-architecture/`
4. **Test edge cases**: Include validation for corner cases
5. **Monitor trends**: Track performance over time in CI/CD

## Troubleshooting

### Benchmarks are slow

- Check if Silero VAD is downloading models (first run)
- Ensure `iqrah` conda environment is activated
- Run with `--benchmark` flag only to skip unit tests

### Reference audio missing

```bash
python validation/data/generate_reference_audio.py
```

### Import errors

```bash
# Ensure package is installed
pip install -e .

# Activate correct environment
conda activate iqrah
```

## Future Enhancements

- [ ] Add regression testing (compare against previous results)
- [ ] Add visualization of benchmark trends
- [ ] Add integration tests for multi-module pipelines
- [ ] Add profiling reports (memory, CPU)
- [ ] Add quality metrics tracking over time
