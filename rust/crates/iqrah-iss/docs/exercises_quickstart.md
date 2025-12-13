# Exercise Framework Quick Start

## What Are Exercises?

Exercises evaluate specific cognitive skills during simulation:
- **Memory exercises**: Test sequential recall (recitation)
- **Translation exercises**: Test independent item comprehension

## Adding Exercises to Your Scenario

```yaml
exercises:
  # Test continuous recitation every 15 days
  - type: memory
    subtype: continuous_recitation
    frequency_days: 15
    parameters:
      sample_size: 7
      sampling_strategy: sequential

  # Sample random ayahs every 10 days
  - type: memory
    subtype: sample_recitation
    frequency_days: 10
    parameters:
      sample_size: 3
      sampling_strategy: random
```

## Understanding Results

### Trials-Based Scoring

Memory exercises report **expected trials** until successful recall:

| Avg Trials | Grade | Score | Interpretation |
|------------|-------|-------|----------------|
| ≤1.5 | Easy | 1.0 | Instant recall, no hesitation |
| 1.5-3.0 | Good | 0.75 | Some hesitation, eventually succeeds |
| 3.0-6.0 | Hard | 0.50 | Significant struggle, many attempts |
| >6.0 | Again | 0.0 | Cannot recall reliably |

### Example Timeline

```
Day 10:  score=0.50, grade=Hard,  trials=3.19  (struggling)
Day 30:  score=0.75, grade=Good,  trials=2.64  (improving)
Day 50:  score=0.75, grade=Good,  trials=2.24  (confident)
```

## Working Example

```bash
cargo run --release -p iqrah-iss -- compare \
  --preset fatiha_from_scratch \
  -V iqrah_default \
  -n 1 --days 60 \
  --emit-events ./results
```

See [fatiha_from_scratch.yaml](../configs/scenarios/fatiha_from_scratch.yaml) for the complete config.

## Tips for Good Results

1. **Clear pre-known items**: Set `known_surah_ids: []` if learning from scratch
2. **Zero initial placement**: Set `verse_base_review_count: 0`
3. **Lower threshold for small goals**: Use `cluster_stability_threshold: 0.15` for <20 items

## Troubleshooting

**Problem**: Exercise scores are 0.00
- **Cause**: Exercise sampling items that aren't being actively learned
- **Fix**: Remove `student_profile` and use `student_params` directly with empty `known_surah_ids`

**Problem**: All grades are "Again"
- **Cause**: Cluster threshold too high, items not getting enough practice
- **Fix**: Lower `cluster_stability_threshold` by 0.05-0.10
