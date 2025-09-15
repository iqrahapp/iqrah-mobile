import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import 'package:iqrah/rust_bridge/api.dart' as api;
import 'package:iqrah/rust_bridge/repository.dart';

class DebugPanel {
  static Future<void> show(BuildContext context) async {
    try {
      final stats = await api.getDebugStats(userId: "default_user");

      if (!context.mounted) return;

      showDialog(
        context: context,
        builder: (context) => AlertDialog(
          title: const Text('Debug Stats'),
          content: SizedBox(
            width: double.maxFinite,
            child: SingleChildScrollView(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                mainAxisSize: MainAxisSize.min,
                children: [
                  _buildSummarySection(stats),
                  const SizedBox(height: 16),
                  _buildNextDueSection(stats.nextDueItems),
                ],
              ),
            ),
          ),
          actions: [
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
        ),
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
    final dueTime = DateTime.fromMillisecondsSinceEpoch(item.state.dueAt);
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
