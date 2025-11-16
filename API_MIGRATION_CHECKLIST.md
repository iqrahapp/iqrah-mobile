# API Migration Checklist

## Current API Functions in rust/src/api/mod.rs

### Setup & Initialization
- [ ] `setup_database(db_path, kg_bytes)` - Initialize DB and import CBOR
- [ ] `setup_database_in_memory(kg_bytes)` - In-memory setup
- [x] `init_app()` - Logging setup (FRB init)

### Exercise Generation
- [ ] `get_exercises(user_id, limit, surah_filter, is_high_yield)` - Generate exercises
- [ ] `get_exercises_for_node(node_id)` - Generate for specific node

### Reviews & Learning
- [x] `process_review(user_id, node_id, grade)` - Process review ✅ (as process_review_async)
- [ ] `refresh_priority_scores(user_id)` - Recalculate priorities

### Session Management
- [ ] `get_session_preview(user_id, limit, surah_filter, is_high_yield)` - Preview session
- [ ] `get_existing_session()` - Get saved session
- [x] `clear_session()` - Clear session ✅ (as clear_session_async)

### Stats & Dashboard
- [x] `get_dashboard_stats(user_id)` - Dashboard data ✅ (as get_stats_async)
- [ ] `get_debug_stats(user_id)` - Debug stats
- [x] `get_due_count(user_id)` - Count due items ✅ (as get_due_count_async)

### Data Access
- [ ] `search_nodes(query, limit)` - Search nodes by prefix
- [ ] `fetch_node_with_metadata(node_id)` - Get node details
- [ ] `get_available_surahs()` - List available surahs

### Database Operations
- [ ] `reseed_database()` - Reset user progress

## New API Functions in iqrah-api/src/api.rs

Already implemented:
- ✅ `init_app_async(content_db, user_db)`
- ✅ `process_review_async(user_id, node_id, grade)`
- ✅ `get_due_items_async(user_id, limit, is_high_yield)`
- ✅ `get_stats_async()`
- ✅ `get_due_count_async(user_id)`
- ✅ `clear_session_async()`

Need to add:
- All remaining functions from old API
- CBOR import functionality
- Exercise generation
- Node search/fetch
