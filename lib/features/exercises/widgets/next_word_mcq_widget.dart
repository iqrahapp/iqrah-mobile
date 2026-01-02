import 'package:flutter/material.dart';
import 'package:iqrah/services/content_service.dart';
import 'package:iqrah/services/node_id_service.dart';
import 'package:iqrah/widgets/error_banner.dart';

class NextWordMcqWidget extends StatefulWidget {
  final String nodeId;
  final int contextPosition;
  final List<String> distractorNodeIds;
  final void Function(bool isCorrect) onComplete;
  final ContentService? contentService;

  const NextWordMcqWidget({
    super.key,
    required this.nodeId,
    required this.contextPosition,
    required this.distractorNodeIds,
    required this.onComplete,
    this.contentService,
  });

  @override
  State<NextWordMcqWidget> createState() => _NextWordMcqWidgetState();
}

class _NextWordMcqWidgetState extends State<NextWordMcqWidget> {
  late final ContentService _contentService;
  bool _loading = true;
  String? _error;
  String? _promptText;
  String? _correctWord;
  List<String> _options = [];

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
      final verseKey = NodeIdService.parseVerseKey(widget.nodeId);
      final words = await _contentService.getWordsForVerse(verseKey);
      if (words.isEmpty) {
        throw Exception('Verse has no words');
      }

      words.sort((a, b) => a.position.compareTo(b.position));
      final buffer = StringBuffer();

      for (final word in words) {
        if (word.position <= widget.contextPosition) {
          buffer.write('${word.textUthmani} ');
        }
        if (word.position == widget.contextPosition + 1) {
          _correctWord = word.textUthmani;
        }
      }

      final options = <String>[];
      if (_correctWord != null) {
        options.add(_correctWord!);
      }

      for (final nodeId in widget.distractorNodeIds) {
        final text = await _resolveWordText(nodeId);
        if (text.isNotEmpty) {
          options.add(text);
        }
      }

      _promptText = buffer.toString().trim();
      if (mounted) {
        setState(() {
          _options = options.toSet().toList()..shuffle();
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

  Future<String> _resolveWordText(String nodeId) async {
    final baseType = NodeIdService.getBaseNodeType(nodeId);
    if (baseType == NodeType.word) {
      final wordId = NodeIdService.parseWordId(nodeId);
      final word = await _contentService.getWord(wordId);
      return word?.textUthmani ?? '';
    }
    if (baseType == NodeType.wordInstance) {
      final (chapter, verse, position) = NodeIdService.parseWordInstance(nodeId);
      final word = await _contentService.getWordAtPosition(
        chapter: chapter,
        verse: verse,
        position: position,
      );
      return word?.textUthmani ?? '';
    }
    return '';
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
          'What is the next word?',
          style: Theme.of(context).textTheme.titleMedium,
        ),
        const SizedBox(height: 12),
        Text(
          _promptText ?? '',
          style: Theme.of(context).textTheme.headlineSmall,
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 20),
        ..._options.asMap().entries.map((entry) {
          return Padding(
            padding: const EdgeInsets.symmetric(vertical: 6),
            child: Semantics(
              button: true,
              label: 'Select next word option ${entry.key + 1}',
              child: ElevatedButton(
                onPressed: () => widget.onComplete(entry.value == _correctWord),
                child: Text(entry.value),
              ),
            ),
          );
        }),
      ],
    );
  }
}
