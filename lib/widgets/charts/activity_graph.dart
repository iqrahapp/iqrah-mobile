import 'package:fl_chart/fl_chart.dart';
import 'package:flutter/material.dart';
import 'package:iqrah/rust_bridge/api.dart';

class ActivityGraph extends StatelessWidget {
  final List<ActivityPointDto> activityHistory;

  const ActivityGraph({super.key, required this.activityHistory});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;

    if (activityHistory.isEmpty) return const SizedBox.shrink();

    return AspectRatio(
      aspectRatio: 2.0,
      child: LineChart(
        LineChartData(
          gridData: FlGridData(show: false),
          titlesData: FlTitlesData(show: false),
          borderData: FlBorderData(show: false),
          minX: 0,
          maxX: (activityHistory.length - 1).toDouble(),
          minY: 0,
          maxY: activityHistory.map((e) => e.count).reduce((a, b) => a > b ? a : b).toDouble() * 1.2,
          lineBarsData: [
            LineChartBarData(
              spots: activityHistory
                  .asMap()
                  .entries
                  .map((e) => FlSpot(e.key.toDouble(), e.value.count.toDouble()))
                  .toList(),
              isCurved: true,
              color: colorScheme.primary,
              barWidth: 3,
              isStrokeCapRound: true,
              dotData: FlDotData(show: false),
              belowBarData: BarAreaData(
                show: true,
                color: colorScheme.primary.withValues(alpha: 0.1),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
