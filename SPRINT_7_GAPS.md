# Sprint 7 Integration Gaps

## Status: Backend Complete, Flutter Bridge Broken

### ✅ Working
- Type-safe SQLx repositories (query_as<_, Struct>)
- LearningService + SessionService with FSRS
- Two-database architecture (content.db + user.db)
- 15 unit tests passing
- Pure SQLx (rusqlite removed)

### ❌ Blocking Issues

**1. Flutter Bridge Mismatch**
- `iqrah-api/src/api.rs` has NO `#[frb]` annotations
- Flutter expects: `setupDatabase()`, `getExercises()`, `processReview()`
- Rust has: `init_app_async()`, `get_due_items_async()`, `process_review_async()`
- Need: Add `#[frb(sync)]` or `#[frb]` to expose functions, regenerate bindings

**2. Empty Databases**
- content.db has schema but no Quran data
- Need: Import knowledge graph or run migration from old iqrah.db

**3. API Signature Incompatibility**
```rust
// Flutter expects (lib/rust_bridge/api.dart:17)
setupDatabase({String? dbPath, required List<int> kgBytes})

// Rust has (api.rs:20)
init_app_async(content_db_path: String, user_db_path: String)
```

### 🔧 Required Before Flutter Works

1. Add flutter_rust_bridge attributes to api.rs functions
2. Regenerate Flutter bindings: `flutter_rust_bridge_codegen`
3. Import Quran content data into content.db
4. Create adapter layer matching old API signatures
5. Test: Flutter → Rust → Database round trip

### Files to Check
- `rust/crates/iqrah-api/src/api.rs` - needs #[frb] annotations
- `lib/rust_bridge/api.dart` - generated bindings (currently stale)
- `rust/crates/iqrah-storage/migrations/*.sql` - schemas exist, data missing

### Quick Fix Path
1. Keep old API wrapper in iqrah-api calling new services
2. Add #[frb] to wrapper functions
3. Regenerate bindings
4. Import data
