import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/providers/stats_provider.dart';
// ignore: depend_on_referenced_packages
import 'package:iqrah/widgets/charts/activity_graph.dart';
import 'package:iqrah/widgets/charts/root_comprehension_chart.dart';
import 'package:percent_indicator/percent_indicator.dart';

class StatsPage extends ConsumerWidget {
  const StatsPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final statsAsync = ref.watch(detailedStatsProvider);
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;

    return Scaffold(
      appBar: AppBar(
        title: const Text('Progress & Stats'),
        centerTitle: true,
      ),
      body: statsAsync.when(
        data: (stats) {
          return SingleChildScrollView(
            padding: const EdgeInsets.all(20),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                // Top Section: Mastery Circle and Summary
                Center(
                  child: CircularPercentIndicator(
                    radius: 80.0,
                    lineWidth: 12.0,
                    percent: stats.comprehension.memorization, // Use memorization as mastery proxy
                    center: Column(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        Text(
                          "${(stats.comprehension.memorization * 100).toInt()}%",
                          style: theme.textTheme.headlineMedium?.copyWith(
                            color: colorScheme.primary,
                            fontWeight: FontWeight.bold,
                          ),
                        ),
                        Text(
                          "Mastery",
                          style: theme.textTheme.labelSmall?.copyWith(
                            color: colorScheme.onSurface.withValues(alpha: 0.6),
                          ),
                        ),
                      ],
                    ),
                    progressColor: colorScheme.primary,
                    backgroundColor: colorScheme.surfaceContainerHighest,
                    circularStrokeCap: CircularStrokeCap.round,
                    animation: true,
                  ),
                ),
                const SizedBox(height: 32),
                
                // Radar Chart Section
                Text(
                  "Skill Breakdown",
                  style: theme.textTheme.titleMedium?.copyWith(fontWeight: FontWeight.bold),
                ),
                const SizedBox(height: 16),
                Container(
                  padding: const EdgeInsets.all(16),
                  decoration: BoxDecoration(
                    color: colorScheme.surfaceContainer,
                    borderRadius: BorderRadius.circular(16),
                  ),
                  child: RootComprehensionChart(data: stats.comprehension),
                ),
                
                const SizedBox(height: 32),

                // Activity Graph Section
                Text(
                  "Activity Trends",
                  style: theme.textTheme.titleMedium?.copyWith(fontWeight: FontWeight.bold),
                ),
                const SizedBox(height: 16),
                Container(
                  padding: const EdgeInsets.all(16),
                  decoration: BoxDecoration(
                    color: colorScheme.surfaceContainer,
                    borderRadius: BorderRadius.circular(16),
                  ),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Padding(
                        padding: const EdgeInsets.only(bottom: 12.0),
                        child: Text(
                          "Last 7 Days",
                          style: theme.textTheme.labelMedium?.copyWith(
                            color: colorScheme.onSurfaceVariant,
                          ),
                        ),
                      ),
                      ActivityGraph(activityHistory: stats.activityHistory),
                    ],
                  ),
                ),
              ],
            ),
          );
        },
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (err, stack) => Center(child: Text('Error loading stats: $err')),
      ),
    );
  }
}
