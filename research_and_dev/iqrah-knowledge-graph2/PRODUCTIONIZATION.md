# Iqrah Knowledge Graph - Production Architecture

**Version:** 2.0
**Status:** Production-Ready
**Date:** 2025-01-15

## Overview

This document describes the production-ready architecture for generating Quranic knowledge graphs. The system has been refactored from experimental notebook code into a robust CLI tool with proper separation of concerns.

## Key Architectural Changes

### 1. Content Separation (Critical)

**Old Architecture (Draft):**
- Content (Arabic text, translations) embedded in graph nodes
- Large CBOR files (~15-20MB)
- Difficult to update translations without regenerating graphs

**New Architecture (Production):**
- **Graph structure** stored in CBOR (nodes, edges, scores only)
- **Content data** stored in SQLite database (text, translations, transliterations)
- Content augmented at runtime via indexed database queries

**Benefits:**
- Smaller CBOR files (structure-only, ~5-10MB)
- Independent content updates (new translations without graph regeneration)
- Better separation of concerns
- Faster graph loading
- SQLite provides fast indexed lookups

### 2. Module Organization

```
src/iqrah/
├── content/              # NEW: Content database system
│   ├── schema.py         # Database schema definitions
│   ├── builder.py        # Build database from offline data
│   └── database.py       # Query interface for content lookups
│
├── export/               # NEW: Graph export/import
│   ├── cbor_export.py    # Structure-only CBOR export
│   └── cbor_import.py    # CBOR import and content augmentation
│
├── graph/
│   ├── builder.py        # Dependency graph building (existing)
│   ├── knowledge_builder.py  # NEW: Knowledge edges (refactored from notebook)
│   ├── scoring.py        # NEW: PageRank scoring (extracted from notebook)
│   ├── knowledge.py      # Edge manager (existing)
│   └── node_manager.py   # Node queries (existing)
│
├── config/               # NEW: Configuration system
│   └── config.py         # YAML config loading
│
└── quran_offline/        # Offline data loading (existing)
```

### 3. Production CLI

**Old CLI:**
```bash
# Single command, API-based
iqrah build corpus.csv output.graphml
```

**New CLI (Sub-commands):**
```bash
# Build content database
iqrah build content-db -o content.db --morphology corpus.csv

# Build dependency graph
iqrah build dependency-graph -o dependency.graphml --morphology corpus.csv --chapters 1-114

# Build knowledge graph
iqrah build knowledge-graph --input dependency.graphml -o knowledge.cbor.zst --preset full

# Build from scratch
iqrah build knowledge-graph --from-scratch --morphology corpus.csv -o knowledge.cbor.zst --preset full

# Build everything
iqrah build all --morphology corpus.csv --preset full
```

## Quick Start

### Prerequisites

```bash
# Install dependencies
cd research_and_dev/iqrah-knowledge-graph2
pip install -e .

# Verify offline data
ls research_and_dev/data/
```

### Example 1: Basic Graph (Minimal)

```bash
# Build with basic preset (memorization + translation only)
iqrah build all \
  --morphology research_and_dev/data/quranic-arabic-corpus-morphology.csv \
  --preset basic \
  --content-db basic-content.db \
  --output basic-graph.cbor.zst
```

**Output:**
- `basic-content.db` - SQLite content database
- `basic-graph.cbor.zst` - Structure-only knowledge graph (~5MB)

### Example 2: Full Production Graph

```bash
# Build complete knowledge graph
iqrah build all \
  --morphology research_and_dev/data/quranic-arabic-corpus-morphology.csv \
  --preset full \
  --content-db content.db \
  --output knowledge-graph.cbor.zst
```

**Output:**
- `content.db` - Complete content database
- `knowledge-graph.cbor.zst` - Full knowledge graph with scoring (~10MB)

### Example 3: Research/Development

```bash
# Build research graph (first 10 chapters, experimental features)
iqrah build all \
  --morphology research_and_dev/data/quranic-arabic-corpus-morphology.csv \
  --preset research \
  --content-db research-content.db \
  --output research-graph.cbor.zst
```

## Configuration Presets

### Basic Preset (`config/presets/basic.yaml`)

**Use Case:** Minimal graph for simple learning apps
**Features:**
- Memorization hierarchy (word → verse → chapter)
- Basic translation understanding
- No scoring (lightweight)

**Size:** ~5MB
**Build Time:** ~2-3 minutes

### Full Preset (`config/presets/full.yaml`)

**Use Case:** Production apps
**Features:**
- Complete memorization hierarchy
- Contextual memorization (word windows)
- Translation understanding
- Grammar connections (word-lemma-root)
- Duplicate verse linking
- PageRank scoring (foundational + influence)

**Size:** ~10MB
**Build Time:** ~5-10 minutes

### Research Preset (`config/presets/research.yaml`)

**Use Case:** Research and development
**Features:**
- All full features
- Experimental tajweed edges
- Enhanced scoring (higher iterations)
- Both CBOR and GraphML export

**Size:** ~20MB (both formats)
**Build Time:** ~10-15 minutes

## Step-by-Step Workflow

### Step 1: Build Content Database

```bash
iqrah build content-db \
  -o content.db \
  --morphology data/quranic-arabic-corpus-morphology.csv
```

**What it does:**
- Loads offline Quran data (114 chapters, 6,236 verses)
- Loads morphology corpus (130,030 segments)
- Creates normalized SQLite database
- Populates: chapters, verses, words, translations, transliterations, morphology, lemmas, roots

**Output:** `content.db` (SQLite database, ~50MB)

### Step 2: Build Dependency Graph

```bash
iqrah build dependency-graph \
  -o dependency.graphml \
  --morphology data/quranic-arabic-corpus-morphology.csv \
  --chapters 1-114
```

**What it does:**
- Creates base graph structure
- Adds nodes: chapters, verses, word instances, words, lemmas, roots
- Adds dependency edges (hierarchical and morphological)

**Output:** `dependency.graphml` (GraphML file)

### Step 3: Build Knowledge Graph

```bash
iqrah build knowledge-graph \
  --input dependency.graphml \
  -o knowledge.cbor.zst \
  --preset full
```

**What it does:**
- Loads dependency graph
- Adds knowledge edges based on preset:
  - Memorization edges (hierarchical + contextual)
  - Translation edges (understanding flow)
  - Grammar edges (word-lemma-root connections)
  - Cross-dimensional edges (translation helps memorization)
- Compiles edge weights
- Calculates PageRank scores
- Exports structure-only CBOR

**Output:** `knowledge.cbor.zst` (Compressed CBOR, ~10MB)

### Step 4: Use the Graph

#### Python API

```python
from iqrah.export import import_graph_from_cbor, augment_graph_with_content

# Load structure only
G = import_graph_from_cbor("knowledge.cbor.zst")

# Augment with content (optional)
augment_graph_with_content(G, "content.db", node_types=["word_instance", "verse"])

# Query
for node_id, data in list(G.nodes(data=True))[:10]:
    print(f"{node_id}: {data.get('foundational_score', 'N/A')}")

    if "arabic" in data:  # Content was augmented
        print(f"  Arabic: {data['arabic']}")
```

#### CLI Inspection

```python
from iqrah.export import inspect_cbor_graph

stats = inspect_cbor_graph("knowledge.cbor.zst", sample_size=10)
print(f"Nodes: {stats['header']['graph']['node_count']}")
print(f"Edges: {stats['header']['graph']['edge_count']}")
print(f"Node types: {stats['node_types']}")
```

## Custom Configuration

### Create Custom YAML

```yaml
# custom-config.yaml
name: custom
description: My custom configuration

chapters: "1-10"  # First 10 chapters

memorization:
  enabled: true
  params:
    window_size: 5

translation:
  enabled: true
  params:
    connect_duplicates: true

grammar:
  enabled: false

scoring:
  enabled: true
  alpha: 0.85
  max_iter: 100000

export:
  format: cbor
  compression_level: 9
```

### Use Custom Config

```bash
iqrah build knowledge-graph \
  --from-scratch \
  --morphology data/morphology.csv \
  --config custom-config.yaml \
  -o custom-graph.cbor.zst
```

## Architecture Details

### Content Database Schema

**Normalized tables:**
- `chapters` - Chapter metadata
- `verses` - Verse content and metadata
- `words` - Word instances with positions
- `word_translations` - Word-by-word translations
- `word_transliterations` - Word transliterations
- `verse_translations` - Verse translations
- `morphology` - Morphological segments
- `lemmas` - Dictionary forms
- `roots` - Triliteral/quadriliteral roots

**Indexes:**
- `node_id` (primary key for all tables)
- `verse_key` (for verse-based queries)
- `arabic` (for lemma/root lookups)

**Query Examples:**
```python
from iqrah.content.database import ContentDatabase

with ContentDatabase("content.db") as db:
    # Get word content
    word = db.get_word("WORD_INSTANCE:1:1:1")
    print(word["text_uthmani"])

    # Get translations
    translation = db.get_word_translation("WORD_INSTANCE:1:1:1", "default")
    transliteration = db.get_word_transliteration("WORD_INSTANCE:1:1:1", "en")

    # Bulk queries
    content = db.get_content_for_nodes(["WORD_INSTANCE:1:1:1", "VERSE:1:1"])
```

### CBOR Export Format

**Version 2.0 Format (Structure-Only):**

```
Header:
{
  "v": 2,
  "format": "structure_only",
  "created_at": "2025-01-15T...",
  "graph": {
    "directed": true,
    "node_count": 100000,
    "edge_count": 500000
  }
}

Nodes (structure only):
{
  "t": "node",
  "id": "WORD_INSTANCE:1:1:1",
  "a": {
    "type": "word_instance",
    "verse_key": "1:1",
    "position": 1,
    "foundational_score": 0.8234,
    "influence_score": 0.6421
  }
  # NO "arabic", "translation", "transliteration"
}

Edges:
{
  "t": "edge",
  "u": "WORD_INSTANCE:1:1:1",
  "v": "VERSE:1:1",
  "a": {
    "dist": "auto",
    "weight": 0.5,
    "knowledge_type": "memorization"
  }
}
```

**Removed from export:**
- `arabic` / `text_uthmani`
- `translation`
- `transliteration`
- `audio_url`
- Any other content fields

**Kept in export:**
- Node IDs
- Node types
- Reference keys (verse_key, position)
- Scores (foundational, influence)
- Edge distributions and weights

### PageRank Scoring

**Foundational Score:**
- Measures how fundamental a concept is
- Personalized PageRank with higher weights for roots/lemmas
- Forward propagation on knowledge graph

**Influence Score:**
- Measures downstream impact
- Reverse PageRank on knowledge graph
- Shows how much a node influences others

**Normalization:**
- Log01 transformation: `log1p(score * scale)`
- Min-max normalization to [0, 1]
- Prevents score collapse from power-law distribution

**Node Type Weights (Personalization):**
```python
ROOT: 3.0       # Most fundamental
LEMMA: 2.5
CHAPTER: 2.0
VERSE: 1.5
WORD: 1.0
WORD_INSTANCE: 0.5  # Most specific
```

## Testing

### Run Tests

```bash
cd research_and_dev/iqrah-knowledge-graph2
pytest tests/test_production_workflow.py -v
```

### Test Coverage

- Content database creation and querying
- Dependency graph building
- Knowledge graph building (all edge types)
- PageRank scoring
- CBOR export/import (structure-only validation)
- Complete pipeline integration

## Performance

### Build Times (Full Quran, Full Preset)

| Step | Time | Notes |
|------|------|-------|
| Content DB | ~1-2 min | One-time build |
| Dependency Graph | ~2-3 min | Base structure |
| Knowledge Edges | ~2-3 min | Edge creation |
| Scoring | ~3-5 min | PageRank (100k iterations) |
| CBOR Export | ~30 sec | Compression |
| **Total** | **~8-13 min** | End-to-end |

### File Sizes

| Component | Size | Notes |
|-----------|------|-------|
| Content DB | ~50MB | SQLite database |
| Dependency Graph (GraphML) | ~30MB | XML format |
| Knowledge Graph (CBOR) | ~10MB | Structure + scores |
| Knowledge Graph (GraphML) | ~50MB | With all edges |

## Migration from Notebook Code

### Old Code (Notebook)

```python
# Notebook cell
from iqrah.quran_offline import load_quran_offline
quran = load_quran_offline()

graph = nx.read_graphml("dependency.graphml")
exp = KnowledgeExperiments(graph, quran)

exp.setup_standard_memorization()
exp.setup_translation_understanding()
exp.setup_grammar_nodes()

exp.compile()
exp.save("output.graphml")
```

### New Code (Production)

```python
from iqrah.quran_offline import load_quran_offline
from iqrah.graph.knowledge_builder import KnowledgeGraphBuilder
from iqrah.export import export_graph_to_cbor

quran = load_quran_offline()
graph = nx.read_graphml("dependency.graphml")

# Build knowledge graph
kb = KnowledgeGraphBuilder(graph, quran)
kb.build_all(
    include_memorization=True,
    include_translation=True,
    include_grammar=True,
)
kb.compile()

# Export structure-only CBOR
export_graph_to_cbor(graph, "output.cbor.zst")
```

## Troubleshooting

### Issue: Content database build fails

**Solution:** Ensure offline data is present:
```bash
ls research_and_dev/data/
# Should contain: structural-metadata/, text/, translations/, etc.
```

### Issue: Morphology corpus not found

**Solution:** Download from Quranic Arabic Corpus:
```bash
wget http://corpus.quran.com/data/quranic-corpus-morphology-0.4.txt
mv quranic-corpus-morphology-0.4.txt research_and_dev/data/morphology.csv
```

### Issue: CBOR import shows "content not found"

**Solution:** This is expected! CBOR is structure-only. Augment with content:
```python
from iqrah.export import augment_graph_with_content

augment_graph_with_content(G, "content.db")
```

### Issue: Graph build is slow

**Solutions:**
- Use fewer chapters: `--chapters 1-10`
- Disable scoring: `--no-scoring`
- Use basic preset: `--preset basic`

## Future Enhancements

### Planned Features

1. **Tajweed Integration**
   - Detect tajweed rules
   - Add tajweed edges to knowledge graph
   - Currently placeholder (preset: `tajweed.enabled = false`)

2. **Tafsir Integration**
   - Add tafsir data to content database
   - Deep understanding edges
   - Currently requires external tafsir source

3. **Multiple Translations**
   - Support multiple translations in content DB
   - Translation switching without graph rebuild
   - Schema supports this (translation_id field)

4. **Visualization**
   - Interactive web-based graph visualization
   - Existing `iqrah visualize` command to be updated

5. **API Server**
   - REST API for graph queries
   - Content database as backend
   - Real-time content augmentation

## License

See main repository LICENSE file.

## Contact

For issues or questions, see the main Iqrah repository.
