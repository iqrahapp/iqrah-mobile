# 13 - Exercise Catalog: Scored And Runtime-Wired

Purpose: keep the strong exercise analysis from `ai_blueprint-2`, but grounded in what is actually wired.

## 1) Runtime Wiring Truth

### A) Default scheduled session path (`get_exercises` / `start_session`)
Default routing in `ExerciseService::generate_exercise_v2(...)`:
- word/word_instance -> `memorization`
- verse + memorization axis -> `echo_recall`
- verse non-memorization axis -> `full_verse_input`
- chapter -> `ayah_chain`

So default scheduled sessions are currently narrow.

### B) Sandbox preview path (`get_exercises_for_node`)
Many additional generators are exposed for a chosen node (translation, cloze variants, root, POS, etc.), but that is not the same as policy-driven daily scheduling.

## 2) Scoring Legend

- `Core`: essential for current product goals
- `Useful`: good after core loop is stable
- `Optional`: niche or mode-specific
- `Defer/Cut`: low ROI now or misaligned

## 3) Full Exercise List (21)

| Exercise type | Current default scheduled usage | Product value | Decision |
|---|---|---|---|
| `memorization` | Yes (word/word_instance) | High for recall | Core |
| `mcq_ar_to_en` | No (not default) | Very high for non-Arab meaning recall | Core |
| `mcq_en_to_ar` | No (not default) | High reverse retrieval | Core |
| `translation` | No (sandbox-level) | Medium; grading complexity | Useful |
| `contextual_translation` | No (sandbox-level) | Very high (meaning in context) | Core |
| `cloze_deletion` | No (sandbox-level) | High memorization drill | Core |
| `first_letter_hint` | No (sandbox-level) | Medium scaffold | Useful |
| `missing_word_mcq` | No (sandbox-level) | High low-friction recall | Core |
| `next_word_mcq` | No (sandbox-level) | High sequence continuity | Core |
| `full_verse_input` | Default for many verse contexts | Medium/low on mobile due typing burden | Optional |
| `ayah_chain` | Yes (chapter) | High for contiguous recitation | Core |
| `find_mistake` | No (sandbox-level) | Medium precision check | Useful |
| `ayah_sequence` | No (sandbox-level/chapter preview) | Low-medium practical ROI | Defer/Cut |
| `sequence_recall` | No (sandbox-level) | Medium for advanced learners | Optional |
| `first_word_recall` | No (sandbox-level) | Medium anchor recall | Useful |
| `identify_root` | No (sandbox-level) | Very high for lexical understanding | Core |
| `reverse_cloze` | No (sandbox-level) | High cross-link meaning -> Arabic | Core |
| `translate_phrase` | No (sandbox-level) | Medium; grading burden | Optional |
| `pos_tagging` | No (sandbox-level) | Low for current audience | Defer/Cut |
| `cross_verse_connection` | No (sandbox-level) | Low/subjective for current stage | Defer/Cut |
| `echo_recall` | Yes (memorization verse) | Very high; signature mechanic | Core |

## 4) Recommended Session Mix (Pragmatic)

For a standard 10-15 minute session, target:
- 40-50% continuity recall (`echo_recall`, `ayah_chain`, selected cloze/next-word)
- 30-40% lexical meaning/root drills (`mcq_ar_to_en`, `contextual_translation`, `identify_root`)
- 10-20% fragile item reinforcement (high-error words/roots)

This mix directly supports your stated goal: memorization and understanding for users with limited time.

## 5) Immediate Exercise Wiring Upgrades

1. Promote these from sandbox-only into scheduled policy:
- `mcq_ar_to_en`
- `contextual_translation`
- `missing_word_mcq` / `next_word_mcq`
- `identify_root`

2. Keep but demote by default:
- `full_verse_input`
- `translate_phrase`
- `sequence_recall`

3. Defer/cut from default pool:
- `pos_tagging`
- `cross_verse_connection`
- `ayah_sequence` (unless specific mode needs it)

## 6) Key Principle

Do not optimize for number of exercise variants. Optimize for measured learning outcomes per minute.

## 7) Per-Exercise UX/Grading Failure Modes (Critical)

This is where good exercise ideas often fail in production.

| Exercise type | Main failure mode today | Fix required |
|---|---|---|
| `identify_root` | Text input for Arabic roots is high-friction and error-prone on mobile | Convert to MCQ root selection or letter-tile assembly from source word |
| `translation` | Exact/naive string grading rejects valid synonyms | Use normalized + synonym-aware grading or convert to MCQ for default mode |
| `translate_phrase` | Free-text phrase grading is unreliable at current stage | Keep optional; avoid core scheduling until robust semantic grading exists |
| `full_verse_input` | Full Arabic typing on phone is too costly for most users | Demote to challenge mode; prefer cloze/MCQ/audio-assisted formats |
| `memorization` | Arabic keyboard friction for beginners | Pair with hint modes and optional audio prompt path |
| `find_mistake` | Weak distractors make task trivial | Generate semantically/orthographically close distractors only |
| `contextual_translation` | High value but can be weak if context window is too small | Always show enough verse context to preserve meaning |
| `echo_recall` | Flagship potential, but weak if not tied to visibility/energy feedback | Keep as signature mode and surface progress/propagation clearly |

## 8) Axis-to-Exercise Mapping Guardrail

Exercise assignment must follow node/axis semantics:
1. Root/lemma meaning axes -> lexical exercises (`mcq_ar_to_en`, `contextual_translation`, `identify_root`).
2. Verse memorization/context axes -> continuity exercises (`echo_recall`, cloze/next-word family).
3. Avoid nonsensical mappings (for example deep grammar drills in default beginner flow).

If axis mapping is wrong, even a strong scheduler will produce low-value sessions.

## 9) Scheduling Policy Upgrade (Concrete)

Default scheduled pool should be intentionally small and high quality:
1. `echo_recall`
2. `missing_word_mcq` or `next_word_mcq`
3. `mcq_ar_to_en`
4. `contextual_translation`
5. `identify_root` (redesigned UX)

Everything else should be either:
- secondary pool (opt-in or advanced),
- or sandbox/experimental only.
