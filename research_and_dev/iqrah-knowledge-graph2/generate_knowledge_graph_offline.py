#!/usr/bin/env python3
"""
Generate Iqrah Knowledge Graph using offline data only (no API calls).

This script replaces the knowledge_experiments.ipynb notebook functionality
using local JSON data files instead of external APIs.
"""

import sys
import argparse
from pathlib import Path

sys.path.insert(0, 'src')

from loguru import logger
import networkx as nx
from tqdm import tqdm

# Offline data loading
from iqrah.quran_offline import load_quran_offline
from iqrah.morphology.corpus import QuranicArabicCorpus

# Graph building
from iqrah.graph.builder import QuranGraphBuilder
from iqrah.graph.identifiers import NIG, NIP
from iqrah.graph.knowledge import KnowledgeEdgeManager, Distribution
from iqrah.graph.node_manager import NodeManager


def main():
    parser = argparse.ArgumentParser(
        description="Generate Iqrah Knowledge Graph from offline data"
    )
    parser.add_argument(
        "--output",
        "-o",
        default="iqrah_knowledge_graph_offline.graphml",
        help="Output GraphML file path",
    )
    parser.add_argument(
        "--chapters",
        "-c",
        type=int,
        nargs="+",
        help="Specific chapters to process (default: all)",
    )
    parser.add_argument(
        "--morphology-corpus",
        default="../data/morphology/quran-morphology-v0.5.csv",
        help="Path to morphology corpus CSV",
    )
    parser.add_argument(
        "--no-progress",
        action="store_true",
        help="Disable progress bars",
    )
    args = parser.parse_args()

    logger.remove()
    logger.add(sys.stderr, level="INFO")

    # Step 1: Load Quran data from offline sources
    logger.info("Loading Quran data from offline sources...")
    quran = load_quran_offline()
    logger.info(f"✓ Loaded {len(quran.chapters)} chapters, {quran.total_verses()} verses")

    # Step 2: Load morphology corpus
    logger.info(f"Loading morphology corpus from {args.morphology_corpus}...")
    corpus = QuranicArabicCorpus()
    corpus.load_data(args.morphology_corpus)
    logger.info(f"✓ Loaded corpus with {len(corpus)} morphological segments")

    # Step 3: Build dependency graph
    logger.info("Building dependency graph...")
    builder = QuranGraphBuilder()

    chapters_to_process = None
    if args.chapters:
        chapters_to_process = [quran.chapters[i - 1] for i in args.chapters]
        logger.info(f"Processing {len(chapters_to_process)} selected chapters")

    graph = builder.build_graph(
        quran=quran,
        corpus=corpus,
        chapters=chapters_to_process,
        show_progress=not args.no_progress,
    )
    logger.info(f"✓ Created dependency graph: {graph.number_of_nodes()} nodes, {graph.number_of_edges()} edges")

    # Step 4: Add knowledge edges
    logger.info("Adding knowledge edges for learning dynamics...")
    edge_manager = KnowledgeEdgeManager(graph)
    node_manager = NodeManager(graph)

    # Get node counts
    chapters = list(node_manager.get_nodes_by_type("chapter"))
    verses = list(node_manager.get_nodes_by_type("verse"))
    word_instances = list(node_manager.get_nodes_by_type("word_instance"))

    logger.info(f"Graph has {len(chapters)} chapters, {len(verses)} verses, {len(word_instances)} word instances")

    # Add memorization edges
    logger.info("Adding memorization edges...")
    for chapter_id in tqdm(chapters, desc="Memorization edges", disable=args.no_progress):
        chapter = quran[NIP.get_chapter_key(chapter_id)]

        for verse in chapter.verses:
            verse_id = NIG.for_verse(verse)

            # Verse to chapter memorization
            edge_manager.add_knowledge_edge(
                f"{verse_id}:memorization",
                f"{chapter_id}:memorization",
                Distribution.auto(weight=verse.get_letters_count()),
            )

            # Word to verse memorization
            for word in verse.words:
                if word.char_type_name == "end":
                    continue

                word_id = NIG.for_word_instance(word, verse)
                edge_manager.add_knowledge_edge(
                    f"{word_id}:memorization",
                    f"{verse_id}:memorization",
                    Distribution.auto(weight=word.get_letters_count()),
                )

            # Contextual memorization (Gaussian window)
            word_nodes = [
                NIG.for_word_instance(w, verse)
                for w in verse.words
                if w.char_type_name != "end"
            ]
            if word_nodes:
                edge_manager.add_gaussian_window_edges(
                    [f"{w}:memorization" for w in word_nodes],
                    window_size=min(3, len(word_nodes)),
                    base_weight=0.5,
                    std_scale=0.15,
                )

    # Add ayah-to-ayah memorization edges (within chapter)
    logger.info("Adding ayah-to-ayah memorization edges...")
    for chapter_id in tqdm(chapters, desc="Ayah-to-ayah edges", disable=args.no_progress):
        chapter = quran[NIP.get_chapter_key(chapter_id)]

        for i in range(len(chapter.verses) - 1):
            current_verse = chapter.verses[i]
            next_verse = chapter.verses[i + 1]

            current_id = NIG.for_verse(current_verse)
            next_id = NIG.for_verse(next_verse)

            # Sequential verse memorization edge
            edge_manager.add_knowledge_edge(
                f"{current_id}:memorization",
                f"{next_id}:memorization",
                Distribution.beta(alpha=3, beta=1.5),  # Strong sequential dependency
            )

    # Add surah-to-surah memorization edges
    logger.info("Adding surah-to-surah memorization edges...")
    chapter_list = sorted(chapters)
    for i in range(len(chapter_list) - 1):
        current_chapter = chapter_list[i]
        next_chapter = chapter_list[i + 1]

        # Sequential chapter memorization edge
        edge_manager.add_knowledge_edge(
            f"{current_chapter}:memorization",
            f"{next_chapter}:memorization",
            Distribution.beta(alpha=2, beta=2),  # Moderate sequential dependency
        )

    # Add translation edges
    logger.info("Adding translation edges...")
    for chapter_id in tqdm(chapters, desc="Translation edges", disable=args.no_progress):
        chapter = quran[NIP.get_chapter_key(chapter_id)]

        for verse in chapter.verses:
            verse_id = NIG.for_verse(verse)

            # Verse to chapter translation
            edge_manager.add_knowledge_edge(
                f"{verse_id}:translation",
                f"{chapter_id}:translation",
                Distribution.auto(weight=verse.get_words_count()),
            )

            # Word translation edges
            for word in verse.words:
                if word.char_type_name == "end":
                    continue

                word_instance_id = NIG.for_word_instance(word, verse)
                word_type_id = NIG.for_word(word)

                # Word instance to verse translation
                edge_manager.add_knowledge_edge(
                    f"{word_instance_id}:translation",
                    f"{verse_id}:translation",
                    Distribution.auto(weight=word.get_letters_count()),
                )

                # Word instance to word type translation
                edge_manager.add_knowledge_edge(
                    f"{word_instance_id}:translation",
                    f"{word_type_id}:translation",
                    Distribution.normal(mean=0.9, std=0.1),
                )

    # Add grammar edges
    logger.info("Adding grammar edges...")
    for word_id in tqdm(
        list(node_manager.get_nodes_by_type("word")),
        desc="Grammar edges",
        disable=args.no_progress,
    ):
        # Word to lemma
        lemma_ids = node_manager.get_related_nodes(word_id, successor_type="lemma")
        for lemma_id in lemma_ids:
            _, lemma = lemma_id.split(":", 1)
            edge_manager.add_bidirectional_knowledge_edge(
                f"{word_id}:translation",
                f"{lemma_id}:translation",
                Distribution.auto(weight=len(lemma)),
            )

            # Lemma to root
            root_ids = node_manager.get_related_nodes(lemma_id, successor_type="root")
            for root_id in root_ids:
                edge_manager.add_bidirectional_knowledge_edge(
                    f"{lemma_id}:translation",
                    f"{root_id}:meaning",
                    Distribution.beta(alpha=4, beta=1.5),
                )

    # Add cross-dimension learning edges
    logger.info("Adding cross-dimension learning edges...")
    translatable_nodes = (
        node_manager.get_nodes_by_type("word_instance")
        | node_manager.get_nodes_by_type("verse")
        | node_manager.get_nodes_by_type("chapter")
    )
    for node_id in tqdm(translatable_nodes, desc="Cross-dimension", disable=args.no_progress):
        # Translation helps memorization
        edge_manager.add_knowledge_edge(
            f"{node_id}:translation",
            f"{node_id}:memorization",
            Distribution.beta(alpha=3, beta=2),
        )

    # Compile all knowledge edges
    logger.info("Compiling knowledge edges...")
    edge_manager.compile()

    logger.info(f"✓ Final graph: {graph.number_of_nodes()} nodes, {graph.number_of_edges()} edges")

    # Save graph
    logger.info(f"Saving knowledge graph to {args.output}...")
    nx.write_graphml(graph, args.output)
    logger.info(f"✅ Knowledge graph saved successfully!")

    # Print summary
    print("\n" + "=" * 60)
    print("KNOWLEDGE GRAPH GENERATION COMPLETE")
    print("=" * 60)
    print(f"Output file: {args.output}")
    print(f"Nodes: {graph.number_of_nodes():,}")
    print(f"Edges: {graph.number_of_edges():,}")
    print(f"Chapters: {len(chapters)}")
    print(f"Verses: {len(verses)}")
    print(f"Word instances: {len(word_instances)}")
    print("=" * 60)


if __name__ == "__main__":
    main()
