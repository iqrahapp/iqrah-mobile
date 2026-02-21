# AGENTS.md

Canonical guidance for AI coding agents. Overrides `CLAUDE.md` and `.github/copilot-instructions.md` on conflict.

---

## 1. Specification Routing (READ FIRST)

| Domain | Task | File |
|:-------|:-----|:-----|
| **Flutter / Rust core** | Architecture & data flow | `.github/copilot-instructions.md` |
| | Rust functions exposed to Flutter | `rust/src/api/` |

---

## 2. Build & Test

### Flutter / Rust core (run from repo root)

```bash
flutter test
flutter test integration_test
```

Regenerate bridge after any change to `rust/src/api/`:

```bash
flutter_rust_bridge_codegen generate
```

---

## 3. Locked Decisions (Non-Negotiable)

### Flutter / Dart

| # | Constraint |
|---|------------|
| 1 | All state management via `flutter_riverpod`. No `setState` outside of trivial local UI state. |
| 2 | All business logic lives in Rust. Dart/Flutter is UI and state management only. |
| 3 | Never call `rust/src/api/` functions directly from widgets. Go through a Riverpod provider. |

---

## 4. Git Commit Style

Format: `type(scope?): subject`
Types: `feat`, `fix`, `perf`, `refactor`, `chore`, `build`, `test`, `docs`, `ci`
