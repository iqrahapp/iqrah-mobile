# 04 - Scheduler Paths: Current vs Intended

## 1) There Are Three Scheduling Layers

### Layer A - Mobile runtime session path (active)
Files:
- `rust/crates/iqrah-api/src/api.rs` (`get_exercises`, `start_session`, `get_next_session_item`)
- `rust/crates/iqrah-core/src/services/session_service.rs`

This is what end users currently get.

### Layer B - `scheduler_v2` engine (implemented, not primary app path)
Files:
- `rust/crates/iqrah-core/src/scheduler_v2/*`
- CLI usage: `rust/crates/iqrah-cli/src/schedule.rs`

### Layer C - ISS simulation policy layer
Files:
- `rust/crates/iqrah-iss/src/simulator.rs`
- related ISS docs in `rust/crates/iqrah-iss/docs/*`

ISS is sophisticated, but it is a simulation framework and introduces additional policy logic not directly equal to mobile runtime behavior.

## 2) Active Mobile Runtime Scheduler Logic

`SessionService::get_due_items(...)`:
- Fetches due states only (`user_repo.get_due_states(user_id, now, limit*3)`).
- Keeps only reviewable node types.
- Scores each candidate with:

`priority = w_due * days_overdue + w_need * mastery_gap + w_yield * importance`

Where:
- `mastery_gap = 1 - energy`
- `importance` is hardcoded by node type (not graph score driven)
- high-yield mode mainly increases `w_yield`

Key consequences:
- If user has no memory states, there are no due items and session list can be empty.
- `goal_id` passed into `start_session` is stored but not used in candidate selection.
- Graph metadata (`foundational_score`, `influence_score`) is not actively driving mobile session selection here.

## 3) `scheduler_v2` Capabilities (Implemented)

`scheduler_v2` includes:
- prerequisite mastery gate
- readiness scoring from parent energies
- urgency from due-ness
- fairness/coverage terms
- session composition by mastery bands or difficulty buckets
- optional bandit profile optimization

It is richer and closer to intended adaptive design, but it depends on:
- usable goal assignments (`goals`, `node_goals`)
- metadata availability (including `difficulty_score` if used)
- clean prerequisite mapping

Current bundled DB has empty `goals/node_goals`, so direct production usage is blocked by data provisioning.

## 4) Potential Bug/Risk In Scheduler V2 Prerequisite Query

In `iqrah-storage` repository implementation, `get_prerequisite_parents(...)` currently filters edges with `edge_type IN (0, 1)`.
- If `0` means dependency and `1` means knowledge, this includes knowledge edges as prerequisites.
- That may make prerequisite gating noisy or overly strict.

This should likely be dependency-only unless intentionally designed otherwise.

## 5) FSRS + Energy Propagation (Review Update Path)

`LearningService::process_review(...)` performs:
- FSRS state transition (stability/difficulty/due date)
- energy update based on grade
- propagation across outgoing edges with attenuation by distribution type
- atomic persistence via `save_review_atomic`

Critical limitation:
- propagated updates are only applied if target memory state already exists.
- unseen connected nodes do not get initialized from propagation.

This weakens the intended "learn one unit, see global progress impact" effect, especially for new users.

## 6) Net Assessment

Current runtime scheduler is functional and stable, but too narrow for your target pedagogy.
It is closer to "due queue review" than "goal-driven Quran memorization with meaningful semantic transfer".
