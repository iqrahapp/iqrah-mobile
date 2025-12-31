import 'package:flutter/material.dart';
import 'package:iqrah/features/exercises/widgets/echo_recall_widget.dart';

/// Debug screen for testing Echo Recall exercise
class EchoRecallDebugScreen extends StatefulWidget {
  const EchoRecallDebugScreen({super.key});

  @override
  State<EchoRecallDebugScreen> createState() => _EchoRecallDebugScreenState();
}

class _EchoRecallDebugScreenState extends State<EchoRecallDebugScreen> {
  final _chapterController = TextEditingController(text: '1');
  final _startVerseController = TextEditingController(text: '1');
  final _endVerseController = TextEditingController(text: '7');
  bool _isRunning = false;

  @override
  void dispose() {
    _chapterController.dispose();
    _startVerseController.dispose();
    _endVerseController.dispose();
    super.dispose();
  }

  List<String> _buildAyahNodeIds() {
    final chapter = int.tryParse(_chapterController.text) ?? 1;
    final startVerse = int.tryParse(_startVerseController.text) ?? 1;
    final endVerse = int.tryParse(_endVerseController.text) ?? 7;

    final nodeIds = <String>[];
    for (var v = startVerse; v <= endVerse; v++) {
      nodeIds.add('VERSE:$chapter:$v');
    }
    return nodeIds;
  }

  void _startSession() {
    setState(() {
      _isRunning = true;
    });
  }

  void _onComplete() {
    setState(() {
      _isRunning = false;
    });
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(
        content: Text('Echo Recall session completed!'),
        backgroundColor: Colors.green,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    if (_isRunning) {
      return Scaffold(
        appBar: AppBar(
          title: const Text('Echo Recall'),
          leading: IconButton(
            icon: const Icon(Icons.close),
            onPressed: () {
              setState(() {
                _isRunning = false;
              });
            },
          ),
        ),
        body: EchoRecallWidget(
          userId: 'test_user',
          ayahNodeIds: _buildAyahNodeIds(),
          onComplete: _onComplete,
        ),
      );
    }

    return Scaffold(
      appBar: AppBar(
        title: const Text('Echo Recall Debug'),
      ),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Card(
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'Select Ayahs',
                      style: Theme.of(context).textTheme.titleMedium,
                    ),
                    const SizedBox(height: 16),
                    Row(
                      children: [
                        Expanded(
                          child: TextField(
                            controller: _chapterController,
                            decoration: const InputDecoration(
                              labelText: 'Chapter',
                              border: OutlineInputBorder(),
                            ),
                            keyboardType: TextInputType.number,
                          ),
                        ),
                        const SizedBox(width: 12),
                        Expanded(
                          child: TextField(
                            controller: _startVerseController,
                            decoration: const InputDecoration(
                              labelText: 'Start Verse',
                              border: OutlineInputBorder(),
                            ),
                            keyboardType: TextInputType.number,
                          ),
                        ),
                        const SizedBox(width: 12),
                        Expanded(
                          child: TextField(
                            controller: _endVerseController,
                            decoration: const InputDecoration(
                              labelText: 'End Verse',
                              border: OutlineInputBorder(),
                            ),
                            keyboardType: TextInputType.number,
                          ),
                        ),
                      ],
                    ),
                    const SizedBox(height: 16),
                    Text(
                      'Will practice: ${_buildAyahNodeIds().join(", ")}',
                      style: Theme.of(context).textTheme.bodySmall,
                    ),
                  ],
                ),
              ),
            ),
            const SizedBox(height: 16),
            Card(
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'Presets',
                      style: Theme.of(context).textTheme.titleMedium,
                    ),
                    const SizedBox(height: 12),
                    Wrap(
                      spacing: 8,
                      runSpacing: 8,
                      children: [
                        _PresetButton(
                          label: 'Al-Fatihah',
                          onTap: () {
                            _chapterController.text = '1';
                            _startVerseController.text = '1';
                            _endVerseController.text = '7';
                            setState(() {});
                          },
                        ),
                        _PresetButton(
                          label: 'Ayat Al-Kursi',
                          onTap: () {
                            _chapterController.text = '2';
                            _startVerseController.text = '255';
                            _endVerseController.text = '255';
                            setState(() {});
                          },
                        ),
                        _PresetButton(
                          label: 'Al-Ikhlas',
                          onTap: () {
                            _chapterController.text = '112';
                            _startVerseController.text = '1';
                            _endVerseController.text = '4';
                            setState(() {});
                          },
                        ),
                        _PresetButton(
                          label: 'Al-Nas',
                          onTap: () {
                            _chapterController.text = '114';
                            _startVerseController.text = '1';
                            _endVerseController.text = '6';
                            setState(() {});
                          },
                        ),
                      ],
                    ),
                  ],
                ),
              ),
            ),
            const Spacer(),
            FilledButton.icon(
              onPressed: _startSession,
              icon: const Icon(Icons.play_arrow),
              label: const Text('Start Echo Recall'),
            ),
          ],
        ),
      ),
    );
  }
}

class _PresetButton extends StatelessWidget {
  final String label;
  final VoidCallback onTap;

  const _PresetButton({
    required this.label,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return OutlinedButton(
      onPressed: onTap,
      child: Text(label),
    );
  }
}
