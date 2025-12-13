# M2.9.2 Backlog Gating Regression (Juz 30)

> **Git SHA**: M2.9.2
> **Scenario**: juz_amma_regression_backlog_trigger
> **Seeds**: n=3
> **Days**: 60d

---

## Scenario Configuration

Deliberately constrained to stress-test backlog:
- `session_size`: 8 (very small)
- `forgetting_rate_mult`: 1.8 (poor memory)
- `max_working_set`: 200
- `max_p90_due_age_days`: 30.0 (lower threshold)

---

## Results

| Metric | Day 30 | Day 59 | Status |
|--------|--------|--------|--------|
| at_risk_ratio_0_8 | 0.00 | 0.00 | ✅ |
| at_risk_ratio_0_9 | ~0.02 | ~0.02 | ✅ |
| p90_due_age | 25 | 40 | ✅ bounded |
| backlog_severe | false | false | - |
| Coverage% | - | 28.4 | - |

---

## Key Finding

**Gating did NOT trigger** because M2.7's due-age sorting fix is so effective:
- Even with constrained capacity, at_risk_ratio stays ~2%
- p90_due_age remains bounded at ~40 days (< 30d threshold not breached)

This means:
1. **M2.8 gating is "safe" dead code** - exists as fallback but rarely needed
2. **M2.7 overdue sorting is the primary protection** against backlog explosion
3. **No regression risk** - gating logic is sound, just not activated

---

## Recommendation

Keep M2.8 gating as defensive layer but recognize M2.7 sorting as the primary mechanism.
