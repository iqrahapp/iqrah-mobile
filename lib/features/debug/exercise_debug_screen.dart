import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/features/debug/exercise_type_dropdown.dart';
import 'package:iqrah/features/debug/node_selector_widget.dart';
import 'package:iqrah/features/session/session_screen.dart';
import 'package:iqrah/providers/session_provider.dart';
import 'package:iqrah/rust_bridge/api.dart' as api;
import 'package:iqrah/utils/app_logger.dart';

/// Debug screen for launching exercises with a specific node.
/// Uses auto-select mode where the system determines the best exercise type.
class ExerciseDebugScreen extends ConsumerStatefulWidget {
  const ExerciseDebugScreen({super.key});

  @override
  ConsumerState<ExerciseDebugScreen> createState() =>
      _ExerciseDebugScreenState();
}

class _ExerciseDebugScreenState extends ConsumerState<ExerciseDebugScreen> {
  static const _autoType = 'Auto (best fit)';
  String? _selectedNodeId;
  api.ExerciseDataDto? _generatedExercise;
  List<api.ExerciseDataDto> _availableExercises = [];
  String? _selectedType;
  bool _loading = false;
  String? _error;

  Future<void> _loadExercises(String nodeId) async {
    setState(() {
      _selectedNodeId = nodeId;
      _loading = true;
      _error = null;
      _generatedExercise = null;
      _availableExercises = [];
      _selectedType = _autoType;
    });

    AppLogger.exercise('Loading exercises for node: $nodeId');

    try {
      final exercises = await api.getExercisesForNode(nodeId: nodeId);
      final exercise = await api.generateExerciseV2(nodeId: nodeId);
      if (mounted) {
        setState(() {
          _generatedExercise = exercise;
          _availableExercises = exercises;
          _loading = false;
        });
        AppLogger.exercise(
            'Generated: ${_exerciseTypeName(exercise)} (available=${exercises.length})');
      }
    } catch (e) {
      AppLogger.exercise('Failed to generate exercise', error: e);
      if (mounted) {
        setState(() {
          _error = e.toString();
          _loading = false;
        });
      }
    }
  }

  Future<void> _selectExerciseType(String? type) async {
    if (_selectedNodeId == null) return;

    setState(() {
      _selectedType = type;
      _error = null;
    });

    if (type == null || type == _autoType) {
      await _generateAutoExercise();
      return;
    }

    final match = _availableExercises
        .where((e) => _exerciseTypeKey(e) == type)
        .toList();
    if (match.isEmpty) {
      setState(() {
        _error = 'No exercise found for type: $type';
        _generatedExercise = null;
      });
      return;
    }

    setState(() {
      _generatedExercise = match.first;
    });
  }

  Future<void> _generateAutoExercise() async {
    if (_selectedNodeId == null) return;

    setState(() {
      _loading = true;
    });

    try {
      final exercise = await api.generateExerciseV2(nodeId: _selectedNodeId!);
      if (mounted) {
        setState(() {
          _generatedExercise = exercise;
          _loading = false;
        });
        AppLogger.exercise('Generated: ${_exerciseTypeName(exercise)}');
      }
    } catch (e) {
      AppLogger.exercise('Failed to generate exercise', error: e);
      if (mounted) {
        setState(() {
          _error = e.toString();
          _loading = false;
        });
      }
    }
  }

  void _launchExercise() {
    if (_generatedExercise == null) return;

    HapticFeedback.lightImpact();
    AppLogger.exercise('Launching exercise for $_selectedNodeId');

    ref.read(sessionProvider.notifier).startAdhocReview([_generatedExercise!]);
    Navigator.of(context).push(
      MaterialPageRoute(builder: (_) => const SessionScreen()),
    );
  }

  String _exerciseTypeName(api.ExerciseDataDto exercise) {
    return _exerciseTypeLabel(exercise);
  }

  String _exerciseTypeKey(api.ExerciseDataDto exercise) {
    final name = exercise.runtimeType.toString();
    const prefix = 'ExerciseDataDto_';
    if (name.startsWith(prefix)) {
      return name.substring(prefix.length);
    }
    return name;
  }

  List<String> _exerciseTypeOptions() {
    final types = _availableExercises.map(_exerciseTypeKey).toSet().toList()
      ..sort();
    return [_autoType, ...types];
  }

  String _exerciseTypeLabel(api.ExerciseDataDto e) {
    return e.map(
      memorization: (_) => 'Memorization',
      mcqArToEn: (_) => 'MCQ Ar to En',
      mcqEnToAr: (_) => 'MCQ En to Ar',
      translation: (_) => 'Translation',
      contextualTranslation: (_) => 'Contextual Translation',
      clozeDeletion: (_) => 'Cloze Deletion',
      firstLetterHint: (_) => 'First Letter Hint',
      missingWordMcq: (_) => 'Missing Word MCQ',
      nextWordMcq: (_) => 'Next Word MCQ',
      fullVerseInput: (_) => 'Full Verse Input',
      ayahChain: (_) => 'Ayah Chain',
      findMistake: (_) => 'Find the Mistake',
      ayahSequence: (_) => 'Ayah Sequence',
      sequenceRecall: (_) => 'Sequence Recall',
      firstWordRecall: (_) => 'First Word Recall',
      identifyRoot: (_) => 'Identify Root',
      reverseCloze: (_) => 'Reverse Cloze',
      translatePhrase: (_) => 'Translate Phrase',
      posTagging: (_) => 'POS Tagging',
      crossVerseConnection: (_) => 'Cross-Verse Connection',
      echoRecall: (_) => 'Echo Recall',
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Exercise Debugger'),
      ),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            NodeSelectorWidget(
              onSelect: _loadExercises,
              initialValue: _selectedNodeId,
            ),
            const SizedBox(height: 12),
            ExerciseTypeDropdown(
              types: _exerciseTypeOptions(),
              selected: _selectedType,
              onSelect: _selectExerciseType,
            ),
            const SizedBox(height: 24),
            if (_loading)
              const Center(child: CircularProgressIndicator())
            else if (_error != null)
              Card(
                color: Theme.of(context).colorScheme.errorContainer,
                child: Padding(
                  padding: const EdgeInsets.all(16),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        'Error',
                        style: TextStyle(
                          fontWeight: FontWeight.bold,
                          color: Theme.of(context).colorScheme.onErrorContainer,
                        ),
                      ),
                      const SizedBox(height: 8),
                      Text(
                        _error!,
                        style: TextStyle(
                          color: Theme.of(context).colorScheme.onErrorContainer,
                        ),
                      ),
                    ],
                  ),
                ),
              )
            else if (_generatedExercise != null) ...[
              Card(
                child: Padding(
                  padding: const EdgeInsets.all(16),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        'Generated Exercise',
                        style: Theme.of(context).textTheme.titleMedium,
                      ),
                      const SizedBox(height: 8),
                      Text('Node: $_selectedNodeId'),
                      const SizedBox(height: 4),
                      Text(
                        'Type: ${_exerciseTypeName(_generatedExercise!)}',
                        style: const TextStyle(fontWeight: FontWeight.bold),
                      ),
                      const SizedBox(height: 8),
                      Text(
                        _generatedExercise.toString(),
                        style: Theme.of(context).textTheme.bodySmall,
                        maxLines: 5,
                        overflow: TextOverflow.ellipsis,
                      ),
                    ],
                  ),
                ),
              ),
              const SizedBox(height: 16),
              ElevatedButton.icon(
                onPressed: _launchExercise,
                icon: const Icon(Icons.play_arrow),
                label: const Text('Launch Exercise'),
                style: ElevatedButton.styleFrom(
                  padding: const EdgeInsets.symmetric(vertical: 16),
                ),
              ),
            ] else
              const Center(
                child: Text(
                  'Enter a node ID above to generate an exercise',
                  style: TextStyle(color: Colors.grey),
                ),
              ),
          ],
        ),
      ),
    );
  }
}
