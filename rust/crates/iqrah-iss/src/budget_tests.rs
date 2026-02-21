//! M2.2 Budget Enforcement Unit Tests
//!
//! Tests for hard intro_budget reservation in session composition.

/// Simulates budget-enforced session selection.
/// This mirrors the logic in simulator.rs simulate_day.
fn select_session_with_budget(
    session_size: usize,
    intro_budget: usize,
    new_items_limit: usize,
    due_candidates_count: usize,
    new_candidates_count: usize,
) -> (usize, usize, usize, usize) {
    // Compute effective intro cap
    let intro_cap = intro_budget.min(new_items_limit);

    // Compute budgets with hard reservation for intro
    let actual_intro_budget = intro_cap.min(session_size);
    let due_budget = session_size.saturating_sub(actual_intro_budget);

    // Step 1: Select new items up to intro_cap (hard reservation)
    let new_selected = intro_cap.min(new_candidates_count);

    // Step 2: Select due items up to due_budget
    let due_selected = due_budget.min(due_candidates_count);

    // Step 3: Compute spillover (only to due, NOT to new)
    let unused_intro_slots = intro_cap.saturating_sub(new_selected);
    let spill_to_due = unused_intro_slots.min(due_candidates_count.saturating_sub(due_selected));

    // No spill to new (M2.2 hard cap)
    let spill_to_new = 0;

    // Final totals
    let final_new_selected = new_selected; // No additional_new
    let final_due_selected = due_selected + spill_to_due;

    (
        final_new_selected,
        final_due_selected,
        spill_to_due,
        spill_to_new,
    )
}

#[test]
fn test_intro_reservation_under_backlog() {
    // Scenario: Due backlog is huge, but intro_budget should still be respected
    let session_size = 20;
    let intro_budget = 5;
    let new_items_limit = 10;
    let due_candidates_count = 100; // Big backlog
    let new_candidates_count = 100; // Plenty of new items

    let (new_selected, due_selected, spill_to_due, spill_to_new) = select_session_with_budget(
        session_size,
        intro_budget,
        new_items_limit,
        due_candidates_count,
        new_candidates_count,
    );

    assert_eq!(
        new_selected, 5,
        "Should select exactly intro_budget new items"
    );
    assert_eq!(due_selected, 15, "Should fill remaining with due items");
    assert_eq!(spill_to_due, 0, "No unused intro slots to spill");
    assert_eq!(spill_to_new, 0, "Spill to new is disabled (hard cap)");
}

#[test]
fn test_spillover_when_no_new_candidates() {
    // Scenario: No new candidates available, unused intro slots spill to due
    let session_size = 20;
    let intro_budget = 5;
    let new_items_limit = 10;
    let due_candidates_count = 100;
    let new_candidates_count = 0; // No new candidates!

    let (new_selected, due_selected, spill_to_due, spill_to_new) = select_session_with_budget(
        session_size,
        intro_budget,
        new_items_limit,
        due_candidates_count,
        new_candidates_count,
    );

    assert_eq!(new_selected, 0, "No new items to select");
    assert_eq!(
        due_selected, 20,
        "Should fill entire session with due items"
    );
    assert_eq!(spill_to_due, 5, "All 5 unused intro slots spill to due");
    assert_eq!(spill_to_new, 0, "Spill to new is disabled");
}

#[test]
fn test_intro_cap_respects_limit() {
    // Scenario: new_items_limit < intro_budget (gate limits)
    let session_size = 20;
    let intro_budget = 10;
    let new_items_limit = 3; // Gate limits to 3
    let due_candidates_count = 50;
    let new_candidates_count = 100;

    let (new_selected, due_selected, _, _) = select_session_with_budget(
        session_size,
        intro_budget,
        new_items_limit,
        due_candidates_count,
        new_candidates_count,
    );

    assert_eq!(
        new_selected, 3,
        "intro_cap should be min(intro_budget, new_items_limit)"
    );
    assert_eq!(due_selected, 17, "Remaining capacity to due");
}

#[test]
fn test_no_spill_to_new_even_when_due_deficit() {
    // Critical test: Even if due candidates are few, do NOT add more new items
    let session_size = 20;
    let intro_budget = 5;
    let new_items_limit = 10;
    let due_candidates_count = 3; // Very few due candidates!
    let new_candidates_count = 100;

    let (new_selected, due_selected, spill_to_due, spill_to_new) = select_session_with_budget(
        session_size,
        intro_budget,
        new_items_limit,
        due_candidates_count,
        new_candidates_count,
    );

    // intro_cap = min(5, 10) = 5
    // new_selected = min(5, 100) = 5
    // due_budget = 20 - 5 = 15
    // due_selected = min(15, 3) = 3
    // spill_to_due = 0 (no unused intro slots since new_selected = intro_cap)
    // spill_to_new = 0 (DISABLED)
    // Total session = 5 + 3 = 8 items (not 20!)

    assert_eq!(new_selected, 5, "Should select full intro_cap");
    assert_eq!(due_selected, 3, "Only 3 due candidates available");
    assert_eq!(spill_to_due, 0, "No unused intro slots");
    assert_eq!(spill_to_new, 0, "M2.2: No spill to new (hard cap enforced)");

    // Session may be smaller than session_size if both pools are depleted
    let total = new_selected + due_selected;
    assert!(total <= session_size, "Session should not exceed size");
}
