import 'package:flutter/material.dart';
import 'package:iqrah/services/content_service.dart';
import 'package:iqrah/services/node_id_service.dart';
import 'package:iqrah/widgets/error_banner.dart';

class PosTaggingWidget extends StatefulWidget {
  final String nodeId;
  final String correctPos;
  final List<String> options;
  final void Function(bool isCorrect) onComplete;
  final ContentService? contentService;

  const PosTaggingWidget({
    super.key,
    required this.nodeId,
    required this.correctPos,
    required this.options,
    required this.onComplete,
    this.contentService,
  });

  @override
  State<PosTaggingWidget> createState() => _PosTaggingWidgetState();
}

class _PosTaggingWidgetState extends State<PosTaggingWidget> {
  late final ContentService _contentService;
  bool _loading = true;
  String? _error;
  String? _wordText;

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
      final nodeType = NodeIdService.getBaseNodeType(widget.nodeId);
      if (nodeType == NodeType.word) {
        final wordId = NodeIdService.parseWordId(widget.nodeId);
        final word = await _contentService.getWord(wordId);
        _wordText = word?.textUthmani ?? widget.nodeId;
      } else if (nodeType == NodeType.wordInstance) {
        final (chapter, verse, position) =
            NodeIdService.parseWordInstance(widget.nodeId);
        final word = await _contentService.getWordAtPosition(
          chapter: chapter,
          verse: verse,
          position: position,
        );
        _wordText = word?.textUthmani ?? widget.nodeId;
      } else {
        throw Exception('POS tagging expects a word node');
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

  void _submit(String option) {
    final correct =
        option.toLowerCase().trim() == widget.correctPos.toLowerCase().trim();
    widget.onComplete(correct);
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

    if (widget.options.isEmpty || widget.correctPos.trim().isEmpty) {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Text('POS data unavailable for this word.'),
            const SizedBox(height: 12),
            Semantics(
              button: true,
              label: 'Continue to next exercise',
              child: ElevatedButton(
                onPressed: () => widget.onComplete(true),
                child: const Text('Continue'),
              ),
            ),
          ],
        ),
      );
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Text(
          'Select the part of speech',
          style: Theme.of(context).textTheme.titleMedium,
        ),
        const SizedBox(height: 12),
        Text(
          _wordText ?? '',
          textAlign: TextAlign.center,
          style: Theme.of(context).textTheme.headlineSmall,
        ),
        const SizedBox(height: 16),
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: widget.options.asMap().entries.map((entry) {
            return Semantics(
              button: true,
              label: 'Select part of speech option ${entry.key + 1}',
              child: ElevatedButton(
                onPressed: () => _submit(entry.value),
                child: Text(entry.value),
              ),
            );
          }).toList(),
        ),
      ],
    );
  }
}
