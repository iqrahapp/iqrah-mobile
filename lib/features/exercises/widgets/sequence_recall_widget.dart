import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:iqrah/services/content_service.dart';
import 'package:iqrah/services/node_id_service.dart';
import 'package:iqrah/widgets/error_banner.dart';

class SequenceRecallWidget extends StatefulWidget {
  final String nodeId;
  final List<String> correctSequence;
  final List<List<String>> options;
  final void Function(bool isCorrect) onComplete;
  final ContentService? contentService;

  const SequenceRecallWidget({
    super.key,
    required this.nodeId,
    required this.correctSequence,
    required this.options,
    required this.onComplete,
    this.contentService,
  });

  @override
  State<SequenceRecallWidget> createState() => _SequenceRecallWidgetState();
}

class _SequenceRecallWidgetState extends State<SequenceRecallWidget> {
  late final ContentService _contentService;
  bool _loading = true;
  String? _error;
  String? _promptText;
  List<String> _optionLabels = [];

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
      final verse = await _contentService.getVerse(verseKey);
      _promptText = verse?.textUthmani ?? verseKey;

      final labels = <String>[];
      for (final option in widget.options) {
        final texts = <String>[];
        for (final nodeId in option) {
          final text = await _fetchVerseText(nodeId);
          texts.add(text);
        }
        labels.add(texts.join(' • '));
      }

      if (mounted) {
        setState(() {
          _optionLabels = labels;
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

  bool _isCorrectOption(int index) {
    final selected = widget.options[index]
        .map(NodeIdService.baseNodeId)
        .toList(growable: false);
    final correct = widget.correctSequence
        .map(NodeIdService.baseNodeId)
        .toList(growable: false);
    return listEquals(selected, correct);
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
          'What comes next?',
          style: Theme.of(context).textTheme.titleMedium,
        ),
        const SizedBox(height: 12),
        if (_promptText != null)
          Text(
            _promptText!,
            style: Theme.of(context).textTheme.headlineSmall,
            textAlign: TextAlign.center,
          ),
        const SizedBox(height: 24),
        ..._optionLabels.asMap().entries.map((entry) {
          return Padding(
            padding: const EdgeInsets.symmetric(vertical: 6),
            child: Semantics(
              button: true,
              label: 'Select sequence option ${entry.key + 1}',
              child: ElevatedButton(
                onPressed: () => widget.onComplete(_isCorrectOption(entry.key)),
                child: Text(entry.value),
              ),
            ),
          );
        }),
      ],
    );
  }
}
