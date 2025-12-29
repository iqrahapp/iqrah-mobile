/// Node types supported in the system
enum NodeType {
  verse,
  word,
  wordInstance,
  chapter,
  root,
  lemma,
  knowledge,
  unknown,
}

/// Service for parsing and validating node IDs
class NodeIdService {
  static const Set<String> _knowledgeAxes = {
    'memorization',
    'translation',
    'tafsir',
    'tajweed',
    'contextual_memorization',
    'meaning',
  };

  /// Returns the knowledge axis suffix if present and valid.
  static String? extractKnowledgeAxis(String nodeId) {
    final parts = nodeId.split(':');
    if (parts.length < 2) return null;

    final axis = parts.last;
    if (_knowledgeAxes.contains(axis)) {
      return axis;
    }

    return null;
  }

  /// True if the node ID has a valid knowledge axis suffix.
  static bool isKnowledgeAxisId(String nodeId) {
    return extractKnowledgeAxis(nodeId) != null;
  }

  /// Returns the base node ID without a knowledge axis suffix.
  static String baseNodeId(String nodeId) {
    final axis = extractKnowledgeAxis(nodeId);
    if (axis == null) return nodeId;

    final parts = nodeId.split(':');
    return parts.sublist(0, parts.length - 1).join(':');
  }

  /// Detect node type from ID prefix
  static NodeType getNodeType(String nodeId) {
    if (isKnowledgeAxisId(nodeId)) return NodeType.knowledge;
    return getBaseNodeType(nodeId);
  }

  /// Detect base node type (ignores knowledge axis suffix).
  static NodeType getBaseNodeType(String nodeId) {
    final baseId = baseNodeId(nodeId);

    if (baseId.startsWith('WORD_INSTANCE:')) return NodeType.wordInstance;
    if (baseId.startsWith('VERSE:')) return NodeType.verse;
    if (baseId.startsWith('WORD:')) return NodeType.word;
    if (baseId.startsWith('CHAPTER:')) return NodeType.chapter;
    if (baseId.startsWith('ROOT:')) return NodeType.root;
    if (baseId.startsWith('LEMMA:')) return NodeType.lemma;

    return NodeType.unknown;
  }

  /// Parse verse key from VERSE node ID
  /// Example: "VERSE:1:1" -> "1:1"
  /// For knowledge nodes like "VERSE:1:1:memorization", extracts base "1:1"
  static String parseVerseKey(String nodeId) {
    final baseId = baseNodeId(nodeId);
    if (!baseId.startsWith('VERSE:')) {
      throw NodeIdParseException('Not a VERSE node: $nodeId');
    }

    final withoutPrefix = baseId.substring(6); // Remove "VERSE:" prefix
    final parts = withoutPrefix.split(':');

    if (parts.length != 2) {
      throw NodeIdParseException('Invalid VERSE format: $nodeId');
    }

    final chapter = int.tryParse(parts[0]);
    final verse = int.tryParse(parts[1]);
    if (chapter == null || verse == null) {
      throw NodeIdParseException('Invalid VERSE components: $nodeId');
    }

    return '${parts[0]}:${parts[1]}';
  }

  /// Parse word ID from WORD node ID
  /// Example: "WORD:123" -> 123
  static int parseWordId(String nodeId) {
    final baseId = baseNodeId(nodeId);
    if (!baseId.startsWith('WORD:')) {
      throw NodeIdParseException('Not a WORD node: $nodeId');
    }
    final idStr = baseId.substring(5); // Remove "WORD:" prefix
    if (idStr.contains(':')) {
      throw NodeIdParseException('Invalid WORD format: $nodeId');
    }
    final id = int.tryParse(idStr);
    if (id == null) {
      throw NodeIdParseException('Invalid word ID: $nodeId');
    }
    return id;
  }

  /// Parse WORD_INSTANCE node ID into components
  /// Example: "WORD_INSTANCE:1:1:3" -> (chapter: 1, verse: 1, position: 3)
  static (int, int, int) parseWordInstance(String nodeId) {
    final baseId = baseNodeId(nodeId);
    if (!baseId.startsWith('WORD_INSTANCE:')) {
      throw NodeIdParseException('Not a WORD_INSTANCE node: $nodeId');
    }

    final parts = baseId.substring(14).split(':'); // Remove "WORD_INSTANCE:"
    if (parts.length != 3) {
      throw NodeIdParseException('Invalid WORD_INSTANCE format: $nodeId');
    }

    final chapter = int.tryParse(parts[0]);
    final verse = int.tryParse(parts[1]);
    final position = int.tryParse(parts[2]);

    if (chapter == null || verse == null || position == null) {
      throw NodeIdParseException('Invalid WORD_INSTANCE components: $nodeId');
    }

    return (chapter, verse, position);
  }

  /// Validate node ID format
  static bool isValid(String nodeId) {
    try {
      final baseType = getBaseNodeType(nodeId);
      if (baseType == NodeType.unknown) return false;

      switch (baseType) {
        case NodeType.verse:
          parseVerseKey(nodeId);
          break;
        case NodeType.word:
          parseWordId(nodeId);
          break;
        case NodeType.wordInstance:
          parseWordInstance(nodeId);
          break;
        default:
          break;
      }

      return true;
    } catch (_) {
      return false;
    }
  }
}

class NodeIdParseException implements Exception {
  final String message;
  NodeIdParseException(this.message);
  @override
  String toString() => message;
}
