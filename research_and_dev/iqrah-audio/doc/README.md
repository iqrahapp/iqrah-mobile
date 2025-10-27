# IQRAH Audio - Documentation Repository

**Last Updated**: 2025-10-25
**Documentation Version**: 2.0 (Modular)
**Project Phase**: Phase 1 - Offline E2E

---

## 🎯 Quick Start

**START HERE**: [NAVIGATION.md](NAVIGATION.md) - Your GPS for all documentation

### For AI Agents
```
1. Read: NAVIGATION.md
2. Identify: Which module/phase you need
3. Load: Only the specific file (100-500 lines)
4. Implement: Follow specifications
5. Return: To NAVIGATION.md for next task
```

### For Developers
```
Day 1: 00-executive/summary.md + 01-architecture/overview.md
Week 1: Read all 01-architecture/m{1-8}-*.md modules
Week 2+: Start implementing tasks from 03-tasks/phase1-offline.md
```

---

## 📁 Directory Structure

```
doc/
├── NAVIGATION.md                          ⭐ START HERE - Documentation GPS
│
├── 00-executive/                          📊 5-Minute Overview
│   └── summary.md                         • Core specs, KPIs, roadmap
│
├── 01-architecture/                       🏗️ Technical Specifications
│   ├── overview.md                        • System design, data flow
│   ├── m1-preprocessing.md                • Audio loading, VAD, normalization
│   ├── m2-pitch.md                        • SwiftF0, RMVPE pitch extraction
│   ├── m3-phoneme-alignment.md            • Wav2Vec2-BERT, CTC, ASR gatekeeper
│   ├── m4-tajweed.md                      • Madd, Ghunnah, Qalqalah validators
│   ├── m5-voice-quality.md                • OpenSMILE, vibrato, breathiness
│   ├── m6-prosody.md                      • Rhythm, melody, style analysis
│   ├── m7-comparison-engine/              ⚡ SUBFOLDER (max efficiency)
│   │   ├── overview.md                    • High-level architecture
│   │   ├── orchestrator.md                • ComparisonEngine class (350 lines)
│   │   ├── gatekeeper-rationale.md        • Two-stage design evidence (120 lines)
│   │   └── comparison-methods.md          • M7.1-M7.4 scoring (170 lines)
│   └── m8-feedback.md                     • Feedback generation, progress tracking
│
├── 02-implementation/                     🛠️ How to Build
│   ├── guide.md                           • Phase roadmap, resource allocation
│   ├── decisions.md                       • Architecture decisions, rationale
│   └── ai-agent-templates.md              • Task templates for AI agents
│
├── 03-tasks/                              ✅ Concrete Work Items
│   ├── overview.md                        • Dependency graph, milestones
│   ├── phase1-offline.md                  • Weeks 1-24, all M1-M8 tasks
│   ├── phase2-realtime.md                 • Months 7-12, streaming tasks
│   └── phase3-mobile.md                   • Months 13-18, mobile tasks
│
├── 04-technical-details/                  🔬 Deep Dive
│   ├── algorithms/                        ⚡ SUBFOLDER (max efficiency)
│   │   ├── prosody.md                     • Fujisaki, Declination, Tilt (290 lines)
│   │   ├── maqam.md                       • Maqam CNN classifier (160 lines)
│   │   └── weights.md                     • User weight profiles (115 lines)
│   ├── phase2-details.md                  • RT1-RT5 detailed task breakdown
│   ├── phase3-details.md                  • MB1-MB4 detailed task breakdown
│   └── infrastructure.md                  • Docker, Redis, GPU setup
│
└── ai-iqrah-docs/                         ⚠️ DEPRECATED (Legacy)
    └── *.md                               • Old monolithic files (do not use)
```

---

## 📊 Documentation Statistics

### New Modular Structure (v2.1 - Maximum Efficiency)

| Category                      | Files             | Total Lines | Total Size |
| ----------------------------- | ----------------- | ----------- | ---------- |
| **00-executive**              | 1                 | 309         | 8.5 KB     |
| **01-architecture**           | 9 + 4 (subfolder) | 2,417       | 82 KB      |
| **02-implementation**         | 3                 | 773         | 19 KB      |
| **03-tasks**                  | 4                 | 1,080       | 31 KB      |
| **04-technical-details**      | 3 + 3 (subfolder) | 1,447       | 42 KB      |
| **NAVIGATION.md + README.md** | 2                 | 552         | 24 KB      |
| **TOTAL**                     | **26**            | **6,578**   | **206 KB** |

### Old Monolithic Structure (v1.0 - Deprecated)

| File                              | Lines | Size   | Status                                  |
| --------------------------------- | ----- | ------ | --------------------------------------- |
| AI_IQRAH_SOTA_ARCHITECTURE.md     | 2,300 | 85 KB  | ❌ Replaced by 01-architecture/*.md      |
| AI_IQRAH_SUPPLEMENTARY_DETAILS.md | 1,290 | 42 KB  | ❌ Replaced by 04-technical-details/*.md |
| AI_IQRAH_TASK_DECOMPOSITION.md    | 537   | 18 KB  | ❌ Replaced by 03-tasks/*.md             |
| AI_IQRAH_IMPLEMENTATION_GUIDE.md  | 523   | 15 KB  | ❌ Replaced by 02-implementation/*.md    |
| AI_IQRAH_EXECUTIVE_SUMMARY.md     | 308   | 8.5 KB | ✅ Moved to 00-executive/summary.md      |
| AI_OPTIMIZATION_SUMMARY.md        | 193   | 6.0 KB | ❌ Merged into relevant sections         |
| **TOTAL**                         | 5,151 | 175 KB | **Deprecated**                          |

---

## 🎓 Key Improvements (v1.0 → v2.1)

### ✅ v2.0: Modular Structure (22 files)
- Split 6 monolithic files into 22 focused modules
- Each module <730 lines for AI agent efficiency
- GPS navigation system (NAVIGATION.md)

### ✅ v2.1: Subfolder Optimization (26 files) - **MAXIMUM EFFICIENCY**
- Split 2 largest files (729 + 551 lines) into subfolders
- **66% reduction** in context load for targeted tasks
- Largest file now **350 lines** (was 729)

**Example efficiency gains:**
- Implement orchestrator: 729 lines → **350 lines** (52% reduction)
- Implement Maqam CNN: 551 lines → **160 lines** (71% reduction)
- Research gatekeeper design: 729 lines → **120 lines** (83% reduction)

### Benefits by Role

1. **AI Agent Efficiency**
   - v1.0: Load 2,300-line file → Context overflow risk
   - v2.0: Load specific 300-line module → Good
   - v2.1: Load targeted 120-350 line file → **OPTIMAL** ⚡

2. **Developer Navigation**
   - Old: Search through massive files
   - New: GPS + subfolders → Direct to specific component

3. **Maintainability**
   - Old: Edit 2,300-line file (risky)
   - New: Edit focused 100-350 line file (safe)

5. **Cross-Referencing**
   - Old: Manual searching, no hyperlinks
   - New: Navigation links in every file

---

## 🗺️ Navigation Patterns

### Pattern 1: Module Implementation
```
Task: "Implement pitch extraction"
Path: NAVIGATION.md → 01-architecture/m2-pitch.md → (implement) → Done
```

### Pattern 2: Architecture Decision Lookup
```
Question: "Why Wav2Vec2-BERT over MMS?"
Path: NAVIGATION.md → 02-implementation/decisions.md → (section Q&A) → Answer
```

### Pattern 3: Task Assignment
```
Sprint: "Week 1 tasks"
Path: NAVIGATION.md → 03-tasks/phase1-offline.md → (FIRST SPRINT) → Tasks
```

### Pattern 4: Algorithm Deep Dive
```
Need: "Fujisaki model implementation"
Path: NAVIGATION.md → 04-technical-details/code-implementations.md → (section 1.1) → Code
```

---

## 📋 File Purpose Reference

### 00-Executive
- **summary.md**: Quick overview for stakeholders, KPIs, 3-phase roadmap

### 01-Architecture (Technical Specs)
- **overview.md**: System architecture, 8-module design, data flow
- **m1-preprocessing.md**: Audio I/O, VAD, normalization, quality checks
- **m2-pitch.md**: SwiftF0 + RMVPE pitch extraction
- **m3-phoneme-alignment.md**: Wav2Vec2-BERT training, CTC alignment, ASR gatekeeper
- **m4-tajweed.md**: Madd/Ghunnah/Qalqalah validators
- **m5-voice-quality.md**: OpenSMILE, vibrato, breathiness features
- **m6-prosody.md**: Rhythm (nPVI, DTW), Melody (Fujisaki, Maqam)
- **m7-comparison-engine.md**: Orchestrator with two-path gating logic
- **m8-feedback.md**: User feedback generation, progress tracking

### 02-Implementation (Build Guide)
- **guide.md**: Phase roadmap, critical path, resource allocation
- **decisions.md**: Why we chose X over Y (rationale)
- **ai-agent-templates.md**: Task format templates for AI agents

### 03-Tasks (Work Breakdown)
- **overview.md**: Dependency graph (Mermaid), all milestones
- **phase1-offline.md**: Weeks 1-24 tasks, templates, Week 1 sprint
- **phase2-realtime.md**: Months 7-12 tasks, streaming overview
- **phase3-mobile.md**: Months 13-18 tasks, mobile overview

### 04-Technical-Details (Deep Dive)
- **code-implementations.md**: Complete algorithms with full Python code
- **phase2-details.md**: RT1-RT5 detailed task breakdown (48 tasks)
- **phase3-details.md**: MB1-MB4 detailed task breakdown (46 tasks)
- **infrastructure.md**: Latency targets, Redis, Docker config

---

## 🔄 Migration Guide (v1.0 → v2.0)

### For Existing AI Agents

**Old Workflow**:
```python
# Agent loads entire architecture doc
doc = load("ai-iqrah-docs/AI_IQRAH_SOTA_ARCHITECTURE.md")  # 2,300 lines
# Context overflow risk!
```

**New Workflow**:
```python
# Agent navigates to specific module
nav = load("doc/NAVIGATION.md")  # 285 lines
module = load("doc/01-architecture/m3-phoneme-alignment.md")  # 345 lines
# Efficient, focused context
```

### For Developers

**Old**: Bookmark specific line numbers in giant files
**New**: Bookmark specific modular files

**Old**: Search within 2,300-line files for relevant sections
**New**: Go directly to the right module via NAVIGATION.md

### For Documentation Updates

**Old**: Edit 2,300-line file, risk breaking other sections
**New**: Edit focused 100-500 line module, isolated changes

---

## 🚀 Quick Reference

### Most Frequently Needed Files

1. **NAVIGATION.md** - Always start here
2. **00-executive/summary.md** - For quick overview
3. **01-architecture/overview.md** - For system design
4. **03-tasks/phase1-offline.md** - For current work items
5. **02-implementation/ai-agent-templates.md** - For task creation

### By Role

**AI Agents**: NAVIGATION.md → Specific module → Implement → Done
**Developers**: 00-executive → 01-architecture → 03-tasks → Code
**Product Managers**: 00-executive/summary.md → 02-implementation/guide.md
**Architects**: 01-architecture/overview.md → decisions.md

---

## 📞 Support

**Can't find something?**
1. Check [NAVIGATION.md](NAVIGATION.md) first
2. Use file tree above to locate the right category
3. Search within the specific category folder

**Documentation Issues?**
- File location: `doc/NAVIGATION.md` (entry point)
- Version: 2.0 (Modular)
- Last updated: 2025-10-25

---

## 🔒 Stability Commitment

**Architecture**: Stable for 3 years (2025-2028)
**Documentation Structure**: This modular layout is stable
**File Locations**: Will not change during Phase 1
**Next Review**: After Phase 1 completion (Week 24)

---

**Remember**: Always start with [NAVIGATION.md](NAVIGATION.md) - it's your GPS for this entire documentation system.
