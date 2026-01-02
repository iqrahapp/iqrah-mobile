import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/features/session/session_screen.dart';
import 'package:iqrah/providers/session_provider.dart';
import 'package:iqrah/rust_bridge/api.dart' as api;
import 'package:iqrah/utils/error_mapper.dart';

class SandboxPage extends ConsumerStatefulWidget {
  const SandboxPage({super.key});

  @override
  ConsumerState<SandboxPage> createState() => _SandboxPageState();
}

class _SandboxPageState extends ConsumerState<SandboxPage> {
  final _controller = TextEditingController();
  Timer? _debounce;
  bool _loading = false;
  String? _selectedNodeId;
  List<String> _suggestions = [];
  List<api.ExerciseDataDto> _preview = [];

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
      _selectedNodeId = null;
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
        setState(() => _suggestions = nodes.map((n) => n.nodeId).toList());
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
        _selectedNodeId = null;
        _preview = [];
      });
      return;
    }

    // Just set the ID, we don't fetch metadata separately anymore
    setState(() {
      _selectedNodeId = q;
      _preview = [];
    });
  }

  Future<void> _loadExercisesFor(String nodeId) async {
    if (!mounted) return;
    setState(() => _loading = true);
    try {
      final item = await api.generateExerciseV2(nodeId: nodeId);
      if (mounted) {
        setState(() => _preview = [item]);
      }
    } catch (e) {
      if (mounted) {
        setState(() => _preview = []);
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(
          SnackBar(
            content: Text(
              ErrorMapper.toMessage(
                e,
                context: 'Unable to load exercises',
              ),
            ),
          ),
        );
      }
    } finally {
      if (mounted) {
        setState(() => _loading = false);
      }
    }
  }

  void _startOne(api.ExerciseDataDto ex) {
    HapticFeedback.lightImpact();
    ref.read(sessionProvider.notifier).startAdhocReview([ex]);
    Navigator.of(
      context,
    ).push(MaterialPageRoute(builder: (_) => const SessionScreen()));
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
            if (_selectedNodeId != null)
              Card(
                child: ListTile(
                  title: Text(
                    _selectedNodeId!,
                    style: theme.textTheme.bodyMedium,
                  ),
                  trailing: IconButton(
                    icon: const Icon(Icons.play_circle_outline),
                    onPressed: () => _loadExercisesFor(_selectedNodeId!),
                  ),
                ),
              ),
            if (_selectedNodeId == null)
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

  String _exerciseLabel(api.ExerciseDataDto e) {
    return _exerciseTypeLabel(e);
  }

  String _exerciseSubtitle(api.ExerciseDataDto e) {
    return _exerciseSubtitleFor(e);
  }

  String _exerciseTypeLabel(api.ExerciseDataDto e) {
    return e.map(
      memorization: (_) => 'Memorization',
      mcqArToEn: (_) => 'MCQ Ar to En',
      mcqEnToAr: (_) => 'MCQ En to Ar',
      translation: (_) => 'Translation',
      contextualTranslation: (_) => 'Contextual Translation',
      clozeDeletion: (_) => 'Cloze Deletion',
      firstLetterHint: (_) => 'First Letter Hint',
      missingWordMcq: (_) => 'Missing Word MCQ',
      nextWordMcq: (_) => 'Next Word MCQ',
      fullVerseInput: (_) => 'Full Verse Input',
      ayahChain: (_) => 'Ayah Chain',
      findMistake: (_) => 'Find the Mistake',
      ayahSequence: (_) => 'Ayah Sequence',
      sequenceRecall: (_) => 'Sequence Recall',
      firstWordRecall: (_) => 'First Word Recall',
      identifyRoot: (_) => 'Identify Root',
      reverseCloze: (_) => 'Reverse Cloze',
      translatePhrase: (_) => 'Translate Phrase',
      posTagging: (_) => 'POS Tagging',
      crossVerseConnection: (_) => 'Cross-Verse Connection',
      echoRecall: (_) => 'Echo Recall',
    );
  }

  String _exerciseSubtitleFor(api.ExerciseDataDto e) {
    return e.map(
      memorization: (e) => 'Node: ${e.nodeId}',
      mcqArToEn: (e) => 'Node: ${e.nodeId}',
      mcqEnToAr: (e) => 'Node: ${e.nodeId}',
      translation: (e) => 'Node: ${e.nodeId}',
      contextualTranslation: (e) => 'Node: ${e.nodeId}',
      clozeDeletion: (e) => 'Node: ${e.nodeId}',
      firstLetterHint: (e) => 'Node: ${e.nodeId}',
      missingWordMcq: (e) => 'Node: ${e.nodeId}',
      nextWordMcq: (e) => 'Node: ${e.nodeId}',
      fullVerseInput: (e) => 'Node: ${e.nodeId}',
      ayahChain: (e) => 'Verses: ${e.verseKeys.length}',
      findMistake: (e) => 'Node: ${e.nodeId}',
      ayahSequence: (e) => 'Sequence length: ${e.correctSequence.length}',
      sequenceRecall: (e) => 'Options: ${e.options.length}',
      firstWordRecall: (e) => 'Verse: ${e.verseKey}',
      identifyRoot: (e) => 'Node: ${e.nodeId}',
      reverseCloze: (e) => 'Node: ${e.nodeId}',
      translatePhrase: (e) => 'Node: ${e.nodeId}',
      posTagging: (e) => 'Node: ${e.nodeId}',
      crossVerseConnection: (e) => 'Related: ${e.relatedVerseIds.length}',
      echoRecall: (e) => 'Ayahs: ${e.ayahNodeIds.length}',
    );
  }
}
