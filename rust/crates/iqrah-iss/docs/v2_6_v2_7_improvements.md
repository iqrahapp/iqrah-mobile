# ISS v2.6-v2.7: From Death Spiral to Stability

## Problem Statement

ISS v2.5 exhibited a death spiral on large goals:
- 549/564 items introduced in 30 days (greedy introduction)
- 67% failure rate (FSRS stability collapse)
- 0.2% coverage (unusable)

**Root causes:**
1. No cluster-based introduction control
2. Exponential forgetting too harsh for low-stability items
3. Initial placement bypassed gates

---

## Solution Architecture

### v2.6: Cluster-Based Introduction

**Key changes:**
- Prerequisite gate enabled in ISS
- Working set limit (max 50 active items)
- Cluster stability gate (weighted energy threshold)
- Batch-based expansion (K=3 items at a time)

**Result**: Active items 549 → 50, but coverage still low (items not stabilizing).

---

### v2.7: FSRS Formula Fix

**The breakthrough**: Changed recall probability formula

```rust
// OLD (exponential - death spiral):
R = exp(-t / S)
// S=5, t=10 → R=0.135 (13.5%)

// NEW (FSRS power law - survivable):
R = 1 / (1 + t / (9*S))
// S=5, t=10 → R=0.818 (81.8%)
```

**Impact**: 6x higher recall probability for low-stability items.

**Why it works:**
- Exponential: Harsh penalty for S<10 days
- Power law: Gentler, allows items to build stability gradually
- Prevents death spiral: Items don't collapse to S=0.05 and stay there

---

### Supporting Changes

**Drift reduction:**
```yaml
drift_alpha_max: 0.10 → 0.01  # 10x slower energy decay
drift_alpha_min: 0.02 → 0.002
```

**Parameter scaling** (after formula proved stable):
```yaml
max_working_set: 50 → 350           # Increase capacity
cluster_threshold: 0.25 → 0.20      # Faster expansion
cluster_batch_size: 5 → 10          # Larger batches
```

---

## Results

| Metric | v2.5 | v2.6 | v2.7 | Change |
|--------|------|------|------|--------|
| Coverage (Juz 30) | 0.2% | 1.4% | 42% | **+200x** |
| Failure rate | 67% | 23% | 7% | **-90%** |
| Active items | 549 | 50 | 350 | Controlled |
| Give-ups | 0% | 0% | 0% | ✓ |

---

## Learning Strategy Trade-offs

### Cluster Gate (iqrah) vs Distributed Practice (random)

The cluster gate implements **blocked practice** - deep consolidation of small sets before expanding. This is optimal for small/medium goals where mastery of a focused set is the objective.

For very large corpora (500+ items over extended periods), **distributed practice** may be more efficient. FSRS naturally creates spacing across the entire item set, and restricting the working set to enable consolidation has an opportunity cost in terms of coverage.

**Benchmark results confirm this trade-off:**

| Strategy | Small Goals (7 items) | Large Goals (564 items) | Use Case |
|----------|----------------------|-------------------------|----------|
| Cluster gate (iqrah) | **+77%** | -5% | Master specific content |
| Distributed (random) | Baseline | **+5%** | Survey large corpus |

**Both strategies achieve high retention** (93-95% success rate). The choice depends on learning objectives:
- **Depth**: Use cluster gate (quality over consolidation)
- **Breadth**: Use distributed practice (coverage over consolidation)

This is consistent with spacing effect research showing benefits of interleaving for large, complex domains vs blocking for focused skill acquisition.

---

## Lessons Learned

1. **Exponential forgetting too harsh**: FSRS power law is more forgiving
2. **Young items need protection**: +50% boost prevents early collapse
3. **Cluster gating works for focused learning**: Excels at small/medium goals
4. **Distributed practice works for large corpora**: FSRS handles spacing naturally
5. **Parameter interactions matter**: Formula change enabled scaling parameters

---

## Configuration Profiles

### Focused Learning (`juz_amma_dedicated.yaml`)
- **Strategy**: Cluster-based consolidation
- **Working set**: 350 items
- **Target**: Deep mastery of subset (42% coverage, 93% retention)
- **Best for**: Mastering specific surahs, focused study

### Full Immersion (optional)
- **Strategy**: Distributed practice across corpus
- **Working set**: Match goal size (no limit)
- **Target**: Maximum coverage (80% coverage, 95% retention)
- **Best for**: Comprehensive corpus review, vocabulary building

---

## Future Work (v3.0 Roadmap)

See architectural improvements for potential future enhancements:
- Energy diffusion scope expansion
- Adaptive threshold (performance-based)
- Decouple energy (urgency) from recall (stability)
- FSRS-aware drift (scales with S growth)

---

## Files Modified

### Core Changes
- `brain.rs`: FSRS power law formula, young item boost
- `simulator.rs`: Cluster gate logic, candidate filtering

### Configuration
- `juz_amma_dedicated.yaml`: Updated drift and cluster parameters
- `surah_fatiha_dedicated.yaml`: Small goal optimization

### Documentation
- This document (`v2_6_v2_7_improvements.md`)
- `tuning_guide.md`: Parameter recommendations by goal size
