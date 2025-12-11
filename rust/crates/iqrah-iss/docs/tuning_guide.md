# ISS Parameter Tuning Guide

## When to Tune

**Don't tune if:**
- Coverage is acceptable for your use case
- System is stable (no give-ups, reasonable success rate)

**Do tune if:**
- Coverage too low (<20% on large goals)
- Coverage too high at expense of retention (>70% but low success rate)
- Student profiles not differentiated enough

---

## Key Parameters (by impact)

### 1. max_working_set
**What:** Maximum active items before consolidation
**Impact:** High (directly limits capacity)
**Typical:** 50-400 depending on goal size and student
**Tune:** Increase if coverage plateaus early, decrease if give-ups occur

### 2. cluster_stability_threshold
**What:** Weighted energy required for expansion
**Impact:** High (gates introduction)
**Typical:** 0.15-0.40
**Tune:** Lower for faster expansion, raise for more consolidation

### 3. drift_alpha_max
**What:** Daily energy decay rate for new items
**Impact:** Medium-High (affects failure rate)
**Typical:** 0.005-0.02
**Tune:** Lower if high failure rate (>30%), raise if too easy (<10% failure)

### 4. cluster_expansion_batch_size
**What:** Items introduced per expansion (K)
**Impact:** Medium (affects expansion speed)
**Typical:** 3-15
**Tune:** Increase for faster coverage, decrease for small goals

---

## Recommended Profiles by Goal Size

### Small Goals (7-50 items)
```yaml
max_working_set: 50
cluster_stability_threshold: 0.30
cluster_expansion_batch_size: 3
drift_alpha_max: 0.02
```
**Expected:** 70-90% coverage, deep consolidation

### Medium Goals (50-200 items)
```yaml
max_working_set: 150
cluster_stability_threshold: 0.25
cluster_expansion_batch_size: 5
drift_alpha_max: 0.015
```
**Expected:** 50-70% coverage, balanced learning

### Large Goals (200-600 items)
```yaml
max_working_set: 350
cluster_stability_threshold: 0.20
cluster_expansion_batch_size: 10
drift_alpha_max: 0.01
```
**Expected:** 40-50% coverage, distributed practice

---

## Troubleshooting

### "Coverage stalls at 20%"
**Symptoms:** Active items reaches max_working_set, no more expansion
**Fix:** Increase `max_working_set: 350 → 500`

### "Too many failures (>35%)"
**Symptoms:** Items not stabilizing, cluster energy low
**Fix:** Reduce `drift_alpha_max: 0.01 → 0.008`

### "Small goals (<20 items) don't complete"
**Symptoms:** Cluster threshold too high for small clusters
**Fix:** Create separate profile with `cluster_threshold: 0.15` and `batch_size: 2`

### "Give-ups occurring"
**Symptoms:** Student frustration leading to early quit
**Fix:** Reduce `max_reviews_per_day`, increase `persistence_threshold`

---

## Performance Benchmarks

| Goal | Items | Days | Coverage | Failure Rate |
|------|-------|------|----------|--------------|
| Fatiha | 7 | 30 | 77% | 5% |
| Juz 30 | 564 | 180 | 42% | 7% |

Both achieve excellent retention with proper tuning.
