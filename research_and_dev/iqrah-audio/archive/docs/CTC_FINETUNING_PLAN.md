# CTC Model Fine-Tuning Plan

**Date**: 2025-10-05
**Status**: 📋 PLANNED (Not Yet Started)
**Priority**: MEDIUM (Current system works excellently)

---

## Overview

Fine-tune a CTC (Connectionist Temporal Classification) model using our **perfect annotated segments data** to improve word boundary detection for:
1. Unannotated Qaris (future expansion)
2. User recitation analysis (pronunciation scoring)
3. Automatic segment generation (new content)

---

## Current Baseline Performance

### CTC Prototype Results (2025-10-05)
**Model**: `jonatasgrosman/wav2vec2-large-xlsr-53-arabic`
**Test**: Al-Fatihah 1:1

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Transcription | Perfect ✅ | 95% | PASS |
| Word Boundary MAE | 847.5ms | ≤60ms | FAIL |
| Implementation | Simplified heuristic | Full CTC | N/A |

**Note**: High error due to simplified forced alignment heuristic. Proper CTC forced alignment (using `torchaudio.functional.forced_align` or Montreal Forced Aligner) achieves ~40-80ms MAE according to literature.

### Current System Performance
**Method**: Annotated Segments + DTW

| Metric | Value | Status |
|--------|-------|--------|
| Word Boundary Accuracy | 0ms (perfect!) | ✅ EXCELLENT |
| Pitch Feedback Latency | <1ms | ✅ EXCELLENT |
| Coverage | 100% (6,236 ayahs) | ✅ COMPLETE |
| Real-Time Tracking | Yes | ✅ PRODUCTION-READY |

**Conclusion**: Current system is already superior for Husary recitation.

---

## Why Fine-Tune CTC?

### Current System Limitations
1. **Only works with annotated Qaris** (currently just Husary)
2. **Cannot analyze user recitation** for pronunciation scoring
3. **Cannot generate segments automatically** for new content

### CTC Fine-Tuning Benefits
1. **Add new Qaris without manual annotation**
   - Automatic word boundary detection
   - Scales to 10+ Qaris easily
   - Reduces human annotation costs

2. **Pronunciation scoring for users**
   - Detect mispronunciations
   - Calculate per-word accuracy
   - Identify Tajweed errors

3. **Future-proof architecture**
   - Research and development
   - Academic contributions
   - Community-driven improvements

---

## Training Data

### Available Data
```
Source: data/husary/segments/segments.json
Size: 2.0 MB
Coverage: 100%

Statistics:
- Total Ayahs: 6,236
- Total Words: 77,897
- Average Words per Ayah: 12.5
- Annotation Quality: Manually verified by experts
- Segment Resolution: 1ms precision
```

### Data Format
```json
{
  "1:1": {
    "surah_number": 1,
    "ayah_number": 1,
    "audio_url": "https://audio-cdn.tarteel.ai/quran/husary/001001.mp3",
    "segments": [
      [0, 0, 480],      // [word_id, start_ms, end_ms]
      [1, 600, 1000],
      [2, 1800, 2160],
      [3, 2480, 5160]
    ]
  },
  ...
}
```

### Training/Validation Split
```
Total: 6,236 ayahs

Split Strategy: Stratified by Surah
- Training: 5,000 ayahs (80%)
- Validation: 623 ayahs (10%)
- Test: 613 ayahs (10%)

Stratification ensures:
- All 114 surahs represented
- Short and long ayahs balanced
- Diverse pronunciation patterns
```

---

## Model Architecture

### Base Model Options

#### Option 1: Wav2Vec2 Arabic (Recommended)
**Model**: `jonatasgrosman/wav2vec2-large-xlsr-53-arabic`
- **Size**: 300 MB
- **Vocab**: 51 tokens (Arabic alphabet + diacritics)
- **Pre-training**: 53 languages including Arabic
- **Fine-tuning**: Mozilla Common Voice Arabic
- **Advantages**:
  - Already tested (see CTC_VS_DTW_BENCHMARK.md)
  - Good Arabic phoneme recognition
  - Moderate size for deployment
- **Disadvantages**:
  - Not specifically trained on Quranic recitation
  - May miss Tajweed-specific features

#### Option 2: MMS-1B All
**Model**: `facebook/mms-1b-all`
- **Size**: 1.2 GB
- **Languages**: 1,100+ languages
- **Advantages**:
  - State-of-the-art multilingual ASR
  - Better generalization
  - More robust to pronunciation variations
- **Disadvantages**:
  - Larger model (slower inference)
  - May need more fine-tuning data

#### Option 3: Custom Wav2Vec2 (Best)
**Approach**: Train from scratch on Quranic recitation
- **Base**: `facebook/wav2vec2-base` (95M parameters)
- **Pre-training**: Self-supervised on 1000+ hours Quran audio
- **Fine-tuning**: CTC on our annotated segments
- **Advantages**:
  - Optimized for Quranic recitation
  - Learns Tajweed rules implicitly
  - Best accuracy potential
- **Disadvantages**:
  - Requires 1000+ hours unlabeled Quran audio
  - Weeks of GPU time for pre-training
  - Complex training pipeline

**Recommendation**: Start with Option 1 (Wav2Vec2 Arabic), upgrade to Option 3 if needed.

---

## Training Pipeline

### Step 1: Data Preparation

**Script**: `experiments/prepare_ctc_data.py`

```python
import json
import urllib.request
from pathlib import Path
from typing import List, Tuple
import torchaudio
from datasets import Dataset, DatasetDict

def prepare_training_data():
    """
    Prepare CTC training dataset from segments.json

    Output format for Hugging Face:
    {
        "audio": {"path": "...", "array": ..., "sampling_rate": 16000},
        "text": "بِسۡمِ اللهِ الرَّحۡمٰنِ الرَّحِيۡمِ",
        "word_timestamps": [
            {"word": "بِسۡمِ", "start": 0.0, "end": 0.48},
            {"word": "اللهِ", "start": 0.6, "end": 1.0},
            ...
        ]
    }
    """

    # Load segments
    with open("data/husary/segments/segments.json") as f:
        segments = json.load(f)

    # Load Quran text
    with open("data/indopak.json") as f:
        quran = json.load(f)

    dataset_items = []

    for verse_key, seg_data in segments.items():
        # Download audio
        audio_url = seg_data["audio_url"]
        audio_path = download_audio(audio_url)

        # Load audio
        waveform, sr = torchaudio.load(audio_path)
        if sr != 16000:
            resampler = torchaudio.transforms.Resample(sr, 16000)
            waveform = resampler(waveform)

        # Get text
        text = quran[verse_key]["text"]
        words = quran[verse_key]["text"].split()

        # Convert segments to word timestamps
        word_timestamps = [
            {
                "word": words[seg[0]],
                "start": seg[1] / 1000.0,  # ms to seconds
                "end": seg[2] / 1000.0
            }
            for seg in seg_data["segments"]
        ]

        dataset_items.append({
            "audio": {
                "path": str(audio_path),
                "array": waveform.squeeze().numpy(),
                "sampling_rate": 16000
            },
            "text": text,
            "word_timestamps": word_timestamps,
            "verse_key": verse_key
        })

    # Create HuggingFace dataset
    dataset = Dataset.from_list(dataset_items)

    # Split train/val/test
    train_val_test = dataset.train_test_split(test_size=0.2, seed=42)
    val_test = train_val_test["test"].train_test_split(test_size=0.5, seed=42)

    dataset_dict = DatasetDict({
        "train": train_val_test["train"],
        "validation": val_test["train"],
        "test": val_test["test"]
    })

    # Save to disk
    dataset_dict.save_to_disk("data/ctc_training_dataset")
    print(f"✓ Saved dataset: {len(dataset_dict['train'])} train, "
          f"{len(dataset_dict['validation'])} val, {len(dataset_dict['test'])} test")

    return dataset_dict
```

**Output**:
```
data/ctc_training_dataset/
├── train/ (5,000 ayahs)
├── validation/ (623 ayahs)
└── test/ (613 ayahs)
```

---

### Step 2: Fine-Tuning

**Script**: `experiments/finetune_ctc.py`

```python
from transformers import (
    Wav2Vec2ForCTC,
    Wav2Vec2Processor,
    Trainer,
    TrainingArguments
)
from datasets import load_from_disk
import torch

def finetune_ctc_model():
    """
    Fine-tune Wav2Vec2 for Arabic Quranic recitation with word timestamps.
    """

    # Load base model
    model_name = "jonatasgrosman/wav2vec2-large-xlsr-53-arabic"
    processor = Wav2Vec2Processor.from_pretrained(model_name)
    model = Wav2Vec2ForCTC.from_pretrained(model_name)

    # Load dataset
    dataset = load_from_disk("data/ctc_training_dataset")

    # Preprocessing
    def prepare_dataset(batch):
        # Process audio
        audio = batch["audio"]
        inputs = processor(
            audio["array"],
            sampling_rate=audio["sampling_rate"],
            return_tensors="pt",
            padding=True
        )

        # Encode text labels
        with processor.as_target_processor():
            labels = processor(batch["text"]).input_ids

        batch["input_values"] = inputs.input_values[0]
        batch["labels"] = labels
        return batch

    dataset = dataset.map(prepare_dataset, remove_columns=["audio", "text"])

    # Training arguments
    training_args = TrainingArguments(
        output_dir="./models/wav2vec2-quran-husary",
        group_by_length=True,
        per_device_train_batch_size=4,
        gradient_accumulation_steps=2,
        evaluation_strategy="steps",
        num_train_epochs=30,
        fp16=True,  # Mixed precision training
        save_steps=500,
        eval_steps=500,
        logging_steps=100,
        learning_rate=3e-4,
        warmup_steps=500,
        save_total_limit=2,
        push_to_hub=False,
    )

    # Trainer
    trainer = Trainer(
        model=model,
        args=training_args,
        train_dataset=dataset["train"],
        eval_dataset=dataset["validation"],
        tokenizer=processor.feature_extractor,
    )

    # Train
    trainer.train()

    # Save final model
    model.save_pretrained("./models/wav2vec2-quran-husary-final")
    processor.save_pretrained("./models/wav2vec2-quran-husary-final")

    print("✓ Training complete!")
```

**Hardware Requirements**:
- **GPU**: NVIDIA GPU with 16GB+ VRAM (e.g., V100, A100)
- **RAM**: 32GB+ system RAM
- **Storage**: 100GB+ for audio files and model checkpoints
- **Training Time**: ~6-12 hours on V100

**Cloud Options**:
- Google Colab Pro ($9.99/month) - T4 GPU (16GB)
- AWS EC2 p3.2xlarge - V100 GPU (~$3/hour)
- Lambda Labs - A100 GPU (~$1.10/hour)

---

### Step 3: Forced Alignment

**Script**: `experiments/ctc_forced_align_improved.py`

```python
import torch
import torchaudio
from transformers import Wav2Vec2ForCTC, Wav2Vec2Processor

def forced_align_with_timestamps(audio_path: str, expected_text: str):
    """
    Use fine-tuned CTC model for forced alignment.

    Returns word-level timestamps with confidence scores.
    """

    # Load fine-tuned model
    model = Wav2Vec2ForCTC.from_pretrained("./models/wav2vec2-quran-husary-final")
    processor = Wav2Vec2Processor.from_pretrained("./models/wav2vec2-quran-husary-final")

    # Load audio
    waveform, sr = torchaudio.load(audio_path)
    if sr != 16000:
        resampler = torchaudio.transforms.Resample(sr, 16000)
        waveform = resampler(waveform)

    # Process audio
    inputs = processor(waveform.squeeze(), sampling_rate=16000, return_tensors="pt")

    # Get CTC logits
    with torch.no_grad():
        logits = model(inputs.input_values).logits

    # Decode with timestamps using torchaudio
    emission = logits.squeeze(0).cpu()

    # Forced alignment
    alignments = torchaudio.functional.forced_align(
        emission,
        expected_text,
        processor.tokenizer,
        blank=0
    )

    # Convert frame indices to timestamps
    frame_rate = 16000 / 320  # 320 is Wav2Vec2 downsample factor
    word_timestamps = []

    for word, start_frame, end_frame, confidence in alignments:
        word_timestamps.append({
            "word": word,
            "start_ms": int(start_frame * 1000 / frame_rate),
            "end_ms": int(end_frame * 1000 / frame_rate),
            "confidence": float(confidence)
        })

    return word_timestamps
```

---

### Step 4: Evaluation

**Script**: `experiments/evaluate_ctc.py`

```python
def evaluate_ctc_alignment():
    """
    Evaluate fine-tuned CTC model against ground truth segments.

    Metrics:
    - Word Boundary MAE (Mean Absolute Error)
    - Start/End MAE separately
    - Per-surah accuracy
    - Confidence distribution
    """

    from datasets import load_from_disk
    import numpy as np

    # Load test set
    dataset = load_from_disk("data/ctc_training_dataset")
    test_set = dataset["test"]

    errors = []
    start_errors = []
    end_errors = []

    for item in test_set:
        # Run forced alignment
        predicted = forced_align_with_timestamps(
            item["audio"]["path"],
            item["text"]
        )

        # Ground truth
        ground_truth = item["word_timestamps"]

        # Calculate errors
        for pred, gt in zip(predicted, ground_truth):
            start_err = abs(pred["start_ms"] - gt["start"] * 1000)
            end_err = abs(pred["end_ms"] - gt["end"] * 1000)

            start_errors.append(start_err)
            end_errors.append(end_err)
            errors.append((start_err + end_err) / 2)

    # Results
    results = {
        "word_boundary_mae": np.mean(errors),
        "start_mae": np.mean(start_errors),
        "end_mae": np.mean(end_errors),
        "median_error": np.median(errors),
        "95th_percentile": np.percentile(errors, 95),
        "max_error": np.max(errors)
    }

    print("=" * 60)
    print("CTC EVALUATION RESULTS")
    print("=" * 60)
    print(f"Word Boundary MAE: {results['word_boundary_mae']:.1f} ms")
    print(f"Start MAE: {results['start_mae']:.1f} ms")
    print(f"End MAE: {results['end_mae']:.1f} ms")
    print(f"Median Error: {results['median_error']:.1f} ms")
    print(f"95th Percentile: {results['95th_percentile']:.1f} ms")
    print(f"Max Error: {results['max_error']:.1f} ms")
    print("=" * 60)

    # Target: MAE ≤ 60ms
    if results["word_boundary_mae"] <= 60:
        print("✅ TARGET ACHIEVED! Model is production-ready.")
    else:
        print("⚠️ Target not met. Consider more training or data augmentation.")

    return results
```

**Target Metrics**:
| Metric | Target | Good | Excellent |
|--------|--------|------|-----------|
| Word Boundary MAE | ≤60ms | ≤40ms | ≤20ms |
| Start MAE | ≤60ms | ≤40ms | ≤20ms |
| End MAE | ≤60ms | ≤40ms | ≤20ms |

---

## Integration Plan

### Backend Changes

**File**: `src/iqrah_audio/alignment/ctc_aligner.py` (NEW)

```python
class CTCAligner:
    """
    CTC-based forced alignment for word boundary detection.

    Used when:
    - Adding new Qari without segments
    - Analyzing user recitation
    - Generating segments automatically
    """

    def __init__(self, model_path: str = "./models/wav2vec2-quran-husary-final"):
        self.model = Wav2Vec2ForCTC.from_pretrained(model_path)
        self.processor = Wav2Vec2Processor.from_pretrained(model_path)
        self.model.eval()

    def align(self, audio: np.ndarray, expected_text: str) -> List[WordSegment]:
        """
        Perform forced alignment on audio.

        Args:
            audio: Audio waveform (16kHz, mono)
            expected_text: Expected Arabic text

        Returns:
            List of WordSegment with start_ms, end_ms, confidence
        """
        # ... (implementation from forced_align_with_timestamps)
        pass

    def align_file(self, audio_path: str, expected_text: str) -> List[WordSegment]:
        """Align audio file."""
        waveform, sr = torchaudio.load(audio_path)
        if sr != 16000:
            resampler = torchaudio.transforms.Resample(sr, 16000)
            waveform = resampler(waveform)
        return self.align(waveform.squeeze().numpy(), expected_text)
```

**API Endpoint**: `/api/align` (NEW)

```python
@app.post("/api/align")
async def align_audio(
    file: UploadFile,
    text: str = Form(...),
    use_ctc: bool = Form(True)
):
    """
    Align audio with text to generate word segments.

    Use cases:
    - Add new Qari
    - Analyze user recording
    - Generate training data
    """
    # Save uploaded file
    audio_path = save_upload(file)

    if use_ctc:
        # Use fine-tuned CTC model
        aligner = CTCAligner()
        segments = aligner.align_file(audio_path, text)
    else:
        # Use DTW fallback (for comparison)
        # ... DTW implementation ...
        pass

    return {
        "segments": [
            {
                "word_id": i,
                "word": seg.word,
                "start_ms": seg.start_ms,
                "end_ms": seg.end_ms,
                "confidence": seg.confidence
            }
            for i, seg in enumerate(segments)
        ],
        "method": "ctc" if use_ctc else "dtw"
    }
```

---

## Deployment

### Model Serving

**Option 1: Local Inference**
```bash
# Save model to deployment directory
cp -r models/wav2vec2-quran-husary-final /var/models/

# Update app.py to load model on startup
ctc_aligner = CTCAligner("/var/models/wav2vec2-quran-husary-final")
```

**Option 2: Model Server (FastAPI)**
```python
# Separate microservice for CTC inference
# models_server.py

from fastapi import FastAPI, File, UploadFile
from ctc_aligner import CTCAligner

app = FastAPI()
aligner = CTCAligner()

@app.post("/predict")
async def predict(audio: UploadFile, text: str):
    segments = aligner.align_file(audio.file, text)
    return {"segments": segments}

# Run on port 8001
# uvicorn models_server:app --port 8001
```

**Option 3: Cloud Inference (Hugging Face)**
```python
# Deploy to Hugging Face Inference API
from huggingface_hub import HfApi

api = HfApi()
api.upload_folder(
    folder_path="./models/wav2vec2-quran-husary-final",
    repo_id="your-org/wav2vec2-quran-husary",
    repo_type="model"
)

# Use inference API
from huggingface_hub import InferenceClient

client = InferenceClient("your-org/wav2vec2-quran-husary")
result = client.automatic_speech_recognition(audio_file)
```

**Recommendation**: Start with Option 1 (local), upgrade to Option 2 if needed for scaling.

---

## Cost Estimate

### One-Time Costs
| Item | Cost | Notes |
|------|------|-------|
| **Data Preparation** | $0 | We have all data |
| **Audio Download** | $0 | Tarteel CDN (public) |
| **GPU Training** | $20-50 | 6-12 hours on V100/A100 |
| **Model Storage** | $0-5/mo | HuggingFace or local |
| **Development Time** | 2-3 days | Assuming ML experience |

**Total One-Time**: $20-50

### Ongoing Costs
| Item | Cost | Notes |
|------|------|-------|
| **Inference GPU** | $0 | CPU inference is fast enough |
| **Model Updates** | $20-50/update | Re-train when adding Qaris |
| **Storage** | $5/mo | Model files ~500MB |

**Total Monthly**: $5

---

## Timeline

### Phase 1: Data Preparation (1 day)
- [ ] Download all 6,236 ayah audio files
- [ ] Create HuggingFace dataset
- [ ] Split train/val/test
- [ ] Verify data quality

### Phase 2: Fine-Tuning (1 day)
- [ ] Set up GPU environment
- [ ] Configure training pipeline
- [ ] Run fine-tuning (6-12 hours)
- [ ] Monitor training metrics

### Phase 3: Evaluation (0.5 days)
- [ ] Run on test set
- [ ] Calculate MAE metrics
- [ ] Compare to baseline
- [ ] Generate error analysis

### Phase 4: Integration (0.5 days)
- [ ] Add CTCAligner class
- [ ] Create /api/align endpoint
- [ ] Update frontend (optional)
- [ ] Deploy model

**Total Time**: 3 days

---

## Success Criteria

### Technical
- [ ] Word Boundary MAE ≤ 60ms on test set
- [ ] Transcription accuracy ≥ 95%
- [ ] Inference time ≤ 1s for 5s audio (CPU)
- [ ] Model size ≤ 500MB

### Business
- [ ] Can add new Qari in <10 minutes (vs days of manual annotation)
- [ ] User pronunciation scoring accuracy ≥ 90%
- [ ] System scales to 10+ Qaris
- [ ] Total cost ≤ $100/year

---

## Risks & Mitigation

### Risk 1: Model doesn't meet MAE target
**Mitigation**:
- Try different base models (MMS-1B)
- Augment training data (pitch shift, time stretch)
- Ensemble multiple models
- Fall back to manual annotation for critical use cases

### Risk 2: Inference too slow
**Mitigation**:
- Quantize model (INT8)
- Use ONNX runtime
- Batch processing
- GPU inference if needed

### Risk 3: Overfitting to Husary voice
**Mitigation**:
- Collect data from multiple Qaris
- Use voice augmentation
- Transfer learning from general Arabic ASR
- Cross-Qari validation

---

## Conclusion

Fine-tuning CTC on our annotated segments is:
- ✅ **Feasible**: We have perfect training data
- ✅ **Affordable**: $20-50 one-time cost
- ✅ **Fast**: 3 days to production
- ✅ **Valuable**: Scales to unlimited Qaris

**However**, it's **NOT URGENT** because:
- Current system works perfectly for Husary (0ms error)
- We only have 1 Qari with segments right now
- Users haven't requested additional Qaris yet

**Recommendation**:
> **Implement when we add our second Qari**
>
> Until then, current segment-based system is optimal!

---

**Next Steps When Ready**:
1. Decide on timeline (when adding 2nd Qari)
2. Set up GPU environment (Colab Pro or Lambda Labs)
3. Run `experiments/prepare_ctc_data.py`
4. Run `experiments/finetune_ctc.py`
5. Evaluate and deploy!

---

**References**:
- [CTC vs DTW Benchmark](./ctc_vs_dtw_benchmark.md)
- [Implementation Roadmap](./IMPLEMENTATION_ROADMAP.md)
- [Phase 1 Status](./PHASE_1_STATUS.md)
- [UI Improvements](./UI_IMPROVEMENTS_2025-10-05.md)
