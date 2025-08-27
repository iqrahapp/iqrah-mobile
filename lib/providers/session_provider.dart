import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/rust_bridge/api.dart' as api;
import 'package:iqrah/rust_bridge/repository.dart';

class SessionState {
  final List<NodeData> items;
  final int currentIndex;

  SessionState({this.items = const [], this.currentIndex = 0});

  SessionState copyWith({List<NodeData>? items, int? currentIndex}) {
    return SessionState(
      items: items ?? this.items,
      currentIndex: currentIndex ?? this.currentIndex,
    );
  }

  NodeData? get currentItem {
    if (items.isEmpty || currentIndex >= items.length) return null;
    return items[currentIndex];
  }

  bool isCompleted() {
    return items.isNotEmpty && currentIndex >= items.length;
  }
}

// 2. Create the Notifier to manage the state
class SessionNotifier extends Notifier<SessionState> {
  @override
  SessionState build() {
    return SessionState();
  }

  // The UI will call this to start a review with the items it got
  // from our FutureProvider.
  void startReview(List<NodeData> dueItems) {
    state = state.copyWith(items: dueItems, currentIndex: 0);
  }

  Future<void> submitReview(ReviewGrade grade) async {
    final item = state.currentItem;
    if (item == null) return;

    try {
      await api.processReview(
        userId: "default_user",
        nodeId: item.id,
        grade: grade,
      );
      state = state.copyWith(currentIndex: state.currentIndex + 1);
    } catch (e) {
      print("❌ Failed to submit review for item ${item.id}: $e");
    }
  }
}

final sessionProvider = NotifierProvider<SessionNotifier, SessionState>(
  SessionNotifier.new,
);
