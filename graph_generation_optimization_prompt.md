# Performance Optimization Task: Accelerate Knowledge Graph Generation

## Context
The project generates a complex knowledge graph for Quranic learning (memorization, translation, tajweed, etc.). The current implementation is written in Python using `networkx` and is unacceptably slow. Generating the full graph (114 chapters, ~30k nodes, ~100k edges) takes an excessive amount of time (estimated > 20 minutes, potentially hours), making iteration painful. Even a small subset (2 chapters) takes ~2-3 minutes.

## Goal
**Reduce full graph generation time to under 10 seconds.**

## Current Implementation
The generation pipeline consists of three steps, orchestrator script is located at `research_and_dev/iqrah-knowledge-graph2/src/iqrah_cli/commands/build.py`:

1.  **Content DB Generation**: Parses raw JSON/CSV data into a SQLite database (`content.db`).
    -   *Performance*: Acceptable (~10s).
    -   *Code*: `research_and_dev/iqrah-knowledge-graph2/src/iqrah/content/builder.py`
2.  **Dependency Graph**: Creates the base graph structure (Chapters -> Verses -> Words -> Lemmas -> Roots).
    -   *Performance*: Slow (~30s-1min).
    -   *Code*: `research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/builder.py`
3.  **Knowledge Graph**: Adds "knowledge edges" (e.g., sequential memorization, cross-axis associations) and computes weights using Gaussian/Beta distributions.
    -   *Performance*: **Critical Bottleneck** (Extremely slow).
    -   *Code*: `research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/knowledge_builder.py`
    -   *Bottlenecks*:
        -   Iterating over thousands of nodes in Python.
        -   Heavy use of `networkx` methods (`add_edge`, `has_edge`) in tight loops.
        -   `KnowledgeEdgeManager` overhead in `research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/knowledge.py`.

## Requirements for the Optimizer Agent
1.  **Rewrite Strategy**: You are authorized and encouraged to rewrite the graph generation logic in **Rust**.
    -   The project already has a Rust workspace in `rust/`.
    -   You can create a new binary crate (e.g., `iqrah-graph-gen`) in the Rust workspace.
    -   Use high-performance graph libraries like `petgraph` (Rust) instead of `networkx`.
2.  **Integration**:
    -   The new tool must produce the same output artifacts:
        -   `content.db` (SQLite) with the correct schema (see `rust/crates/iqrah-storage/migrations_content/20241126000001_unified_content_schema.sql`).
        -   Graph export (GraphML or CBOR) compatible with the existing system.
    -   It should leverage the existing `iqrah-core` domain logic where possible (e.g., node ID encoding/decoding in `rust/crates/iqrah-core/src/domain/node_id.rs`).
3.  **Testing & Safety**:
    -   Implement unit tests to ensure graph structure correctness (node counts, edge types, connectivity).
    -   Verify that the generated IDs match the strict 64-bit encoding expected by the backend.
4.  **CLI**:
    -   The new tool should expose a CLI compatible with or replacing the current `iqrah_cli build` command.

## Key Files to Reference
-   **Python Logic (To be optimized/replaced)**:
    -   `research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/knowledge_builder.py`
    -   `research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/knowledge.py`
-   **Data Source**: `research_and_dev/data/` (Raw Quran JSON/CSV files).
-   **Target Schema**: `rust/crates/iqrah-storage/migrations_content/20241126000001_unified_content_schema.sql`.
-   **ID Contracts**: `docs/architecture/data-architecture-v2.md` and `rust/crates/iqrah-core/src/domain/node_id.rs`.

## Expected Deliverable
A high-performance Rust CLI tool that generates the full `content.db` and Knowledge Graph in < 10 seconds, completely replacing the Python generation pipeline.
