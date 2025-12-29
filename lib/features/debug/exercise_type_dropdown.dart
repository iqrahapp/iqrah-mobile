import 'package:flutter/material.dart';

class ExerciseTypeDropdown extends StatelessWidget {
  final List<String> types;
  final String? selected;
  final ValueChanged<String?> onSelect;

  const ExerciseTypeDropdown({
    super.key,
    required this.types,
    required this.selected,
    required this.onSelect,
  });

  @override
  Widget build(BuildContext context) {
    return DropdownButtonFormField<String>(
      key: ValueKey(selected),
      initialValue: selected,
      items: types
          .map(
            (type) => DropdownMenuItem<String>(
              value: type,
              child: Text(type),
            ),
          )
          .toList(),
      decoration: const InputDecoration(
        labelText: 'Exercise type',
        border: OutlineInputBorder(),
      ),
      onChanged: types.isEmpty ? null : onSelect,
    );
  }
}
