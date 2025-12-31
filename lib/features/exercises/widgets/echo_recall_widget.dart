import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:iqrah/features/exercises/widgets/blurred_arabic_word.dart';
import 'package:iqrah/rust_bridge/api.dart';
import 'package:iqrah/utils/app_logger.dart';

/// Echo Recall exercise widget with progressive blur, word timing, and struggle detection.
///
/// This is the primary memorization exercise. Words are displayed with varying
/// visibility based on the learner's mastery level (energy). As the user recalls
/// words by tapping, energy increases and words transition from Visible -> Obscured -> Hidden.
class EchoRecallWidget extends StatefulWidget {
  final String userId;
  final List<String> ayahNodeIds;
  final VoidCallback onComplete;

  const EchoRecallWidget({
    super.key,
    required this.userId,
    required this.ayahNodeIds,
    required this.onComplete,
  });

  @override
  State<EchoRecallWidget> createState() => _EchoRecallWidgetState();
}

class _EchoRecallWidgetState extends State<EchoRecallWidget>
    with SingleTickerProviderStateMixin {
  // State
  EchoRecallStateDto? _state;
  EchoRecallStatsDto? _stats;
  bool _isLoading = true;
  String? _error;
  int _currentWordIndex = 0;
  int _struggles = 0;
  bool _showingHelp = false;

  // Timing
  final Stopwatch _sessionStopwatch = Stopwatch();
  final Stopwatch _wordStopwatch = Stopwatch();
  Timer? _struggleTimer;
  static const _struggleThresholdMs = 5000;

  // Per-word timing collection for metrics pipeline
  final List<WordTimingDto> _wordTimings = [];

  // Animation
  late AnimationController _completionController;
  late Animation<double> _completionAnimation;

  @override
  void initState() {
    super.initState();
    _completionController = AnimationController(
      duration: const Duration(milliseconds: 500),
      vsync: this,
    );
    _completionAnimation = CurvedAnimation(
      parent: _completionController,
      curve: Curves.easeOutBack,
    );
    _initializeSession();
  }

  @override
  void dispose() {
    _struggleTimer?.cancel();
    _sessionStopwatch.stop();
    _wordStopwatch.stop();
    _completionController.dispose();
    super.dispose();
  }

  Future<void> _initializeSession() async {
    setState(() {
      _isLoading = true;
      _error = null;
    });

    try {
      AppLogger.exercise(
        'Starting Echo Recall session for ${widget.ayahNodeIds.length} ayahs',
      );

      final state = await startEchoRecall(
        userId: widget.userId,
        ayahNodeIds: widget.ayahNodeIds,
      );

      final stats = await echoRecallStats(state: state);

      if (mounted) {
        setState(() {
          _state = state;
          _stats = stats;
          _isLoading = false;
        });

        // Start timing
        _sessionStopwatch.start();
        _wordStopwatch.start();
        _startStruggleTimer();

        AppLogger.exercise(
          'Echo Recall loaded: ${stats.totalWords} words, '
          '${stats.visibleCount} visible, ${stats.obscuredCount} obscured, '
          '${stats.hiddenCount} hidden',
        );
      }
    } catch (e) {
      AppLogger.exercise('Failed to start Echo Recall', error: e);
      if (mounted) {
        setState(() {
          _error = e.toString();
          _isLoading = false;
        });
      }
    }
  }

  void _startStruggleTimer() {
    _struggleTimer?.cancel();
    _struggleTimer = Timer(
      const Duration(milliseconds: _struggleThresholdMs),
      _onStruggleDetected,
    );
  }

  /// Handle explicit help request from user (Phase 3 spec requirement)
  void _onHelpRequested() {
    if (!mounted || _state == null) return;

    // Cancel the automatic struggle timer since user requested help
    _struggleTimer?.cancel();

    // Trigger the same help behavior as automatic struggle detection
    _onStruggleDetected();

    AppLogger.exercise(
      'User requested help at word $_currentWordIndex',
    );
  }

  void _onStruggleDetected() {
    if (!mounted || _state == null) return;

    setState(() {
      _struggles++;
      _showingHelp = true;
      // Regress blur level for current word (Phase 3 spec requirement)
      _regressCurrentWordBlur();
    });

    // Haptic feedback for struggle
    HapticFeedback.mediumImpact();

    AppLogger.exercise(
      'Struggle detected at word $_currentWordIndex (total: $_struggles)',
    );

    // Auto-hide help after 2 seconds
    Future.delayed(const Duration(seconds: 2), () {
      if (mounted) {
        setState(() {
          _showingHelp = false;
        });
      }
    });

    // Restart timer for next potential struggle
    _startStruggleTimer();
  }

  /// Regress the current word's blur by one level (decrease coverage)
  /// This makes the word more visible when user struggles.
  void _regressCurrentWordBlur() {
    if (_state == null || _currentWordIndex >= _state!.words.length) return;

    final currentWord = _state!.words[_currentWordIndex];
    final visibility = currentWord.visibility;

    // Only regress if word is obscured (has coverage)
    if (visibility.visibilityType != 'obscured') return;

    final currentCoverage = visibility.coverage ?? 0.0;

    // Regress by one blur level (decrease coverage by ~0.17)
    // Blur levels: 0.0-0.17, 0.17-0.33, 0.33-0.50, 0.50-0.67, 0.67-0.83, 0.83-1.0
    final newCoverage = (currentCoverage - 0.17).clamp(0.0, 1.0);

    // If coverage drops to near zero, make it visible
    if (newCoverage < 0.05) {
      _state = EchoRecallStateDto(
        words: _state!.words.asMap().entries.map((entry) {
          if (entry.key == _currentWordIndex) {
            return EchoRecallWordDto(
              nodeId: entry.value.nodeId,
              text: entry.value.text,
              visibility: WordVisibilityDto(
                visibilityType: 'visible',
                hint: null,
                coverage: null,
              ),
              energy: entry.value.energy,
            );
          }
          return entry.value;
        }).toList(),
      );
    } else {
      // Update coverage while keeping word obscured
      _state = EchoRecallStateDto(
        words: _state!.words.asMap().entries.map((entry) {
          if (entry.key == _currentWordIndex) {
            return EchoRecallWordDto(
              nodeId: entry.value.nodeId,
              text: entry.value.text,
              visibility: WordVisibilityDto(
                visibilityType: 'obscured',
                hint: entry.value.visibility.hint,
                coverage: newCoverage,
              ),
              energy: entry.value.energy,
            );
          }
          return entry.value;
        }).toList(),
      );
    }

    AppLogger.exercise(
      'Regressed blur for word $_currentWordIndex: $currentCoverage -> $newCoverage',
    );
  }

  Future<void> _onWordTap(int index) async {
    if (_state == null) return;
    if (index != _currentWordIndex) return; // Only allow tapping current word

    final elapsed = _wordStopwatch.elapsedMilliseconds;
    final word = _state!.words[index];

    // Haptic feedback for tap
    HapticFeedback.lightImpact();

    _struggleTimer?.cancel();

    // Record per-word timing for metrics pipeline
    _wordTimings.add(WordTimingDto(
      wordNodeId: word.nodeId,
      durationMs: BigInt.from(elapsed),
    ));

    try {
      // Submit recall to backend (spec-compliant signature)
      final newState = await submitEchoRecall(
        userId: widget.userId,
        ayahNodeIds: widget.ayahNodeIds,
        state: _state!,
        wordNodeId: word.nodeId,
        recallTimeMs: elapsed,
      );

      final newStats = await echoRecallStats(state: newState);

      if (mounted) {
        setState(() {
          _state = newState;
          _stats = newStats;
          _currentWordIndex++;
          _showingHelp = false; // Clear help on successful tap
        });

        AppLogger.exercise(
          'Word ${word.nodeId} recalled in ${elapsed}ms, '
          'energy: ${word.energy.toStringAsFixed(2)} -> '
          '${newState.words[index].energy.toStringAsFixed(2)}',
        );

        // Reset stopwatch for next word
        _wordStopwatch.reset();
        _wordStopwatch.start();
        _startStruggleTimer();

        // Check if session is complete
        if (_currentWordIndex >= _state!.words.length) {
          await _onSessionComplete();
        }
      }
    } catch (e) {
      AppLogger.exercise('Failed to submit word recall', error: e);
      // Error recovery: restart timer and allow retry
      if (mounted) {
        setState(() {
          _showingHelp = false;
        });
        // Restart stopwatch and timer to allow user to retry
        _wordStopwatch.reset();
        _wordStopwatch.start();
        _startStruggleTimer();
      }
    }
  }

  Future<void> _onSessionComplete() async {
    _struggleTimer?.cancel();
    _sessionStopwatch.stop();
    _wordStopwatch.stop();

    // Play completion animation
    await _completionController.forward();
    HapticFeedback.heavyImpact();

    try {
      // Build metrics DTO with per-word timings
      final metrics = EchoRecallMetricsDto(
        wordTimings: _wordTimings,
        totalDurationMs: BigInt.from(_sessionStopwatch.elapsedMilliseconds),
        struggles: _struggles,
      );

      // Finalize and persist energy updates with full metrics
      final result = await finalizeEchoRecall(
        userId: widget.userId,
        state: _state!,
        metrics: metrics,
      );

      AppLogger.exercise(
        'Echo Recall complete: ${result.energyUpdates.length} energy updates, '
        '${_wordTimings.length} word timings, $_struggles struggles, '
        '${_sessionStopwatch.elapsedMilliseconds}ms total, '
        'avg energy: ${result.averageEnergy.toStringAsFixed(2)}',
      );

      // Delay before calling onComplete for animation to finish
      await Future.delayed(const Duration(milliseconds: 300));

      if (mounted) {
        widget.onComplete();
      }
    } catch (e) {
      AppLogger.exercise('Failed to finalize Echo Recall', error: e);
      // Still complete even if finalize fails
      widget.onComplete();
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_isLoading) {
      return const Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            CircularProgressIndicator(),
            SizedBox(height: 16),
            Text('Loading Echo Recall...'),
          ],
        ),
      );
    }

    if (_error != null) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            mainAxisSize: MainAxisSize.min,
            children: [
              const Icon(Icons.error_outline, size: 48, color: Colors.red),
              const SizedBox(height: 16),
              Text(
                'Error: $_error',
                textAlign: TextAlign.center,
                style: const TextStyle(color: Colors.red),
              ),
              const SizedBox(height: 16),
              Row(
                mainAxisAlignment: MainAxisAlignment.center,
                mainAxisSize: MainAxisSize.min,
                children: [
                  ElevatedButton(
                    onPressed: _initializeSession,
                    child: const Text('Retry'),
                  ),
                  const SizedBox(width: 16),
                  TextButton(
                    onPressed: widget.onComplete,
                    child: const Text('Skip'),
                  ),
                ],
              ),
            ],
          ),
        ),
      );
    }

    if (_state == null) {
      return const Center(child: Text('No data available'));
    }

    return Column(
      children: [
        // Progress header
        _buildProgressHeader(),

        // Main word display
        Expanded(
          child: _buildWordDisplay(),
        ),

        // Stats footer
        _buildStatsFooter(),
      ],
    );
  }

  Widget _buildProgressHeader() {
    final progress = _currentWordIndex / _state!.words.length;

    return Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        children: [
          // Progress bar
          LinearProgressIndicator(
            value: progress,
            backgroundColor: Theme.of(context).colorScheme.surfaceContainerHighest,
            valueColor: AlwaysStoppedAnimation<Color>(
              Theme.of(context).colorScheme.primary,
            ),
          ),
          const SizedBox(height: 8),
          // Progress text
          Text(
            '$_currentWordIndex / ${_state!.words.length} words',
            style: Theme.of(context).textTheme.bodyMedium,
          ),
        ],
      ),
    );
  }

  Widget _buildWordDisplay() {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            // Instruction text
            if (_showingHelp)
              Container(
                padding: const EdgeInsets.all(12),
                margin: const EdgeInsets.only(bottom: 16),
                decoration: BoxDecoration(
                  color: Theme.of(context).colorScheme.primaryContainer,
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(
                      Icons.lightbulb_outline,
                      color: Theme.of(context).colorScheme.primary,
                    ),
                    const SizedBox(width: 8),
                    Text(
                      'Take your time - tap when ready',
                      style: TextStyle(
                        color: Theme.of(context).colorScheme.onPrimaryContainer,
                      ),
                    ),
                  ],
                ),
              ),

            // Word flow (RTL)
            Directionality(
              textDirection: TextDirection.rtl,
              child: Wrap(
                alignment: WrapAlignment.center,
                spacing: 8,
                runSpacing: 12,
                children: _buildWordList(),
              ),
            ),

            const SizedBox(height: 24),

            // Tap instruction and help button
            if (_currentWordIndex < _state!.words.length) ...[
              Text(
                'Tap the highlighted word to continue',
                style: Theme.of(context).textTheme.bodySmall?.copyWith(
                      color: Theme.of(context).colorScheme.outline,
                    ),
              ),
              const SizedBox(height: 12),
              // Explicit help request button (Phase 3 spec requirement)
              TextButton.icon(
                onPressed: _onHelpRequested,
                icon: Icon(
                  Icons.help_outline,
                  size: 18,
                  color: Theme.of(context).colorScheme.outline,
                ),
                label: Text(
                  'Need a hint?',
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                        color: Theme.of(context).colorScheme.outline,
                      ),
                ),
              ),
            ],

            // Completion animation
            if (_currentWordIndex >= _state!.words.length)
              ScaleTransition(
                scale: _completionAnimation,
                child: Column(
                  children: [
                    Icon(
                      Icons.check_circle,
                      size: 64,
                      color: Theme.of(context).colorScheme.primary,
                    ),
                    const SizedBox(height: 8),
                    Text(
                      'Session Complete!',
                      style: Theme.of(context).textTheme.headlineSmall,
                    ),
                  ],
                ),
              ),
          ],
        ),
      ),
    );
  }

  List<Widget> _buildWordList() {
    return _state!.words.asMap().entries.map((entry) {
      final index = entry.key;
      final word = entry.value;
      final isActive = index == _currentWordIndex;
      final isCompleted = index < _currentWordIndex;

      return AnimatedScale(
        scale: isActive ? 1.05 : 1.0,
        duration: const Duration(milliseconds: 150),
        child: BlurredArabicWord(
          text: word.text,
          visibility: word.visibility,
          isActive: isActive,
          isCompleted: isCompleted,
          onTap: isActive ? () => _onWordTap(index) : null,
        ),
      );
    }).toList();
  }

  Widget _buildStatsFooter() {
    if (_stats == null) return const SizedBox.shrink();

    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHighest,
        borderRadius: const BorderRadius.vertical(top: Radius.circular(16)),
      ),
      child: SafeArea(
        top: false,
        child: Row(
          mainAxisAlignment: MainAxisAlignment.spaceAround,
          children: [
            _buildStatItem(
              'Visible',
              _stats!.visibleCount.toString(),
              Icons.visibility,
            ),
            _buildStatItem(
              'Learning',
              _stats!.obscuredCount.toString(),
              Icons.blur_on,
            ),
            _buildStatItem(
              'Mastered',
              _stats!.hiddenCount.toString(),
              Icons.check_circle_outline,
            ),
            _buildStatItem(
              'Mastery',
              '${_stats!.masteryPercentage.toStringAsFixed(0)}%',
              Icons.star_outline,
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildStatItem(String label, String value, IconData icon) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        Icon(
          icon,
          size: 20,
          color: Theme.of(context).colorScheme.primary,
        ),
        const SizedBox(height: 4),
        Text(
          value,
          style: Theme.of(context).textTheme.titleMedium?.copyWith(
                fontWeight: FontWeight.bold,
              ),
        ),
        Text(
          label,
          style: Theme.of(context).textTheme.bodySmall,
        ),
      ],
    );
  }
}
