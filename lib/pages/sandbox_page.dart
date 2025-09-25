import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/pages/excercise_page.dart';
import 'package:iqrah/providers/session_provider.dart';
import 'package:iqrah/rust_bridge/api.dart' as api;
import 'package:iqrah/rust_bridge/exercises.dart';
import 'package:iqrah/rust_bridge/repository.dart';

class SandboxPage extends ConsumerStatefulWidget {
  const SandboxPage({super.key});

  @override
  ConsumerState<SandboxPage> createState() => _SandboxPageState();
}

class _SandboxPageState extends ConsumerState<SandboxPage> {
  final _controller = TextEditingController();
  Timer? _debounce;
  bool _loading = false;
  NodeData? _selectedNode;
  List<String> _suggestions = [];
  List<Exercise> _preview = [];

  @override
  void dispose() {
    _controller.dispose();
    _debounce?.cancel();
    super.dispose();
  }

  void _onChanged(String value) {
    _debounce?.cancel();
    _debounce = Timer(
      const Duration(milliseconds: 200),
      () => _updateSuggestions(value),
    );
    setState(() {
      _selectedNode = null;
      _preview = [];
    });
  }

  Future<void> _updateSuggestions(String value) async {
    final q = value.trim();
    if (q.isEmpty) {
      if (!mounted) return;
      setState(() => _suggestions = []);
      return;
    }
    if (!mounted) return;
    setState(() => _loading = true);
    try {
      // Use searchNodes to get up to 10 node IDs that start with the query
      final nodes = await api.searchNodes(query: q, limit: 10);
      if (mounted) {
        setState(() => _suggestions = nodes.map((n) => n.id).toList());
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

  Future<void> _search() async {
    final q = _controller.text.trim();
    if (q.isEmpty) {
      setState(() {
        _selectedNode = null;
        _preview = [];
      });
      return;
    }
    if (!mounted) return;
    setState(() => _loading = true);
    try {
      final nodeOpt = await api.fetchNodeWithMetadata(nodeId: q);
      if (mounted) {
        setState(() {
          _selectedNode = nodeOpt;
          _preview = [];
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() => _selectedNode = null);
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text('Search failed: $e')));
      }
    } finally {
      if (mounted) {
        setState(() => _loading = false);
      }
    }
  }

  Future<void> _loadExercisesFor(String nodeId) async {
    if (!mounted) return;
    setState(() => _loading = true);
    try {
      final items = await api.getExercisesForNode(nodeId: nodeId);
      if (mounted) {
        setState(() => _preview = items);
      }
    } catch (e) {
      if (mounted) {
        setState(() => _preview = []);
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text('Load exercises failed: $e')));
      }
    } finally {
      if (mounted) {
        setState(() => _loading = false);
      }
    }
  }

  void _startOne(Exercise ex) {
    HapticFeedback.lightImpact();
    ref.read(sessionProvider.notifier).startReview([ex]);
    Navigator.of(
      context,
    ).push(MaterialPageRoute(builder: (_) => const ExcercisePage()));
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Scaffold(
      appBar: AppBar(title: const Text('Sandbox')),
      body: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          children: [
            TextField(
              controller: _controller,
              decoration: InputDecoration(
                labelText: 'Node ID',
                suffixIcon: _loading
                    ? const Padding(
                        padding: EdgeInsets.all(12.0),
                        child: SizedBox(
                          width: 16,
                          height: 16,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        ),
                      )
                    : IconButton(
                        icon: const Icon(Icons.search),
                        onPressed: _search,
                      ),
              ),
              onChanged: _onChanged,
              onSubmitted: (_) => _search(),
            ),
            if (_suggestions.isNotEmpty) ...[
              const SizedBox(height: 8),
              ConstrainedBox(
                constraints: const BoxConstraints(maxHeight: 200),
                child: Material(
                  elevation: 2,
                  borderRadius: BorderRadius.circular(4),
                  child: ListView.builder(
                    shrinkWrap: true,
                    itemCount: _suggestions.length,
                    itemBuilder: (context, index) {
                      final suggestion = _suggestions[index];
                      return ListTile(
                        dense: true,
                        title: Text(suggestion),
                        onTap: () {
                          setState(() {
                            _controller.text = suggestion;
                            _suggestions = [];
                          });
                          _search();
                        },
                      );
                    },
                  ),
                ),
              ),
            ],
            const SizedBox(height: 12),
            if (_selectedNode != null)
              Card(
                child: ListTile(
                  title: Text(
                    _selectedNode!.id,
                    style: theme.textTheme.bodyMedium,
                  ),
                  subtitle: Text(_selectedNode!.metadata.toString()),
                  trailing: IconButton(
                    icon: const Icon(Icons.play_circle_outline),
                    onPressed: () => _loadExercisesFor(_selectedNode!.id),
                  ),
                ),
              ),
            if (_selectedNode == null)
              const Align(
                alignment: Alignment.centerLeft,
                child: Text('Select a node to preview exercises'),
              ),
            const SizedBox(height: 12),
            Expanded(child: _buildPreview(theme)),
          ],
        ),
      ),
    );
  }

  // _buildResults removed: now only single node lookup

  Widget _buildPreview(ThemeData theme) {
    if (_preview.isEmpty) {
      return const Center(child: Text('Select a node to preview exercises'));
    }
    return ListView.separated(
      itemCount: _preview.length,
      separatorBuilder: (_, index) => const Divider(height: 1),
      itemBuilder: (context, index) {
        final ex = _preview[index];
        return ListTile(
          dense: true,
          title: Text(_exerciseLabel(ex)),
          subtitle: Text(
            _exerciseSubtitle(ex),
            maxLines: 2,
            overflow: TextOverflow.ellipsis,
          ),
          trailing: ElevatedButton(
            onPressed: () => _startOne(ex),
            child: const Text('Try'),
          ),
        );
      },
    );
  }

  String _exerciseLabel(Exercise e) => e.map(
    recall: (_) => 'Recall',
    cloze: (_) => 'Cloze',
    mcqArToEn: (_) => 'MCQ Ar→En',
    mcqEnToAr: (_) => 'MCQ En→Ar',
  );

  String _exerciseSubtitle(Exercise e) => e.when(
    recall: (nodeId, arabic, translation) => arabic,
    cloze: (nodeId, question, answer) => question,
    mcqArToEn:
        (
          nodeId,
          arabic,
          verseArabic,
          surahNumber,
          ayahNumber,
          wordIndex,
          choicesEn,
          correctIndex,
        ) => 'Word: $arabic\n$verseArabic',
    mcqEnToAr:
        (
          nodeId,
          english,
          verseArabic,
          surahNumber,
          ayahNumber,
          wordIndex,
          choicesAr,
          correctIndex,
        ) => 'Word: $english\n$verseArabic',
  );
}
