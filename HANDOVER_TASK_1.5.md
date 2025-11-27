# Handover Report: Task 1.5 - Node ID Stability & Knowledge Node Decoding

## Status Summary
- **Task 1.5** (Node ID Stability) is **INCOMPLETE**.
- **Current State:** The `i64` Node ID system works for `Chapter`, `Verse`, `Word`, and `WordInstance`. However, `Knowledge` nodes cannot be decoded back to their components, meaning `to_ukey(id)` returns `None` for them.
- **Blocker:** The current `encode_knowledge` implementation strips the "type" bits from the base node ID to fit it into the payload, but it fails to store that type anywhere. Consequently, the decoder has the raw payload (e.g., chapter/verse numbers) but doesn't know if it belongs to a Verse, Word, or WordInstance.

## The Problem
In `rust/crates/iqrah-core/src/domain/node_id.rs`:

```rust
pub fn encode_knowledge(base_id: i64, axis: KnowledgeAxis) -> i64 {
    // ... maps axis to 1-6 ...

    // PROBLEM: This mask strips the top 8 bits (the type) from base_id.
    // The type is lost forever.
    (TYPE_KNOWLEDGE << TYPE_SHIFT) | (axis_id << 48) | (base_id & 0xFFFFFFFFFFFF)
}
```

When decoding:
1. We see `TYPE_KNOWLEDGE` in the top 8 bits.
2. We extract `axis_id` from bits 48-55.
3. We extract `payload` from bits 0-47.
4. **Missing Info:** We don't know if `payload` represents a `VERSE` structure, a `WORD` structure, etc.

## Proposed Solution
We have 8 bits available in the "metadata" section of the ID (bits 48-55), currently occupied wholly by `axis_id`.
Since we only have ~6 axes and ~4 base node types, we can pack both into this byte.

**New Bit Layout for Knowledge Nodes:**
- **Bits 56-63 (8 bits):** `TYPE_KNOWLEDGE` (0x05)
- **Bits 52-55 (4 bits):** **Base Node Type** (e.g., `TYPE_VERSE`=2, `TYPE_WORD_INSTANCE`=4)
- **Bits 48-51 (4 bits):** **Knowledge Axis** (1-15)
- **Bits 0-47 (48 bits):** Base Node Payload (The `base_id` without its type prefix)

## Action Plan for Next Agent

1. **Modify `encode_knowledge` in `rust/crates/iqrah-core/src/domain/node_id.rs`:**
   - Extract the type from `base_id` before stripping it: `let base_type = (base_id >> 56) & 0xFF;`
   - Ensure `base_type` fits in 4 bits (it should, types are small integers).
   - Ensure `axis_id` fits in 4 bits.
   - Pack them: `(TYPE_KNOWLEDGE << 56) | (base_type << 52) | (axis_id << 48) | (base_id & 0xFFFFFFFFFFFF)`

2. **Implement `decode_knowledge(id: i64) -> Option<(i64, KnowledgeAxis)>`:**
   - Extract `base_type` from bits 52-55.
   - Extract `axis_id` from bits 48-51.
   - Extract `payload` from bits 0-47.
   - Reconstruct `base_id`: `(base_type << 56) | payload`.
   - Map `axis_id` back to `KnowledgeAxis`.

3. **Update `to_ukey` and `from_ukey`:**
   - Implement the `NodeType::Knowledge` branch in `to_ukey` using `decode_knowledge`.
   - Implement the `NodeType::Knowledge` branch in `from_ukey` (already mostly done, just ensure it uses `encode_knowledge`).

4. **Add Stability Tests:**
   - Create a new test file or update `rust/tests/knowledge_axis_test.rs`.
   - Verify round-trip: `String UKey -> i64 -> String UKey`.
   - Verify decomposition: `i64 -> (Base ID i64, Axis)`.

## Relevant Files
- `rust/crates/iqrah-core/src/domain/node_id.rs`: Core logic to update.
- `rust/tests/knowledge_axis_test.rs`: Test file to expand.

## Expected Outcome
- `cargo test --package iqrah-core --lib domain::node_id` passes all tests.
- `cargo test --test knowledge_axis_test` passes and includes `i64` round-trip checks.
- Task 1.5 can then be marked COMPLETE.
