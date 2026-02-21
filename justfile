set shell := ["bash", "-cu"]

default:
    @just --list

# Regenerate Dart API client from latest openapi.json
gen-api:
    flutter pub run build_runner build --delete-conflicting-outputs

# Regenerate SQLx offline metadata for Rust workspace
sqlx-prepare:
    cd rust && ./scripts/sqlx_prepare_workspace.sh

# Check SQLx offline metadata is up-to-date
sqlx-check:
    cd rust && ./scripts/sqlx_prepare_workspace.sh --check
