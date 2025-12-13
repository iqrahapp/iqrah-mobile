# Juz 30 Recommended Presets

> **Last Updated**: 2025-12-13
> **Validated**: n=10 seeds, SS40 confirmation

---

## Dedicated Default (SS40)

**File**: `configs/scenarios/juz_amma_dedicated.yaml`

```yaml
name: juz_amma_dedicated
session_size: 40
target_reviews_per_active: 0.06
intro_min_per_day: 5
max_working_set_ratio_of_goal: 1.0
```

### Validation Results (n=10 seeds, 180d)

| Metric | Value | Threshold | Status |
|--------|-------|-----------|--------|
| **Coverage%** | 65.8 | > baseline | ✅ +17pp |
| at_risk_ratio_0.9 | 0.029 | ≤ 0.15 | ✅ |
| p10_R_today | 0.920 | ≥ 0.85 | ✅ |
| total_active | 416 | - | - |

### Why This Config?

1. **+17pp coverage** vs SS20 baseline (48.9% → 65.8%)
2. **Bounded at-risk** even with weak student profile
3. **M2.7 due-age sorting** provides primary backlog protection

---

## Metric Definitions

| Type | Metrics |
|------|---------|
| **Horizon** | Coverage% (mean_retrievability_horizon), coverage_h_0_9 |
| **Today** | mean_R_today, p10_R_today, at_risk_ratio_0.9 |

See `docs/design/metrics_definitions.md` for details.

---

## Safety Layers

1. **M2.7 due-age sorting** (primary): Most overdue items reviewed first
2. **Cluster gate**: Blocks new intros when cluster energy < threshold
3. **M2.8 backlog gating** (fallback): Disables intro floor when severe

In healthy configs, backlog gating rarely triggers.

---

## Legacy

Previous default: `configs/scenarios/juz_amma_dedicated_ss20_legacy.yaml`
