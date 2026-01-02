import 'package:iqrah/rust_bridge/api.dart' as api;

class SessionService {
  Future<api.SessionDto> startSession({
    required String userId,
    required String goalId,
  }) {
    return api.startSession(userId: userId, goalId: goalId);
  }

  Future<api.SessionDto?> getActiveSession(String userId) {
    return api.getActiveSession(userId: userId);
  }

  Future<api.SessionItemDto?> getNextItem(String sessionId) {
    return api.getNextSessionItem(sessionId: sessionId);
  }

  Future<void> submitItem({
    required String sessionId,
    required String nodeId,
    required String exerciseType,
    required int grade,
    required int durationMs,
  }) async {
    await api.submitSessionItem(
      sessionId: sessionId,
      nodeId: nodeId,
      exerciseType: exerciseType,
      grade: grade,
      durationMs: BigInt.from(durationMs),
    );
  }

  Future<api.SessionSummaryDto> completeSession(String sessionId) {
    return api.completeSession(sessionId: sessionId);
  }
}
