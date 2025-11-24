# Task 4.2: Package Download Foundation (Design)

## Metadata
- **Priority:** P1 (Package Management Foundation)
- **Estimated Effort:** 1 day (design only, no implementation)
- **Dependencies:** None
- **Agent Type:** Research + Documentation
- **Parallelizable:** Yes (with 4.1)

## Goal

Design the package download and installation system architecture without implementing it, creating a roadmap for post-MVP implementation.

## Context

**Current State:**
- App ships with 5 translations embedded
- No download capability
- No package management

**Future Vision:**
- Minimal app size (1 translation only)
- Download additional translations on demand
- Download audio recitations
- Download alternative text variants (Imlaei, Indopak)

**This Task:**
- Design architecture (no code)
- Define package manifest format
- Document API endpoints (mock)
- Create implementation task breakdown

## Deliverables

### Document 1: Package System Architecture

**File:** `docs/architecture/package-system-design.md`

**Contents:**
1. **Package Types:**
   - Translation packages
   - Audio recitation packages
   - Text variant packages

2. **Package Manifest Format (JSON):**
```json
{
  "package_id": "translation:sahih-international",
  "type": "translation",
  "version": "1.0.0",
  "name": "Sahih International",
  "author": "Sahih International",
  "language": "en",
  "size_bytes": 524288,
  "checksum": "sha256:...",
  "download_url": "https://cdn.iqrah.app/packages/translation-sahih-international-1.0.0.sql.gz",
  "dependencies": [],
  "compatible_app_versions": [">=1.0.0"]
}
```

3. **Download Process:**
```
User → Select Package → Check Compatibility → Download → Verify → Install → Update DB
```

4. **Storage:**
   - Downloaded files: `/data/app/iqrah/packages/`
   - Installed packages tracked in content.db: `installed_packages` table

5. **Error Handling:**
   - Network failures (retry logic)
   - Checksum mismatches (re-download)
   - Insufficient storage (warn user)
   - Installation failures (rollback)

### Document 2: API Design (Mock)

**File:** `docs/api/package-api-spec.md`

**Endpoints (to be implemented later):**

```
GET /api/packages/list
Response: [{ package_id, name, version, size, ... }]

GET /api/packages/{package_id}/manifest
Response: { package manifest JSON }

GET /api/packages/{package_id}/download
Response: Binary SQL file (gzipped)
```

**Client-Side API (Rust):**
```rust
pub trait PackageManager {
    async fn list_available_packages(&self) -> Result<Vec<PackageInfo>>;
    async fn download_package(&self, package_id: &str, progress: impl Fn(f64)) -> Result<PathBuf>;
    async fn verify_package(&self, path: &Path, checksum: &str) -> Result<bool>;
    async fn install_package(&self, path: &Path) -> Result<()>;
    async fn uninstall_package(&self, package_id: &str) -> Result<()>;
}
```

### Document 3: Implementation Roadmap

**File:** `docs/todo/package-system-implementation-roadmap.md`

**Phase 1: Download Infrastructure (1 week)**
- HTTP client (reqwest)
- Progress tracking
- Retry logic
- Checksum verification

**Phase 2: Package Installation (3-5 days)**
- SQL parsing and validation
- Transaction-based installation
- Rollback on failure

**Phase 3: Package Management UI (1 week)**
- Package list screen
- Download progress UI
- Storage management

**Phase 4: Audio Support (1-2 weeks)**
- Audio package format
- Audio player integration
- Streaming vs local storage

**Estimated Total Effort:** 3-4 weeks

### Document 4: Database Schema Updates

**File:** `docs/schema/package-management-tables.md`

**New Tables:**
```sql
CREATE TABLE installed_packages (
    package_id TEXT PRIMARY KEY,
    package_type TEXT NOT NULL,
    version TEXT NOT NULL,
    installed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    enabled BOOLEAN NOT NULL DEFAULT 1
);

CREATE TABLE package_files (
    file_id INTEGER PRIMARY KEY AUTOINCREMENT,
    package_id TEXT NOT NULL,
    file_path TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    FOREIGN KEY (package_id) REFERENCES installed_packages(package_id)
);
```

## Implementation Steps

### Step 1: Research Existing Solutions (2 hours)

Research how other apps handle content packages:
- Google Play Asset Delivery
- Unity Asset Bundles
- Mobile game asset downloads

Document best practices.

### Step 2: Define Package Manifest Spec (2 hours)

Create JSON schema for package manifests with:
- Required fields
- Optional fields
- Validation rules
- Examples

### Step 3: Design Download Flow (1 hour)

Create flowchart:
```
[User Taps Package] → [Check Storage] → [Download] → [Verify] → [Install] → [Enable]
         ↓
    [Show Progress]
         ↓
   [Handle Errors]
```

### Step 4: Design API Spec (1 hour)

Document all API endpoints (even if mock):
- Request/response formats
- Error codes
- Rate limiting
- Authentication (if needed)

### Step 5: Create Implementation Tasks (2 hours)

Break down implementation into 10-15 subtasks:
- Task: Implement HTTP client
- Task: Add progress tracking
- Task: Create package verification
- Task: Build installation logic
- etc.

### Step 6: Document Security Considerations (1 hour)

Address:
- Package integrity (checksums)
- HTTPS enforcement
- Package signing (future)
- Malicious package prevention

## Verification Plan

### Design Review Checklist

- [ ] Architecture document complete (2000+ words)
- [ ] Package manifest format defined
- [ ] API endpoints documented
- [ ] Implementation roadmap created
- [ ] Database schema designed
- [ ] Security considerations documented
- [ ] All documents in markdown format

### Peer Review

- [ ] Architecture makes sense
- [ ] No obvious security holes
- [ ] Implementation seems feasible
- [ ] Estimated effort reasonable

## Success Criteria

- [ ] 4 design documents created
- [ ] Package manifest JSON schema defined
- [ ] API spec documented (mock endpoints)
- [ ] Implementation roadmap with 10+ tasks
- [ ] Database schema for package management
- [ ] Security considerations addressed
- [ ] Documents reviewed for completeness

## Related Files

**Create These Files:**
- `/docs/architecture/package-system-design.md`
- `/docs/api/package-api-spec.md`
- `/docs/todo/package-system-implementation-roadmap.md`
- `/docs/schema/package-management-tables.md`

**No Code Changes:** This is design-only task.

## Notes

### Why Design-Only?

Full package system is 3-4 weeks of work. For MVP:
- Ship with embedded translations (Task 4.1 handles selection)
- Design the system architecture now
- Implement post-MVP based on user demand

### Key Design Decisions

1. **SQL Files as Packages:** Packages are SQL files, not custom formats
   - Pros: Simple, SQLite-compatible, inspectable
   - Cons: Larger size (but gzip helps)

2. **Pull Model:** User downloads packages (not pushed)
   - Pros: User control, bandwidth-friendly
   - Cons: Requires manual selection

3. **No App Store:** Custom CDN, not relying on platform stores
   - Pros: Faster updates, cross-platform
   - Cons: Need to host files

### Future Enhancements

- Auto-update packages
- Differential updates (only changed data)
- P2P distribution (torrent-like)
- Package subscriptions
- Premium packages
