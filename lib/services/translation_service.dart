import 'package:iqrah/rust_bridge/api.dart' as api;
import 'package:iqrah/services/node_id_service.dart';
import 'package:iqrah/utils/error_mapper.dart';

/// Service for fetching translations
class TranslationService {
  static final Map<String, String?> _cache = {};
  static const _cacheExpiry = Duration(minutes: 30);
  static DateTime? _lastCacheClear;

  /// Fetch translation for a node (verse, word, or word instance)
  Future<String?> getTranslation({
    required String nodeId,
    required int translatorId,
  }) async {
    _maybeClearCache();
    final cacheKey = '$nodeId:$translatorId';
    if (_cache.containsKey(cacheKey)) {
      return _cache[cacheKey];
    }
    try {
      final nodeType = NodeIdService.getBaseNodeType(nodeId);
      String? translation;

      switch (nodeType) {
        case NodeType.verse:
          // Extract verse key (e.g., "VERSE:1:1" -> "1:1")
          final verseKey = NodeIdService.parseVerseKey(nodeId);
          translation = await api.getVerseTranslationByTranslator(
            verseKey: verseKey,
            translatorId: translatorId,
          );
          break;

        case NodeType.word:
          // Extract word ID (e.g., "WORD:123" -> 123)
          final wordId = NodeIdService.parseWordId(nodeId);
          translation = await api.getWordTranslation(
            wordId: wordId,
            translatorId: translatorId,
          );
          break;

        case NodeType.wordInstance:
          // Parse position and fetch word, then get its translation
          final (chapter, verse, position) =
              NodeIdService.parseWordInstance(nodeId);
          final word = await api.getWordAtPosition(
            chapter: chapter,
            verse: verse,
            position: position,
          );
          if (word == null) return null;

          translation = await api.getWordTranslation(
            wordId: word.id,
            translatorId: translatorId,
          );
          break;

        default:
          throw TranslationServiceException(
            'Unsupported node type for translation: $nodeId',
          );
      }
      _cache[cacheKey] = translation;
      return translation;
    } catch (e) {
      throw TranslationServiceException(
        ErrorMapper.toMessage(
          e,
          context: 'Unable to load translation for $nodeId',
        ),
      );
    }
  }

  void _maybeClearCache() {
    if (_lastCacheClear == null ||
        DateTime.now().difference(_lastCacheClear!) > _cacheExpiry) {
      _cache.clear();
      _lastCacheClear = DateTime.now();
    }
  }
}

class TranslationServiceException implements Exception {
  final String message;
  TranslationServiceException(this.message);
  @override
  String toString() => message;
}
