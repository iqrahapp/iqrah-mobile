import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/rust_bridge/api.dart';
import 'package:iqrah/services/exercise_content_service.dart';

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
    setState(() {
      _isLoading = true;
      _error = null;
    });

    try {
      final service = ref.read(exerciseContentServiceProvider);
      final prefs = ref.read(userPreferencesProvider);

      await widget.exercise.map(
        memorization: (e) async {
          // Fetch word/verse text
          final content = await service.fetchWordContent(
            int.parse(e.nodeId.split(':').last),
            prefs,
          );
          _questionText = content.text;

          // Fetch translation
          final translation = await service.fetchTranslation(
            e.nodeId,
            prefs.preferredTranslatorId ?? 1,
          );
          _answerText = translation.text;
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
          _choices = allChoices;
          _correctIndex = allChoices.indexOf(correctTrans.text);

          // Fetch question (Arabic text)
          // Assuming nodeId is like "WORD:1:1:1"
          final wordId = int.tryParse(e.nodeId.split(':').last);
          if (wordId != null) {
            final content = await service.fetchWordContent(wordId, prefs);
            _questionText = content.text;
          } else {
            _questionText = "Error parsing ID";
          }
        },
        mcqEnToAr: (e) async {
          // Fetch correct answer (Arabic text)
          final wordId = int.tryParse(e.nodeId.split(':').last);
          if (wordId == null) throw Exception("Invalid node ID");

          final content = await service.fetchWordContent(wordId, prefs);
          final correctAnswer = content.text;

          // Fetch distractors (Arabic text)
          final distractors = <String>[];
          for (final id in e.distractorNodeIds) {
            final dId = int.tryParse(id.split(':').last);
            if (dId != null) {
              final d = await service.fetchWordContent(dId, prefs);
              distractors.add(d.text);
            }
          }

          // Shuffle
          final allChoices = [correctAnswer, ...distractors]..shuffle();
          _choices = allChoices;
          _correctIndex = allChoices.indexOf(correctAnswer);

          // Fetch question (English translation)
          final translation = await service.fetchTranslation(
            e.nodeId,
            prefs.preferredTranslatorId ?? 1,
          );
          _questionText = translation.text;
        },
        // Implement other variants as needed...
        translation: (e) async {},
        contextualTranslation: (e) async {},
        clozeDeletion: (e) async {
          // Parse verseKey from "VERSE:chapter:verse"
          // e.nodeId might be "VERSE:1:1"
          final parts = e.nodeId.split(':');
          if (parts.length < 3) throw Exception("Invalid verse ID");
          final verseKey = "${parts[1]}:${parts[2]}";

          // Fetch all words
          final words = await service.fetchWordsForVerse(verseKey);

          if (words.isEmpty) {
            _questionText = "Error loading verse";
            return;
          }

          // Construct question with blank
          final buffer = StringBuffer();
          String? answer;

          // Sort words by position just in case
          words.sort((a, b) => a.position.compareTo(b.position));

          for (final word in words) {
            if (word.position == e.blankPosition) {
              buffer.write("_____ ");
              answer = word.textUthmani;
            } else {
              buffer.write("${word.textUthmani} ");
            }
          }

          _questionText = buffer.toString().trim();
          _answerText = answer ?? "Error";
        },
        firstLetterHint: (e) async {},
        missingWordMcq: (e) async {},
        nextWordMcq: (e) async {},
        fullVerseInput: (e) async {},
        ayahChain: (e) async {},
        findMistake: (e) async {},
        ayahSequence: (e) async {},
        identifyRoot: (e) async {},
        reverseCloze: (e) async {},
        translatePhrase: (e) async {},
        posTagging: (e) async {},
        crossVerseConnection: (e) async {},
      );

      if (mounted) {
        setState(() {
          _isLoading = false;
        });
      }
    } catch (e, st) {
      if (mounted) {
        setState(() {
          _error = e.toString();
          _isLoading = false;
        });
      }
      debugPrint('Error loading exercise content: $e\n$st');
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_isLoading) {
      return const Center(child: CircularProgressIndicator());
    }

    if (_error != null) {
      return Center(child: Text('Error: $_error'));
    }

    // Render appropriate widget based on data
    // For now, using simple placeholders or reusing existing logic if possible
    return widget.exercise.map(
      memorization: (_) => _buildMemorization(),
      mcqArToEn: (_) => _buildMcq(isArabicToEnglish: true),
      mcqEnToAr: (_) => _buildMcq(isArabicToEnglish: false),
      // Fallback for unimplemented types
      translation: (_) => const Text('Translation exercise not implemented'),
      contextualTranslation: (_) =>
          const Text('Contextual Translation not implemented'),
      clozeDeletion: (_) => _buildMemorization(),
      firstLetterHint: (_) => const Text('First Letter Hint not implemented'),
      missingWordMcq: (_) => const Text('Missing Word MCQ not implemented'),
      nextWordMcq: (_) => const Text('Next Word MCQ not implemented'),
      fullVerseInput: (_) => const Text('Full Verse Input not implemented'),
      ayahChain: (_) => const Text('Ayah Chain not implemented'),
      findMistake: (_) => const Text('Find Mistake not implemented'),
      ayahSequence: (_) => const Text('Ayah Sequence not implemented'),
      identifyRoot: (_) => const Text('Identify Root not implemented'),
      reverseCloze: (_) => const Text('Reverse Cloze not implemented'),
      translatePhrase: (_) => const Text('Translate Phrase not implemented'),
      posTagging: (_) => const Text('POS Tagging not implemented'),
      crossVerseConnection: (_) =>
          const Text('Cross Verse Connection not implemented'),
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
              child: ElevatedButton(
                onPressed: () {
                  widget.onComplete(entry.key == _correctIndex);
                },
                child: Text(entry.value),
              ),
            );
          }),
      ],
    );
  }
}
