# Claude AI Agent Instructions

This document provides **MANDATORY** guidelines for AI agents (Claude, etc.) working on the iqrah-mobile project.

## Architecture Overview

This project is a Flutter application with a Rust core:

- **Flutter UI**: `lib/` directory
- **Rust core**: `rust/` directory
- **Bridge**: `flutter_rust_bridge` connects Flutter to Rust
- See `.github/copilot-instructions.md` for detailed architecture

## ⚠️ CRITICAL: Pre-Commit CI Validation (MANDATORY)

**BEFORE EVERY COMMIT AND PUSH**, you MUST validate that CI will pass. This is NOT optional.

### Why This Matters

The CI pipeline runs with `RUSTFLAGS: -D warnings`, which treats ALL warnings as errors. A passing local build does NOT guarantee CI will pass. You MUST replicate CI conditions locally.

### Mandatory Pre-Commit Checklist

When working on Rust code, you MUST run ALL of the following commands and ensure they ALL pass BEFORE committing:

#### 1. Build with Warnings as Errors (MANDATORY)
```bash
cd rust
RUSTFLAGS="-D warnings" cargo build --all-features
```

For specific packages (e.g., iqrah-cli):
```bash
cd rust
RUSTFLAGS="-D warnings" cargo build --package iqrah-cli --release
```

**This MUST pass with zero warnings/errors.**

#### 2. Run Clippy (MANDATORY)
```bash
cd rust
cargo clippy --all-features --all-targets -- -D warnings
```

**This MUST pass with zero warnings/errors.**

#### 3. Run All Tests (MANDATORY)
```bash
cd rust
cargo test --all-features
```

For specific packages:
```bash
cd rust
cargo test --package iqrah-cli
```

**All tests MUST pass.**

#### 4. Check Formatting (MANDATORY)
```bash
cd rust
cargo fmt --all -- --check
```

**This MUST report no formatting issues.** If it fails, run `cargo fmt --all` to fix.

#### 5. Flutter Tests (When Applicable)
If you modified Flutter code:
```bash
flutter test
flutter test integration_test
```

### Common CI Failures and How to Avoid Them

#### Dead Code Warnings

When adding structs, fields, or functions that are not immediately used, the CI will fail with dead code warnings. Solutions:

1. **Preferred**: Only add code when it's actually used
2. **If truly needed for future use**: Add `#[allow(dead_code)]` attribute with a comment explaining why:
   ```rust
   // TODO: This field will be used when implementing feature X
   #[allow(dead_code)]
   field_name: Type,
   ```

3. **For API response structs**: Use the fields or mark them as allowed:
   ```rust
   #[allow(dead_code)]
   struct ApiResponse {
       // Fields from API that we'll need later
   }
   ```

#### Unused Imports

The CI will fail on unused imports. Run clippy to catch these:
```bash
cargo clippy --fix --allow-dirty
```

#### Formatting Issues

The CI checks formatting. Always run:
```bash
cargo fmt --all
```

## Development Workflow

### When Making Changes

1. **Make your code changes**
2. **Run the complete pre-commit checklist above** (ALL commands MUST pass)
3. **Only then commit and push**

### When Rust API Changes

If you modify `rust/src/api/`, regenerate the bridge:
```bash
flutter_rust_bridge_codegen generate
```

### Project-Specific Build Commands

- **iqrah-cli**: `cargo build --package iqrah-cli --release`
- **iqrah-knowledge-graph**: `cargo build --package iqrah-knowledge-graph`
- **Full workspace**: `cargo build --all-features`

## Agent Workflow Requirement

As an AI agent, you MUST follow this workflow:

1. Make code changes
2. Run `RUSTFLAGS="-D warnings" cargo build` (for affected packages)
3. Run `cargo clippy -- -D warnings`
4. Run `cargo test`
5. Run `cargo fmt --all -- --check`
6. If ANY of the above fail, FIX THE ISSUES before committing
7. Only commit and push when ALL checks pass
8. Include in commit message that all CI checks passed locally

## Summary

**DO NOT commit and push changes without validating ALL pre-commit checks pass.** The CI pipeline is strict by design (with `-D warnings`), and local validation is the ONLY way to ensure CI will pass.

Every failed CI run wastes time and creates false positives. Take the extra 2-3 minutes to validate locally—it's always faster than debugging CI failures.
