[↑ Navigation](../NAVIGATION.md)

# Infrastructure & DevOps

**Purpose**: Infrastructure specifications and DevOps configurations
**Includes**: Latency breakdown, Redis caching schema, Docker configuration

---

## **LOCAL INFERENCE TARGET (MVP)**

**Status**: Current implementation optimized for consumer-grade hardware.

### Hardware Target

The MVP is designed to run on a **single NVIDIA RTX 3060-Ti** (or equivalent):

**Specifications**:
- VRAM: 8 GB GDDR6
- CUDA Cores: 4864
- Tensor Cores: 152 (3rd gen)
- Memory Bandwidth: 448 GB/s
- TDP: 200W
- Compute Capability: 8.6

**Rationale**: The RTX 3060-Ti represents a mainstream consumer GPU (MSRP ~$400) that is widely available and affordable for educational institutions and individual developers. This ensures the MVP is accessible rather than requiring expensive enterprise hardware.

### Model Configuration

**ASR Model**: `obadx/muaalem-model-v3_2`
- Precision: **FP16** (half-precision)
- VRAM Usage: ~2.5 GB (with batch size 1)
- Inference Time: ~0.8-1.2s per 15-second segment (FP16 + CUDA)
- **Chunking**: Inputs >20s are split into 20s windows with 0.4s stride

**Optional Waqf Segmenter**: `obadx/recitation-segmenter-v2`
- Type: `AutoModelForAudioFrameClassification` (NOT used for transcription)
- Precision: FP16
- Usage: Pre-segment very long audio (>60s) at pause boundaries before ASR
- VRAM Usage: ~1.2 GB

**Total VRAM Budget**: ~3.7 GB (leaves 4.3 GB headroom for audio tensors and gradients)

### Precision Strategy

```python
# PyTorch FP16 autocast
import torch

with torch.cuda.amp.autocast():
    # All model inference runs in FP16
    logits = asr_model(audio_tensor)
    segmentation = waqf_model(audio_tensor)
```

**Benefits**:
- 2× memory reduction vs FP32
- ~1.7× speedup on Tensor Cores
- Negligible accuracy loss for CTC models

**NOT Used**:
- INT8 quantization: Requires additional calibration, deferred to post-MVP
- Dynamic batching: Adds complexity, deferred to Phase 2

### Latency Breakdown (MVP)

```
Component                     Target (ms)   Implementation
================================================================
1. Audio load & preprocess    20-40         librosa + resampling
2. ASR transcription           800-1200      muaalem-model-v3_2 (FP16)
   - Chunking (if >20s)        +200-400      Overlapping windows
3. Content verification        5-10          rapidfuzz Levenshtein
4. CTC forced alignment        50-100        Lightweight Viterbi
5. LLR confidence scoring      10-20         NumPy operations
6. Tajweed validation          15-30         Duration + energy rules
   - Madd                      5-10
   - Shadda                    5-10
   - Waqf                      5-10
7. JSON serialization          5-10          Standard library
================================================================
TOTAL (15s audio segment)      900-1410ms    <1.5s target ✓
```

**Critical**: The MVP targets **≤1.3 seconds** total latency for a 15-second audio segment. This provides acceptable responsiveness for an educational tool.

### FP16 Configuration Example

```python
import torch
from transformers import Wav2Vec2ForCTC, Wav2Vec2Processor

# Load model
model_name = "obadx/muaalem-model-v3_2"
processor = Wav2Vec2Processor.from_pretrained(model_name)
model = Wav2Vec2ForCTC.from_pretrained(model_name)

# Move to CUDA and convert to FP16
device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
model = model.to(device).half()  # Convert to FP16
model.eval()

# Inference with autocast
with torch.no_grad(), torch.cuda.amp.autocast():
    inputs = processor(audio, sampling_rate=16000, return_tensors="pt")
    inputs = {k: v.to(device) for k, v in inputs.items()}
    logits = model(**inputs).logits  # FP16 computation
```

### Chunking Strategy (Audio >20s)

```python
def chunk_audio(audio: np.ndarray, sr: int = 16000,
                chunk_duration: float = 20.0, stride: float = 0.4):
    """
    Split long audio into overlapping chunks for ASR.

    Args:
        audio: Audio array (mono)
        sr: Sample rate
        chunk_duration: Chunk size in seconds
        stride: Overlap between chunks in seconds

    Returns:
        List of (chunk_audio, start_time, end_time) tuples
    """
    chunk_samples = int(chunk_duration * sr)
    stride_samples = int(stride * sr)

    chunks = []
    for start in range(0, len(audio), chunk_samples - stride_samples):
        end = min(start + chunk_samples, len(audio))
        chunk = audio[start:end]

        if len(chunk) > sr:  # Skip chunks <1s
            chunks.append((chunk, start/sr, end/sr))

    return chunks
```

**Rationale**: Wav2Vec2-BERT models have positional embeddings tuned for ~20-30s segments. Longer inputs degrade accuracy and increase VRAM usage. Chunking with overlap prevents boundary artifacts.

### Testing Requirements

**Hardware Validation**:
- Test on RTX 3060-Ti (8GB VRAM)
- Test on RTX 2060 (6GB VRAM) as minimum viable hardware
- Test on CPU (fallback, expect ~3-4× slower)

**Latency Benchmarks**:
- 5s audio → <600ms total
- 15s audio → <1300ms total
- 30s audio (chunked) → <2000ms total

**Memory Profiling**:
- Peak VRAM usage must not exceed 6GB (for 6GB GPU compatibility)
- Use `torch.cuda.max_memory_allocated()` to monitor

---

> **MVP Reason Note**: These specifications guarantee a responsive user experience on common consumer hardware. The RTX 3060-Ti provides an excellent price/performance ratio for educational applications. FP16 precision offers significant speedup with negligible accuracy loss. Chunking ensures the system handles both short practice sessions and full Surah recitations without memory issues.

---

## Detailed Latency Breakdown (Phase 2)

### Target Latency per Component (GPU)

```
Component                    Target (ms)   Notes
====================================================
1. WebSocket receive         10-20         Network overhead
2. Audio decode              5-10          Base64 → NumPy
3. VAD segmentation          10-15         Silero VAD (ONNX)
4. Pitch extraction          20-30         SwiftF0 on GPU
5. Phoneme alignment         150-200       Wav2Vec2-BERT INT8
6. Tajweed validation        20-30         All 3 validators
   - Madd                    5-10
   - Ghunnah                 5-10
   - Qalqalah                5-10
7. Prosody analysis          80-100        OpenSMILE + features
   - OpenSMILE               50-60
   - nPVI/Varco              10-15
   - Maqam CNN               20-25
8. Comparison engine         30-50         Multi-dimensional fusion
9. Feedback generation       20-30         Template rendering
10. JSON serialization       10-20         Response encoding
11. WebSocket send           10-20         Network overhead
====================================================
TOTAL                        400-500ms     Within <500ms target
```

### Optimization Priorities

**Critical Path (must optimize)**:
1. Phoneme alignment (150-200ms) - INT8 quantization essential
2. Prosody analysis (80-100ms) - Can parallelize with GPU
3. Comparison engine (30-50ms) - Cache reference features

**Acceptable**:
- Pitch extraction (20-30ms) - SwiftF0 already fast
- Tajweed validation (20-30ms) - Pure NumPy, fast enough
- Feedback (20-30ms) - I/O bound, acceptable

---

## Redis Caching Schema

```python
# Key format
CACHE_KEY_FORMAT = "{model_version}:{surah}:{ayah}"

# Example key
"v1.2.0:1:1"  # Al-Fatiha, Ayah 1, model v1.2.0

# Value format (JSON)
{
    "pitch": {
        "pitch_hz": [float, ...],
        "times": [float, ...],
        "stats": {...}
    },
    "phonemes": [
        {"phoneme": str, "start": float, "end": float, ...},
        ...
    ],
    "prosody": {
        "rhythm": {...},
        "melody": {...},
        "style": {...}
    },
    "voice_quality": {...},
    "computed_at": "2025-10-23T12:00:00Z",
    "model_version": "v1.2.0"
}

# TTL: 30 days (2,592,000 seconds)
# Size per entry: ~50-100KB
# Total cache size: 6,236 × 75KB ≈ 467MB
```

---

## Docker Configuration

```dockerfile
# Dockerfile for Phase 2 Real-Time Server
FROM nvidia/cuda:12.1.0-runtime-ubuntu22.04

# Install Python 3.10
RUN apt-get update && apt-get install -y python3.10 python3-pip

# Install dependencies
COPY requirements.txt /app/
RUN pip install --no-cache-dir -r /app/requirements.txt

# Install ONNX Runtime with GPU support
RUN pip install onnxruntime-gpu==1.16.0

# Copy application
COPY src/ /app/src/
COPY models/ /app/models/

# Set working directory
WORKDIR /app

# Expose ports
EXPOSE 8000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8000/health || exit 1

# Run application
CMD ["uvicorn", "src.main:app", "--host", "0.0.0.0", "--port", "8000"]
```

---

## Mermaid Graph Styling

```mermaid
style M1 fill:#90EE90    # Light green (complete/mostly done)
style M2 fill:#90EE90
style M3 fill:#FFD700    # Gold (in progress)
style M4 fill:#FFA500    # Orange (not started, Phase 1)
style M5 fill:#FFA500
style M6 fill:#FFA500
style M7 fill:#FFD700
style M8 fill:#FFA500
style V1 fill:#FF6347    # Red (critical path)

style RT1 fill:#D3D3D3   # Gray (future Phase 2)
style RT2 fill:#D3D3D3
style RT3 fill:#D3D3D3
style RT4 fill:#D3D3D3
style RT5 fill:#D3D3D3

style MB1 fill:#D3D3D3   # Gray (future Phase 3)
style MB2 fill:#D3D3D3
style MB3 fill:#D3D3D3
style MB4 fill:#D3D3D3
```

**Legend**:
- 🟢 Green (#90EE90): Complete/Mostly Done
- 🟡 Yellow (#FFD700): In Progress
- 🟠 Orange (#FFA500): Not Started (Phase 1)
- ⚪ Gray (#D3D3D3): Future (Phase 2-3)
- 🔴 Red (#FF6347): Critical Path

---

**Related**: See main [Architecture](../01-architecture/overview.md) docs
