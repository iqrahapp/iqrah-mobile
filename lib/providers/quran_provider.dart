import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/rust_bridge/api.dart';

final surahsProvider = FutureProvider<List<SurahInfo>>((ref) async {
  return await getAvailableSurahs();
});
