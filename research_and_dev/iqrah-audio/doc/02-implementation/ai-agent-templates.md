# AI Agent Task Templates

[↑ Navigation](../NAVIGATION.md)

---

## AI AGENT TASK CLASSIFICATION

### HIGH SUCCESS RATE

**Pure Implementation**:
- Task format: "Implement X supporting Y"
- Example: "Implement audio loader supporting MP3/WAV/WebM"
- Characteristics: Clear I/O, standard libraries
- Template: See Task Decomposition Template 1

**Feature Extraction**:
- Task format: "Extract X features"
- Example: "Extract OpenSMILE eGeMAPS features"
- Code: 20-30 lines, single function

**Data Processing**:
- Task format: "Prepare X from Y"
- Example: "Prepare Ghunnah training CSV from annotations"
- Output: Validated CSV/JSON

**Template-Based**:
- Task format: "Generate X for Y"
- Example: "Generate feedback text for madd violations"
- Pattern: Fill-in-the-blank templates

### MEDIUM SUCCESS (requires guidance)

**Training Scripts**:
- Provide: Exact config, HuggingFace example
- Monitor: Training curves, validation metrics
- Example: "Train Wav2Vec2-BERT on Quranic data"

**Optimization**:
- Provide: Pseudocode, NumPy reference
- Test: Unit tests with known I/O
- Example: "Implement Soft-DTW with Sakoe-Chiba band"

**Integration**:
- Provide: Interface contracts, test cases
- Review: Edge case handling
- Example: "Integrate 3 validators into comparison engine"

### HUMAN REQUIRED (low AI success)

**Research**: "Design qalqalah burst detection algorithm"
**Expert Annotation**: "Create 100-ayah test set with phoneme boundaries"
**Architecture Decisions**: "Redesign comparison fusion weights"
**User Studies**: "Validate correlation with human ratings"

**Strategy**: Human designs → AI implements

---

## AI AGENT TASK TEMPLATE

```markdown
## Task: [Specific title]

**Context:**
- Module: [Which of 8 modules]
- Purpose: [Problem solved]
- Location: [Exact file path]

**Requirements:**
1. [Requirement 1]
2. [Requirement 2]
...

**Input:** [Exact type and format]
**Output:** [Exact type and format]

**Dependencies:**
- [Required libraries]
- [Prerequisites]

**Test Cases:**
1. [Input → expected output]
2. [Edge case → expected behavior]
...

**Acceptance Criteria:**
- [ ] All test cases pass
- [ ] Type hints complete
- [ ] Docstring with examples
- [ ] No runtime errors

**Estimated Time:** [2-8 hours typical]

**Reference Code:** [Link if exists]
```

---

## MODULE INTERFACE CONTRACTS

### M1: Preprocessing
```python
def preprocess_audio(file_path: str) -> dict:
    """
    Returns: {
        "audio_path": str,
        "sample_rate": 16000,
        "duration": float,
        "segments": [{"start": float, "end": float}],
        "quality_metrics": {"snr_db": float, "clipping_ratio": float}
    }
    """
```

### M2: Pitch Extraction
```python
def extract_pitch(audio: np.ndarray, sr: int) -> dict:
    """
    Returns: {
        "pitch_hz": np.ndarray,
        "times": np.ndarray,
        "confidence": np.ndarray,
        "voicing": np.ndarray,
        "method": str,
        "stats": {"mean_hz": float, "std_hz": float, "range_hz": tuple}
    }
    """
```

### M3: Phoneme Alignment
```python
def align_phonemes(audio: np.ndarray, sr: int, surah: int, ayah: int) -> dict:
    """
    Returns: {
        "phonemes": [
            {
                "phoneme": str,
                "start": float,
                "end": float,
                "confidence": float,
                "tajweed_rule": str,
                "gop_score": float
            }
        ],
        "alignment_method": str,
        "quality_score": float
    }
    """
```

### M4: Tajweed Validation
```python
class TajweedValidator:
    def validate(self, phonemes: list, audio: np.ndarray) -> dict:
        """
        Returns: {
            "rule_name": str,
            "violations": [
                {
                    "phoneme_idx": int,
                    "severity": str,
                    "expected": Any,
                    "actual": Any,
                    "confidence": float,
                    "feedback": str
                }
            ],
            "overall_score": float
        }
        """
```

---

## TASK TEMPLATE EXAMPLES

### Example 1: Pure Implementation Task

```markdown
## Task: Implement Audio File Loader with Multi-Format Support

**Context:**
- Module: M1 - Preprocessing
- Purpose: Load and normalize audio files from various formats
- Location: `src/iqrah_audio/preprocessing/audio_loader.py`

**Requirements:**
1. Support MP3, WAV, WebM, M4A formats
2. Convert all inputs to 16kHz mono
3. Return NumPy array with metadata
4. Handle file validation and error cases

**Input:** File path (str)
**Output:** dict with {audio: np.ndarray, sr: int, duration: float, format: str}

**Dependencies:**
- librosa >= 0.10.0
- soundfile >= 0.12.0
- pydub >= 0.25.0

**Test Cases:**
1. test_load_wav.wav → shape (160000,), sr=16000, duration=10.0
2. test_load_mp3.mp3 → correctly converted to mono
3. corrupted_file.wav → raise AudioLoadError with clear message
4. silent_audio.wav → returns zeros, no crash

**Acceptance Criteria:**
- [ ] All test cases pass
- [ ] Type hints complete
- [ ] Docstring with examples
- [ ] No runtime errors
- [ ] Handles edge cases (empty file, corrupted file)

**Estimated Time:** 3-4 hours

**Reference Code:** librosa.load() documentation
```

---

### Example 2: Feature Extraction Task

```markdown
## Task: Extract OpenSMILE eGeMAPS Features

**Context:**
- Module: M5 - Voice Quality
- Purpose: Extract standardized acoustic features for voice quality analysis
- Location: `src/iqrah_audio/voice_quality/opensmile_features.py`

**Requirements:**
1. Use OpenSMILE eGeMAPS v02 feature set (88 features)
2. Extract features from audio segment
3. Return structured feature dict with feature names
4. Handle audio normalization internally

**Input:** audio (np.ndarray), sr (int)
**Output:** dict with feature_name → float value mapping

**Dependencies:**
- opensmile >= 2.5.0
- numpy >= 1.24.0

**Test Cases:**
1. Normal speech → 88 features extracted
2. Silent audio → returns valid features (zeros/low energy)
3. Very short audio (<0.5s) → handles gracefully or raises clear error

**Acceptance Criteria:**
- [ ] All test cases pass
- [ ] Type hints complete
- [ ] Docstring with examples
- [ ] Returns exactly 88 named features
- [ ] Features normalized to reasonable ranges

**Estimated Time:** 2-3 hours

**Reference Code:** opensmile.Smile.process_signal() examples
```

---

### Example 3: Data Processing Task

```markdown
## Task: Prepare Ghunnah Training Dataset from Annotations

**Context:**
- Module: M4 - Tajweed Validation
- Purpose: Convert expert annotations to ML-ready training CSV
- Location: `src/iqrah_audio/tajweed/data_prep/ghunnah_dataset.py`

**Requirements:**
1. Read annotation JSON files (format: {ayah_id, phoneme_idx, has_ghunnah, duration_ms})
2. Extract audio segments for each annotation
3. Compute acoustic features (formants F1-F3, duration, energy)
4. Output CSV with columns: [audio_path, f1, f2, f3, duration, energy, label]
5. Split into train/val/test (70/15/15)

**Input:** annotations_dir (str), audio_dir (str)
**Output:** CSV files saved to output_dir

**Dependencies:**
- pandas >= 2.0.0
- librosa >= 0.10.0
- json (stdlib)

**Test Cases:**
1. 100 annotations → 70/15/15 split, all features present
2. Missing audio file → skip with warning, continue processing
3. Invalid annotation → log error, continue
4. Verify feature ranges: F1 [200-1000Hz], F2 [800-3000Hz], F3 [2000-4000Hz]

**Acceptance Criteria:**
- [ ] All test cases pass
- [ ] Type hints complete
- [ ] Docstring with examples
- [ ] CSV validates (no missing values, correct dtypes)
- [ ] Logging for skipped/invalid entries

**Estimated Time:** 4-5 hours

**Reference Code:** pandas DataFrame.to_csv(), sklearn.model_selection.train_test_split()
```

---

### Example 4: Integration Task

```markdown
## Task: Integrate Tajweed Validators into Comparison Engine

**Context:**
- Module: M7 - Comparison Engine
- Purpose: Add tajweed rule validation to comparison pipeline
- Location: `src/iqrah_audio/comparison/engine.py`

**Requirements:**
1. Import MaddValidator, GhunnahValidator, QalqalahValidator
2. Add tajweed validation step after phoneme alignment
3. Aggregate tajweed scores into overall comparison score
4. Handle validator failures gracefully (log, continue with partial results)
5. Add tajweed_results to output dict

**Input:** user_audio, reference_audio, phonemes
**Output:** Enhanced comparison dict with tajweed_results field

**Dependencies:**
- src.iqrah_audio.tajweed.validators (internal)
- numpy >= 1.24.0

**Test Cases:**
1. All validators pass → tajweed_results has 3 entries
2. Ghunnah validator fails → continues, logs error, returns partial results
3. No tajweed rules in segment → returns empty tajweed_results
4. Verify score aggregation: overall_score includes weighted tajweed component

**Acceptance Criteria:**
- [ ] All test cases pass
- [ ] Type hints complete
- [ ] Docstring updated
- [ ] Error handling for validator failures
- [ ] Integration tests pass

**Estimated Time:** 5-6 hours

**Reference Code:** Existing comparison engine structure in engine.py
```

---

## WEEK 1 EXAMPLE TASK ASSIGNMENT

### Main Developer
1. Setup AWS/Lambda Labs account
2. Download Tarteel dataset (~100GB)
3. Setup HuggingFace Hub
4. Setup PostgreSQL
5. Begin Wav2Vec2 training (T3.1.1 → T3.1.2 → T3.1.3)
6. Monitor training curves
7. Assign/review AI agent tasks

### AI Agent 1: T1.1.1 - Audio Preprocessing
- Deliverable: `audio_loader.py` with tests
- Format support: MP3/WAV/WebM/M4A
- Deadline: Day 3

### AI Agent 2: T2.1.1 - Pitch Extraction
- Deliverable: `pitch_extractor_swiftf0.py` enhanced
- SwiftF0 integration improvements
- Deadline: Day 2

### AI Agent 3: T5.1.1 - Voice Quality
- Deliverable: `opensmile_features.py` with eGeMAPS
- OpenSMILE wrapper
- Deadline: Day 4

### End of Week 1
- ✅ Training started
- ✅ 3 modules enhanced
- ✅ Integration testing plan ready
- ✅ Week 2 tasks assigned

---

**Related**: [Architecture Overview](../01-architecture/overview.md) | [Implementation Guide](./guide.md) | [Task Breakdown](../03-tasks/task-breakdown.md)
