# M2.9.1 Student Profile Variability (Juz 30)

> **Git SHA**: M2.9.1
> **Scenarios**: juz_amma_dedicated_{strong,avg,weak}
> **Seeds**: n=10 per profile
> **Days**: 180d

---

## Profile Definitions

| Param | Strong | Avg | Weak |
|-------|--------|-----|------|
| forgetting_rate_mult | 0.6 | 0.9 | 1.5 |
| spacing_sensitivity | 1.5 | 1.2 | 0.8 |
| fatigue_onset_min | 60 | 45 | 30 |
| drift_alpha_max | 0.008 | 0.01 | 0.015 |
| skip_day_prob | 0.02 | 0.05 | 0.10 |
| persistence_threshold | 2000 | 1500 | 800 |

---

## Results Summary (n=10 seeds)

| Profile | Coverage% | GaveUp% | at_risk_0.9 | p10_R | Status |
|---------|-----------|---------|-------------|-------|--------|
| Strong | 48.7 | 0.0 | ~0.03 | 0.94 | ✅ |
| **Avg** | 48.9 | 0.0 | ~0.02 | 0.94 | ✅ |
| **Weak** | 49.2 | 0.0 | **0.08-0.11** | 0.90 | ✅ |

---

## Acceptance Criteria

| Criterion | Requirement | Weak Result | Status |
|-----------|-------------|-------------|--------|
| at_risk_ratio_0_9 | < 0.30 | 0.11 | ✅ PASS |
| No runaway p90_due_age | stable | 145d | ✅ PASS |
| No collapse | 0% give-up | 0% | ✅ PASS |

---

## Key Findings

1. **Weak profile does NOT collapse** - scheduler adapts appropriately
2. **at_risk_ratio_0_9** stays bounded (0.08-0.11 < 0.30 threshold)
3. **Similar coverage** across all profiles (~49%) due to capacity-controlled introduction
4. **Tail health** (p10_R_today) drops from 0.94 to 0.90 but remains healthy
