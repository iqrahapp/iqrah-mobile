#!/usr/bin/env python3
"""
Test script to verify the notebook flow works with offline data.
This script simulates the key parts of the knowledge_experiments.ipynb notebook.
"""

import sys
sys.path.insert(0, 'src')

from loguru import logger
import networkx as nx

# Test 1: Load offline Quran data
print("\n=== Test 1: Loading Quran Data ===")
from iqrah.quran_offline import load_quran_offline

logger.remove()
quran = load_quran_offline()
print(f"✓ Loaded {len(quran.chapters)} chapters, {quran.total_verses()} verses")
print(f"✓ First chapter: {quran.chapters[0].name_simple}")
print(f"✓ Can access verse: {quran['1:1'].verse_key}")

# Test 2: Load morphology corpus
print("\n=== Test 2: Loading Morphology Corpus ===")
from iqrah.morphology.corpus import QuranicArabicCorpus

corpus = QuranicArabicCorpus()
# Load the full corpus
corpus_path = "../data/morphology/quran-morphology-v0.5.csv"
print(f"Loading morphology corpus from {corpus_path}...")
corpus.load_data(corpus_path)
print(f"✓ Loaded corpus with {len(corpus)} segments")

# Test 3: Build dependency graph
print("\n=== Test 3: Building Dependency Graph ===")
from iqrah.graph.builder import QuranGraphBuilder
from iqrah.graph.identifiers import NIG, NIP
from iqrah.graph.knowledge import KnowledgeEdgeManager, Distribution
from iqrah.graph.node_manager import NodeManager

# Create graph builder
builder = QuranGraphBuilder()

# Build graph for just the first chapter (for testing)
print("Building dependency graph for Al-Fatihah...")
graph = builder.build_graph(
    quran=quran,
    corpus=corpus,
    chapters=[quran.chapters[0]],  # Just Al-Fatihah
    show_progress=True
)
print(f"✓ Created graph with {graph.number_of_nodes()} nodes, {graph.number_of_edges()} edges")

# Test 4: Knowledge edges
print("\n=== Test 4: Knowledge Edge Management ===")
edge_manager = KnowledgeEdgeManager(graph)
node_manager = NodeManager(graph)

# Get some nodes
verse_nodes = list(node_manager.get_nodes_by_type("verse"))
print(f"✓ Found {len(verse_nodes)} verse nodes")

word_instance_nodes = list(node_manager.get_nodes_by_type("word_instance"))
print(f"✓ Found {len(word_instance_nodes)} word instance nodes")

# Add a simple knowledge edge as a test
if verse_nodes and word_instance_nodes:
    # Add memorization edges (simplified version)
    for verse_id in verse_nodes[:1]:  # Just test with first verse
        verse = quran[NIP.get_verse_key(verse_id)]

        # Test adding a knowledge edge
        for word in verse.words[:2]:  # Just first 2 words
            word_id = NIG.for_word_instance(word, verse)
            if graph.has_node(word_id):
                edge_manager.add_knowledge_edge(
                    f"{word_id}:memorization",
                    f"{verse_id}:memorization",
                    Distribution.auto(weight=word.get_letters_count())
                )

    print(f"✓ Added test knowledge edges")

    # Compile edges
    edge_manager.compile()
    print(f"✓ Compiled knowledge edges")
    print(f"✓ Final graph: {graph.number_of_nodes()} nodes, {graph.number_of_edges()} edges")

print("\n=== All Tests Passed ===")
print("✅ The notebook flow should work with offline data!")
