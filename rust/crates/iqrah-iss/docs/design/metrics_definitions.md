# ISS Metrics Definitions (M2.9)

> **Version**: M2.9 (Dec 2025)
> **Status**: Horizon vs Today separation enforced

---

## Horizon Metrics (End-of-Run Evaluation)

Metrics evaluated at simulation end using evaluation horizon T_eval.

| Metric | Formula | Description |
|--------|---------|-------------|
| `mean_retrievability_horizon` | avgᵢ(Rᵢ(Tₑᵥₐₗ)) | Mean FSRS retrievability at horizon |
| `coverage_h_0_9_horizon` | #{Rᵢ ≥ 0.9} / goal | Items with R ≥ 0.9 at horizon |
| `coverage_h_0_7_horizon` | #{Rᵢ ≥ 0.7} / goal | Items with R ≥ 0.7 at horizon |
| `introduced_ratio` | introduced / goal | Progress: fraction introduced |

> **Note**: "Coverage%" = `mean_retrievability_horizon` × 100

---

## Today Metrics (Daily State Health)

Metrics computed at end of each day using R(S, t=days_since_review).

| Metric | Formula | Description |
|--------|---------|-------------|
| `mean_R_today_active` | avgᵢ(Rᵢ(t)) | Mean retrievability now |
| `p10_R_today_active` | p10(Rᵢ(t)) | Tail health (10th percentile) |
| `at_risk_ratio_0_8` | #{Rᵢ < 0.80} / active | Items at-risk (strict) |
| `at_risk_ratio_0_9` | #{Rᵢ < 0.90} / active | Items at-risk (weak) |
| `p90_due_age_at_risk_0_9` | p90(due_age | R < 0.9) | Backlog among weak items |

---

## Capacity Metrics

| Metric | Description |
|--------|-------------|
| `total_active` | Items in working set |
| `effective_max_ws` | Working set cap |
| `reviews_per_active` | session_len / active |

---

## Usage Rules

### Reports MUST show:
1. **Horizon**: `mean_retrievability_horizon` (Coverage%)
2. **Today**: `mean_R_today_active` OR `at_risk_ratio_0_9`

### Never print alone:
- ❌ "Coverage%" without mean_R_today
- ❌ "p90_due_age" without at_risk_ratio

### Example format:
```
| Config | Coverage% | R_today | at_risk_0.9 | intro% |
|--------|-----------|---------|-------------|--------|
| iqrah  |   50.9    |  0.961  |    0.02     |  57.6  |
```
