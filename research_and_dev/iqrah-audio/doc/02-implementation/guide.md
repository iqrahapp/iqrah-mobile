# IQRAH Audio - Implementation Guide

[↑ Navigation](../NAVIGATION.md)

---

## ARCHITECTURAL DECISIONS

### KEEP (already working)

**Pitch Extraction**:
- SwiftF0 primary (42× CREPE speed)
- CREPE fallback for melodic passages
- Action: None required

**Phoneme Alignment**:
- Current: Use pre-trained Muaalem v3.2 (phonemes + sifat output)
- No training required for MVP
- Phonetic-first approach (not grapheme-based)
- Windowed alignment within words: Keep

**Comparison Framework**:
- Soft-DTW rhythm: Keep
- ΔF0 melody: Keep
- GOP pronunciation: Keep

### ENHANCE (add features)

**Tajweed Validation (Priority 1)**:
- **Two-Tier Architecture (NEW)**:
  - Tier 1: Baseline sifat from Muaalem (10+ rules, 70-85%, free)
  - Tier 2: Specialized modules (pluggable, enhanced accuracy)
- Madd: Probabilistic duration modeling (95%+ target)
- Ghunnah: Baseline + formant analysis (90%+ target)
- Qalqalah: Baseline + burst detection (85%+ target)
- Rollout: Tier 1 baseline → Madd Tier 2 → Ghunnah Tier 2 → Qalqalah Tier 2

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

## PHASE 1 CRITICAL PATH (Weeks 1-18)

```
Week 1:     Setup: Download Muaalem, extract phonetizer
Week 2-4:   M3: Muaalem integration + phoneme alignment
Week 5:     M4.1: Baseline sifat interpreter (Tier 1)
Week 5-6:   M4.2-4.4: Madd probabilistic validator (Tier 2)
Week 6:     M4.5: Tajweed orchestrator
Week 7:     M4.6-4.8: Ghunnah formants (optional) + E2E testing
Week 8-9:   M5: Voice quality (OpenSMILE)
Week 9-10:  M6: Prosody analysis
Week 11-12: M7: Comparison engine
Week 13-14: M8: Feedback generation
Week 15-16: Integration + bug fixes
Week 17-18: Validation study
```

### Parallel Workstreams

| Workstream | Start | Duration |
|------------|-------|----------|
| Voice quality (M5) | Week 8 | 2 weeks |
| Prosody (M6) | Week 9 | 2 weeks |
| Ghunnah Tier 2 (M4.6) | Week 7 | 1 week (optional) |
| Feedback templates | Week 3 | Ongoing |

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

**Model Integration**:
- Provide: Model card, HuggingFace example
- Test: Inference on sample audio
- Example: "Integrate Muaalem model for phoneme + sifat extraction"

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

## PHASE COMPLETION CHECKLISTS

### Phase 1 Complete When:

**Technical Metrics:**
- [ ] PER < 2% (using pre-trained Muaalem)
- [ ] Boundary precision: 90% within 50ms
- [ ] Madd: 95%+ (Tier 2 probabilistic)
- [ ] Ghunnah: 70-85% (Tier 1 baseline) or 90%+ (Tier 2 formants)
- [ ] Qalqalah: 75-80% (Tier 1 baseline) or 85%+ (Tier 2 burst)
- [ ] Comprehensive Tajweed: 10+ rules covered
- [ ] Correlation: r > 0.75 vs human

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

## **MVP TESTING AND VALIDATION STRATEGY**

**Status**: This section defines the testing approach for the grapheme-based MVP.

### Unit Tests (Required for Each Module)

**1. Arabic Text Normalization** (`tests/test_normalization.py`):
- Test diacritic removal: Input with all combining marks → stripped output
- Test alif normalization: أ/إ/آ/ٱ → all become ا
- Test hamza handling: ؤ → و, ئ → ي, ء → (removed)
- Test tatweel removal: Text with kashida → cleaned
- Test punctuation stripping: Text with ،؛؟ → removed
- Test whitespace collapse: Multiple spaces/newlines → single space
- **Acceptance**: 100% pass on edge cases

**2. Hybrid WER/CER Gatekeeper** (`tests/test_gate.py`):
- Test metric selection: ≤3 words → CER, >3 words → WER
- Test high confidence: Error rate 0.02 → confidence="high", should_proceed=True
- Test medium confidence: Error rate 0.07 → confidence="medium", should_proceed=True
- Test fail: Error rate 0.12 → confidence="fail", should_proceed=False
- Test normalization application: Ensure both reference and hypothesis normalized
- Test short text stability: 2-word reference with 1 error → CER not WER
- **Acceptance**: All threshold boundaries correctly enforced

**3. CTC Forced Alignment** (`tests/test_ctc_align.py`):
- Test basic alignment: Simple 5-token reference → 5 aligned tokens with times
- Test empty reference: Empty input → empty output (no crash)
- Test duration sanity: All token durations between 20ms and 500ms
- Test confidence calculation: Mean CTC posterior correctly computed
- Test alignment quality: Mean confidence ≥ 0.5 on clean audio
- **Acceptance**: No crashes, sanity checks pass

**4. LLR Confidence Scoring** (`tests/test_llr.py`):
- Test LLR monotonicity: Higher CTC posterior → higher LLR
- Test discriminative power: Target token much more likely → LLR > 1.0
- Test low confidence detection: Ambiguous token → LLR < 0.5
- Test numerical stability: No NaN/Inf on edge cases
- **Acceptance**: LLR behaves as discriminative confidence measure

**5. Tajweed MVP Rules** (`tests/test_tajweed_mvp.py`):
- Test Madd detection: Long vowel ا/و/ي → flagged for duration check
- Test Madd validation: 150ms duration, 200ms threshold → violation
- Test Shadda detection: Doubled consonant → flagged for duration check
- Test Shadda validation: Duration 1.4× median → violation
- Test Waqf detection: Final token + energy drop → pass
- Test Waqf validation: No energy drop → violation
- **Acceptance**: Rules trigger correctly, no false positives on clean data

### Integration Tests

**1. End-to-End Pipeline** (`tests/test_pipeline_integration.py`):
- Test full flow: Audio → ASR → Gate (pass) → Align → LLR → Tajweed → Output
- Test gate blocking: Wrong verse → WER >8% → pipeline stops, returns error
- Test output schema: All required fields present, correct types
- Test latency: 15s audio processed in <1.5s on GPU
- **Acceptance**: Pipeline completes successfully, output valid JSON

**2. Gatekeeper Pathing** (`tests/test_gate_integration.py`):
- Test high confidence path: WER 0.03 → full analysis runs
- Test medium confidence path: WER 0.06 → warning flag added
- Test fail path: WER 0.15 → analysis stops, returns content mismatch error
- **Acceptance**: Correct path taken based on WER/CER threshold

**3. Multi-Segment Audio** (`tests/test_chunking.py`):
- Test chunking trigger: 30s audio → 2 chunks created
- Test chunk overlap: Verify 0.4s stride applied
- Test alignment stitching: Chunks merged correctly without gaps
- **Acceptance**: Long audio handled without memory errors

### Performance Tests

**1. Latency Benchmarks** (`tests/test_latency.py`):
- 5s audio → <600ms total
- 15s audio → <1300ms total
- 30s audio (chunked) → <2000ms total
- **Hardware**: RTX 3060-Ti, FP16
- **Acceptance**: 95th percentile meets targets

**2. Memory Profiling** (`tests/test_memory.py`):
- Peak VRAM usage: <6GB on RTX 3060-Ti
- Peak RAM usage: <4GB
- **Monitoring**: `torch.cuda.max_memory_allocated()`
- **Acceptance**: No OOM errors on target hardware

**3. Accuracy Benchmarks** (`tests/test_accuracy.py`):
- Content verification: WER <5% on clean recitations
- Alignment boundary: 80% within 50ms (grapheme-level, not phoneme)
- Tajweed Madd: >90% precision (no false positives)
- **Dataset**: 50 hand-labeled test cases
- **Acceptance**: Meets precision targets

### Deferred Tests (Post-MVP)

These tests require phoneme-level models and are **explicitly deferred**:
- Phoneme recognition accuracy (PER)
- Ghunnah formant analysis
- Qalqalah burst detection
- Ikhfa'/Idgham complex rule validation

**Rationale**: The MVP uses grapheme-level alignment. Testing phoneme-level accuracy would require a different ASR model and labeled phoneme data, which is out of scope for the zero-training MVP.

### Test Execution

```bash
# Run all MVP tests
pytest tests/ -v --tb=short

# Run with coverage
pytest tests/ --cov=src/iqrah --cov-report=html

# Run latency benchmarks (requires GPU)
pytest tests/test_latency.py --benchmark

# Run integration tests only
pytest tests/test_*_integration.py -v
```

### Continuous Integration

**Pre-commit**:
- Unit tests must pass
- Type checking (mypy)
- Linting (ruff)

**Pull Request**:
- All tests pass
- Coverage >80%
- Integration tests pass on GPU CI runner

**Release**:
- Full test suite pass
- Latency benchmarks meet targets
- Memory profiling clean

---

> **MVP Reason Note**: The MVP testing strategy focuses on verifying the **grapheme-level pipeline** without requiring phoneme-level ground truth or model training. Normalization and gatekeeper tests ensure robust content verification. Alignment and LLR tests validate the CTC pipeline. Tajweed tests verify duration/energy rules only. Phoneme-level tests are deferred to avoid dependency on training data or phonetic expertise. This approach enables rapid iteration and validation without blocking on data collection.

---

## RISK MANAGEMENT

### High-Risk Items

| Risk | Mitigation | Check |
|------|------------|-------|
| Muaalem PER >5% on learners | Fine-tune in Phase 2 | Week 4: test set validation |
| Ghunnah Tier 1 <70% | Enable Tier 2 formants | Week 7: test set validation |
| User confusion | Simplify feedback | Week 17: user interviews |
| GPU cost exceeds budget | Use CPU for inference | Monthly: cloud bills |

### Medium-Risk Items

| Risk | Contingency |
|------|-------------|
| Qalqalah Tier 1 <75% | Ship Tier 1 only, defer Tier 2 to Phase 2 |
| Madd distribution estimation fails | Fallback to fixed duration rules |
| Real-time >500ms | Launch async mode only |
| Mobile model >200MB | Hybrid: basic on-device, advanced server |
| Expert annotations delayed | Use Tier 1 baseline only initially |

---

## RESOURCE ALLOCATION

### Solo Developer Timeline

| Phase | Focus | Outcome |
|-------|-------|---------|
| Weeks 1-4 | Muaalem integration (M3) | Phonemes + sifat working |
| Weeks 5-7 | Two-tier Tajweed (M4) | Baseline + Madd specialized |
| Weeks 8-10 | Voice quality + Prosody (M5-M6) | Multi-dimensional analysis |
| Weeks 11-12 | Comparison engine (M7) | Score fusion working |
| Weeks 13-14 | Feedback generation (M8) | User-ready output |
| Weeks 15-16 | Integration + bug fixes | E2E pipeline stable |
| Weeks 17-18 | User testing + validation | Production release |

### With AI Agents (Parallel)

| Week | Main Dev | Agent 1 | Agent 2 | Agent 3 |
|------|----------|---------|---------|---------|
| 1-2 | M3.1 phonetizer + Muaalem | M1 preprocessing | M2 pitch | M5 OpenSMILE |
| 3-4 | M3.3 phoneme aligner | M3.5 gate | M8.1 templates | M4.1 baseline |
| 5-6 | M4.2-4.4 Madd validator | M6.1 rhythm | M7.1 comparison | T4.6 Ghunnah |
| 7-8 | M4.5 orchestrator | M6.2 melody | M7.5 fusion | T8.3 progress DB |
| 9-10 | Integration test | Documentation | Visualization | Benchmarking |
| 11-12 | User testing | Bug fixes | Refinement | Deployment |

**Speedup**: 2-3× faster with coordinated AI agents

---

**Related**: [Architecture Overview](../01-architecture/overview.md) | [Task Breakdown](../03-tasks/task-breakdown.md) | [AI Agent Templates](./ai-agent-templates.md) | [Decisions & Rationale](./decisions.md)
