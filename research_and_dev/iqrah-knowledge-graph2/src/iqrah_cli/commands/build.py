# src/iqrah_cli/commands/build.py

import sys
import logging
from loguru import logger
from pathlib import Path
import networkx as nx
from typing import Optional, List

from iqrah.quran_offline import load_quran_offline
from iqrah.morphology.corpus import QuranicArabicCorpus
from iqrah.graph.builder import QuranGraphBuilder
from iqrah.graph.knowledge_builder import KnowledgeGraphBuilder
from iqrah.graph.scoring import calculate_knowledge_scores
from iqrah.graph.statistics import compute_graph_statistics
from iqrah.content.builder import ContentDatabaseBuilder
from iqrah.export import export_graph_to_cbor
from iqrah.config import load_config, load_preset, get_available_presets


def setup_parser(subparsers):
    """Setup CLI parsers for build commands."""
    parser = subparsers.add_parser(
        "build",
        help="Build Quranic knowledge graphs and content databases"
    )

    # Create sub-commands for different build operations
    build_subparsers = parser.add_subparsers(dest="build_command", required=True)

    # Content database command
    _setup_content_db_parser(build_subparsers)

    # Dependency graph command
    _setup_dependency_graph_parser(build_subparsers)

    # Knowledge graph command
    _setup_knowledge_graph_parser(build_subparsers)

    # All-in-one command
    _setup_all_parser(build_subparsers)


def _setup_content_db_parser(subparsers):
    """Setup parser for content database building."""
    parser = subparsers.add_parser(
        "content-db",
        help="Build content database from offline Quran data"
    )

    parser.add_argument(
        "-o", "--output",
        type=str,
        required=True,
        help="Output SQLite database path (e.g., content.db)"
    )

    parser.add_argument(
        "--morphology",
        type=str,
        help="Path to morphology corpus CSV file (optional)"
    )

    parser.add_argument(
        "--data-dir",
        type=str,
        help="Path to offline data directory (optional, uses default)"
    )

    parser.add_argument(
        "--no-progress",
        action="store_true",
        help="Disable progress bars"
    )


def _setup_dependency_graph_parser(subparsers):
    """Setup parser for dependency graph building."""
    parser = subparsers.add_parser(
        "dependency-graph",
        help="Build dependency graph (chapters -> verses -> words -> lemmas -> roots)"
    )

    parser.add_argument(
        "-o", "--output",
        type=str,
        required=True,
        help="Output graph file path (e.g., dependency.graphml)"
    )

    parser.add_argument(
        "--morphology",
        type=str,
        required=True,
        help="Path to morphology corpus CSV file"
    )

    parser.add_argument(
        "--chapters",
        type=str,
        help="Chapter range (e.g., '1-114', '1', '1-10,20'). Default: all chapters"
    )

    parser.add_argument(
        "--format",
        type=str,
        choices=["graphml", "gexf"],
        default="graphml",
        help="Output format (default: graphml)"
    )

    parser.add_argument(
        "--no-progress",
        action="store_true",
        help="Disable progress bars"
    )


def _setup_knowledge_graph_parser(subparsers):
    """Setup parser for knowledge graph building."""
    parser = subparsers.add_parser(
        "knowledge-graph",
        help="Build knowledge graph with learning edges"
    )

    # Input options (mutually exclusive)
    input_group = parser.add_mutually_exclusive_group(required=True)

    input_group.add_argument(
        "--input",
        type=str,
        help="Input dependency graph file (GraphML)"
    )

    input_group.add_argument(
        "--from-scratch",
        action="store_true",
        help="Build from scratch (no input graph)"
    )

    # Output
    parser.add_argument(
        "-o", "--output",
        type=str,
        required=True,
        help="Output file path (e.g., knowledge.cbor.zst or knowledge.graphml)"
    )

    # Configuration
    config_group = parser.add_mutually_exclusive_group()

    config_group.add_argument(
        "--preset",
        type=str,
        choices=["basic", "full", "research"],
        help="Use a preset configuration (basic, full, research)"
    )

    config_group.add_argument(
        "--config",
        type=str,
        help="Path to custom YAML configuration file"
    )

    # Build from scratch options
    parser.add_argument(
        "--morphology",
        type=str,
        help="Path to morphology corpus (required if --from-scratch)"
    )

    parser.add_argument(
        "--chapters",
        type=str,
        help="Chapter range (e.g., '1-114'). Default: from config or all"
    )

    # Edge type overrides
    parser.add_argument(
        "--no-memorization",
        action="store_true",
        help="Disable memorization edges"
    )

    parser.add_argument(
        "--no-translation",
        action="store_true",
        help="Disable translation edges"
    )

    parser.add_argument(
        "--no-grammar",
        action="store_true",
        help="Disable grammar edges"
    )

    parser.add_argument(
        "--enable-tajweed",
        action="store_true",
        help="Enable tajweed edges (experimental)"
    )

    parser.add_argument(
        "--enable-deep-understanding",
        action="store_true",
        help="Enable deep understanding edges"
    )

    # Scoring
    parser.add_argument(
        "--no-scoring",
        action="store_true",
        help="Disable PageRank scoring"
    )

    # Format
    parser.add_argument(
        "--format",
        type=str,
        choices=["cbor", "graphml", "both"],
        help="Output format (default: inferred from output file)"
    )

    parser.add_argument(
        "--no-progress",
        action="store_true",
        help="Disable progress bars"
    )


def _setup_all_parser(subparsers):
    """Setup parser for building everything."""
    parser = subparsers.add_parser(
        "all",
        help="Build everything (content DB + dependency graph + knowledge graph)"
    )

    parser.add_argument(
        "--morphology",
        type=str,
        required=True,
        help="Path to morphology corpus CSV file"
    )

    parser.add_argument(
        "--content-db",
        type=str,
        default="content.db",
        help="Output content database path (default: content.db)"
    )

    parser.add_argument(
        "--output",
        type=str,
        default="knowledge-graph.cbor.zst",
        help="Output knowledge graph path (default: knowledge-graph.cbor.zst)"
    )

    parser.add_argument(
        "--preset",
        type=str,
        choices=["basic", "full", "research"],
        default="full",
        help="Configuration preset (default: full)"
    )

    parser.add_argument(
        "--chapters",
        type=str,
        help="Chapter range (e.g., '1-114'). Default: from preset"
    )

    parser.add_argument(
        "--no-progress",
        action="store_true",
        help="Disable progress bars"
    )

    parser.add_argument(
        "--resume",
        action="store_true",
        help="Resume from last completed step (skip existing outputs)"
    )


def build_content_db(args) -> None:
    """Build content database."""
    logger.info("Building content database...")

    builder = ContentDatabaseBuilder(data_dir=args.data_dir if hasattr(args, 'data_dir') else None)

    builder.build(
        output_path=args.output,
        morphology_corpus_path=args.morphology if hasattr(args, 'morphology') else None,
        show_progress=not args.no_progress,
    )

    logger.success(f"Content database created: {args.output}")


def build_dependency_graph(args) -> nx.DiGraph:
    """Build dependency graph."""
    logger.info("Building dependency graph...")

    # Load Quran data
    logger.info("Loading Quran data from offline sources...")
    quran = load_quran_offline()
    logger.success(f"Loaded {len(quran.chapters)} chapters, {quran.total_verses()} verses")

    # Load morphology
    logger.info(f"Loading morphology corpus from {args.morphology}")
    corpus = QuranicArabicCorpus()
    corpus.load_data(args.morphology)
    logger.success(f"Loaded {len(corpus.segments)} morphology segments")

    # Parse chapters
    if hasattr(args, 'chapters') and args.chapters:
        from iqrah.config import KnowledgeGraphConfig
        config = KnowledgeGraphConfig(chapters=args.chapters)
        chapter_numbers = config.parse_chapters()
        chapters = [c for c in quran.chapters if c.id in chapter_numbers]
        logger.info(f"Building graph for chapters: {chapter_numbers}")
    else:
        chapters = quran.chapters
        logger.info("Building graph for all 114 chapters")

    # Build dependency graph
    builder = QuranGraphBuilder()
    G = builder.build_graph(
        quran=quran,
        corpus=corpus,
        chapters=chapters,
        show_progress=not args.no_progress
    )

    logger.success(f"Dependency graph built: {len(G.nodes)} nodes, {len(G.edges)} edges")

    # Save if output specified
    if hasattr(args, 'output') and args.output:
        output_format = args.format if hasattr(args, 'format') else "graphml"

        logger.info(f"Saving dependency graph to {args.output}...")
        if output_format == "graphml":
            nx.write_graphml(G, args.output)
        elif output_format == "gexf":
            nx.write_gexf(G, args.output)

        logger.success(f"Dependency graph saved: {args.output}")

        # Compute basic statistics for dependency graph
        logger.info("Computing dependency graph statistics...")
        stats_path = args.output.replace(".graphml", ".stats.json").replace(".gexf", ".stats.json")
        compute_graph_statistics(
            G,
            export_path=stats_path,
            print_summary=False  # Don't print full summary for dependency graph
        )
        logger.info(f"Basic statistics: {len(G.nodes)} nodes, {len(G.edges)} edges")
        logger.success(f"Statistics saved: {stats_path}")

    return G


def build_knowledge_graph(args) -> nx.DiGraph:
    """Build knowledge graph."""
    logger.info("Building knowledge graph...")

    # Load configuration
    if hasattr(args, 'config') and args.config:
        logger.info(f"Loading config from {args.config}")
        config = load_config(args.config)
    elif hasattr(args, 'preset') and args.preset:
        logger.info(f"Loading preset: {args.preset}")
        config = load_preset(args.preset)
    else:
        logger.info("Using default configuration")
        from iqrah.config import KnowledgeGraphConfig
        config = KnowledgeGraphConfig()

    # Override chapters if specified
    if hasattr(args, 'chapters') and args.chapters:
        config.chapters = args.chapters

    # Load or create dependency graph
    if hasattr(args, 'input') and args.input:
        logger.info(f"Loading dependency graph from {args.input}")
        G = nx.read_graphml(args.input)

        # Need Quran data for knowledge building
        logger.info("Loading Quran data...")
        quran = load_quran_offline()
    elif hasattr(args, 'from_scratch') and args.from_scratch:
        if not args.morphology:
            logger.error("--morphology is required when using --from-scratch")
            sys.exit(1)

        logger.info("Building dependency graph from scratch...")
        G = build_dependency_graph(args)

        # Quran already loaded in build_dependency_graph
        quran = load_quran_offline()
    else:
        logger.error("Either --input or --from-scratch must be specified")
        sys.exit(1)

    # Build knowledge edges
    logger.info("Adding knowledge edges...")
    kb = KnowledgeGraphBuilder(G, quran)

    # Apply CLI overrides to config
    if hasattr(args, 'no_memorization') and args.no_memorization:
        config.memorization.enabled = False
    if hasattr(args, 'no_translation') and args.no_translation:
        config.translation.enabled = False
    if hasattr(args, 'no_grammar') and args.no_grammar:
        config.grammar.enabled = False
    if hasattr(args, 'enable_tajweed') and args.enable_tajweed:
        config.tajweed.enabled = True
    if hasattr(args, 'enable_deep_understanding') and args.enable_deep_understanding:
        config.deep_understanding.enabled = True

    # Build edges based on config
    kb.build_all(
        include_memorization=config.memorization.enabled,
        include_tajweed=config.tajweed.enabled,
        include_translation=config.translation.enabled,
        include_grammar=config.grammar.enabled,
        include_deep_understanding=config.deep_understanding.enabled,
    )

    # Compile
    logger.info("Compiling knowledge graph...")
    kb.compile()

    # Scoring
    if config.scoring.enabled and not (hasattr(args, 'no_scoring') and args.no_scoring):
        logger.info("Calculating knowledge scores...")
        calculate_knowledge_scores(
            G,
            alpha=config.scoring.alpha,
            max_iter=config.scoring.max_iter,
            personalize_foundational=config.scoring.personalize_foundational,
            personalize_influence=config.scoring.personalize_influence,
        )

    logger.success(f"Knowledge graph complete: {len(G.nodes)} nodes, {len(G.edges)} edges")

    # Determine output format
    output_format = None
    if hasattr(args, 'format') and args.format:
        output_format = args.format
    else:
        # Infer from extension
        if args.output.endswith('.cbor.zst') or args.output.endswith('.cbor'):
            output_format = "cbor"
        elif args.output.endswith('.graphml'):
            output_format = "graphml"
        else:
            output_format = config.export.format

    # Save
    logger.info(f"Exporting graph (format: {output_format})...")

    if output_format in ("cbor", "both"):
        cbor_path = args.output if output_format == "cbor" else args.output + ".cbor.zst"
        export_graph_to_cbor(
            G,
            cbor_path,
            compression_level=config.export.compression_level,
            show_progress=not args.no_progress,
        )
        logger.success(f"CBOR export saved: {cbor_path}")

    if output_format in ("graphml", "both"):
        graphml_path = args.output if output_format == "graphml" else args.output.replace(".cbor.zst", ".graphml")
        nx.write_graphml(G, graphml_path)
        logger.success(f"GraphML export saved: {graphml_path}")

    # Compute and display statistics
    logger.info("Computing graph statistics...")
    stats_path = args.output.replace(".cbor.zst", ".stats.json").replace(".graphml", ".stats.json")
    compute_graph_statistics(
        G,
        export_path=stats_path,
        print_summary=True
    )
    logger.success(f"Statistics saved: {stats_path}")

    return G


def build_all(args) -> None:
    """Build everything in one command."""
    logger.info("Building complete knowledge graph pipeline...")

    resume = hasattr(args, 'resume') and args.resume
    content_db_path = Path(args.content_db)
    graph_path = Path(args.output)

    # 1. Content database
    if content_db_path.exists() and resume:
        logger.info(f"Step 1/3: Content database already exists at {args.content_db} - SKIPPING")
    else:
        logger.info("Step 1/3: Building content database...")
        content_args = type('obj', (object,), {
            'output': args.content_db,
            'morphology': args.morphology,
            'no_progress': args.no_progress,
        })
        build_content_db(content_args)

    # 2. Build knowledge graph from scratch
    if graph_path.exists() and resume:
        logger.info(f"Step 2/3: Knowledge graph already exists at {args.output} - SKIPPING")
        logger.success("Complete knowledge graph pipeline finished!")
        logger.info(f"Content database: {args.content_db}")
        logger.info(f"Knowledge graph: {args.output}")
        logger.info("(All steps were skipped - outputs already exist)")
    else:
        logger.info("Step 2/3: Building knowledge graph...")
        kg_args = type('obj', (object,), {
            'from_scratch': True,
            'morphology': args.morphology,
            'output': args.output,
            'preset': args.preset,
            'chapters': args.chapters if hasattr(args, 'chapters') and args.chapters else None,
            'no_progress': args.no_progress,
        })
        build_knowledge_graph(kg_args)

        logger.success("Complete knowledge graph pipeline finished!")
        logger.info(f"Content database: {args.content_db}")
        logger.info(f"Knowledge graph: {args.output}")



def run(args):
    """Execute the build command."""
    # Setup logging
    logger.remove()
    logger.add(
        sys.stderr,
        format="<green>{time:HH:mm:ss}</green> | <level>{level: <8}</level> | <level>{message}</level>",
        level="INFO",
    )

    # Route to appropriate sub-command
    if args.build_command == "content-db":
        build_content_db(args)
    elif args.build_command == "dependency-graph":
        build_dependency_graph(args)
    elif args.build_command == "knowledge-graph":
        build_knowledge_graph(args)
    elif args.build_command == "all":
        build_all(args)
    else:
        logger.error(f"Unknown build command: {args.build_command}")
        sys.exit(1)
