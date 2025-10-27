# Module M5: Voice Quality Analysis

[← Back to Overview](overview.md) | [↑ Navigation](../NAVIGATION.md)

---

## M5: VOICE QUALITY ANALYSIS

**Input**: Preprocessed audio (16kHz), phoneme alignment
**Output**:
```python
{
    "vibrato": {
        "rate_hz": float,           # 4-7 typical
        "extent_semitones": float,  # 0.5-2 typical
        "regularity": float         # 0-1
    },
    "breathiness": {
        "h1_h2_db": float,          # >5 dB = breathy
        "hnr_db": float,            # <10 dB = breathy/hoarse
        "cpp": float                # Cepstral peak prominence
    },
    "roughness": {
        "jitter_percent": float,    # >1.04% = pathological
        "shimmer_percent": float    # >3.81% = pathological
    },
    "timbre": {
        "spectral_centroid_hz": float,   # 2-4k typical
        "spectral_flux": float,
        "spectral_rolloff_hz": float,    # 85% energy point
        "formants": {
            "f1_hz": float,
            "f2_hz": float,
            "f3_hz": float,
            "f4_hz": float
        }
    },
    "embeddings": {
        "x_vector": np.ndarray,      # 512-d
        "wav2vec2_cls": np.ndarray   # 768-d
    }
}
```

### M5.1: OpenSMILE eGeMAPS

**Features**: 88 standardized prosodic dimensions

**Code**:
```python
import opensmile

smile = opensmile.Smile(
    feature_set=opensmile.FeatureSet.eGeMAPSv02,
    feature_level=opensmile.FeatureLevel.Functionals
)

features = smile.process_file("recitation.wav")
# Returns DataFrame with 88 features
```

**Includes**: Jitter, shimmer, HNR, formants, MFCC, spectral features

**Latency**: ~200ms per minute (CPU)

### M5.2: Vibrato Detection

**Code**:
```python
def detect_vibrato(pitch_contour, times):
    """Analyze vibrato characteristics"""
    # Bandpass filter pitch (2-15 Hz vibrato range)
    from scipy.signal import butter, filtfilt
    b, a = butter(4, [2, 15], btype='band', fs=100)  # 100 Hz pitch sampling
    filtered = filtfilt(b, a, pitch_contour)

    # Autocorrelation for rate
    autocorr = np.correlate(filtered, filtered, mode='full')
    autocorr = autocorr[len(autocorr)//2:]
    peaks = find_peaks(autocorr)[0]
    if len(peaks) > 0:
        rate_hz = 100 / peaks[0]
    else:
        rate_hz = 0

    # Extent (amplitude of oscillation)
    extent_semitones = 12 * np.log2(np.ptp(filtered) + 1e-6)

    # Regularity (CV of peak intervals)
    if len(peaks) > 1:
        intervals = np.diff(peaks)
        regularity = 1 - (np.std(intervals) / np.mean(intervals))
    else:
        regularity = 0

    return {
        "rate_hz": rate_hz,
        "extent_semitones": extent_semitones,
        "regularity": max(0, regularity)
    }
```

### M5.3: Breathiness Features

**Code**:
```python
def extract_breathiness(audio, sr=16000):
    """Compute H1-H2, HNR, CPP"""
    sound = parselmouth.Sound(audio, sr)

    # H1-H2 (first two harmonics difference)
    harmonicity = sound.to_harmonicity()
    h1_h2 = harmonicity.values[:, :2].diff(axis=1).mean()

    # HNR (Harmonic-to-Noise Ratio)
    hnr = harmonicity.values.mean()

    # CPP (Cepstral Peak Prominence)
    pitch = sound.to_pitch()
    cpp = pitch.to_cpp()

    return {
        "h1_h2_db": h1_h2,
        "hnr_db": hnr,
        "cpp": cpp
    }
```

### M5.4: X-Vector Embeddings

**Code**:
```python
from speechbrain.pretrained import EncoderClassifier

classifier = EncoderClassifier.from_hparams(
    source="speechbrain/spkrec-xvect-voxceleb",
    savedir="models/xvector"
)

embedding = classifier.encode_batch(audio_tensor)  # 512-d
```

### M5.5: Wav2Vec2 [CLS] Token

**Code**:
```python
from transformers import Wav2Vec2Model, Wav2Vec2Processor

model = Wav2Vec2Model.from_pretrained("facebook/wav2vec2-base")
processor = Wav2Vec2Processor.from_pretrained("facebook/wav2vec2-base")

inputs = processor(audio, sampling_rate=16000, return_tensors="pt")
outputs = model(**inputs)

embedding = outputs.last_hidden_state[:, 0, :].numpy()  # 768-d CLS token
```

**Dependencies**:
```python
opensmile>=3.0.1
praat-parselmouth>=0.4.3
speechbrain>=0.5.16
transformers>=4.35.0
```

**Latency**:
- OpenSMILE: 200ms/min (CPU)
- Parselmouth: 100-200ms/phoneme
- X-vectors: 500ms/ayah (GPU), 2-3s (CPU)
- Wav2Vec2: 300ms/ayah (GPU), 1-2s (CPU)

---

**Next**: [Module M6: Prosodic Analysis](m6-prosody.md) | [← Back to Overview](overview.md)
