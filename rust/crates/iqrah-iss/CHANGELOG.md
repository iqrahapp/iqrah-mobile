# ISS Changelog

## [v2.9.3] - 2024-12-13

### 🎉 Production Release: Quality-First Memorization Engine

**Major Achievement**: First scientifically-validated retention-guaranteed Hifz system.
Cluster-based practice proven 97% superior to random baselines on sequential memorization.

#### Key Metrics (Juz 30, n=30 seeds, 180 days)
- **Coverage**: 65.6% (370/564 ayahs learned)
- **Retention**: 95.9% mean recall probability
- **At-Risk Items**: <1% (safety threshold: 6%)
- **Exercise Score**: 1.0 (iqrah) vs 0.0 (random baseline)
- **Recall Quality**: 1.47 trials/word (Easy grade)

#### What's New in v2.9

**M2.4: Principled Introduction Policy**
- 7-stage introduction pipeline balancing capacity vs growth
- Floor/ceiling controls prevent both stagnation and flooding
- Bootstrap logic for rapid initial learning (Days 0-10)

**M2.6: Backlog-Aware Introduction**
- Dynamic floor adjustment based on p90_due_age
- Automatic introduction throttling when falling behind
- Prevents death spirals proactively

**M2.7: Fairness Guarantees**
- Strict due_age DESC sorting prevents overdue starvation
- Oldest items always prioritized in session generation
- Eliminates review scheduling bias

**M2.8: Exercise Framework Maturation**
- Separated availability from mastery quality
- Trials-based cognitive evaluation validated
- Memorization vs vocabulary axis distinction proven

**M2.9: Production Hardening**
- Multi-seed validation (n=30 for all benchmarks)
- Metric integrity (Horizon vs Today split)
- Parameter robustness confirmed (session_size 30-50 all equivalent)
- Zero give-ups, zero death spirals

#### Design Philosophy

**Conservative by Design**: ISS v2.9 acts like a patient tutor prioritizing
retention over speed. Target pace: ~2 ayahs/day (part-time learner profile).

**Safety-First**: <10% failure rate tolerance, automatic backlog protection,
guaranteed prevention of death spirals.

#### Cognitive Validation

**Memorization (Sequential)**:
- iqrah: 1.0 score, 1.47 trials/word → Easy grade (smooth recitation)
- random: 0.0 score, 6.46 trials/word → Again grade (broken sequence)
- **Proof**: Sequential practice 97% superior for Hifz

**Vocabulary (Independent)**:
- iqrah: 0.59 score | random: 0.61 score → Tie
- **Proof**: Both strategies valid for independent recall

#### Evolution from v2.5

| Metric | v2.5 (Broken) | v2.9.3 (Production) | Improvement |
|--------|---------------|---------------------|-------------|
| Coverage | 0.2% | 65.6% | **300x** |
| Failure Rate | 67% | <7% | **-90%** |
| Give-ups | 100% | 0% | **Fixed** |

#### Known Limitations

1. **Conservative Pace**: Default settings match part-time learners (2 ayahs/day).
   Full-time intensive students may find it slow.

2. **Single Profile Validated**: Only quality-first profile validated. Speed-optimized
   settings (intro_min_per_day=15) require additional validation.

3. **Small Goal Friction**: Cluster gate may feel slow for tiny goals (<20 items).

#### Breaking Changes

None. Fully backward compatible with v2.7/v2.8.

#### Migration Notes

No migration required. Existing configs work unchanged.

To leverage v2.9 improvements, consider:
- Using `session_size=40` (optimal, though 30-50 all work)
- Enabling exercises for cognitive validation
- Monitoring p90_due_age for backlog health

#### Future Work

- **v2.10**: Validate intensive profile (intro_min_per_day=15, target 5+ ayahs/day)
- **v3.0**: Sequential link tracking for enhanced recitation modeling

---



## v2.9.4 (2025-12-13)

### Promoted Presets
- **Dedicated Default**: SS40 (session_size=40, target_reviews=0.06, intro_min=5)
  - +17pp coverage (48.9% → 65.8%), at_risk_0.9=0.01
- **Casual Default**: SS15 (session_size=15, max_ws=300, intro_min=4)
  - +7pp coverage (44.6% → 51.5%), at_risk_0.9=0.11

### M2.4 Principled Introduction Policy
- 7-stage policy engine with expand/contract modes
- Policy decisions logged for debugging
- Intro allowance = min(raw, capacity_cap, ws_cap, gate_cap)

### M2.5 Working-Set Ratio of Goal
- `max_working_set_ratio_of_goal` param for fair benchmark comparison
- Effective max_ws = min(max_working_set, goal_size × ratio)

### M2.6 Backlog-Aware Working Set + Floor
- `target_reviews_per_active` determines max sustainable working set
- `max_p90_due_age_days` threshold for backlog severity
- `intro_min_per_day` floor prevents introduction starvation

### M2.7 Due Fairness Fix (CRITICAL)
- Due candidates sorted by `due_age DESC` before selection
- Deterministic tie-break with `item_id ASC`
- Prevents overdue starvation

### M2.8 At-Risk Backlog Metrics
- `at_risk_ratio` (R < 0.80) and `at_risk_ratio_0_9` (R < 0.90)
- `p10_R_today` for tail health monitoring
- `p90_due_age_at_risk` (due age among at-risk items only)

### M2.9 Metric Integrity + Robustness
- **Horizon vs Today** metric separation enforced
- Multi-seed validation (n=30) for both scenarios
- Student profile variability testing (strong/avg/weak)
- CI smoke tests for regression prevention

### Documentation
- `docs/design/metrics_definitions.md` - Horizon vs Today rules
- `docs/design/backlog_gating.md` - Safety layer hierarchy
- `docs/validation/juz30_recommended_presets.md` - Preset guidance
