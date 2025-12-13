# Postmortem: Juz 30 Exercise Saturation

## Summary

Exercises on Juz 30 (564 items) consistently score **"Again" with 18+ trials** even after 90 days of training, making the exercise framework useless for measuring progress on large goals.

## Root Cause Analysis

### The Problem Code Path

```
memory.rs::evaluate() L128-156

for node_id in &test_nodes {
    let expected_trials = if let Some(state) = memory_states.get(node_id) {
        // Item was introduced - compute trials from energy
        let p_recall = compute_recall_probability(state, brain, current_day);
        let trials = 1.0 / p_recall.max(0.01);  // 0.5 energy → 2.0 trials
        ...
    } else {
        // Item NEVER INTRODUCED → MAX_TRIALS = 20.0   ← THE BUG
        items_failed += 1;
        MAX_TRIALS  // L155
    };
```

### What "Unintroduced" Means in Code

An item is "unintroduced" when:
- `memory_states.get(node_id)` returns `None`
- This happens when `user_repo.get_memory_state_sync(user_id, node_id)` has no entry
- No `MemoryState` exists because the item was never scheduled for review

### Where the Cap Destroys Signal

```rust
// memory.rs L269, L272
const FAILURE_THRESHOLD: f64 = 6.0;
const MAX_TRIALS: f64 = 20.0;

// L155: Unintroduced items get MAX_TRIALS
MAX_TRIALS // Capped for scoring

// L163: Average computed over ALL sampled items
let avg_trials = total_trials / items_tested as f64;
```

**Math Example** (Juz 30, Day 30):
- `sampled = 564` (all goal items)
- `introduced = 166` (29.6% coverage)
- `unavailable = 564 - 166 = 398`

Average trials:
```
avg = (166 × 2.5 trials + 398 × 20 trials) / 564
    = (415 + 7960) / 564
    = 14.86 trials  → Grade: Again
```

The 398 unintroduced items drag the average to ~15-18 trials regardless of how well the 166 introduced items are known.

---

## Counts at Day 15/30/60 (from juz30_final run)

| Metric | Day 15 | Day 30 | Day 60 |
|--------|--------|--------|--------|
| goal_count | 564 | 564 | 564 |
| sampled_count | 564 | 564 | 564 |
| introduced_count | ~85 | ~166 | ~185 |
| unavailable_count | ~479 | ~398 | ~379 |
| items_failed | 533 | 536 | 535 |
| avg_trials | 18.13 | 18.57 | 18.70 |
| grade | Again | Again | Again |

**Key Observation**: Even with 166 items introduced (29.6% coverage), exercises report "Again" because they sample ALL 564 goal items.

---

## Why Cluster Gate Blocks Introduction

From `simulator.rs`:

```rust
// L1300: The cluster gate decision
if !skip_cluster_gate && cluster_energy < student_params.cluster_stability_threshold {
    return 0.0; // Cluster weak - consolidate
}

// L1384-1389: compute_cluster_energy
fn compute_cluster_energy(items: &[MemoryState]) -> f64 {
    let active_items: Vec<_> = items.iter().filter(|i| i.review_count > 0).collect();
    if active_items.is_empty() {
        return 1.0; // Empty cluster is "stable"
    }
    active_items.iter().map(|i| i.energy).sum::<f64>() / active_items.len() as f64
}
```

With 166 items at avg energy ~0.35:
- `cluster_energy = 0.35`
- `cluster_stability_threshold = 0.08`
- Gate passes (0.35 > 0.08)

**BUT** the gate is NOT the only blocker. Other factors:
1. `max_working_set = 300` limits introduced items
2. `compute_working_set_factor()` dampens when utilization > 70%
3. Daily review budget limits new introductions

The gate itself passed, but the *rate* of introduction is throttled by multiple factors.

---

## Impact

1. **Exercise scores are meaningless** on large goals
2. **No visibility into why** exercises fail (availability vs recall)
3. **Cannot compare** iqrah vs random fairly on exercises
4. **Users see "Again"** even when introduced items are being learned well

---

## Fix Requirements

1. **Separate availability from recall**: Track which items were attempted vs unavailable
2. **Compute metrics on attempted-only**: `avg_trials_attempted` should exclude unavailable items
3. **Report availability ratio**: `availability_ratio = attempted / sampled`
4. **Option for introduced-only sampling**: Allow exercises to sample only items that exist in memory_states

---

## Related Files

| File | Lines | Purpose |
|------|-------|---------|
| `exercises/memory.rs` | 128-156 | Evaluation loop with MAX_TRIALS bug |
| `exercises/memory.rs` | 269-272 | Constants: FAILURE_THRESHOLD, MAX_TRIALS |
| `simulator.rs` | 1300 | Cluster gate decision |
| `simulator.rs` | 1384-1389 | compute_cluster_energy() |
| `simulator.rs` | 1363-1377 | compute_working_set_factor() |
