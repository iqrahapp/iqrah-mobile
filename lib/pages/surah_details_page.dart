import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:iqrah/providers/quran_provider.dart';
import 'package:iqrah/rust_bridge/api.dart';
import 'package:media_kit/media_kit.dart';

class SurahDetailsPage extends ConsumerStatefulWidget {
  final SurahInfo surah;

  const SurahDetailsPage({super.key, required this.surah});

  @override
  ConsumerState<SurahDetailsPage> createState() => _SurahDetailsPageState();
}

class _SurahDetailsPageState extends ConsumerState<SurahDetailsPage> {
  late Player _player;
  bool _isPlaying = false;
  Duration _duration = Duration.zero;
  Duration _position = Duration.zero;
  bool _isLoading = true;

  @override
  void initState() {
    super.initState();
    _player = Player();
    _initAudio();
  }

  Future<void> _initAudio() async {
    try {
      final surahNum = widget.surah.number.toString().padLeft(3, '0');
      final url = "https://download.quranicaudio.com/quran/mishaari_raashid_al_3afaasee/$surahNum.mp3";
      
      // Listen to streams
      _player.stream.playing.listen((playing) {
        if (mounted) setState(() => _isPlaying = playing);
      });

      _player.stream.duration.listen((d) {
        if (mounted) setState(() => _duration = d);
      });

      _player.stream.position.listen((p) {
        if (mounted) setState(() => _position = p);
      });

      _player.stream.completed.listen((completed) {
        if (mounted && completed) {
          _player.seek(Duration.zero);
          _player.pause();
        }
      });

      // Open the media
      await _player.open(Media(url), play: false);
      
      if (mounted) setState(() => _isLoading = false);
    } catch (e) {
      debugPrint("Error loading audio: $e");
      if (mounted) setState(() => _isLoading = false);
    }
  }

  @override
  void dispose() {
    _player.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final versesAsync = ref.watch(surahDetailsProvider(widget.surah.number));
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(
        title: Column(
          children: [
            Text(widget.surah.nameTransliteration),
            Text(
              widget.surah.nameTranslation,
              style: theme.textTheme.labelSmall?.copyWith(
                color: theme.colorScheme.onSurface.withValues(alpha: 0.7),
              ),
            ),
          ],
        ),
        actions: [
          IconButton(
            icon: const Icon(Icons.settings_outlined),
            onPressed: () {},
          ),
        ],
      ),
      body: Column(
        children: [
          // Audio Player Bar
          _buildAudioPlayer(theme),
          
          Expanded(
            child: versesAsync.when(
              data: (verses) {
                return ListView.builder(
                  padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 24),
                  itemCount: verses.length + 1, // +1 for Bismillah
                  itemBuilder: (context, index) {
                    if (index == 0) {
                      // Bismillah Header (except for Surah 1 & 9 logic, simplified here)
                      return _buildBismillah(theme);
                    }
                    final verse = verses[index - 1];
                    return _buildVerseItem(verse, theme);
                  },
                );
              },
              loading: () => const Center(child: CircularProgressIndicator()),
              error: (err, stack) => Center(child: Text('Error: $err')),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildAudioPlayer(ThemeData theme) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      decoration: BoxDecoration(
        color: theme.colorScheme.surfaceContainerHighest.withValues(alpha: 0.5),
        border: Border(bottom: BorderSide(color: theme.dividerColor.withValues(alpha: 0.1))),
      ),
      child: Row(
        children: [
          if (_isLoading)
             const SizedBox(width: 24, height: 24, child: CircularProgressIndicator(strokeWidth: 2))
          else
            IconButton(
              icon: Icon(_isPlaying ? Icons.pause_circle_filled : Icons.play_circle_filled),
              color: theme.colorScheme.primary,
              iconSize: 40,
              onPressed: () {
                if (_isPlaying) {
                  _player.pause();
                } else {
                  _player.play();
                }
              },
            ),
          const SizedBox(width: 12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  "Mishary Rashid Alafasy",
                  style: theme.textTheme.labelMedium?.copyWith(fontWeight: FontWeight.bold),
                ),
                const SizedBox(height: 4),
                LinearProgressIndicator(
                  value: _duration.inSeconds > 0 ? _position.inSeconds / _duration.inSeconds : 0.0,
                  backgroundColor: theme.colorScheme.surfaceContainerHighest,
                  borderRadius: BorderRadius.circular(4),
                ),
              ],
            ),
          ),
          const SizedBox(width: 12),
          Text(
            _formatDuration(_duration - _position),
            style: theme.textTheme.labelSmall,
          ),
        ],
      ),
    );
  }

  Widget _buildBismillah(ThemeData theme) {
    // Basic Bismillah logic: Show for all except At-Tawbah (9). 
    // And Al-Fatihah (1) usually counts Bismillah as verse 1, so data might dupe it.
    // For MVP/Pro Demo, just showing a nice calligraphy image or text is fine.
    if (widget.surah.number == 9) return const SizedBox.shrink();

    return Container(
      margin: const EdgeInsets.only(bottom: 32),
      alignment: Alignment.center,
      child: Text(
        "بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ",
        style: GoogleFonts.amiri(
          fontSize: 24,
          color: theme.colorScheme.primary,
          height: 2,
        ),
      ),
    );
  }

  Widget _buildVerseItem(VerseWithTranslationDto verse, ThemeData theme) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 32.0),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // Action / Number Row
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                decoration: BoxDecoration(
                  color: theme.colorScheme.surfaceContainerHighest,
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Text(
                  "${widget.surah.number}:${verse.number}",
                  style: theme.textTheme.labelSmall?.copyWith(
                    color: theme.colorScheme.onSurfaceVariant,
                  ),
                ),
              ),
              Row(
                children: [
                  IconButton(icon: const Icon(Icons.play_arrow, size: 20), onPressed: () {}),
                  IconButton(icon: const Icon(Icons.bookmark_border, size: 20), onPressed: () {}),
                ],
              )
            ],
          ),
          const SizedBox(height: 16),
          // Arabic Text
          Directionality(
            textDirection: TextDirection.rtl,
            child: Text(
              verse.textUthmani,
              textAlign: TextAlign.justify,
              style: GoogleFonts.amiri(
                fontSize: 28,
                height: 2.2, // Generous line height for Quran
                color: theme.colorScheme.onSurface,
              ),
            ),
          ),
          const SizedBox(height: 16),
          // Translation
          Text(
            verse.translation ?? "Translation not available",
            textAlign: TextAlign.left,
            style: theme.textTheme.bodyLarge?.copyWith(
              color: theme.colorScheme.onSurface.withValues(alpha: 0.8),
              height: 1.6,
            ),
          ),
          const SizedBox(height: 8),
          Divider(color: theme.dividerColor.withValues(alpha: 0.1)),
        ],
      ),
    );
  }
  String _formatDuration(Duration d) {
    if (d.inHours > 0) {
      return "-${d.inHours}:${(d.inMinutes % 60).toString().padLeft(2, '0')}:${(d.inSeconds % 60).toString().padLeft(2, '0')}";
    }
    return "-${d.inMinutes}:${(d.inSeconds % 60).toString().padLeft(2, '0')}";
  }
}
