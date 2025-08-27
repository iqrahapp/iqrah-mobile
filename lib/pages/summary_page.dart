import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/providers/due_items_provider.dart';

class SummaryPage extends ConsumerWidget {
  const SummaryPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Done!'),
        automaticallyImplyLeading: false,
      ),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Text('Session complete!', style: TextStyle(fontSize: 24)),
            const SizedBox(height: 20),
            ElevatedButton(
              onPressed: () {
                // Invalidate provider before going back
                ref.invalidate(dueItemsProvider);

                // Go back to the dashboard page clearing the excersise/summary pages
                Navigator.of(context).popUntil((route) => route.isFirst);
              },
              child: const Text('Back to Dashboard'),
            ),
            const Text('You have completed the exercise.'),
          ],
        ),
      ),
    );
  }
}
