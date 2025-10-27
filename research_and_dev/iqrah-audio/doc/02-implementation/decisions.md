# Architecture Decisions & Rationale

[↑ Navigation](../NAVIGATION.md)

---

## DECISION RATIONALE

### Q: Why use pre-trained Muaalem instead of training Wav2Vec2-BERT?
**Decision**: Use `obadx/muaalem-model-v3_2` pre-trained model as-is
**Rationale**:
- **Outputs phonemes + sifat**: Muaalem provides both phoneme recognition AND comprehensive Tajweed properties (10+ rules) in a single model
- **No training required**: Eliminates 8-10 weeks of fine-tuning + €1,500 GPU costs
- **Sufficient accuracy**: <2% PER on Quranic audio (vs <1% with custom fine-tuning)
- **Faster MVP**: Phase 1 reduced from 6 months to 4 months
- **Comprehensive Tajweed from Day 1**: 10+ Tajweed rules available immediately (vs 3 rules in grapheme-based approach)

**Trade-off**: Slightly higher PER (2% vs 1%) but massive time/cost savings
**Verdict**: Worth it for MVP; can fine-tune in Phase 2 if needed

### Q: Why two-tier Tajweed architecture?
**Decision**: Tier 1 (Baseline sifat from Muaalem) + Tier 2 (Specialized modules)
**Rationale**:
- **Tier 1 advantages**:
  - Free: No training, data collection, or annotation needed
  - Comprehensive: 10+ Tajweed rules from Day 1
  - Sufficient for MVP: 70-85% accuracy acceptable for initial feedback
  - Baseline for comparison: Establishes floor for Tier 2 improvements
- **Tier 2 advantages**:
  - Targeted enhancement: Focus effort on most important/difficult rules
  - Pluggable: Can enable/disable per rule without affecting others
  - Incremental: Add modules one by one as needed
  - Higher accuracy: 90-99% for critical rules (Madd, Ghunnah)
- **Design principles**:
  - Modularity: Each rule = independent module
  - Graceful degradation: If Tier 2 fails, fall back to Tier 1
  - Resource efficiency: Only run expensive Tier 2 for low-confidence cases

**Trade-off**: More complex architecture vs better accuracy + faster MVP
**Verdict**: Two-tier enables fast MVP with clear upgrade path

### Q: Why Progressive Tier 2 Rollout?
**Decision**: Madd (Tier 2) → Ghunnah (Tier 2) → Qalqalah (Tier 2) → others
**Rationale**:
- **Madd (PRIORITY 1)**: Most critical rule, probabilistic approach achieves 95%+
- **Ghunnah (PRIORITY 2)**: Well-established formant analysis methods, can reach 90%+
- **Qalqalah (PRIORITY 3)**: Acoustic burst detection, 85%+ achievable
- **Others**: Complex rules (Idgham, Ikhfa) deferred to Phase 2+

**Trade-off**: Delayed full Tier 2 coverage vs high quality per rule
**Verdict**: Ship incrementally with high confidence

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
- Tier 1 baseline: 100 examples per rule → 70-85% (all 10+ rules)
- Madd (Tier 2): 500 examples (all types) → 95%+
- Ghunnah (Tier 2): 300 examples → 90%+
- Qalqalah (Tier 2): 200 examples → 85%+

**Prosody Validation**:
- 100 expert-rated pairs (10-point scale per dimension)
- Correlation: Automated vs human
- Target: r > 0.75 for rhythm, melody, style

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

## SUCCESS DEFINITIONS

### Phase 1 Success

**Quantitative**:
- 100 expert cases: r > 0.75
- PER < 2% (using pre-trained Muaalem)
- Tier 1 baseline: 10+ rules at 70-85% accuracy
- Tier 2 specialized: Madd 95%+, Ghunnah 90%+, Qalqalah 85%+
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

**Q: If Muaalem PER >5% on learner audio?**
A: Fine-tune model in Phase 2 on collected edge cases. For MVP, use confidence flags to warn users.

**Q: Should I parallelize Phase 1 and Phase 2?**
A: No. Validate Phase 1 accuracy first. Real-time optimization pointless if core quality bad.

**Q: If Tier 1 Ghunnah accuracy <70%?**
A: Enable Tier 2 formant analysis. If still insufficient, collect more training data for fine-tuning in Phase 2.

**Q: Should I skip Tier 1 and go straight to Tier 2?**
A: No. Tier 1 provides comprehensive coverage (10+ rules) with zero effort. Build Tier 2 incrementally on top.

**Q: Can AI agents handle everything?**
A: No. 60-70% of tasks. You're needed for: architecture, research, training, validation, user studies.

**Q: Task takes longer than estimated?**
A: Estimates ideal conditions. 2× multiplier typical. Adjust timeline, not quality bar.

**Q: When move to Phase 2?**
A: When all Phase 1 checkboxes ✅. Don't rush.

---

**Related**: [Architecture Overview](../01-architecture/overview.md) | [Implementation Guide](./guide.md) | [Task Breakdown](../03-tasks/task-breakdown.md)
