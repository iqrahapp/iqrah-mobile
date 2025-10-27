# Iqrah Audio - Implementation Guide
**How to Use the SOTA Architecture Documents**

---

## Document Overview

You now have **two comprehensive documents** that define Iqrah Audio for the next 3 years:

### 1. IQRAH_SOTA_ARCHITECTURE_V1.md
**Purpose:** Complete technical specification  
**Use:** Reference for all implementation decisions  
**Sections:**
- System architecture (8 core modules)
- Algorithms and methods (with code examples)
- Model specifications and parameters
- Validation metrics and targets
- Technology stack
- Progressive rollout strategy (Offline → Real-time → Mobile)

### 2. IQRAH_TASK_DECOMPOSITION.md
**Purpose:** Actionable task breakdown  
**Use:** Project management and AI agent delegation  
**Contains:**
- Mermaid dependency graph (visual roadmap)
- 100+ specific tasks with IDs
- AI-agent-friendly task templates
- Priority matrix and timeline
- Resource allocation guidance

---

## Key Architectural Decisions

### ✅ What's Already Working (Keep)

1. **Pitch Extraction:**
   - SwiftF0 (primary) - Already implemented, 42× faster than CREPE
   - CREPE (fallback) - Good for melodic passages
   - ✅ No changes needed

2. **Phoneme Alignment:**
   - Wav2Vec2 CTC - Currently using MMS
   - **Action:** Fine-tune Wav2Vec2-BERT for <1% PER (SOTA upgrade)
   - Windowed alignment within words - Already working

3. **Comparison Framework:**
   - Soft-DTW rhythm comparison - Good foundation
   - ΔF0 melody comparison - Good foundation
   - GOP pronunciation - Already implemented
   - ✅ Core structure solid, enhance with SOTA features

### 🔄 What Needs Enhancement (Add)

1. **Tajweed Validation (Priority 1):**
   - **Madd:** Currently basic, upgrade to 99% rule-based accuracy
   - **Ghunnah:** Add formant analysis + MLP classifier (NEW)
   - **Qalqalah:** Add burst detection + SVM (NEW)
   - **Progressive rollout:** madd → ghunnah → qalqalah

2. **Voice Quality (Priority 2):**
   - **Add:** OpenSMILE eGeMAPS (88-d features)
   - **Add:** Vibrato detection (rate, extent, regularity)
   - **Add:** Breathiness (H1-H2, HNR, CPP)
   - **Add:** X-vector embeddings (style matching)

3. **Advanced Prosody (Priority 3):**
   - **Add:** nPVI, Varco (rhythm metrics beyond DTW)
   - **Add:** Fujisaki decomposition (melody structure)
   - **Add:** Maqam classification (CNN on Maqam478 dataset)
   - **Add:** Declination modeling

4. **Feedback Generation (Priority 4):**
   - **Add:** Pedagogical text generation
   - **Add:** Progress tracking (PostgreSQL)
   - **Add:** Actionable recommendations

### ❌ What to Defer (Phase 2+)

1. **Real-time streaming:** Wait until Phase 1 complete (offline 90% accurate)
2. **Mobile deployment:** Wait until Phase 2 (real-time working)
3. **Complex Tajweed rules:** Idghaam, Ikhfaa, Iqlaab (after basic rules perfect)
4. **Multi-Qira'at support:** Focus on Hafs first

---

## Implementation Strategy

### Phase 1 Focus: Offline E2E (Months 1-6)

**Goal:** 90% accuracy on basic Tajweed, comprehensive prosodic analysis

**Critical Path:**
```
1. Fine-tune Wav2Vec2-BERT (Weeks 1-4)
   ↓
2. Implement Madd validator (Week 5)
   ↓
3. Train Ghunnah classifier (Weeks 6-8)
   ↓
4. Integrate OpenSMILE + prosody (Weeks 9-12)
   ↓
5. Build comparison engine (Weeks 13-14)
   ↓
6. Feedback generation (Weeks 15-16)
   ↓
7. Validation study (Weeks 17-24)
```

**Parallel Workstreams:**
- Voice quality analysis (OpenSMILE, X-vectors) - Can start immediately
- Qalqalah validator - After data collection (Week 6+)
- Feedback templates - Can start early (Week 3+)

### How to Use AI Agents Effectively

#### ✅ Good Tasks for AI Agents (High Success Rate)

**Pure Implementation:**
- Task: "Implement audio loader supporting MP3/WAV/WebM"
- Why: Clear input/output, standard libraries
- Template: See Template 1 in decomposition doc

**Feature Extraction:**
- Task: "Extract OpenSMILE eGeMAPS features"
- Why: Well-documented API, straightforward integration
- Code: One function, 20-30 lines

**Data Processing:**
- Task: "Prepare Ghunnah training CSV from existing annotations"
- Why: Clear schema, deterministic logic
- Output: CSV file with validation checks

**Template-Based:**
- Task: "Generate feedback text for madd violations"
- Why: Fill-in-the-blank templates
- Example: "Your madd at {timestamp} is {actual} but should be {expected}"

#### ⚠️ Medium Tasks (Needs Guidance)

**Training Scripts:**
- Task: "Train Wav2Vec2-BERT on Quranic data"
- Why: Standard flow but needs hyperparameter tuning
- Provide: Exact config file, HuggingFace example
- Monitor: Training curves, validation metrics

**Optimization:**
- Task: "Implement Soft-DTW with Sakoe-Chiba band"
- Why: Non-trivial algorithm, needs mathematical understanding
- Provide: Pseudocode, NumPy reference implementation
- Test: Unit tests with known inputs/outputs

**Integration:**
- Task: "Integrate 3 validators into comparison engine"
- Why: Multiple components, error handling needed
- Provide: Interface contracts, test cases
- Review: Edge case handling

#### ❌ Hard Tasks (Human Required)

**Research:**
- Task: "Design qalqalah burst detection algorithm"
- Why: No established SOTA, requires experimentation
- Approach: Human designs algorithm → AI implements

**Expert Annotation:**
- Task: "Create 100-ayah test set with phoneme boundaries"
- Why: Requires domain expertise (Tajweed knowledge)
- Approach: Hire Qaris → AI processes annotations

**Architecture Decisions:**
- Task: "Redesign comparison fusion weights"
- Why: Requires understanding of user needs, trade-offs
- Approach: Human decides → AI implements

**User Studies:**
- Task: "Validate correlation with human ratings"
- Why: Requires real users, qualitative analysis
- Approach: Human designs study → AI analyzes data

---

## AI Agent Task Template

When delegating to AI agents, use this format:

```markdown
## Task: [Clear, specific title]

**Context:**
- Module: [Which of the 8 modules]
- Purpose: [What problem this solves]
- Location: [Exact file path]

**Requirements:**
1. [Specific requirement 1]
2. [Specific requirement 2]
...

**Input:** [Exact type and format]
**Output:** [Exact type and format]

**Dependencies:**
- [List all required libraries]
- [List any previous tasks that must be done first]

**Test Cases:**
1. [Concrete example: input → expected output]
2. [Edge case: input → expected behavior]
...

**Acceptance Criteria:**
- [ ] All test cases pass
- [ ] Type hints complete
- [ ] Docstring with examples
- [ ] No runtime errors

**Estimated Time:** [2-8 hours typical for AI agent tasks]

**Reference Code:** [Link to similar working code if exists]
```

**Example:** See templates in IQRAH_TASK_DECOMPOSITION.md

---

## Module Interfaces (Contract for AI Agents)

### Module 1: Preprocessing
```python
def preprocess_audio(file_path: str) -> dict:
    """
    Returns:
    {
        "audio_path": str,
        "sample_rate": 16000,
        "duration": float,
        "segments": [{"start": float, "end": float}],
        "quality_metrics": {"snr_db": float, "clipping_ratio": float}
    }
    """
```

### Module 2: Pitch Extraction
```python
def extract_pitch(audio: np.ndarray, sr: int) -> dict:
    """
    Returns:
    {
        "pitch_hz": np.ndarray,
        "times": np.ndarray,
        "confidence": np.ndarray,
        "voicing": np.ndarray,
        "method": str,
        "stats": {"mean_hz": float, "std_hz": float, "range_hz": tuple}
    }
    """
```

### Module 3: Phoneme Alignment
```python
def align_phonemes(audio: np.ndarray, sr: int, surah: int, ayah: int) -> dict:
    """
    Returns:
    {
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

### Module 4: Tajweed Validation
```python
class TajweedValidator:
    def validate(self, phonemes: list, audio: np.ndarray) -> dict:
        """
        Returns:
        {
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

**Clear contracts = AI agents can implement without confusion**

---

## Validation Checklist

Before moving to next phase, verify:

### Phase 1 Complete When:

**Technical Metrics:**
- [ ] Phoneme Error Rate (PER) < 1%
- [ ] Boundary precision: 90% within 50ms
- [ ] Madd accuracy: 99%+
- [ ] Ghunnah accuracy: 85%+
- [ ] Qalqalah accuracy: 80%+
- [ ] Overall comparison correlation r > 0.7 vs human ratings

**User Metrics:**
- [ ] 100 expert-rated test cases complete
- [ ] 20-30 alpha users tested
- [ ] Qualitative feedback positive (4+/5 average)
- [ ] <5% technical issues reported

**System Metrics:**
- [ ] Latency p95 < 5 seconds per ayah (offline acceptable)
- [ ] Memory usage < 4GB
- [ ] No crashes on 100 diverse audio files

**Documentation:**
- [ ] All modules have docstrings
- [ ] README with installation guide
- [ ] API documentation complete
- [ ] Architecture decisions recorded

### Phase 2 Ready When:

- [ ] Phase 1 all checkboxes above ✅
- [ ] GPU infrastructure provisioned
- [ ] Quantization experiments done (INT8 validated)
- [ ] Streaming protocol designed
- [ ] Latency target <500ms achievable on benchmark

---

## Risk Management

### High-Risk Items (Monitor Closely)

| Risk | Mitigation | Status Check |
|------|------------|--------------|
| **Wav2Vec2 training fails** | Use pretrained MMS as fallback | Week 2: Check training curves |
| **Ghunnah accuracy <70%** | Collect more training data | Week 8: Validate on test set |
| **User testing reveals confusion** | Simplify feedback language | Week 20: User interview round |
| **GPU cost exceeds budget** | Implement aggressive caching | Monthly: Check cloud bills |

### Medium-Risk Items (Contingency Plans)

| Risk | Contingency |
|------|-------------|
| Qalqalah doesn't reach 80% | Defer to Phase 3, ship without it |
| Real-time latency >500ms | Launch with async mode only |
| Mobile model too large | Hybrid: basic on-device, advanced server |
| Expert annotations delayed | Use rule-based validation initially |

---

## Decision Log

### Why Wav2Vec2-BERT over MMS?

**Decision:** Fine-tune Wav2Vec2-BERT instead of using pretrained MMS  
**Rationale:**
- SOTA research shows 0.16% PER achievable (vs MMS ~2-5%)
- Task-adaptive pretraining critical for Quranic domain
- MMS designed for 1000+ languages (generalist), we need specialist
- Cost: ~€500-1000 training, worth it for accuracy gain

**Trade-off:** 1-2 weeks training time vs immediate use of MMS  
**Verdict:** Worth it for production quality

### Why Progressive Tajweed Rollout?

**Decision:** madd → ghunnah → qalqalah → complex rules  
**Rationale:**
- Madd is easiest (rule-based, 99% accurate)
- Ghunnah requires ML but has established methods
- Qalqalah is exploratory (no strong SOTA)
- Complex rules need more research

**Trade-off:** Delayed full feature set vs higher confidence  
**Verdict:** Better to ship incrementally with high quality

### Why Offline-First Instead of Real-Time?

**Decision:** Perfect offline pipeline before streaming  
**Rationale:**
- Real-time adds 10× complexity (WebSocket, VAD, chunking)
- Offline allows thorough validation (impossible with streaming)
- Users accept 5-10s latency for uploaded audio
- Easier to debug and iterate

**Trade-off:** Delayed mobile launch by 6 months  
**Verdict:** Foundation first, speed second

### Why OpenSMILE Over Custom Feature Extraction?

**Decision:** Use OpenSMILE eGeMAPS instead of custom  
**Rationale:**
- 88 standardized features (research-validated)
- Maintained by industry experts
- Cross-study comparability
- Saves 2-3 weeks implementation time

**Trade-off:** Black-box vs full control  
**Verdict:** Standardization > customization

---

## Success Metrics

### Phase 1 Success Definition:

**Quantitative:**
- 100 expert-validated test cases: r > 0.7 correlation
- PER < 1% (SOTA is 0.16%, acceptable margin)
- Madd: 99%, Ghunnah: 85%, Qalqalah: 80%
- Latency p95 < 5s (offline acceptable)

**Qualitative:**
- Users say "This actually helps me improve"
- Teachers willing to recommend to students
- Expert Qaris validate accuracy

**Business:**
- 50+ beta users actively using weekly
- 70%+ retention after 1 month
- Ready for B2B pilot (Islamic schools)

### Phase 2 Success Definition:

- Real-time latency <500ms p95
- 10+ concurrent users supported
- 70%+ cache hit rate
- $0.10/user/month cost (GPU amortized)

### Phase 3 Success Definition:

- iOS + Android apps in stores
- <300ms on-device latency
- 4.5+ stars average rating
- 1000+ weekly active users

---

## Next Steps (Week 1)

### Main Developer:
1. **Setup infrastructure:**
   - AWS/Lambda Labs account
   - Download Tarteel dataset (~100GB)
   - Setup HuggingFace Hub for models
   - PostgreSQL for user data

2. **Begin Wav2Vec2 training:**
   - Follow T3.1.1 → T3.1.2 → T3.1.3
   - Monitor training curves
   - Validate PER on test set

3. **Coordinate AI agents:**
   - Assign T1.1.1, T2.1.1, T5.1.1 (independent tasks)
   - Review daily progress
   - Merge completed PRs

### AI Agent 1: Audio Preprocessing
- Task: T1.1.1 - Support MP3/WAV/WebM/M4A
- Deliverable: `audio_loader.py` with tests
- Deadline: Day 3

### AI Agent 2: Pitch Extraction
- Task: T2.1.1 - Integrate SwiftF0
- Deliverable: `pitch_extractor_swiftf0.py` enhanced
- Deadline: Day 2

### AI Agent 3: Voice Quality
- Task: T5.1.1 - OpenSMILE wrapper
- Deliverable: `opensmile_features.py` with eGeMAPS
- Deadline: Day 4

### End of Week 1:
- ✅ Training started (main dev)
- ✅ 3 modules enhanced (AI agents)
- ✅ Integration testing plan ready
- ✅ Week 2 tasks assigned

---

## FAQs

**Q: Can I change the architecture?**  
A: This is the 3-year commitment design. Small tweaks OK, major changes should wait for v2.0 (2028).

**Q: What if Wav2Vec2 training fails?**  
A: Fallback to MMS (already working). You'll get 2-5% PER instead of <1%, still acceptable.

**Q: Should I parallelize Phase 1 and Phase 2?**  
A: No. Validate Phase 1 accuracy first. Real-time optimization is pointless if core quality is bad.

**Q: What if I can't collect 1000 Ghunnah examples?**  
A: Start with 300-500. Accuracy will be ~75-80% instead of 85%. Still useful, iterate later.

**Q: Can AI agents handle everything?**  
A: No. They're great for 60-70% of tasks (implementation, data processing, integration). You're needed for: architecture, research, training, validation, user studies.

**Q: What if a task takes longer than estimated?**  
A: Estimates are for ideal conditions. 2× multiplier typical. Adjust timeline, not quality bar.

**Q: How do I know when to move to Phase 2?**  
A: When all checkboxes in "Phase 1 Complete When" section are ✅. Don't rush.

---

## Conclusion

You now have:

1. **Complete architecture** - Every module defined with algorithms, code examples, dependencies
2. **Actionable roadmap** - 100+ tasks broken down, prioritized, estimated
3. **AI agent templates** - Ready-to-use task formats for delegation
4. **Validation criteria** - Clear metrics for each phase
5. **Risk mitigation** - Contingency plans for major risks

**Your job:** Execute methodically, validate thoroughly, iterate based on user feedback.

**AI agents' job:** Implement well-defined tasks, integrate modules, test thoroughly.

**Commitment:** This design is stable. Focus 100% on execution, not redesign.

**Timeline:** 6 months offline, 6 months real-time, 6 months mobile = 18 months to full production.

**Success:** When users say "Iqrah made me a better Qari" and experts validate accuracy.

---

**Good luck! 🚀**
