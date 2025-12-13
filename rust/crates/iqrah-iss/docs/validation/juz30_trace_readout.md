# ISS v2.9: Trace Readout - Why Override Isn't Achieving Coverage

## Summary

**Scenario**: Juz 30 (564 items), 60 days, v2.9 floor mechanism enabled

| Variant | Coverage | Composite |
|---------|----------|-----------|
| iqrah_default | 9.8% | 0.305 |
| baseline_random | 84.4% | 0.614 |

## Root Cause Analysis

### The Math Problem

For 60% coverage on 564 items in 60 days:
- **Items needed**: 564 × 60% = **339 items minimum**
- **Days available**: 60
- **Required rate**: 339 / 60 = **5.65 items/day minimum**

### Current Configuration:

```yaml
intro_min_per_day: 3           # Floor: only 3/day
intro_bootstrap_until_active: 100
max_new_items_per_day: 10      # Cap at 10
```

### Analysis

1. **Days 1-10 (Bootstrap Phase)**:
   - `active_count < 100` → skip cluster gate → can intro freely
   - With `intro_min_per_day: 3` and `max_new_items_per_day: 10`
   - Best case: 10 items/day × 10 days = **100 items**

2. **Days 10-60 (Post-Bootstrap)**:
   - Cluster gate kicks in
   - If `cluster_energy < 0.15`, gate blocks
   - Override checks: `capacity_used < 0.6` → allow max_new_items_per_day
   - Floor: `intro_min_per_day: 3` → guarantees 3/day

3. **Low-end projection**:
   - Days 1-10: ~100 items (best bootstrap)
   - Days 11-60: 50 days × 3 items/day = 150 items
   - **Total: ~250 items** = 250/564 = **44% introduced**

4. **Actual result matches**: 9.8% coverage means even fewer were *retained*, not just introduced.

### Why Floor Isn't Enough

- `intro_min_per_day: 3` is **too low** for 564 items in 60 days
- Need at least `intro_min_per_day: 6` to hit 60% coverage target
- Or extend `intro_bootstrap_until_active: 340` to bypass gate longer

---

## Recommended Configuration Fix

```yaml
# For 60% coverage on Juz 30 in 60 days
intro_min_per_day: 7           # Ensures 420 items in 60 days
intro_bootstrap_until_active: 200  # Skip gate until 200 active
max_new_items_per_day: 15      # Allow faster expansion
intro_override_enabled: true
intro_slack_ratio: 0.8         # Higher slack ratio
```

---

## v2.9 Code Status

### What Was Implemented ✓

1. **intro_bootstrap_until_active** - Skips cluster gate until N active items
2. **intro_min_per_day** - Floor on daily introductions
3. **max_new_items_per_day** - Cap on daily introductions
4. **Capacity-based override** - Uses `capacity_used` instead of `due_reviews`

### What Needs Tuning

- Current defaults are too conservative for large goals
- Need per-goal-size calibration

---

## Next Steps

1. **Increase floor params** for the v2.9 candidate scenario
2. **Run 180-day test** with higher intro_min_per_day (7+)
3. **Verify trajectory** - are introduced items being retained?
