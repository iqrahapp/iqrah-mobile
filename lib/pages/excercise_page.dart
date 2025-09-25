import 'dart:async';

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
  final Stopwatch _stopwatch = Stopwatch();
  Timer? _timer;
  Duration _elapsed = Duration.zero;
  ReviewGrade? _autoGrade;
  String? _feedbackLabel;
  Color? _feedbackColor;
  bool _showOverrideOptions = false;
  bool _isSubmittingAutoGrade = false;

  @override
  Widget build(BuildContext context) {
    ref.listen<SessionState>(sessionProvider, (prev, next) {
      if (prev == null) {
        if (next.currentExercise != null) {
          _handleExerciseChange(next);
        }
        return;
      }

      if ((!prev.isCompleted() && next.isCompleted()) ||
          next.exercises.isEmpty) {
        _stopTimer();
        Navigator.of(context).pushReplacement(
          MaterialPageRoute(builder: (_) => const SummaryPage()),
        );
      } else if (prev.currentIndex != next.currentIndex) {
        _handleExerciseChange(next);
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

  void _handleExerciseChange(SessionState state) {
    final exercise = state.currentExercise;
    _stopTimer();
    if (!mounted) return;
    setState(() {
      _isAnswerVisible = false;
      _selectedIndex = null;
      _autoGrade = null;
      _feedbackLabel = null;
      _feedbackColor = null;
      _showOverrideOptions = false;
      _elapsed = Duration.zero;
      _isSubmittingAutoGrade = false;
    });
    if (exercise != null) {
      _startTimer();
    }
  }

  void _startTimer() {
    _stopwatch
      ..reset()
      ..start();
    _timer?.cancel();
    _timer = Timer.periodic(const Duration(milliseconds: 100), (_) {
      if (!mounted) return;
      setState(() {
        _elapsed = _stopwatch.elapsed;
      });
    });
  }

  void _stopTimer() {
    _stopwatch.stop();
    _timer?.cancel();
    _timer = null;
  }

  void _handleMcqSelection(int selectedIndex, int correctIndex) {
    final elapsedNow = _stopwatch.elapsed;
    final isCorrect = selectedIndex == correctIndex;
    final grade = _computeAutoGrade(isCorrect, elapsedNow);
    final feedback = _feedbackTextFor(grade, isCorrect: isCorrect);
    final feedbackColor = _colorForGrade(grade);

    _stopTimer();
    setState(() {
      _elapsed = elapsedNow;
      _selectedIndex = selectedIndex;
      _isAnswerVisible = true;
      _autoGrade = grade;
      _feedbackLabel = feedback;
      _feedbackColor = feedbackColor;
      _showOverrideOptions = false;
    });
  }

  Future<void> _submitAutoGrade() async {
    final grade = _autoGrade;
    if (grade == null || _isSubmittingAutoGrade) {
      return;
    }

    setState(() {
      _isSubmittingAutoGrade = true;
    });

    try {
      await ref.read(sessionProvider.notifier).submitReview(grade);
    } finally {
      if (mounted) {
        setState(() {
          _isSubmittingAutoGrade = false;
        });
      }
    }
  }

  @override
  void dispose() {
    _stopTimer();
    super.dispose();
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
        _buildTimerIndicator(theme),
        const SizedBox(height: 16),
        Expanded(child: _buildExerciseCard(currentItem, theme)),
        const SizedBox(height: 24),
        _buildActionButtons(),
      ],
    );
  }

  Widget _buildTimerIndicator(ThemeData theme) {
    final projectedGrade = _autoGrade ?? _gradeForElapsed(_elapsed);
    final color = _colorForGrade(projectedGrade);
    final background = color.withValues(alpha: 0.12);
    final timeLabel = _formatElapsed(_elapsed);

    return AnimatedContainer(
      duration: const Duration(milliseconds: 200),
      padding: const EdgeInsets.symmetric(vertical: 10, horizontal: 16),
      decoration: BoxDecoration(
        color: background,
        borderRadius: BorderRadius.circular(14),
      ),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.center,
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(Icons.timer_rounded, color: color),
          const SizedBox(width: 8),
          Text(
            '$timeLabel s',
            style: theme.textTheme.titleMedium?.copyWith(
              color: color,
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(width: 12),
          Text(
            _autoGrade != null
                ? 'Auto: ${_labelForGrade(projectedGrade)}'
                : 'Target: ${_labelForGrade(projectedGrade)}',
            style: theme.textTheme.labelLarge?.copyWith(
              color: color,
              fontWeight: FontWeight.w600,
            ),
          ),
        ],
      ),
    );
  }

  ReviewGrade _gradeForElapsed(Duration elapsed) {
    if (elapsed < const Duration(seconds: 3)) {
      return ReviewGrade.easy;
    }
    if (elapsed <= const Duration(seconds: 10)) {
      return ReviewGrade.good;
    }
    return ReviewGrade.hard;
  }

  Color _colorForGrade(ReviewGrade grade) {
    switch (grade) {
      case ReviewGrade.again:
        return Colors.red.shade600;
      case ReviewGrade.hard:
        return Colors.orange.shade600;
      case ReviewGrade.good:
        return Colors.green.shade600;
      case ReviewGrade.easy:
        return Colors.blue.shade600;
    }
  }

  String _labelForGrade(ReviewGrade grade) {
    switch (grade) {
      case ReviewGrade.again:
        return 'Again';
      case ReviewGrade.hard:
        return 'Hard';
      case ReviewGrade.good:
        return 'Good';
      case ReviewGrade.easy:
        return 'Easy';
    }
  }

  String _formatElapsed(Duration elapsed) {
    final seconds = elapsed.inMilliseconds / 1000.0;
    return seconds.toStringAsFixed(1);
  }

  ReviewGrade _computeAutoGrade(bool isCorrect, Duration elapsed) {
    if (!isCorrect) {
      return ReviewGrade.again;
    }
    return _gradeForElapsed(elapsed);
  }

  String _feedbackTextFor(ReviewGrade grade, {required bool isCorrect}) {
    switch (grade) {
      case ReviewGrade.again:
        return isCorrect ? 'Again' : 'Again…';
      case ReviewGrade.hard:
        return 'Hard!';
      case ReviewGrade.good:
        return 'Good!';
      case ReviewGrade.easy:
        return 'Easy!';
    }
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
                        onSelect: (index) =>
                            _handleMcqSelection(index, correctIndex),
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
                        onSelect: (index) =>
                            _handleMcqSelection(index, correctIndex),
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
    final showAutoControls =
        _isAnswerVisible && _autoGrade != null && !_showOverrideOptions;
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
      child: showAutoControls
          ? _buildAutoContinueControls()
          : _isAnswerVisible
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
          _stopTimer();
          setState(() {
            _elapsed = _stopwatch.elapsed;
            _isAnswerVisible = true;
          });
        },
        child: const Text("Show Answer"),
      ),
    );
  }

  Widget _buildAutoContinueControls() {
    final grade = _autoGrade;
    if (grade == null) {
      return const SizedBox.shrink();
    }
    final label = _feedbackLabel ?? _labelForGrade(grade);
    final color = _feedbackColor ?? _colorForGrade(grade);

    return Column(
      key: const ValueKey('autoControls'),
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Container(
          padding: const EdgeInsets.symmetric(vertical: 12),
          alignment: Alignment.center,
          child: Text(
            label,
            style: Theme.of(context).textTheme.headlineSmall?.copyWith(
              color: color,
              fontWeight: FontWeight.bold,
            ),
          ),
        ),
        const SizedBox(height: 8),
        FilledButton(
          style: FilledButton.styleFrom(
            backgroundColor: color,
            foregroundColor: Colors.white,
            padding: const EdgeInsets.symmetric(vertical: 16),
            textStyle: const TextStyle(
              fontSize: 18,
              fontWeight: FontWeight.bold,
            ),
          ),
          onPressed: _submitAutoGrade,
          child: Text('Continue (${_labelForGrade(grade)})'),
        ),
        TextButton(
          onPressed: () {
            setState(() {
              _showOverrideOptions = true;
            });
          },
          child: const Text('Override grade'),
        ),
      ],
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
