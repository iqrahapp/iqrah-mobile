# Sprint 7: Stability & Foundation - Complete Planning Documents

**Mission:** Transform Iqrah from a working prototype to a production-ready, maintainable, and scalable system.

---

## 📋 Documents Overview

This directory contains comprehensive planning for the most critical sprint in the Iqrah project. Read them in order:

### 1. [Current State Analysis](./00-CURRENT-STATE-ANALYSIS.md)
**What's wrong and why it matters**

- Identifies all architectural debt
- Quantifies technical issues (59 SQL queries embedded, 1,378-line god object)
- Explains performance bottlenecks
- Lists success metrics

**Key Takeaway:** ~55 story points of technical debt must be addressed

---

### 2. [Library Research](./01-LIBRARY-RESEARCH.md)
**The tools that will save us months of work**

- **SQLx** for compile-time checked queries
- **Mockall** for dependency injection
- **Clap** for CLI tooling
- **PropTest** for property-based testing

**Key Takeaway:** Use battle-tested libraries, not custom solutions

---

### 3. [Database Schema Design](./02-DATABASE-SCHEMA-DESIGN.md)
**The two-database architecture**

#### `content.db` (Immutable)
- Qur'anic knowledge graph
- Structured metadata tables (not key-value!)
- Pre-computed importance scores
- ~200-300 MB (complete Qur'an)

#### `user.db` (Mutable)
- User progress (FSRS states, energy)
- Review history & propagation log
- Session state & stats
- ~10-20 MB (active user)

**Key Innovations:**
- Lazy user data generation (90% size reduction)
- Dedicated metadata tables (2x query performance)
- Migration framework support

**Key Takeaway:** Separation of concerns enables safe updates and easy backups

---

### 4. [Architecture Blueprint](./03-ARCHITECTURE-BLUEPRINT.md)
**The clean, modular codebase structure**

#### New Structure: 4 Crates
```
iqrah-core      # Pure business logic (zero DB dependencies)
iqrah-storage   # SQLx repositories
iqrah-api       # Flutter bridge (FRB)
iqrah-cli       # Developer tool
```

#### Hexagonal Architecture
- **Domain logic** = Core
- **Ports** = Traits (`ContentRepository`, `UserRepository`)
- **Adapters** = SQLx implementations

**Key Principles:**
- Dependency Inversion
- Single Responsibility
- Testability First

**Key Takeaway:** Domain logic is 100% testable without database

---

### 5. [Migration Strategy](./04-MIGRATION-STRATEGY.md)
**Step-by-step execution plan**

#### Timeline: 2-2.5 Weeks
| Phase | Duration | Focus |
|-------|----------|-------|
| Workspace setup | 0.5 days | Cargo structure |
| Domain extraction | 1.5 days | Move models to iqrah-core |
| Storage layer | 3 days | SQLx repos + migrations |
| API update | 1.5 days | Wire new services |
| Data migration | 1 day | One-time user data move |
| Testing | 1.5 days | Integration & E2E |
| Cleanup | 1 day | Remove old code |

**Rollback Plan:** Git tags + database backups at each phase

**Key Takeaway:** Zero downtime, zero data loss

---

### 6. [Testing Strategy](./05-TESTING-STRATEGY.md)
**Achieving 80%+ coverage**

#### Test Pyramid
```
E2E (5%)        ← Integration tests in Flutter
Integration (20%) ← Real SQLite, test fixtures
Unit (75%)      ← Mocked dependencies, fast
```

**Coverage Goals:**
- iqrah-core: 90%+
- iqrah-storage: 80%+
- iqrah-api: 60%+

**Key Takeaway:** Test-first development, not test-last

---

### 7. [Research Pipeline Refactor](./06-RESEARCH-PIPELINE-REFACTOR.md)
**Transform R&D from notebooks to production Python package**

#### Current Problems (Jupyter Notebook)
- ❌ Depends on offline web API (cached only)
- ❌ Hardcodes metadata INTO knowledge graph
- ❌ Not reproducible or testable
- ❌ Cannot be automated

#### New Approach (Python Package)
- ✅ Proper `pyproject.toml` structure
- ✅ CLI tool: `iqrah-kg build`
- ✅ Uses Tarteel SQLite (offline, no API)
- ✅ Separates graph from metadata
- ✅ 80%+ test coverage
- ✅ CI/CD pipeline

**Outputs:**
1. `knowledge_graph.cbor.zst` - Pure graph structure
2. `content.db` - SQLite with all metadata

**Key Takeaway:** R&D pipeline must be as production-ready as the app itself

#### Test Pyramid
```
E2E (5%)        ← Integration tests in Flutter
Integration (20%) ← Real SQLite, test fixtures
Unit (75%)      ← Mocked dependencies, fast
```

**Tools:**
- `mockall` for mocking
- `proptest` for property-based testing
- `rstest` for fixtures
- `cargo tarpaulin` for coverage

**Coverage Goals:**
- iqrah-core: 90%+
- iqrah-storage: 80%+
- iqrah-api: 60%+

**Key Takeaway:** Test-first development, not test-last

---

## 🎯 Sprint 7 Success Criteria

### Functional Requirements ✅
- [ ] All existing features work identically
- [ ] No regressions in user experience
- [ ] Stats, sessions, propagation all functional
- [ ] Two databases (content.db + user.db)

### Non-Functional Requirements ✅
- [ ] 80%+ test coverage
- [ ] Session generation < 50ms (2x faster)
- [ ] user.db 90% smaller for new users
- [ ] Zero SQL in business logic
- [ ] All queries compile-time checked

### Code Quality ✅
- [ ] Dependency injection throughout
- [ ] Mockable repositories
- [ ] CLI tool for debugging
- [ ] Migration framework

---

## 🚀 Why This Matters

### Current State: Prototype ❌
- Cannot test without full app
- Cannot update content without risk
- Cannot add features without breaking things
- Cannot deploy confidently

### Future State: Production-Ready ✅
- Unit test any component in isolation
- Update content.db independently
- Add features with confidence
- Deploy with migration safety

---

## 📊 Estimated Effort

| Category | Story Points |
|----------|--------------|
| Workspace setup | 1 |
| Domain refactor | 8 |
| Storage layer | 13 |
| API update | 5 |
| Data migration | 5 |
| Testing infrastructure | 8 |
| CLI tool | 3 |
| Documentation | 2 |

**Total: ~45 Story Points**

**Timeline:** 2-3 weeks of focused work

---

## 🔄 After Sprint 7

### Immediate Benefits
1. **Velocity:** 3x faster feature development
2. **Confidence:** Regression tests catch bugs
3. **Maintainability:** Clear module boundaries
4. **Scalability:** Performance optimizations possible

### Unblocked Roadmap
- ✅ Sprint 8: Audio analysis (complex feature, needs solid foundation)
- ✅ Sprint 9: Advanced exercise variants (requires testable scheduler)
- ✅ Sprint 10: Multi-user support (needs separation of concerns)
- ✅ Sprint 11: Offline sync (requires migration framework)

---

## 📝 Next Steps

### For the Developer (You)
1. **Review all documents** (2-3 hours)
2. **Approve architecture decisions**
3. **Gather Qur'an metadata** (translations, audio URLs)
4. **Prepare content.db schema** with proper metadata tables

### For the Agent (Me)
1. **Wait for your approval** on this plan
2. **Execute migration** following 04-MIGRATION-STRATEGY.md
3. **Implement testing** following 05-TESTING-STRATEGY.md
4. **Deliver production-ready Sprint 7**

---

## 💡 Key Insights from Planning

### What We Learned
1. **Technical Debt is Manageable:** 55 SP sounds scary, but with a plan it's achievable
2. **Libraries Save Time:** SQLx, Mockall, etc. eliminate 100+ hours of work
3. **Two DBs is Critical:** Can't ship audio MVP without safe content updates
4. **Testing Prevents Chaos:** 80% coverage is the difference between toy and product

### What Surprised Us
1. **90% size reduction** for user.db (lazy generation insight)
2. **2x performance gain** from metadata restructuring
3. **Compile-time SQL checking** eliminates entire class of bugs
4. **CLI tool** becomes critical debugging asset

---

## 🎓 Lessons for Future Sprints

### Architectural Principles (Permanent)
1. **Separate concerns** (content ≠ user data)
2. **Inject dependencies** (never global singletons)
3. **Test first** (not test later)
4. **Choose libraries** (over custom code)

### Process Principles (Permanent)
1. **Plan thoroughly** before coding
2. **Migrate incrementally** (not big bang)
3. **Validate continuously** (rollback plan ready)
4. **Document decisions** (for future you)

---

## 📚 Additional Resources

### External Reading
- [Hexagonal Architecture](https://alistair.cockburn.us/hexagonal-architecture/)
- [SQLx GitHub](https://github.com/launchbadge/sqlx)
- [Rust Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Test Pyramid](https://martinfowler.com/articles/practical-test-pyramid.html)

### Internal Documentation (Post-Sprint 7)
- `ARCHITECTURE.md` (new design)
- `TESTING.md` (how to run tests)
- `CLI.md` (developer tool usage)
- `MIGRATIONS.md` (schema evolution)

---

## ✅ Approval Required

**Status:** 📋 Planning Complete, Awaiting Approval

**Questions for Review:**
1. Is the two-database design acceptable?
2. Are the library choices (SQLx, Mockall, etc.) approved?
3. Is the 2-3 week timeline feasible?
4. Any concerns about the migration strategy?

**Once approved, Sprint 7 execution begins immediately.**

---

*Last Updated: 2025-10-04*
*Planning Time Investment: ~6 hours*
*Expected ROI: 10x development velocity*
