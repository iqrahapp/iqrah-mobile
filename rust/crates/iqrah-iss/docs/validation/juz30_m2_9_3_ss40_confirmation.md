# SS40 Configuration Confirmation (M2.9.3)

> **Git SHA**: M2.9.3
> **Config**: session_size=40, target_reviews_per_active=0.06, intro_min_per_day=5
> **Seeds**: n=10
> **Days**: 180d

---

## Horizon Metrics

| Metric | SS40 | Baseline (SS20) | Δ |
|--------|------|-----------------|---|
| **Coverage%** | **65.8** | 48.9 | **+17pp** |
| Cov(T) | 50.2 | 31.1 | +19pp |
| total_active | 416 | 312 | +33% |
| RPM | 0.213 | 0.127 | +68% |

---

## Today Health (Final Day 179)

| Metric | SS40 Value | Threshold | Status |
|--------|------------|-----------|--------|
| mean_R_today | 0.958 | - | ✅ |
| **p10_R_today** | **0.920** | ≥ 0.85 | ✅ |
| at_risk_ratio_0.8 | 0.00 | - | ✅ |
| **at_risk_ratio_0.9** | **0.029** | ≤ 0.15 | ✅ |

---

## Today Health Over Time

| Day | mean_R | total_active | at_risk_0.9 | p10_R |
|-----|--------|--------------|-------------|-------|
| 30 | 0.968 | 205 | 0.024 | 0.938 |
| 60 | 0.957 | 331 | 0.042 | 0.906 |
| 90 | 0.987 | 333 | 0.000 | 0.975 |
| 120 | 0.978 | 408 | 0.000 | 0.957 |
| 179 | 0.958 | 416 | 0.029 | 0.920 |

---

## Weak Profile Under SS40

| Metric | Value | Status |
|--------|-------|--------|
| Coverage% | 62.3 | - |
| at_risk_ratio_0.9 | 0.023-0.028 | < 0.15 ✅ |
| p10_R_today | 0.91 | ≥ 0.85 ✅ |
| Give-up | 0% | ✅ |

---

## Acceptance Criteria

| Criterion | Requirement | Result | Status |
|-----------|-------------|--------|--------|
| Coverage improvement | > baseline | +17pp | ✅ PASS |
| at_risk_ratio_0.9 | ≤ 0.15 | 0.029 | ✅ PASS |
| p10_R_today | ≥ 0.85 | 0.920 | ✅ PASS |
| No collapse/give-up | 0% | 0% | ✅ PASS |
| Weak profile bounded | at_risk < 0.15 | 0.028 | ✅ PASS |

---

## Recommendation

**PROMOTE SS40 as Recommended Dedicated Preset**

```yaml
name: juz_amma_dedicated_ss40
session_size: 40
target_reviews_per_active: 0.06
intro_min_per_day: 5
```
