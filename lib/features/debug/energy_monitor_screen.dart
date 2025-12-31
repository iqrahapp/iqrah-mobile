import 'package:flutter/material.dart';
import 'package:iqrah/features/debug/node_selector_widget.dart';
import 'package:iqrah/rust_bridge/api.dart' as api;
import 'package:iqrah/utils/app_logger.dart';

/// Debug screen for monitoring energy values and simulating propagation.
class EnergyMonitorScreen extends StatefulWidget {
  const EnergyMonitorScreen({super.key});

  @override
  State<EnergyMonitorScreen> createState() => _EnergyMonitorScreenState();
}

class _EnergyMonitorScreenState extends State<EnergyMonitorScreen> {
  String? _selectedNodeId;
  api.EnergySnapshotDto? _snapshot;
  api.PropagationResultDto? _propagationResult;
  bool _loading = false;
  String? _error;
  double _energyDelta = 0.1;

  Future<void> _loadSnapshot(String nodeId) async {
    setState(() {
      _selectedNodeId = nodeId;
      _loading = true;
      _error = null;
      _snapshot = null;
      _propagationResult = null;
    });

    AppLogger.energy('Loading energy snapshot for: $nodeId');

    try {
      final snapshot = await api.getEnergySnapshot(
        userId: 'default',
        nodeId: nodeId,
      );
      if (mounted) {
        setState(() {
          _snapshot = snapshot;
          _loading = false;
        });
        AppLogger.energy(
            'Loaded: energy=${snapshot.energy}, neighbors=${snapshot.neighbors.length}');
      }
    } catch (e) {
      AppLogger.energy('Failed to load snapshot', error: e);
      if (mounted) {
        setState(() {
          _error = e.toString();
          _loading = false;
        });
      }
    }
  }

  Future<void> _simulatePropagation() async {
    if (_selectedNodeId == null) return;

    setState(() {
      _loading = true;
      _propagationResult = null;
    });

    AppLogger.energy(
        'Simulating propagation: delta=$_energyDelta for $_selectedNodeId');

    try {
      final result = await api.simulatePropagation(
        userId: 'default',
        nodeId: _selectedNodeId!,
        energyDelta: _energyDelta,
      );
      if (mounted) {
        setState(() {
          _propagationResult = result;
          _loading = false;
        });
        AppLogger.energy('Propagation simulated for ${result.after.length} nodes');
      }
    } catch (e) {
      AppLogger.energy('Propagation simulation failed', error: e);
      if (mounted) {
        setState(() {
          _error = e.toString();
          _loading = false;
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Energy Monitor'),
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            NodeSelectorWidget(
              onSelect: _loadSnapshot,
              initialValue: _selectedNodeId,
            ),
            const SizedBox(height: 16),
            if (_loading)
              const Center(
                child: Padding(
                  padding: EdgeInsets.all(32),
                  child: CircularProgressIndicator(),
                ),
              )
            else if (_error != null)
              Card(
                color: Theme.of(context).colorScheme.errorContainer,
                child: Padding(
                  padding: const EdgeInsets.all(16),
                  child: Text(
                    _error!,
                    style: TextStyle(
                      color: Theme.of(context).colorScheme.onErrorContainer,
                    ),
                  ),
                ),
              )
            else if (_snapshot != null) ...[
              _buildSnapshotCard(),
              const SizedBox(height: 16),
              _buildPropagationControls(),
              if (_propagationResult != null) ...[
                const SizedBox(height: 16),
                _buildPropagationResults(),
              ],
            ] else
              const Center(
                child: Padding(
                  padding: EdgeInsets.all(32),
                  child: Text(
                    'Select a node to view its energy',
                    style: TextStyle(color: Colors.grey),
                  ),
                ),
              ),
          ],
        ),
      ),
    );
  }

  Widget _buildSnapshotCard() {
    final snapshot = _snapshot!;
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Energy Snapshot',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 12),
            Row(
              children: [
                const Text('Node: '),
                Expanded(
                  child: Text(
                    snapshot.nodeId,
                    style: const TextStyle(fontWeight: FontWeight.bold),
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            Row(
              children: [
                if (snapshot.nodeType != null) ...[
                  Chip(
                    label: Text(snapshot.nodeType!),
                    padding: EdgeInsets.zero,
                    labelPadding: const EdgeInsets.symmetric(horizontal: 8),
                    materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
                    backgroundColor:
                        Theme.of(context).colorScheme.primaryContainer,
                  ),
                  const SizedBox(width: 8),
                ],
                if (snapshot.knowledgeAxis != null)
                  Chip(
                    label: Text(snapshot.knowledgeAxis!),
                    padding: EdgeInsets.zero,
                    labelPadding: const EdgeInsets.symmetric(horizontal: 8),
                    materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
                    backgroundColor:
                        Theme.of(context).colorScheme.secondaryContainer,
                  ),
              ],
            ),
            const SizedBox(height: 8),
            Row(
              children: [
                const Text('Energy: '),
                Text(
                  snapshot.energy.toStringAsFixed(4),
                  style: TextStyle(
                    fontWeight: FontWeight.bold,
                    color: _energyColor(snapshot.energy),
                  ),
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: LinearProgressIndicator(
                    value: snapshot.energy,
                    backgroundColor: Colors.grey.shade300,
                  ),
                ),
              ],
            ),
            if (snapshot.neighbors.isNotEmpty) ...[
              const SizedBox(height: 16),
              Text(
                'Neighbors (${snapshot.neighbors.length})',
                style: Theme.of(context).textTheme.titleSmall,
              ),
              const SizedBox(height: 8),
              ...snapshot.neighbors.map((n) => Padding(
                    padding: const EdgeInsets.symmetric(vertical: 4),
                    child: Row(
                      children: [
                        Expanded(
                          flex: 2,
                          child: Text(
                            n.nodeId,
                            style: Theme.of(context).textTheme.bodySmall,
                            overflow: TextOverflow.ellipsis,
                          ),
                        ),
                        SizedBox(
                          width: 60,
                          child: Text(
                            n.energy.toStringAsFixed(3),
                            style: TextStyle(color: _energyColor(n.energy)),
                          ),
                        ),
                        SizedBox(
                          width: 50,
                          child: Text(
                            'w=${n.edgeWeight.toStringAsFixed(2)}',
                            style: Theme.of(context).textTheme.bodySmall,
                          ),
                        ),
                      ],
                    ),
                  )),
            ] else
              const Padding(
                padding: EdgeInsets.only(top: 8),
                child: Text(
                  'No neighbors found',
                  style: TextStyle(color: Colors.grey),
                ),
              ),
          ],
        ),
      ),
    );
  }

  Widget _buildPropagationControls() {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Propagation Simulation',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 12),
            Row(
              children: [
                const Text('Energy Delta: '),
                Text(
                  _energyDelta.toStringAsFixed(2),
                  style: const TextStyle(fontWeight: FontWeight.bold),
                ),
              ],
            ),
            Slider(
              value: _energyDelta,
              min: -0.5,
              max: 0.5,
              divisions: 20,
              label: _energyDelta.toStringAsFixed(2),
              onChanged: (value) => setState(() => _energyDelta = value),
            ),
            const SizedBox(height: 8),
            ElevatedButton.icon(
              onPressed: _simulatePropagation,
              icon: const Icon(Icons.science),
              label: const Text('Simulate'),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildPropagationResults() {
    final result = _propagationResult!;
    final diag = result.diagnostics;
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Propagation Results',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 12),
            // Diagnostics info card
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: diag.nodeFound
                    ? (diag.totalEdges > 0
                        ? Colors.green.withValues(alpha: 0.1)
                        : Colors.orange.withValues(alpha: 0.1))
                    : Colors.red.withValues(alpha: 0.1),
                borderRadius: BorderRadius.circular(8),
                border: Border.all(
                  color: diag.nodeFound
                      ? (diag.totalEdges > 0 ? Colors.green : Colors.orange)
                      : Colors.red,
                  width: 1,
                ),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      Icon(
                        diag.nodeFound
                            ? (diag.totalEdges > 0
                                ? Icons.check_circle
                                : Icons.warning)
                            : Icons.error,
                        size: 18,
                        color: diag.nodeFound
                            ? (diag.totalEdges > 0 ? Colors.green : Colors.orange)
                            : Colors.red,
                      ),
                      const SizedBox(width: 8),
                      Text(
                        diag.nodeFound ? 'Node Found' : 'Node Not Found',
                        style: const TextStyle(fontWeight: FontWeight.bold),
                      ),
                      const Spacer(),
                      if (diag.nodeType != null)
                        Chip(
                          label: Text(diag.nodeType!),
                          padding: EdgeInsets.zero,
                          labelPadding:
                              const EdgeInsets.symmetric(horizontal: 8),
                          materialTapTargetSize:
                              MaterialTapTargetSize.shrinkWrap,
                        ),
                    ],
                  ),
                  const SizedBox(height: 8),
                  Text(
                    diag.message,
                    style: Theme.of(context).textTheme.bodySmall,
                  ),
                  const SizedBox(height: 4),
                  Text(
                    'Total edges: ${diag.totalEdges}',
                    style: Theme.of(context).textTheme.bodySmall?.copyWith(
                          color: Colors.grey,
                        ),
                  ),
                ],
              ),
            ),
            const SizedBox(height: 16),
            if (result.before.isEmpty)
              const Text(
                'No propagation targets to display',
                style: TextStyle(color: Colors.grey),
              )
            else
              Table(
                columnWidths: const {
                  0: FlexColumnWidth(2),
                  1: FixedColumnWidth(70),
                  2: FixedColumnWidth(70),
                  3: FixedColumnWidth(70),
                },
                children: [
                  const TableRow(
                    children: [
                      Text('Node',
                          style: TextStyle(fontWeight: FontWeight.bold)),
                      Text('Before',
                          style: TextStyle(fontWeight: FontWeight.bold)),
                      Text('After',
                          style: TextStyle(fontWeight: FontWeight.bold)),
                      Text('Delta',
                          style: TextStyle(fontWeight: FontWeight.bold)),
                    ],
                  ),
                  ...List.generate(result.before.length, (i) {
                    final before = result.before[i];
                    final after = result.after[i];
                    final delta = after.energy - before.energy;
                    return TableRow(
                      children: [
                        Padding(
                          padding: const EdgeInsets.symmetric(vertical: 4),
                          child: Text(
                            before.nodeId,
                            style: Theme.of(context).textTheme.bodySmall,
                            overflow: TextOverflow.ellipsis,
                          ),
                        ),
                        Text(before.energy.toStringAsFixed(3)),
                        Text(after.energy.toStringAsFixed(3)),
                        Text(
                          (delta >= 0 ? '+' : '') + delta.toStringAsFixed(3),
                          style: TextStyle(
                            color: delta >= 0 ? Colors.green : Colors.red,
                          ),
                        ),
                      ],
                    );
                  }),
                ],
              ),
          ],
        ),
      ),
    );
  }

  Color _energyColor(double energy) {
    if (energy < 0.3) return Colors.red;
    if (energy < 0.7) return Colors.orange;
    return Colors.green;
  }
}
