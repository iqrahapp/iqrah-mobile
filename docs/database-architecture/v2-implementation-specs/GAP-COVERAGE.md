# Gap Coverage Matrix

**Purpose:** Map gaps identified in [../09-gaps-and-recommendations.md](../09-gaps-and-recommendations.md) to v2 implementation specs.

**Status:** Complete coverage with one exception (noted below)

## Gap to Spec Mapping

### Gap 1: Knowledge Axis Implementation
**Severity:** 🔴 CRITICAL
**Priority:** P1 (Decision: MVP or post-MVP)

**Coverage:** ✅ **FULLY ADDRESSED**
**Spec:** [05-knowledge-axis-and-session-integration.md](05-knowledge-axis-and-session-integration.md)

**Details:**
- 4-phase implementation plan provided
- KnowledgeAxis enum design
- Axis-aware session generation
- Exercise type system (memorization, translation, tajweed)
- Cross-axis energy propagation

**Status:** Marked as **POST-MVP** in spec (P3 priority)

**Decision Required:** If axis-specific exercises are critical for MVP, elevate priority to P1 and implement Phase 1-2 before launch.

---

### Gap 2: Flexible Content Package System
**Severity:** 🟡 HIGH
**Priority:** P4 (Post-MVP)

**Coverage:** ✅ **FULLY ADDRESSED**
**Spec:** [03-flexible-content-packages-plan.md](03-flexible-content-packages-plan.md)

**Details:**
- 3-phase plan (MVP → Multi-translation → Full packages)
- Phase 1: Single translation (current state) - **DONE**
- Phase 2: Multi-translation without packages (3-5 days) - Covered in [02-translations-and-translators-normalization.md](02-translations-and-translators-normalization.md)
- Phase 3: Full downloadable package system (2-3 weeks) - Detailed workflow provided

**Status:** Phase 2 recommended for MVP (translator selection), Phase 3 deferred to post-MVP

---

### Gap 3: Knowledge Graph Migration Strategy
**Severity:** 🔴 CRITICAL
**Priority:** P0 (Blocker for production)

**Coverage:** ✅ **FULLY ADDRESSED**
**Spec:** [04-versioning-and-migration-strategy.md](04-versioning-and-migration-strategy.md) - Part 2

**Details:**
- ID Stability Guarantee (recommended approach)
- Python validation script for CI/CD
- Graph version tracking in CBOR
- Enforcement mechanism (prevent breaking changes)

**Status:** **P0 - MUST IMPLEMENT** before production release

---

### Gap 4: Content Versioning
**Severity:** 🟡 MEDIUM
**Priority:** P2 (Next sprint)

**Coverage:** ✅ **FULLY ADDRESSED**
**Spec:** [04-versioning-and-migration-strategy.md](04-versioning-and-migration-strategy.md) - Part 1

**Details:**
- `schema_version` table design
- Version validation on app startup
- Feature gating based on version
- Migration strategy between versions

**Status:** **P0 - BUNDLED** with graph migration (same spec, easy to implement together)

---

### Gap 5: Schema Mismatch (Python vs Rust)
**Severity:** 🟡 MEDIUM
**Priority:** P3 (Clean-up)

**Coverage:** ✅ **FULLY ADDRESSED**
**Spec:** [01-content-schema-v2-purist.md](01-content-schema-v2-purist.md)

**Details:**
- Complete v2 schema specification
- Natural keys replace generic `nodes` table
- Domain-specific tables (chapters, verses, words, lemmas, roots, morphology)
- Clear migration path from v1 to v2

**Status:** **P0 - CORE REDESIGN** (Purist schema is the foundation for v2)

**Note:** This addresses the schema mismatch by making **Rust authoritative** and aligning Python to generate data for the new schema.

---

### Gap 6: String-Based Translators (Performance)
**Severity:** 🟡 MEDIUM
**Priority:** P1 (MVP enhancement)

**Coverage:** ✅ **FULLY ADDRESSED**
**Spec:** [02-translations-and-translators-normalization.md](02-translations-and-translators-normalization.md)

**Details:**
- Normalized `translators` table with INTEGER PK
- `languages` table for metadata
- `verse_translations` with `translator_id` FK
- Performance analysis (44% space savings, 2-3x faster queries)

**Status:** **P1 - RECOMMENDED FOR MVP** (significant performance improvement, low effort)

**Note:** This gap wasn't explicitly called out in the priority matrix but is addressed by the normalization spec.

---

## Summary Table

| Gap # | Gap Name | Severity | Priority | Spec | Status |
|-------|----------|----------|----------|------|--------|
| 1 | Knowledge Axis Implementation | CRITICAL | P1 → P3* | 05-knowledge-axis | ✅ Covered (deferred) |
| 2 | Flexible Content Packages | HIGH | P4 | 03-packages-plan | ✅ Covered (phased) |
| 3 | Graph Migration Strategy | CRITICAL | P0 | 04-versioning (Part 2) | ✅ Covered |
| 4 | Content Versioning | MEDIUM | P2 → P0* | 04-versioning (Part 1) | ✅ Covered |
| 5 | Schema Mismatch | MEDIUM | P3 → P0* | 01-content-schema-v2 | ✅ Covered |
| 6 | String Translators | MEDIUM | P1 | 02-translations-normalization | ✅ Covered |

**Legend:**
- ✅ = Fully addressed in spec
- * = Priority adjusted in v2 roadmap

## Priority Adjustments in v2 Roadmap

The v2 implementation specs made strategic priority adjustments:

### Elevated to P0 (Before Production):
1. **Content Versioning** (was P2) → Bundled with graph migration, easy to implement
2. **Schema Mismatch** (was P3) → Purist schema is foundation of v2, must be done first

### Deferred from P1 to P3:
1. **Knowledge Axis** (was P1) → Marked as post-MVP feature
   - Rationale: Complex implementation (2 weeks), not critical for MVP
   - Can be enabled later without breaking changes
   - Decision point clearly documented

### Phased Implementation:
1. **Flexible Packages** (P4) → Phase 2 elevated to P1
   - Phase 2 (multi-translation) is low-effort MVP enhancement
   - Phase 3 (full packages) remains P4 (post-MVP)

## Coverage Verification

**All 6 gaps from audit are addressed:**

✅ Gap 1 (Axis) → Spec 05 (with decision point for timing)
✅ Gap 2 (Packages) → Spec 03 (phased approach)
✅ Gap 3 (Graph Migration) → Spec 04 Part 2
✅ Gap 4 (Content Versioning) → Spec 04 Part 1
✅ Gap 5 (Schema Mismatch) → Spec 01
✅ Gap 6 (String Translators) → Spec 02

**No gaps left unaddressed.**

## Additional Improvements in v2 Specs

Beyond closing the identified gaps, v2 specs add:

1. **CHECK + NULL Handling** - Fixed SQLite NULL bugs throughout schema
2. **XOR + Partial Indexes** - Enforced for text_variants uniqueness
3. **Explicit CASCADE Semantics** - Documented for all foreign keys
4. **Performance Indexes** - Added for common query patterns
5. **Validation Checklists** - Clear "done" criteria for each spec
6. **Effort Estimates** - Realistic timelines for planning

## Recommended Implementation Order

Based on gap priorities and dependencies:

### Phase 1 (Week 1-2): P0 Items
1. [01-content-schema-v2-purist.md](01-content-schema-v2-purist.md) - Foundation
2. [02-translations-and-translators-normalization.md](02-translations-and-translators-normalization.md) - Performance + multi-translation
3. [04-versioning-and-migration-strategy.md](04-versioning-and-migration-strategy.md) - Both parts

**Closes gaps:** 3 (P0), 4 (P0), 5 (P0), 6 (P1)

### Phase 2 (Week 3): P1 Items
- Translator selection UI (part of spec 02)
- User preference persistence

**Enhances:** Multi-translation user experience

### Phase 3 (Post-MVP): P3-P4 Items
1. [05-knowledge-axis-and-session-integration.md](05-knowledge-axis-and-session-integration.md) if needed
2. [03-flexible-content-packages-plan.md](03-flexible-content-packages-plan.md) Phase 3 when audio becomes priority

**Closes gaps:** 1 (P3), 2 (P4)

## Decision Points for User

### 1. Knowledge Axis Timing

**Question:** Implement axis-specific exercises now or defer?

**If MVP:**
- Effort: +2 weeks
- Benefit: Sophisticated multi-dimensional learning from day 1
- Risk: Delays MVP launch

**If Post-MVP:**
- Effort: 0 now (defer to later)
- Benefit: Faster MVP
- Trade-off: Users can't target specific learning dimensions initially

**Recommendation:** Defer to post-MVP unless axis exercises are core differentiator.

### 2. Package System Timing

**Question:** When to implement downloadable packages?

**Phase 2 (Multi-translation):** Recommended for MVP (3-5 days)
**Phase 3 (Full packages):** Post-MVP when audio or app size becomes issue

**Recommendation:** Phase 2 now, Phase 3 later.

## References

- **Audit Document:** [../09-gaps-and-recommendations.md](../09-gaps-and-recommendations.md)
- **Implementation Roadmap:** [README.md](README.md)
- **Architecture Overview:** [00-overview.md](00-overview.md)

---

**Last Updated:** 2025-11-17
**Status:** All gaps mapped to specs
**Coverage:** 100% (6/6 gaps addressed)
