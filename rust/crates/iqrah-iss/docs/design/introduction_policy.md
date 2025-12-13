# Introduction Policy (ISS v2.9/M2.4)

## Overview

The introduction policy controls how new items are introduced to a learner's working set. It implements a 4-stage clamp pipeline to balance learning progress with cognitive load management.

## Key Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| `intro_min_per_day` | `0` | **Floor**: minimum items introduced when gate blocks. Set `>0` for "never stall" semantics on large goals. |
| `max_working_set` | `50` | Maximum active items before hard stop |
| `cluster_stability_threshold` | `0.40` | Energy threshold for gate (expand if above) |
| `cluster_gate_hysteresis` | `0.01` | Dead zone to prevent gate flapping |
| `cluster_expansion_batch_size` | `3` | Items per batch when expanding |

## Semantics

### `intro_min_per_day = 0` (default)
- No forced progress
- Gate can fully consolidate
- Use when: fine-grained control is desired, or experimenting

### `intro_min_per_day > 0` (e.g., 3)
- Guarantees minimum progress even when gate blocks
- Use for: large from-scratch goals where "never stall" is required
- **Cannot exceed working-set capacity** (hard stop preserved)

## Clamp Order

```
Stage 0: allowance_raw = cluster_expansion_batch_size
Stage 1: allowance_after_capacity = capacity_throttle(raw)
Stage 2: allowance_after_workingset = min(stage1, remaining_workingset) [HARD]
Stage 3: allowance_after_gate = if expand_mode { stage2 } else { 0 }
Stage 4: allowance_final = max(stage3, intro_min_per_day)
         [floor cannot exceed stage2]
```

## Invariants

1. **Working-set full = HARD STOP** - floor cannot override max_working_set
2. **Floor respects capacity** - floor clamped to remaining working-set slots
3. **Hysteresis prevents flapping** - gate only flips at threshold ± hysteresis

## Configuration Guidelines

| Scenario | Recommended `intro_min_per_day` |
|----------|--------------------------------|
| Small goal (Fatiha) | 0-1 |
| Medium goal (1 Juz) | 3 |
| Large from-scratch (Juz 30) | 3-5 |
| Research/debugging | 0 |

## Derived max_working_set (M2.5)

When `max_working_set_ratio_of_goal` is set:
```
effective_max_working_set = ceil(goal_count * ratio), clamped to [1, goal_count]
```

When `None` (default):
```
effective_max_working_set = max_working_set (raw parameter)
```

### Example
- Goal: 564 items (Juz 30)
- `max_working_set_ratio_of_goal: 1.0`
- `effective_max_working_set = 564` (full goal, no ceiling constraint)

This allows fair comparison between schedulers by eliminating artificial ceiling constraints for large goals.

## Backlog-Aware Working Set + Floor (M2.6)

### New Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| `target_reviews_per_active` | None | Target reviews/item/day. If set, caps working set to match review budget. Recommended: 0.08 (dedicated), 0.05 (casual). |
| `max_p90_due_age_days` | None | Threshold for "severe backlog". If p90_due_age exceeds this, intro_floor is disabled. Recommended: 45 (dedicated), 90 (casual). |

### Budgeted Working Set

When `target_reviews_per_active` is set:
```
max_ws_budget = floor(session_size / target_reviews_per_active)
effective_max_ws = min(effective_max_ws_from_ratio, max_ws_budget)
```

This ensures the working set only grows as fast as review capacity can support.

### Backlog-Aware Floor

When `max_p90_due_age_days` is set:
```
if p90_due_age_days > max_p90_due_age_days:
    intro_floor_effective = 0  # Disable floor
else:
    intro_floor_effective = intro_min_per_day
```

This prevents introduction when backlog is already severe, avoiding the death spiral.

### Example (180d Dedicated)

Configuration:
- `session_size = 20`
- `target_reviews_per_active = 0.08`
- `max_p90_due_age_days = 45`

Behavior:
- `max_ws_budget = 20 / 0.08 = 250 items`
- Day 30: p90_due_age=27 < 45 → floor active
- Day 61: p90_due_age=55 > 45 → **floor disabled**, no forced intros
- Working set caps at ~166 items (matching actual review budget)
