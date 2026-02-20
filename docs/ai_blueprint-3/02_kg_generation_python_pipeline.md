# 02 - Knowledge Graph Generation (Python R&D Pipeline)

This section documents how `research_and_dev/iqrah-knowledge-graph2` generates the graph.

## 1) Entry Points

Main CLI flow:
- `research_and_dev/iqrah-knowledge-graph2/src/iqrah_cli/commands/build.py`

Core builders:
- Dependency graph builder: `src/iqrah/graph/builder.py`
- Knowledge edge builder: `src/iqrah/graph/knowledge_builder.py`
- Distribution compiler: `src/iqrah/graph/knowledge.py`
- Score computation: `src/iqrah/graph/scoring.py`
- Export/import: `src/iqrah/export/cbor_export.py`, `src/iqrah/export/cbor_import.py`
- Stability validation: `src/iqrah/validation/pipeline_validation.py`

## 2) Build Stages

### Stage A - Dependency graph
`QuranGraphBuilder` creates a directed structure:
- `CHAPTER -> VERSE`
- `VERSE -> WORD_INSTANCE`
- `WORD_INSTANCE -> WORD`
- `WORD -> LEMMA -> ROOT`
- sequential verse and word dependencies also added

### Stage B - Knowledge axes and learning edges
`KnowledgeGraphBuilder` adds axis-specific nodes/edges, including:
- Memorization hierarchy (word/verse/chapter memory)
- Context windows (gaussian local context)
- Translation hierarchy and duplicate verse linking
- Grammar links (word <-> lemma, lemma <-> root meaning)
- Cross-axis links (translation -> memorization)
- Optional deep-understanding/tajweed paths

### Stage C - Distribution compilation
`KnowledgeEdgeManager.compile()` resolves `Distribution.auto(...)` into normalized edge distributions per target.
- If all incoming edges to a target are weighted, weights are normalized.
- Compiled edges become explicit distributions (mostly normal with mean/std).

### Stage D - Graph scoring
`KnowledgeGraphScoring.calculate_scores(...)` computes:
- `foundational_score`: PageRank over knowledge graph
- `influence_score`: PageRank over reversed graph
- Both log-normalized to [0,1]

### Stage E - Structure-only CBOR export
`export_graph_to_cbor(...)` writes:
- header metadata
- nodes with structural attrs + scores
- edges with distribution attrs

It intentionally excludes full Quran text/translation content.

### Stage F - Stability check
`validate_graph_stability(...)` compares new build with baseline and fails if node-id stability breaks (unless explicitly skipped).

## 3) Preset Configs

Preset files in `config/presets/`:
- `basic.yaml`: minimal, no scoring
- `full.yaml`: production-like, scoring enabled
- `research.yaml`: experimental knobs enabled
- `test-30-verses.yaml`: small targeted graph for tests

## 4) Evidence From Existing Python Build Artifact

`research_and_dev/iqrah-knowledge-graph2/knowledge-graph.stats.json` reports (snapshot in repo):
- total nodes: 326,068
- total edges: 1,352,658
- knowledge nodes: 208,043
- dependency edges: 500,766
- knowledge edges: 851,892

This is materially denser than the currently shipped mobile `rust/content.db` graph (see file 03).

## 5) Critical Mismatch With Mobile Runtime IDs

Python graph IDs commonly use semantic strings such as:
- `WORD:<word.text>`
- `WORD_INSTANCE:<chapter>:<verse>:<position>`

Rust runtime canonical IDs are encoded i64 with canonical ukeys like:
- `WORD:<word_id>`
- `WORD_INSTANCE:<chapter>:<verse>:<position>`
- knowledge IDs encoded via bit packing

There is a `NodeIdEncoder` in Python (`identifiers.py`), but current end-to-end production path in app is not driven by direct Python CBOR import into runtime DB.

## 6) Main Takeaway

Python KG pipeline is rich, modular, and research-grade.
The shipping app experience does not currently exploit most of its depth directly.
