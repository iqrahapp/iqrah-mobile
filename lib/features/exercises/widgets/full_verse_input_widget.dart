import 'package:flutter/material.dart';
import 'package:iqrah/services/content_service.dart';
import 'package:iqrah/services/node_id_service.dart';
import 'package:iqrah/utils/arabic_normalizer.dart';
import 'package:iqrah/widgets/error_banner.dart';

class FullVerseInputWidget extends StatefulWidget {
  final String nodeId;
  final void Function(bool isCorrect) onComplete;
  final ContentService? contentService;

  const FullVerseInputWidget({
    super.key,
    required this.nodeId,
    required this.onComplete,
    this.contentService,
  });

  @override
  State<FullVerseInputWidget> createState() => _FullVerseInputWidgetState();
}

class _FullVerseInputWidgetState extends State<FullVerseInputWidget> {
  late final ContentService _contentService;
  final TextEditingController _controller = TextEditingController();
  bool _loading = true;
  String? _error;
  String? _verseKey;
  String? _correctText;

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
      _verseKey = NodeIdService.parseVerseKey(widget.nodeId);
      final verse = await _contentService.getVerse(_verseKey!);
      _correctText = verse?.textUthmani;
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
    final correct = _normalize(_controller.text) == _normalize(_correctText);
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
          'Type the full verse',
          style: Theme.of(context).textTheme.titleMedium,
        ),
        const SizedBox(height: 8),
        if (_verseKey != null)
          Text(
            _verseKey!,
            textAlign: TextAlign.center,
            style: Theme.of(context).textTheme.bodyLarge,
          ),
        const SizedBox(height: 16),
        TextField(
          controller: _controller,
          textDirection: TextDirection.rtl,
          decoration: const InputDecoration(
            labelText: 'Verse text',
            border: OutlineInputBorder(),
          ),
          onSubmitted: (_) => _submit(),
        ),
        const SizedBox(height: 12),
        Semantics(
          button: true,
          label: 'Submit verse',
          child: ElevatedButton(
            onPressed: _submit,
            child: const Text('Check'),
          ),
        ),
      ],
    );
  }
}
