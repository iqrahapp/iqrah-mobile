import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:intl/intl.dart';
import 'package:iqrah/providers/due_items_provider.dart';
import 'package:iqrah/providers/propagation_log_provider.dart';
import 'package:iqrah/rust_bridge/api.dart' as api;
import 'package:iqrah/rust_bridge/api/types.dart' as rust_types;
import 'package:iqrah/rust_bridge/repository.dart';

class _AggregateData {
  final String targetNodeText;
  double totalEnergyChange;
  int eventCount;
  int lastTimestamp;

  _AggregateData(
    this.targetNodeText,
    this.totalEnergyChange,
    this.eventCount,
    this.lastTimestamp,
  );
}

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
              const PropagationLogView(),
              const SizedBox(height: 16),
              const PropagationLeaderboardView(),
              const SizedBox(height: 16),
              _buildNextDueSection(widget.stats.nextDueItems),
            ],
          ),
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => _showSessionSummary(context),
          style: TextButton.styleFrom(foregroundColor: Colors.green),
          child: const Text('Session Summary'),
        ),
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

  static Future<void> _showSessionSummary(BuildContext context) async {
    try {
      showDialog(
        context: context,
        builder: (context) => const SessionSummaryDialog(),
      );
    } catch (e) {
      if (!context.mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('Session summary failed: $e'),
          backgroundColor: Colors.red[700],
        ),
      );
    }
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

class PropagationLogView extends ConsumerStatefulWidget {
  const PropagationLogView({super.key});

  @override
  ConsumerState<PropagationLogView> createState() => _PropagationLogViewState();
}

class _PropagationLogViewState extends ConsumerState<PropagationLogView> {
  bool _sortByImpact = false;
  bool _showAggregated = false;

  @override
  Widget build(BuildContext context) {
    final asyncLog = ref.watch(propagationLogProvider);
    final notifier = ref.watch(propagationLogProvider.notifier);
    final entries = asyncLog.value ?? <rust_types.PropagationDetailSummary>[];

    final displayEntries = _showAggregated
        ? _aggregateEntries(entries, _sortByImpact)
        : _sortByImpact
        ? (List.of(entries)..sort(
            (a, b) => b.energyChange.abs().compareTo(a.energyChange.abs()),
          ))
        : entries;
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
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(
                'Propagation Log',
                style: TextStyle(
                  fontWeight: FontWeight.bold,
                  color: Colors.amber[300],
                ),
              ),
              Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  TextButton.icon(
                    onPressed: () =>
                        setState(() => _showAggregated = !_showAggregated),
                    icon: Icon(
                      _showAggregated ? Icons.list : Icons.analytics,
                      color: Colors.purple[300],
                    ),
                    label: Text(
                      _showAggregated ? 'Timeline' : 'Aggregate',
                      style: TextStyle(color: Colors.purple[200]),
                    ),
                  ),
                  TextButton.icon(
                    onPressed: () =>
                        setState(() => _sortByImpact = !_sortByImpact),
                    icon: Icon(
                      _sortByImpact ? Icons.filter_alt_off : Icons.bolt,
                      color: Colors.blue[300],
                    ),
                    label: Text(
                      _sortByImpact ? 'Timestamp' : 'Top impact',
                      style: TextStyle(color: Colors.blue[200]),
                    ),
                  ),
                  IconButton(
                    icon: const Icon(Icons.refresh),
                    tooltip: 'Refresh',
                    color: Colors.teal[200],
                    onPressed: () {
                      unawaited(notifier.refreshLog());
                    },
                  ),
                ],
              ),
            ],
          ),
          const SizedBox(height: 8),
          _buildWindowControls(notifier),
          const SizedBox(height: 12),
          SizedBox(
            height: 240,
            child: asyncLog.when(
              loading: () => const Center(child: CircularProgressIndicator()),
              error: (err, _) =>
                  Text('Error: $err', style: TextStyle(color: Colors.red[300])),
              data: (_) {
                if (displayEntries.isEmpty) {
                  return const Center(
                    child: Text(
                      'No propagation activity recorded yet.',
                      style: TextStyle(fontStyle: FontStyle.italic),
                    ),
                  );
                }

                return ListView.separated(
                  shrinkWrap: true,
                  physics: const BouncingScrollPhysics(),
                  itemCount: displayEntries.length,
                  separatorBuilder: (context, _) =>
                      Divider(color: Colors.grey[800], height: 12),
                  itemBuilder: (context, index) {
                    final item = displayEntries[index];
                    final timestamp = DateTime.fromMillisecondsSinceEpoch(
                      item.eventTimestamp * 1000,
                    );
                    final timestampLabel = DateFormat(
                      'MMM dd HH:mm:ss',
                    ).format(timestamp);
                    return ListTile(
                      contentPadding: EdgeInsets.zero,
                      dense: true,
                      title: Text(
                        _showAggregated
                            ? '${item.targetNodeText} (${_getEventCount(item)} events)'
                            : '${item.sourceNodeText} -> ${item.targetNodeText}',
                        style: const TextStyle(
                          fontSize: 13,
                          fontWeight: FontWeight.w600,
                        ),
                      ),
                      subtitle: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          if (!_showAggregated)
                            Text(
                              timestampLabel,
                              style: TextStyle(
                                fontSize: 11,
                                color: Colors.grey[400],
                              ),
                            ),
                          if (item.path != null &&
                              item.path!.isNotEmpty &&
                              !_showAggregated)
                            Text(
                              'Path: ${item.path}',
                              style: TextStyle(
                                fontSize: 11,
                                color: Colors.grey[500],
                              ),
                            ),
                          if (item.reason != null &&
                              item.reason!.isNotEmpty &&
                              !_showAggregated)
                            Text(
                              'Reason: ${item.reason}',
                              style: TextStyle(
                                fontSize: 11,
                                color: Colors.grey[500],
                              ),
                            ),
                        ],
                      ),
                      trailing: Text(
                        _showAggregated
                            ? 'Σ ${item.energyChange.toStringAsFixed(4)}'
                            : item.energyChange.toStringAsFixed(4),
                        style: TextStyle(
                          fontSize: 13,
                          fontWeight: FontWeight.bold,
                          color: _energyColor(item.energyChange),
                        ),
                      ),
                    );
                  },
                );
              },
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildWindowControls(PropagationLogNotifier notifier) {
    final window = notifier.window;
    final List<(PropagationLogWindow, String)> options = [
      (PropagationLogWindow.allTime, 'All time'),
      (PropagationLogWindow.last7Days, 'Last 7d'),
      (PropagationLogWindow.lastDay, 'Last 24h'),
      (PropagationLogWindow.lastHour, 'Last hour'),
      (PropagationLogWindow.last5Min, 'Last 5min'),
    ];

    return Wrap(
      spacing: 8,
      children: options
          .map(
            (option) => ChoiceChip(
              label: Text(option.$2),
              selected: window == option.$1,
              onSelected: (selected) {
                if (selected) {
                  unawaited(notifier.setWindow(option.$1));
                }
              },
            ),
          )
          .toList(),
    );
  }

  List<rust_types.PropagationDetailSummary> _aggregateEntries(
    List<rust_types.PropagationDetailSummary> entries,
    bool sortByImpact,
  ) {
    final Map<String, _AggregateData> aggregated = {};

    for (final entry in entries) {
      final key = entry.targetNodeText;
      if (aggregated.containsKey(key)) {
        aggregated[key]!.totalEnergyChange += entry.energyChange;
        aggregated[key]!.eventCount += 1;
        aggregated[key]!.lastTimestamp =
            aggregated[key]!.lastTimestamp > entry.eventTimestamp
            ? aggregated[key]!.lastTimestamp
            : entry.eventTimestamp;
      } else {
        aggregated[key] = _AggregateData(
          entry.targetNodeText,
          entry.energyChange,
          1,
          entry.eventTimestamp,
        );
      }
    }

    final result = aggregated.values
        .map(
          (data) => rust_types.PropagationDetailSummary(
            eventTimestamp: data.lastTimestamp,
            sourceNodeText: 'Multiple Sources',
            targetNodeText: data.targetNodeText,
            energyChange: data.totalEnergyChange,
            path: null,
            reason: '${data.eventCount} events',
          ),
        )
        .toList();

    if (sortByImpact) {
      result.sort(
        (a, b) => b.energyChange.abs().compareTo(a.energyChange.abs()),
      );
    } else {
      result.sort((a, b) => b.eventTimestamp.compareTo(a.eventTimestamp));
    }
    return result;
  }

  int _getEventCount(rust_types.PropagationDetailSummary item) {
    if (item.reason != null && item.reason!.contains('events')) {
      final match = RegExp(r'(\d+) events').firstMatch(item.reason!);
      if (match != null) {
        return int.tryParse(match.group(1)!) ?? 1;
      }
    }
    return 1;
  }

  Color _energyColor(double delta) {
    if (delta >= 0) {
      return Colors.green[300]!;
    }
    return Colors.red[300]!;
  }
}

class PropagationLeaderboardView extends ConsumerStatefulWidget {
  const PropagationLeaderboardView({super.key});

  @override
  ConsumerState<PropagationLeaderboardView> createState() =>
      _PropagationLeaderboardViewState();
}

class _PropagationLeaderboardViewState
    extends ConsumerState<PropagationLeaderboardView> {
  @override
  Widget build(BuildContext context) {
    final asyncLog = ref.watch(propagationLogProvider);
    final notifier = ref.watch(propagationLogProvider.notifier);
    final entries = asyncLog.value ?? <rust_types.PropagationDetailSummary>[];

    final positiveLeaderboard = _buildLeaderboard(entries, positiveOnly: true);
    final negativeLeaderboard = _buildLeaderboard(entries, positiveOnly: false);

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
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(
                'Propagation Sources Leaderboard',
                style: TextStyle(
                  fontWeight: FontWeight.bold,
                  color: Colors.amber[300],
                ),
              ),
              IconButton(
                icon: const Icon(Icons.refresh),
                tooltip: 'Refresh',
                color: Colors.teal[200],
                onPressed: () {
                  unawaited(notifier.refreshLog());
                },
              ),
            ],
          ),
          const SizedBox(height: 8),
          _buildWindowControls(notifier),
          const SizedBox(height: 12),
          SizedBox(
            height: 200,
            child: asyncLog.when(
              loading: () => const Center(child: CircularProgressIndicator()),
              error: (err, _) =>
                  Text('Error: $err', style: TextStyle(color: Colors.red[300])),
              data: (_) {
                if (positiveLeaderboard.isEmpty &&
                    negativeLeaderboard.isEmpty) {
                  return const Center(
                    child: Text(
                      'No propagation sources recorded yet.',
                      style: TextStyle(fontStyle: FontStyle.italic),
                    ),
                  );
                }

                return Row(
                  children: [
                    // Positive impacts column
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            '🟢 Best Sources (Positive)',
                            style: TextStyle(
                              fontWeight: FontWeight.bold,
                              color: Colors.green[300],
                              fontSize: 12,
                            ),
                          ),
                          const SizedBox(height: 8),
                          Expanded(
                            child: _buildLeaderboardColumn(
                              positiveLeaderboard,
                              true,
                            ),
                          ),
                        ],
                      ),
                    ),
                    const SizedBox(width: 16),
                    // Negative impacts column
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            '🔴 Worst Sources (Negative)',
                            style: TextStyle(
                              fontWeight: FontWeight.bold,
                              color: Colors.red[300],
                              fontSize: 12,
                            ),
                          ),
                          const SizedBox(height: 8),
                          Expanded(
                            child: _buildLeaderboardColumn(
                              negativeLeaderboard,
                              false,
                            ),
                          ),
                        ],
                      ),
                    ),
                  ],
                );
              },
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildWindowControls(PropagationLogNotifier notifier) {
    final window = notifier.window;
    final List<(PropagationLogWindow, String)> options = [
      (PropagationLogWindow.allTime, 'All time'),
      (PropagationLogWindow.last7Days, 'Last 7d'),
      (PropagationLogWindow.lastDay, 'Last 24h'),
      (PropagationLogWindow.lastHour, 'Last hour'),
      (PropagationLogWindow.last5Min, 'Last 5min'),
    ];

    return Wrap(
      spacing: 8,
      children: options
          .map(
            (option) => ChoiceChip(
              label: Text(option.$2),
              selected: window == option.$1,
              onSelected: (selected) {
                if (selected) {
                  unawaited(notifier.setWindow(option.$1));
                }
              },
            ),
          )
          .toList(),
    );
  }

  List<_LeaderboardEntry> _buildLeaderboard(
    List<rust_types.PropagationDetailSummary> entries, {
    required bool positiveOnly,
  }) {
    final Map<String, _LeaderboardEntry> aggregated = {};

    for (final entry in entries) {
      // Filter by positive/negative energy change
      if (positiveOnly && entry.energyChange < 0) continue;
      if (!positiveOnly && entry.energyChange >= 0) continue;

      final source = entry.sourceNodeText;
      if (aggregated.containsKey(source)) {
        aggregated[source]!.eventCount += 1;
        aggregated[source]!.totalEnergyOut += entry.energyChange.abs();
        if (entry.path != 'Self') {
          aggregated[source]!.totalPropagated += 1;
        }
      } else {
        aggregated[source] = _LeaderboardEntry(
          sourceNodeText: source,
          eventCount: 1,
          totalEnergyOut: entry.energyChange.abs(),
          totalPropagated: entry.path != 'Self' ? 1 : 0,
        );
      }
    }

    final result = aggregated.values.toList();
    result.sort((a, b) => b.totalEnergyOut.compareTo(a.totalEnergyOut));
    return result.take(5).toList(); // Limit to 5 per column
  }

  Widget _buildLeaderboardColumn(
    List<_LeaderboardEntry> leaderboard,
    bool isPositive,
  ) {
    if (leaderboard.isEmpty) {
      return Center(
        child: Text(
          'No ${isPositive ? 'positive' : 'negative'} impacts yet.',
          style: const TextStyle(fontStyle: FontStyle.italic, fontSize: 11),
        ),
      );
    }

    return ListView.separated(
      shrinkWrap: true,
      physics: const BouncingScrollPhysics(),
      itemCount: leaderboard.length,
      separatorBuilder: (context, _) =>
          Divider(color: Colors.grey[800], height: 8),
      itemBuilder: (context, index) {
        final item = leaderboard[index];
        final rank = index + 1;
        final rankIcon = _getRankIcon(rank);

        return Container(
          padding: const EdgeInsets.symmetric(vertical: 4, horizontal: 8),
          decoration: BoxDecoration(
            color: Colors.grey[850],
            borderRadius: BorderRadius.circular(4),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  Text(rankIcon, style: const TextStyle(fontSize: 14)),
                  const SizedBox(width: 4),
                  Text(
                    '#$rank',
                    style: TextStyle(
                      fontSize: 10,
                      fontWeight: FontWeight.bold,
                      color: Colors.grey[400],
                    ),
                  ),
                  const Spacer(),
                  Text(
                    'Σ ${item.totalEnergyOut.toStringAsFixed(2)}',
                    style: TextStyle(
                      fontSize: 11,
                      fontWeight: FontWeight.bold,
                      color: isPositive ? Colors.green[300] : Colors.red[300],
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 2),
              Text(
                item.sourceNodeText,
                style: const TextStyle(
                  fontSize: 11,
                  fontWeight: FontWeight.w600,
                ),
                maxLines: 2,
                overflow: TextOverflow.ellipsis,
              ),
              const SizedBox(height: 2),
              Text(
                '${item.eventCount} reviews → ${item.totalPropagated} propagations',
                style: TextStyle(fontSize: 9, color: Colors.grey[400]),
              ),
            ],
          ),
        );
      },
    );
  }

  String _getRankIcon(int rank) {
    switch (rank) {
      case 1:
        return '🥇';
      case 2:
        return '🥈';
      case 3:
        return '🥉';
      default:
        return '📍';
    }
  }
}

class _LeaderboardEntry {
  final String sourceNodeText;
  int eventCount;
  double totalEnergyOut;
  int totalPropagated;

  _LeaderboardEntry({
    required this.sourceNodeText,
    required this.eventCount,
    required this.totalEnergyOut,
    required this.totalPropagated,
  });
}

class SessionSummaryDialog extends ConsumerStatefulWidget {
  const SessionSummaryDialog({super.key});

  @override
  ConsumerState<SessionSummaryDialog> createState() =>
      _SessionSummaryDialogState();
}

class _SessionSummaryDialogState extends ConsumerState<SessionSummaryDialog> {
  @override
  Widget build(BuildContext context) {
    final asyncLog = ref.watch(propagationLogProvider);

    return AlertDialog(
      title: Row(
        children: [
          Icon(Icons.analytics, color: Colors.amber[300]),
          const SizedBox(width: 8),
          const Text('Session Impact Summary'),
        ],
      ),
      content: SizedBox(
        width: double.maxFinite,
        height: 400,
        child: asyncLog.when(
          loading: () => const Center(child: CircularProgressIndicator()),
          error: (err, _) => Text('Error: $err'),
          data: (entries) {
            final sessionEntries = _getSessionEntries(entries);
            final topSources = _getTopSources(sessionEntries);
            final topTargets = _getTopTargets(sessionEntries);

            return SingleChildScrollView(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  _buildSummaryStats(sessionEntries),
                  const SizedBox(height: 16),
                  _buildTopSourcesSection(topSources),
                  const SizedBox(height: 16),
                  _buildTopTargetsSection(topTargets),
                ],
              ),
            );
          },
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Close'),
        ),
      ],
    );
  }

  List<rust_types.PropagationDetailSummary> _getSessionEntries(
    List<rust_types.PropagationDetailSummary> allEntries,
  ) {
    final now = DateTime.now();
    final sessionStart = now.subtract(const Duration(hours: 1));

    return allEntries
        .where(
          (entry) => DateTime.fromMillisecondsSinceEpoch(
            entry.eventTimestamp * 1000,
          ).isAfter(sessionStart),
        )
        .toList();
  }

  Widget _buildSummaryStats(List<rust_types.PropagationDetailSummary> entries) {
    final totalEvents = entries.length;
    final totalEnergyChange = entries.fold<double>(
      0,
      (sum, entry) => sum + entry.energyChange.abs(),
    );
    final uniqueSources = entries.map((e) => e.sourceNodeText).toSet().length;
    final uniqueTargets = entries.map((e) => e.targetNodeText).toSet().length;

    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Colors.grey[850],
        borderRadius: BorderRadius.circular(8),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            'Session Statistics',
            style: TextStyle(
              fontWeight: FontWeight.bold,
              color: Colors.amber[300],
            ),
          ),
          const SizedBox(height: 8),
          Text('Total Propagation Events: $totalEvents'),
          Text('Total Energy Change: ${totalEnergyChange.toStringAsFixed(3)}'),
          Text('Unique Sources: $uniqueSources'),
          Text('Unique Targets: $uniqueTargets'),
        ],
      ),
    );
  }

  Widget _buildTopSourcesSection(List<_SessionSource> sources) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          'Top Energy Sources',
          style: TextStyle(
            fontWeight: FontWeight.bold,
            color: Colors.green[300],
          ),
        ),
        const SizedBox(height: 8),
        ...sources
            .take(5)
            .map(
              (source) => Padding(
                padding: const EdgeInsets.symmetric(vertical: 2),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    Expanded(
                      child: Text(
                        source.nodeText,
                        style: const TextStyle(fontSize: 13),
                      ),
                    ),
                    Text(
                      '+${source.totalEnergy.toStringAsFixed(3)}',
                      style: TextStyle(
                        fontWeight: FontWeight.bold,
                        color: Colors.green[300],
                      ),
                    ),
                  ],
                ),
              ),
            ),
      ],
    );
  }

  Widget _buildTopTargetsSection(List<_SessionTarget> targets) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          'Most Impacted Concepts',
          style: TextStyle(
            fontWeight: FontWeight.bold,
            color: Colors.blue[300],
          ),
        ),
        const SizedBox(height: 8),
        ...targets
            .take(5)
            .map(
              (target) => Padding(
                padding: const EdgeInsets.symmetric(vertical: 2),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    Expanded(
                      child: Text(
                        target.nodeText,
                        style: const TextStyle(fontSize: 13),
                      ),
                    ),
                    Text(
                      'Σ${target.totalImpact.toStringAsFixed(3)}',
                      style: TextStyle(
                        fontWeight: FontWeight.bold,
                        color: Colors.blue[300],
                      ),
                    ),
                  ],
                ),
              ),
            ),
      ],
    );
  }

  List<_SessionSource> _getTopSources(
    List<rust_types.PropagationDetailSummary> entries,
  ) {
    final Map<String, double> sourceEnergy = {};

    for (final entry in entries) {
      if (entry.path == 'Self') continue; // Skip direct reviews
      sourceEnergy[entry.sourceNodeText] =
          (sourceEnergy[entry.sourceNodeText] ?? 0) + entry.energyChange.abs();
    }

    final sources = sourceEnergy.entries
        .map((e) => _SessionSource(e.key, e.value))
        .toList();
    sources.sort((a, b) => b.totalEnergy.compareTo(a.totalEnergy));
    return sources;
  }

  List<_SessionTarget> _getTopTargets(
    List<rust_types.PropagationDetailSummary> entries,
  ) {
    final Map<String, double> targetImpact = {};

    for (final entry in entries) {
      targetImpact[entry.targetNodeText] =
          (targetImpact[entry.targetNodeText] ?? 0) + entry.energyChange.abs();
    }

    final targets = targetImpact.entries
        .map((e) => _SessionTarget(e.key, e.value))
        .toList();
    targets.sort((a, b) => b.totalImpact.compareTo(a.totalImpact));
    return targets;
  }
}

class _SessionSource {
  final String nodeText;
  final double totalEnergy;

  _SessionSource(this.nodeText, this.totalEnergy);
}

class _SessionTarget {
  final String nodeText;
  final double totalImpact;

  _SessionTarget(this.nodeText, this.totalImpact);
}
