import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/pages/excercise_page.dart';
import 'package:iqrah/providers/due_items_provider.dart';
import 'package:iqrah/providers/session_provider.dart';
import 'package:iqrah/widgets/debug_panel.dart';

class DashboardPage extends ConsumerWidget {
  const DashboardPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final dueItemsAsync = ref.watch(dueItemsProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Iqrah MVP'),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: () => ref.invalidate(dueItemsProvider),
          ),
          GestureDetector(
            onLongPress: () => DebugPanel.show(context),
            child: const Padding(
              padding: EdgeInsets.all(16.0),
              child: Icon(Icons.info_outline),
            ),
          ),
        ],
      ),
      body: Center(
        child: dueItemsAsync.when(
          data: (items) {
            if (items.isEmpty) {
              return const Text(
                'No items are due for review. Please come back later!',
                style: TextStyle(fontSize: 20),
                textAlign: TextAlign.center,
              );
            }
            return Column(
              children: [
                Text(
                  'You have ${items.length} items due for review.',
                  style: const TextStyle(fontSize: 20),
                ),
                const SizedBox(height: 20),
                ElevatedButton(
                  style: ElevatedButton.styleFrom(
                    padding: const EdgeInsets.symmetric(
                      horizontal: 40,
                      vertical: 20,
                    ),
                    textStyle: const TextStyle(fontSize: 20),
                  ),
                  onPressed: () {
                    // 1. Tell the SessionNotifier to start a review with these items
                    ref.read(sessionProvider.notifier).startReview(items);
                    // 2. Navigate to the ExcercisePage
                    Navigator.of(context).push(
                      MaterialPageRoute(builder: (_) => const ExcercisePage()),
                    );
                  },
                  child: const Text('Start Review'),
                ),
              ],
            );
          },
          loading: () => const Column(
            children: [
              CircularProgressIndicator(),
              SizedBox(height: 20),
              Text("Loading due items..."),
            ],
          ),
          error: (e, st) =>
              const Text('An error occurred while loading the session.'),
        ),
      ),
    );
  }
}
