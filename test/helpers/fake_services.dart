import 'package:iqrah/rust_bridge/api.dart' as api;
import 'package:iqrah/services/content_service.dart';
import 'package:iqrah/services/translation_service.dart';

class FakeContentService implements ContentService {
  FakeContentService({
    Map<String, api.VerseDto>? verses,
    Map<String, List<api.WordDto>>? verseWords,
    Map<int, api.WordDto>? words,
    Map<String, api.WordDto>? wordAtPosition,
  })  : _verses = verses ?? {},
        _verseWords = verseWords ?? {},
        _words = words ?? {},
        _wordAtPosition = wordAtPosition ?? {};

  final Map<String, api.VerseDto> _verses;
  final Map<String, List<api.WordDto>> _verseWords;
  final Map<int, api.WordDto> _words;
  final Map<String, api.WordDto> _wordAtPosition;

  @override
  Future<api.VerseDto?> getVerse(String verseKey) async {
    return _verses[verseKey];
  }

  @override
  Future<List<api.WordDto>> getWordsForVerse(String verseKey) async {
    return List<api.WordDto>.from(_verseWords[verseKey] ?? []);
  }

  @override
  Future<api.WordDto?> getWord(int wordId) async {
    return _words[wordId];
  }

  @override
  Future<api.WordDto?> getWordAtPosition({
    required int chapter,
    required int verse,
    required int position,
  }) async {
    return _wordAtPosition['$chapter:$verse:$position'];
  }
}

class FakeTranslationService implements TranslationService {
  FakeTranslationService({Map<String, String>? translations})
      : _translations = translations ?? {};

  final Map<String, String> _translations;

  @override
  Future<String?> getTranslation({
    required String nodeId,
    required int translatorId,
  }) async {
    return _translations[nodeId];
  }
}
