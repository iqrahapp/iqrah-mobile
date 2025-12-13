# Cluster Gate Diagnostics Design

## Overview

The cluster gate is designed to prevent "death spirals" where too many items are introduced before existing ones are stable. However, on large from-scratch goals like Juz 30, it can over-restrict introduction.

## Where the Gate Decision is Made

### Primary Location: `simulator.rs` L1300

```rust
// If cluster energy is below threshold and not in bootstrap phase, consolidate
if !skip_cluster_gate && cluster_energy < student_params.cluster_stability_threshold {
    return 0.0; // Cluster weak - consolidate
}
```

### Bootstrap Exception: L1294-1297

```rust
// Bootstrap exception: Skip cluster gate when fewer than 10 active items
const BOOTSTRAP_THRESHOLD: usize = 10;
let skip_cluster_gate = active_count < BOOTSTRAP_THRESHOLD;
```

## Signals Used for Gating

| Signal | Computed At | Formula |
|--------|-------------|---------|
| `cluster_energy` | L1292 | `compute_cluster_energy(items)` |
| `cluster_stability_threshold` | config | `brain.params.cluster_stability_threshold` |
| `active_count` | L1286 | Items with `review_count > 0` |
| `skip_cluster_gate` | L1297 | `active_count < 10` |

### `compute_cluster_energy()` - L1384-1389

```rust
fn compute_cluster_energy(items: &[MemoryState]) -> f64 {
    let active_items: Vec<_> = items.iter().filter(|i| i.review_count > 0).collect();
    if active_items.is_empty() {
        return 1.0; // Empty cluster is "stable" (allows bootstrap)
    }
    active_items.iter().map(|i| i.energy).sum::<f64>() / active_items.len() as f64
}
```

**Key insight**: Returns **mean energy** of all active items. If many items have low energy, gate blocks.

## Why Gate Blocks ~74% of Days on Juz 30

### Scenario Analysis

With `cluster_stability_threshold = 0.08` and 166 introduced items:

**Expected behavior**: Gate should NOT block when avg energy > 0.08

**Actual behavior**: From logs, gate is NOT the primary blocker!

The real limiters are:

1. **Working Set Factor** (L1363-1377):
   - `max_working_set = 300`
   - When `active_count > 210` (70% of 300), introduction rate drops linearly
   - At 166 items: some dampening occurring

2. **Capacity Utilization** (L1316):
   - `capacity_used = maintenance_burden / session_capacity`
   - If heavily used, introduction slows

3. **Introduction Rate Dampening** (L1347):
   - `rate = raw_intro_rate * damping * session_capacity * working_set_factor`
   - Multiple multiplicative factors can reduce to near-zero

### The Real Problem

Even when gate passes, the **rate** is too low:

```
Day 30: active=166
  - cluster_energy = 0.35 > threshold 0.08 → PASSES
  - working_set_factor = (1.0 - 0.55) / 0.3 = 1.0 → FULL RATE
  - But only ~2 items introduced this day
```

Root cause: `session_capacity` and day budget limits actual introductions even when gate is open.

## Proposed Diagnostic Instrumentation

### Config Flag

```yaml
debug_trace:
  enabled: true
  out_dir: "./trace_output"
```

### CSV Per Simulation Run

Filename: `{out_dir}/{scenario}_{variant}_gate_trace.csv`

| Column | Type | Description |
|--------|------|-------------|
| day | u32 | Simulation day |
| due_reviews | usize | Reviews scheduled as due |
| reviews_done | usize | Actual reviews completed |
| new_introduced | usize | New items introduced today |
| total_introduced | usize | Cumulative introduced |
| working_set_size | usize | Active items count |
| cluster_energy | f64 | Mean energy of active items |
| gate_blocked | bool | Did gate block? |
| gate_reason | enum | Why blocked: `ClusterWeak`, `WorkingSetFull`, `CapacityExceeded`, `None` |
| threshold | f64 | Current threshold value |
| working_set_factor | f64 | Rate multiplier from working set |
| capacity_used | f64 | Fraction of session capacity used |

### Markdown Summary

Generated at end: `{out_dir}/{scenario}_{variant}_gate_summary.md`

```markdown
# Gate Diagnostics: {scenario}_{variant}

## Overview
- Days simulated: 90
- Days gate blocked: 67 (74%)
- Average introductions/day: 1.84

## Gate Signal Statistics
| Signal | Min | Mean | Max |
|--------|-----|------|-----|
| cluster_energy | 0.15 | 0.35 | 0.52 |
| working_set_factor | 0.45 | 0.78 | 1.00 |
| capacity_used | 0.62 | 0.89 | 1.10 |

## Block Reasons
- ClusterWeak: 12 days (13%)
- WorkingSetFull: 8 days (9%)
- CapacityExceeded: 47 days (52%)
- None (allowed): 23 days (26%)
```

### Implementation Notes

```rust
pub enum GateReason {
    None,           // Introduction allowed
    ClusterWeak,    // cluster_energy < threshold
    WorkingSetFull, // active_count >= max_working_set
    CapacityExceeded, // capacity_used > 1.1
    RateTooLow,     // Computed rate < 1.0 (multiple factors)
}

pub struct GateTraceRow {
    day: u32,
    due_reviews: usize,
    reviews_done: usize,
    new_introduced: usize,
    total_introduced: usize,
    working_set_size: usize,
    cluster_energy: f64,
    gate_blocked: bool,
    gate_reason: GateReason,
    threshold: f64,
    working_set_factor: f64,
    capacity_used: f64,
}
```

### Test Requirement

```rust
#[test]
fn test_trace_disabled_writes_no_files() {
    // Run simulation with debug_trace.enabled = false
    // Assert no files written to out_dir
}
```

## Files to Modify

| File | Change |
|------|--------|
| `config.rs` | Add `DebugTraceConfig` struct |
| `simulator.rs` | Emit trace rows during simulation |
| `simulator.rs` | Write CSV and summary at end |
| `tests/trace_test.rs` | Verify disabled trace writes nothing |
