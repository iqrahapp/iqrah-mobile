# Step 1: Setup Workspace

## Goal
Create a Cargo workspace with 4 crates for clean separation of concerns.

## Current Structure
```
rust/
├── Cargo.toml (single package)
└── src/
    ├── lib.rs
    ├── api/mod.rs
    ├── sqlite_repo.rs (1,378 lines - THE PROBLEM)
    └── ...
```

## Target Structure
```
rust/
├── Cargo.toml (workspace root)
└── crates/
    ├── iqrah-core/      # Domain logic (no DB dependencies)
    ├── iqrah-storage/   # Database access (SQLx)
    ├── iqrah-api/       # Flutter bridge (FRB)
    └── iqrah-cli/       # Developer CLI tool
```

## Implementation

### 1.1 Create Workspace Directory Structure

```bash
cd /home/user/iqrah-mobile/rust

# Create crate directories
mkdir -p crates/{iqrah-core,iqrah-storage,iqrah-api,iqrah-cli}/src
```

### 1.2 Create Workspace Root Cargo.toml

**File:** `rust/Cargo.toml`

```toml
[workspace]
members = [
    "crates/iqrah-core",
    "crates/iqrah-storage",
    "crates/iqrah-api",
    "crates/iqrah-cli",
]
resolver = "2"

[workspace.dependencies]
# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Async runtime
tokio = { version = "1", features = ["full"] }

# Database
sqlx = { version = "0.7", features = ["runtime-tokio-native-tls", "sqlite", "macros"] }

# Traits
async-trait = "0.1"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Time
chrono = { version = "0.4", features = ["serde"] }

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# Global state
once_cell = "1.19"
```

### 1.3 Create iqrah-core Cargo.toml

**File:** `rust/crates/iqrah-core/Cargo.toml`

```toml
[package]
name = "iqrah-core"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = { workspace = true }
thiserror = { workspace = true }
async-trait = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
chrono = { workspace = true }
tracing = { workspace = true }

# FSRS algorithm
fsrs = "1.3"

[dev-dependencies]
mockall = "0.12"
proptest = "1.0"
tokio-test = "0.4"
rstest = "0.18"
```

### 1.4 Create iqrah-storage Cargo.toml

**File:** `rust/crates/iqrah-storage/Cargo.toml`

```toml
[package]
name = "iqrah-storage"
version = "0.1.0"
edition = "2021"

[dependencies]
iqrah-core = { path = "../iqrah-core" }

sqlx = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
async-trait = { workspace = true }
tokio = { workspace = true }
tracing = { workspace = true }

[dev-dependencies]
tokio-test = "0.4"
rstest = "0.18"
```

### 1.5 Create iqrah-api Cargo.toml

**File:** `rust/crates/iqrah-api/Cargo.toml`

```toml
[package]
name = "iqrah-api"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["staticlib", "cdylib"]

[dependencies]
iqrah-core = { path = "../iqrah-core" }
iqrah-storage = { path = "../iqrah-storage" }

flutter_rust_bridge = "2.0"
anyhow = { workspace = true }
tokio = { workspace = true }
once_cell = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }

[build-dependencies]
flutter_rust_bridge_codegen = "2.0"
```

### 1.6 Create iqrah-cli Cargo.toml

**File:** `rust/crates/iqrah-cli/Cargo.toml`

```toml
[package]
name = "iqrah-cli"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "iqrah"
path = "src/main.rs"

[dependencies]
iqrah-core = { path = "../iqrah-core" }
iqrah-storage = { path = "../iqrah-storage" }

clap = { version = "4", features = ["derive"] }
tokio = { workspace = true }
anyhow = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
```

### 1.7 Create Placeholder lib.rs Files

```bash
# iqrah-core
echo 'pub mod domain;
pub mod ports;
pub mod services;' > crates/iqrah-core/src/lib.rs

# iqrah-storage
echo 'pub mod content;
pub mod user;
pub mod migrations;' > crates/iqrah-storage/src/lib.rs

# iqrah-api
echo '// Flutter bridge API
pub mod api;' > crates/iqrah-api/src/lib.rs

# iqrah-cli
echo 'fn main() {
    println!("Iqrah CLI - Coming soon");
}' > crates/iqrah-cli/src/main.rs
```

### 1.8 Create Module Files

```bash
# iqrah-core modules
mkdir -p crates/iqrah-core/src/{domain,ports,services}
touch crates/iqrah-core/src/domain/mod.rs
touch crates/iqrah-core/src/ports/mod.rs
touch crates/iqrah-core/src/services/mod.rs

# iqrah-storage modules
mkdir -p crates/iqrah-storage/src/{content,user,migrations}
touch crates/iqrah-storage/src/content/mod.rs
touch crates/iqrah-storage/src/user/mod.rs
touch crates/iqrah-storage/src/migrations/mod.rs

# iqrah-api modules
mkdir -p crates/iqrah-api/src
touch crates/iqrah-api/src/api.rs
```

## Validation

### Test Workspace Build

```bash
cd /home/user/iqrah-mobile/rust
cargo check --workspace
```

Expected output:
```
Checking iqrah-core v0.1.0
Checking iqrah-storage v0.1.0
Checking iqrah-api v0.1.0
Checking iqrah-cli v0.1.0
Finished dev [unoptimized + debuginfo] target(s) in X.XXs
```

### Verify Structure

```bash
tree -L 3 crates/
```

Expected:
```
crates/
├── iqrah-api
│   ├── Cargo.toml
│   └── src
│       ├── api.rs
│       └── lib.rs
├── iqrah-cli
│   ├── Cargo.toml
│   └── src
│       └── main.rs
├── iqrah-core
│   ├── Cargo.toml
│   └── src
│       ├── domain
│       ├── lib.rs
│       ├── ports
│       └── services
└── iqrah-storage
    ├── Cargo.toml
    └── src
        ├── content
        ├── lib.rs
        ├── migrations
        └── user
```

## Success Criteria

- [ ] Workspace compiles with `cargo check --workspace`
- [ ] 4 crates created
- [ ] No compilation errors
- [ ] Directory structure matches target

## Next Step

Proceed to `02-DATABASE-SCHEMA.md`
