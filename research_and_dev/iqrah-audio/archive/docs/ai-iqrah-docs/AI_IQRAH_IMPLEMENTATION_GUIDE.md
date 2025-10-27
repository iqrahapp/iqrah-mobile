# IQRAH AUDIO - IMPLEMENTATION REFERENCE

---

## ARCHITECTURAL DECISIONS

### KEEP (already working)

**Pitch Extraction**:
- SwiftF0 primary (42× CREPE speed)
- CREPE fallback for melodic passages
- Action: None required

**Phoneme Alignment**:
- Current: Wav2Vec2 CTC (MMS)
- Upgrade: Fine-tune Wav2Vec2-BERT → <1% PER
- Windowed alignment within words: Keep

**Comparison Framework**:
- Soft-DTW rhythm: Keep
- ΔF0 melody: Keep
- GOP pronunciation: Keep

### ENHANCE (add features)

**Tajweed Validation (Priority 1)**:
- Madd: Upgrade to 99% rule-based
- Ghunnah: Add formant + MLP (NEW)
- Qalqalah: Add burst + SVM (NEW)
- Rollout: madd → ghunnah → qalqalah

**Voice Quality (Priority 2)**:
- Add OpenSMILE eGeMAPS (88-d)
- Add vibrato detection (rate, extent, regularity)
- Add breathiness (H1-H2, HNR, CPP)
- Add X-vector embeddings (512-d)

**Advanced Prosody (Priority 3)**:
- Add nPVI, Varco (rhythm beyond DTW)
- Add Fujisaki decomposition
- Add Maqam CNN (Maqam478 dataset)
- Add declination modeling

**Feedback Generation (Priority 4)**:
- Add pedagogical text
- Add progress tracking (PostgreSQL)
- Add actionable recommendations

### DEFER (Phase 2+)

- Real-time streaming (wait for Phase 1 offline 90% accurate)
- Mobile deployment (wait for Phase 2 real-time)
- Complex Tajweed: Idghaam, Ikhfaa, Iqlaab (after basic rules)
- Multi-Qira'at (focus Hafs first)

---

## PHASE 1 CRITICAL PATH (Weeks 1-24)

```
Week 1-4:   Fine-tune Wav2Vec2-BERT
Week 5:     Implement Madd validator
Week 6-8:   Train Ghunnah classifier
Week 9-12:  Integrate OpenSMILE + prosody
Week 13-14: Build comparison engine
Week 15-16: Feedback generation
Week 17-24: Validation study
```

### Parallel Workstreams

| Workstream         | Start  | Duration |
| ------------------ | ------ | -------- |
| Voice quality (M5) | Week 1 | 2 weeks  |
| Qalqalah (M4.3)    | Week 6 | 3 weeks  |
| Feedback templates | Week 3 | Ongoing  |

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

## PHASE COMPLETION CHECKLISTS

### Phase 1 Complete When:

**Technical Metrics:**
- [ ] PER < 1%
- [ ] Boundary precision: 90% within 50ms
- [ ] Madd: 99%+
- [ ] Ghunnah: 85%+
- [ ] Qalqalah: 80%+
- [ ] Correlation: r > 0.7 vs human

**User Metrics:**
- [ ] 100 expert-rated test cases complete
- [ ] 20-30 alpha users tested
- [ ] 4+/5 average feedback
- [ ] <5% technical issues

**System Metrics:**
- [ ] Latency p95 < 5s
- [ ] Memory < 4GB
- [ ] No crashes on 100 diverse files

**Documentation:**
- [ ] All modules have docstrings
- [ ] README with installation
- [ ] API documentation complete
- [ ] Architecture decisions recorded

### Phase 2 Ready When:

- [ ] All Phase 1 checkboxes ✅
- [ ] GPU infrastructure provisioned
- [ ] INT8 quantization validated
- [ ] Streaming protocol designed
- [ ] <500ms latency achievable on benchmark

---

## RISK MANAGEMENT

### High-Risk Items

| Risk                    | Mitigation         | Check                       |
| ----------------------- | ------------------ | --------------------------- |
| Wav2Vec2 training fails | MMS fallback       | Week 2: training curves     |
| Ghunnah <70%            | More training data | Week 8: test set validation |
| User confusion          | Simplify feedback  | Week 20: user interviews    |
| GPU cost exceeds budget | Aggressive caching | Monthly: cloud bills        |

### Medium-Risk Items

| Risk                       | Contingency                              |
| -------------------------- | ---------------------------------------- |
| Qalqalah <80%              | Defer to Phase 3, ship without           |
| Real-time >500ms           | Launch async mode only                   |
| Mobile model >200MB        | Hybrid: basic on-device, advanced server |
| Expert annotations delayed | Use rule-based initially                 |

---

## DECISION RATIONALE

### Q: Why Wav2Vec2-BERT over MMS?
**Decision**: Fine-tune Wav2Vec2-BERT
**Rationale**:
- SOTA: 0.16% PER achievable (MMS: 2-5%)
- Task-adaptive pretraining critical for Quranic domain
- MMS is generalist (1000+ languages), need specialist
- Cost: ~€500-1000 training (worth accuracy gain)

**Trade-off**: 1-2 weeks training vs immediate MMS use
**Verdict**: Worth it for production quality

### Q: Why Progressive Tajweed Rollout?
**Decision**: madd → ghunnah → qalqalah → complex
**Rationale**:
- Madd easiest (rule-based, 99%)
- Ghunnah has established ML methods
- Qalqalah exploratory (no strong SOTA)
- Complex rules need more research

**Trade-off**: Delayed full feature set vs higher confidence
**Verdict**: Ship incrementally with high quality

### Q: Why Offline-First?
**Decision**: Perfect offline before streaming
**Rationale**:
- Real-time adds 10× complexity (WebSocket, VAD, chunking)
- Offline allows thorough validation
- Users accept 5-10s for uploaded audio
- Easier debug and iteration

**Trade-off**: Delayed mobile by 6 months
**Verdict**: Foundation first, speed second

### Q: Why OpenSMILE over Custom?
**Decision**: Use OpenSMILE eGeMAPS
**Rationale**:
- 88 standardized features (research-validated)
- Maintained by industry experts
- Cross-study comparability
- Saves 2-3 weeks implementation

**Trade-off**: Black-box vs full control
**Verdict**: Standardization > customization

---

## VALIDATION METHODOLOGY

### Accuracy Validation

**Phoneme Alignment Test Set**:
- 100 ayahs with manual boundaries
- Metrics: PER, boundary precision (20ms/50ms thresholds)
- Target: PER <1%, 90% within 50ms

**Tajweed Rules Test Sets**:
- Madd: 500 examples (all types) → 99%
- Ghunnah: 300 examples → 85%
- Qalqalah: 200 examples → 80%

**Prosody Validation**:
- 100 expert-rated pairs (10-point scale per dimension)
- Correlation: Automated vs human
- Target: r > 0.7 for rhythm, melody, style

**Expert Annotation**:
- Hire 3-5 qualified Qaris
- Inter-rater: Krippendorff's α > 0.75
- Budget: €2,000-3,000 for 200 hours

### Performance Benchmarks

**Latency Targets (Offline, CPU)**:
- Preprocessing: p95 <500ms
- Pitch: p95 <300ms
- Alignment: p95 <2s
- Tajweed: p95 <100ms
- Prosody: p95 <500ms
- Comparison: p95 <100ms
- **Total: p95 <4s per ayah**

**Latency Targets (Real-time, GPU)**:
- **Total: p95 <500ms per chunk**

### User Testing Protocol

**Alpha (N=10)**: Internal users, qualitative feedback
**Beta (N=50-100)**: Public, quantitative metrics, A/B testing
**Validation Study (N=60-100)**: Pre/post measurements, academic publication

---

## WEEK 1 TASKS

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


### AI Agent 4: T3.5.1 - Content Verification Module

**Task ID**: T3.5.1
**Module**: M3.5 Content Accuracy Verification
**Priority**: Critical (blocking for M7 integration)

**Deliverable**: `content_verifier.py` with ASR integration and WER gatekeeper logic

**Requirements**:
1. Integrate `obadx/recitation-segmenter-v2` ASR model
2. Implement Arabic text normalization function
3. Implement Levenshtein edit distance calculation
4. Calculate WER and extract error details
5. Provide fallback to Whisper-large-v3 if primary model fails

**Input**:
```python
audio: np.ndarray  # 16kHz mono audio
reference_text: str  # Ground-truth Quranic text with diacritics
```

**Output**:
```python
{
    "wer": float,
    "transcript": str,
    "normalized_transcript": str,
    "errors": List[dict]
}
```

**Test Cases**:
1. **Exact Match**: Correct recitation → WER <1%
2. **Wrong Verse**: Different verse → WER >50%
3. **Single Word Error**: One substitution → WER = 1/N (where N = number of words)
4. **Normalization**: Text with/without diacritics should match after normalization
5. **Alef Variants**: أ، إ، آ should normalize to ا

**Acceptance Criteria**:
- [ ] All 5 test cases pass
- [ ] ASR model loads successfully on first call
- [ ] Normalization handles all Arabic diacritics
- [ ] Levenshtein operations return correct error types (insert/delete/substitute)
- [ ] Function executes in <2s for 10s audio
- [ ] Docstring with usage example included

**Estimated Time**: 6-8 hours

**Dependencies**:
- `transformers` (HuggingFace)
- `torch`
- `python-Levenshtein`
- `pyarabic` (for normalization)

**Reference Code**:
```python
# Example: Loading obadx/recitation-segmenter-v2
from transformers import Wav2Vec2ForCTC, Wav2Vec2Processor

model = Wav2Vec2ForCTC.from_pretrained("obadx/recitation-segmenter-v2")
processor = Wav2Vec2Processor.from_pretrained("obadx/recitation-segmenter-v2")

# Example: Arabic normalization
from pyarabic.araby import strip_diacritics

def normalize_arabic(text: str) -> str:
    text = strip_diacritics(text)
    text = text.replace('أ', 'ا').replace('إ', 'ا').replace('آ', 'ا')
    text = text.replace('ـ', '')  # Remove tatweel
    return text
```

**Deadline**: Day 5 (end of Week 1)


---

## SUCCESS DEFINITIONS

### Phase 1 Success

**Quantitative**:
- 100 expert cases: r > 0.7
- PER < 1%
- Madd: 99%, Ghunnah: 85%, Qalqalah: 80%
- Latency p95 < 5s

**Qualitative**:
- Users: "This actually helps me improve"
- Teachers: Willing to recommend
- Expert Qaris: Validate accuracy

**Business**:
- 50+ beta users weekly active
- 70%+ retention after 1 month
- Ready for B2B pilot

### Phase 2 Success

- Real-time: <500ms p95
- 10+ concurrent users
- 70%+ cache hit rate
- $0.10/user/month cost

### Phase 3 Success

- iOS + Android in stores
- <300ms on-device
- 4.5+ stars
- 1000+ weekly active users

---

## FAQ

**Q: Can I change the architecture?**
A: This is 3-year commitment. Small tweaks OK, major changes wait for v2.0 (2028).

**Q: If Wav2Vec2 training fails?**
A: Fallback to MMS (2-5% PER instead of <1%). Still acceptable.

**Q: Should I parallelize Phase 1 and Phase 2?**
A: No. Validate Phase 1 accuracy first. Real-time optimization pointless if core quality bad.

**Q: If I can't collect 1000 Ghunnah examples?**
A: Start with 300-500. Accuracy ~75-80% instead of 85%. Still useful, iterate later.

**Q: Can AI agents handle everything?**
A: No. 60-70% of tasks. You're needed for: architecture, research, training, validation, user studies.

**Q: Task takes longer than estimated?**
A: Estimates ideal conditions. 2× multiplier typical. Adjust timeline, not quality bar.

**Q: When move to Phase 2?**
A: When all Phase 1 checkboxes ✅. Don't rush.

---

## RESOURCE ALLOCATION

### Solo Developer Timeline

| Phase       | Focus                        | Outcome               |
| ----------- | ---------------------------- | --------------------- |
| Weeks 1-4   | Core alignment (M3)          | Phoneme recognition   |
| Weeks 5-8   | Tajweed madd+ghunnah (M4)    | Basic rules validated |
| Weeks 9-12  | Prosody+comparison (M6-M7)   | Full pipeline E2E     |
| Weeks 13-16 | Feedback+validation (M8, V1) | User-ready prototype  |
| Weeks 17-20 | Qalqalah+refinement          | Advanced rules        |
| Weeks 21-24 | User testing+iteration       | Production release    |

### With AI Agents (Parallel)

| Week  | Main Dev           | Agent 1          | Agent 2         | Agent 3          |
| ----- | ------------------ | ---------------- | --------------- | ---------------- |
| 1-2   | M3.1 training      | M1 preprocessing | M2 pitch        | M5 OpenSMILE     |
| 3-4   | M3.2 alignment     | M4.1 madd        | M8.1 templates  | V1.1.1 data prep |
| 5-6   | M4.2 ghunnah train | M6.1 rhythm      | M7.1 comparison | T5.5 embeddings  |
| 7-8   | M4.3 qalqalah      | M6.2 melody      | M7.5 fusion     | T8.3 progress DB |
| 9-10  | Integration test   | Documentation    | Visualization   | Benchmarking     |
| 11-12 | User testing       | Bug fixes        | Refinement      | Deployment       |

**Speedup**: 2-3× faster with coordinated AI agents
