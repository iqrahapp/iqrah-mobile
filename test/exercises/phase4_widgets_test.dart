import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/features/exercises/widgets/cloze_deletion_widget.dart';
import 'package:iqrah/features/exercises/widgets/first_letter_hint_widget.dart';
import 'package:iqrah/features/exercises/widgets/first_word_recall_widget.dart';
import 'package:iqrah/features/exercises/widgets/missing_word_mcq_widget.dart';
import 'package:iqrah/features/exercises/widgets/next_word_mcq_widget.dart';
import 'package:iqrah/features/exercises/widgets/sequence_recall_widget.dart';
import 'package:iqrah/features/exercises/widgets/translation_widget.dart';
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

void main() {
  testWidgets('SequenceRecallWidget reports correct answer', (tester) async {
    bool? result;
    final contentService = FakeContentService(
      verses: {
        '1:1': _verse('1:1', 'Verse One', 1, 1),
        '1:2': _verse('1:2', 'Verse Two', 1, 2),
        '1:3': _verse('1:3', 'Verse Three', 1, 3),
      },
    );

    await tester.pumpWidget(
      _wrap(
        SequenceRecallWidget(
          nodeId: 'VERSE:1:1',
          correctSequence: const ['VERSE:1:2'],
          options: const [
            ['VERSE:1:2'],
            ['VERSE:1:3'],
          ],
          onComplete: (value) => result = value,
          contentService: contentService,
        ),
      ),
    );

    await tester.pumpAndSettle();
    await tester.tap(find.text('Verse Two'));
    expect(result, isTrue);
  });

  testWidgets('SequenceRecallWidget reports incorrect answer', (tester) async {
    bool? result;
    final contentService = FakeContentService(
      verses: {
        '1:1': _verse('1:1', 'Verse One', 1, 1),
        '1:2': _verse('1:2', 'Verse Two', 1, 2),
        '1:3': _verse('1:3', 'Verse Three', 1, 3),
      },
    );

    await tester.pumpWidget(
      _wrap(
        SequenceRecallWidget(
          nodeId: 'VERSE:1:1',
          correctSequence: const ['VERSE:1:2'],
          options: const [
            ['VERSE:1:2'],
            ['VERSE:1:3'],
          ],
          onComplete: (value) => result = value,
          contentService: contentService,
        ),
      ),
    );

    await tester.pumpAndSettle();
    await tester.tap(find.text('Verse Three'));
    expect(result, isFalse);
  });

  testWidgets('FirstWordRecallWidget validates answer', (tester) async {
    bool? result;
    final contentService = FakeContentService(
      verseWords: {
        '1:1': [
          _word(1, 'بِسْمِ', '1:1', 1),
          _word(2, 'ٱللَّهِ', '1:1', 2),
        ],
      },
    );

    await tester.pumpWidget(
      _wrap(
        FirstWordRecallWidget(
          nodeId: 'VERSE:1:1',
          verseKey: '1:1',
          onComplete: (value) => result = value,
          contentService: contentService,
        ),
      ),
    );

    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField), 'بِسْمِ');
    await tester.tap(find.text('Check'));
    expect(result, isTrue);
  });

  testWidgets('FirstWordRecallWidget reports incorrect answer', (tester) async {
    bool? result;
    final contentService = FakeContentService(
      verseWords: {
        '1:1': [
          _word(1, 'بِسْمِ', '1:1', 1),
          _word(2, 'ٱللَّهِ', '1:1', 2),
        ],
      },
    );

    await tester.pumpWidget(
      _wrap(
        FirstWordRecallWidget(
          nodeId: 'VERSE:1:1',
          verseKey: '1:1',
          onComplete: (value) => result = value,
          contentService: contentService,
        ),
      ),
    );

    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField), 'ٱللَّهِ');
    await tester.tap(find.text('Check'));
    expect(result, isFalse);
  });

  testWidgets('TranslationWidget validates answer', (tester) async {
    bool? result;
    final contentService = FakeContentService(
      words: {
        1: _word(1, 'بِسْمِ', '1:1', 1),
      },
    );
    final translationService = FakeTranslationService(
      translations: {
        'WORD:1': 'In the name',
      },
    );

    await tester.pumpWidget(
      ProviderScope(
        child: _wrap(
          TranslationWidget(
            nodeId: 'WORD:1',
            translatorId: 1,
            onComplete: (value) => result = value,
            contentService: contentService,
            translationService: translationService,
          ),
        ),
      ),
    );

    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField), 'In the name');
    await tester.tap(find.text('Check'));
    expect(result, isTrue);
  });

  testWidgets('TranslationWidget reports incorrect answer', (tester) async {
    bool? result;
    final contentService = FakeContentService(
      words: {
        1: _word(1, 'بِسْمِ', '1:1', 1),
      },
    );
    final translationService = FakeTranslationService(
      translations: {
        'WORD:1': 'In the name',
      },
    );

    await tester.pumpWidget(
      ProviderScope(
        child: _wrap(
          TranslationWidget(
            nodeId: 'WORD:1',
            translatorId: 1,
            onComplete: (value) => result = value,
            contentService: contentService,
            translationService: translationService,
          ),
        ),
      ),
    );

    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField), 'Wrong answer');
    await tester.tap(find.text('Check'));
    expect(result, isFalse);
  });

  testWidgets('ClozeDeletionWidget validates answer', (tester) async {
    bool? result;
    final contentService = FakeContentService(
      verseWords: {
        '1:1': [
          _word(1, 'بِسْمِ', '1:1', 1),
          _word(2, 'ٱللَّهِ', '1:1', 2),
        ],
      },
    );

    await tester.pumpWidget(
      _wrap(
        ClozeDeletionWidget(
          nodeId: 'VERSE:1:1',
          blankPosition: 2,
          onComplete: (value) => result = value,
          contentService: contentService,
        ),
      ),
    );

    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField), 'ٱللَّهِ');
    await tester.tap(find.text('Check'));
    expect(result, isTrue);
  });

  testWidgets('ClozeDeletionWidget reports incorrect answer', (tester) async {
    bool? result;
    final contentService = FakeContentService(
      verseWords: {
        '1:1': [
          _word(1, 'بِسْمِ', '1:1', 1),
          _word(2, 'ٱللَّهِ', '1:1', 2),
        ],
      },
    );

    await tester.pumpWidget(
      _wrap(
        ClozeDeletionWidget(
          nodeId: 'VERSE:1:1',
          blankPosition: 2,
          onComplete: (value) => result = value,
          contentService: contentService,
        ),
      ),
    );

    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField), 'بِسْمِ');
    await tester.tap(find.text('Check'));
    expect(result, isFalse);
  });

  testWidgets('FirstLetterHintWidget validates answer', (tester) async {
    bool? result;
    final contentService = FakeContentService(
      verseWords: {
        '1:1': [
          _word(1, 'بِسْمِ', '1:1', 1),
          _word(2, 'ٱللَّهِ', '1:1', 2),
        ],
      },
    );

    await tester.pumpWidget(
      _wrap(
        FirstLetterHintWidget(
          nodeId: 'VERSE:1:1',
          wordPosition: 1,
          onComplete: (value) => result = value,
          contentService: contentService,
        ),
      ),
    );

    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField), 'بِسْمِ');
    await tester.tap(find.text('Check'));
    expect(result, isTrue);
  });

  testWidgets('FirstLetterHintWidget reports incorrect answer', (tester) async {
    bool? result;
    final contentService = FakeContentService(
      verseWords: {
        '1:1': [
          _word(1, 'بِسْمِ', '1:1', 1),
          _word(2, 'ٱللَّهِ', '1:1', 2),
        ],
      },
    );

    await tester.pumpWidget(
      _wrap(
        FirstLetterHintWidget(
          nodeId: 'VERSE:1:1',
          wordPosition: 1,
          onComplete: (value) => result = value,
          contentService: contentService,
        ),
      ),
    );

    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField), 'ٱللَّهِ');
    await tester.tap(find.text('Check'));
    expect(result, isFalse);
  });

  testWidgets('MissingWordMcqWidget validates answer', (tester) async {
    bool? result;
    final contentService = FakeContentService(
      verseWords: {
        '1:1': [
          _word(1, 'بِسْمِ', '1:1', 1),
          _word(2, 'ٱللَّهِ', '1:1', 2),
        ],
      },
      words: {
        99: _word(99, 'الرَّحْمَٰنِ', '1:1', 3),
      },
    );

    await tester.pumpWidget(
      _wrap(
        MissingWordMcqWidget(
          nodeId: 'VERSE:1:1',
          blankPosition: 2,
          distractorNodeIds: const ['WORD:99'],
          onComplete: (value) => result = value,
          contentService: contentService,
        ),
      ),
    );

    await tester.pumpAndSettle();
    await tester.tap(find.text('ٱللَّهِ'));
    expect(result, isTrue);
  });

  testWidgets('MissingWordMcqWidget reports incorrect answer', (tester) async {
    bool? result;
    final contentService = FakeContentService(
      verseWords: {
        '1:1': [
          _word(1, 'بِسْمِ', '1:1', 1),
          _word(2, 'ٱللَّهِ', '1:1', 2),
        ],
      },
      words: {
        99: _word(99, 'الرَّحْمَٰنِ', '1:1', 3),
      },
    );

    await tester.pumpWidget(
      _wrap(
        MissingWordMcqWidget(
          nodeId: 'VERSE:1:1',
          blankPosition: 2,
          distractorNodeIds: const ['WORD:99'],
          onComplete: (value) => result = value,
          contentService: contentService,
        ),
      ),
    );

    await tester.pumpAndSettle();
    await tester.tap(find.text('الرَّحْمَٰنِ'));
    expect(result, isFalse);
  });

  testWidgets('NextWordMcqWidget validates answer', (tester) async {
    bool? result;
    final contentService = FakeContentService(
      verseWords: {
        '1:1': [
          _word(1, 'بِسْمِ', '1:1', 1),
          _word(2, 'ٱللَّهِ', '1:1', 2),
          _word(3, 'ٱلرَّحْمَٰنِ', '1:1', 3),
        ],
      },
      words: {
        99: _word(99, 'الرَّحِيمِ', '1:1', 4),
      },
    );

    await tester.pumpWidget(
      _wrap(
        NextWordMcqWidget(
          nodeId: 'VERSE:1:1',
          contextPosition: 1,
          distractorNodeIds: const ['WORD:99'],
          onComplete: (value) => result = value,
          contentService: contentService,
        ),
      ),
    );

    await tester.pumpAndSettle();
    await tester.tap(find.text('ٱللَّهِ'));
    expect(result, isTrue);
  });

  testWidgets('NextWordMcqWidget reports incorrect answer', (tester) async {
    bool? result;
    final contentService = FakeContentService(
      verseWords: {
        '1:1': [
          _word(1, 'بِسْمِ', '1:1', 1),
          _word(2, 'ٱللَّهِ', '1:1', 2),
          _word(3, 'ٱلرَّحْمَٰنِ', '1:1', 3),
        ],
      },
      words: {
        99: _word(99, 'الرَّحِيمِ', '1:1', 4),
      },
    );

    await tester.pumpWidget(
      _wrap(
        NextWordMcqWidget(
          nodeId: 'VERSE:1:1',
          contextPosition: 1,
          distractorNodeIds: const ['WORD:99'],
          onComplete: (value) => result = value,
          contentService: contentService,
        ),
      ),
    );

    await tester.pumpAndSettle();
    await tester.tap(find.text('الرَّحِيمِ'));
    expect(result, isFalse);
  });
}
