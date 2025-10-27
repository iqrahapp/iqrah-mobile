# Module M1: Audio Preprocessing

[← Back to Overview](overview.md) | [↑ Navigation](../NAVIGATION.md)

---

## M1: AUDIO PREPROCESSING

**Input**: Raw audio (MP3/WAV/WebM/M4A), optional surah/ayah metadata
**Output**:
```python
{
    "audio_path": str,
    "sample_rate": 16000,
    "duration": float,
    "segments": [{"start": float, "end": float, "confidence": float}],
    "quality_metrics": {
        "snr_db": float,
        "clipping_ratio": float,
        "rms_energy": float,
        "quality_flag": str  # "excellent"/"good"/"poor"
    }
}
```

**Note**: The preprocessed 16kHz audio output from M1 is passed to:
- **M2** for pitch extraction
- **M3** for phoneme alignment via Muaalem (after text is converted to phonetic reference by the phonetizer)

---

### M1.1: Audio Loading
- Library: `soundfile` or `librosa`
- Validate format, check corruption
- Extract metadata (original SR, bit depth)

### M1.2: Resampling
- Target: 16kHz (ASR optimal)
- Method: Kaiser windowed sinc interpolation
- Anti-aliasing before downsample

### M1.3: Normalization
- Peak: -1 to +1 dB
- Optional: LUFS for loudness consistency

### M1.4: Noise Reduction (Optional)
- Trigger: SNR < 15 dB
- Method: Spectral subtraction / Wiener filtering
- Library: `noisereduce` or `scipy.signal`
- Risk: Can damage speech if aggressive

### M1.5: VAD (Voice Activity Detection)
- Model: Silero VAD (ONNX, fast)
- Threshold: 0.5 confidence
- Min speech: 250ms
- Min silence: 300ms

### M1.6: Quality Checks
- SNR: Compare signal power voiced vs unvoiced
- Clipping: Count samples at ±1.0
- Dynamic range: Crest factor
- Flags:
  - `excellent`: SNR > 20 dB, no clipping
  - `good`: SNR 10-20 dB, < 1% clipping
  - `poor`: SNR < 10 dB or > 5% clipping

### M1.7: Caching
- Method: SHA256 hash of original
- TTL: Configurable
- Hit/miss logging

**Dependencies**:
```python
soundfile>=0.12.1
librosa>=0.10.0
noisereduce>=3.0.0
silero-vad>=3.1
scipy>=1.10.0
```

**Latency**:
- Offline: 200-500ms per minute
- Real-time: <50ms per chunk

---

**Next**: [Module M2: Pitch Extraction](m2-pitch.md) | [← Back to Overview](overview.md)
