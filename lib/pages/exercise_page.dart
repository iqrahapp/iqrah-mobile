import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/features/exercises/widgets/exercise_container.dart';
import 'package:iqrah/pages/session_summary_page.dart';
import 'package:iqrah/providers/session_provider.dart';
import 'package:iqrah/rust_bridge/api.dart' as api;
import 'package:iqrah/rust_bridge/repository.dart';

class ExercisePage extends ConsumerStatefulWidget {
  const ExercisePage({super.key});

  @override
  ConsumerState<ExercisePage> createState() => _ExercisePageState();
}

class _ExercisePageState extends ConsumerState<ExercisePage>
    with WidgetsBindingObserver {
  bool _isAnswerVisible = false;
  final Stopwatch _stopwatch = Stopwatch();
  Timer? _timer;
  Duration _elapsed = Duration.zero;
  ReviewGrade? _autoGrade;
  String? _feedbackLabel;
  Color? _feedbackColor;
  bool _showOverrideOptions = false;
  bool _isSubmittingAutoGrade = false;

  static const _timingQuick = _TimingProfile(
    easy: Duration(seconds: 3),
    good: Duration(seconds: 7),
    hard: Duration(seconds: 12),
  );
  static const _timingStandard = _TimingProfile(
    easy: Duration(seconds: 6),
    good: Duration(seconds: 12),
    hard: Duration(seconds: 20),
  );
  static const _timingExtended = _TimingProfile(
    easy: Duration(seconds: 10),
    good: Duration(seconds: 20),
    hard: Duration(seconds: 35),
  );

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
  }

  @override
  Widget build(BuildContext context) {
    ref.listen<SessionState>(sessionProvider, (prev, next) {
      if (prev == null) {
        if (next.currentExercise != null) {
          _handleExerciseChange(next);
        }
        return;
      }

      if (!prev.isCompleted() && next.isCompleted()) {
        _stopTimer();
        final reviewCount = next.mode == SessionMode.adhoc
            ? next.adhocExercises.length
            : (next.summary?.itemsCompleted ?? 0);
        Navigator.of(context).pushReplacement(
          MaterialPageRoute(
            builder: (_) => SessionSummaryPage(
              reviewCount: reviewCount,
              summary: next.summary,
            ),
          ),
        );
      } else if (prev.currentExercise != next.currentExercise) {
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

  void _startTimer({bool reset = true}) {
    if (reset) {
      _stopwatch.reset();
      if (mounted) {
        setState(() {
          _elapsed = Duration.zero;
        });
      } else {
        _elapsed = Duration.zero;
      }
    }
    if (!_stopwatch.isRunning) {
      _stopwatch.start();
    }
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

  void _pauseTimerForLifecycle() {
    if (!_stopwatch.isRunning) return;
    final elapsedNow = _stopwatch.elapsed;
    _stopTimer();
    if (!mounted) return;
    setState(() {
      _elapsed = elapsedNow;
    });
  }

  void _resumeTimerForLifecycle() {
    final currentExercise = ref.read(sessionProvider).currentExercise;
    if (currentExercise == null) {
      return;
    }
    if (_stopwatch.isRunning) {
      return;
    }
    _startTimer(reset: false);
  }

  void _handleCompletion(bool isCorrect) {
    final elapsedNow = _stopwatch.elapsed;
    final exercise = ref.read(sessionProvider).currentExercise;
    final grade = _computeAutoGrade(isCorrect, elapsedNow, exercise);
    final feedback = _feedbackTextFor(grade, isCorrect: isCorrect);
    final feedbackColor = _colorForGrade(grade);

    _stopTimer();
    setState(() {
      _elapsed = elapsedNow;
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
      // Map ReviewGrade to int for API
      // Assuming 1=Again, 2=Hard, 3=Good, 4=Easy
      int gradeInt = 3;
      switch (grade) {
        case ReviewGrade.again:
          gradeInt = 1;
          break;
        case ReviewGrade.hard:
          gradeInt = 2;
          break;
        case ReviewGrade.good:
          gradeInt = 3;
          break;
        case ReviewGrade.easy:
          gradeInt = 4;
          break;
      }

      await ref
          .read(sessionProvider.notifier)
          .submitReview(gradeInt, _elapsed.inMilliseconds);
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
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    switch (state) {
      case AppLifecycleState.resumed:
        _resumeTimerForLifecycle();
        break;
      case AppLifecycleState.inactive:
      case AppLifecycleState.paused:
      case AppLifecycleState.detached:
      case AppLifecycleState.hidden:
        _pauseTimerForLifecycle();
        break;
    }
  }

  Widget _buildLoadingState() {
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

  Widget _buildExerciseContent(api.ExerciseDataDto currentItem, ThemeData theme) {
    return Column(
      key: const ValueKey('content'),
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        _buildTimerIndicator(theme, currentItem),
        const SizedBox(height: 16),
        Expanded(
          child: Card(
            elevation: 4.0,
            shape:
                RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
            child: Padding(
              padding: const EdgeInsets.all(16.0),
              child: ExerciseContainer(
                exercise: currentItem,
                onComplete: _handleCompletion,
              ),
            ),
          ),
        ),
        const SizedBox(height: 24),
        _buildActionButtons(),
      ],
    );
  }

  ReviewGrade _gradeForElapsed(Duration elapsed, api.ExerciseDataDto? exercise) {
    final profile = exercise == null
        ? _timingStandard
        : _timingProfileFor(exercise);
    if (elapsed <= profile.easy) {
      return ReviewGrade.easy;
    }
    if (elapsed <= profile.good) {
      return ReviewGrade.good;
    }
    if (elapsed <= profile.hard) {
      return ReviewGrade.hard;
    }
    return ReviewGrade.again;
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

  ReviewGrade _computeAutoGrade(
    bool isCorrect,
    Duration elapsed,
    api.ExerciseDataDto? exercise,
  ) {
    if (!isCorrect) {
      return ReviewGrade.again;
    }
    return _gradeForElapsed(elapsed, exercise);
  }

  String _feedbackTextFor(ReviewGrade grade, {required bool isCorrect}) {
    switch (grade) {
      case ReviewGrade.again:
        return isCorrect ? 'Too slow' : 'Again…';
      case ReviewGrade.hard:
        return 'Hard!';
      case ReviewGrade.good:
        return 'Good!';
      case ReviewGrade.easy:
        return 'Easy!';
    }
  }

  Widget _buildTimerIndicator(
    ThemeData theme,
    api.ExerciseDataDto currentItem,
  ) {
    final projectedGrade =
        _autoGrade ?? _gradeForElapsed(_elapsed, currentItem);
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
      child: Semantics(
        button: true,
        label: 'Show answer',
        child: FilledButton(
          style: FilledButton.styleFrom(
            padding: const EdgeInsets.symmetric(vertical: 16),
            textStyle:
                const TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
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
          child: Semantics(
            button: true,
            label: 'Continue with ${_labelForGrade(grade)}',
            child: Text('Continue (${_labelForGrade(grade)})'),
          ),
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
        child: Semantics(
          button: true,
          label: 'Grade as $title',
          child: ElevatedButton(
            style: ElevatedButton.styleFrom(
              backgroundColor: color,
              foregroundColor: Colors.white,
              padding: const EdgeInsets.symmetric(vertical: 12),
              shape: RoundedRectangleBorder(
                borderRadius: BorderRadius.circular(12),
              ),
            ),
            onPressed: () async {
              // Map ReviewGrade to int
              int gradeInt = 3;
              switch (grade) {
                case ReviewGrade.again:
                  gradeInt = 1;
                  break;
                case ReviewGrade.hard:
                  gradeInt = 2;
                  break;
                case ReviewGrade.good:
                  gradeInt = 3;
                  break;
                case ReviewGrade.easy:
                  gradeInt = 4;
                  break;
              }
              await ref
                  .read(sessionProvider.notifier)
                  .submitReview(gradeInt, _elapsed.inMilliseconds);
            },
            child: Text(title),
          ),
        ),
      ),
    );
  }

  _TimingProfile _timingProfileFor(api.ExerciseDataDto exercise) {
    return exercise.map(
      memorization: (_) => _timingExtended,
      mcqArToEn: (_) => _timingQuick,
      mcqEnToAr: (_) => _timingQuick,
      translation: (_) => _timingStandard,
      contextualTranslation: (_) => _timingStandard,
      clozeDeletion: (_) => _timingStandard,
      firstLetterHint: (_) => _timingQuick,
      missingWordMcq: (_) => _timingQuick,
      nextWordMcq: (_) => _timingQuick,
      fullVerseInput: (_) => _timingExtended,
      ayahChain: (_) => _timingExtended,
      findMistake: (_) => _timingStandard,
      ayahSequence: (_) => _timingStandard,
      sequenceRecall: (_) => _timingStandard,
      firstWordRecall: (_) => _timingQuick,
      identifyRoot: (_) => _timingQuick,
      reverseCloze: (_) => _timingStandard,
      translatePhrase: (_) => _timingStandard,
      posTagging: (_) => _timingQuick,
      crossVerseConnection: (_) => _timingStandard,
      echoRecall: (_) => _timingExtended,
    );
  }
}

class _TimingProfile {
  final Duration easy;
  final Duration good;
  final Duration hard;

  const _TimingProfile({
    required this.easy,
    required this.good,
    required this.hard,
  });
}
