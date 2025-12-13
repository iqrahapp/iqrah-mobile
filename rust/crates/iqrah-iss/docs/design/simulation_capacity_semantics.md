# Simulation Capacity Semantics

**Created**: 2025-12-12
**Purpose**: Clarify trace column semantics after M1.3 showed apparent invariant violations.

---

## Findings Summary

> **The trace contradictions are SEMANTICS issues, not bugs.**
> Column names are misleading but simulator behavior is correct according to design.

---

## Field Definitions

### `session_capacity`

**Definition**: A soft budget used only for **capacity utilization ratio computation**, NOT a hard cap on reviews.

- **Value**: `brain.params.session_capacity` (default: 15.0)
- **Usage**:
  - `capacity_util = maintenance_burden / session_capacity`
  - Used by `compute_sustainable_intro_rate()` to throttle introductions
- **NOT USED FOR**: Limiting actual reviews per day

**Code Reference**:
- [brain.rs:157-158](file:///home/shared/ws/iqrah/iqrah-mobile/rust/crates/iqrah-iss/src/brain.rs#L157-L158): Definition
- [simulator.rs:1155](file:///home/shared/ws/iqrah/iqrah-mobile/rust/crates/iqrah-iss/src/simulator.rs#L1155): Usage

```rust
// simulator.rs:1155
let capacity_util = maintenance_burden / brain.params.session_capacity as f32;
```

---

### `reviews_done` (trace column)

**Definition**: The actual number of items reviewed this day = `session_len`.

- **Source**: `session_items.len()` after scheduler generates session
- **Determined by**: `brain.sample_daily_reviews()` → Normal distribution clamped to `[min_reviews_per_day, max_reviews_per_day]`

**Code Reference**:
- [simulator.rs:480](file:///home/shared/ws/iqrah/iqrah-mobile/rust/crates/iqrah-iss/src/simulator.rs#L480): `let session_size = brain.sample_daily_reviews();`
- [simulator.rs:902](file:///home/shared/ws/iqrah/iqrah-mobile/rust/crates/iqrah-iss/src/simulator.rs#L902): `let session_len = session_items.len();`
- [brain.rs:1028-1055](file:///home/shared/ws/iqrah/iqrah-mobile/rust/crates/iqrah-iss/src/brain.rs#L1028-L1055): `sample_daily_reviews()`

**NOT AN INVARIANT**: `reviews_done <= session_capacity`
This is expected to be false. `session_capacity` and `reviews_per_day` are independent parameters.

---

### `new_introduced` (trace column)

**Definition**: Number of items with `review_count == 1` after the day (first-time reviews).

- **Counted from**: Post-day memory state snapshot
- **Source code**: `states.values().filter(|s| s.review_count == 1).count()`

**Code Reference**:
- [simulator.rs:1231](file:///home/shared/ws/iqrah/iqrah-mobile/rust/crates/iqrah-iss/src/simulator.rs#L1231): `let introduced_today = states.values().filter...`

---

### `active_count` (trace column)

**Definition**: Number of items with `review_count > 0`.

- **Scope**: All items in goal that have been reviewed at least once
- **Source**: `states.values().filter(|s| s.review_count > 0).count()`

**Code Reference**:
- [simulator.rs:1228](file:///home/shared/ws/iqrah/iqrah-mobile/rust/crates/iqrah-iss/src/simulator.rs#L1228): Used in trace

---

### `max_working_set`

**Definition**: Soft limit on NEW item introductions per day, NOT a cap on total active items.

- **Usage**: When `active_count >= max_working_set`, `new_items_limit = 0`
- **ONLY affects**: The `new_items_limit` variable used for candidate filtering
- **DOES NOT prevent**: Reviewing already-active items

**Code Reference**:
- [simulator.rs:738-742](file:///home/shared/ws/iqrah/iqrah-mobile/rust/crates/iqrah-iss/src/simulator.rs#L738-L742):

```rust
if active_count >= brain.params.max_working_set {
    0 // At working set limit - consolidate (no NEW items)
} else {
    // Allow new items up to remaining capacity
    let remaining_capacity = brain.params.max_working_set - active_count;
    batch_size.min(remaining_capacity)
}
```

**NOT AN INVARIANT**: `active_count <= max_working_set`
This can exceed because `max_working_set` only gates NEW introductions. Once items are active, reviews continue regardless.

---

## Corrected Trace Column Names

| Original Name | Actual Meaning | Suggested Rename |
|--------------|----------------|------------------|
| `session_capacity` | Soft budget for capacity modeling | `capacity_budget` |
| `reviews_done` | Actual session size (items reviewed) | `actual_reviews` |
| `remaining_capacity` | `capacity_budget - actual_reviews` | `budget_delta` |
| `working_set_size` | `max_working_set` param value | `max_new_items_param` |
| `active_count` | Items with review_count > 0 | `total_active` |

---

## Invariants That SHOULD Hold

| Check | Formula | Expected |
|-------|---------|----------|
| `new_introduced <= new_items_limit` | If cluster gate open | Only if gate not blocked |
| `session_len == reviews_done` | Always | True |
| `active_count >= total_introduced` | Active ⊇ Introduced | True |

## "Invariants" That Do NOT Hold (by design)

| False "Invariant" | Why |
|-------------------|-----|
| `reviews_done <= session_capacity` | `session_capacity` is only for utilization ratio |
| `active_count <= max_working_set` | `max_working_set` only limits NEW introductions |

---

## Conclusion

**Outcome A: Semantics were misleading.**

The trace column names imply hard constraints that don't exist:
- `session_capacity` sounds like a max, but it's a soft budget
- `max_working_set` sounds like a total limit, but only gates new items

**Action**: Rename trace columns in M2.1 to match actual semantics.
