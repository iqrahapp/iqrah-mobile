import 'package:fl_chart/fl_chart.dart';
import 'package:flutter/material.dart';
import 'package:iqrah/rust_bridge/api.dart';

class RootComprehensionChart extends StatelessWidget {
  final ComprehensionDto data;

  const RootComprehensionChart({super.key, required this.data});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;

    return AspectRatio(
      aspectRatio: 1.3,
      child: RadarChart(
        RadarChartData(
          dataSets: [
            RadarDataSet(
              fillColor: colorScheme.primary.withValues(alpha: 0.2),
              borderColor: colorScheme.primary,
              entryRadius: 3,
              dataEntries: [
                RadarEntry(value: data.memorization),
                RadarEntry(value: data.understanding),
                RadarEntry(value: data.context),
              ],
              borderWidth: 2,
            ),
          ],
          radarBackgroundColor: Colors.transparent,
          borderData: FlBorderData(show: false),
          radarBorderData: const BorderSide(color: Colors.transparent),
          titlePositionPercentageOffset: 0.2,
          titleTextStyle: theme.textTheme.labelSmall?.copyWith(
            color: colorScheme.onSurface.withValues(alpha: 0.7),
          ),
          getTitle: (index, angle) {
            switch (index) {
              case 0:
                return RadarChartTitle(text: 'Memorization', angle: angle);
              case 1:
                return RadarChartTitle(text: 'Understanding', angle: angle);
              case 2:
                return RadarChartTitle(text: 'Context', angle: angle);
              default:
                return const RadarChartTitle(text: '');
            }
          },
          tickCount: 1,
          ticksTextStyle: const TextStyle(color: Colors.transparent),
          tickBorderData: BorderSide(color: colorScheme.outline.withValues(alpha: 0.2)),
          gridBorderData: BorderSide(color: colorScheme.outline.withValues(alpha: 0.2), width: 1),
        ),
        swapAnimationDuration: const Duration(milliseconds: 400),
      ),
    );
  }
}
