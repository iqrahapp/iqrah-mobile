# Juz 30 Milestone Results (M2.5 through M2.7)

> **Generated**: 2025-12-13T12:12Z
> **Git SHA**: current (M2.7)
> **Seeds**: 42 (single-seed)

---

## M2.7: Overdue Fairness Fix (CRITICAL BUG)

### Bug Description
Due candidates were selected with `.take(N)` in arbitrary order (goal_items order).
This caused **most overdue items to never be selected**, as older items remained at the
end of the list while newer items consumed all session slots.

### Fix
```rust
// M2.7: Sort due candidates by due_age DESC (most overdue first)
due_candidates.sort_by(|a, b| {
    let due_age_a = now_ts - a.next_due_ts;
    let due_age_b = now_ts - b.next_due_ts;
    due_age_b.cmp(&due_age_a)
});
```

### Before/After Comparison (180d Dedicated)

| Metric | M2.6 | M2.7 | Delta |
|--------|------|------|-------|
| **Coverage%** | 8.3% | **50.9%** | +513%! |
| **p90_due_age** | 172.5 | **125.6** | -27% ✅ |
| active items | 166 | 325 | +96% |
| mean_stability | 14.4 | 263 | +1725% |
| mean_R | 0.40 | 0.96 | +140% |

### Before/After Comparison (365d Casual)

| Metric | M2.6 | M2.7 | Delta |
|--------|------|------|-------|
| **Coverage%** | 9.2% | **40.7%** | +342%! |
| **p90_due_age** | 345 | **266** | -23% ✅ |
| active items | 251 | 266 | +6% |
| mean_stability | 20.5 | 336.4 | +1540% |

---

## M2.6: Backlog-Aware Working Set + Floor

Added budgeted working set (target_reviews_per_active) and backlog-aware floor (disabled when p90_due_age exceeds threshold).
This correctly throttled introduction but exposed the M2.7 bug.

---

## Trace Files

| Run | Directory |
|-----|-----------|
| M2.6 180d | `trace_output/m2_6_budgeted_180d_seed42/` |
| M2.6 365d | `trace_output/m2_6_budgeted_365d_seed42/` |
| M2.7 180d | `trace_output/m2_7_fairness_180d_seed42/` |
| M2.7 365d | `trace_output/m2_7_fairness_365d_seed42/` |
