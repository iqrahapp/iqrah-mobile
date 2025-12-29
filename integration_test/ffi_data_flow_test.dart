import 'dart:io';

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:iqrah/rust_bridge/frb_generated.dart';
import 'package:iqrah/rust_bridge/api.dart' as api;
import 'package:iqrah/services/content_service.dart';
import 'package:iqrah/services/translation_service.dart';
import 'package:iqrah/services/node_id_service.dart';

// Use separate test database with sample data (448KB)
// Production app uses rust/content.db (87MB with full Quran data)
const _contentDbAssetPath = 'rust/content_test.db';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  setUpAll(() async {
    await RustLib.init();
    final contentBytes =
        (await rootBundle.load(_contentDbAssetPath)).buffer.asUint8List();
    final tempDir = await Directory.systemTemp.createTemp('iqrah_test_');
    final contentDbPath = '${tempDir.path}/content.db';
    final userDbPath = '${tempDir.path}/user.db';

    await File(contentDbPath).writeAsBytes(contentBytes, flush: true);
    await api.setupDatabase(
      contentDbPath: contentDbPath,
      userDbPath: userDbPath,
      kgBytes: const [],
    );
  });

  group('FFI Data Flow Tests', () {
    test('Can fetch verse by key', () async {
      final verse = await api.getVerse(verseKey: '1:1');

      expect(verse, isNotNull);
      expect(verse!.key, '1:1');
      expect(verse.chapterNumber, 1);
      expect(verse.verseNumber, 1);
      expect(verse.textUthmani, isNotEmpty);
    });

    test('Can fetch words for verse', () async {
      final words = await api.getWordsForVerse(verseKey: '1:1');

      expect(words, isNotEmpty);
      expect(words.first.verseKey, '1:1');
      expect(words.first.position, greaterThan(0));
      expect(words.first.textUthmani, isNotEmpty);
    });

    test('Can fetch word at position (WordInstance resolution)', () async {
      final word = await api.getWordAtPosition(
        chapter: 1,
        verse: 1,
        position: 1,
      );

      expect(word, isNotNull);
      expect(word!.verseKey, '1:1');
      expect(word.position, 1);
      expect(word.textUthmani, isNotEmpty);
    });

    test('ContentService wrapper works', () async {
      final service = ContentService();

      final verse = await service.getVerse('1:1');
      expect(verse, isNotNull);

      final words = await service.getWordsForVerse('1:1');
      expect(words, isNotEmpty);

      final word = await service.getWordAtPosition(
        chapter: 1,
        verse: 1,
        position: 1,
      );
      expect(word, isNotNull);
    });

    test('NodeIdService can parse node IDs', () {
      // Test VERSE parsing
      expect(NodeIdService.getNodeType('VERSE:1:1'), NodeType.verse);
      expect(NodeIdService.parseVerseKey('VERSE:1:1'), '1:1');
      expect(NodeIdService.getNodeType('VERSE:1:1:memorization'),
          NodeType.knowledge);
      expect(NodeIdService.getBaseNodeType('VERSE:1:1:memorization'),
          NodeType.verse);
      expect(NodeIdService.parseVerseKey('VERSE:1:1:memorization'), '1:1');

      // Test WORD parsing
      expect(NodeIdService.getNodeType('WORD:123'), NodeType.word);
      expect(NodeIdService.parseWordId('WORD:123'), 123);
      expect(NodeIdService.getNodeType('WORD:123:translation'),
          NodeType.knowledge);
      expect(NodeIdService.getBaseNodeType('WORD:123:translation'),
          NodeType.word);
      expect(NodeIdService.parseWordId('WORD:123:translation'), 123);

      // Test WORD_INSTANCE parsing
      expect(NodeIdService.getNodeType('WORD_INSTANCE:1:1:3'),
             NodeType.wordInstance);
      final (ch, vs, pos) = NodeIdService.parseWordInstance('WORD_INSTANCE:1:1:3');
      expect(ch, 1);
      expect(vs, 1);
      expect(pos, 3);
      expect(
          NodeIdService.getNodeType('WORD_INSTANCE:1:1:3:meaning'),
          NodeType.knowledge);
      expect(
          NodeIdService.getBaseNodeType('WORD_INSTANCE:1:1:3:meaning'),
          NodeType.wordInstance);
      final (kch, kvs, kpos) =
          NodeIdService.parseWordInstance('WORD_INSTANCE:1:1:3:meaning');
      expect(kch, 1);
      expect(kvs, 1);
      expect(kpos, 3);

      // Test validation
      expect(NodeIdService.isValid('VERSE:1:1'), true);
      expect(NodeIdService.isValid('WORD:123'), true);
      expect(NodeIdService.isValid('WORD_INSTANCE:1:1:3'), true);
      expect(NodeIdService.isValid('VERSE:1:1:memorization'), true);
      expect(NodeIdService.isValid('WORD:123:translation'), true);
      expect(NodeIdService.isValid('WORD_INSTANCE:1:1:3:meaning'), true);
      expect(NodeIdService.isValid('VERSE:1:1:invalid_axis'), false);
      expect(NodeIdService.isValid('INVALID'), false);
    });

    test('TranslationService routes correctly', () async {
      final service = TranslationService();

      // Note: This test assumes translator_id 1 exists
      // If translations are not available, we just check the routing works
      // without errors

      // Test verse translation routing
      try {
        await service.getTranslation(
          nodeId: 'VERSE:1:1',
          translatorId: 1,
        );
      } catch (e) {
        // Expected if translator not available
        expect(e.toString(), contains('translation'));
      }

      // Test word instance translation routing (should not throw routing error)
      try {
        await service.getTranslation(
          nodeId: 'WORD_INSTANCE:1:1:1',
          translatorId: 1,
        );
      } catch (e) {
        // Expected if translator not available
        // But should NOT be a routing error
        expect(e.toString(), isNot(contains('Unsupported node type')));
      }
    });
  });
}
