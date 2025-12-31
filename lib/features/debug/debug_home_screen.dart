import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart' show rootBundle;
import 'package:path_provider/path_provider.dart';
import 'package:iqrah/features/debug/exercise_debug_screen.dart';
import 'package:iqrah/features/debug/energy_monitor_screen.dart';
import 'package:iqrah/features/debug/db_inspector_screen.dart';
import 'package:iqrah/features/debug/echo_recall_debug_screen.dart';
import 'package:iqrah/rust_bridge/api.dart' as api;

const contentDbAssetPath = "rust/content.db";

/// Debug tools home screen - only visible in debug builds
class DebugHomeScreen extends StatefulWidget {
  const DebugHomeScreen({super.key});

  @override
  State<DebugHomeScreen> createState() => _DebugHomeScreenState();
}

class _DebugHomeScreenState extends State<DebugHomeScreen> {
  api.DbHealthDto? _health;
  bool _isLoading = true;
  String? _error;

  @override
  void initState() {
    super.initState();
    _loadHealth();
  }

  Future<void> _loadHealth() async {
    setState(() {
      _isLoading = true;
      _error = null;
    });
    try {
      final health = await api.getDbHealth();
      setState(() {
        _health = health;
        _isLoading = false;
      });
    } catch (e) {
      setState(() {
        _error = e.toString();
        _isLoading = false;
      });
    }
  }

  Future<String> _getDbDir() async {
    final directory = await getApplicationDocumentsDirectory();
    return directory.path;
  }

  Future<void> _reloadContentDb() async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Reload Content DB'),
        content: const Text(
          'This will close the database, delete the local content.db, '
          'and recopy from bundled assets.\n\n'
          'The app will need to be restarted afterward.\n\n'
          'User progress will be preserved.',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx, false),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () => Navigator.pop(ctx, true),
            child: const Text('Reload', style: TextStyle(color: Colors.orange)),
          ),
        ],
      ),
    );

    if (confirmed != true) return;

    try {
      // Close database pools
      await api.closeDatabases();

      // Delete local content.db
      final dbDir = await _getDbDir();
      final contentDbFile = File('$dbDir/content.db');
      if (await contentDbFile.exists()) {
        await contentDbFile.delete();
        debugPrint('Deleted content.db');
      }

      // Copy from assets
      final assetData = await rootBundle.load(contentDbAssetPath);
      await contentDbFile.writeAsBytes(
        assetData.buffer.asUint8List(),
        flush: true,
      );
      debugPrint('Recopied content.db (${assetData.lengthInBytes} bytes)');

      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text('Content DB reloaded. Please restart the app.'),
            backgroundColor: Colors.green,
          ),
        );
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Error: $e'),
            backgroundColor: Colors.red,
          ),
        );
      }
    }
  }

  Future<void> _fullReset() async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Full Reset'),
        content: const Text(
          'This will DELETE ALL DATA including:\n\n'
          '- Content database\n'
          '- User progress and memory states\n\n'
          'This action cannot be undone!\n\n'
          'The app will need to be restarted afterward.',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx, false),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () => Navigator.pop(ctx, true),
            child: const Text('DELETE ALL', style: TextStyle(color: Colors.red)),
          ),
        ],
      ),
    );

    if (confirmed != true) return;

    try {
      // Close database pools
      await api.closeDatabases();

      // Delete both databases
      final dbDir = await _getDbDir();
      final contentDbFile = File('$dbDir/content.db');
      final userDbFile = File('$dbDir/user.db');

      if (await contentDbFile.exists()) {
        await contentDbFile.delete();
        debugPrint('Deleted content.db');
      }
      if (await userDbFile.exists()) {
        await userDbFile.delete();
        debugPrint('Deleted user.db');
      }

      // Copy content.db from assets
      final assetData = await rootBundle.load(contentDbAssetPath);
      await contentDbFile.writeAsBytes(
        assetData.buffer.asUint8List(),
        flush: true,
      );
      debugPrint('Recopied content.db (${assetData.lengthInBytes} bytes)');

      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text('Full reset complete. Please restart the app.'),
            backgroundColor: Colors.green,
          ),
        );
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Error: $e'),
            backgroundColor: Colors.red,
          ),
        );
      }
    }
  }

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
          _buildHealthCard(),
          const SizedBox(height: 16),
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
            icon: Icons.blur_on,
            title: 'Echo Recall',
            subtitle: 'Progressive blur memorization exercise',
            onTap: () => Navigator.push(
              context,
              MaterialPageRoute(builder: (_) => const EchoRecallDebugScreen()),
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

  Widget _buildHealthCard() {
    final theme = Theme.of(context);

    if (_isLoading) {
      return const Card(
        child: Padding(
          padding: EdgeInsets.all(16),
          child: Center(child: CircularProgressIndicator()),
        ),
      );
    }

    if (_error != null) {
      return Card(
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            children: [
              Row(
                children: [
                  const Icon(Icons.error, color: Colors.red),
                  const SizedBox(width: 8),
                  Expanded(child: Text('Error: $_error')),
                ],
              ),
              const SizedBox(height: 12),
              ElevatedButton.icon(
                onPressed: _loadHealth,
                icon: const Icon(Icons.refresh),
                label: const Text('Retry'),
              ),
            ],
          ),
        ),
      );
    }

    final health = _health!;
    final isHealthy = health.isHealthy;

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(
                  isHealthy ? Icons.check_circle : Icons.warning,
                  color: isHealthy ? Colors.green : Colors.orange,
                  size: 28,
                ),
                const SizedBox(width: 8),
                Text(
                  'Database Health',
                  style: theme.textTheme.titleMedium?.copyWith(
                    fontWeight: FontWeight.bold,
                  ),
                ),
                const Spacer(),
                Text(
                  isHealthy ? 'Healthy' : 'Issues Found',
                  style: TextStyle(
                    color: isHealthy ? Colors.green : Colors.orange,
                    fontWeight: FontWeight.w500,
                  ),
                ),
              ],
            ),
            const Divider(height: 24),
            _buildCountGrid(health),
            if (health.issues.isNotEmpty) ...[
              const SizedBox(height: 12),
              Container(
                padding: const EdgeInsets.all(8),
                decoration: BoxDecoration(
                  color: Colors.orange.withValues(alpha: 0.1),
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: health.issues
                      .map((issue) => Padding(
                            padding: const EdgeInsets.symmetric(vertical: 2),
                            child: Row(
                              children: [
                                const Icon(Icons.warning_amber,
                                    size: 16, color: Colors.orange),
                                const SizedBox(width: 8),
                                Expanded(child: Text(issue)),
                              ],
                            ),
                          ))
                      .toList(),
                ),
              ),
            ],
            const SizedBox(height: 16),
            Row(
              children: [
                Expanded(
                  child: OutlinedButton.icon(
                    onPressed: _reloadContentDb,
                    icon: const Icon(Icons.refresh),
                    label: const Text('Reload Content'),
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: OutlinedButton.icon(
                    onPressed: _fullReset,
                    style: OutlinedButton.styleFrom(
                      foregroundColor: Colors.red,
                      side: const BorderSide(color: Colors.red),
                    ),
                    icon: const Icon(Icons.delete_forever),
                    label: const Text('Full Reset'),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            Center(
              child: TextButton.icon(
                onPressed: _loadHealth,
                icon: const Icon(Icons.refresh, size: 16),
                label: const Text('Refresh'),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildCountGrid(api.DbHealthDto health) {
    return Wrap(
      spacing: 16,
      runSpacing: 8,
      children: [
        _buildCountChip('Chapters', health.chaptersCount.toInt(), expected: 114),
        _buildCountChip('Verses', health.versesCount.toInt(), expected: 6236),
        _buildCountChip('Words', health.wordsCount.toInt()),
        _buildCountChip('Nodes', health.nodesCount.toInt()),
        _buildCountChip('Edges', health.edgesCount.toInt()),
        _buildCountChip('User States', health.userMemoryCount.toInt()),
      ],
    );
  }

  Widget _buildCountChip(String label, int count, {int? expected}) {
    final hasIssue = count == 0 || (expected != null && count != expected);
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
      decoration: BoxDecoration(
        color: hasIssue
            ? Colors.orange.withValues(alpha: 0.1)
            : Colors.grey.withValues(alpha: 0.1),
        borderRadius: BorderRadius.circular(16),
        border: hasIssue ? Border.all(color: Colors.orange) : null,
      ),
      child: Text(
        '$label: ${_formatNumber(count)}',
        style: TextStyle(
          fontWeight: FontWeight.w500,
          color: hasIssue ? Colors.orange : null,
        ),
      ),
    );
  }

  String _formatNumber(int n) {
    if (n >= 1000000) return '${(n / 1000000).toStringAsFixed(1)}M';
    if (n >= 1000) return '${(n / 1000).toStringAsFixed(1)}K';
    return n.toString();
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
