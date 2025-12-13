# Backlog Metrics Design (M2.8)

## Overview
M2.8 introduces **retrievability-aware** backlog severity metrics to replace raw due_age-based gating.

## Problem Statement
After M2.7 (overdue sorting fix), mean_stability jumped to ~260 days.
With high stability, items can be 120+ days old yet still have high retrievability (R > 0.95).
Raw `p90_due_age_days` is no longer a good "severe backlog" signal.

## At-Risk Definition

An item is **at-risk** if its today retrievability is below threshold:

```
at_risk = R_today < R_RISK_THRESHOLD (default: 0.80)
```

Where:
```
R_today = (1 + t / (9 * S))^-1   (FSRS formula)
t = days since last review
S = stability
```

## New Metrics

| Metric | Description |
|--------|-------------|
| `at_risk_count` | Items with R_today < 0.80 |
| `at_risk_ratio` | at_risk_count / total_active |
| `p10_R_today` | 10th percentile of R_today (tail health) |
| `p90_due_age_at_risk` | p90 due_age only among at-risk items |

## Backlog Gating Update

### Before (M2.6)
```
backlog_severe = p90_due_age_days > max_p90_due_age_days
```

### After (M2.8)
```
backlog_severe = at_risk_ratio > max_at_risk_ratio
  OR
p90_due_age_at_risk > max_p90_due_age_days
```

Recommended thresholds:
- `max_at_risk_ratio`: 0.25 (dedicated), 0.35 (casual)

## Trace Output

New columns in `memory_health_trace.csv`:
- `at_risk_count`
- `at_risk_ratio`
- `p10_R_today`
- `p90_due_age_at_risk`

## Example (180d Dedicated, M2.8)

Day 179:
- total_active: 325
- mean_R_today: 0.9613
- **at_risk_count: 0**
- **at_risk_ratio: 0.00** (✅ no items at risk)
- p10_R_today: 0.9372

The M2.7 sorting fix was so effective that **no items are at-risk** - all have R > 0.80.
