# Session Composition v2.9

**Created**: 2025-12-12
**Purpose**: Document the session composition logic to explain how items are selected each day.

---

## Call Chain

```
simulate_student_internal()
  └── simulate_day()
        ├── brain.sample_daily_reviews() → session_size
        ├── compute cluster_energy, active_count
        ├── compute new_items_limit (from gate logic)
        ├── get_candidates() → filters goal_items
        │     └── filters: cluster membership, almost-due window, prerequisite gate
        ├── generate_session(candidates, session_size)
        │     └── iqrah-core session_generator.rs → picks due + new items
        └── process session items
```

---

## Key Variables

| Variable | Source | Meaning |
|----------|--------|---------|
| `session_size` | `brain.sample_daily_reviews()` | How many items to review today (random within clamp) |
| `new_items_limit` | Cluster gate logic | Max NEW items allowed today |
| `intro_min_per_day` | `StudentParams` | Floor on new items per day |
| `max_new_items_per_day` | `StudentParams` | Ceiling on new items per day |
| `max_working_set` | `StudentParams` | Only gates NEW intros, not total active |

---

## new_items_limit Computation

Located in `simulator.rs` lines 735-748:

```rust
let new_items_limit = if can_expand {
    // Check working set limit first
    if active_count >= brain.params.max_working_set {
        0 // At working set limit - consolidate
    } else {
        // Within limit - allow batch
        let remaining_capacity = brain.params.max_working_set - active_count;
        batch_size.min(remaining_capacity)
    }
} else {
    0 // Cluster not stable - consolidate
};
```

**Key insight**: `new_items_limit` is only computed for filtering candidates.
It is NOT directly used to control session composition.

---

## Where New Items Are (or Aren't) Selected

### Current Flow (PROBLEM)

1. `get_candidates()` filters goal_items to:
   - Items in cluster (if cluster gating enabled)
   - Items that are due/overdue OR within almost_due_window
   - Items not blocked by prerequisite gate

2. `generate_session()` picks from candidates without explicit intro budget:
   - Due items get priority via urgency scoring
   - New items only appear if they pass all filters AND score higher than due items

3. **Result**: If due backlog is large, new items are either:
   - Filtered out by candidate filters (not due, not in window)
   - Ranked below due items and not selected

### M2.1 Fix: Explicit Budgets (TO BE IMPLEMENTED)

Proposed split:
```
session_size = brain.sample_daily_reviews()
intro_budget = clamp(intro_min_per_day, 0..max_new_items_per_day)
due_budget = session_size - intro_budget

1. Select due items: pick up to due_budget due/overdue items
2. Select new items: pick up to min(intro_budget, new_items_limit_today)
3. Spillover: if due < due_budget, leftover capacity goes to new
```

---

## Why iqrah_default Stalls

From M1.3 trace (with OLD misdefined metrics):
- `gate_blocked = false` for 100% of days
- `new_items_limit_today > 0` (cluster not blocking)
- Yet `introduced_today = 0` from day 31

**Root cause hypothesis** (to be verified with corrected trace):

1. Candidates filter excludes new items once working set is full-ish
2. Or: `generate_session()` ranks due items above new items
3. Or: `intro_min_per_day = 5` but no floor enforcement

---

## Session Budget Trace Columns (M2.1)

| Column | Meaning |
|--------|---------|
| `session_size` | Total items to review today |
| `due_budget` | Slots reserved for due items |
| `intro_budget` | Slots reserved for new items |
| `due_selected` | Actual due items selected |
| `new_selected` | Actual new items selected |
| `new_items_limit_today` | Cap from gate logic |

---

## Next Steps

1. Verify with corrected trace (introduced_today = delta)
2. If stall persists, implement explicit budget split
3. Add unit test: "new items are selected when intro_budget > 0 even if due backlog is high"
