# Iqrah Scheduler Upgrade – Implementation Plan for AI Agent
**Spec Version:** V2.1 FINAL – Prerequisite Gate + Bandit Optimizer
**Target Iterations:** ~2–3 implementation passes

---

## 0. How You (the AI Agent) Should Work

You are acting as an implementation agent for the Iqrah project.

For each iteration:

1. **Read this document fully.**
2. **Plan the changes** (files, modules, structs, SQL, tests).
3. **Implement in small, coherent commits/patches.**
4. **Update a `docs/scheduler_v2/progress.md` file** to document:
   - What you implemented (per section/step from this spec),
   - Any deviations or TODOs,
   - How to run tests / CLI validation.

Keep the implementation:

- Idiomatic Rust,
- Modular and testable,
- With minimal surprises for a human maintainer.

---

## 1. High-Level Overview

We are replacing the current "flat priority queue" scheduler with:

1. **Scheduler v2.0 – Hybrid Two-Stage Scheduler (Prerequisite Gate Model)**
   - Uses a knowledge graph (nodes + `prereq` edges),
   - Uses user memory state (`knowledge_energy`, `next_due_ts`),
   - Applies a **Prerequisite Mastery Gate** based on long-term knowledge,
   - Ranks eligible nodes using a unified scoring function,
   - Builds sessions with a **60% easy / 30% medium / 10% hard** difficulty mix.

2. **Scheduler v2.1 – Bandit-Driven Hyper-Personalization**
   - Uses a **Thompson Sampling Multi-Armed Bandit** per user and `goal_group`,
   - Chooses among several `UserProfile` presets (different weightings),
   - Blends the chosen profile with a safe default for UX stability,
   - Updates the bandit state based on session performance (accuracy + completion).

You must implement **v2.0 first**, fully tested and validated via CLI.
Then add **v2.1** on top.

---

## 2. Data & Schema Requirements

You may not control schema creation in code, but you must assume and use these tables and fields as specified.

### 2.1 Content / Graph DB (`content.db`)

#### 2.1.1 `nodes` and `node_metadata`

We assume:

- `nodes` table has at least: `id TEXT PRIMARY KEY`.
- `node_metadata` contains per-node numeric scores.

The R&D pipeline must provide these per-node values:

- `foundational_score` (REAL): PageRank on forward graph.
- `influence_score` (REAL): PageRank on reversed graph.
- `difficulty_score` (REAL): `0.0` (easy) → `1.0` (hard).
- `quran_order` (INTEGER): `(surah_idx * 1_000_000) + (ayah_idx * 1000) + word_idx`.

All four may live either directly in `nodes` or in `node_metadata`; you should support the `node_metadata` pattern (key/value style). Example pattern:

```sql
-- Example pattern (not necessarily the exact schema):
CREATE TABLE node_metadata (
    node_id TEXT NOT NULL,
    key TEXT NOT NULL,
    value REAL NOT NULL,
    PRIMARY KEY (node_id, key)
);
```

You will assume keys: `'foundational_score'`, `'influence_score'`, `'difficulty_score'`, `'quran_order'`.

#### 2.1.2 `edges` with `edge_type`

```sql
CREATE TABLE edges (
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    edge_type TEXT NOT NULL,         -- e.g., 'prereq', 'similar', ...
    -- other fields...
    PRIMARY KEY (source_id, target_id, edge_type)
);
```

* **Scheduler must only use** edges where `edge_type = 'prereq'`.
* The `prereq` subgraph is guaranteed to be a DAG by the R&D pipeline.

#### 2.1.3 Goals & Goal Mapping

```sql
CREATE TABLE goals (
    goal_id TEXT PRIMARY KEY,
    goal_type TEXT NOT NULL,   -- e.g., 'surah', 'root', 'theme'
    goal_group TEXT NOT NULL,  -- e.g., 'memorization', 'vocab', 'tajweed'
    label TEXT NOT NULL        -- e.g., "Surah Al-Mulk", "Root K-T-B"
);

CREATE TABLE node_goals (
    goal_id TEXT NOT NULL,
    node_id TEXT NOT NULL,
    PRIMARY KEY (goal_id, node_id)
);
```

You will use `node_goals` to restrict candidate nodes for a given `goal_id`.
You will use `goals.goal_group` for the bandit layer.

---

### 2.2 User DB (`user.db`)

#### 2.2.1 Memory State

Assume a table like:

```sql
CREATE TABLE user_memory_states (
    user_id INTEGER NOT NULL,
    node_id TEXT NOT NULL,
    energy REAL NOT NULL,         -- knowledge_energy ∈ [0,1]
    next_due_ts INTEGER NOT NULL, -- unix timestamp (seconds)
    PRIMARY KEY (user_id, node_id)
);
```

Semantics:

* `energy`: long-term mastery proxy (FSRS-like), 0.0–1.0.
* `next_due_ts`: when this node should be reviewed again.
* If a node has **no row** for a user, treat it as "new":

  * `energy = 0.0`,
  * `next_due_ts = 0` (or "not scheduled yet").

#### 2.2.2 Bandit State

```sql
CREATE TABLE user_bandit_state (
    user_id INTEGER NOT NULL,
    goal_group TEXT NOT NULL,      -- "memorization", "vocab", etc.
    profile_name TEXT NOT NULL,    -- e.g., "Balanced", "FoundationHeavy"
    successes REAL NOT NULL DEFAULT 1.0,
    failures REAL NOT NULL DEFAULT 1.0,
    PRIMARY KEY (user_id, goal_group, profile_name)
);
```

This table is used only in v2.1.

---

## 3. Core Concepts & Heuristics

These MUST be implemented exactly; do not improvise.

### 3.1 UserProfile Weights

```rust
pub struct UserProfile {
    pub w_urgency: f32,
    pub w_readiness: f32,
    pub w_foundation: f32,
    pub w_influence: f32,
}
```

* `UserProfile` instances are presets (e.g., "Balanced", "FoundationHeavy", etc.).
* v2.1 will choose among these via the bandit.

### 3.2 Per-Node Structs

```rust
pub struct CandidateNode {
    pub id: String,
    pub foundational_score: f32,
    pub influence_score: f32,
    pub difficulty_score: f32,
    pub energy: f32,        // from user_memory_states or 0.0 if none
    pub next_due_ts: i64,   // from user_memory_states or 0
    pub quran_order: i64,
}

pub struct InMemNode {
    pub data: CandidateNode,
    pub parent_ids: Vec<String>,  // only 'prereq' parents
}
```

### 3.3 Thresholds & Definitions

* `MASTERY_THRESHOLD = 0.3f32`
* `knowledge_energy`: from `user_memory_states.energy` or `0.0` if none.
* `days_overdue`:

  * Let `now_ts` = current UNIX timestamp (seconds).
  * If `next_due_ts < now_ts`:

    * `days_overdue = ((now_ts - next_due_ts) as f32 / 86400.0).floor()`.
  * Else:

    * `days_overdue = 0.0`.

### 3.4 Prerequisite Mastery Gate (v2.0)

A node is **eligible** for a session only if **all** of its `prereq` parents are sufficiently mastered.

For each candidate node:

1. Fetch energies for all its `prereq` parents.

2. Compute:

   ```text
   unsatisfied_parent_count = number of parents where energy < MASTERY_THRESHOLD
   ```

3. Rule:

   * Only nodes with `unsatisfied_parent_count == 0` are admitted into the ranking pool.
   * This is computed once per session and **does not change** during ranking.
   * Scheduling a node in this session does **not** unlock its children in the same session; children become eligible in future sessions once their parent's `energy` has been updated and passes the threshold.

### 3.5 Readiness

For a node that passes the gate:

* Let `parent_energies` be the list of energies of its `prereq` parents.
* If it has no parents:
  `readiness = 1.0`.
* Else:
  `readiness = mean(parent_energies)`.

### 3.6 Urgency Factor

```rust
urgency_factor = 1.0 + profile.w_urgency * (1.0 + days_overdue.max(0.0)).ln();
```

* This grows quickly for small `days_overdue` and saturates slowly.

### 3.7 Final Priority Score

```rust
fn calculate_priority_score(
    node: &InMemNode,
    profile: &UserProfile,
    readiness: f32,
    days_overdue: f32,
) -> (f64, i64) {
    let urgency_factor =
        1.0 + (profile.w_urgency * (1.0 + days_overdue.max(0.0)).ln());

    let learning_potential =
          profile.w_readiness   * readiness
        + profile.w_foundation * node.data.foundational_score
        + profile.w_influence  * node.data.influence_score;

    let final_score = urgency_factor * learning_potential;

    // First element: priority (descending); second: tie-breaker (ascending by Qur'an order)
    (final_score as f64, -node.data.quran_order as i64)
}
```

Sorting on this tuple with a max-heap / descending order gives:

* Highest score first,
* Ties broken by Qur'an ordering.

---

## 4. Session Composition Logic (Difficulty Mix)

We want: **60% Easy, 30% Medium, 10% Hard**, with fallback.

Let:

* `session_size` = requested number of items for this session (e.g., 20).

Steps:

1. **Generate ranked list of eligible nodes**:

   * Apply Prerequisite Gate.
   * Compute `readiness`, `days_overdue`, priority score.
   * Sort all eligible candidates by `(score, -quran_order)` descending.
   * Take a head slice of size `K = 3 * session_size` (or fewer if not enough).

2. **Bucket by difficulty** based on `difficulty_score`:

   * Easy: `difficulty_score < 0.4`
   * Medium: `0.4 <= difficulty_score < 0.7`
   * Hard: `difficulty_score >= 0.7`
   * Preserve original rank order within each bucket.

3. **Target counts**:

   * `target_easy   = (session_size as f32 * 0.6).round() as usize`
   * `target_medium = (session_size as f32 * 0.3).round() as usize`
   * `target_hard   = session_size - target_easy - target_medium`

4. **Populate session**:

   * Take up to `target_easy` from Easy bucket.
   * Take up to `target_medium` from Medium bucket.
   * Take up to `target_hard` from Hard bucket.

5. **Fallback rule**:

   * If any bucket has fewer items than its target:

     * Take all that bucket's items,
     * Then fill remaining slots from other buckets **in order of global priority**:

       * Prefer Medium, then Easy (but you may just merge remaining items and pick by original global ranking, as long as you stay consistent and documented).
   * Ensure final session length ≤ `session_size`.

---

## 5. Stage 1 – SQL Data Retrieval (v2.0)

You will likely implement a repository / data access layer. The exact SQL layer abstraction may vary, but the semantics must match.

### 5.1 Fetch Candidate Nodes

Inputs:

* `user_id`
* `goal_id`
* `now_ts` (UNIX timestamp)

Logic:

* Candidates are nodes:

  * That belong to `goal_id` via `node_goals`,
  * That are either **due** or **new**:

    * Due: `user_memory_states.next_due_ts <= now_ts`,
    * New: no row in `user_memory_states` or `energy == 0.0`.

Example SQL (you can adapt to your query builder):

```sql
SELECT
    n.id AS node_id,
    m_found.value  AS foundational_score,
    m_infl.value   AS influence_score,
    m_diff.value   AS difficulty_score,
    COALESCE(ums.energy, 0.0)        AS energy,
    COALESCE(ums.next_due_ts, 0)     AS next_due_ts,
    m_quran.value AS quran_order
FROM node_goals ng
JOIN nodes n ON ng.node_id = n.id
LEFT JOIN node_metadata m_found
    ON n.id = m_found.node_id AND m_found.key = 'foundational_score'
LEFT JOIN node_metadata m_infl
    ON n.id = m_infl.node_id AND m_infl.key = 'influence_score'
LEFT JOIN node_metadata m_diff
    ON n.id = m_diff.node_id AND m_diff.key = 'difficulty_score'
LEFT JOIN node_metadata m_quran
    ON n.id = m_quran.node_id AND m_quran.key = 'quran_order'
LEFT JOIN user_memory_states ums
    ON n.id = ums.node_id AND ums.user_id = :user_id
WHERE
    ng.goal_id = :goal_id
    AND (
        (ums.next_due_ts IS NOT NULL AND ums.next_due_ts <= :now_ts)
        OR ums.user_id IS NULL
        OR ums.energy = 0.0
    );
```

### 5.2 Fetch Prerequisite Parents for Candidates

We need `prereq` parents for each candidate node.

Because of parameter limits, you must support **chunking** when the candidate list is large.

For each chunk of candidate IDs:

```sql
SELECT target_id AS node_id, source_id AS parent_id
FROM edges
WHERE edge_type = 'prereq'
  AND target_id IN (:candidate_ids_chunk);
```

You then group by `node_id` to build `parent_ids` per node.

### 5.3 Fetch Parent Energies

For all unique parent IDs across all candidates:

```sql
SELECT node_id, energy
FROM user_memory_states
WHERE user_id = :user_id
  AND node_id IN (:parent_ids_chunk);
```

Parent nodes without entries: `energy = 0.0`.

You'll then compute `unsatisfied_parent_count` and `readiness` in Rust.

---

## 6. Scheduler v2.0 – Orchestrator Design

Implement a central function (exact API up to you, but conceptually):

```rust
pub fn generate_session(
    user_id: i64,
    goal_id: &str,
    profile: &UserProfile,
    session_size: usize,
    now_ts: i64,
) -> Vec<String> {
    // returns Vec<node_id>
}
```

### 6.1 Steps Inside `generate_session`

1. **Fetch candidates** from `content.db` + `user.db` (Section 5.1).
2. **Fetch prereq parents** for all candidates (Section 5.2).
3. **Fetch parent energies** (Section 5.3).
4. For each candidate:

   * Compute `unsatisfied_parent_count` from parent energies.
   * If `unsatisfied_parent_count > 0`, discard this candidate.
   * Otherwise:

     * Compute `readiness`:

       * No parents → `1.0`,
       * Else mean(parent energies).
     * Compute `days_overdue`.
     * Compute priority `(score, -quran_order)` using `calculate_priority_score`.
5. Sort eligible candidates by this tuple descending.
6. Apply difficulty bucketing and fallback logic (Section 4).
7. Return final `Vec<String>` of node IDs.

---

## 7. CLI Validation Tool

Add / extend a CLI command, e.g.:

```bash
iqrah-cli schedule \
    --user-id <id> \
    --goal-id <goal> \
    --session-size <n> \
    --now-ts <timestamp> \
    --verbose
```

**Verbose output must show**, per selected node:

* `node_id`
* `foundational_score`
* `influence_score`
* `difficulty_score`
* `energy`
* `readiness`
* `days_overdue`
* `unsatisfied_parent_count` (should always be 0 for final session)
* `final_score`
* `quran_order`

Tabular output is fine.

---

## 8. Testing (v2.0)

### 8.1 Unit Tests

* `calculate_priority_score`:

  * Vary `readiness`, `days_overdue`, `profile`.
  * Check monotonicity and sanity (e.g., increasing `days_overdue` increases score when everything else same).
* Readiness:

  * No parent → `1.0`.
  * Mixed parent energies → correct mean.

### 8.2 Integration Tests (Pure In-Memory)

Construct a small mock graph with:

* Nodes A, B, C, D.
* Edges: A → C, B → C, C → D (prereq).
* User energies:

  * A, B >= 0.3; C < 0.3.
* `generate_session`:

  * Should include C but **not** D.
* Difficulty bucketing:

  * Test sessions where one bucket is under-populated and fallback logic kicks in.

You can implement fake in-memory repositories for tests instead of hitting real SQLite.

---

## 9. Bandit v2.1 – Hyper-Personalization

After v2.0 is implemented and tested, implement v2.1.

### 9.1 SessionResult Data

You need a struct summarizing a completed session:

```rust
pub struct SessionResult {
    pub correct: u32,
    pub total: u32,
    pub completed: u32,   // number of items the user answered (vs skipped / abandoned)
    pub presented: u32,   // number of items shown
}
```

### 9.2 Reward Function

```rust
pub fn calculate_session_reward(session_result: &SessionResult) -> f32 {
    let accuracy   = session_result.correct   as f32 / session_result.total.max(1)      as f32;
    let completion = session_result.completed as f32 / session_result.presented.max(1) as f32;
    ((0.6 * accuracy) + (0.4 * completion)).clamp(0.0, 1.0)
}
```

### 9.3 UserProfile Presets

Define an enum and mapping:

```rust
pub enum ProfileName {
    Balanced,
    FoundationHeavy,
    InfluenceHeavy,
    UrgencyHeavy,
    // ...
}

pub fn profile_weights(name: ProfileName) -> UserProfile {
    match name {
        ProfileName::Balanced => UserProfile {
            w_urgency:    1.0,
            w_readiness:  1.0,
            w_foundation: 1.0,
            w_influence:  1.0,
        },
        ProfileName::FoundationHeavy => UserProfile {
            w_urgency:    0.8,
            w_readiness:  1.0,
            w_foundation: 1.5,
            w_influence:  0.8,
        },
        // etc...
    }
}
```

### 9.4 BanditOptimizer

Use `rand_distr::Beta` for Thompson Sampling.

Define something like:

```rust
pub struct BanditOptimizer<R: Rng> {
    rng: R,
    // possibly a DB pool handle
}

impl<R: Rng> BanditOptimizer<R> {
    pub fn choose_arm(
        &mut self,
        user_id: i64,
        goal_group: &str,
    ) -> ProfileName {
        // 1. Load all rows from user_bandit_state for (user_id, goal_group).
        // 2. For each row: sample Beta(successes, failures).
        // 3. Return the ProfileName with the highest sampled value.
        // 4. If no rows exist, initialize default rows (e.g., all arms with 1.0/1.0).
    }

    pub fn update_arm(
        &mut self,
        user_id: i64,
        goal_group: &str,
        profile_name: ProfileName,
        reward: f32,
    ) {
        // successes += reward; failures += (1.0 - reward).
        // Upsert into user_bandit_state.
    }
}
```

### 9.5 Profile Blending (UX Guard-Rail)

Define a safe default, e.g. `ProfileName::Balanced`.

When you choose an arm:

```rust
let chosen = profile_weights(chosen_profile_name);
let safe   = profile_weights(ProfileName::Balanced);

let final_profile = UserProfile {
    w_urgency:    0.8 * chosen.w_urgency    + 0.2 * safe.w_urgency,
    w_readiness:  0.8 * chosen.w_readiness  + 0.2 * safe.w_readiness,
    w_foundation: 0.8 * chosen.w_foundation + 0.2 * safe.w_foundation,
    w_influence:  0.8 * chosen.w_influence  + 0.2 * safe.w_influence,
};
```

Use `final_profile` in `generate_session`.

---

## 10. v2.1 Integration Flow

When a session is requested:

1. **Before scheduling:**

   * Look up `goal_group` from `goals` for this `goal_id`.
   * Call `BanditOptimizer::choose_arm(user_id, goal_group)` → `profile_name`.
   * Build `chosen_profile = profile_weights(profile_name)`.
   * Blend with default → `final_profile`.

2. **Schedule with v2.0:**

   * `generate_session(user_id, goal_id, &final_profile, session_size, now_ts)`.

3. **After session completion (when results are available):**

   * Compute `reward = calculate_session_reward(&session_result)`.
   * Call `BanditOptimizer::update_arm(user_id, goal_group, profile_name, reward)`.

---

## 11. Implementation Roadmap for You (the AI Agent)

### Iteration 1 – Foundations

* [ ] Create / update `docs/scheduler_v2/progress.md`.
* [ ] Implement `UserProfile`, `CandidateNode`, `InMemNode` structs in a dedicated `scheduler` module/crate.
* [ ] Implement `calculate_priority_score`, `days_overdue` helper, `readiness` helper.
* [ ] Implement data access for:

  * candidate nodes,
  * prereq parents,
  * parent energies,
  * with parameter chunking.
* [ ] Add CLI skeleton for `iqrah-cli schedule --verbose` (no full logic yet).
* [ ] Write basic unit tests for scoring helpers.

### Iteration 2 – Full v2.0 Scheduler

* [ ] Implement `generate_session` end-to-end:

  * Prerequisite Mastery Gate,
  * `readiness`, `days_overdue`,
  * priority scoring,
  * difficulty bucketing + fallback.
* [ ] Wire into CLI, printing all relevant fields in verbose mode.
* [ ] Add integration tests with a small mock graph.
* [ ] Update `docs/scheduler_v2/progress.md` with details & usage notes.

### Iteration 3 – v2.1 Bandit Integration

* [ ] Implement `SessionResult` and `calculate_session_reward`.
* [ ] Implement `ProfileName` enum + `profile_weights`.
* [ ] Implement `BanditOptimizer` with `choose_arm` and `update_arm`.
* [ ] Integrate bandit into the scheduling flow:

  * choose arm → blend profile → call `generate_session`.
* [ ] Add tests for bandit behavior (simple sanity checks).
* [ ] Update CLI and docs to mention bandit-based personalization.
* [ ] Update `docs/scheduler_v2/progress.md` with final status.

---

**End of spec.**
Implement this exactly, document your progress, and keep the code clean and testable.
