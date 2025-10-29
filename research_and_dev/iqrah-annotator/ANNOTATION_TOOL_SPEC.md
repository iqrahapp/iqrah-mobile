# Tajweed Annotation Tool - Technical Specification

**Purpose**: Web-based annotation tool for generating training data for Tajweed (Quranic recitation rules) validation system.

**Context**: Building validation dataset for 13+ Tajweed rules. Need experts/students to record violations (anti-patterns) with precise frame-level annotations.

---

## 1. SYSTEM OVERVIEW

### 1.1 Core Functions
1. **Record**: User records recitation with anti-pattern (e.g., weak ghunnah)
2. **Annotate**: Visual waveform editor with frame-level timestamps
3. **Review**: Waveform visualization + playback + manual annotation editing
4. **Export**: Audio + annotations → JSON/Parquet for training/validation
5. **Filter & Search**: Tag-based filtering (AND logic) for ayahs by surah, patterns, rules
6. **Stats & Priority**: Dashboard with sortable stats showing priority patterns/ayahs/surahs

### 1.2 User Roles
- **Student**: Record samples, view own annotations
- **Expert**: Record + verify others' annotations
- **Admin**: Manage rules, anti-patterns, export datasets

### 1.3 Existing Tools Research (REUSE > REINVENT)

**IMPORTANT**: Before implementing custom solutions, evaluate these proven audio annotation tools:

#### Open-Source Audio Annotation Tools
1. **Audacity + Label Track**
   - Pros: Mature, widely used, supports region annotations
   - Cons: Desktop-only, not web-based, no multi-user
   - Reuse Potential: Can export to Audacity label format for cross-compatibility

2. **Praat** (phonetics research standard)
   - Pros: Industry standard for phonetic analysis, TextGrid format
   - Cons: Desktop-only, steep learning curve
   - Reuse Potential: Import/export TextGrid format for phoneme boundaries

3. **label-studio** (Heartex)
   - Pros: Web-based, multi-user, supports audio + ML integration
   - Cons: Generic (not speech-specific), requires configuration
   - **Reuse Potential**: HIGH - Can be customized for Tajweed use case

4. **WebMAUS** (Munich Automatic Segmentation)
   - Pros: Web-based forced alignment for speech
   - Cons: Limited to alignment, not annotation
   - Reuse Potential: Can pre-segment audio using phoneme alignment

5. **ELAN** (Max Planck Institute)
   - Pros: Annotation tiers, video + audio, linguistic focus
   - Cons: Desktop-only, Java-based
   - Reuse Potential: Export format compatible with linguistic corpora

6. **Sonic Visualiser**
   - Pros: Spectrogram visualization, plugin support
   - Cons: Desktop-only, research-focused
   - Reuse Potential: Can generate spectrogram layers

#### Recommended Approach
- **Primary**: Use **WaveSurfer.js** (web library) + custom React UI
  - Reason: Web-based, lightweight, mobile-friendly, integrates with existing pipeline
- **Secondary**: Export to **Audacity Label** + **Praat TextGrid** formats for compatibility
- **Integration**: Consider **label-studio** as backend if project scales beyond MVP

### 1.4 Tech Stack (Best Practices)
- **Frontend**: React + TypeScript
- **Waveform**: WaveSurfer.js (industry standard, web-based Audacity alternative)
- **Audio**: Web Audio API (recording), Tone.js (playback)
- **UI**: Material-UI or Ant Design (professional, accessible, intuitive)
- **DB**: SQLite (MVP), PostgreSQL (production scale)
- **Backend**: FastAPI (Python) - integrates with existing ML pipeline
- **Export**: Pandas → Parquet (efficient columnar format for ML)
- **Optional**: label-studio integration for advanced multi-annotator workflows

---

## 2. DATABASE SCHEMA

### 2.1 Existing: `qpc_words` (Read-Only Reference)
```sql
-- From data/qpc-hafs-tajweed.db
CREATE TABLE words (
    id INTEGER PRIMARY KEY,
    location TEXT,      -- Format: "surah:ayah:word" (e.g., "89:27:3")
    surah INTEGER,
    ayah INTEGER,
    word INTEGER,
    text TEXT           -- Uthmani script with HTML tags for rules
                       -- Example: "ٱلۡمُ<rule class=qalaqah>طۡ</rule>مَئِ<rule class=ghunnah>نّ</rule>َةُ"
);
-- 83,668 rows total
```

**Rule Classes Found** (from HTML tags):
```
qalaqah (3,733), ghunnah (4,907), madda_normal (7,471),
ham_wasl, laam_shamsiyah, ikhafa, idgham_ghunnah, etc.
```

### 2.2 New: `annotation.db` Schema

```sql
-- Rules Definition
CREATE TABLE rules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,          -- e.g., "ghunnah", "qalqalah", "madd"
    description TEXT,                    -- Human-readable description
    category TEXT,                       -- "tier1_baseline", "tier2_specialized"
    target_accuracy REAL,                -- Target accuracy (e.g., 0.90 for 90%)
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Anti-Patterns (Violations to Record)
CREATE TABLE anti_patterns (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    rule_id INTEGER NOT NULL,
    name TEXT NOT NULL,                  -- e.g., "no-ghunnah", "weak-ghunnah", "overly-strong-ghunnah"
    description TEXT,
    severity TEXT,                       -- "critical", "moderate", "minor"
    FOREIGN KEY (rule_id) REFERENCES rules(id),
    UNIQUE(rule_id, name)
);

-- Word Instances (Link to QPC Database)
CREATE TABLE word_instances (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    qpc_location TEXT NOT NULL,          -- "89:27:3" from qpc_words.location
    rule_id INTEGER NOT NULL,
    has_rule BOOLEAN DEFAULT 1,          -- 1 if word has this rule
    phoneme_indices TEXT,                -- JSON array: which phonemes have rule
                                        -- Example: "[2, 5]" means phonemes at index 2 and 5
    FOREIGN KEY (rule_id) REFERENCES rules(id)
);

-- Users
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE,
    role TEXT NOT NULL,                  -- "student", "expert", "admin"
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Recording Sessions
CREATE TABLE recordings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    word_instance_id INTEGER NOT NULL,
    anti_pattern_id INTEGER NOT NULL,
    audio_path TEXT NOT NULL,            -- filesystem path: "audio/{user_id}/{recording_id}.wav"
    duration_sec REAL,
    sample_rate INTEGER DEFAULT 16000,
    status TEXT DEFAULT 'draft',         -- "draft", "submitted", "verified", "rejected"
    verified_by INTEGER,                 -- user_id of expert who verified
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (word_instance_id) REFERENCES word_instances(id),
    FOREIGN KEY (anti_pattern_id) REFERENCES anti_patterns(id),
    FOREIGN KEY (verified_by) REFERENCES users(id)
);

-- Frame-Level Annotations
CREATE TABLE annotations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    recording_id INTEGER NOT NULL,
    start_sec REAL NOT NULL,             -- Start time in seconds
    end_sec REAL NOT NULL,               -- End time in seconds
    label TEXT NOT NULL,                 -- e.g., "weak-ghunnah-onset", "burst-missing"
    confidence REAL,                     -- Optional: annotator confidence (0-1)
    notes TEXT,                          -- Optional: free-text notes
    annotator_id INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (recording_id) REFERENCES recordings(id),
    FOREIGN KEY (annotator_id) REFERENCES users(id)
);

-- Tags (for flexible filtering)
CREATE TABLE tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,              -- e.g., "surah-89", "pattern-ghunnah-weak", "priority-high"
    category TEXT,                          -- "surah", "pattern", "priority", "custom"
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Word Instance Tags (many-to-many)
CREATE TABLE word_instance_tags (
    word_instance_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (word_instance_id, tag_id),
    FOREIGN KEY (word_instance_id) REFERENCES word_instances(id),
    FOREIGN KEY (tag_id) REFERENCES tags(id)
);

-- Recording Tags (for filtering recordings)
CREATE TABLE recording_tags (
    recording_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (recording_id, tag_id),
    FOREIGN KEY (recording_id) REFERENCES recordings(id),
    FOREIGN KEY (tag_id) REFERENCES tags(id)
);

-- Statistics Cache (for dashboard with priority metrics)
CREATE TABLE stats (
    rule_id INTEGER,
    anti_pattern_id INTEGER,
    count_total INTEGER DEFAULT 0,
    count_verified INTEGER DEFAULT 0,
    count_pending INTEGER DEFAULT 0,
    priority_score REAL DEFAULT 0.0,         -- Calculated: (target - current_coverage) * severity_weight
    current_coverage REAL DEFAULT 0.0,       -- verified / (target * total_instances)
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (rule_id, anti_pattern_id),
    FOREIGN KEY (rule_id) REFERENCES rules(id),
    FOREIGN KEY (anti_pattern_id) REFERENCES anti_patterns(id)
);

-- Surah Statistics (for surah-level priority)
CREATE TABLE surah_stats (
    surah INTEGER PRIMARY KEY,
    rule_id INTEGER,
    count_words INTEGER DEFAULT 0,           -- Total words with this rule in surah
    count_recorded INTEGER DEFAULT 0,        -- Words with recordings
    count_verified INTEGER DEFAULT 0,        -- Words with verified recordings
    coverage REAL DEFAULT 0.0,               -- verified / count_words
    priority_score REAL DEFAULT 0.0,         -- Higher = needs more data
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (rule_id) REFERENCES rules(id)
);
```

---

## 3. DATA FORMATS

### 3.1 Audio Format
- **Storage**: WAV, 16kHz, mono, 16-bit PCM
  - Why: Standard for speech ML, lossless, compatible with ML pipelines
  - Compression: Use FLAC for archival (convert to WAV for training)
- **Filesystem**: `audio/{user_id}/{recording_id}.wav`
- **DB Reference**: Store relative path only

### 3.2 Annotation Export Format

**JSON** (human-readable, debugging):
```json
{
  "version": "1.0",
  "export_date": "2025-10-28T12:00:00Z",
  "recordings": [
    {
      "id": 123,
      "audio_path": "audio/5/123.wav",
      "duration_sec": 4.2,
      "sample_rate": 16000,
      "user": {"id": 5, "username": "student1", "role": "student"},
      "qpc_location": "89:27:3",
      "rule": "ghunnah",
      "anti_pattern": "weak-ghunnah",
      "status": "verified",
      "verified_by": {"id": 2, "username": "expert1"},
      "annotations": [
        {
          "id": 456,
          "start_sec": 1.2,
          "end_sec": 1.5,
          "label": "weak-ghunnah-onset",
          "confidence": 0.9,
          "notes": "Nasal resonance too low"
        }
      ]
    }
  ]
}
```

**Parquet** (ML training, efficient):
```
Columns:
- recording_id: int64
- audio_path: string
- duration_sec: float64
- sample_rate: int32
- user_id: int64
- user_role: string
- qpc_location: string
- surah: int32
- ayah: int32
- word: int32
- rule: string
- anti_pattern: string
- status: string
- verified_by: int64
- annotation_id: int64
- ann_start_sec: float64
- ann_end_sec: float64
- ann_label: string
- ann_confidence: float64
- ann_notes: string
```

### 3.3 Waveform Visualization Data
- **Amplitude**: Raw waveform (downsampled to ~1000 points for display)
- **Spectrogram**: Optional (Mel-spectrogram, 80 bins)
- **Phoneme Overlay**: From M3 alignment (existing pipeline)
  ```json
  {
    "phonemes": [
      {"phoneme": "ن", "start": 1.2, "end": 1.5, "sifat": {...}}
    ]
  }
  ```

---

## 4. UI/UX SPECIFICATION

### 4.1 Main Views

**1. Enhanced Dashboard** (`/dashboard`)

**Key Principle**: Intuitive data-driven UI that guides users to high-priority tasks

**Layout** (3-column responsive):

**Left Panel - Quick Stats**
- Total recordings: verified / pending / rejected
- My contribution: recordings this week
- Active annotators: count

**Center Panel - Priority Matrix** (Sortable Table)
| Rule | Anti-Pattern | Verified | Target | Coverage % | Priority ⬆️⬇️ | Action |
|------|--------------|----------|--------|------------|--------------|--------|
| Ghunnah | weak-ghunnah | 45 | 100 | 45% | 🔴 High | [Record] |
| Qalqalah | no-qalqalah | 78 | 100 | 78% | 🟡 Medium | [Record] |
| Madd | short-madd | 95 | 100 | 95% | 🟢 Low | [Record] |

**Sorting Options** (click column headers):
- Priority (default): Show most-needed patterns first
- Coverage: Show least-covered patterns first
- Verified Count: Show patterns with most/least data
- Rule Name: Alphabetical

**Visual Indicators**:
- 🔴 Red: < 50% coverage (critical priority)
- 🟡 Yellow: 50-80% coverage (medium priority)
- 🟢 Green: > 80% coverage (low priority)
- Progress bars with gradient fills

**Right Panel - Surah Coverage** (Sortable)
| Surah | Words w/ Rules | Recorded | Verified | Coverage % | Priority |
|-------|----------------|----------|----------|------------|----------|
| 89 | 45 | 12 | 8 | 18% | 🔴 High |
| 35 | 120 | 56 | 45 | 38% | 🔴 High |
| 4 | 320 | 280 | 250 | 78% | 🟡 Medium |

**Bottom Panel - Recent Activity Feed**
- Live updates: "Expert @ali verified 3 recordings"
- Filterable by: My Activity / All Activity / Verification Events

**2. Recording Studio with Advanced Filtering** (`/record`)

**Top Bar - Smart Filters** (AND logic):
```
┌─────────────────────────────────────────────────────────────────┐
│ 🔍 Filter Ayahs (AND logic - all conditions must match)         │
│                                                                  │
│ Rule: [Ghunnah ▼]  Anti-Pattern: [weak-ghunnah ▼]              │
│                                                                  │
│ Tags (click to add):                                            │
│ ☐ Surah 89   ☐ Surah 35   ☐ Surah 4   [+More Surahs]           │
│ ☐ High Priority   ☐ Medium Priority   ☐ Low Priority            │
│ ☐ Pattern-A   ☐ Pattern-B   [+Custom Tags]                      │
│                                                                  │
│ Active Filters: [Ghunnah] [weak-ghunnah] [Surah 89] [×Clear All]│
│ Results: 12 ayahs match all filters                             │
└─────────────────────────────────────────────────────────────────┘
```

**Left Panel - Word Instance List** (filtered results):
- Searchable table with columns:
  - Location (e.g., "89:27:3")
  - Arabic Text (Uthmani with highlighted rule)
  - Tags (pills: `surah-89`, `priority-high`)
  - Status (🔴 Not Recorded / 🟡 Pending / 🟢 Verified)
  - Last Recorded (date/time)
- Sort by: Priority / Location / Status / Last Modified
- Click row → loads in recording panel

**Center Panel - Recording Interface**:
- Selected word preview (large Arabic text)
- Record button (Web Audio API)
- Pause/Resume buttons
- Real-time waveform (streaming)
- Duration counter
- Auto-save on stop

**Right Panel - Context & Tips**:
- Rule description
- Anti-pattern description
- Example audio player:
  - ✅ Correct recitation
  - ❌ Violation example (this anti-pattern)
- Recording checklist:
  - [ ] Microphone positioned correctly
  - [ ] No background noise
  - [ ] Clear enunciation of violation
- Quick stats: "You've recorded 3/12 from this filter today"

**3. Annotation Editor** (`/annotate/:recording_id`)
- **Waveform Area** (WaveSurfer.js):
  - Zoomable waveform (scroll wheel)
  - Playback controls (play/pause, speed control)
  - Cursor shows current time
  - Annotations rendered as colored regions
- **Annotation Toolbar**:
  - Add region: Click-drag on waveform
  - Label input: Text field
  - Confidence slider: 0-100%
  - Notes: Textarea
  - Save/Delete buttons
- **Annotation List** (bottom):
  - Table of all annotations
  - Click to jump to time
  - Edit/Delete actions
- **Phoneme Overlay** (optional toggle):
  - Vertical lines at phoneme boundaries
  - Labels above waveform
  - Color-coded by sifat

**4. Review Queue** (`/review`)
- **For Experts Only**:
  - List of submitted recordings
  - Filter by rule, user, date
  - Quick listen + approve/reject
  - Batch operations

**5. Admin Panel** (`/admin`)
- **Rule Management**:
  - Add/Edit/Delete rules
  - Set target accuracy
- **Anti-Pattern Management**:
  - Add/Edit/Delete anti-patterns
  - Link to rules
- **User Management**:
  - Create users, assign roles
- **Export**:
  - Date range selector
  - Format selector (JSON/Parquet)
  - Filter by status (verified only)
  - Download button

### 4.2 Color Scheme (for Annotations)
```javascript
// Material Design inspired
const annotationColors = {
  "weak-ghunnah": "#FF5722",      // Deep Orange
  "no-ghunnah": "#F44336",        // Red
  "weak-qalqalah": "#FF9800",     // Orange
  "no-burst": "#FF5722",          // Deep Orange
  "short-madd": "#FFC107",        // Amber
  "long-madd": "#FFEB3B",         // Yellow
  "default": "#9E9E9E"            // Grey
};
```

---

## 5. API SPECIFICATION

### 5.1 Backend Endpoints (FastAPI)

```python
# Rules & Anti-Patterns
GET    /api/rules                      # List all rules
POST   /api/rules                      # Create rule (admin)
GET    /api/rules/{id}/anti-patterns   # List anti-patterns for rule
POST   /api/anti-patterns              # Create anti-pattern (admin)

# Tags (NEW - for filtering)
GET    /api/tags                       # List all tags
POST   /api/tags                       # Create tag (admin)
GET    /api/tags?category=surah        # Filter tags by category
DELETE /api/tags/{id}                  # Delete tag (admin)

# Word Instances with Advanced Filtering (AND logic)
GET    /api/word-instances             # Get word instances with filters
  Query params:
    - rule_id: int (required)
    - anti_pattern_id: int (optional)
    - tags: comma-separated tag IDs (AND logic: "1,5,12" = all 3 tags)
    - surah: int (shortcut for surah tag)
    - status: "not_recorded" | "recorded" | "pending" | "verified"
    - sort_by: "priority" | "location" | "status" | "last_modified"
    - sort_order: "asc" | "desc"
    - limit: int (default 50)
    - offset: int (pagination)

  Example: GET /api/word-instances?rule_id=1&tags=5,12&surah=89&sort_by=priority

  Response:
  {
    "total": 120,
    "results": [
      {
        "id": 123,
        "qpc_location": "89:27:3",
        "text": "ٱلۡمُطۡمَئِنَّةُ",
        "rule": {"id": 1, "name": "ghunnah"},
        "tags": [
          {"id": 5, "name": "surah-89", "category": "surah"},
          {"id": 12, "name": "priority-high", "category": "priority"}
        ],
        "status": "not_recorded",
        "priority_score": 0.85,
        "last_recording": null
      }
    ]
  }

GET    /api/word-instances/{id}        # Get word details

# Recordings
POST   /api/recordings                 # Create recording session
POST   /api/recordings/{id}/upload     # Upload audio file
PATCH  /api/recordings/{id}            # Update status
GET    /api/recordings?status=submitted&tags=1,5  # List recordings (filterable)
DELETE /api/recordings/{id}            # Delete recording

# Annotations
GET    /api/recordings/{id}/annotations  # Get all annotations for recording
POST   /api/annotations                  # Create annotation
PATCH  /api/annotations/{id}             # Update annotation
DELETE /api/annotations/{id}             # Delete annotation

# Export
POST   /api/export                       # Generate export
  Body: {
    "format": "parquet",
    "status": "verified",
    "date_from": "2025-01-01",
    "date_to": "2025-12-31",
    "rules": [1, 2, 3],
    "tags": [5, 12],               # NEW: filter by tags
    "surahs": [89, 35]             # NEW: filter by surahs
  }
  Returns: {"task_id": "uuid"}
GET    /api/export/{task_id}/download    # Download export file

# Statistics (Enhanced with Priority & Sorting)
GET    /api/stats                        # Dashboard stats (priority matrix)
  Query params:
    - sort_by: "priority" | "coverage" | "verified_count" | "rule_name"
    - sort_order: "asc" | "desc" (default "desc" for priority)

  Response:
  {
    "priority_matrix": [
      {
        "rule_id": 1,
        "rule_name": "ghunnah",
        "anti_pattern_id": 2,
        "anti_pattern_name": "weak-ghunnah",
        "count_verified": 45,
        "count_pending": 15,
        "count_total": 60,
        "target": 100,
        "coverage": 0.45,           # 45%
        "priority_score": 0.82,      # High priority
        "priority_label": "🔴 High"  # Visual indicator
      }
    ],
    "surah_coverage": [
      {
        "surah": 89,
        "rule_id": 1,
        "rule_name": "ghunnah",
        "count_words": 45,
        "count_recorded": 12,
        "count_verified": 8,
        "coverage": 0.18,            # 18%
        "priority_score": 0.92,
        "priority_label": "🔴 High"
      }
    ],
    "totals": {
      "recordings": 1250,
      "verified": 980,
      "pending": 200,
      "rejected": 70
    }
  }

GET    /api/stats/surahs                 # Surah-level stats
  Query params:
    - rule_id: int (optional)
    - sort_by: "priority" | "coverage" | "surah"
    - sort_order: "asc" | "desc"

GET    /api/stats/rules                  # Rule-level aggregated stats
```

### 5.2 WebSocket (Real-time Updates)
```python
WS     /ws/recording/{id}                # Streaming waveform during recording
```

---

## 6. PRIORITY CALCULATION ALGORITHM

### 6.1 Pattern Priority Score

**Formula**:
```python
def calculate_priority_score(
    verified_count: int,
    target_count: int,
    total_instances: int,
    severity: str  # "critical" | "moderate" | "minor"
) -> float:
    """
    Calculate priority score (0.0 - 1.0, higher = more urgent)

    Factors:
    1. Coverage gap: How far from target?
    2. Severity weight: Critical patterns prioritized
    3. Diminishing returns: As coverage increases, priority decreases
    """

    # Severity weights
    severity_weights = {
        "critical": 1.0,
        "moderate": 0.7,
        "minor": 0.4
    }

    # Current coverage (0.0 - 1.0)
    coverage = verified_count / target_count if target_count > 0 else 0.0

    # Coverage gap (1.0 = 0% coverage, 0.0 = 100% coverage)
    gap = 1.0 - coverage

    # Non-linear urgency: gap^2 means <50% coverage is much more urgent
    urgency = gap ** 2

    # Apply severity weight
    priority = urgency * severity_weights.get(severity, 0.5)

    return round(priority, 2)

# Example:
# Ghunnah/weak-ghunnah: verified=45, target=100, severity=critical
# coverage = 45/100 = 0.45
# gap = 1 - 0.45 = 0.55
# urgency = 0.55^2 = 0.30
# priority = 0.30 * 1.0 = 0.30 → 🟡 Medium

# Qalqalah/no-qalqalah: verified=10, target=100, severity=critical
# coverage = 10/100 = 0.10
# gap = 1 - 0.10 = 0.90
# urgency = 0.90^2 = 0.81
# priority = 0.81 * 1.0 = 0.81 → 🔴 High
```

### 6.2 Surah Priority Score

**Formula**:
```python
def calculate_surah_priority(
    surah: int,
    rule_id: int,
    count_words: int,
    count_verified: int
) -> float:
    """
    Prioritize surahs with most remaining work
    """
    coverage = count_verified / count_words if count_words > 0 else 1.0
    gap = 1.0 - coverage

    # Weight by surah length (more words = higher impact)
    length_factor = min(count_words / 100.0, 2.0)  # Cap at 2x

    priority = gap * length_factor

    return round(priority, 2)
```

### 6.3 Priority Labels

```python
def priority_label(score: float) -> str:
    if score >= 0.65:
        return "🔴 High"
    elif score >= 0.35:
        return "🟡 Medium"
    else:
        return "🟢 Low"
```

### 6.4 Tag-Based Filtering Logic (AND)

**SQL Query Example**:
```sql
-- Find word instances matching ALL specified tags
SELECT wi.*
FROM word_instances wi
WHERE wi.rule_id = :rule_id
  AND wi.id IN (
    -- Word must have ALL tag_ids
    SELECT wit.word_instance_id
    FROM word_instance_tags wit
    WHERE wit.tag_id IN (:tag_ids)
    GROUP BY wit.word_instance_id
    HAVING COUNT(DISTINCT wit.tag_id) = :tag_count
  )
ORDER BY
  CASE :sort_by
    WHEN 'priority' THEN (SELECT priority_score FROM stats WHERE ...)
    WHEN 'location' THEN wi.qpc_location
    WHEN 'status' THEN (SELECT status FROM recordings WHERE ...)
  END;
```

**Frontend Logic**:
```typescript
// User selects: Rule=Ghunnah, Tags=[surah-89, priority-high]
const filters = {
  rule_id: 1,
  tags: [5, 12],  // AND logic: must have BOTH tags
  sort_by: 'priority'
};

// API call
const response = await fetch(
  `/api/word-instances?rule_id=1&tags=5,12&sort_by=priority`
);

// Result: Only words in Surah 89 with high priority for Ghunnah rule
```

---

## 7. IMPLEMENTATION CONSTRAINTS

### 6.1 Audio Constraints
- **Max file size**: 50MB per recording (~5 minutes at 16kHz)
- **Sample rate**: 16kHz (downsampled if higher)
- **Channels**: Mono (convert stereo to mono)
- **Format**: WAV PCM 16-bit (no compression during recording)

### 6.2 Annotation Constraints
- **Precision**: 10ms minimum annotation duration
- **Overlap**: Allow overlapping annotations (different labels)
- **Validation**: `start_sec < end_sec`, both within `[0, duration_sec]`

### 6.3 Performance
- **Waveform load**: < 1s for 5min audio
- **Playback**: Low-latency (<100ms)
- **Export**: Background job (Celery + Redis)

---

## 7. INTEGRATION WITH EXISTING SYSTEM

### 7.1 Existing Components to Use
```python
# From iqrah-audio project
from iqrah.pipeline import M3Pipeline          # Phoneme alignment
from iqrah.tajweed import (
    MaddValidator,
    GhunnahValidator,
    QalqalahValidator
)
```

### 7.2 Workflow Integration
1. **Recording** → Save WAV to filesystem
2. **On Submit** → Run M3Pipeline to get phoneme alignment
3. **Store Phonemes** → Cache in DB for waveform overlay
4. **Validation** → Optionally run validators to suggest annotation regions

### 7.3 Data Bridge
```python
# Export annotations → Validation script
def validate_against_ground_truth(export_path: str):
    """
    Load annotations from Parquet
    Run validators on audio
    Compare predictions vs ground truth
    Generate precision/recall/F1 metrics
    """
```

---

## 8. SEED DATA

### 8.1 Initial Rules (from existing validators)
```sql
INSERT INTO rules (name, description, category, target_accuracy) VALUES
('ghunnah', 'Nasal resonance (ن، م)', 'tier2_specialized', 0.90),
('qalqalah', 'Echoing/bouncing (ق، ط، ب، ج، د)', 'tier2_specialized', 0.85),
('madd', 'Vowel elongation (ا، و، ي)', 'tier2_specialized', 0.95),
('tafkhim', 'Heavy/thick pronunciation', 'tier1_baseline', 0.75),
('tarqeeq', 'Light/thin pronunciation', 'tier1_baseline', 0.75);
```

### 8.2 Initial Anti-Patterns
```sql
INSERT INTO anti_patterns (rule_id, name, description, severity) VALUES
(1, 'no-ghunnah', 'Missing nasal resonance entirely', 'critical'),
(1, 'weak-ghunnah', 'Insufficient nasal resonance', 'moderate'),
(1, 'over-ghunnah', 'Excessive nasal resonance', 'minor'),
(2, 'no-qalqalah', 'Missing echoing sound', 'critical'),
(2, 'weak-qalqalah', 'Weak echoing/burst', 'moderate'),
(3, 'short-madd', 'Vowel too short', 'critical'),
(3, 'long-madd', 'Vowel too long', 'moderate');
```

### 8.3 Seed Tags (Auto-generated)
```sql
-- Surah tags (1-114)
INSERT INTO tags (name, category, description) VALUES
('surah-1', 'surah', 'Al-Fatihah'),
('surah-2', 'surah', 'Al-Baqarah'),
-- ... (auto-generate for all 114 surahs)
('surah-114', 'surah', 'An-Nas');

-- Priority tags
INSERT INTO tags (name, category, description) VALUES
('priority-high', 'priority', 'Critical - needs immediate attention'),
('priority-medium', 'priority', 'Moderate - fill when high is complete'),
('priority-low', 'priority', 'Low - nice to have');

-- Pattern tags (for specific anti-patterns)
INSERT INTO tags (name, category, description) VALUES
('pattern-ghunnah-weak', 'pattern', 'Weak nasal resonance'),
('pattern-ghunnah-none', 'pattern', 'Missing nasal resonance'),
('pattern-qalqalah-weak', 'pattern', 'Weak echoing burst'),
('pattern-qalqalah-none', 'pattern', 'Missing echoing burst');

-- Custom tags (user-defined)
INSERT INTO tags (name, category, description) VALUES
('beginner-friendly', 'custom', 'Easy words for new annotators'),
('difficult', 'custom', 'Complex tajweed combinations'),
('verified-by-expert', 'custom', 'Double-checked by senior expert');
```

### 8.4 Import QPC Words with Auto-Tagging
```python
# Migration script: data/scripts/import_qpc_words.py
import sqlite3
import re

def import_qpc_words():
    """Import words from QPC database and auto-generate tags"""

    # Read from qpc-hafs-tajweed.db
    qpc_conn = sqlite3.connect('data/qpc-hafs-tajweed.db')
    annotation_conn = sqlite3.connect('data/annotation.db')

    words = qpc_conn.execute("SELECT location, text FROM words").fetchall()

    for location, text in words:
        # Parse location: "89:27:3"
        surah, ayah, word = map(int, location.split(':'))

        # Parse HTML tags to extract rules
        rules = re.findall(r'<rule class=([^>]+)>', text)

        for rule in rules:
            # Get rule_id from rules table
            rule_id = annotation_conn.execute(
                "SELECT id FROM rules WHERE name = ?", (rule,)
            ).fetchone()

            if not rule_id:
                continue

            rule_id = rule_id[0]

            # Insert word instance
            cursor = annotation_conn.execute(
                """
                INSERT INTO word_instances (qpc_location, rule_id, has_rule)
                VALUES (?, ?, 1)
                """,
                (location, rule_id)
            )
            word_instance_id = cursor.lastrowid

            # Auto-tag with surah
            surah_tag_id = annotation_conn.execute(
                "SELECT id FROM tags WHERE name = ?", (f"surah-{surah}",)
            ).fetchone()[0]

            annotation_conn.execute(
                """
                INSERT INTO word_instance_tags (word_instance_id, tag_id)
                VALUES (?, ?)
                """,
                (word_instance_id, surah_tag_id)
            )

    annotation_conn.commit()
    qpc_conn.close()
    annotation_conn.close()

if __name__ == "__main__":
    import_qpc_words()
    print("✅ Imported QPC words with tags")
```

---

## 9. DELIVERABLES

### 9.1 Code Structure
```
tajweed-annotation-tool/
├── frontend/               # React + TypeScript
│   ├── src/
│   │   ├── components/
│   │   │   ├── WaveformEditor.tsx    # WaveSurfer.js wrapper
│   │   │   ├── AnnotationList.tsx
│   │   │   ├── RecordingStudio.tsx
│   │   │   └── Dashboard.tsx
│   │   ├── api/
│   │   │   └── client.ts             # API client
│   │   ├── store/                    # State management (Redux/Zustand)
│   │   └── App.tsx
│   ├── package.json
│   └── tsconfig.json
├── backend/                # FastAPI + Python
│   ├── app/
│   │   ├── api/
│   │   │   ├── routes/
│   │   │   │   ├── rules.py
│   │   │   │   ├── recordings.py
│   │   │   │   └── annotations.py
│   │   ├── db/
│   │   │   ├── models.py             # SQLAlchemy models
│   │   │   └── init_db.py            # Migration script
│   │   ├── integrations/
│   │   │   └── iqrah_pipeline.py     # M3Pipeline wrapper
│   │   └── main.py
│   ├── requirements.txt
│   └── alembic/                      # DB migrations
├── data/
│   ├── annotation.db                 # SQLite (created on init)
│   ├── qpc-hafs-tajweed.db          # Read-only reference
│   └── audio/                        # Audio files
├── exports/                          # Generated exports
├── docker-compose.yml                # Development environment
└── README.md
```

### 9.2 Tech Dependencies
```json
// frontend/package.json
{
  "dependencies": {
    "react": "^18.2.0",
    "wavesurfer.js": "^7.0.0",
    "tone": "^14.7.0",
    "@mui/material": "^5.14.0",
    "axios": "^1.5.0",
    "zustand": "^4.4.0"
  }
}
```

```python
# backend/requirements.txt
fastapi==0.104.0
uvicorn[standard]==0.24.0
sqlalchemy==2.0.23
alembic==1.12.1
pydantic==2.5.0
python-multipart==0.0.6
pandas==2.1.3
pyarrow==14.0.1
librosa==0.10.1
soundfile==0.12.1
celery==5.3.4
redis==5.0.1
```

---

## 10. SUCCESS METRICS

### 10.1 MVP Goals (Week 1-2)
- ✅ 100 recordings across 3 rules (Ghunnah, Qalqalah, Madd)
- ✅ At least 50 verified by expert
- ✅ Dashboard shows live stats
- ✅ Export works (Parquet format)

### 10.2 Production Goals (Month 1-2)
- ✅ 1000+ verified recordings
- ✅ 10+ experts active
- ✅ All 13 rules covered
- ✅ Integration with existing validators tested
- ✅ Precision/Recall measured: >90% for Tier 2 rules

---

## 11. DEVELOPMENT PHASES

### Phase 1: Core Recording (Week 1)
- [ ] DB schema + migrations
- [ ] Recording API (upload, save)
- [ ] Basic waveform viewer
- [ ] Simple annotation (click-drag)

### Phase 2: Annotation Editor (Week 2)
- [ ] Full WaveSurfer.js integration
- [ ] Annotation CRUD
- [ ] Playback controls
- [ ] Export JSON

### Phase 3: Review & Stats (Week 3)
- [ ] Review queue for experts
- [ ] Dashboard with stats
- [ ] Multi-user support
- [ ] Export Parquet

### Phase 4: Integration (Week 4)
- [ ] M3 phoneme overlay
- [ ] Auto-suggest annotations (validators)
- [ ] Batch export
- [ ] Production deployment

---

## 12. CONSTRAINTS FOR AI IMPLEMENTATION

1. **Use WaveSurfer.js v7+** (not custom canvas - too complex)
2. **Use Material-UI or Ant Design** (not Tailwind - need rich components)
3. **SQLite for MVP** (PostgreSQL for prod - schema compatible)
4. **FastAPI backend** (Python - integrates with existing ML pipeline)
5. **Store audio on filesystem** (not DB blobs - performance)
6. **Parquet for ML exports** (columnar, efficient, Pandas/PyArrow compatible)
7. **Responsive design** (desktop-first, but mobile-friendly for review)
8. **Keyboard shortcuts** (Space=play/pause, R=record, S=save, Delete=delete annotation)
9. **Accessibility** (ARIA labels, keyboard nav, screen reader support)
10. **Dark mode** (optional but recommended for long annotation sessions)

---

## 13. ENHANCED EXAMPLE USER FLOWS

### Flow 1: Student Records with Priority-Based Filtering

**Goal**: Record high-priority weak ghunnah samples from Surah 89

1. **Login** → Dashboard
   - Dashboard shows: "🔴 High Priority: Ghunnah/weak-ghunnah (45% coverage)"
   - Clicks [Record] button for this pattern

2. **Recording Studio** - Smart Filtering
   - Rule auto-selected: "Ghunnah"
   - Anti-pattern auto-selected: "weak-ghunnah"
   - **Applies filters** (AND logic):
     - ✅ Surah 89 (clicks checkbox)
     - ✅ High Priority (clicks checkbox)
   - System shows: "12 ayahs match all filters" (sorted by priority)

3. **Word Selection**
   - Table displays filtered results:
     | Location | Arabic Text | Tags | Status | Priority |
     |----------|-------------|------|--------|----------|
     | 89:27:3 | ٱلۡمُطۡمَئِنَّةُ | surah-89, priority-high | 🔴 Not Recorded | 0.92 |
     | 89:28:1 | ... | surah-89, priority-high | 🟡 Pending | 0.85 |
   - Clicks first row → loads in recording panel

4. **Recording**
   - Selected word preview (large): "ٱلۡمُطۡمَئِنَّةُ"
   - Listens to example violation (❌ weak ghunnah)
   - Click Record → recites with intentionally weak nasal resonance
   - Click Stop → waveform appears (3.5 seconds)

5. **Annotation**
   - Click-drag on waveform → create region (1.2s - 1.5s)
   - Label: "weak-ghunnah-onset" (auto-suggested from anti-pattern)
   - Confidence: 90% (slider)
   - Notes: "Nasal cavity not engaged" (optional)
   - Click Save Annotation

6. **Submit**
   - Click "Submit Recording" → status="submitted"
   - System updates stats: Coverage now 46% (was 45%)
   - Dashboard updates in real-time for all users

7. **Expert Review**
   - Expert navigates to Review Queue
   - Filters by: Rule=Ghunnah, Tags=surah-89
   - Sees student's recording, clicks Play
   - Approves → status="verified"
   - Stats update: Verified count +1

8. **Result**
   - Recording now available in exports (Parquet/JSON)
   - Student sees contribution: "You verified 1 sample today"
   - Dashboard priority recalculated: 🔴 High → 🟡 Medium (if coverage crosses threshold)

---

### Flow 2: Admin Generates Export for ML Training

**Goal**: Export all verified recordings for Tier 2 rules from Surahs 89, 35, 4

1. **Login as Admin** → Admin Panel → Export

2. **Configure Export**
   - Format: Parquet ✅
   - Status: Verified only ✅
   - Date range: 2025-01-01 to 2025-10-28
   - Rules: [Ghunnah, Qalqalah, Madd] ✅
   - **Tags (AND filter)**:
     - ✅ surah-89
     - ✅ surah-35
     - ✅ surah-4
   - Note: "Training data for Phase 2 validators"

3. **Generate Export**
   - Clicks "Generate Export" → task_id returned
   - Background job (Celery) processes:
     - Queries recordings matching all filters (AND logic)
     - Joins with annotations, users, word_instances
     - Generates Parquet file
   - Progress bar: 75% complete...

4. **Download**
   - Export complete → "Download" button enabled
   - Downloads: `tajweed_export_20251028_verified.parquet`
   - File size: 245 MB (1,250 recordings, 3,200 annotations)

5. **ML Integration**
   - Loads Parquet into Pandas:
     ```python
     import pandas as pd
     df = pd.read_parquet('tajweed_export_20251028_verified.parquet')
     print(df.shape)  # (3200, 20) - 3200 annotations
     ```
   - Trains validator models using annotations as ground truth
   - Evaluates: Precision 0.92, Recall 0.88, F1 0.90 ✅

---

### Flow 3: Expert Uses Surah Coverage View to Prioritize Work

**Goal**: Identify which surahs need more recordings

1. **Login as Expert** → Dashboard

2. **Surah Coverage Panel** (right side)
   - Default sort: Priority (highest first)
   - Table shows:
     | Surah | Words | Verified | Coverage | Priority |
     |-------|-------|----------|----------|----------|
     | 89 | 45 | 8 | 18% | 🔴 High (0.92) |
     | 35 | 120 | 45 | 38% | 🔴 High (0.85) |
     | 4 | 320 | 250 | 78% | 🟡 Medium (0.32) |

3. **Clicks Surah 89** (highest priority)
   - Navigates to Recording Studio
   - Filters auto-applied: `surah=89`
   - Shows all word instances from Surah 89 needing recordings

4. **Records Multiple Samples**
   - Records 5 violations from Surah 89
   - All submitted for review

5. **Stats Update**
   - Returns to Dashboard
   - Surah 89 coverage updated: 18% → 29% (8 → 13 verified)
   - Priority recalculated: 0.92 → 0.78 (still 🔴 High, but improving)

---

## 14. KEY ENHANCEMENTS SUMMARY

### ✅ Tag-Based Filtering (AND Logic)
- Users can filter by multiple tags simultaneously (e.g., surah-89 + priority-high)
- All conditions must match (AND, not OR)
- Enables precise targeting: "Show me high-priority weak-ghunnah samples from Surah 89"

### ✅ Priority-Driven Dashboard
- Sortable priority matrix shows which patterns need data most urgently
- Visual indicators (🔴🟡🟢) guide users to high-impact work
- Real-time updates as recordings are verified

### ✅ Surah-Level Coverage Tracking
- Identifies which surahs are under-represented
- Helps balance dataset across Quran
- Prevents over-concentration in specific surahs

### ✅ Intuitive Sorting & Discovery
- Sort by: Priority, Coverage, Verified Count, Rule Name, Location
- Users can discover patterns/ayahs needing attention without manual searching
- "Smart defaults": Dashboard always shows highest priority first

### ✅ Existing Tool Research Documented
- Evaluated 6+ audio annotation tools (Audacity, Praat, label-studio, etc.)
- Conclusion: WaveSurfer.js (web) + custom UI is optimal
- Export compatibility with Audacity/Praat for cross-tool workflows

### ✅ Production-Ready Architecture
- FastAPI backend integrates with existing Iqrah ML pipeline
- SQLite (MVP) → PostgreSQL (production) migration path
- Parquet exports for efficient ML training
- WebSocket for real-time updates

---

**END OF SPECIFICATION**

**Implementation Instruction for AI**:

**CRITICAL**: Before building custom components, evaluate and reuse existing open-source audio annotation tools (see Section 1.3). Preference order:
1. **label-studio** with audio config (if multi-user needed)
2. **WaveSurfer.js** + React (custom UI, recommended for tight ML integration)
3. **Praat/Audacity export** for compatibility with existing workflows

**Core Requirements**:
- Build with React + TypeScript + FastAPI (Python 3.9+)
- Use WaveSurfer.js for waveform visualization
- Use Material-UI or Ant Design for intuitive, accessible UI
- Implement tag-based AND filtering exactly as specified
- Implement priority calculation algorithm (Section 6)
- Store audio as 16kHz mono WAV on filesystem
- Support Parquet + JSON exports
- Integrate with existing M3 pipeline for phoneme overlays
- Auto-tag all word instances with surah tags during import
- Implement sortable dashboard with real-time stats updates
- Ensure accessibility: keyboard shortcuts, ARIA labels, screen reader support
