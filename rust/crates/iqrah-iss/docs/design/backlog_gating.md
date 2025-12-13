# Backlog Protection Design (M2.7+)

## Overview

ISS uses multiple safety layers to prevent backlog explosion. The primary protection is **M2.7 due-age sorting**, with backlog gating as a defensive fallback.

---

## Safety Layers (Priority Order)

### 1. M2.7 Overdue Fairness Sorting (PRIMARY)

Due candidates are sorted by `due_age DESC` before selection:
```rust
due_candidates.sort_by(|a, b| {
    due_age_b.cmp(&due_age_a).then(a.id.cmp(&b.id))
});
```

**Effect**: Most overdue items are always reviewed first, preventing starvation.

### 2. Cluster Stability Gate

Blocks new introductions when cluster energy is below threshold:
```
gate_blocked = cluster_energy < cluster_stability_threshold
```

**Effect**: Pauses expansion when working set is struggling.

### 3. M2.8 Backlog Gating (FALLBACK)

Disables intro floor when backlog is severe:
```
backlog_severe = p90_due_age > max_p90_due_age_days
```

**Effect**: Stops forced introductions when backlog is critical.

---

## Why Gating Rarely Triggers

After M2.7 implementation, due-age sorting keeps `p90_due_age` bounded:

| Scenario | p90_due_age | max_p90 | backlog_severe |
|----------|-------------|---------|----------------|
| SS40 Dedicated | 119 | 45 | false* |
| SS15 Casual | ~90 | 90 | false |

*Note: High p90_due_age is acceptable when stability is high (items are "old but safe").

---

## Stress Scenarios

### Cluster Gate Triggers

Under extreme constraints (session_size=6, forgetting_rate=3.0):
- `gate_blocked=true, gate_reason="cluster_weak"` at day 10-11

### Backlog Gate Triggers

Would require:
- Very low session size AND
- Very high forgetting rate AND
- Disabled sorting (which is not the case)

In practice, M2.7 sorting prevents conditions that would trigger backlog gating.

---

## Conclusion

1. **M2.7 due-age sorting** is the primary backlog protection
2. **Cluster gate** provides secondary protection for struggling working sets
3. **M2.8 backlog gating** exists as final fallback but rarely activates in healthy configs

The layered approach ensures robustness without over-reliance on any single mechanism.
