import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/features/debug/debug_home_screen.dart';
import 'package:iqrah/pages/excercise_page.dart';
import 'package:iqrah/pages/sandbox_page.dart';
import 'package:iqrah/pages/settings_page.dart';
import 'package:iqrah/providers/due_items_provider.dart';
import 'package:iqrah/providers/session_provider.dart';
import 'package:iqrah/widgets/surah_dropdown.dart';

class DashboardPage extends ConsumerWidget {
  const DashboardPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final dueItemsAsync = ref.watch(exercisesProvider);
    final statsAsync = ref.watch(dashboardStatsProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Iqrah MVP'),
        actions: [
          IconButton(
            tooltip: 'Settings',
            icon: const Icon(Icons.settings),
            onPressed: () {
              Navigator.of(
                context,
              ).push(MaterialPageRoute(builder: (_) => const SettingsPage()));
            },
          ),
          IconButton(
            tooltip: 'Sandbox',
            icon: const Icon(Icons.science_outlined),
            onPressed: () {
              Navigator.of(
                context,
              ).push(MaterialPageRoute(builder: (_) => const SandboxPage()));
            },
          ),
          if (kDebugMode)
            IconButton(
              tooltip: 'Debug Tools',
              icon: const Icon(Icons.bug_report),
              onPressed: () {
                Navigator.of(context).push(
                  MaterialPageRoute(builder: (_) => const DebugHomeScreen()),
                );
              },
            ),
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: () => ref.invalidate(exercisesProvider),
          ),
        ],
      ),
      body: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          children: [
            // Stats widget
            statsAsync.when(
              data: (stats) {
                return Card(
                  margin: const EdgeInsets.only(bottom: 16),
                  child: Padding(
                    padding: const EdgeInsets.all(16.0),
                    child: Row(
                      mainAxisAlignment: MainAxisAlignment.spaceAround,
                      children: [
                        _buildStatColumn(
                          'Reviews Today',
                          '${stats.reviewsToday}',
                          Icons.check_circle,
                          Colors.green,
                        ),
                        _buildStatColumn(
                          'Streak',
                          '${stats.streakDays} day${stats.streakDays == 1 ? '' : 's'}',
                          Icons.local_fire_department,
                          Colors.orange,
                        ),
                      ],
                    ),
                  ),
                );
              },
              loading: () => const SizedBox.shrink(),
              error: (e, st) => const SizedBox.shrink(),
            ),
            // Sūrah filter dropdown
            Row(
              children: [
                const Text(
                  'Filter by Sūrah:',
                  style: TextStyle(fontSize: 16, fontWeight: FontWeight.w500),
                ),
                const SizedBox(width: 16),
                const SurahDropdown(),
              ],
            ),
            const SizedBox(height: 16),
            // High-Yield Mode toggle
            SwitchListTile(
              title: const Text('High-Yield Mode'),
              subtitle: const Text('Focus on widely-applicable concepts'),
              value: ref.watch(highYieldModeProvider),
              onChanged: (value) {
                ref.read(highYieldModeProvider.notifier).state = value;
                ref.invalidate(exercisesProvider);
              },
              secondary: const Icon(Icons.stars),
            ),
            const SizedBox(height: 20),
            Expanded(
              child: Center(
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
                            ref
                                .read(sessionProvider.notifier)
                                .startReview(items);
                            // 2. Navigate to the ExcercisePage
                            Navigator.of(context).push(
                              MaterialPageRoute(
                                builder: (_) => const ExcercisePage(),
                              ),
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
                  error: (e, st) => const Text(
                    'An error occurred while loading the session.',
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  static Widget _buildStatColumn(
    String label,
    String value,
    IconData icon,
    Color color,
  ) {
    return Column(
      children: [
        Icon(icon, size: 32, color: color),
        const SizedBox(height: 8),
        Text(
          value,
          style: const TextStyle(fontSize: 24, fontWeight: FontWeight.bold),
        ),
        Text(label, style: const TextStyle(fontSize: 12)),
      ],
    );
  }
}
