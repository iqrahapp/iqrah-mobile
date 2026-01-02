import 'package:flutter/material.dart';
import 'package:iqrah/services/content_service.dart';
import 'package:iqrah/services/node_id_service.dart';
import 'package:iqrah/widgets/error_banner.dart';

class CrossVerseConnectionWidget extends StatefulWidget {
  final String nodeId;
  final List<String> relatedVerseIds;
  final String connectionTheme;
  final void Function(bool isCorrect) onComplete;
  final ContentService? contentService;

  const CrossVerseConnectionWidget({
    super.key,
    required this.nodeId,
    required this.relatedVerseIds,
    required this.connectionTheme,
    required this.onComplete,
    this.contentService,
  });

  @override
  State<CrossVerseConnectionWidget> createState() =>
      _CrossVerseConnectionWidgetState();
}

class _CrossVerseConnectionWidgetState
    extends State<CrossVerseConnectionWidget> {
  late final ContentService _contentService;
  bool _loading = true;
  String? _error;
  List<_VerseOption> _options = [];
  String? _selectedId;

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
      _options = [];
      for (final nodeId in widget.relatedVerseIds) {
        final text = await _fetchVerseText(nodeId);
        _options.add(_VerseOption(nodeId: nodeId, text: text));
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

  Future<String> _fetchVerseText(String nodeId) async {
    try {
      final verseKey = NodeIdService.parseVerseKey(nodeId);
      final verse = await _contentService.getVerse(verseKey);
      return verse?.textUthmani ?? verseKey;
    } catch (_) {
      return nodeId;
    }
  }

  void _submit() {
    if (_selectedId == null) return;
    final correctId =
        widget.relatedVerseIds.isNotEmpty ? widget.relatedVerseIds.first : null;
    final correct = correctId != null && _selectedId == correctId;
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

    if (widget.relatedVerseIds.isEmpty) {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Text('No related verses provided.'),
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
          'Connection theme',
          style: Theme.of(context).textTheme.titleMedium,
        ),
        const SizedBox(height: 8),
        Text(
          widget.connectionTheme,
          textAlign: TextAlign.center,
          style: Theme.of(context).textTheme.bodyLarge,
        ),
        const SizedBox(height: 16),
        RadioGroup<String>(
          groupValue: _selectedId,
          onChanged: (value) {
            setState(() {
              _selectedId = value;
            });
          },
          child: Column(
            children: _options.map((option) {
              return RadioListTile<String>(
                value: option.nodeId,
                title: Text(option.text),
              );
            }).toList(),
          ),
        ),
        const SizedBox(height: 12),
        Semantics(
          button: true,
          label: 'Check verse connection answer',
          child: ElevatedButton(
            onPressed: _selectedId == null ? null : _submit,
            child: const Text('Check'),
          ),
        ),
      ],
    );
  }
}

class _VerseOption {
  final String nodeId;
  final String text;

  const _VerseOption({
    required this.nodeId,
    required this.text,
  });
}
