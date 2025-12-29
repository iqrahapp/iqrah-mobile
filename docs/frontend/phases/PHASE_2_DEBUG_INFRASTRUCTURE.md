# Phase 2: Debug Infrastructure

Document Version: 1.0
Date: 2024-12-28

## Purpose
Provide a dedicated debug toolset to quickly launch exercises, inspect energy and propagation, and run DB queries in debug builds. This phase is essential for fast iteration and QA on new exercise renderers.

## Goals
- Exercise Debug Screen to launch any exercise with one tap.
- Node selector with search + range parsing.
- Energy propagation monitor and snapshot visualization.
- DB inspector (debug-only SQL execution).
- Categorized logging framework.

## Dependencies
- Phase 1 completed (FFI content access + service layer).

## Acceptance Criteria
- Any exercise can be launched within 30 seconds from the debug screen.
- Energy snapshot shows node + neighbor energies.
- Propagation simulation shows before/after deltas.
- SQL queries can be executed in debug builds only.
- Logs are categorized (exercise, energy, session, ffi, db).

## Task Breakdown

### Task 2.1: Exercise Debug Screen
Create a debug UI that lets the user pick a node and exercise type, then launch the renderer.

Files to add:
- `lib/features/debug/exercise_debug_screen.dart`
- `lib/features/debug/node_selector_widget.dart`
- `lib/features/debug/exercise_type_dropdown.dart`

Dart skeleton:
```dart
class ExerciseDebugScreen extends StatefulWidget {
  @override
  State<ExerciseDebugScreen> createState() => _ExerciseDebugScreenState();
}

class _ExerciseDebugScreenState extends State<ExerciseDebugScreen> {
  String? selectedNodeId;
  String? selectedExerciseType;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Exercise Debugger')),
      body: Column(
        children: [
          NodeSelectorWidget(onSelect: (nodeId) => setState(() => selectedNodeId = nodeId)),
          ExerciseTypeDropdown(onSelect: (type) => setState(() => selectedExerciseType = type)),
          ElevatedButton(
            onPressed: (selectedNodeId != null && selectedExerciseType != null)
                ? _launch
                : null,
            child: const Text('Launch'),
          ),
        ],
      ),
    );
  }

  Future<void> _launch() async {
    // Use existing ExerciseDataDto generator or new debug API
  }
}
```

### Task 2.2: Node Selector + Range Parser
Add a filterable node query API in Rust, and a parser for ranges like `1:1-7`.

Files to modify:
- `rust/crates/iqrah-api/src/api.rs`
- `lib/features/debug/node_selector_widget.dart`

Rust signatures:
```rust
pub async fn query_nodes(filter: NodeFilterDto) -> Result<Vec<NodeSearchDto>>;

pub fn parse_node_range(range: String) -> Result<Vec<String>>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeFilterDto {
    pub node_type: Option<String>,
    pub min_energy: Option<f64>,
    pub max_energy: Option<f64>,
    pub range: Option<String>,
}
```

### Task 2.3: Energy Snapshot + Propagation
Expose read APIs for energy snapshot and propagation simulation.

Files to modify:
- `rust/crates/iqrah-api/src/api.rs`
- `lib/features/debug/energy_monitor_screen.dart`

Rust signatures:
```rust
pub async fn get_energy_snapshot(user_id: String, node_id: String) -> Result<EnergySnapshotDto>;

pub async fn simulate_propagation(
    user_id: String,
    node_id: String,
    energy_delta: f64,
) -> Result<PropagationResultDto>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnergySnapshotDto {
    pub node_id: String,
    pub energy: f64,
    pub neighbors: Vec<NodeEnergyDto>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeEnergyDto {
    pub node_id: String,
    pub energy: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PropagationResultDto {
    pub before: Vec<NodeEnergyDto>,
    pub after: Vec<NodeEnergyDto>,
}
```

### Task 2.4: DB Inspector (Debug-only)
Expose a debug-only SQL executor and render results in a table.

Files to modify:
- `rust/crates/iqrah-api/src/api.rs`
- `lib/features/debug/db_inspector_screen.dart`

Rust signature (debug-only):
```rust
#[cfg(debug_assertions)]
pub async fn execute_debug_query(sql: String) -> Result<Vec<HashMap<String, String>>>;
```

### Task 2.5: Logging Framework
Create a single logger utility with categories.

Files to add:
- `lib/utils/app_logger.dart`

Dart skeleton:
```dart
enum LogCategory { exercise, energy, session, ffi, db, ui }

class AppLogger {
  static void log(LogCategory category, String message) {
    // Format and print
  }
}
```

## Testing Requirements
- Manual QA: Debug screen can launch any exercise in <30 seconds.
- Unit test: node range parsing.
- Widget test: debug screens render with mock data.

## Estimated Effort
- 5 to 7 days.

## Deliverables
- Debug screens and widgets.
- Energy snapshot + propagation APIs.
- DB inspector (debug-only).
- Logger utility.
