import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/pages/summary_page.dart';
import 'package:iqrah/providers/session_provider.dart';
import 'package:iqrah/rust_bridge/exercises.dart';
import 'package:iqrah/rust_bridge/repository.dart';

class ExcercisePage extends ConsumerStatefulWidget {
  const ExcercisePage({super.key});

  @override
  ConsumerState<ExcercisePage> createState() => _ExcercisePageState();
}

class _ExcercisePageState extends ConsumerState<ExcercisePage> {
  bool _isAnswerVisible = false;
  int? _selectedIndex; // for MCQ

  @override
  Widget build(BuildContext context) {
    ref.listen<SessionState>(sessionProvider, (prev, next) {
      if (prev == null) return;

      if ((!prev.isCompleted() && next.isCompleted()) ||
          next.exercises.isEmpty) {
        Navigator.of(context).pushReplacement(
          MaterialPageRoute(builder: (_) => const SummaryPage()),
        );
      } else if (prev.currentIndex != next.currentIndex) {
        setState(() {
          _isAnswerVisible = false;
          _selectedIndex = null;
        });
      }
    });

    final currentItem = ref.watch(
      sessionProvider.select((s) => s.currentExercise),
    );

    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;

    return Scaffold(
      backgroundColor: colorScheme.surfaceContainerLowest,
      appBar: AppBar(
        title: const Text('Review Session'),
        backgroundColor: colorScheme.surface,
        elevation: 1,
      ),
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(20.0),
          child: AnimatedSwitcher(
            duration: const Duration(milliseconds: 300),
            child: currentItem == null
                ? _buildLoadingState()
                : _buildExerciseContent(currentItem, theme),
          ),
        ),
      ),
    );
  }

  Widget _buildLoadingState() {
    // ... (This widget is unchanged)
    return const Center(
      key: ValueKey('loading'),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          CircularProgressIndicator(),
          SizedBox(height: 20),
          Text("Preparing your review..."),
        ],
      ),
    );
  }

  Widget _buildExerciseContent(Exercise currentItem, ThemeData theme) {
    // ... (This widget is unchanged)
    return Column(
      key: const ValueKey('content'),
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Expanded(child: _buildExerciseCard(currentItem, theme)),
        const SizedBox(height: 24),
        _buildActionButtons(),
      ],
    );
  }

  // MODIFIED to build different content for question and answer
  Widget _buildExerciseCard(Exercise currentItem, ThemeData theme) {
    return Card(
      elevation: 4.0,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
      child: AnimatedSwitcher(
        duration: const Duration(milliseconds: 400),
        transitionBuilder: (child, animation) {
          return FadeTransition(opacity: animation, child: child);
        },
        child: _isAnswerVisible
            ? _buildCardFace(
                "Answer",
                theme,
                key: const ValueKey('answer'),
                content: currentItem.when(
                  recall: (nodeId, arabic, translation) => _buildPlainText(
                    translation,
                    theme,
                    style: theme.textTheme.headlineSmall,
                  ),
                  cloze: (nodeId, question, answer) =>
                      _buildPlainText(answer, theme, isArabic: true),
                  mcqArToEn:
                      (
                        nodeId,
                        arabic,
                        verseArabic,
                        surahNumber,
                        ayahNumber,
                        wordIndex,
                        choicesEn,
                        correctIndex,
                      ) => _buildMcqAnswer(
                        theme,
                        selectedIndex: _selectedIndex,
                        correctIndex: correctIndex,
                        correctLabel: choicesEn[correctIndex],
                        verseArabic: verseArabic,
                        wordIndex: wordIndex,
                      ),
                  mcqEnToAr:
                      (
                        nodeId,
                        english,
                        verseArabic,
                        surahNumber,
                        ayahNumber,
                        wordIndex,
                        choicesAr,
                        correctIndex,
                      ) => _buildMcqAnswer(
                        theme,
                        selectedIndex: _selectedIndex,
                        correctIndex: correctIndex,
                        correctLabel: choicesAr[correctIndex],
                        verseArabic: verseArabic,
                        wordIndex: wordIndex,
                        isArabic: true,
                      ),
                ),
              )
            : _buildCardFace(
                "Question",
                theme,
                key: const ValueKey('question'),
                content: currentItem.when(
                  recall: (nodeId, arabic, translation) => _buildPlainText(
                    arabic,
                    theme,
                    isArabic: true,
                    style: theme.textTheme.displaySmall?.copyWith(
                      fontFamily: 'Amiri',
                      height: 1.5,
                    ),
                  ),
                  cloze: (nodeId, question, answer) =>
                      _buildPlainText(question, theme, isArabic: true),
                  mcqArToEn:
                      (
                        nodeId,
                        arabic,
                        verseArabic,
                        surahNumber,
                        ayahNumber,
                        wordIndex,
                        choicesEn,
                        correctIndex,
                      ) => _buildMcqQuestion(
                        theme,
                        verseArabic: verseArabic,
                        wordIndex: wordIndex,
                        prompt: arabic,
                        choices: choicesEn,
                        isArabicChoices: false,
                        onSelect: (index) {
                          setState(() {
                            _selectedIndex = index;
                            _isAnswerVisible = true;
                          });
                        },
                      ),
                  mcqEnToAr:
                      (
                        nodeId,
                        english,
                        verseArabic,
                        surahNumber,
                        ayahNumber,
                        wordIndex,
                        choicesAr,
                        correctIndex,
                      ) => _buildMcqQuestion(
                        theme,
                        verseArabic: verseArabic,
                        wordIndex: wordIndex,
                        prompt: english,
                        choices: choicesAr,
                        isArabicChoices: true,
                        onSelect: (index) {
                          setState(() {
                            _selectedIndex = index;
                            _isAnswerVisible = true;
                          });
                        },
                      ),
                ),
              ),
      ),
    );
  }

  Widget _buildPlainText(
    String text,
    ThemeData theme, {
    bool isArabic = false,
    TextStyle? style,
  }) {
    return Text(
      text,
      textAlign: TextAlign.center,
      textDirection: isArabic ? TextDirection.rtl : TextDirection.ltr,
      style:
          style ??
          (isArabic
              ? theme.textTheme.headlineMedium?.copyWith(fontFamily: 'Amiri')
              : theme.textTheme.headlineSmall),
    );
  }

  Widget _buildMcqQuestion(
    ThemeData theme, {
    required String verseArabic,
    required int wordIndex,
    required String prompt,
    required List<String> choices,
    required bool isArabicChoices,
    required void Function(int index) onSelect,
  }) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        _buildHighlightedVerse(verseArabic, wordIndex, theme),
        const SizedBox(height: 16),
        Text(
          isArabicChoices ? prompt : 'What is the meaning of: $prompt',
          style: theme.textTheme.titleMedium,
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 16),
        ...List.generate(choices.length, (i) {
          final choice = choices[i];
          return Padding(
            padding: const EdgeInsets.symmetric(vertical: 4.0),
            child: OutlinedButton(
              onPressed: () => onSelect(i),
              style: OutlinedButton.styleFrom(
                alignment: isArabicChoices
                    ? Alignment.centerRight
                    : Alignment.center,
              ),
              child: Text(
                choice,
                style:
                    (isArabicChoices
                            ? theme.textTheme.titleLarge?.copyWith(
                                fontFamily: 'Amiri',
                              )
                            : theme.textTheme.titleMedium)
                        ?.copyWith(height: 1.4),
                textAlign: isArabicChoices ? TextAlign.right : TextAlign.center,
                textDirection: isArabicChoices
                    ? TextDirection.rtl
                    : TextDirection.ltr,
              ),
            ),
          );
        }),
      ],
    );
  }

  Widget _buildMcqAnswer(
    ThemeData theme, {
    required int? selectedIndex,
    required int correctIndex,
    required String correctLabel,
    required String verseArabic,
    required int wordIndex,
    bool isArabic = false,
  }) {
    final isCorrect =
        (selectedIndex != null) && (selectedIndex == correctIndex);
    final color = isCorrect ? Colors.green : Colors.red;
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        _buildHighlightedVerse(verseArabic, wordIndex, theme),
        const SizedBox(height: 16),
        if (selectedIndex != null)
          Text(
            isCorrect ? 'Correct' : 'Incorrect',
            style: theme.textTheme.titleMedium?.copyWith(color: color),
          ),
        const SizedBox(height: 8),
        Text(
          'Answer: $correctLabel',
          style:
              (isArabic
                      ? theme.textTheme.titleLarge?.copyWith(
                          fontFamily: 'Amiri',
                        )
                      : theme.textTheme.titleMedium)
                  ?.copyWith(height: 1.4),
          textAlign: isArabic ? TextAlign.right : TextAlign.center,
          textDirection: isArabic ? TextDirection.rtl : TextDirection.ltr,
        ),
      ],
    );
  }

  Widget _buildHighlightedVerse(
    String verseArabic,
    int wordIndex,
    ThemeData theme,
  ) {
    final words = verseArabic.split(RegExp(r"\s+"));
    final idx = (wordIndex - 1).clamp(0, words.length - 1);
    return Wrap(
      alignment: WrapAlignment.center,
      textDirection: TextDirection.rtl,
      runSpacing: 4,
      spacing: 6,
      children: [
        for (var i = 0; i < words.length; i++)
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 2),
            decoration: i == idx
                ? BoxDecoration(
                    color: theme.colorScheme.primaryContainer,
                    borderRadius: BorderRadius.circular(6),
                  )
                : null,
            child: Text(
              words[i],
              style: theme.textTheme.titleLarge?.copyWith(
                fontFamily: 'Amiri',
                height: 1.6,
              ),
              textDirection: TextDirection.rtl,
            ),
          ),
      ],
    );
  }

  // MODIFIED to accept a generic `content` widget
  Widget _buildCardFace(
    String label,
    ThemeData theme, {
    required Widget content,
    Key? key,
  }) {
    return Container(
      key: key,
      width: double.infinity,
      padding: const EdgeInsets.all(16.0),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Text(
            label,
            style: theme.textTheme.titleSmall?.copyWith(
              color: theme.colorScheme.secondary,
            ),
          ),
          const SizedBox(height: 16),
          content, // Display the passed-in content widget
        ],
      ),
    );
  }

  // --- The widgets below are unchanged ---

  Widget _buildActionButtons() {
    return AnimatedSwitcher(
      duration: const Duration(milliseconds: 200),
      transitionBuilder: (child, animation) {
        return FadeTransition(
          opacity: animation,
          child: SlideTransition(
            position: Tween<Offset>(
              begin: const Offset(0.0, 0.5),
              end: Offset.zero,
            ).animate(animation),
            child: child,
          ),
        );
      },
      child: _isAnswerVisible
          ? _buildGradeButtonsRow()
          : _buildShowAnswerButton(),
    );
  }

  Widget _buildShowAnswerButton() {
    return SizedBox(
      key: const ValueKey('showAnswer'),
      width: double.infinity,
      child: FilledButton(
        style: FilledButton.styleFrom(
          padding: const EdgeInsets.symmetric(vertical: 16),
          textStyle: const TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
        ),
        onPressed: () {
          HapticFeedback.mediumImpact();
          setState(() {
            _isAnswerVisible = true;
          });
        },
        child: const Text("Show Answer"),
      ),
    );
  }

  Widget _buildGradeButtonsRow() {
    return Row(
      key: const ValueKey('gradeButtons'),
      mainAxisAlignment: MainAxisAlignment.spaceBetween,
      children: [
        _buildGradeButton("Again", ReviewGrade.again, Colors.red.shade700),
        _buildGradeButton("Hard", ReviewGrade.hard, Colors.orange.shade700),
        _buildGradeButton("Good", ReviewGrade.good, Colors.green.shade700),
        _buildGradeButton("Easy", ReviewGrade.easy, Colors.blue.shade700),
      ],
    );
  }

  Widget _buildGradeButton(String title, ReviewGrade grade, Color color) {
    return Expanded(
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 4.0),
        child: ElevatedButton(
          style: ElevatedButton.styleFrom(
            backgroundColor: color,
            foregroundColor: Colors.white,
            padding: const EdgeInsets.symmetric(vertical: 12),
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(12),
            ),
          ),
          onPressed: () {
            ref.read(sessionProvider.notifier).submitReview(grade);
          },
          child: Text(title),
        ),
      ),
    );
  }
}
