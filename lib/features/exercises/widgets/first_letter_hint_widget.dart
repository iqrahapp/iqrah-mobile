import 'package:flutter/material.dart';
import 'package:iqrah/services/content_service.dart';
import 'package:iqrah/services/node_id_service.dart';
import 'package:iqrah/utils/arabic_normalizer.dart';
import 'package:iqrah/widgets/error_banner.dart';

class FirstLetterHintWidget extends StatefulWidget {
  final String nodeId;
  final int wordPosition;
  final void Function(bool isCorrect) onComplete;
  final ContentService? contentService;

  const FirstLetterHintWidget({
    super.key,
    required this.nodeId,
    required this.wordPosition,
    required this.onComplete,
    this.contentService,
  });

  @override
  State<FirstLetterHintWidget> createState() => _FirstLetterHintWidgetState();
}

class _FirstLetterHintWidgetState extends State<FirstLetterHintWidget> {
  late final ContentService _contentService;
  final TextEditingController _controller = TextEditingController();
  bool _loading = true;
  String? _error;
  String? _hintLetter;
  String? _correctWord;

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
      final verseKey = NodeIdService.parseVerseKey(widget.nodeId);
      final words = await _contentService.getWordsForVerse(verseKey);
      if (words.isEmpty) {
        throw Exception('Verse has no words');
      }

      words.sort((a, b) => a.position.compareTo(b.position));
      final target = words.firstWhere(
        (word) => word.position == widget.wordPosition,
        orElse: () => words.first,
      );
      _correctWord = target.textUthmani;
      _hintLetter = target.textUthmani.isNotEmpty
          ? target.textUthmani.substring(0, 1)
          : '';

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
    final correct = _normalize(_controller.text) == _normalize(_correctWord);
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

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Text(
          'First letter hint',
          style: Theme.of(context).textTheme.titleMedium,
        ),
        const SizedBox(height: 12),
        if (_hintLetter != null)
          Text(
            'Starts with: $_hintLetter',
            style: Theme.of(context).textTheme.headlineSmall,
            textAlign: TextAlign.center,
          ),
        const SizedBox(height: 20),
        TextField(
          controller: _controller,
          textDirection: TextDirection.rtl,
          decoration: const InputDecoration(
            labelText: 'Answer',
            border: OutlineInputBorder(),
          ),
          onSubmitted: (_) => _submit(),
        ),
        const SizedBox(height: 12),
        Semantics(
          button: true,
          label: 'Submit word answer',
          child: ElevatedButton(
            onPressed: _submit,
            child: const Text('Check'),
          ),
        ),
      ],
    );
  }
}
