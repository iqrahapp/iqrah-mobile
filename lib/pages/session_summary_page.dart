import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/providers/due_items_provider.dart';
import 'package:iqrah/rust_bridge/api.dart' as api;
import 'package:iqrah/utils/error_mapper.dart';
import 'package:iqrah/widgets/error_banner.dart';

class SessionSummaryPage extends ConsumerStatefulWidget {
  final int reviewCount;
  final api.SessionSummaryDto? summary;

  const SessionSummaryPage({
    super.key,
    required this.reviewCount,
    this.summary,
  });

  @override
  ConsumerState<SessionSummaryPage> createState() => _SessionSummaryPageState();
}

class _SessionSummaryPageState extends ConsumerState<SessionSummaryPage> {
  @override
  Widget build(BuildContext context) {
    final statsAsync = ref.watch(dashboardStatsProvider);
    final summary = widget.summary;
    final reviewCount =
        summary?.itemsCompleted ?? widget.reviewCount;

    return Scaffold(
      appBar: AppBar(
        title: const Text('Session Complete'),
        automaticallyImplyLeading: false,
      ),
      body: Center(
        child: Padding(
          padding: const EdgeInsets.all(24.0),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              const Icon(
                Icons.celebration,
                size: 100,
                color: Colors.amber,
              ),
              const SizedBox(height: 24),
              const Text(
                'Great Work!',
                style: TextStyle(
                  fontSize: 32,
                  fontWeight: FontWeight.bold,
                ),
              ),
              const SizedBox(height: 16),
              Text(
                'You reviewed $reviewCount item${reviewCount == 1 ? '' : 's'}',
                style: const TextStyle(fontSize: 20),
              ),
              if (summary != null) ...[
                const SizedBox(height: 24),
                Card(
                  child: Padding(
                    padding: const EdgeInsets.all(16.0),
                    child: Column(
                      children: [
                        _buildStatRow(
                          'Session Duration',
                          _formatDuration(summary.durationMs.toInt()),
                          Icons.schedule,
                        ),
                        const SizedBox(height: 12),
                        _buildStatRow(
                          'Items Completed',
                          '${summary.itemsCompleted}/${summary.itemsCount}',
                          Icons.check_circle,
                        ),
                        const Divider(height: 24),
                        _buildStatRow(
                          'Easy',
                          '${summary.easyCount}',
                          Icons.thumb_up,
                        ),
                        const SizedBox(height: 8),
                        _buildStatRow(
                          'Good',
                          '${summary.goodCount}',
                          Icons.sentiment_satisfied,
                        ),
                        const SizedBox(height: 8),
                        _buildStatRow(
                          'Hard',
                          '${summary.hardCount}',
                          Icons.sentiment_dissatisfied,
                        ),
                        const SizedBox(height: 8),
                        _buildStatRow(
                          'Again',
                          '${summary.againCount}',
                          Icons.refresh,
                        ),
                      ],
                    ),
                  ),
                ),
              ],
              const SizedBox(height: 32),
              statsAsync.when(
                data: (stats) {
                  return Card(
                    child: Padding(
                      padding: const EdgeInsets.all(16.0),
                      child: Column(
                        children: [
                          _buildStatRow(
                            'Reviews Today',
                            '${stats.reviewsToday}',
                            Icons.check_circle,
                          ),
                          const SizedBox(height: 12),
                          _buildStatRow(
                            'Current Streak',
                            '${stats.streakDays} day${stats.streakDays == 1 ? '' : 's'}',
                            Icons.local_fire_department,
                          ),
                        ],
                      ),
                    ),
                  );
                },
                loading: () => const CircularProgressIndicator(),
                error: (e, st) => ErrorBanner(
                  message: ErrorMapper.toMessage(
                    e,
                    context: 'Unable to load stats',
                  ),
                  dense: true,
                ),
              ),
              const SizedBox(height: 40),
              ElevatedButton(
                onPressed: () {
                  Navigator.of(context).pop();
                },
                style: ElevatedButton.styleFrom(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 48,
                    vertical: 16,
                  ),
                ),
                child: Semantics(
                  button: true,
                  label: 'Back to dashboard',
                  child: const Text(
                    'Back to Dashboard',
                    style: TextStyle(fontSize: 18),
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildStatRow(String label, String value, IconData icon) {
    return Row(
      mainAxisAlignment: MainAxisAlignment.spaceBetween,
      children: [
        Row(
          children: [
            Icon(icon, size: 24),
            const SizedBox(width: 8),
            Text(
              label,
              style: const TextStyle(fontSize: 16),
            ),
          ],
        ),
        Text(
          value,
          style: const TextStyle(
            fontSize: 20,
            fontWeight: FontWeight.bold,
          ),
        ),
      ],
    );
  }

  String _formatDuration(int durationMs) {
    if (durationMs <= 0) return '0s';
    final totalSeconds = (durationMs / 1000).round();
    final minutes = totalSeconds ~/ 60;
    final seconds = totalSeconds % 60;
    if (minutes == 0) return '${seconds}s';
    return '${minutes}m ${seconds.toString().padLeft(2, '0')}s';
  }
}
