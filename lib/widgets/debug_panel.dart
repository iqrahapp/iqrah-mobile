import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:intl/intl.dart';
import 'package:iqrah/providers/due_items_provider.dart';
import 'package:iqrah/rust_bridge/api.dart' as api;
import 'package:iqrah/rust_bridge/repository.dart';

class DebugPanel {
  static Future<void> show(BuildContext context) async {
    try {
      final stats = await api.getDebugStats(userId: "default_user");

      if (!context.mounted) return;

      showDialog(
        context: context,
        builder: (context) => _DebugPanelDialog(stats: stats),
      );
    } catch (e) {
      if (!context.mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('Debug panel failed: $e'),
          backgroundColor: Colors.red[700],
        ),
      );
    }
  }
}

class _DebugPanelDialog extends ConsumerStatefulWidget {
  final DebugStats stats;

  const _DebugPanelDialog({required this.stats});

  @override
  ConsumerState<_DebugPanelDialog> createState() => _DebugPanelDialogState();
}

class _DebugPanelDialogState extends ConsumerState<_DebugPanelDialog> {
  Key _previewKey = UniqueKey(); // Force rebuild of session preview

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Text('Debug Stats'),
      content: SizedBox(
        width: double.maxFinite,
        child: SingleChildScrollView(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisSize: MainAxisSize.min,
            children: [
              _buildSummarySection(widget.stats),
              const SizedBox(height: 16),
              _buildSessionPreviewSection(),
              const SizedBox(height: 16),
              _buildNextDueSection(widget.stats.nextDueItems),
            ],
          ),
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => _refreshPriorityScores(context),
          style: TextButton.styleFrom(foregroundColor: Colors.blue),
          child: const Text('Refresh Scores'),
        ),
        TextButton(
          onPressed: () => _reseedDatabase(context),
          style: TextButton.styleFrom(foregroundColor: Colors.orange),
          child: const Text('Re-seed DB'),
        ),
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Close'),
        ),
      ],
    );
  }

  static Widget _buildSummarySection(DebugStats stats) {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Colors.grey[900],
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: Colors.grey[700]!),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            'Session Overview',
            style: TextStyle(
              fontWeight: FontWeight.bold,
              color: Colors.amber[300],
            ),
          ),
          const SizedBox(height: 8),
          _buildStatRow(
            'Total Nodes',
            '${stats.totalNodesCount}',
            Colors.green[300],
          ),
          _buildStatRow(
            'Total Edges',
            '${stats.totalEdgesCount}',
            Colors.green[300],
          ),
          _buildStatRow('Due Today', '${stats.dueToday}', Colors.red[300]),
          _buildStatRow(
            'Total Reviewed',
            '${stats.totalReviewed}',
            Colors.blue[300],
          ),
          _buildStatRow(
            'Avg Energy',
            stats.avgEnergy.toStringAsFixed(3),
            _getEnergyColor(stats.avgEnergy),
          ),
        ],
      ),
    );
  }

  static Widget _buildNextDueSection(List<DueItem> items) {
    if (items.isEmpty) {
      return const Text(
        'No due items found',
        style: TextStyle(fontStyle: FontStyle.italic),
      );
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          'Next Due Items',
          style: TextStyle(
            fontWeight: FontWeight.bold,
            color: Colors.amber[300],
          ),
        ),
        const SizedBox(height: 8),
        ...items.map((item) => _buildDueItemCard(item)),
      ],
    );
  }

  static Widget _buildDueItemCard(DueItem item) {
    final dueTime = DateTime.fromMillisecondsSinceEpoch(
      item.state.dueAt.toInt(),
    );
    final now = DateTime.now();
    final isOverdue = dueTime.isBefore(now);
    final timeStr = DateFormat('MMM dd HH:mm').format(dueTime);

    return Container(
      margin: const EdgeInsets.only(bottom: 8),
      padding: const EdgeInsets.all(8),
      decoration: BoxDecoration(
        color: isOverdue ? Colors.red[900]!.withOpacity(0.3) : Colors.grey[850],
        borderRadius: BorderRadius.circular(6),
        border: Border.all(
          color: isOverdue ? Colors.red[600]! : Colors.grey[700]!,
          width: 1,
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(
                item.nodeId.toString(),
                style: const TextStyle(
                  fontFamily: 'monospace',
                  fontSize: 12,
                  fontWeight: FontWeight.w500,
                ),
              ),
              Text(
                timeStr,
                style: TextStyle(
                  fontSize: 11,
                  color: isOverdue ? Colors.red[300] : Colors.grey[400],
                ),
              ),
            ],
          ),
          const SizedBox(height: 4),
          Row(
            children: [
              _buildMiniStat(
                'S',
                item.state.stability.toStringAsFixed(1),
                Colors.green[300],
              ),
              const SizedBox(width: 12),
              _buildMiniStat(
                'D',
                item.state.difficulty.toStringAsFixed(1),
                Colors.orange[300],
              ),
              const SizedBox(width: 12),
              _buildMiniStat(
                'E',
                item.state.energy.toStringAsFixed(2),
                _getEnergyColor(item.state.energy),
              ),
              const SizedBox(width: 12),
              _buildMiniStat(
                'R',
                '${item.state.reviewCount}',
                Colors.blue[300],
              ),
            ],
          ),
        ],
      ),
    );
  }

  static Widget _buildStatRow(String label, String value, Color? valueColor) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 2),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label, style: const TextStyle(fontSize: 13)),
          Text(
            value,
            style: TextStyle(
              fontSize: 13,
              fontWeight: FontWeight.w600,
              color: valueColor,
            ),
          ),
        ],
      ),
    );
  }

  static Widget _buildMiniStat(String label, String value, Color? color) {
    return Column(
      children: [
        Text(
          label,
          style: TextStyle(
            fontSize: 9,
            color: Colors.grey[500],
            fontWeight: FontWeight.w500,
          ),
        ),
        Text(
          value,
          style: TextStyle(
            fontSize: 11,
            color: color,
            fontWeight: FontWeight.w600,
          ),
        ),
      ],
    );
  }

  Widget _buildSessionPreviewSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          'Session Preview',
          style: TextStyle(
            fontWeight: FontWeight.bold,
            color: Colors.amber[300],
          ),
        ),
        const SizedBox(height: 8),
        FutureBuilder<List<ItemPreview>>(
          key: _previewKey, // Force rebuild on re-fetch
          future: api.getSessionPreview(
            userId: "default_user",
            limit: 5,
            surahFilter: ref.watch(surahFilterProvider),
          ),
          builder: (context, snapshot) {
            if (snapshot.connectionState == ConnectionState.waiting) {
              return const Center(child: CircularProgressIndicator());
            }

            if (snapshot.hasError) {
              return Text(
                'Error: ${snapshot.error}',
                style: TextStyle(color: Colors.red[300]),
              );
            }

            final previews = snapshot.data ?? [];
            if (previews.isEmpty) {
              return const Text('No items in preview');
            }

            return Column(
              children: previews
                  .map((item) => _buildPreviewCard(item))
                  .toList(),
            );
          },
        ),
      ],
    );
  }

  Future<void> _refreshPriorityScores(BuildContext context) async {
    try {
      showDialog(
        context: context,
        barrierDismissible: false,
        builder: (context) => const AlertDialog(
          content: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              CircularProgressIndicator(),
              SizedBox(width: 16),
              Text('Refreshing priority scores...'),
            ],
          ),
        ),
      );

      await api.refreshPriorityScores(userId: "default_user");

      if (!mounted) return;

      Navigator.of(context).pop(); // Close loading dialog

      // Force refresh of the preview section
      setState(() {
        _previewKey = UniqueKey();
      });

      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: const Text('Priority scores refreshed successfully'),
          backgroundColor: Colors.green[700],
          duration: const Duration(seconds: 2),
        ),
      );
    } catch (e) {
      if (!mounted) return;

      Navigator.of(context).pop(); // Close loading dialog
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('Refresh failed: $e'),
          backgroundColor: Colors.red[700],
        ),
      );
    }
  }

  Widget _buildPreviewCard(ItemPreview item) {
    final breakdown = item.scoreBreakdown;
    final memoryState = item.memoryState;
    final struggleLevel = _getStruggleLevel(memoryState);

    return Container(
      margin: const EdgeInsets.only(bottom: 8),
      padding: const EdgeInsets.all(8),
      decoration: BoxDecoration(
        color: Colors.grey[850],
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: Colors.grey[700]!),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Header with node ID and scores
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Expanded(
                child: Text(
                  item.nodeId,
                  style: const TextStyle(
                    fontFamily: 'monospace',
                    fontSize: 11,
                    fontWeight: FontWeight.w500,
                  ),
                ),
              ),
              Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  // FSRS Score (based on stability)
                  Container(
                    padding: const EdgeInsets.symmetric(
                      horizontal: 4,
                      vertical: 2,
                    ),
                    margin: const EdgeInsets.only(right: 4),
                    decoration: BoxDecoration(
                      color: _getFsrsScoreColor(memoryState.stability),
                      borderRadius: BorderRadius.circular(3),
                    ),
                    child: Text(
                      'FSRS ${_getFsrsScore(memoryState.stability)}',
                      style: const TextStyle(
                        fontSize: 9,
                        fontWeight: FontWeight.bold,
                        color: Colors.black,
                      ),
                    ),
                  ),
                  // Priority Score
                  Container(
                    padding: const EdgeInsets.symmetric(
                      horizontal: 6,
                      vertical: 2,
                    ),
                    decoration: BoxDecoration(
                      color: Colors.amber[700],
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: Text(
                      item.priorityScore.toStringAsFixed(1),
                      style: const TextStyle(
                        fontSize: 11,
                        fontWeight: FontWeight.bold,
                        color: Colors.black,
                      ),
                    ),
                  ),
                ],
              ),
            ],
          ),

          // Content preview
          if (item.arabic != null) ...[
            const SizedBox(height: 4),
            Text(
              item.arabic!,
              style: TextStyle(fontSize: 12, color: Colors.grey[300]),
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
            ),
          ],

          // FSRS Parameters
          const SizedBox(height: 6),
          Row(
            children: [
              _buildFsrsParameter(
                'Stability',
                memoryState.stability,
                Colors.blue[300],
              ),
              const SizedBox(width: 8),
              _buildFsrsParameter(
                'Difficulty',
                memoryState.difficulty,
                Colors.purple[300],
              ),
              const SizedBox(width: 8),
              _buildStruggleIndicator(struggleLevel),
            ],
          ),

          // Score breakdown
          const SizedBox(height: 6),
          Row(
            children: [
              _buildScoreComponent(
                'Due',
                breakdown.daysOverdue,
                breakdown.weights.wDue,
                Colors.red[300],
              ),
              const SizedBox(width: 8),
              _buildScoreComponent(
                'Need',
                breakdown.masteryGap,
                breakdown.weights.wNeed,
                Colors.orange[300],
              ),
              const SizedBox(width: 8),
              _buildScoreComponent(
                'Yield',
                breakdown.importance,
                breakdown.weights.wYield,
                Colors.green[300],
              ),
            ],
          ),
        ],
      ),
    );
  }

  static Widget _buildScoreComponent(
    String label,
    double value,
    double weight,
    Color? color,
  ) {
    final contribution = value * weight;

    return Expanded(
      child: Column(
        children: [
          Text(
            label,
            style: TextStyle(
              fontSize: 9,
              color: Colors.grey[500],
              fontWeight: FontWeight.w500,
            ),
          ),
          Text(
            '${value.toStringAsFixed(2)}×${weight.toStringAsFixed(1)}',
            style: TextStyle(fontSize: 10, color: color),
          ),
          Text(
            '=${contribution.toStringAsFixed(2)}',
            style: TextStyle(
              fontSize: 9,
              color: Colors.grey[400],
              fontWeight: FontWeight.w600,
            ),
          ),
        ],
      ),
    );
  }

  static Widget _buildFsrsParameter(String label, double value, Color? color) {
    return Expanded(
      child: Column(
        children: [
          Text(
            label,
            style: TextStyle(
              fontSize: 9,
              color: Colors.grey[500],
              fontWeight: FontWeight.w500,
            ),
          ),
          Text(
            value.toStringAsFixed(1),
            style: TextStyle(
              fontSize: 10,
              color: color,
              fontWeight: FontWeight.w600,
            ),
          ),
        ],
      ),
    );
  }

  static Widget _buildStruggleIndicator(String struggleLevel) {
    final (label, color) = _getStruggleLevelDisplay(struggleLevel);

    return Expanded(
      child: Column(
        children: [
          Text(
            'Struggle',
            style: TextStyle(
              fontSize: 9,
              color: Colors.grey[500],
              fontWeight: FontWeight.w500,
            ),
          ),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 1),
            decoration: BoxDecoration(
              color: color,
              borderRadius: BorderRadius.circular(3),
            ),
            child: Text(
              label,
              style: const TextStyle(
                fontSize: 9,
                fontWeight: FontWeight.bold,
                color: Colors.black,
              ),
            ),
          ),
        ],
      ),
    );
  }

  static String _getStruggleLevel(memoryState) {
    // Based on stability (higher = better retention) and difficulty (higher = harder to learn)
    final stability = memoryState.stability;
    final difficulty = memoryState.difficulty;
    final energy = memoryState.energy;

    // Create a composite struggle score (lower = less struggle)
    final struggleScore =
        difficulty * 10 + (1.0 - stability) * 5 + (1.0 - energy) * 3;

    if (struggleScore < 5) return 'Mastered';
    if (struggleScore < 10) return 'Good';
    if (struggleScore < 15) return 'Fair';
    if (struggleScore < 20) return 'Weak';
    if (struggleScore < 25) return 'Poor';
    return 'Struggle';
  }

  static (String, Color) _getStruggleLevelDisplay(String level) {
    switch (level) {
      case 'Mastered':
        return ('Mastered', Colors.green[400]!);
      case 'Good':
        return ('Good', Colors.lightGreen[400]!);
      case 'Fair':
        return ('Fair', Colors.yellow[400]!);
      case 'Weak':
        return ('Weak', Colors.orange[400]!);
      case 'Poor':
        return ('Poor', Colors.red[400]!);
      case 'Struggle':
        return ('Struggle', Colors.red[600]!);
      default:
        return ('Unknown', Colors.grey[400]!);
    }
  }

  static int _getFsrsScore(double stability) {
    // Convert stability to a 0-100 score
    // Stability typically ranges from 0 to ~50+ for well-learned items
    return (stability * 2).clamp(0, 100).round();
  }

  static Color _getFsrsScoreColor(double stability) {
    final score = _getFsrsScore(stability);
    if (score >= 80) return Colors.green[400]!;
    if (score >= 60) return Colors.lightGreen[400]!;
    if (score >= 40) return Colors.yellow[400]!;
    if (score >= 20) return Colors.orange[400]!;
    return Colors.red[400]!;
  }

  static Color _getEnergyColor(double energy) {
    if (energy >= 0.8) return Colors.green[300]!;
    if (energy >= 0.5) return Colors.yellow[300]!;
    if (energy >= 0.2) return Colors.orange[300]!;
    return Colors.red[300]!;
  }

  static Future<void> _reseedDatabase(BuildContext context) async {
    try {
      showDialog(
        context: context,
        barrierDismissible: false,
        builder: (context) => const AlertDialog(
          content: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              CircularProgressIndicator(),
              SizedBox(width: 16),
              Text('Re-seeding database...'),
            ],
          ),
        ),
      );

      await api.reseedDatabase();

      if (!context.mounted) return;

      Navigator.of(context).pop(); // Close loading
      Navigator.of(context).pop(); // Close debug dialog

      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: const Text('Database re-seeded successfully'),
          backgroundColor: Colors.green[700],
          duration: const Duration(seconds: 2),
        ),
      );
    } catch (e) {
      if (!context.mounted) return;

      Navigator.of(context).pop(); // Close loading
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('Re-seed failed: $e'),
          backgroundColor: Colors.red[700],
        ),
      );
    }
  }
}
