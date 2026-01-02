import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/features/exercises/widgets/ayah_chain_widget.dart';
import 'package:iqrah/features/exercises/widgets/ayah_sequence_widget.dart';
import 'package:iqrah/features/exercises/widgets/cloze_deletion_widget.dart';
import 'package:iqrah/features/exercises/widgets/contextual_translation_widget.dart';
import 'package:iqrah/features/exercises/widgets/cross_verse_connection_widget.dart';
import 'package:iqrah/features/exercises/widgets/echo_recall_widget.dart';
import 'package:iqrah/features/exercises/widgets/find_mistake_widget.dart';
import 'package:iqrah/features/exercises/widgets/first_letter_hint_widget.dart';
import 'package:iqrah/features/exercises/widgets/first_word_recall_widget.dart';
import 'package:iqrah/features/exercises/widgets/full_verse_input_widget.dart';
import 'package:iqrah/features/exercises/widgets/identify_root_widget.dart';
import 'package:iqrah/features/exercises/widgets/missing_word_mcq_widget.dart';
import 'package:iqrah/features/exercises/widgets/next_word_mcq_widget.dart';
import 'package:iqrah/features/exercises/widgets/pos_tagging_widget.dart';
import 'package:iqrah/features/exercises/widgets/reverse_cloze_widget.dart';
import 'package:iqrah/features/exercises/widgets/sequence_recall_widget.dart';
import 'package:iqrah/features/exercises/widgets/translate_phrase_widget.dart';
import 'package:iqrah/features/exercises/widgets/translation_widget.dart';
import 'package:iqrah/rust_bridge/api.dart';
import 'package:iqrah/services/exercise_content_service.dart';
import 'package:iqrah/widgets/error_banner.dart';

/// A container that fetches content for an exercise and renders the appropriate widget.
class ExerciseContainer extends ConsumerStatefulWidget {
  final ExerciseDataDto exercise;
  final Function(bool isCorrect) onComplete;

  const ExerciseContainer({
    super.key,
    required this.exercise,
    required this.onComplete,
  });

  @override
  ConsumerState<ExerciseContainer> createState() => _ExerciseContainerState();
}

class _ExerciseContainerState extends ConsumerState<ExerciseContainer> {
  bool _isLoading = true;
  String? _error;
  int _loadRequestId = 0;

  // Loaded content
  String? _questionText;
  String? _answerText;
  List<String>? _choices;
  int? _correctIndex;

  @override
  void initState() {
    super.initState();
    _loadContent();
  }

  @override
  void didUpdateWidget(ExerciseContainer oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.exercise != widget.exercise) {
      _loadContent();
    }
  }

  Future<void> _loadContent() async {
    final requestId = ++_loadRequestId;
    setState(() {
      _isLoading = true;
      _error = null;
      _questionText = null;
      _answerText = null;
      _choices = null;
      _correctIndex = null;
    });

    try {
      final service = ref.read(exerciseContentServiceProvider);
      final prefs = ref.read(userPreferencesProvider);
      String? questionText;
      String? answerText;
      List<String>? choices;
      int? correctIndex;

      await widget.exercise.map(
        memorization: (e) async {
          // Fetch word/verse text
          questionText = await service.fetchNodeText(e.nodeId);

          // Fetch translation
          final translation = await service.fetchTranslation(
            e.nodeId,
            prefs.preferredTranslatorId ?? 1,
          );
          answerText = translation.text;
        },
        mcqArToEn: (e) async {
          // Fetch correct answer (English translation)
          final correctTrans = await service.fetchTranslation(
            e.nodeId,
            prefs.preferredTranslatorId ?? 1,
          );

          // Fetch distractors
          final distractors = <String>[];
          for (final id in e.distractorNodeIds) {
            final d = await service.fetchTranslation(
              id,
              prefs.preferredTranslatorId ?? 1,
            );
            distractors.add(d.text);
          }

          // Shuffle and set up choices
          final allChoices = [correctTrans.text, ...distractors]..shuffle();
          choices = allChoices;
          correctIndex = allChoices.indexOf(correctTrans.text);

          // Fetch question (Arabic text)
          questionText = await service.fetchNodeText(e.nodeId);
        },
        mcqEnToAr: (e) async {
          // Fetch correct answer (Arabic text)
          final correctAnswer = await service.fetchNodeText(e.nodeId);

          // Fetch distractors (Arabic text)
          final distractors = <String>[];
          for (final id in e.distractorNodeIds) {
            final text = await service.fetchNodeText(id);
            distractors.add(text);
          }

          // Shuffle
          final allChoices = [correctAnswer, ...distractors]..shuffle();
          choices = allChoices;
          correctIndex = allChoices.indexOf(correctAnswer);

          // Fetch question (English translation)
          final translation = await service.fetchTranslation(
            e.nodeId,
            prefs.preferredTranslatorId ?? 1,
          );
          questionText = translation.text;
        },
        // Variants handled in their own widgets
        translation: (e) async {},
        contextualTranslation: (e) async {},
        clozeDeletion: (e) async {},
        firstLetterHint: (e) async {},
        missingWordMcq: (e) async {},
        nextWordMcq: (e) async {},
        fullVerseInput: (e) async {},
        ayahChain: (e) async {},
        findMistake: (e) async {},
        ayahSequence: (e) async {},
        sequenceRecall: (e) async {},
        firstWordRecall: (e) async {},
        identifyRoot: (e) async {},
        reverseCloze: (e) async {},
        translatePhrase: (e) async {},
        posTagging: (e) async {},
        crossVerseConnection: (e) async {},
        echoRecall: (e) async {
          // EchoRecall handles its own loading via start_echo_recall
          // No content pre-loading needed here
        },
      );

      if (!mounted || requestId != _loadRequestId) {
        return;
      }
      setState(() {
        _questionText = questionText;
        _answerText = answerText;
        _choices = choices;
        _correctIndex = correctIndex;
        _isLoading = false;
      });
    } catch (e, st) {
      if (!mounted || requestId != _loadRequestId) {
        return;
      }
      setState(() {
        _error = e.toString();
        _isLoading = false;
      });
      debugPrint('Error loading exercise content: $e\n$st');
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_isLoading) {
      return const Center(child: CircularProgressIndicator());
    }

    if (_error != null) {
      return Center(
        child: ErrorBanner(
          message: _error!,
          onRetry: _loadContent,
        ),
      );
    }

    // Render appropriate widget based on data
    // For now, using simple placeholders or reusing existing logic if possible
    return widget.exercise.map(
      memorization: (_) => _buildMemorization(),
      mcqArToEn: (_) => _buildMcq(isArabicToEnglish: true),
      mcqEnToAr: (_) => _buildMcq(isArabicToEnglish: false),
      translation: (e) => TranslationWidget(
        nodeId: e.nodeId,
        onComplete: widget.onComplete,
      ),
      contextualTranslation: (e) => ContextualTranslationWidget(
        nodeId: e.nodeId,
        verseKey: e.verseKey,
        onComplete: widget.onComplete,
      ),
      clozeDeletion: (e) => ClozeDeletionWidget(
        nodeId: e.nodeId,
        blankPosition: e.blankPosition,
        onComplete: widget.onComplete,
      ),
      firstLetterHint: (e) => FirstLetterHintWidget(
        nodeId: e.nodeId,
        wordPosition: e.wordPosition,
        onComplete: widget.onComplete,
      ),
      missingWordMcq: (e) => MissingWordMcqWidget(
        nodeId: e.nodeId,
        blankPosition: e.blankPosition,
        distractorNodeIds: e.distractorNodeIds,
        onComplete: widget.onComplete,
      ),
      nextWordMcq: (e) => NextWordMcqWidget(
        nodeId: e.nodeId,
        contextPosition: e.contextPosition,
        distractorNodeIds: e.distractorNodeIds,
        onComplete: widget.onComplete,
      ),
      fullVerseInput: (e) => FullVerseInputWidget(
        nodeId: e.nodeId,
        onComplete: widget.onComplete,
      ),
      ayahChain: (e) => AyahChainWidget(
        nodeId: e.nodeId,
        verseKeys: e.verseKeys,
        currentIndex: e.currentIndex.toInt(),
        completedCount: e.completedCount.toInt(),
        onComplete: widget.onComplete,
      ),
      findMistake: (e) => FindMistakeWidget(
        nodeId: e.nodeId,
        mistakePosition: e.mistakePosition,
        correctWordNodeId: e.correctWordNodeId,
        incorrectWordNodeId: e.incorrectWordNodeId,
        onComplete: widget.onComplete,
      ),
      ayahSequence: (e) => AyahSequenceWidget(
        nodeId: e.nodeId,
        correctSequence: e.correctSequence,
        onComplete: widget.onComplete,
      ),
      sequenceRecall: (e) => SequenceRecallWidget(
        nodeId: e.nodeId,
        correctSequence: e.correctSequence,
        options: e.options,
        onComplete: widget.onComplete,
      ),
      firstWordRecall: (e) => FirstWordRecallWidget(
        nodeId: e.nodeId,
        verseKey: e.verseKey,
        onComplete: widget.onComplete,
      ),
      identifyRoot: (e) => IdentifyRootWidget(
        nodeId: e.nodeId,
        root: e.root,
        onComplete: widget.onComplete,
      ),
      reverseCloze: (e) => ReverseClozeWidget(
        nodeId: e.nodeId,
        blankPosition: e.blankPosition,
        onComplete: widget.onComplete,
      ),
      translatePhrase: (e) => TranslatePhraseWidget(
        nodeId: e.nodeId,
        translatorId: e.translatorId,
        onComplete: widget.onComplete,
      ),
      posTagging: (e) => PosTaggingWidget(
        nodeId: e.nodeId,
        correctPos: e.correctPos,
        options: e.options,
        onComplete: widget.onComplete,
      ),
      crossVerseConnection: (e) => CrossVerseConnectionWidget(
        nodeId: e.nodeId,
        relatedVerseIds: e.relatedVerseIds,
        connectionTheme: e.connectionTheme,
        onComplete: widget.onComplete,
      ),
      echoRecall: (e) => EchoRecallWidget(
        userId: e.userId,
        ayahNodeIds: e.ayahNodeIds,
        onComplete: () => widget.onComplete(true),
      ),
    );
  }

  Widget _buildMemorization() {
    return Column(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        Text(
          _questionText ?? '',
          style: Theme.of(context).textTheme.headlineMedium,
        ),
        const SizedBox(height: 20),
        // TODO: Add reveal answer logic
        Text(_answerText ?? '', style: Theme.of(context).textTheme.bodyLarge),
      ],
    );
  }

  Widget _buildMcq({required bool isArabicToEnglish}) {
    return Column(
      children: [
        Text(
          _questionText ?? '',
          style: Theme.of(context).textTheme.headlineMedium,
        ),
        const SizedBox(height: 20),
        if (_choices != null)
          ..._choices!.asMap().entries.map((entry) {
            return Padding(
              padding: const EdgeInsets.all(8.0),
              child: Semantics(
                button: true,
                label: 'Select answer option ${entry.key + 1}',
                child: ElevatedButton(
                  onPressed: () {
                    widget.onComplete(entry.key == _correctIndex);
                  },
                  child: Text(entry.value),
                ),
              ),
            );
          }),
      ],
    );
  }
}
