# Database Test Utilities

## create_test_db

Creates a lightweight test database with sample data for integration testing.

### Usage

```bash
cd rust
cargo run --example create_test_db -- <output_path>
```

### Example

```bash
# Create content_test.db with sample data (Bismillah + Chapter 1)
cargo run --example create_test_db -- content_test.db
```

### What it does

1. Creates a new SQLite database at the specified path
2. Runs all content database migrations (ensuring correct checksums)
3. Seeds sample data:
   - Chapter 1 (Al-Fatihah) metadata
   - Verse 1:1 (Bismillah) with Arabic text
   - 4 words from Bismillah with positions and text

### Size comparison

- **content_test.db**: ~448KB (sample data only)
- **content.db**: ~87MB (full Quran with 6236 verses)

### Integration testing

The Flutter integration tests use `content_test.db` to:
- Keep test execution fast
- Avoid bundling the full 87MB database in test builds
- Ensure consistent test data

Production builds use `content.db` with the complete Quran.
