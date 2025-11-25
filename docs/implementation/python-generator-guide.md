# Python Knowledge Graph Generator: Two-Phase Implementation

**Date**: 2025-01-25
**Status**: Implementation Specification
**Location**: `research_and_dev/iqrah-knowledge-graph2/`

---

## Overview

The Python knowledge graph generator must be updated to use the two-phase approach:
1. **Phase 1**: Register all nodes and get INTEGER IDs
2. **Phase 2**: Create edges using INTEGER IDs

This ensures referential integrity and enables integer-based graph operations in Rust.

---

## Architecture Changes

### Before (String-based):
```python
# Single-phase: Insert scores and edges directly
for verse in verses:
    for axis in axes:
        node_id = f"VERSE:{verse.chapter}:{verse.num}:{axis}"

        # Insert scores and edges in one pass
        insert_score(node_id, "foundational_score", score)
        insert_edge(node_id, next_node_id)
```

### After (Integer-based):
```python
# Phase 1: Register all nodes, get integer IDs
node_id_map = {}  # ukey -> id
for verse in verses:
    for axis in axes:
        ukey = f"VERSE:{verse.chapter}:{verse.num}:{axis}"
        node_id = register_node(ukey, node_type=NodeType.KNOWLEDGE)
        node_id_map[ukey] = node_id

# Phase 2: Create edges using integer IDs
for source_ukey, target_ukey in edge_pairs:
    add_edge(
        source_id=node_id_map[source_ukey],
        target_id=node_id_map[target_ukey],
        edge_type=EdgeType.KNOWLEDGE,
        weight=0.8
    )
```

---

## Implementation Details

### File: `research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/builder.py`

#### Add Enum Definitions

```python
from enum import IntEnum

class NodeType(IntEnum):
    """Maps to Rust NodeType enum"""
    VERSE = 0
    CHAPTER = 1
    WORD = 2
    KNOWLEDGE = 3
    WORD_INSTANCE = 4

class KnowledgeAxis(IntEnum):
    """Maps to Rust KnowledgeAxis enum"""
    MEMORIZATION = 0
    TRANSLATION = 1
    TAFSIR = 2
    TAJWEED = 3
    CONTEXTUAL_MEMORIZATION = 4
    MEANING = 5

class EdgeType(IntEnum):
    """Maps to Rust EdgeType enum"""
    DEPENDENCY = 0
    KNOWLEDGE = 1
```

#### Update KnowledgeGraphBuilder

```python
class KnowledgeGraphBuilder:
    def __init__(self):
        self.nodes: Dict[str, int] = {}  # ukey -> id
        self.edges: List[Tuple[int, int, int, float]] = []  # (source_id, target_id, edge_type, weight)
        self.knowledge_nodes: List[Tuple[int, int, int]] = []  # (node_id, base_node_id, axis)
        self.node_metadata: Dict[int, Dict[str, float]] = {}  # node_id -> {key: value}
        self.node_goals: Dict[str, List[Tuple[int, int]]] = {}  # goal_id -> [(node_id, priority)]
        self.next_node_id = 1

    def register_node(self, ukey: str, node_type: NodeType) -> int:
        """Register a node and return its integer ID"""
        if ukey in self.nodes:
            return self.nodes[ukey]

        node_id = self.next_node_id
        self.next_node_id += 1
        self.nodes[ukey] = node_id

        return node_id

    def add_knowledge_node(self, node_id: int, base_node_id: int, axis: KnowledgeAxis):
        """Define a knowledge node"""
        self.knowledge_nodes.append((node_id, base_node_id, axis))

    def add_edge(self, source_id: int, target_id: int, edge_type: EdgeType, weight: float):
        """Add an edge between nodes"""
        self.edges.append((source_id, target_id, edge_type, weight))

    def add_metadata(self, node_id: int, key: str, value: float):
        """Add metadata to a node"""
        if node_id not in self.node_metadata:
            self.node_metadata[node_id] = {}
        self.node_metadata[node_id][key] = value

    def add_node_goal(self, goal_id: str, node_id: int, priority: int = 0):
        """Associate a node with a learning goal"""
        if goal_id not in self.node_goals:
            self.node_goals[goal_id] = []
        self.node_goals[goal_id].append((node_id, priority))
```

#### Two-Phase Build Method

```python
def build_knowledge_graph(self, chapters: List[int]) -> KnowledgeGraph:
    """Build knowledge graph using two-phase approach"""

    # ==================== PHASE 1: Node Registration ====================
    print("Phase 1: Registering nodes...")

    # Step 1.1: Register chapter nodes
    for chapter_num in chapters:
        ukey = f"CHAPTER:{chapter_num}"
        node_id = self.register_node(ukey, NodeType.CHAPTER)
        print(f"  Registered {ukey} -> {node_id}")

    # Step 1.2: Register verse nodes
    verses = self.load_verses(chapters)
    for verse in verses:
        ukey = f"VERSE:{verse.chapter}:{verse.num}"
        node_id = self.register_node(ukey, NodeType.VERSE)
        print(f"  Registered {ukey} -> {node_id}")

    # Step 1.3: Register knowledge nodes (verse-level)
    verse_axes = [
        KnowledgeAxis.MEMORIZATION,
        KnowledgeAxis.TRANSLATION,
        KnowledgeAxis.TAFSIR,
        KnowledgeAxis.TAJWEED,
    ]

    for verse in verses:
        verse_ukey = f"VERSE:{verse.chapter}:{verse.num}"
        verse_node_id = self.nodes[verse_ukey]

        for axis in verse_axes:
            # Register knowledge node
            kn_ukey = f"{verse_ukey}:{axis.name.lower()}"
            kn_node_id = self.register_node(kn_ukey, NodeType.KNOWLEDGE)

            # Link to base node
            self.add_knowledge_node(kn_node_id, verse_node_id, axis)

            print(f"  Registered {kn_ukey} -> {kn_node_id} (base: {verse_node_id})")

    # Step 1.4: Register word nodes
    words = self.load_words(chapters)
    for word in words:
        ukey = f"WORD_INSTANCE:{word.verse_key}:{word.position}"
        node_id = self.register_node(ukey, NodeType.WORD_INSTANCE)
        print(f"  Registered {ukey} -> {node_id}")

    # Step 1.5: Register word-level knowledge nodes
    word_axes = [
        KnowledgeAxis.CONTEXTUAL_MEMORIZATION,
        KnowledgeAxis.MEANING,
    ]

    for word in words:
        word_ukey = f"WORD_INSTANCE:{word.verse_key}:{word.position}"
        word_node_id = self.nodes[word_ukey]

        for axis in word_axes:
            kn_ukey = f"{word_ukey}:{axis.name.lower()}"
            kn_node_id = self.register_node(kn_ukey, NodeType.KNOWLEDGE)

            self.add_knowledge_node(kn_node_id, word_node_id, axis)

            print(f"  Registered {kn_ukey} -> {kn_node_id} (base: {word_node_id})")

    print(f"Phase 1 complete: {len(self.nodes)} nodes registered\n")

    # ==================== PHASE 2: Edge Creation ====================
    print("Phase 2: Creating edges...")

    # Step 2.1: Sequential verse dependencies
    for i in range(len(verses) - 1):
        curr_verse = verses[i]
        next_verse = verses[i + 1]

        # For each knowledge axis, create edge between knowledge nodes
        for axis in verse_axes:
            curr_ukey = f"VERSE:{curr_verse.chapter}:{curr_verse.num}:{axis.name.lower()}"
            next_ukey = f"VERSE:{next_verse.chapter}:{next_verse.num}:{axis.name.lower()}"

            curr_id = self.nodes[curr_ukey]
            next_id = self.nodes[next_ukey]

            self.add_edge(curr_id, next_id, EdgeType.DEPENDENCY, weight=0.8)
            print(f"  Edge: {curr_id} -> {next_id} (sequential)")

    # Step 2.2: Cross-axis dependencies within same verse
    for verse in verses:
        verse_ukey = f"VERSE:{verse.chapter}:{verse.num}"

        # memorization -> translation
        mem_id = self.nodes[f"{verse_ukey}:memorization"]
        trans_id = self.nodes[f"{verse_ukey}:translation"]
        self.add_edge(mem_id, trans_id, EdgeType.KNOWLEDGE, weight=0.7)

        # translation -> tafsir
        tafsir_id = self.nodes[f"{verse_ukey}:tafsir"]
        self.add_edge(trans_id, tafsir_id, EdgeType.KNOWLEDGE, weight=0.6)

        # memorization -> tajweed
        tajweed_id = self.nodes[f"{verse_ukey}:tajweed"]
        self.add_edge(mem_id, tajweed_id, EdgeType.KNOWLEDGE, weight=0.5)

        print(f"  Cross-axis edges for {verse_ukey}")

    # Step 2.3: Word-to-verse knowledge edges
    for word in words:
        word_ukey = f"WORD_INSTANCE:{word.verse_key}:{word.position}"
        verse_ukey = f"VERSE:{word.verse_key}"

        # contextual_memorization (word) -> memorization (verse)
        word_cm_id = self.nodes[f"{word_ukey}:contextual_memorization"]
        verse_mem_id = self.nodes[f"{verse_ukey}:memorization"]
        self.add_edge(word_cm_id, verse_mem_id, EdgeType.DEPENDENCY, weight=0.9)

        # meaning (word) -> translation (verse)
        word_meaning_id = self.nodes[f"{word_ukey}:meaning"]
        verse_trans_id = self.nodes[f"{verse_ukey}:translation"]
        self.add_edge(word_meaning_id, verse_trans_id, EdgeType.DEPENDENCY, weight=0.9)

    print(f"Phase 2 complete: {len(self.edges)} edges created\n")

    # ==================== PHASE 3: Metadata & Goals ====================
    print("Phase 3: Adding metadata and goals...")

    # Calculate foundational scores using PageRank-like algorithm
    scores = self.calculate_foundational_scores()
    for node_id, score in scores.items():
        self.add_metadata(node_id, "foundational_score", score)
        self.add_metadata(node_id, "influence_score", score * 0.5)
        self.add_metadata(node_id, "difficulty_score", 0.5)  # Default

    # Add nodes to goals
    for verse in verses:
        verse_ukey = f"VERSE:{verse.chapter}:{verse.num}"
        goal_id = f"chapter_{verse.chapter}"

        for axis in verse_axes:
            kn_ukey = f"{verse_ukey}:{axis.name.lower()}"
            kn_node_id = self.nodes[kn_ukey]
            self.add_node_goal(goal_id, kn_node_id, priority=10)

    print(f"Phase 3 complete: {len(self.node_metadata)} nodes with metadata\n")

    # Return built graph
    return KnowledgeGraph(
        nodes=self.nodes,
        edges=self.edges,
        knowledge_nodes=self.knowledge_nodes,
        node_metadata=self.node_metadata,
        node_goals=self.node_goals,
    )
```

---

## SQL Export

### File: `research_and_dev/iqrah-knowledge-graph2/src/iqrah/exporter/sql_exporter.py`

```python
class SQLExporter:
    def export(self, graph: KnowledgeGraph, output_path: str):
        """Export knowledge graph to SQL file"""

        with open(output_path, 'w') as f:
            # Preamble
            f.write("-- Knowledge Graph SQL Export\n")
            f.write("-- Generated: " + datetime.now().isoformat() + "\n\n")
            f.write("PRAGMA foreign_keys = ON;\n\n")

            # Insert nodes
            f.write("-- Insert nodes\n")
            f.write("BEGIN TRANSACTION;\n")
            for ukey, node_id in graph.nodes.items():
                node_type = self.infer_node_type(ukey)
                f.write(f"INSERT INTO nodes (id, ukey, node_type) VALUES ({node_id}, '{ukey}', {node_type});\n")
            f.write("COMMIT;\n\n")

            # Insert knowledge nodes
            f.write("-- Insert knowledge nodes\n")
            f.write("BEGIN TRANSACTION;\n")
            for node_id, base_node_id, axis in graph.knowledge_nodes:
                f.write(f"INSERT INTO knowledge_nodes (node_id, base_node_id, axis) VALUES ({node_id}, {base_node_id}, {axis});\n")
            f.write("COMMIT;\n\n")

            # Insert edges
            f.write("-- Insert edges\n")
            f.write("BEGIN TRANSACTION;\n")
            for source_id, target_id, edge_type, weight in graph.edges:
                f.write(f"INSERT INTO edges (source_id, target_id, edge_type, weight) VALUES ({source_id}, {target_id}, {edge_type}, {weight});\n")
            f.write("COMMIT;\n\n")

            # Insert metadata
            f.write("-- Insert node metadata\n")
            f.write("BEGIN TRANSACTION;\n")
            for node_id, metadata in graph.node_metadata.items():
                for key, value in metadata.items():
                    f.write(f"INSERT INTO node_metadata (node_id, key, value) VALUES ({node_id}, '{key}', {value});\n")
            f.write("COMMIT;\n\n")

            # Insert goals
            f.write("-- Insert node goals\n")
            f.write("BEGIN TRANSACTION;\n")
            for goal_id, nodes in graph.node_goals.items():
                for node_id, priority in nodes:
                    f.write(f"INSERT INTO node_goals (goal_id, node_id, priority) VALUES ('{goal_id}', {node_id}, {priority});\n")
            f.write("COMMIT;\n\n")

        print(f"Exported graph to {output_path}")
```

---

## CBOR Export (Binary Format)

### File: `research_and_dev/iqrah-knowledge-graph2/src/iqrah/exporter/cbor_exporter.py`

```python
import cbor2
from pathlib import Path

class CBORExporter:
    def export(self, graph: KnowledgeGraph, output_path: str):
        """Export knowledge graph to CBOR binary format"""

        # Serialize to CBOR
        data = {
            'nodes': [
                {'id': node_id, 'ukey': ukey, 'node_type': self.infer_node_type(ukey)}
                for ukey, node_id in graph.nodes.items()
            ],
            'knowledge_nodes': [
                {'node_id': node_id, 'base_node_id': base_id, 'axis': axis}
                for node_id, base_id, axis in graph.knowledge_nodes
            ],
            'edges': [
                {'source_id': src, 'target_id': tgt, 'edge_type': et, 'weight': w}
                for src, tgt, et, w in graph.edges
            ],
            'node_metadata': {
                str(node_id): metadata
                for node_id, metadata in graph.node_metadata.items()
            },
            'node_goals': {
                goal_id: [{'node_id': nid, 'priority': p} for nid, p in nodes]
                for goal_id, nodes in graph.node_goals.items()
            }
        }

        # Write to file
        with open(output_path, 'wb') as f:
            cbor2.dump(data, f)

        # Print stats
        file_size = Path(output_path).stat().st_size
        print(f"Exported graph to {output_path} ({file_size / 1024:.1f} KB)")
```

---

## CLI Integration

### File: `research_and_dev/iqrah-knowledge-graph2/src/iqrah/cli/__main__.py`

```python
import argparse

def main():
    parser = argparse.ArgumentParser(description='Iqrah Knowledge Graph Generator')
    subparsers = parser.add_subparsers(dest='command')

    # Build command
    build_parser = subparsers.add_parser('build', help='Build knowledge graph')
    build_parser.add_argument('--chapters', required=True, help='Chapter range (e.g., 1-3)')
    build_parser.add_argument('--format', choices=['sql', 'cbor', 'both'], default='both')
    build_parser.add_argument('--output-dir', default='output/')

    args = parser.parse_args()

    if args.command == 'build':
        # Parse chapter range
        start, end = map(int, args.chapters.split('-'))
        chapters = list(range(start, end + 1))

        # Build graph
        builder = KnowledgeGraphBuilder()
        graph = builder.build_knowledge_graph(chapters)

        # Export
        output_dir = Path(args.output_dir)
        output_dir.mkdir(exist_ok=True)

        if args.format in ['sql', 'both']:
            sql_path = output_dir / f'knowledge_graph_{start}_{end}.sql'
            SQLExporter().export(graph, str(sql_path))

        if args.format in ['cbor', 'both']:
            cbor_path = output_dir / f'knowledge_graph_{start}_{end}.cbor'
            CBORExporter().export(graph, str(cbor_path))

        print(f"\n✓ Knowledge graph built successfully!")
        print(f"  Nodes: {len(graph.nodes)}")
        print(f"  Edges: {len(graph.edges)}")
        print(f"  Knowledge nodes: {len(graph.knowledge_nodes)}")

if __name__ == '__main__':
    main()
```

---

## Validation

### Referential Integrity Checks

```python
def validate_graph(graph: KnowledgeGraph) -> bool:
    """Validate graph referential integrity before export"""

    errors = []

    # Check: All edges reference registered nodes
    for source_id, target_id, _, _ in graph.edges:
        if source_id not in [nid for nid in graph.nodes.values()]:
            errors.append(f"Edge source {source_id} not in nodes")
        if target_id not in [nid for nid in graph.nodes.values()]:
            errors.append(f"Edge target {target_id} not in nodes")

    # Check: All knowledge nodes reference registered nodes
    for node_id, base_id, _ in graph.knowledge_nodes:
        if node_id not in [nid for nid in graph.nodes.values()]:
            errors.append(f"Knowledge node {node_id} not in nodes")
        if base_id not in [nid for nid in graph.nodes.values()]:
            errors.append(f"Knowledge base {base_id} not in nodes")

    # Check: All metadata references registered nodes
    for node_id in graph.node_metadata.keys():
        if node_id not in [nid for nid in graph.nodes.values()]:
            errors.append(f"Metadata node {node_id} not in nodes")

    if errors:
        print("❌ Validation FAILED:")
        for error in errors:
            print(f"  - {error}")
        return False

    print("✓ Validation passed!")
    return True
```

---

## Migration Checklist

- [ ] Add enum definitions (NodeType, KnowledgeAxis, EdgeType)
- [ ] Update KnowledgeGraphBuilder with node registration
- [ ] Implement two-phase build method
- [ ] Update SQL exporter to use integer IDs
- [ ] Implement CBOR exporter
- [ ] Add validation checks
- [ ] Update CLI to use new builder
- [ ] Test with chapters 1-3
- [ ] Verify generated SQL imports correctly

---

## Expected Output

### For Chapters 1-3:

**Nodes**: ~2000
- 3 chapters
- ~143 verses
- ~500 words
- ~572 knowledge nodes (143 verses × 4 axes)
- ~1000 word knowledge nodes (500 words × 2 axes)

**Edges**: ~5000+
- Sequential dependencies
- Cross-axis edges
- Word-to-verse edges

**File Sizes**:
- SQL: ~5-10 MB
- CBOR: ~500 KB - 1 MB

---

## References

- [Schema Design](schema-design.md) - Database schema DDL
- [Rust Implementation Guide](rust-implementation-guide.md) - Repository layer
- [Enum Mappings](../reference/enum-mappings.md) - Integer enum values
