# Session Modes & Composition Strategies – Add-on to Scheduler v2.1

This document extends the main scheduler specification with support for two distinct session types.

---

## 12. Session Modes & Composition Strategies

The scheduler must support **two distinct session types**, using the same core logic (Prerequisite Gate, scoring, bandit), but with different:

- Candidate filters, and
- Final composition rules.

### 12.1 SessionMode Enum

Introduce a simple mode flag:

```rust
pub enum SessionMode {
    Revision,       // Mode 1: revision-only, mix by content difficulty
    MixedLearning,  // Mode 2: mix of new + various mastery levels
}
```

Update the `generate_session` function signature to include the mode:

```rust
pub fn generate_session(
    user_id: i64,
    goal_id: &str,
    profile: &UserProfile,
    session_size: usize,
    now_ts: i64,
    mode: SessionMode,
) -> Vec<String> {
    // ...
}
```

All logic described below is in addition to the existing v2.0 spec.

---

### 12.2 Shared Concepts (Already Available)

You will reuse:

* `energy` from `user_memory_states`:

  * `energy == 0.0` or no row ⇒ **new**.
  * `energy > 0.0` ⇒ **seen before**.
* `next_due_ts`:

  * `next_due_ts <= now_ts` ⇒ **due/overdue**.
* `difficulty_score`:

  * Content difficulty from 0.0 (easy) → 1.0 (hard).
* **Prerequisite Gate**:

  * Parents' `energy >= MASTERY_THRESHOLD` (0.3) required.

The **core pipeline stays identical**:

1. Stage 1: Fetch candidate nodes.
2. Stage 2: Apply Prerequisite Gate and compute readiness, days_overdue, score.
3. Stage 3: Sort by score.
4. **New:** Compose session according to `SessionMode`.

Only the **candidate filter** and **composition step** differ by mode.

---

### 12.3 Mode 1 – Revision Sessions (Review Only)

**Goal:**
Review things the user has already seen (no new content), with a mix of easy/medium/hard **content difficulty**.

#### 12.3.1 Candidate Filter (Stage 1)

For `SessionMode::Revision`, candidates must be:

* Seen before:

  * `user_memory_states` row exists for `(user_id, node_id)`,
* And due:

  * `next_due_ts <= now_ts`.

In SQL terms, change the WHERE clause to something equivalent to:

```sql
WHERE
    ng.goal_id = :goal_id
    AND ums.user_id = :user_id
    AND ums.next_due_ts <= :now_ts
```

Do **not** include completely new nodes or not-yet-due items in Revision mode.

Then apply the normal Prerequisite Gate and scoring as in v2.0.

#### 12.3.2 Composition for Revision

For Revision, use the existing **difficulty-based** bucketing:

* Easy:   `difficulty_score < 0.4`
* Medium: `0.4 <= difficulty_score < 0.7`
* Hard:   `difficulty_score >= 0.7`

A reasonable default ratio is (for example):

* 60% Easy,
* 30% Medium,
* 10% Hard,

with the same fallback rule already defined in the spec:

* If a bucket lacks enough items, take all of them,
* Then fill remaining slots from other buckets based on **global priority**.

This gives a revision session that:

* Contains only seen items (due reviews),
* Is mixed by **content difficulty**,
* Still respects all the core heuristics (urgency, foundation, influence, readiness).

---

### 12.4 Mode 2 – Mixed Learning Sessions

**Goal:**
A "normal" session that mixes new content with items at different mastery levels. For example:

* 10% new content,
* 10% almost mastered content,
* 50% "almost there" content,
* 20% struggling content,
* 10% really struggling content.

This is driven by **user mastery** (`energy`), not content difficulty.

#### 12.4.1 Candidate Filter (Stage 1)

For `SessionMode::MixedLearning`, reuse the **original v2.0 candidate logic**:

* Candidates are nodes that:

  * Belong to `goal_id` via `node_goals`, and
  * Are either:

    * Due: `next_due_ts <= now_ts`, **or**
    * New: no `user_memory_states` row or `energy = 0.0`.

In SQL, this is the same WHERE condition as in the main spec:

```sql
AND (
    (ums.next_due_ts IS NOT NULL AND ums.next_due_ts <= :now_ts)
    OR ums.user_id IS NULL
    OR ums.energy = 0.0
)
```

Then apply the Prerequisite Gate and scoring as usual.

#### 12.4.2 Mastery Bands (By Energy)

After scoring and sorting, you will bucket nodes by **energy** (user mastery), not `difficulty_score`.

Define energy bands (tunable thresholds):

```text
New:               energy == 0.0 OR no memory row
Really struggling: 0.0  < energy <= 0.2
Struggling:        0.2  < energy <= 0.4
Almost there:      0.4  < energy <= 0.7
Almost mastered:   0.7  < energy <= 1.0
```

You must preserve the **global priority order** within each band (i.e., no re-sorting inside bands; just stable partition).

#### 12.4.3 Session Mix Targets

For `session_size = N`, define target counts:

```text
target_new               = round(0.10 * N)
target_almost_mastered   = round(0.10 * N)
target_almost_there      = round(0.50 * N)
target_struggling        = round(0.20 * N)
target_really_struggling = N - (sum of others)
```

These can be slightly adjusted if rounding causes minor mismatches; the key is the rough 10/10/50/20/10 ratio.

#### 12.4.4 Composition Algorithm (MixedLearning)

1. Take the top `K = 3 * session_size` candidates by global score (or fewer if not enough).
2. Partition this list into the five energy bands listed above, preserving order.
3. Initialize an empty `session` list.
4. For each band, in any order you deem logical (e.g. New → Almost mastered → Almost there → Struggling → Really struggling):

   * Take up to `target_*` nodes from that band,
   * Append them to `session`.
5. **Fallback rule for MixedLearning:**

   * If a band has fewer nodes than its target:

     * Use all nodes from that band,
     * Then fill remaining slots from **other bands** based on overall priority.
   * Simple rule:

     * After consuming each band up to its capacity, merge all remaining candidates (from all bands) in original score order and fill the remaining slots until `session.len() == session_size`.

This yields sessions like:

* Some brand-new content,
* Some nearly mastered content (for consolidation),
* A majority of items in the "almost there" region,
* A meaningful portion of struggling / really struggling items for targeted reinforcement.

All of this is still constrained by:

* The **Prerequisite Gate** (no child before its parents are truly mastered),
* Unified scoring (urgency, foundation, influence, readiness),
* Bandit-chosen `UserProfile` weights.

---

### 12.5 Interaction with Bandit v2.1

The bandit logic is unchanged:

* It chooses a `ProfileName` per `(user_id, goal_group)`,
* You blend the chosen profile with the safe default to get `UserProfile`,
* You pass that `UserProfile` to `generate_session`.

The `SessionMode` does **not** change how the bandit works:

* You may apply the bandit to both `Revision` and `MixedLearning` sessions (same `goal_group`),
* Or limit bandit optimization to `MixedLearning` if you prefer.

The only requirement is that for each completed session, you can still compute a `SessionResult` (correct/total/completed/presented) and call the bandit's `update_arm` with the resulting reward.

---

**Summary:**

* `SessionMode::Revision`:

  * Filter: seen & due only.
  * Mix: by `difficulty_score` (content difficulty).
* `SessionMode::MixedLearning`:

  * Filter: new + due (original v2.0 spec).
  * Mix: by `energy` (mastery bands) with 10/10/50/20/10 proportions.

Both modes reuse the same core scheduler, Prerequisite Gate, scoring function, and bandit architecture.
