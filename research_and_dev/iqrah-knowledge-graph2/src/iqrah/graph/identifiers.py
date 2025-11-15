# iqrah/graph/identifiers.py
from enum import Enum, auto
from typing import Any
from ..quran_api.models import Word, Verse, Chapter


class NodeType(Enum):
    ROOT = auto()
    LEMMA = auto()
    WORD = auto()
    WORD_INSTANCE = auto()
    VERSE = auto()
    CHAPTER = auto()


class NodeIdentifierGenerator:
    """Generates consistent node identifiers for Quranic graph nodes."""

    @staticmethod
    def get_id(node_type: NodeType, node: Any, context: Any = None) -> str:
        """Generate a node identifier based on type and node data."""
        match node_type:
            case NodeType.ROOT:
                return f"ROOT:{node}"
            case NodeType.LEMMA:
                return f"LEMMA:{node}"
            case NodeType.WORD:
                return f"WORD:{node.text}"
            case NodeType.WORD_INSTANCE:
                if context is None:
                    raise ValueError("Verse context required for word instance ID")
                return f"WORD_INSTANCE:{context.verse_key}:{node.position}"
            case NodeType.VERSE:
                return f"VERSE:{node.verse_key if isinstance(node, Verse) else node}"
            case NodeType.CHAPTER:
                return f"CHAPTER:{node.id if isinstance(node, Chapter) else node}"

    @staticmethod
    def for_root(root: str) -> str:
        return NodeIdentifierGenerator.get_id(NodeType.ROOT, root)

    @staticmethod
    def for_lemma(lemma: str) -> str:
        return NodeIdentifierGenerator.get_id(NodeType.LEMMA, lemma)

    @staticmethod
    def for_word(word: Word) -> str:
        return NodeIdentifierGenerator.get_id(NodeType.WORD, word)

    @staticmethod
    def for_word_instance(word: Word, verse: Verse) -> str:
        return NodeIdentifierGenerator.get_id(NodeType.WORD_INSTANCE, word, verse)

    @staticmethod
    def for_verse(verse: Verse | str) -> str:
        if isinstance(verse, Verse):
            verse = verse.verse_key
        elif isinstance(verse, str):
            v = verse.split(":")
            if len(v) != 2 or not v[0].isdigit() or not v[1].isdigit():
                raise ValueError(f"Invalid verse key: {verse}")
        elif not isinstance(verse, str):
            raise ValueError(f"Expected Verse or str, got {type(verse)}")

        return NodeIdentifierGenerator.get_id(NodeType.VERSE, verse)

    @staticmethod
    def for_chapter(chapter: Chapter) -> str:
        if isinstance(chapter, Chapter):
            chapter = chapter.id
        elif isinstance(chapter, (int, str)):
            chapter = str(chapter)
        else:
            raise ValueError(f"Expected Chapter or str or int, got {type(chapter)}")

        return NodeIdentifierGenerator.get_id(NodeType.CHAPTER, chapter)


class NodeIdentifierParseError(Exception):
    """Raised when a node identifier cannot be parsed."""

    pass


class NodeIdentifierParser:
    """Parses node identifiers back into their constituent parts."""

    @staticmethod
    def parse(node_id: str) -> tuple[NodeType, str]:
        """
        Parse a node identifier into its type and value components.

        Args:
            node_id: The node identifier string to parse

        Returns:
            A tuple of (NodeType, str) where the string is the extracted value

        Raises:
            NodeIdentifierParseError: If the identifier format is invalid
        """
        try:
            type_str, *rest = node_id.split(":", 1)

            if not rest:
                raise NodeIdentifierParseError(f"Invalid identifier format: {node_id}")

            value = rest[0]

            try:
                node_type = NodeType[type_str]
            except KeyError:
                raise NodeIdentifierParseError(f"Unknown node type: {type_str}")

            match node_type:
                case NodeType.ROOT | NodeType.LEMMA | NodeType.WORD:
                    return node_type, value

                case NodeType.WORD_INSTANCE:
                    # Validate format chapter:verse:position
                    try:
                        chapter, verse, position = value.split(":")
                        # Ensure these are valid integers
                        int(chapter), int(verse), int(position)
                        return node_type, value
                    except ValueError:
                        raise NodeIdentifierParseError(
                            f"Invalid WORD_INSTANCE format: {value}"
                        )

                case NodeType.VERSE:
                    # Validate format chapter:verse
                    try:
                        chapter, verse = value.split(":")
                        # Ensure these are valid integers
                        int(chapter), int(verse)
                        return node_type, value
                    except ValueError:
                        raise NodeIdentifierParseError(f"Invalid VERSE format: {value}")

                case NodeType.CHAPTER:
                    # Validate it's an integer
                    try:
                        int(value)
                        return node_type, value
                    except ValueError:
                        raise NodeIdentifierParseError(
                            f"Invalid CHAPTER format: {value}"
                        )

        except Exception as e:
            if isinstance(e, NodeIdentifierParseError):
                raise
            raise NodeIdentifierParseError(
                f"Failed to parse identifier: {node_id}"
            ) from e

    @staticmethod
    def get_chapter_key(node_id: str) -> str:
        """
        Extract the chapter key from a CHAPTER identifier.

        Args:
            node_id: The node identifier to parse

        Returns:
            The chapter key in the format "chapter"

        Raises:
            NodeIdentifierParseError: If the identifier is not a CHAPTER
                or is malformed
        """
        node_type, value = NodeIdentifierParser.parse(node_id)

        match node_type:
            case NodeType.CHAPTER:
                return value
            case _:
                raise NodeIdentifierParseError(
                    f"Cannot extract chapter key from {node_type} identifier"
                )

    @staticmethod
    def get_verse_key(node_id: str) -> str:
        """
        Extract the verse key from a VERSE or WORD_INSTANCE identifier.

        Args:
            node_id: The node identifier to parse

        Returns:
            The verse key in the format "chapter:verse"

        Raises:
            NodeIdentifierParseError: If the identifier is not a VERSE or WORD_INSTANCE
                or is malformed
        """
        node_type, value = NodeIdentifierParser.parse(node_id)

        match node_type:
            case NodeType.VERSE:
                return value
            case NodeType.WORD_INSTANCE:
                chapter, verse, _ = value.split(":")
                return f"{chapter}:{verse}"
            case _:
                raise NodeIdentifierParseError(
                    f"Cannot extract verse key from {node_type} identifier"
                )

    @staticmethod
    def get_word_instance_key(node_id: str) -> str:
        """
        Extract the full key from a WORD_INSTANCE identifier.

        Args:
            node_id: The node identifier to parse

        Returns:
            The full key in the format "chapter:verse:position"

        Raises:
            NodeIdentifierParseError: If the identifier is not a WORD_INSTANCE or is malformed
        """
        node_type, value = NodeIdentifierParser.parse(node_id)

        if node_type != NodeType.WORD_INSTANCE:
            raise NodeIdentifierParseError(
                f"Cannot extract word instance key from {node_type} identifier"
            )

        return value


NIP = NodeIdentifierParser
NIG = NodeIdentifierGenerator
