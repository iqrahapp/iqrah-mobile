import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/services/content_service.dart';
import 'package:iqrah/services/exercise_content_service.dart';
import 'package:iqrah/services/node_id_service.dart';
import 'package:iqrah/services/translation_service.dart';
import 'package:iqrah/utils/arabic_normalizer.dart';
import 'package:iqrah/widgets/error_banner.dart';

class ReverseClozeWidget extends ConsumerStatefulWidget {
  final String nodeId;
  final int blankPosition;
  final void Function(bool isCorrect) onComplete;
  final ContentService? contentService;
  final TranslationService? translationService;
  final int? translatorId;

  const ReverseClozeWidget({
    super.key,
    required this.nodeId,
    required this.blankPosition,
    required this.onComplete,
    this.contentService,
    this.translationService,
    this.translatorId,
  });

  @override
  ConsumerState<ReverseClozeWidget> createState() => _ReverseClozeWidgetState();
}

class _ReverseClozeWidgetState extends ConsumerState<ReverseClozeWidget> {
  late final ContentService _contentService;
  late final TranslationService _translationService;
  final TextEditingController _controller = TextEditingController();
  bool _loading = true;
  String? _error;
  String? _translation;
  String? _correctWord;

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
      final verseKey = NodeIdService.parseVerseKey(widget.nodeId);
      final words = await _contentService.getWordsForVerse(verseKey);
      if (words.isEmpty) {
        throw Exception('Verse has no words');
      }

      words.sort((a, b) => a.position.compareTo(b.position));
      final target = words.firstWhere(
        (word) => word.position == widget.blankPosition,
        orElse: () => words.first,
      );
      _correctWord = target.textUthmani;

      final prefs = ref.read(userPreferencesProvider);
      final translatorId = widget.translatorId ?? prefs.preferredTranslatorId ?? 1;
      _translation = await _translationService.getTranslation(
        nodeId: widget.nodeId,
        translatorId: translatorId,
      );
      if (_translation == null || _translation!.trim().isEmpty) {
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
          'Fill the missing Arabic word',
          style: Theme.of(context).textTheme.titleMedium,
        ),
        const SizedBox(height: 12),
        Text(
          _translation ?? '',
          textAlign: TextAlign.center,
          style: Theme.of(context).textTheme.bodyLarge,
        ),
        const SizedBox(height: 20),
        TextField(
          controller: _controller,
          textDirection: TextDirection.rtl,
          decoration: const InputDecoration(
            labelText: 'Missing word',
            border: OutlineInputBorder(),
          ),
          onSubmitted: (_) => _submit(),
        ),
        const SizedBox(height: 12),
        Semantics(
          button: true,
          label: 'Submit Arabic word',
          child: ElevatedButton(
            onPressed: _submit,
            child: const Text('Check'),
          ),
        ),
      ],
    );
  }
}
