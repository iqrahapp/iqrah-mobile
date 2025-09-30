import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/rust_bridge/api/simple.dart' as rust_simple;
import 'package:iqrah/rust_bridge/api/types.dart' as rust_types;

enum PropagationLogWindow { allTime, lastDay, lastHour }

class PropagationLogNotifier
    extends AsyncNotifier<List<rust_types.PropagationDetailSummary>> {
  late rust_types.PropagationFilter _filter;
  late PropagationLogWindow _window;

  rust_types.PropagationFilter get filter => _filter;
  PropagationLogWindow get window => _window;

  @override
  Future<List<rust_types.PropagationDetailSummary>> build() async {
    _window = PropagationLogWindow.allTime;
    _filter = _filterForWindow(_window);
    return _fetchLog();
  }

  Future<void> refreshLog() async {
    state = const AsyncValue.loading();
    state = await AsyncValue.guard(_fetchLog);
  }

  Future<void> setWindow(PropagationLogWindow window) async {
    _window = window;
    _filter = _filterForWindow(window);
    await setFilter(_filter);
  }

  Future<void> setFilter(rust_types.PropagationFilter filter) async {
    _filter = filter;
    state = const AsyncValue.loading();
    state = await AsyncValue.guard(_fetchLog);
  }

  rust_types.PropagationFilter _filterForWindow(PropagationLogWindow window) {
    final nowSecs = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    switch (window) {
      case PropagationLogWindow.lastHour:
        return rust_types.PropagationFilter(
          startTimeSecs: nowSecs - 3600,
          endTimeSecs: null,
          limit: 100,
        );
      case PropagationLogWindow.lastDay:
        return rust_types.PropagationFilter(
          startTimeSecs: nowSecs - 86400,
          endTimeSecs: null,
          limit: 150,
        );
      case PropagationLogWindow.allTime:
        return rust_types.PropagationFilter(
          startTimeSecs: null,
          endTimeSecs: null,
          limit: 150,
        );
    }
  }

  Future<List<rust_types.PropagationDetailSummary>> _fetchLog() {
    return rust_simple.queryPropagationDetails(filter: _filter);
  }
}

final propagationLogProvider =
    AsyncNotifierProvider<
      PropagationLogNotifier,
      List<rust_types.PropagationDetailSummary>
    >(PropagationLogNotifier.new);
