# GitHub Copilot Instructions

This document provides guidance for AI coding agents to effectively contribute to the `iqrah` project.

## Architecture Overview

This project is a Flutter application with a Rust core, where:

- The **Flutter UI** is in the `lib/` directory.
- The **core business logic** is written in Rust in the `rust/` directory.
- **`flutter_rust_bridge`** connects the Flutter frontend to the Rust backend.

### Data Flow

1.  **UI Layer (`lib/pages/`)**: Comprises Flutter widgets that represent the application's screens.
2.  **State Management (`lib/providers/`)**: Uses `flutter_riverpod` to manage application state. Providers are the primary way the UI interacts with the business logic.
3.  **Bridge Layer (`lib/rust_bridge/`)**: The auto-generated `flutter_rust_bridge` code that marshals calls from Dart to Rust. The public Rust API is defined in `rust/src/api/`.
4.  **Rust Core (`rust/src/`)**: Contains the main business logic, including exercises and data propagation models.
5.  **Persistence (`rust/src/sqlite_repo.rs`)**: Data is stored in a SQLite database. The repository pattern is used (`rust/src/repository.rs`) with a concrete implementation for SQLite.

On first launch, the application loads and decompresses a CBOR data file (`assets/iqrah-graph-v1.0.1.cbor.zst`) into the SQLite database. The logic for this is in `rust/src/cbor_import.rs`.

## Key Directories

- `lib/pages/`: Main application screens (e.g., `dashboard_page.dart`).
- `lib/providers/`: Riverpod providers for state management (e.g., `due_items_provider.dart`).
- `rust/src/api/`: The Rust functions exposed to the Flutter application.
- `rust/src/sqlite_repo.rs`: The implementation of the repository pattern for SQLite.
- `rust/src/cbor_import.rs`: Logic for importing the initial dataset from an asset.

## Developer Workflow

### Running the App

The project is configured to build the Rust code automatically. Simply run the app as you would any other Flutter project:

```sh
flutter run
```

### Code Generation

`flutter_rust_bridge` requires a code generation step if you modify the Rust API (`rust/src/api/`). The generated files are `lib/rust_bridge/frb_generated.dart` and `rust/src/frb_generated.rs`.

To re-generate the bridge code, run:

```sh
flutter_rust_bridge_codegen generate
```

### Testing

- **Flutter/Dart Tests**: Run using `flutter test`.
- **Rust Tests**: Navigate to the `rust` directory and use Cargo.
  ```sh
  cd rust
  cargo test
  ```
- **Integration Tests**: Run using `flutter test integration_test`.

## Conventions

- State management is handled exclusively by `flutter_riverpod`. When adding new state, create a new provider in the `lib/providers/` directory.
- All business logic and data persistence should be implemented in the Rust core. The Flutter app should be primarily concerned with UI and state management.
- The public API for the Rust library is defined in the `rust/src/api/` directory. Any function that needs to be called from Flutter must be defined there.
