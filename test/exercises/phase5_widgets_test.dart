import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/features/exercises/widgets/ayah_chain_widget.dart';
import 'package:iqrah/features/exercises/widgets/ayah_sequence_widget.dart';
import 'package:iqrah/features/exercises/widgets/contextual_translation_widget.dart';
import 'package:iqrah/features/exercises/widgets/cross_verse_connection_widget.dart';
import 'package:iqrah/features/exercises/widgets/find_mistake_widget.dart';
import 'package:iqrah/features/exercises/widgets/full_verse_input_widget.dart';
import 'package:iqrah/features/exercises/widgets/identify_root_widget.dart';
import 'package:iqrah/features/exercises/widgets/pos_tagging_widget.dart';
import 'package:iqrah/features/exercises/widgets/reverse_cloze_widget.dart';
import 'package:iqrah/features/exercises/widgets/translate_phrase_widget.dart';
import 'package:iqrah/rust_bridge/api.dart' as api;
import '../helpers/fake_services.dart';

api.VerseDto _verse(String key, String text, int chapter, int verse) {
  return api.VerseDto(
    key: key,
    textUthmani: text,
    chapterNumber: chapter,
    verseNumber: verse,
  );
}

api.WordDto _word(int id, String text, String verseKey, int position) {
  return api.WordDto(
    id: id,
    textUthmani: text,
    verseKey: verseKey,
    position: position,
  );
}

Widget _wrap(Widget child) {
  return MaterialApp(
    home: Scaffold(
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: child,
      ),
    ),
  );
}

Future<void> _pumpUntilFound(
  WidgetTester tester,
  Finder finder, {
  Duration step = const Duration(milliseconds: 50),
  int maxPumps = 20,
}) async {
  for (var i = 0; i < maxPumps; i += 1) {
    if (finder.evaluate().isNotEmpty) {
      return;
    }
    await tester.pump(step);
  }
  throw StateError('Widget not found: $finder');
}

void main() {
  testWidgets('FullVerseInputWidget validates answer', (tester) async {
    bool? result;
    final contentService = FakeContentService(
      verses: {'1:1': _verse('1:1', 'ALPHA', 1, 1)},
    );

    await tester.pumpWidget(
      _wrap(
        FullVerseInputWidget(
          nodeId: 'VERSE:1:1',
          onComplete: (value) => result = value,
          contentService: contentService,
        ),
      ),
    );

    await _pumpUntilFound(tester, find.byType(TextField));
    await tester.enterText(find.byType(TextField), 'ALPHA');
    await tester.tap(find.text('Check'));
    expect(result, isTrue);
  });

  testWidgets('FullVerseInputWidget reports incorrect answer', (tester) async {
    bool? result;
    final contentService = FakeContentService(
      verses: {'1:1': _verse('1:1', 'ALPHA', 1, 1)},
    );

    await tester.pumpWidget(
      _wrap(
        FullVerseInputWidget(
          nodeId: 'VERSE:1:1',
          onComplete: (value) => result = value,
          contentService: contentService,
        ),
      ),
    );

    await _pumpUntilFound(tester, find.byType(TextField));
    await tester.enterText(find.byType(TextField), 'BETA');
    await tester.tap(find.text('Check'));
    expect(result, isFalse);
  });

  testWidgets('AyahChainWidget completes with correct input', (tester) async {
    bool? result;
    final contentService = FakeContentService(
      verses: {'1:1': _verse('1:1', 'VERSE ONE', 1, 1)},
    );

    await tester.pumpWidget(
      _wrap(
        AyahChainWidget(
          nodeId: 'CHAPTER:1',
          verseKeys: const ['1:1'],
          currentIndex: 0,
          completedCount: 0,
          onComplete: (value) => result = value,
          contentService: contentService,
          directoryProvider: () async => Directory.systemTemp,
          enablePersistence: false,
        ),
      ),
    );

    await _pumpUntilFound(tester, find.byType(TextField));
    final field = tester.widget<TextField>(find.byType(TextField));
    field.controller!.text = 'VERSE ONE';
    await tester.pump();
    await tester.tap(find.text('Check'));
    expect(result, isTrue);
  });

  testWidgets('AyahChainWidget reports incorrect input', (tester) async {
    bool? result;
    final contentService = FakeContentService(
      verses: {'1:1': _verse('1:1', 'VERSE ONE', 1, 1)},
    );

    await tester.pumpWidget(
      _wrap(
        AyahChainWidget(
          nodeId: 'CHAPTER:1',
          verseKeys: const ['1:1'],
          currentIndex: 0,
          completedCount: 0,
          onComplete: (value) => result = value,
          contentService: contentService,
          directoryProvider: () async => Directory.systemTemp,
          enablePersistence: false,
        ),
      ),
    );

    await _pumpUntilFound(tester, find.byType(TextField));
    final field = tester.widget<TextField>(find.byType(TextField));
    field.controller!.text = 'WRONG';
    await tester.pump();
    await tester.tap(find.text('Check'));
    expect(result, isFalse);
  });

  testWidgets('FindMistakeWidget detects incorrect word', (tester) async {
    bool? result;
    final contentService = FakeContentService(
      verseWords: {
        '1:1': [
          _word(1, 'ONE', '1:1', 1),
          _word(2, 'TWO', '1:1', 2),
          _word(3, 'THREE', '1:1', 3),
        ],
      },
      wordAtPosition: {
        '1:1:3': _word(33, 'WRONG', '1:1', 3),
      },
    );

    await tester.pumpWidget(
      _wrap(
        FindMistakeWidget(
          nodeId: 'VERSE:1:1',
          mistakePosition: 2,
          correctWordNodeId: 'WORD_INSTANCE:1:1:2',
          incorrectWordNodeId: 'WORD_INSTANCE:1:1:3',
          onComplete: (value) => result = value,
          contentService: contentService,
        ),
      ),
    );

    await tester.pumpAndSettle();
    await tester.tap(find.text('WRONG'));
    expect(result, isTrue);
  });

  testWidgets('FindMistakeWidget reports incorrect selection', (tester) async {
    bool? result;
    final contentService = FakeContentService(
      verseWords: {
        '1:1': [
          _word(1, 'ONE', '1:1', 1),
          _word(2, 'TWO', '1:1', 2),
          _word(3, 'THREE', '1:1', 3),
        ],
      },
      wordAtPosition: {
        '1:1:3': _word(33, 'WRONG', '1:1', 3),
      },
    );

    await tester.pumpWidget(
      _wrap(
        FindMistakeWidget(
          nodeId: 'VERSE:1:1',
          mistakePosition: 2,
          correctWordNodeId: 'WORD_INSTANCE:1:1:2',
          incorrectWordNodeId: 'WORD_INSTANCE:1:1:3',
          onComplete: (value) => result = value,
          contentService: contentService,
        ),
      ),
    );

    await tester.pumpAndSettle();
    await tester.tap(find.text('ONE'));
    expect(result, isFalse);
  });

  testWidgets('AyahSequenceWidget validates selection', (tester) async {
    bool? result;
    final contentService = FakeContentService(
      verses: {'1:2': _verse('1:2', 'VERSE TWO', 1, 2)},
    );

    await tester.pumpWidget(
      _wrap(
        AyahSequenceWidget(
          nodeId: 'CHAPTER:1',
          correctSequence: const ['VERSE:1:2'],
          onComplete: (value) => result = value,
          contentService: contentService,
        ),
      ),
    );

    await tester.pumpAndSettle();
    await tester.tap(find.text('VERSE TWO'));
    await tester.pump();
    await tester.tap(find.text('Submit'));
    expect(result, isTrue);
  });

  testWidgets('AyahSequenceWidget reports incorrect order', (tester) async {
    bool? result;
    final contentService = FakeContentService(
      verses: {
        '1:1': _verse('1:1', 'VERSE ONE', 1, 1),
        '1:2': _verse('1:2', 'VERSE TWO', 1, 2),
      },
    );

    await tester.pumpWidget(
      _wrap(
        AyahSequenceWidget(
          nodeId: 'CHAPTER:1',
          correctSequence: const ['VERSE:1:1', 'VERSE:1:2'],
          onComplete: (value) => result = value,
          contentService: contentService,
        ),
      ),
    );

    await tester.pumpAndSettle();
    await tester.tap(find.text('VERSE TWO'));
    await tester.pump();
    await tester.tap(find.text('VERSE ONE'));
    await tester.pump();
    await tester.tap(find.text('Submit'));
    expect(result, isFalse);
  });

  testWidgets('IdentifyRootWidget validates root input', (tester) async {
    bool? result;
    final contentService = FakeContentService(
      words: {1: _word(1, 'WORD', '1:1', 1)},
    );

    await tester.pumpWidget(
      _wrap(
        IdentifyRootWidget(
          nodeId: 'WORD:1',
          root: 'ROOT',
          onComplete: (value) => result = value,
          contentService: contentService,
        ),
      ),
    );

    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField), 'ROOT');
    await tester.tap(find.text('Check'));
    expect(result, isTrue);
  });

  testWidgets('IdentifyRootWidget reports incorrect root', (tester) async {
    bool? result;
    final contentService = FakeContentService(
      words: {1: _word(1, 'WORD', '1:1', 1)},
    );

    await tester.pumpWidget(
      _wrap(
        IdentifyRootWidget(
          nodeId: 'WORD:1',
          root: 'ROOT',
          onComplete: (value) => result = value,
          contentService: contentService,
        ),
      ),
    );

    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField), 'WRONG');
    await tester.tap(find.text('Check'));
    expect(result, isFalse);
  });

  testWidgets('ReverseClozeWidget validates answer', (tester) async {
    bool? result;
    final contentService = FakeContentService(
      verseWords: {
        '1:1': [
          _word(1, 'ALPHA', '1:1', 1),
          _word(2, 'BETA', '1:1', 2),
          _word(3, 'GAMMA', '1:1', 3),
        ],
      },
    );
    final translationService = FakeTranslationService(
      translations: {'VERSE:1:1': 'Translation'},
    );

    await tester.pumpWidget(
      ProviderScope(
        child: _wrap(
          ReverseClozeWidget(
            nodeId: 'VERSE:1:1',
            blankPosition: 2,
            onComplete: (value) => result = value,
            contentService: contentService,
            translationService: translationService,
          ),
        ),
      ),
    );

    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField), 'BETA');
    await tester.tap(find.text('Check'));
    expect(result, isTrue);
  });

  testWidgets('ReverseClozeWidget reports incorrect answer', (tester) async {
    bool? result;
    final contentService = FakeContentService(
      verseWords: {
        '1:1': [
          _word(1, 'ALPHA', '1:1', 1),
          _word(2, 'BETA', '1:1', 2),
          _word(3, 'GAMMA', '1:1', 3),
        ],
      },
    );
    final translationService = FakeTranslationService(
      translations: {'VERSE:1:1': 'Translation'},
    );

    await tester.pumpWidget(
      ProviderScope(
        child: _wrap(
          ReverseClozeWidget(
            nodeId: 'VERSE:1:1',
            blankPosition: 2,
            onComplete: (value) => result = value,
            contentService: contentService,
            translationService: translationService,
          ),
        ),
      ),
    );

    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField), 'WRONG');
    await tester.tap(find.text('Check'));
    expect(result, isFalse);
  });

  testWidgets('TranslatePhraseWidget validates answer', (tester) async {
    bool? result;
    final contentService = FakeContentService(
      verses: {'1:1': _verse('1:1', 'PHRASE', 1, 1)},
    );
    final translationService = FakeTranslationService(
      translations: {'VERSE:1:1': 'Translation'},
    );

    await tester.pumpWidget(
      _wrap(
        TranslatePhraseWidget(
          nodeId: 'VERSE:1:1',
          translatorId: 1,
          onComplete: (value) => result = value,
          contentService: contentService,
          translationService: translationService,
        ),
      ),
    );

    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField), 'Translation');
    await tester.tap(find.text('Check'));
    expect(result, isTrue);
  });

  testWidgets('TranslatePhraseWidget reports incorrect answer', (tester) async {
    bool? result;
    final contentService = FakeContentService(
      verses: {'1:1': _verse('1:1', 'PHRASE', 1, 1)},
    );
    final translationService = FakeTranslationService(
      translations: {'VERSE:1:1': 'Translation'},
    );

    await tester.pumpWidget(
      _wrap(
        TranslatePhraseWidget(
          nodeId: 'VERSE:1:1',
          translatorId: 1,
          onComplete: (value) => result = value,
          contentService: contentService,
          translationService: translationService,
        ),
      ),
    );

    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField), 'Wrong');
    await tester.tap(find.text('Check'));
    expect(result, isFalse);
  });

  testWidgets('PosTaggingWidget validates POS selection', (tester) async {
    bool? result;
    final contentService = FakeContentService(
      words: {1: _word(1, 'WORD', '1:1', 1)},
    );

    await tester.pumpWidget(
      _wrap(
        PosTaggingWidget(
          nodeId: 'WORD:1',
          correctPos: 'verb',
          options: const ['noun', 'verb'],
          onComplete: (value) => result = value,
          contentService: contentService,
        ),
      ),
    );

    await tester.pumpAndSettle();
    await tester.tap(find.text('verb'));
    expect(result, isTrue);
  });

  testWidgets('PosTaggingWidget reports incorrect selection', (tester) async {
    bool? result;
    final contentService = FakeContentService(
      words: {1: _word(1, 'WORD', '1:1', 1)},
    );

    await tester.pumpWidget(
      _wrap(
        PosTaggingWidget(
          nodeId: 'WORD:1',
          correctPos: 'verb',
          options: const ['noun', 'verb'],
          onComplete: (value) => result = value,
          contentService: contentService,
        ),
      ),
    );

    await tester.pumpAndSettle();
    await tester.tap(find.text('noun'));
    expect(result, isFalse);
  });

  testWidgets('CrossVerseConnectionWidget validates selection', (tester) async {
    bool? result;
    final contentService = FakeContentService(
      verses: {
        '1:1': _verse('1:1', 'VERSE ONE', 1, 1),
        '1:2': _verse('1:2', 'VERSE TWO', 1, 2),
      },
    );

    await tester.pumpWidget(
      _wrap(
        CrossVerseConnectionWidget(
          nodeId: 'VERSE:1:1',
          relatedVerseIds: const ['VERSE:1:1', 'VERSE:1:2'],
          connectionTheme: 'Adjacent verses',
          onComplete: (value) => result = value,
          contentService: contentService,
        ),
      ),
    );

    await tester.pumpAndSettle();
    await tester.tap(find.text('VERSE ONE'));
    await tester.pump();
    await tester.tap(find.text('Check'));
    expect(result, isTrue);
  });

  testWidgets('CrossVerseConnectionWidget reports incorrect selection',
      (tester) async {
    bool? result;
    final contentService = FakeContentService(
      verses: {
        '1:1': _verse('1:1', 'VERSE ONE', 1, 1),
        '1:2': _verse('1:2', 'VERSE TWO', 1, 2),
      },
    );

    await tester.pumpWidget(
      _wrap(
        CrossVerseConnectionWidget(
          nodeId: 'VERSE:1:1',
          relatedVerseIds: const ['VERSE:1:1', 'VERSE:1:2'],
          connectionTheme: 'Adjacent verses',
          onComplete: (value) => result = value,
          contentService: contentService,
        ),
      ),
    );

    await tester.pumpAndSettle();
    await tester.tap(find.text('VERSE TWO'));
    await tester.pump();
    await tester.tap(find.text('Check'));
    expect(result, isFalse);
  });

  testWidgets('ContextualTranslationWidget validates answer', (tester) async {
    bool? result;
    final contentService = FakeContentService(
      verses: {'1:1': _verse('1:1', 'VERSE', 1, 1)},
      words: {1: _word(1, 'WORD', '1:1', 1)},
    );
    final translationService = FakeTranslationService(
      translations: {'WORD:1': 'Translation'},
    );

    await tester.pumpWidget(
      ProviderScope(
        child: _wrap(
          ContextualTranslationWidget(
            nodeId: 'WORD:1',
            verseKey: '1:1',
            onComplete: (value) => result = value,
            contentService: contentService,
            translationService: translationService,
          ),
        ),
      ),
    );

    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField), 'Translation');
    await tester.tap(find.text('Check'));
    expect(result, isTrue);
  });

  testWidgets('ContextualTranslationWidget reports incorrect answer',
      (tester) async {
    bool? result;
    final contentService = FakeContentService(
      verses: {'1:1': _verse('1:1', 'VERSE', 1, 1)},
      words: {1: _word(1, 'WORD', '1:1', 1)},
    );
    final translationService = FakeTranslationService(
      translations: {'WORD:1': 'Translation'},
    );

    await tester.pumpWidget(
      ProviderScope(
        child: _wrap(
          ContextualTranslationWidget(
            nodeId: 'WORD:1',
            verseKey: '1:1',
            onComplete: (value) => result = value,
            contentService: contentService,
            translationService: translationService,
          ),
        ),
      ),
    );

    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField), 'Wrong');
    await tester.tap(find.text('Check'));
    expect(result, isFalse);
  });
}
