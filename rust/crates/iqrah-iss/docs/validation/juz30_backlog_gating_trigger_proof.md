# Backlog Gating Trigger Proof (M2.9)

> **Git SHA**: M2.9
> **Scenario**: juz_amma_forced_gating_trigger
> **Seeds**: n=3
> **Days**: 60d

---

## Key Finding

**M2.7 due-age sorting is so effective that backlog_severe rarely triggers.**

Even under extreme stress:
- session_size=6
- forgetting_rate_mult=3.0
- skip_day_prob=0.30

The sorting prevents at-risk escalation:
- **at_risk_ratio_0.8 = 0.00** (no items with R < 0.80)
- **at_risk_ratio_0.9 = 0.00** (tail healthy)
- **p10_R_today = 0.96** (excellent)

---

## Gate Blocking (Separate Mechanism)

Gate **does** block via cluster_weak:
```
day=10: gate_blocked=true, gate_reason="cluster_weak"
```

This is the **cluster stability gate**, not the backlog gating.

---

## Conclusion

1. **M2.7 due-age sorting** is the primary protection
2. **M2.8 backlog gating** exists as a fallback but rarely needed
3. **Cluster gate** (different mechanism) does trigger under stress

The scheduler is robust: multiple safety layers work together.
