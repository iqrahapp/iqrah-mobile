import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:iqrah/services/content_service.dart';
import 'package:iqrah/utils/arabic_normalizer.dart';
import 'package:iqrah/widgets/error_banner.dart';
import 'package:path_provider/path_provider.dart';

class AyahChainWidget extends StatefulWidget {
  final String nodeId;
  final List<String> verseKeys;
  final int currentIndex;
  final int completedCount;
  final void Function(bool isCorrect) onComplete;
  final ContentService? contentService;
  final Future<Directory> Function()? directoryProvider;
  final bool enablePersistence;

  const AyahChainWidget({
    super.key,
    required this.nodeId,
    required this.verseKeys,
    required this.currentIndex,
    required this.completedCount,
    required this.onComplete,
    this.contentService,
    this.directoryProvider,
    this.enablePersistence = true,
  });

  @override
  State<AyahChainWidget> createState() => _AyahChainWidgetState();
}

class _AyahChainWidgetState extends State<AyahChainWidget> {
  late final ContentService _contentService;
  final TextEditingController _controller = TextEditingController();
  bool _loading = true;
  String? _error;
  int _index = 0;
  int _completedCount = 0;
  String? _currentVerseKey;
  String? _correctText;

  @override
  void initState() {
    super.initState();
    _contentService = widget.contentService ?? ContentService();
    _index = widget.currentIndex;
    _completedCount = widget.completedCount;
    _initialize();
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  Future<void> _initialize() async {
    final saved = await _readProgress();
    if (saved != null && _shouldApplyProgress(saved)) {
      _index = saved.currentIndex;
      _completedCount = saved.completedCount;
    }
    await _loadCurrentVerse();
  }

  Future<void> _loadCurrentVerse() async {
    setState(() {
      _loading = true;
      _error = null;
    });

    try {
      if (_index >= widget.verseKeys.length) {
        _currentVerseKey = null;
        _correctText = null;
        await _clearProgress();
      } else {
        _currentVerseKey = widget.verseKeys[_index];
        final verse = await _contentService.getVerse(_currentVerseKey!);
        _correctText = verse?.textUthmani;
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
    final correct = _normalize(_controller.text) == _normalize(_correctText);
    if (!correct) {
      widget.onComplete(false);
      return;
    }

    if (_index + 1 >= widget.verseKeys.length) {
      _clearProgress();
      widget.onComplete(true);
      return;
    }

    setState(() {
      _index += 1;
      _completedCount = (_completedCount + 1).clamp(0, widget.verseKeys.length);
      _controller.clear();
    });
    _saveProgress();
    _loadCurrentVerse();
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
          onRetry: _loadCurrentVerse,
        ),
      );
    }

    if (_currentVerseKey == null) {
      return const Center(child: Text('No verses remaining.'));
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Text(
          'Ayah Chain',
          style: Theme.of(context).textTheme.titleMedium,
        ),
        const SizedBox(height: 8),
        Text(
          'Progress ${_index + 1} / ${widget.verseKeys.length}',
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 8),
        Text(
          _currentVerseKey ?? '',
          textAlign: TextAlign.center,
          style: Theme.of(context).textTheme.bodyLarge,
        ),
        const SizedBox(height: 16),
        TextField(
          controller: _controller,
          textDirection: TextDirection.rtl,
          decoration: const InputDecoration(
            labelText: 'Type the verse',
            border: OutlineInputBorder(),
          ),
          onSubmitted: (_) => _submit(),
        ),
        const SizedBox(height: 12),
        Semantics(
          button: true,
          label: 'Submit ayah chain answer',
          child: ElevatedButton(
            onPressed: _submit,
            child: const Text('Check'),
          ),
        ),
      ],
    );
  }

  Future<Directory?> _getPersistenceDir() async {
    if (!widget.enablePersistence) {
      return null;
    }
    try {
      if (widget.directoryProvider != null) {
        return await widget.directoryProvider!();
      }
      return await getApplicationDocumentsDirectory();
    } catch (_) {
      return null;
    }
  }

  String _sanitizeKey(String value) {
    return value.replaceAll(RegExp(r'[^A-Za-z0-9_-]+'), '_');
  }

  Future<File?> _progressFile() async {
    final dir = await _getPersistenceDir();
    if (dir == null) return null;
    final filename = 'ayah_chain_${_sanitizeKey(widget.nodeId)}.json';
    return File('${dir.path}/$filename');
  }

  bool _shouldApplyProgress(_AyahChainProgress progress) {
    if (progress.nodeId != widget.nodeId) return false;
    if (progress.verseKeys.length != widget.verseKeys.length) return false;
    for (var i = 0; i < progress.verseKeys.length; i += 1) {
      if (progress.verseKeys[i] != widget.verseKeys[i]) {
        return false;
      }
    }
    if (progress.currentIndex < 0 ||
        progress.currentIndex >= widget.verseKeys.length) {
      return false;
    }
    return progress.currentIndex >= _index;
  }

  Future<void> _saveProgress() async {
    final file = await _progressFile();
    if (file == null) return;
    final payload = _AyahChainProgress(
      nodeId: widget.nodeId,
      verseKeys: widget.verseKeys,
      currentIndex: _index,
      completedCount: _completedCount,
    );
    try {
      await file.writeAsString(jsonEncode(payload.toJson()));
    } catch (_) {}
  }

  Future<void> _clearProgress() async {
    final file = await _progressFile();
    if (file == null) return;
    try {
      if (await file.exists()) {
        await file.delete();
      }
    } catch (_) {}
  }

  Future<_AyahChainProgress?> _readProgress() async {
    final file = await _progressFile();
    if (file == null) return null;
    try {
      if (!await file.exists()) return null;
      final data = jsonDecode(await file.readAsString());
      if (data is! Map<String, dynamic>) return null;
      return _AyahChainProgress.fromJson(data);
    } catch (_) {
      return null;
    }
  }
}

class _AyahChainProgress {
  final String nodeId;
  final List<String> verseKeys;
  final int currentIndex;
  final int completedCount;

  const _AyahChainProgress({
    required this.nodeId,
    required this.verseKeys,
    required this.currentIndex,
    required this.completedCount,
  });

  Map<String, dynamic> toJson() => {
        'nodeId': nodeId,
        'verseKeys': verseKeys,
        'currentIndex': currentIndex,
        'completedCount': completedCount,
      };

  factory _AyahChainProgress.fromJson(Map<String, dynamic> json) {
    final keys = (json['verseKeys'] as List?)
            ?.map((e) => e.toString())
            .toList() ??
        [];
    return _AyahChainProgress(
      nodeId: json['nodeId']?.toString() ?? '',
      verseKeys: keys,
      currentIndex: json['currentIndex'] is int ? json['currentIndex'] as int : 0,
      completedCount:
          json['completedCount'] is int ? json['completedCount'] as int : 0,
    );
  }
}
