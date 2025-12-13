# Exercise Scoring v2.9 Design

## Overview

v2.9 fixes the exercise saturation problem by separating **availability** (was item introduced?) from **recall quality** (how well is it remembered?).

## New Terminology

| Term | Definition |
|------|------------|
| **Sampled** | Total items selected for testing by exercise |
| **Attempted** | Items that have a `MemoryState` (were introduced) |
| **Unavailable** | Sampled items with no `MemoryState` (never introduced) |
| **Availability Ratio** | `attempted / sampled` (0.0 to 1.0) |

## Revised `ExerciseResult` Structure

```rust
pub struct ExerciseResult {
    // Existing fields (preserved for backward compat)
    pub score: f64,              // Composite score [0, 1]
    pub grade: ReviewGrade,      // Overall grade mapping
    pub details: ExerciseDetails,
    pub metadata: ExerciseMetadata,
    pub summary: String,

    // NEW v2.9: Availability tracking
    pub sampled: usize,          // Total items sampled
    pub attempted: usize,        // Items with MemoryState
    pub unavailable: usize,      // Items without MemoryState
    pub availability_ratio: f64, // attempted / sampled

    // NEW v2.9: Attempted-only metrics (None if attempted == 0)
    pub mean_trials_attempted: Option<f64>,
    pub grade_attempted: Option<ReviewGrade>,
}
```

## Computation Rules

### 1. Availability Ratio

```rust
availability_ratio = attempted as f64 / sampled.max(1) as f64;
// Ranges from 0.0 (nothing learned) to 1.0 (all sampled items learned)
```

### 2. Mean Trials (Attempted Only)

```rust
mean_trials_attempted = if attempted > 0 {
    Some(attempted_trials_sum / attempted as f64)
} else {
    None  // Cannot compute - no items to test
};
```

### 3. Grade (Attempted Only)

```rust
grade_attempted = mean_trials_attempted.map(|t| trials_to_grade(t));
// None if no items attempted
```

### 4. Overall Score (Composite)

Two options for composite scoring:

**Option A: Weighted Composite (recommended)**
```rust
// Blend availability with attempted-grade score
let recall_component = mean_trials_attempted.map(|t| grade_to_score(trials_to_grade(t))).unwrap_or(0.0);
let availability_component = availability_ratio;

// Weight: 70% recall quality, 30% availability
score = 0.7 * recall_component + 0.3 * availability_component;
```

**Option B: Keep Separate (for analysis)**
```rust
// Don't blend - report both separately
// Overall grade = grade_attempted (or Again if None)
// Separately report availability_ratio for dashboard
```

## Grade Mapping Rules

### Recall Quality Grade (attempted only)

| Avg Trials | Grade | Score |
|------------|-------|-------|
| ≤1.5 | Easy | 1.0 |
| ≤3.0 | Good | 0.75 |
| ≤6.0 | Hard | 0.5 |
| >6.0 | Again | 0.0 |
| N/A (attempted=0) | None | N/A |

### Availability Grade (optional, for dashboard)

| Availability Ratio | Interpretation |
|-------------------|----------------|
| ≥0.90 | Excellent coverage |
| ≥0.70 | Good coverage |
| ≥0.50 | Moderate coverage |
| <0.50 | Low coverage (many unintroduced) |

**Recommendation**: Do NOT grade availability - just report the ratio. Users can interpret based on goal size and timeline.

## Report Format

### Before (v2.8)
```
Recitation: 564 items, avg 18.57 trials/item, 536 failures, grade: Again
```

### After (v2.9)
```
Recitation: 564 sampled, 166 attempted (29.4% availability)
  - Attempted: avg 2.45 trials/item, 12 failures, grade: Good
  - Unavailable: 398 items (not yet introduced)
```

## Implementation Location

| File | Change |
|------|--------|
| `exercises/mod.rs` | Add new fields to `ExerciseResult` |
| `exercises/memory.rs` | Track attempted vs unavailable in evaluation loop |
| `exercises/helpers.rs` | Add `filter_introduced_only()` helper |
| `analyzer.rs` | Update report formatting |
| `events/types.rs` | Add new fields to `ExerciseEvaluation` event |

## Backward Compatibility

- Existing configs continue to work unchanged
- `score` field still computed (using Option B initially)
- `grade` field still present
- New fields are additions, not replacements
- Old event consumers ignore new fields

## Testing Requirements

1. **Zero availability**: All sampled items unavailable
   - `attempted = 0`, `availability_ratio = 0`
   - `mean_trials_attempted = None`, `grade_attempted = None`
   - NO division by zero

2. **Full availability**: All sampled items have MemoryState
   - Metrics unchanged from v2.8

3. **Mixed availability**: Some available, some not
   - `mean_trials_attempted` computed on attempted-only
   - `availability_ratio` reflects partial coverage

4. **Backward compat**: Old configs parse and run correctly
