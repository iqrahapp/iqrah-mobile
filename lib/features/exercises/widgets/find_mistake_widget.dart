import 'package:flutter/material.dart';
import 'package:iqrah/services/content_service.dart';
import 'package:iqrah/services/node_id_service.dart';
import 'package:iqrah/widgets/error_banner.dart';

class FindMistakeWidget extends StatefulWidget {
  final String nodeId;
  final int mistakePosition;
  final String correctWordNodeId;
  final String incorrectWordNodeId;
  final void Function(bool isCorrect) onComplete;
  final ContentService? contentService;

  const FindMistakeWidget({
    super.key,
    required this.nodeId,
    required this.mistakePosition,
    required this.correctWordNodeId,
    required this.incorrectWordNodeId,
    required this.onComplete,
    this.contentService,
  });

  @override
  State<FindMistakeWidget> createState() => _FindMistakeWidgetState();
}

class _FindMistakeWidgetState extends State<FindMistakeWidget> {
  late final ContentService _contentService;
  bool _loading = true;
  String? _error;
  List<_WordSlot> _slots = [];

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
      final incorrectText = await _resolveWordText(widget.incorrectWordNodeId);
      if (incorrectText == null) {
        throw Exception('Invalid incorrect word');
      }

      _slots = words
          .map(
            (word) => _WordSlot(
              position: word.position,
              text: word.position == widget.mistakePosition
                  ? incorrectText
                  : word.textUthmani,
            ),
          )
          .toList();

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

  Future<String?> _resolveWordText(String nodeId) async {
    final nodeType = NodeIdService.getBaseNodeType(nodeId);
    if (nodeType == NodeType.word) {
      final wordId = NodeIdService.parseWordId(nodeId);
      final word = await _contentService.getWord(wordId);
      return word?.textUthmani;
    }

    if (nodeType == NodeType.wordInstance) {
      final (chapter, verse, position) = NodeIdService.parseWordInstance(nodeId);
      final word = await _contentService.getWordAtPosition(
        chapter: chapter,
        verse: verse,
        position: position,
      );
      return word?.textUthmani;
    }

    return null;
  }

  void _onSelect(int position) {
    widget.onComplete(position == widget.mistakePosition);
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
          'Tap the incorrect word',
          style: Theme.of(context).textTheme.titleMedium,
        ),
        const SizedBox(height: 16),
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: _slots.map((slot) {
            return Semantics(
              button: true,
              label: 'Select word at position ${slot.position}',
              child: ElevatedButton(
                onPressed: () => _onSelect(slot.position),
                child: Text(slot.text),
              ),
            );
          }).toList(),
        ),
      ],
    );
  }
}

class _WordSlot {
  final int position;
  final String text;

  const _WordSlot({
    required this.position,
    required this.text,
  });
}
