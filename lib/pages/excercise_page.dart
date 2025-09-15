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
                "Translation",
                theme,
                key: const ValueKey('answer'),
                content: _buildHighlightedTranslation(currentItem, theme),
              )
            : _buildCardFace(
                "Original",
                theme,
                key: const ValueKey('question'),
                content: Text(
                  currentItem.arabic,
                  textAlign: TextAlign.center,
                  style: theme.textTheme.displaySmall?.copyWith(
                    fontFamily: 'Amiri',
                    height: 1.5,
                  ),
                ),
              ),
      ),
    );
  }

  Widget _buildHighlightedTranslation(Exercise item, ThemeData theme) {
    final fullText = item.translation;
    final clozeWord = item.arabic;

    // KEY CHANGE: Use a smaller, more consistent text style for all translations.
    // This prevents awkward wrapping on long sentences.
    final textStyle = theme.textTheme.headlineSmall;

    if (clozeWord.isEmpty || !fullText.contains(clozeWord)) {
      return Text(
        fullText,
        textAlign: TextAlign.center,
        style: textStyle, // Apply the consistent style
      );
    }

    final parts = fullText.split(clozeWord);

    final highlightStyle = textStyle?.copyWith(
      // fontWeight: FontWeight.bold,
      color: theme.colorScheme.primary,
      // backgroundColor: theme.colorScheme.primaryContainer.withOpacity(
      //   0.3,
      // ), // Added a subtle background highlight
    );

    return RichText(
      textAlign: TextAlign.center,
      text: TextSpan(
        style: textStyle, // Apply the consistent style
        children: [
          TextSpan(text: parts[0]),
          TextSpan(text: clozeWord, style: highlightStyle),
          if (parts.length > 1) TextSpan(text: parts[1]),
        ],
      ),
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
