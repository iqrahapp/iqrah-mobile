# M2.9.3 Pareto Sweep Results (Juz 30)

> **Git SHA**: M2.9.3
> **Seeds**: n=3 (screening)
> **Scenarios**: juz_amma_dedicated variants

---

## Dedicated 180d Pareto Frontier

| Config | session_size | target_reviews | Coverage% | RPM | Cov(T) |
|--------|--------------|----------------|-----------|-----|--------|
| baseline (ss20) | 20 | 0.08 | 48.9 | 0.127 | 31.1 |
| ss30 | 30 | 0.08 | 49.5 | 0.152 | 35.9 |
| **ss40 (BEST)** | 40 | 0.06 | **65.9** | 0.191 | 46.2 |

---

## Key Insight

**session_size=40 with target_reviews_per_active=0.06** produces:
- **+17pp coverage** (48.9% → 65.9%)
- **423 active items** (vs 312 at ss20)
- **at_risk_ratio still bounded** (< 0.02)

The increased capacity allows more items to be maintained while M2.7 sorting keeps them healthy.

---

## Recommended Configuration (Dedicated)

```yaml
session_size: 40
target_reviews_per_active: 0.06
intro_min_per_day: 5
```

---

## Trade-offs

| More Reviews (ss40) | Fewer Reviews (ss20) |
|---------------------|----------------------|
| Higher coverage | Lower coverage |
| More items active | Fewer items active |
| Higher RPM | Lower RPM |
| Same at_risk_ratio | Same at_risk_ratio |
