# ISS Telemetry Bridge Plan

## Purpose

Enable real-user validation by having the mobile app emit the same metrics ISS computes in simulation. This allows "shadow ISS" comparison.

---

## Required Daily Session Events

### Event: `iss.session_complete`

Emitted at end of each study session.

```json
{
  "event": "iss.session_complete",
  "timestamp": "2025-12-13T10:30:00Z",
  "user_id": "uuid",
  "session_id": "uuid",

  "items_reviewed": 20,
  "session_duration_min": 15.5,

  "reviewed_items": [
    {
      "item_id": 12345,
      "grade": "good",           // again/hard/good/easy
      "stability": 45.2,         // FSRS stability (days)
      "last_reviewed": "2025-12-10",
      "review_count": 5
    }
  ]
}
```

### Event: `iss.daily_health`

Emitted once per day (or computed from session events).

```json
{
  "event": "iss.daily_health",
  "timestamp": "2025-12-13T23:59:00Z",
  "user_id": "uuid",
  "day": 45,

  "total_active": 332,
  "introduced_today": 3,
  "reviewed_today": 18,

  "mean_R_today": 0.946,
  "p10_R_today": 0.898,
  "at_risk_ratio_0_9": 0.108,
  "p90_due_age_days": 85.0,

  "goal_size": 564,
  "introduced_ratio": 0.59
}
```

---

## Computing Today Health Metrics

### 1. Retrievability R(t)

For each active item:
```
R = (1 + t / (9 * S))^-1
where:
  t = days since last review
  S = FSRS stability
```

### 2. at_risk_ratio_0_9

```
at_risk_count = count(items where R < 0.90)
at_risk_ratio_0_9 = at_risk_count / total_active
```

### 3. p10_R_today

Sort all R values ascending, take 10th percentile.

### 4. mean_R_today

Average of all R values for active items.

---

## Minimal Schema (If Storage Limited)

If full item-level logging is too heavy, emit just:

```json
{
  "event": "iss.health_summary",
  "user_id": "uuid",
  "day": 45,
  "total_active": 332,
  "mean_R_today": 0.946,
  "at_risk_ratio_0_9": 0.108,
  "reviewed_today": 18
}
```

---

## Shadow ISS Comparison

1. Collect real user `iss.daily_health` events
2. Run ISS simulation with matching params
3. Compare Horizon + Today metrics
4. Identify divergence → tune ISS model
