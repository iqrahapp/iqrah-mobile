import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/pages/dashboard_page.dart';
import 'package:iqrah/pages/excercise_page.dart';
import 'package:iqrah/providers/session_provider.dart';
import 'package:iqrah/rust_bridge/api.dart' as api;
import 'package:iqrah/rust_bridge/exercises.dart';
import 'package:iqrah/rust_bridge/repository.dart';

/// Checks for existing session and navigates appropriately on app startup
class AppInitializer extends ConsumerStatefulWidget {
  const AppInitializer({super.key});

  @override
  ConsumerState<AppInitializer> createState() => _AppInitializerState();
}

class _AppInitializerState extends ConsumerState<AppInitializer> {
  bool _isChecking = true;

  @override
  void initState() {
    super.initState();
    _checkForExistingSession();
  }

  Future<void> _checkForExistingSession() async {
    try {
      final existingNodes = await api.getExistingSession();

      if (existingNodes != null && existingNodes.isNotEmpty && mounted) {
        // Build exercises from the saved node data
        final exercises = _buildExercisesFromNodes(existingNodes);

        if (exercises.isNotEmpty) {
          // Resume the session
          ref.read(sessionProvider.notifier).startReview(exercises);

          // Navigate to exercise page
          Navigator.of(context).pushReplacement(
            MaterialPageRoute(builder: (_) => const ExcercisePage()),
          );
        }
      }
    } catch (e) {
      print('Error checking for existing session: $e');
    } finally {
      if (mounted) {
        setState(() {
          _isChecking = false;
        });
      }
    }
  }

  List<Exercise> _buildExercisesFromNodes(List<NodeData> nodes) {
    final exercises = <Exercise>[];

    for (final node in nodes) {
      final arabic = node.metadata['arabic'];
      final translation = node.metadata['translation'];

      if (arabic != null && translation != null) {
        // Simple recall exercise for now
        exercises.add(Exercise.recall(
          nodeId: node.id,
          arabic: arabic,
          translation: translation,
        ));
      }
    }

    return exercises;
  }

  @override
  Widget build(BuildContext context) {
    if (_isChecking) {
      return const Scaffold(
        body: Center(
          child: CircularProgressIndicator(),
        ),
      );
    }

    return const DashboardPage();
  }
}
