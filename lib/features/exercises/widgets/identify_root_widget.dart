import 'package:flutter/material.dart';
import 'package:iqrah/services/content_service.dart';
import 'package:iqrah/services/node_id_service.dart';
import 'package:iqrah/utils/arabic_normalizer.dart';
import 'package:iqrah/widgets/error_banner.dart';

class IdentifyRootWidget extends StatefulWidget {
  final String nodeId;
  final String root;
  final void Function(bool isCorrect) onComplete;
  final ContentService? contentService;

  const IdentifyRootWidget({
    super.key,
    required this.nodeId,
    required this.root,
    required this.onComplete,
    this.contentService,
  });

  @override
  State<IdentifyRootWidget> createState() => _IdentifyRootWidgetState();
}

class _IdentifyRootWidgetState extends State<IdentifyRootWidget> {
  late final ContentService _contentService;
  final TextEditingController _controller = TextEditingController();
  bool _loading = true;
  String? _error;
  String? _wordText;

  @override
  void initState() {
    super.initState();
    _contentService = widget.contentService ?? ContentService();
    _loadContent();
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
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
      } else {
        final (chapter, verse, position) =
            NodeIdService.parseWordInstance(widget.nodeId);
        final word = await _contentService.getWordAtPosition(
          chapter: chapter,
          verse: verse,
          position: position,
        );
        _wordText = word?.textUthmani ?? widget.nodeId;
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

  void _submit() {
    final correct =
        _normalize(_controller.text) == _normalize(widget.root);
    widget.onComplete(correct);
  }

  String _normalize(String? text) {
    if (text == null) return '';
    return ArabicNormalizer.normalize(text)
        .replaceAll(RegExp(r'\s+'), '')
        .trim();
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

    if (widget.root.trim().isEmpty) {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Text('Root data unavailable for this word.'),
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
          'Identify the root',
          style: Theme.of(context).textTheme.titleMedium,
        ),
        const SizedBox(height: 12),
        Text(
          _wordText ?? '',
          textAlign: TextAlign.center,
          style: Theme.of(context).textTheme.headlineSmall,
        ),
        const SizedBox(height: 20),
        TextField(
          controller: _controller,
          textDirection: TextDirection.rtl,
          decoration: const InputDecoration(
            labelText: 'Root letters',
            border: OutlineInputBorder(),
          ),
          onSubmitted: (_) => _submit(),
        ),
        const SizedBox(height: 12),
        Semantics(
          button: true,
          label: 'Submit root answer',
          child: ElevatedButton(
            onPressed: _submit,
            child: const Text('Check'),
          ),
        ),
      ],
    );
  }
}
