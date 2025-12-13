# How to Run Long Validations

## n=30 Seed Confirmation

```bash
# SS40 Dedicated (180d, n=30)
cargo run --release -p iqrah-iss -- compare \
  --scenario juz_amma_dedicated \
  -V iqrah_default -V baseline_random \
  -n 30 --days 180 \
  --trace --trace-dir trace_output/ss40_n30_confirm

# Casual (365d, n=30)
cargo run --release -p iqrah-iss -- compare \
  --scenario juz_amma_casual_365d \
  -V iqrah_default -V baseline_random \
  -n 30 --days 365 \
  --trace --trace-dir trace_output/casual_n30
```

## Output Directory Convention

```
trace_output/
├── {scenario}_{config}_{date}/
│   ├── {scenario}_{variant}_gate_trace.csv
│   ├── {scenario}_{variant}_memory_health_trace.csv
│   └── run.log
```

## Key Metrics to Check (Final Day)

| Metric | Dedicated Target | Casual Target |
|--------|-----------------|---------------|
| Coverage% | ≥ 60 | ≥ 40 |
| at_risk_ratio_0.9 | ≤ 0.15 | ≤ 0.20 |
| p10_R_today | ≥ 0.85 | ≥ 0.80 |

## Generate Summary

```bash
# Extract final day row
tail -1 trace_output/ss40_n30_confirm/*_memory_health_trace.csv
```
