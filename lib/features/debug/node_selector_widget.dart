import 'dart:async';
import 'package:flutter/material.dart';
import 'package:iqrah/rust_bridge/api.dart' as api;
import 'package:iqrah/services/node_id_service.dart';

/// Widget for selecting a node by searching or entering a node ID.
/// Supports debounced search and suggestions.
class NodeSelectorWidget extends StatefulWidget {
  final void Function(String nodeId) onSelect;
  final String? initialValue;

  const NodeSelectorWidget({
    super.key,
    required this.onSelect,
    this.initialValue,
  });

  @override
  State<NodeSelectorWidget> createState() => _NodeSelectorWidgetState();
}

class _NodeSelectorWidgetState extends State<NodeSelectorWidget> {
  final _controller = TextEditingController();
  final _minEnergyController = TextEditingController();
  final _maxEnergyController = TextEditingController();
  final _rangeController = TextEditingController();
  Timer? _debounce;
  bool _loading = false;
  List<api.NodeSearchDto> _suggestions = [];
  String? _nodeTypeFilter;

  static const _nodeTypeOptions = {
    'verse': 'Verse',
    'word': 'Word',
    'word_instance': 'Word Instance',
    'chapter': 'Chapter',
    'root': 'Root',
    'lemma': 'Lemma',
    'knowledge': 'Knowledge',
  };

  @override
  void initState() {
    super.initState();
    if (widget.initialValue != null) {
      _controller.text = widget.initialValue!;
    }
  }

  @override
  void dispose() {
    _controller.dispose();
    _minEnergyController.dispose();
    _maxEnergyController.dispose();
    _rangeController.dispose();
    _debounce?.cancel();
    super.dispose();
  }

  void _onChanged(String value) {
    _debounce?.cancel();
    _debounce = Timer(
      const Duration(milliseconds: 300),
      () => _updateSuggestions(value),
    );
  }

  Future<void> _updateSuggestions(String value) async {
    final q = value.trim();
    final hasFilters = _hasFilters();
    final isRange = _isRangeInput(q);
    if ((q.isEmpty || q.length < 2) && !hasFilters && !isRange) {
      if (mounted) setState(() => _suggestions = []);
      return;
    }

    if (mounted) setState(() => _loading = true);

    try {
      if (hasFilters || isRange) {
        final filter = api.NodeFilterDto(
          nodeType: _nodeTypeFilter,
          minEnergy: _parseDouble(_minEnergyController.text),
          maxEnergy: _parseDouble(_maxEnergyController.text),
          range: _rangeController.text.trim().isEmpty
              ? (isRange ? q : null)
              : _rangeController.text.trim(),
        );
        final nodes = await api.queryNodesFiltered(
          userId: 'default',
          filter: filter,
          limit: 20,
        );
        if (mounted) {
          setState(() => _suggestions = nodes);
        }
      } else {
        final nodes = await api.searchNodes(query: q, limit: 10);
        if (mounted) {
          setState(() => _suggestions = nodes);
        }
      }
    } catch (e) {
      if (mounted) {
        setState(() => _suggestions = []);
      }
    } finally {
      if (mounted) {
        setState(() => _loading = false);
      }
    }
  }

  void _selectNode(String nodeId) {
    _controller.text = nodeId;
    setState(() => _suggestions = []);
    widget.onSelect(nodeId);
  }

  void _submit() {
    final nodeId = _controller.text.trim();
    if (nodeId.isEmpty) return;

    if (_hasFilters() || _isRangeInput(nodeId) || !NodeIdService.isValid(nodeId)) {
      _updateSuggestions(nodeId);
      return;
    }

    setState(() => _suggestions = []);
    widget.onSelect(nodeId);
  }

  bool _isRangeInput(String value) {
    return value.contains('-') && value.contains(':');
  }

  bool _hasFilters() {
    return _nodeTypeFilter != null ||
        _rangeController.text.trim().isNotEmpty ||
        _parseDouble(_minEnergyController.text) != null ||
        _parseDouble(_maxEnergyController.text) != null;
  }

  double? _parseDouble(String value) {
    final trimmed = value.trim();
    if (trimmed.isEmpty) return null;
    return double.tryParse(trimmed);
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        TextField(
          controller: _controller,
          decoration: InputDecoration(
            labelText: 'Node ID',
            hintText: 'e.g., VERSE:1:1 or WORD:123',
            suffixIcon: _loading
                ? const Padding(
                    padding: EdgeInsets.all(12),
                    child: SizedBox(
                      width: 16,
                      height: 16,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    ),
                  )
                : IconButton(
                    icon: const Icon(Icons.search),
                    onPressed: _submit,
                  ),
            border: const OutlineInputBorder(),
          ),
          onChanged: _onChanged,
          onSubmitted: (_) => _submit(),
        ),
        const SizedBox(height: 8),
        ExpansionTile(
          tilePadding: EdgeInsets.zero,
          title: const Text('Filters (optional)'),
          children: [
            Row(
              children: [
                Expanded(
                  child: DropdownButtonFormField<String>(
                    key: ValueKey(_nodeTypeFilter),
                    initialValue: _nodeTypeFilter,
                    items: _nodeTypeOptions.entries
                        .map(
                          (e) => DropdownMenuItem<String>(
                            value: e.key,
                            child: Text(e.value),
                          ),
                        )
                        .toList(),
                    decoration: const InputDecoration(
                      labelText: 'Node type',
                      border: OutlineInputBorder(),
                    ),
                    onChanged: (value) {
                      setState(() => _nodeTypeFilter = value);
                    },
                  ),
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: TextField(
                    controller: _rangeController,
                    decoration: const InputDecoration(
                      labelText: 'Range',
                      hintText: '1:1-7',
                      border: OutlineInputBorder(),
                    ),
                    onSubmitted: (_) => _updateSuggestions(_controller.text),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            Row(
              children: [
                Expanded(
                  child: TextField(
                    controller: _minEnergyController,
                    decoration: const InputDecoration(
                      labelText: 'Min energy',
                      hintText: '0.2',
                      border: OutlineInputBorder(),
                    ),
                    keyboardType: const TextInputType.numberWithOptions(
                      decimal: true,
                      signed: false,
                    ),
                  ),
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: TextField(
                    controller: _maxEnergyController,
                    decoration: const InputDecoration(
                      labelText: 'Max energy',
                      hintText: '0.8',
                      border: OutlineInputBorder(),
                    ),
                    keyboardType: const TextInputType.numberWithOptions(
                      decimal: true,
                      signed: false,
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            Align(
              alignment: Alignment.centerRight,
              child: OutlinedButton.icon(
                onPressed: () => _updateSuggestions(_controller.text),
                icon: const Icon(Icons.filter_alt_outlined),
                label: const Text('Apply filters'),
              ),
            ),
          ],
        ),
        if (_suggestions.isNotEmpty) ...[
          const SizedBox(height: 4),
          ConstrainedBox(
            constraints: const BoxConstraints(maxHeight: 200),
            child: Card(
              margin: EdgeInsets.zero,
              child: ListView.builder(
                shrinkWrap: true,
                itemCount: _suggestions.length,
                itemBuilder: (context, index) {
                  final node = _suggestions[index];
                  return ListTile(
                    dense: true,
                    title: Text(node.nodeId),
                    subtitle: Text(
                      '${node.nodeType} - ${node.preview}',
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                    onTap: () => _selectNode(node.nodeId),
                  );
                },
              ),
            ),
          ),
        ],
      ],
    );
  }
}
