import 'package:flutter/material.dart';
import 'package:iqrah/services/content_service.dart';
import 'package:iqrah/utils/arabic_normalizer.dart';
import 'package:iqrah/widgets/error_banner.dart';

class FirstWordRecallWidget extends StatefulWidget {
  final String nodeId;
  final String verseKey;
  final void Function(bool isCorrect) onComplete;
  final ContentService? contentService;

  const FirstWordRecallWidget({
    super.key,
    required this.nodeId,
    required this.verseKey,
    required this.onComplete,
    this.contentService,
  });

  @override
  State<FirstWordRecallWidget> createState() => _FirstWordRecallWidgetState();
}

class _FirstWordRecallWidgetState extends State<FirstWordRecallWidget> {
  late final ContentService _contentService;
  final TextEditingController _controller = TextEditingController();
  bool _loading = true;
  String? _error;
  String? _promptText;
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
      final words = await _contentService.getWordsForVerse(widget.verseKey);
      if (words.isEmpty) {
        throw Exception('Verse has no words');
      }

      words.sort((a, b) => a.position.compareTo(b.position));
      final first = words.first;
      _correctWord = first.textUthmani;

      final buffer = StringBuffer();
      for (final word in words) {
        if (word.position == first.position) {
          buffer.write('_____ ');
        } else {
          buffer.write('${word.textUthmani} ');
        }
      }

      _promptText = buffer.toString().trim();
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
          'Type the first word',
          style: Theme.of(context).textTheme.titleMedium,
        ),
        const SizedBox(height: 12),
        Text(
          _promptText ?? '',
          style: Theme.of(context).textTheme.headlineSmall,
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 20),
        TextField(
          controller: _controller,
          textDirection: TextDirection.rtl,
          decoration: const InputDecoration(
            labelText: 'First word',
            border: OutlineInputBorder(),
          ),
          onSubmitted: (_) => _submit(),
        ),
        const SizedBox(height: 12),
        Semantics(
          button: true,
          label: 'Submit first word',
          child: ElevatedButton(
            onPressed: _submit,
            child: const Text('Check'),
          ),
        ),
      ],
    );
  }
}
