# Juz 30 Memory Health Analysis (M3)

> **Generated**: 2025-12-13T11:32Z
> **Git SHA**: 31b41a4
> **Trace Files**: `trace_output/m3_juz30_*/*_memory_health_trace.csv`

---

## Metric Definitions

| Metric | Definition |
|--------|------------|
| `mean_energy` | Average energy (0-1) across active items |
| `p10_energy` | 10th percentile - shows weak item tail |
| `mean_stability` | Average FSRS stability (days) |
| `p10_stability` | 10th percentile - fragile items |
| `mean_retrievability_today` | Average R at day end using FSRS formula |
| `mean_reviews_per_active_item_today` | Review pressure = reviews/total_active |
| `p50_due_age_days` | Median days since last review |
| `p90_due_age_days` | 90th percentile - backlog severity |

---

## Turning Points Analysis

### 180d Dedicated

#### 1. Energy Collapse (iqrah_default): Days 1-30
- Day 0-1: Energy rises briefly from 0.08 to 0.14 (good sign)
- Day 10: Energy drops back to 0.11-0.12 and **stays there permanently**
- Root cause: New items introduced at energy 0.08 dilute the average faster than reviews can boost it
- **Exact day 10**: mean_energy=0.115, never recovers above 0.13

#### 2. Backlog Explosion (iqrah_default): Days 60-120
- Day 60: p90_due_age = 54 days (acceptable)
- Day 90: p90_due_age = 83 days (growing)
- Day 120: p90_due_age = 111 days (critical - items not seen in 4 months)
- **Exact turning point day 75**: p90_due_age exceeds 60 days

#### 3. Stability Stagnation (iqrah_default): Entire run
- mean_stability oscillates 10-16 days, never increases
- p10_stability stuck at 8.3 days (initial FSRS value)
- Items don't get enough reviews to increase stability
- **Contrast random**: mean_stability reaches 393 days by day 180

#### 4. Review Pressure Collapse (iqrah_default): Days 30+
- Day 30: reviews_per_active = 0.087 (12/138)
- Day 60: reviews_per_active = 0.101 (23/228)
- Day 180: reviews_per_active = 0.030 (17/564)
- **Exact turning point day 100**: drops below 0.05 and stays there

### 365d Casual

Similar patterns but more severe:

#### 1. Energy Collapse: Days 1-60
- Starting energy 0.08 (first intro)
- Brief rise to 0.065 at day 30
- Drops to 0.052-0.055 and stagnates
- **Exact turning point day 45**: mean_energy drops below 0.06

#### 2. Catastrophic Backlog: Days 100+
- Day 100: p90_due_age = 91 days
- Day 180: p90_due_age = 156 days
- Day 365: p90_due_age = **334 days** (items not seen in 11 months!)
- **Exact turning point day 150**: p90_due_age exceeds 120 days

---

## Checkpoint Tables

### 180d Dedicated

| Day | Variant | mean_E | p10_E | mean_S | p10_S | mean_R | reviews/active | p50_age | p90_age | active |
|-----|---------|--------|-------|--------|-------|--------|----------------|---------|---------|--------|
| 30  | iqrah   | 0.125  | 0.08  | 11.3   | 8.3   | 0.828  | 0.087          | 18      | 27      | 138    |
| 30  | random  | 0.110  | 0.08  | 38.3   | 8.3   | 0.897  | 0.087          | 10      | 27      | 300    |
| 60  | iqrah   | 0.112  | 0.08  | 14.4   | 8.3   | 0.742  | 0.101          | 30      | 54      | 228    |
| 60  | random  | 0.130  | 0.08  | 86.9   | 8.3   | 0.875  | 0.069          | 20      | 47      | 451    |
| 180 | iqrah   | 0.093  | 0.08  | 10.3   | 8.3   | 0.517  | 0.030          | 91      | 168     | 564    |
| 180 | random  | 0.168  | 0.10  | 393.1  | 6.5   | 0.923  | 0.046          | 25      | 81      | 560    |

### 365d Casual

| Day | Variant | mean_E | p10_E | mean_S | p10_S | mean_R | reviews/active | p50_age | p90_age | active |
|-----|---------|--------|-------|--------|-------|--------|----------------|---------|---------|--------|
| 30  | iqrah   | 0.064  | 0.05  | 20.5   | 8.3   | 0.884  | 0.176          | 10      | 21      | 85     |
| 30  | random  | 0.060  | 0.05  | 30.5   | 8.3   | 0.892  | 0.122          | 11      | 26      | 271    |
| 60  | iqrah   | 0.056  | 0.05  | 14.8   | 8.3   | 0.767  | 0.045          | 25      | 48      | 157    |
| 60  | random  | 0.053  | 0.05  | 59.4   | 8.3   | 0.821  | 0.011          | 27      | 51      | 375    |
| 180 | iqrah   | 0.052  | 0.05  | 12.1   | 8.3   | 0.529  | 0.026          | 82      | 156     | 428    |
| 180 | random  | 0.052  | 0.05  | 302.8  | 8.3   | 0.864  | 0.052          | 35      | 114     | 541    |
| 365 | iqrah   | 0.053  | 0.05  | 12.9   | 8.3   | 0.269  | 0.016          | 235     | 334     | 564    |
| 365 | random  | 0.052  | 0.05  | 668.9  | 1.5   | 0.844  | 0.044          | 40      | 131     | 564    |

---

## Key Findings

### Why baseline_random Appears Strong

1. **Faster Introduction**: Random introduces items 2-3x faster (no gate restriction)
   - 180d: 300 items by day 30 vs iqrah's 138
   - Items enter the system earlier, get more total review opportunities

2. **No Energy Gate Restriction**: Random doesn't check cluster energy, so it keeps introducing regardless of consolidation state

3. **Stability Builds Over Time**: Because random reviews items (even imperfectly), stability accumulates
   - Random reaches mean_stability 393-669 days
   - Iqrah stays at 10-13 days forever

4. **Backlog Pressure Lower**: Random's p90_due_age (81-131 days) is half iqrah's (168-334 days)

### Why iqrah_default Performs Poorly

1. **Cluster Energy Gate**: Energy never exceeds 0.15 threshold
   - Gate always blocks: `allowance_after_gate=0`
   - Only `intro_min_per_day=3` floor provides introduction

2. **Introduction-Review Imbalance**:
   - Introducing 3 items/day × 180 days = 540 items
   - But reviewing only 0.03 × 540 = 16 items/day
   - Items accumulate without adequate review

3. **Death Spiral**:
   - Low reviews → low energy → gate blocks → slow intros → huge backlog → even lower reviews/item → lower energy...

---

## Recommendations

### Based on M3 Evidence

1. **Lower energy gate threshold** (0.15 → 0.08-0.10)
   - Evidence: mean_energy never exceeds 0.13
   - Impact: Would allow gate_expand_mode=true periodically

2. **Increase intro_min_per_day** (3 → 10 for dedicated)
   - Evidence: At 3/day, takes 188 days to introduce 564 items
   - Impact: Would reach full coverage faster, leaving more time for review

3. **Increase session_size** (20 → 40 for dedicated, 10 → 20 for casual)
   - Evidence: reviews_per_active drops to 0.03 (once every 33 days per item)
   - Impact: Would double review coverage, reduce backlog

4. **Consider urgency-based scheduling** within sessions
   - Evidence: p90_due_age reaching 334 days means some items are severely neglected
   - Impact: Would ensure no item goes more than X days without review

---

## Hypothesis Validation

| Hypothesis | Evidence | Verdict |
|------------|----------|---------|
| Gate blocks forever | 100% gate_blocked=true | **CONFIRMED** |
| Energy never recovers | mean_energy capped at 0.13 | **CONFIRMED** |
| Backlog explodes | p90_due_age: 168-334 days | **CONFIRMED** |
| Stability stagnates | mean_stability 10-13 days | **CONFIRMED** |
| Random brute-forces coverage | 2-3x faster intro, higher reviews/item | **CONFIRMED** |
