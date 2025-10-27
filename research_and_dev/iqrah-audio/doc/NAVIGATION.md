# IQRAH AUDIO - DOCUMENTATION NAVIGATION

**Purpose**: Guide AI agents and developers to the right documentation module
**Last Updated**: 2025-10-26
**Version**: 2.1 (Maximum Efficiency - Subfolder Optimization)
**Status**: Active - use this as entry point for all documentation queries

---

## 📋 QUICK START

### For AI Agents Starting Fresh
1. Read [Executive Summary](00-executive/summary.md) (5 min overview)
2. Read [Architecture Overview](01-architecture/overview.md) (system design)
3. Pick specific module based on your task

### For Specific Tasks
- **Implementing a module**: Go to `01-architecture/m{N}-{name}.md`
- **Understanding decisions**: Go to `02-implementation/decisions.md`
- **Getting task assignments**: Go to `03-tasks/phase{N}-{name}.md`
- **Looking up technical details**: Go to `04-technical-details/{topic}.md`

---

## 📚 DOCUMENTATION TREE

### 00-EXECUTIVE (Start Here)
- [**summary.md**](00-executive/summary.md) - 5-minute overview, core specs, KPIs

### 01-ARCHITECTURE (Technical Specifications)
- [**overview.md**](01-architecture/overview.md) - 8-module system, data flow, interfaces
- [**m1-preprocessing.md**](01-architecture/m1-preprocessing.md) - Audio loading, VAD, normalization
- [**m2-pitch.md**](01-architecture/m2-pitch.md) - SwiftF0, RMVPE, pitch extraction
- [**m3-phoneme-alignment.md**](01-architecture/m3-phoneme-alignment.md) - Wav2Vec2-BERT, CTC alignment, GOP
- [**m4-tajweed.md**](01-architecture/m4-tajweed.md) - Madd, Ghunnah, Qalqalah validators
- [**m5-voice-quality.md**](01-architecture/m5-voice-quality.md) - OpenSMILE, vibrato, breathiness
- [**m6-prosody.md**](01-architecture/m6-prosody.md) - Rhythm, melody, style analysis
- **m7-comparison-engine/** (subfolder for maximum efficiency)
  - [**overview.md**](01-architecture/m7-comparison-engine/overview.md) - High-level architecture, navigation hub
  - [**orchestrator.md**](01-architecture/m7-comparison-engine/orchestrator.md) - ComparisonEngine class, two-path flow
  - [**gatekeeper-rationale.md**](01-architecture/m7-comparison-engine/gatekeeper-rationale.md) - Why two-stage architecture
  - [**comparison-methods.md**](01-architecture/m7-comparison-engine/comparison-methods.md) - M7.1-M7.4 scoring algorithms
- [**m8-feedback.md**](01-architecture/m8-feedback.md) - Feedback generation, progress tracking

### 02-IMPLEMENTATION (How to Build)
- [**guide.md**](02-implementation/guide.md) - Phase roadmap, AI agent delegation
- [**decisions.md**](02-implementation/decisions.md) - Architecture decisions, rationale
- [**ai-agent-templates.md**](02-implementation/ai-agent-templates.md) - Task templates for AI agents

### 03-TASKS (Concrete Work Items)
- [**overview.md**](03-tasks/overview.md) - Dependency graph, milestones
- [**phase1-offline.md**](03-tasks/phase1-offline.md) - Weeks 1-24, all M1-M8 tasks
- [**phase2-realtime.md**](03-tasks/phase2-realtime.md) - Streaming, optimization, caching
- [**phase3-mobile.md**](03-tasks/phase3-mobile.md) - On-device inference, app development

### 04-TECHNICAL-DETAILS (Deep Dive)
- **algorithms/** (subfolder for maximum efficiency)
  - [**prosody.md**](04-technical-details/algorithms/prosody.md) - Fujisaki, Declination, Tilt implementations
  - [**maqam.md**](04-technical-details/algorithms/maqam.md) - Complete Maqam CNN classifier
  - [**weights.md**](04-technical-details/algorithms/weights.md) - User-adjustable weight profiles
- [**phase2-details.md**](04-technical-details/phase2-details.md) - RT1-RT5 detailed task breakdown
- [**phase3-details.md**](04-technical-details/phase3-details.md) - MB1-MB4 detailed task breakdown
- [**infrastructure.md**](04-technical-details/infrastructure.md) - Docker, Redis, GPU setup

---

## 🎯 USE CASES

### "I need to implement Module M3"
```
doc/NAVIGATION.md (you are here)
  → 01-architecture/m3-phoneme-alignment.md (specifications)
  → 03-tasks/phase1-offline.md (tasks T3.1.1 - T3.5.4)
  → 04-technical-details/code-implementations.md (if you need CTC details)
```

### "What's the overall system architecture?"
```
doc/NAVIGATION.md (you are here)
  → 00-executive/summary.md (quick overview)
  → 01-architecture/overview.md (detailed system design)
```

### "Why did we choose Wav2Vec2-BERT over MMS?"
```
doc/NAVIGATION.md (you are here)
  → 02-implementation/decisions.md (section: "Q: Why Wav2Vec2-BERT over MMS?")
```

### "I need to train the Maqam classifier"
```
doc/NAVIGATION.md (you are here)
  → 01-architecture/m6-prosody.md (Maqam section)
  → 04-technical-details/algorithms/maqam.md (complete training code - 160 lines)
```

### "I need to implement the Comparison Engine orchestrator"
```
doc/NAVIGATION.md (you are here)
  → 01-architecture/m7-comparison-engine/overview.md (high-level)
  → 01-architecture/m7-comparison-engine/orchestrator.md (implementation - 350 lines)
```

### "Why did we choose two-stage architecture for M7?"
```
doc/NAVIGATION.md (you are here)
  → 01-architecture/m7-comparison-engine/gatekeeper-rationale.md (evidence - 120 lines)
```

### "What are my tasks for Week 1?"
```
doc/NAVIGATION.md (you are here)
  → 03-tasks/phase1-offline.md (section: "FIRST SPRINT (Week 1)")
  → 02-implementation/ai-agent-templates.md (task format examples)
```

---

## 📊 DOCUMENT SIZES (for context planning)

| Document                                    | Lines | Words  | Topic                           |
| ------------------------------------------- | ----- | ------ | ------------------------------- |
| **00-executive/summary.md**                 | ~300  | ~2,500 | Overview, KPIs, roadmap         |
| **01-architecture/overview.md**             | ~200  | ~1,500 | System design, data flow        |
| **01-architecture/m1-preprocessing.md**     | ~100  | ~800   | Audio preprocessing             |
| **01-architecture/m2-pitch.md**             | ~120  | ~1,000 | Pitch extraction                |
| **01-architecture/m3-phoneme-alignment.md** | ~500  | ~4,000 | Alignment + ASR gatekeeper      |
| **01-architecture/m4-tajweed.md**           | ~350  | ~2,800 | Tajweed rules                   |
| **01-architecture/m5-voice-quality.md**     | ~180  | ~1,400 | Voice analysis                  |
| **01-architecture/m6-prosody.md**           | ~280  | ~2,200 | Prosody features                |
| **01-architecture/m7-comparison-engine/**   | -     | -      | **Subfolder (4 files)**         |
| → overview.md                               | 116   | ~900   | High-level, navigation hub      |
| → orchestrator.md                           | 452   | ~3,600 | ComparisonEngine implementation |
| → gatekeeper-rationale.md                   | 117   | ~900   | Two-stage architecture evidence |
| → comparison-methods.md                     | 175   | ~1,400 | M7.1-M7.4 scoring               |
| **01-architecture/m8-feedback.md**          | ~200  | ~1,600 | User feedback                   |
| **02-implementation/guide.md**              | ~280  | ~2,200 | How to build                    |
| **02-implementation/decisions.md**          | ~180  | ~1,400 | Why we chose X                  |
| **02-implementation/ai-agent-templates.md** | ~150  | ~1,200 | Task formats                    |
| **03-tasks/overview.md**                    | ~100  | ~800   | Dependency graph                |
| **03-tasks/phase1-offline.md**              | ~200  | ~1,600 | Week 1-24 tasks                 |
| **03-tasks/phase2-realtime.md**             | ~350  | ~2,800 | Streaming tasks                 |
| **03-tasks/phase3-mobile.md**               | ~300  | ~2,400 | Mobile tasks                    |
| **04-technical-details/algorithms/**        | -     | -      | **Subfolder (3 files)**         |
| → prosody.md                                | 302   | ~2,400 | Fujisaki, Declination, Tilt     |
| → maqam.md                                  | 170   | ~1,400 | Maqam CNN classifier            |
| → weights.md                                | 116   | ~900   | User weight profiles            |
| **04-technical-details/phase2-details.md**  | ~306  | ~2,400 | RT1-RT5 tasks                   |
| **04-technical-details/phase3-details.md**  | ~264  | ~2,100 | MB1-MB4 tasks                   |
| **04-technical-details/infrastructure.md**  | ~159  | ~1,300 | DevOps, Docker                  |

**Total**: ~5,200 lines, ~38,000 words across **26 files** (v2.1 - Maximum Efficiency)

---

## 🔍 SEARCH TIPS FOR AI AGENTS

### By Topic
- **Accuracy targets**: `00-executive/summary.md` (Core Specs section)
- **Latency targets**: `01-architecture/overview.md` + each module doc
- **Dependencies**: Each module doc has "Dependencies" section
- **Testing**: `04-technical-details/validation.md`
- **Cost estimates**: `00-executive/summary.md` (Roadmap section)

### By Module (M1-M8)
- **Input/Output schemas**: `01-architecture/m{N}-{name}.md` (top of file)
- **Implementation code**: `01-architecture/m{N}-{name}.md` (code blocks)
- **Tasks breakdown**: `03-tasks/phase1-offline.md` (search "M{N}")

### By Phase
- **Phase 1 (Offline)**: `03-tasks/phase1-offline.md`
- **Phase 2 (Real-time)**: `03-tasks/phase2-realtime.md`
- **Phase 3 (Mobile)**: `03-tasks/phase3-mobile.md`

---

## 🚫 DEPRECATED DOCS (do not use)

The following files in `doc/ai-iqrah-docs/` are deprecated and replaced by this modular structure:

- ❌ `AI_IQRAH_SOTA_ARCHITECTURE.md` (2300 lines) → Split into `01-architecture/*.md`
- ❌ `AI_IQRAH_IMPLEMENTATION_GUIDE.md` (523 lines) → Split into `02-implementation/*.md`
- ❌ `AI_IQRAH_SUPPLEMENTARY_DETAILS.md` (1290 lines) → Split into `04-technical-details/*.md`
- ❌ `AI_IQRAH_TASK_DECOMPOSITION.md` (537 lines) → Split into `03-tasks/*.md`
- ✅ `AI_IQRAH_EXECUTIVE_SUMMARY.md` (308 lines) → Moved to `00-executive/summary.md`
- ✅ `AI_OPTIMIZATION_SUMMARY.md` (193 lines) → Merged into relevant sections

**Why deprecated**: Too large for AI agent context windows (2300 lines max). Modular docs load only what's needed (~100-500 lines per file).

---

## 📝 MAINTENANCE

### Updating Documentation
- **Add new module**: Create `01-architecture/m{N}-{name}.md`, update this navigation
- **Add new task**: Update appropriate `03-tasks/phase{N}-{name}.md`
- **Add code example**: Update `04-technical-details/code-implementations.md`
- **Change decision**: Update `02-implementation/decisions.md`

### Version Control
- **Current version**: v1.0 (2025-10-25)
- **Next review**: After Phase 1 completion (Week 24)
- **Stability commitment**: 3-year architecture (2025-2028)

---

## 🤖 AI AGENT INSTRUCTIONS

When you receive a task:

1. **Start here** (`NAVIGATION.md`)
2. **Identify module** (M1-M8) or phase (P1-P3)
3. **Load specific doc** (don't load all docs)
4. **Check dependencies** in that doc
5. **Implement** following specifications
6. **Return to NAVIGATION.md** for next task

**Example workflow**:
```
User: "Implement pitch extraction"
Agent:
  1. Read NAVIGATION.md (this file)
  2. Go to 01-architecture/m2-pitch.md
  3. Read specifications (150 lines, manageable)
  4. Implement SwiftF0 integration
  5. Done - return to NAVIGATION.md
```

**DO NOT**:
- ❌ Load all documentation at once (context overflow)
- ❌ Read deprecated docs in `ai-iqrah-docs/`
- ❌ Guess file locations (use this navigation)

---

## 🎓 LEARNING PATH

### For New Developers
Day 1: `00-executive/summary.md` + `01-architecture/overview.md`
Day 2-3: Read all `01-architecture/m{1-8}-*.md` modules
Day 4-5: `02-implementation/guide.md` + `decisions.md`
Week 2+: Start implementing tasks from `03-tasks/phase1-offline.md`

### For Experienced ML Engineers
Skip to: `01-architecture/overview.md` → Pick module → Implement

### For Product Managers
Read: `00-executive/summary.md` + `02-implementation/guide.md`

---

## 📞 SUPPORT

**Questions about**:
- Architecture design → Check `01-architecture/overview.md` or `02-implementation/decisions.md`
- Specific algorithm → Check `04-technical-details/code-implementations.md`
- Task priority → Check `03-tasks/overview.md` (dependency graph)
- Testing strategy → Check `04-technical-details/validation.md`

**Still stuck?**
- Re-read this NAVIGATION.md
- Check if you're reading the right module doc
- Verify you're not using deprecated docs

---

**Remember**: This navigation structure exists to **prevent context overflow**. Load only what you need for your current task.
