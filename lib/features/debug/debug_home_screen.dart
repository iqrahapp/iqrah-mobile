import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:iqrah/features/debug/exercise_debug_screen.dart';
import 'package:iqrah/features/debug/energy_monitor_screen.dart';
import 'package:iqrah/features/debug/db_inspector_screen.dart';
import 'package:iqrah/rust_bridge/api.dart' as api;

/// Debug tools home screen - only visible in debug builds
class DebugHomeScreen extends StatelessWidget {
  const DebugHomeScreen({super.key});

  @override
  Widget build(BuildContext context) {
    assert(kDebugMode, 'Debug screen should only be accessible in debug mode');

    return Scaffold(
      appBar: AppBar(
        title: const Text('Debug Tools'),
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          _DebugCard(
            icon: Icons.play_circle_outline,
            title: 'Exercise Debugger',
            subtitle: 'Select a node and launch exercises',
            onTap: () => Navigator.push(
              context,
              MaterialPageRoute(builder: (_) => const ExerciseDebugScreen()),
            ),
          ),
          const SizedBox(height: 12),
          _DebugCard(
            icon: Icons.bolt,
            title: 'Energy Monitor',
            subtitle: 'View node energy and propagation',
            onTap: () => Navigator.push(
              context,
              MaterialPageRoute(builder: (_) => const EnergyMonitorScreen()),
            ),
          ),
          const SizedBox(height: 12),
          _DebugCard(
            icon: Icons.storage,
            title: 'Database Inspector',
            subtitle: 'Execute SQL queries (SELECT only)',
            onTap: () => Navigator.push(
              context,
              MaterialPageRoute(builder: (_) => const DbInspectorScreen()),
            ),
          ),
          const SizedBox(height: 12),
          FutureBuilder<api.DebugStatsDto>(
            future: api.getDebugStats(userId: 'default'),
            builder: (context, snapshot) {
              final stats = snapshot.data;
              return _DebugCard(
                icon: Icons.analytics,
                title: 'Debug Stats',
                subtitle: stats != null
                    ? 'Nodes: ${stats.totalNodesCount}, Due: ${stats.dueCount}'
                    : 'Loading...',
                onTap: null,
              );
            },
          ),
        ],
      ),
    );
  }
}

class _DebugCard extends StatelessWidget {
  final IconData icon;
  final String title;
  final String subtitle;
  final VoidCallback? onTap;

  const _DebugCard({
    required this.icon,
    required this.title,
    required this.subtitle,
    this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return Card(
      child: ListTile(
        leading: Icon(icon, size: 32),
        title: Text(title),
        subtitle: Text(subtitle),
        trailing: onTap != null ? const Icon(Icons.chevron_right) : null,
        onTap: onTap,
      ),
    );
  }
}
