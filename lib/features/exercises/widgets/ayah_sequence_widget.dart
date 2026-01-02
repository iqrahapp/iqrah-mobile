import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:iqrah/services/content_service.dart';
import 'package:iqrah/services/node_id_service.dart';
import 'package:iqrah/widgets/error_banner.dart';

class AyahSequenceWidget extends StatefulWidget {
  final String nodeId;
  final List<String> correctSequence;
  final void Function(bool isCorrect) onComplete;
  final ContentService? contentService;

  const AyahSequenceWidget({
    super.key,
    required this.nodeId,
    required this.correctSequence,
    required this.onComplete,
    this.contentService,
  });

  @override
  State<AyahSequenceWidget> createState() => _AyahSequenceWidgetState();
}

class _AyahSequenceWidgetState extends State<AyahSequenceWidget> {
  late final ContentService _contentService;
  bool _loading = true;
  String? _error;
  List<String> _options = [];
  List<String> _selected = [];
  final Map<String, String> _optionText = {};

  @override
  void initState() {
    super.initState();
    _contentService = widget.contentService ?? ContentService();
    _loadContent();
  }

  Future<void> _loadContent() async {
    setState(() {
      _loading = true;
      _error = null;
    });

    try {
      _options = List<String>.from(widget.correctSequence);
      _options.shuffle();
      _selected = [];

      for (final nodeId in _options) {
        _optionText[nodeId] = await _fetchNodeText(nodeId);
      }

      if (mounted) {
        setState(() {
          _loading = false;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _error = e.toString();
          _loading = false;
        });
      }
    }
  }

  Future<String> _fetchNodeText(String nodeId) async {
    try {
      final nodeType = NodeIdService.getBaseNodeType(nodeId);
      if (nodeType == NodeType.word) {
        final wordId = NodeIdService.parseWordId(nodeId);
        final word = await _contentService.getWord(wordId);
        return word?.textUthmani ?? nodeId;
      }
      if (nodeType == NodeType.wordInstance) {
        final (chapter, verse, position) = NodeIdService.parseWordInstance(nodeId);
        final word = await _contentService.getWordAtPosition(
          chapter: chapter,
          verse: verse,
          position: position,
        );
        return word?.textUthmani ?? nodeId;
      }
      final verseKey = NodeIdService.parseVerseKey(nodeId);
      final verse = await _contentService.getVerse(verseKey);
      return verse?.textUthmani ?? verseKey;
    } catch (_) {
      return nodeId;
    }
  }

  void _selectOption(String nodeId) {
    if (_selected.contains(nodeId)) {
      return;
    }
    setState(() {
      _selected.add(nodeId);
    });
  }

  void _reset() {
    setState(() {
      _selected = [];
    });
  }

  void _submit() {
    final selected = _selected.map(NodeIdService.baseNodeId).toList();
    final correct = widget.correctSequence
        .map(NodeIdService.baseNodeId)
        .toList(growable: false);
    widget.onComplete(listEquals(selected, correct));
  }

  @override
  Widget build(BuildContext context) {
    if (_loading) {
      return const Center(child: CircularProgressIndicator());
    }

    if (_error != null) {
      return Center(
        child: ErrorBanner(
          message: _error!,
          onRetry: _loadContent,
        ),
      );
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Text(
          'Order the verses',
          style: Theme.of(context).textTheme.titleMedium,
        ),
        const SizedBox(height: 12),
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: _options.asMap().entries.map((entry) {
            final nodeId = entry.value;
            final text = _optionText[nodeId] ?? nodeId;
            final isSelected = _selected.contains(nodeId);
            return Semantics(
              button: true,
              label: 'Select verse option ${entry.key + 1}',
              child: ElevatedButton(
                onPressed: isSelected ? null : () => _selectOption(nodeId),
                child: Text(text, maxLines: 2, overflow: TextOverflow.ellipsis),
              ),
            );
          }).toList(),
        ),
        const SizedBox(height: 16),
        Text(
          'Selected: ${_selected.length} / ${_options.length}',
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 8),
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: _selected.map((nodeId) {
            return Chip(label: Text(_optionText[nodeId] ?? nodeId));
          }).toList(),
        ),
        const SizedBox(height: 16),
        Row(
          children: [
            Semantics(
              button: true,
              label: 'Reset selection',
              child: TextButton(
                onPressed: _reset,
                child: const Text('Reset'),
              ),
            ),
            const Spacer(),
            Semantics(
              button: true,
              label: 'Submit sequence order',
              child: ElevatedButton(
                onPressed: _selected.length == _options.length ? _submit : null,
                child: const Text('Submit'),
              ),
            ),
          ],
        ),
      ],
    );
  }
}
