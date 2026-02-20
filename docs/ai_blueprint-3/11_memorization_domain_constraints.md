# 11 - Memorization Domain Constraints (Quran-Specific)

This file adds domain constraints that should govern scheduling/product choices.

## 1) Two Different Objectives Must Coexist

1. Hifz (exact recitation, sequence continuity)
2. Fahm (understanding of words/roots/meanings)

Treating them as one metric causes bad scheduling decisions.

## 2) Why "Chunk Mode" Is Not Optional

Your real-world practice insight is correct: contiguous chunks matter.
- Traditional memorization relies on verse continuity and transition anchors.
- Purely global priority ranking can feel disorienting even if mathematically efficient.

Requirement:
- Keep global review obligations, but support a user-selected contiguous focus scope (surah/range/chunk).

## 3) Lexical ROI Constraint

For non-Arab learners, frequent words/roots have outsized value.
Scheduling must explicitly reserve capacity for:
- fragile high-frequency words
- high-impact roots
- contextual meaning recall

This should be a hard session budget component, not an accidental side effect.

## 4) Audio-First Reality

Hifz is strongly auditory-motor.
Even with strong text-based exercises, product should support:
- listen + repeat loops
- verse audio in practice context
- eventual qari selection in active learning flow

Audio support should be prioritized as learning infrastructure, not cosmetic feature.

## 5) Soft vs Hard Prerequisite Policy

Hard prerequisite blocking can stall users.
Domain-friendly policy:
- default to soft penalties/priority reduction for unmet prerequisites
- allow user override for intentional progression
- never let learner feel "locked out" of desired ayah study

## 6) Product-Level Scheduling Invariant

Every short session (5-15 min) should provide:
1. continuity progress (chunk/surah flow),
2. retention maintenance (due reviews),
3. lexical understanding gain (word/root meaning).

If one of these three is consistently absent, the session design is not aligned with Quran memorization reality.

## 7) Word/Root Frequency Prioritization (Practical, Not Optional)

The lexical budget should not be uniform across vocabulary. It should be front-loaded by frequency and daily prayer relevance.

Priority tiers for non-Arab learners:
1. Particle/conjunction function words first (`من`, `في`, `على`, `إلى`, `ما`, `لا`, `و`).
2. High-salience divine names/attributes early (`الله`, `الرحمن`, `الرحيم`, `رب`).
3. High-frequency verb roots next (`ق-و-ل`, `ك-و-ن`, `ع-ل-م`, `ع-م-ل`).

Constraint:
- `Al-Fatiha` vocabulary must have permanent boost because it is used in daily salah, independent of global graph ranking.

## 8) Lexical ROI Scoring Policy

For lexical candidate selection, use an explicit policy term, not only generic graph influence:

`lexical_priority = fragility * frequency_weight * spread_weight * prayer_boost`

Where:
- `fragility`: recent error burden / low retrievability.
- `frequency_weight`: corpus frequency bucket.
- `spread_weight`: cross-surah spread of the word/root.
- `prayer_boost`: extra weight for Fatiha and common salah surahs.

This keeps sessions aligned with real-world recitation value per minute.

## 9) Current Data Limitation (Must Be Addressed)

Runtime `rust/content.db` currently lacks direct lexical frequency metadata keys and has `morphology_segments` empty in this audit snapshot.

Implication:
- Frequency-aware lexical prioritization is currently underpowered unless precomputed metadata is added during generation.

Required generator output additions:
1. per-word and per-root occurrence counts,
2. cross-surah spread counts,
3. optional prayer-context boost tags.
