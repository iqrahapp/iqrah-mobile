import 'package:flutter/material.dart';
import 'package:iqrah/services/content_service.dart';
import 'package:iqrah/services/node_id_service.dart';
import 'package:iqrah/services/translation_service.dart';
import 'package:iqrah/utils/english_normalizer.dart';
import 'package:iqrah/widgets/error_banner.dart';

class TranslatePhraseWidget extends StatefulWidget {
  final String nodeId;
  final int translatorId;
  final void Function(bool isCorrect) onComplete;
  final ContentService? contentService;
  final TranslationService? translationService;

  const TranslatePhraseWidget({
    super.key,
    required this.nodeId,
    required this.translatorId,
    required this.onComplete,
    this.contentService,
    this.translationService,
  });

  @override
  State<TranslatePhraseWidget> createState() => _TranslatePhraseWidgetState();
}

class _TranslatePhraseWidgetState extends State<TranslatePhraseWidget> {
  late final ContentService _contentService;
  late final TranslationService _translationService;
  final TextEditingController _controller = TextEditingController();
  bool _loading = true;
  String? _error;
  String? _arabicText;
  String? _correctTranslation;

  @override
  void initState() {
    super.initState();
    _contentService = widget.contentService ?? ContentService();
    _translationService = widget.translationService ?? TranslationService();
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
        _arabicText = word?.textUthmani ?? widget.nodeId;
      } else if (nodeType == NodeType.wordInstance) {
        final (chapter, verse, position) =
            NodeIdService.parseWordInstance(widget.nodeId);
        final word = await _contentService.getWordAtPosition(
          chapter: chapter,
          verse: verse,
          position: position,
        );
        _arabicText = word?.textUthmani ?? widget.nodeId;
      } else {
        final verseKey = NodeIdService.parseVerseKey(widget.nodeId);
        final verse = await _contentService.getVerse(verseKey);
        _arabicText = verse?.textUthmani ?? verseKey;
      }
      _correctTranslation = await _translationService.getTranslation(
        nodeId: widget.nodeId,
        translatorId: widget.translatorId,
      );
      if (_correctTranslation == null ||
          _correctTranslation!.trim().isEmpty) {
        throw Exception('Translation unavailable for this item.');
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
    final correct = EnglishNormalizer.normalize(_controller.text) ==
        EnglishNormalizer.normalize(_correctTranslation ?? '');
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

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Text(
          'Translate the phrase',
          style: Theme.of(context).textTheme.titleMedium,
        ),
        const SizedBox(height: 12),
        Text(
          _arabicText ?? '',
          textAlign: TextAlign.center,
          style: Theme.of(context).textTheme.headlineSmall,
        ),
        const SizedBox(height: 20),
        TextField(
          controller: _controller,
          decoration: const InputDecoration(
            labelText: 'Translation',
            border: OutlineInputBorder(),
          ),
          onSubmitted: (_) => _submit(),
        ),
        const SizedBox(height: 12),
        Semantics(
          button: true,
          label: 'Submit phrase translation',
          child: ElevatedButton(
            onPressed: _submit,
            child: const Text('Check'),
          ),
        ),
      ],
    );
  }
}
