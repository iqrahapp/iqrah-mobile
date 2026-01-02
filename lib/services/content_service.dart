import 'package:iqrah/rust_bridge/api.dart' as api;
import 'package:iqrah/utils/error_mapper.dart';

/// Service for accessing Quran content (verses, words)
class ContentService {
  static final Map<String, api.VerseDto> _verseCache = {};
  static final Map<String, List<api.WordDto>> _verseWordsCache = {};
  static final Map<int, api.WordDto> _wordCache = {};
  static final Map<String, api.WordDto> _wordAtPositionCache = {};
  static const _cacheExpiry = Duration(minutes: 30);
  static DateTime? _lastCacheClear;

  /// Fetch verse by key (e.g., "1:1")
  Future<api.VerseDto?> getVerse(String verseKey) async {
    _maybeClearCache();
    final cached = _verseCache[verseKey];
    if (cached != null) return cached;
    try {
      final verse = await api.getVerse(verseKey: verseKey);
      if (verse != null) {
        _verseCache[verseKey] = verse;
      }
      return verse;
    } catch (e) {
      throw ContentServiceException(
        ErrorMapper.toMessage(e, context: 'Unable to load verse $verseKey'),
      );
    }
  }

  /// Fetch all words for a verse
  Future<List<api.WordDto>> getWordsForVerse(String verseKey) async {
    _maybeClearCache();
    final cached = _verseWordsCache[verseKey];
    if (cached != null) return cached;
    try {
      final words = await api.getWordsForVerse(verseKey: verseKey);
      _verseWordsCache[verseKey] = words;
      return words;
    } catch (e) {
      throw ContentServiceException(
        ErrorMapper.toMessage(
          e,
          context: 'Unable to load words for $verseKey',
        ),
      );
    }
  }

  /// Fetch word by ID
  Future<api.WordDto?> getWord(int wordId) async {
    _maybeClearCache();
    final cached = _wordCache[wordId];
    if (cached != null) return cached;
    try {
      final word = await api.getWord(wordId: wordId);
      if (word != null) {
        _wordCache[wordId] = word;
      }
      return word;
    } catch (e) {
      throw ContentServiceException(
        ErrorMapper.toMessage(e, context: 'Unable to load word $wordId'),
      );
    }
  }

  /// Fetch word at specific position in a verse (resolves WORD_INSTANCE)
  Future<api.WordDto?> getWordAtPosition({
    required int chapter,
    required int verse,
    required int position,
  }) async {
    _maybeClearCache();
    final cacheKey = '$chapter:$verse:$position';
    final cached = _wordAtPositionCache[cacheKey];
    if (cached != null) return cached;
    try {
      final word = await api.getWordAtPosition(
        chapter: chapter,
        verse: verse,
        position: position,
      );
      if (word != null) {
        _wordAtPositionCache[cacheKey] = word;
      }
      return word;
    } catch (e) {
      throw ContentServiceException(
        ErrorMapper.toMessage(
          e,
          context: 'Unable to load word at $chapter:$verse:$position',
        ),
      );
    }
  }

  void _maybeClearCache() {
    if (_lastCacheClear == null ||
        DateTime.now().difference(_lastCacheClear!) > _cacheExpiry) {
      _verseCache.clear();
      _verseWordsCache.clear();
      _wordCache.clear();
      _wordAtPositionCache.clear();
      _lastCacheClear = DateTime.now();
    }
  }
}

class ContentServiceException implements Exception {
  final String message;
  ContentServiceException(this.message);
  @override
  String toString() => message;
}
