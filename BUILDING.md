# Building iqrah-mobile

This document describes how to build the iqrah-mobile Flutter application.

## Prerequisites

- Flutter SDK (latest stable)
- Rust toolchain (for building native components)
- Git LFS (for large files)

## Quick Start

### 1. Download Knowledge Graph Assets

Before building the app, you need to download the knowledge graph data:

```bash
# Download the latest graph release (v1.1.0)
./scripts/download-graph-release.sh v1.1.0 assets

# Or specify a different version
./scripts/download-graph-release.sh v1.0.0 assets
```

This will download:
- `assets/knowledge-graph.cbor.zst` - The compressed knowledge graph
- `assets/content.db` - The content database (if included in release)

### 2. Build Native Rust Components

```bash
cd rust
cargo build --release
```

### 3. Build Flutter App

```bash
# Development build
flutter build apk

# Or for iOS
flutter build ios

# Production build
flutter build apk --release
```

## CI/CD

The CI pipeline automatically downloads the knowledge graph assets using the same script:

- **Headless Tests**: Downloads graph before running integration tests
- **Knowledge Graph CI**: Tests the CBOR build pipeline

See `.github/workflows/` for CI configuration.

## Manual Graph Download (Alternative)

If the script doesn't work, you can manually download:

1. Go to https://github.com/iqrahapp/iqrah-mobile/releases/tag/iqrah-graph-v1.1.0
2. Download `iqrah-graph-v1.1.0.tar.gz`
3. Extract to root: `tar -xzf iqrah-graph-v1.1.0.tar.gz`
4. Move files to assets: `mv knowledge-graph.cbor.zst assets/`

## Troubleshooting

### Missing Graph Assets

If you get errors about missing `knowledge-graph.cbor.zst`:

```bash
# Verify the file exists
ls -lh assets/knowledge-graph.cbor.zst

# Re-download if missing
./scripts/download-graph-release.sh v1.1.0 assets
```

### Build Fails with Import Errors

The Rust CLI needs the graph to import:

```bash
cd rust
export CONTENT_DB_PATH="./data/content.db"
export USER_DB_PATH="./data/user.db"
./target/release/iqrah import ../assets/knowledge-graph.cbor.zst
```

## Development Workflow

When working with knowledge graphs:

1. **Generate new graph**: Use `iqrah build all` command (see research_and_dev/iqrah-knowledge-graph2/)
2. **Test locally**: Import with `iqrah import` command
3. **Create release**: Package as tar.gz and create GitHub release
4. **Update version**: Modify `scripts/download-graph-release.sh` to use new version
