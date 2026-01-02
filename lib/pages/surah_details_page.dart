import 'package:flutter/material.dart';
import 'package:iqrah/rust_bridge/api.dart';

class SurahDetailsPage extends StatelessWidget {
  final SurahInfo surah;

  const SurahDetailsPage({super.key, required this.surah});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text(surah.transliteration ?? 'Surah ${surah.surahNumber}')),
      body: const Center(child: Text("Details Coming Soon")),
    );
  }
}
