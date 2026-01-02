import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:iqrah/rust_bridge/api.dart' as api;
import 'package:iqrah/utils/app_logger.dart';
import 'package:iqrah/widgets/error_banner.dart';

/// Debug screen for executing SQL queries and viewing results.
class DbInspectorScreen extends StatefulWidget {
  const DbInspectorScreen({super.key});

  @override
  State<DbInspectorScreen> createState() => _DbInspectorScreenState();
}

class _DbInspectorScreenState extends State<DbInspectorScreen> {
  final TextEditingController _queryController = TextEditingController();
  api.DbQueryResultDto? _result;
  bool _loading = false;
  String? _error;

  static const _quickQueries = {
    'Tables': 'SELECT name FROM sqlite_master WHERE type=\'table\' ORDER BY name',
    'Nodes (10)': 'SELECT id, ukey, node_type FROM nodes LIMIT 10',
    'Edges (10)': 'SELECT source_id, target_id, edge_type FROM edges LIMIT 10',
    'User Memory (10)': 'SELECT user_id, content_key, energy FROM user_memory_states LIMIT 10',
    'Count Nodes': 'SELECT node_type, COUNT(*) as cnt FROM nodes GROUP BY node_type',
  };

  @override
  void dispose() {
    _queryController.dispose();
    super.dispose();
  }

  Future<void> _executeQuery() async {
    final sql = _queryController.text.trim();
    if (sql.isEmpty) return;

    setState(() {
      _loading = true;
      _error = null;
      _result = null;
    });

    AppLogger.database('Executing query: ${sql.substring(0, sql.length.clamp(0, 50))}...');

    try {
      final result = await api.executeDebugQuery(sql: sql);
      if (mounted) {
        setState(() {
          _result = result;
          _loading = false;
        });
        AppLogger.database(
            'Query returned ${result.rows.length} rows, ${result.columns.length} columns');
      }
    } catch (e) {
      AppLogger.database('Query failed', error: e);
      if (mounted) {
        setState(() {
          _error = e.toString();
          _loading = false;
        });
      }
    }
  }

  void _copyAsCsv() {
    if (_result == null) return;

    final buffer = StringBuffer();
    buffer.writeln(_result!.columns.join(','));
    for (final row in _result!.rows) {
      buffer.writeln(row.map((v) => '"${v.replaceAll('"', '""')}"').join(','));
    }

    Clipboard.setData(ClipboardData(text: buffer.toString()));
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('Copied to clipboard as CSV')),
    );
    AppLogger.database('Results copied to clipboard');
  }

  @override
  Widget build(BuildContext context) {
    if (!kDebugMode) {
      return Scaffold(
        appBar: AppBar(title: const Text('DB Inspector')),
        body: const Center(
          child: Text('DB Inspector is only available in debug builds'),
        ),
      );
    }

    return Scaffold(
      appBar: AppBar(
        title: const Text('DB Inspector'),
        actions: [
          if (_result != null)
            IconButton(
              icon: const Icon(Icons.copy),
              tooltip: 'Copy as CSV',
              onPressed: _copyAsCsv,
            ),
        ],
      ),
      body: Column(
        children: [
          Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                TextField(
                  controller: _queryController,
                  decoration: const InputDecoration(
                    labelText: 'SQL Query',
                    hintText: 'SELECT * FROM ...',
                    border: OutlineInputBorder(),
                  ),
                  maxLines: 3,
                  style: const TextStyle(fontFamily: 'monospace'),
                ),
                const SizedBox(height: 8),
                Wrap(
                  spacing: 8,
                  runSpacing: 8,
                  children: _quickQueries.entries.map((e) {
                    return ActionChip(
                      label: Text(e.key),
                      onPressed: () {
                        _queryController.text = e.value;
                      },
                    );
                  }).toList(),
                ),
                const SizedBox(height: 8),
                ElevatedButton.icon(
                  onPressed: _loading ? null : _executeQuery,
                  icon: _loading
                      ? const SizedBox(
                          width: 16,
                          height: 16,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Icon(Icons.play_arrow),
                  label: const Text('Execute'),
                ),
              ],
            ),
          ),
          if (_error != null)
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: ErrorBanner(
                message: _error!,
                dense: true,
              ),
            ),
          if (_result != null) ...[
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: Text(
                '${_result!.rows.length} rows',
                style: Theme.of(context).textTheme.bodySmall,
              ),
            ),
            const SizedBox(height: 8),
            Expanded(
              child: _buildResultsTable(),
            ),
          ],
        ],
      ),
    );
  }

  Widget _buildResultsTable() {
    final result = _result!;
    if (result.columns.isEmpty) {
      return const Center(
        child: Text('No columns returned'),
      );
    }

    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      child: SingleChildScrollView(
        child: DataTable(
          columnSpacing: 24,
          columns: result.columns
              .map((c) => DataColumn(
                    label: Text(
                      c,
                      style: const TextStyle(fontWeight: FontWeight.bold),
                    ),
                  ))
              .toList(),
          rows: result.rows.map((row) {
            return DataRow(
              cells: row
                  .map((v) => DataCell(
                        ConstrainedBox(
                          constraints: const BoxConstraints(maxWidth: 200),
                          child: Text(
                            v,
                            overflow: TextOverflow.ellipsis,
                            style: const TextStyle(fontFamily: 'monospace'),
                          ),
                        ),
                      ))
                  .toList(),
            );
          }).toList(),
        ),
      ),
    );
  }
}
