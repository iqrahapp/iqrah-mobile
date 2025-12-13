# Juz 30 Casual Pareto Sweep Results

> **Date**: 2025-12-13
> **Seeds**: n=5 (screening)
> **Days**: 365

---

## Results

| Config | session_size | max_ws | Coverage% | Status |
|--------|--------------|--------|-----------|--------|
| Baseline | 10 | 150 | 44.6 | - |
| **SS15 (WINNER)** | 15 | 300 | **51.5** | ✅ +7pp |

---

## Winner Config

```yaml
name: juz_amma_casual_365d
session_size: 15
max_working_set: 300
intro_min_per_day: 4
target_reviews_per_active: 0.05
```

**Why it won:**
- +7pp coverage (44.6% → 51.5%)
- More items active (332 vs 266)
- Same Today health profile
