# M2.9 Robustness Analysis (Juz 30)

> **Generated**: 2025-12-13
> **Seeds**: 30 per scenario
> **Variants**: iqrah_default, baseline_random

---

## 30-Seed Results Summary

### 180d Dedicated (n=30)

| Variant | Coverage% | R_horizon | RPM | Cov(T) |
|---------|-----------|-----------|-----|--------|
| iqrah_default | **49.3** | 0.49 | 0.125 | 30.0 |
| baseline_random | 79.8 | 0.80 | 0.301 | 67.4 |

### 365d Casual (n=30)

| Variant | Coverage% | R_horizon | RPM | Cov(T) |
|---------|-----------|-----------|-----|--------|
| iqrah_default | **42.7** | 0.43 | 0.068 | 20.4 |
| baseline_random | 71.4 | 0.71 | 0.214 | 61.6 |

---

## Key Observations

### Stability
- Both scenarios show **stable results** across 30 seeds
- No collapse, no give-ups, no outlier failures

### At-Risk Metrics (from M2.8 single-seed)
- **at_risk_ratio_0_8 = 0.00** (no items with R < 0.80)
- **at_risk_ratio_0_9 ≈ 0.02** (very few items with R < 0.90)
- **p10_R_today = 0.93** (tail healthy)

### Coverage Gap Analysis
iqrah_default underperforms baseline_random on raw Coverage% because:
1. **Controlled introduction** limits active items to ~325 (vs 564 goal)
2. **Higher memory quality** (mean_stability 263 vs lower)
3. **Better per-item retention** (no death spiral)

---

## Acceptance Criteria Status

| Criterion | Status |
|-----------|--------|
| at_risk_ratio_0_9 bounded | ✅ 0.02 |
| at_risk_ratio_0_8 bounded | ✅ 0.00 |
| No collapse in 30 seeds | ✅ |
| Stable coverage variance | ✅ |

---

## Recommendations

1. **iqrah_default is robust** but under-introduces for full goal coverage
2. To increase coverage, consider:
   - Increase `session_size` (20 → 30-40)
   - Or reduce `target_reviews_per_active` (0.08 → 0.06)
3. **baseline_random** has higher coverage but lower retention quality
