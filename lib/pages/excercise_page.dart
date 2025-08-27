import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/pages/summary_page.dart';
import 'package:iqrah/providers/session_provider.dart';
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
    // Listen to the current index to know when the session is over.
    ref.listen<SessionState>(sessionProvider, (prev, next) {
      final items = next.items;

      if ((!prev!.isCompleted() && next.isCompleted()) || items.isEmpty) {
        Navigator.of(context).pushReplacement(
          MaterialPageRoute(builder: (_) => const SummaryPage()),
        );
      } else {
        // Reset the card state for the next item
        setState(() {
          _isAnswerVisible = false;
        });
      }
    });

    final currentItem = ref.watch(sessionProvider.select((s) => s.currentItem));

    if (currentItem == null) {}
    return Scaffold(
      appBar: AppBar(title: const Text('Review Session')),
      body: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            if (currentItem == null)
              const Column(
                mainAxisAlignment: MainAxisAlignment.center,
                crossAxisAlignment: CrossAxisAlignment.center,
                children: [
                  CircularProgressIndicator(),
                  SizedBox(height: 20),
                  Text("Loading summary..."),
                ],
              )
            else ...[
              // The card
              Expanded(
                child: Card(
                  child: Center(
                    child: Text(
                      _isAnswerVisible
                          ? currentItem.arabic
                          : currentItem.translation,
                      textAlign: TextAlign.center,
                      style: const TextStyle(fontSize: 28),
                    ),
                  ),
                ),
              ),

              const SizedBox(height: 20),

              // The "Show Answer" button
              if (!_isAnswerVisible)
                ElevatedButton(
                  onPressed: () {
                    setState(() {
                      _isAnswerVisible = true;
                    });
                  },
                  child: const Text("Show Answer"),
                ),

              if (_isAnswerVisible) ...[
                _buildGradeButtons(ref, context, "Again", ReviewGrade.again),
                _buildGradeButtons(ref, context, "Hard", ReviewGrade.hard),
                _buildGradeButtons(ref, context, "Good", ReviewGrade.good),
                _buildGradeButtons(ref, context, "Easy", ReviewGrade.easy),
              ],
            ],
          ],
        ),
      ),
    );
  }
}

Widget _buildGradeButtons(
  WidgetRef ref,
  BuildContext context,
  String title,
  ReviewGrade grade,
) {
  return Padding(
    padding: const EdgeInsets.symmetric(vertical: 4.0),
    child: ElevatedButton(
      onPressed: () {
        ref.read(sessionProvider.notifier).submitReview(grade);
      },
      child: Text(title),
    ),
  );
}
