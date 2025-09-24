import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/providers/due_items_provider.dart';

class SurahDropdown extends ConsumerWidget {
  const SurahDropdown({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final availableSurahs = ref.watch(availableSurahsProvider);
    final selectedSurah = ref.watch(surahFilterProvider);

    return availableSurahs.when(
      loading: () =>
          const SizedBox(width: 200, child: LinearProgressIndicator()),
      error: (error, _) => Container(
        width: 200,
        padding: const EdgeInsets.all(8),
        child: Text(
          'Error loading Sūrahs: $error',
          style: TextStyle(color: Colors.red[300]),
        ),
      ),
      data: (surahs) {
        return Container(
          width: 250,
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(8),
            border: Border.all(color: Colors.grey[600]!),
            color: Colors.grey[850],
          ),
          child: DropdownButtonHideUnderline(
            child: DropdownButton<int?>(
              value: selectedSurah,
              isExpanded: true,
              hint: const Text('Select Sūrah'),
              style: const TextStyle(color: Colors.white),
              dropdownColor: Colors.grey[800],
              items: [
                const DropdownMenuItem<int?>(
                  value: null,
                  child: Row(
                    children: [
                      Icon(Icons.all_inclusive, size: 16, color: Colors.blue),
                      SizedBox(width: 8),
                      Text(
                        'All Sūrahs',
                        style: TextStyle(fontWeight: FontWeight.w500),
                      ),
                    ],
                  ),
                ),
                ...surahs.map(
                  (surah) => DropdownMenuItem<int?>(
                    value: surah.number,
                    child: Row(
                      children: [
                        Container(
                          width: 24,
                          height: 24,
                          decoration: BoxDecoration(
                            color: Colors.green[700],
                            borderRadius: BorderRadius.circular(4),
                          ),
                          child: Center(
                            child: Text(
                              '${surah.number}',
                              style: const TextStyle(
                                fontSize: 12,
                                fontWeight: FontWeight.bold,
                                color: Colors.white,
                              ),
                            ),
                          ),
                        ),
                        const SizedBox(width: 8),
                        Expanded(
                          child: Text(
                            surah.name,
                            style: const TextStyle(fontSize: 14),
                            overflow: TextOverflow.ellipsis,
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
              ],
              onChanged: (value) {
                ref.read(surahFilterProvider.notifier).state = value;
              },
            ),
          ),
        );
      },
    );
  }
}
