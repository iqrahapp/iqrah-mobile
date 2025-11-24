# Task 5.1: Error Handling & Logging Audit

## Metadata
- **Priority:** P2 (Production Hardening)
- **Estimated Effort:** 2 days
- **Dependencies:** All Phase 1-4 tasks
- **Agent Type:** Refactoring + Testing
- **Parallelizable:** No (final polish)

## Goal

Audit and improve error handling throughout the Rust codebase, replacing panics with graceful errors, adding structured logging, and ensuring production-quality error messages.

## Context

**Current Issues:**
- `.unwrap()` and `.expect()` in production code → potential panics
- Generic error messages → hard to debug
- No structured logging → difficult to trace issues
- Errors not user-friendly → confusion

**Production Requirements:**
- No panics (all errors handled gracefully)
- Clear error messages for users
- Detailed logs for debugging
- Error telemetry (future)

## Implementation Steps

### Step 1: Audit for Panics (2-3 hours)

**Find all unwraps:**
```bash
cd rust
rg "\.unwrap\(\)" --type rust crates/
rg "\.expect\(" --type rust crates/
rg "panic!" --type rust crates/
```

**Create audit report:**
```
File: learning_service.rs:45
Issue: .unwrap() on Option<Node>
Risk: HIGH - could panic if node not found
Fix: Replace with .ok_or(Error::NodeNotFound)?

File: session_service.rs:102
Issue: .expect("Failed to parse")
Risk: MEDIUM - unclear why parsing could fail
Fix: Use proper error type with context
```

### Step 2: Replace Unwraps with Error Handling (4-6 hours)

**Pattern to follow:**

**Before:**
```rust
let node = content_repo.get_node(node_id).await.unwrap();
```

**After:**
```rust
let node = content_repo
    .get_node(node_id)
    .await?
    .ok_or(LearningError::NodeNotFound {
        node_id: node_id.to_string(),
    })?;
```

**Priority areas:**
1. Public API functions (highest priority)
2. Service layer
3. Repository layer
4. Internal utilities (lowest priority - debug_assert! OK)

### Step 3: Add Structured Logging (2-3 hours)

**Add tracing crate:**

**File:** `rust/crates/iqrah-core/Cargo.toml`
```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = "0.3"
```

**Initialize logging:**

**File:** `rust/crates/iqrah-api/src/lib.rs`
```rust
pub fn init_logging() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}
```

**Add log statements:**

**File:** `rust/crates/iqrah-core/src/services/learning_service.rs`
```rust
use tracing::{info, warn, error, debug, instrument};

#[instrument(skip(self), fields(user_id, node_id))]
pub async fn record_review(&self, user_id: &str, node_id: &str, rating: Rating) -> Result<()> {
    info!("Recording review: user={}, node={}, rating={:?}", user_id, node_id, rating);

    match self.try_record_review(user_id, node_id, rating).await {
        Ok(_) => {
            info!("Review recorded successfully");
            Ok(())
        }
        Err(e) => {
            error!("Failed to record review: {}", e);
            Err(e)
        }
    }
}
```

### Step 4: Improve Error Messages (2-3 hours)

**Make errors user-friendly:**

**File:** `rust/crates/iqrah-core/src/error.rs`
```rust
#[derive(Debug, thiserror::Error)]
pub enum LearningError {
    #[error("Node '{node_id}' not found. It may have been removed from the content database.")]
    NodeNotFound {
        node_id: String,
    },

    #[error("Failed to update progress: {reason}")]
    ProgressUpdateFailed {
        reason: String,
    },

    #[error("Invalid rating value: {rating}. Must be 1-4.")]
    InvalidRating {
        rating: i32,
    },

    #[error("Database error: {0}. Please try again.")]
    DatabaseError(#[from] sqlx::Error),
}
```

**Add helper methods:**
```rust
impl LearningError {
    pub fn user_message(&self) -> String {
        match self {
            Self::NodeNotFound { .. } => {
                "This content is no longer available. Please refresh.".to_string()
            }
            Self::DatabaseError(_) => {
                "A database error occurred. Please try again.".to_string()
            }
            _ => self.to_string(),
        }
    }

    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::DatabaseError(_))
    }
}
```

### Step 5: Add Error Reporting CLI Command (1 hour)

**File:** `rust/crates/iqrah-cli/src/commands/report_errors.rs`
```rust
pub async fn report_errors() -> Result<()> {
    // Read error logs from last 24 hours
    // Group by error type
    // Display summary

    println!("Error Report (Last 24 hours):");
    println!("  NodeNotFound: 5 occurrences");
    println!("  DatabaseError: 2 occurrences");

    Ok(())
}
```

### Step 6: Add Error Tests (2 hours)

**File:** `rust/crates/iqrah-core/tests/error_handling_test.rs`
```rust
#[tokio::test]
async fn test_graceful_handling_of_missing_node() {
    let learning_service = setup_service().await;

    let result = learning_service
        .record_review("default", "NON_EXISTENT:NODE", Rating::Good)
        .await;

    // Should return error, not panic
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), LearningError::NodeNotFound { .. }));
}

#[tokio::test]
async fn test_error_messages_are_clear() {
    let error = LearningError::NodeNotFound {
        node_id: "VERSE:1:1:memorization".to_string(),
    };

    let message = error.to_string();
    assert!(message.contains("VERSE:1:1:memorization"));
    assert!(message.contains("not found"));
}
```

### Step 7: Update FFI Error Handling (1-2 hours)

**File:** `rust/crates/iqrah-api/src/api.rs`
```rust
pub async fn record_review_ffi(
    user_id: String,
    node_id: String,
    rating: i32,
) -> Result<(), String> {  // Return user-friendly error string
    let learning_service = get_learning_service()?;

    match learning_service.record_review(&user_id, &node_id, Rating::from(rating)).await {
        Ok(_) => Ok(()),
        Err(e) => {
            tracing::error!("FFI error: {}", e);
            Err(e.user_message())  // User-friendly message to Flutter
        }
    }
}
```

## Verification Plan

### Panic Audit

```bash
cd rust
rg "\.unwrap\(\)" --type rust crates/ | wc -l
# Target: 0 in main code paths (tests OK)

rg "\.expect\(" --type rust crates/ | wc -l
# Target: < 5 (only for truly unreachable code)

rg "panic!" --type rust crates/ | wc -l
# Target: 0
```

### Error Tests

```bash
cargo test error_handling
```

- [ ] All error scenarios have tests
- [ ] No panics in tests
- [ ] Error messages are clear

### Logging Verification

```bash
RUST_LOG=info cargo run --bin iqrah-cli -- schedule --limit 5
```

- [ ] Info logs appear
- [ ] Errors logged with context
- [ ] No sensitive data in logs

### Manual Testing

Trigger various errors:
- Invalid node ID
- Database connection failure
- Malformed input

Verify:
- [ ] App doesn't crash
- [ ] Error message is helpful
- [ ] Logs contain debug info

## Success Criteria

- [ ] Zero `.unwrap()` in public APIs
- [ ] < 5 `.expect()` in entire codebase
- [ ] Zero `panic!()` in production code
- [ ] Structured logging added (tracing)
- [ ] Error messages user-friendly
- [ ] Error tests pass (5+ test cases)
- [ ] FFI errors return strings (not panics)
- [ ] All CI checks pass
- [ ] Manual error testing successful

## Related Files

**Audit These Files:**
- `/rust/crates/iqrah-core/src/services/*.rs`
- `/rust/crates/iqrah-storage/src/**/*.rs`
- `/rust/crates/iqrah-api/src/api.rs`

**Modify:**
- All files with unwraps
- Error type definitions
- FFI layer

**Create:**
- `/rust/crates/iqrah-core/tests/error_handling_test.rs`
- `/rust/crates/iqrah-cli/src/commands/report_errors.rs`

## Notes

### Error Handling Philosophy

**Rust's Result<T, E>:**
- Use Result for expected errors (node not found, network failure)
- Use panic!() only for programmer errors (array out of bounds in test)

**User vs Developer Errors:**
- User errors: Friendly message + retry option
- Developer errors: Detailed context + stack trace in logs

### Logging Levels

- **ERROR:** Something failed, user affected
- **WARN:** Something unexpected, but handled
- **INFO:** Important state changes (review recorded, session generated)
- **DEBUG:** Detailed flow for debugging
- **TRACE:** Very verbose (not used in production)

### Production Monitoring (Future)

After this task, consider:
- Error aggregation (Sentry, Rollbar)
- Performance monitoring (OpenTelemetry)
- User feedback integration

### Security Note

Never log:
- User IDs (use hashed IDs)
- Personal data
- API keys or secrets
- Full database queries (may contain user data)

Log safely:
```rust
info!("Review recorded for node {}", node_id);  // OK
error!("Database error: {}", e);  // OK
debug!("User {} reviewed node {}", hash(user_id), node_id);  // OK
```
