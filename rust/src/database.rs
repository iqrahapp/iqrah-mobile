use anyhow::Result;
use rusqlite::Connection;

/// Create schema with indexes and constraints
pub fn create_schema(conn: &Connection) -> Result<()> {
    // Nodes: Individual learning items (words/phrases)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS nodes (
            id TEXT PRIMARY KEY,
            content_type TEXT NOT NULL CHECK (content_type IN ('word','verse')),
            created_at INTEGER NOT NULL
        )",
        [],
    )?;

    // Node metadata: Arabic text, translations, etc.
    conn.execute(
        "CREATE TABLE IF NOT EXISTS node_metadata (
            node_id TEXT NOT NULL,
            key TEXT NOT NULL,
            value TEXT NOT NULL,
            PRIMARY KEY (node_id, key),
            FOREIGN KEY (node_id) REFERENCES nodes (id) ON DELETE CASCADE
        )",
        [],
    )?;

    // User memory states: FSRS scheduling data per user per node
    conn.execute(
        "CREATE TABLE IF NOT EXISTS user_memory_states (
            user_id TEXT NOT NULL,
            node_id TEXT NOT NULL,
            stability REAL NOT NULL DEFAULT 0,
            difficulty REAL NOT NULL DEFAULT 0,
            energy REAL NOT NULL DEFAULT 0.0,           -- mastery 0-1 scale
            last_reviewed INTEGER NOT NULL DEFAULT 0,   -- epoch ms
            due_at INTEGER NOT NULL DEFAULT 0,          -- epoch ms
            review_count INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (user_id, node_id),
            FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // High-impact indexes for due items queries
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_ums_user_due ON user_memory_states(user_id, due_at)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_ums_user_last ON user_memory_states(user_id, last_reviewed)",
        [],
    )?;

    Ok(())
}

/// Seed nodes and metadata tables (shared across all users)
fn seed_nodes_and_metadata(conn: &Connection) -> Result<()> {
    let tx = conn.unchecked_transaction()?;

    // Clear existing data (idempotent approach)
    tx.execute("DELETE FROM node_metadata", [])?;
    tx.execute("DELETE FROM nodes", [])?;

    let now_ms = chrono::Utc::now().timestamp_millis();

    // Surah Al-Fatiha word-by-word data
    let al_fatiha_data = vec![
        ("بِسْمِ", "In (the) name"),
        ("اللَّهِ", "of Allah"),
        ("الرَّحْمَٰنِ", "the Most Gracious"),
        ("الرَّحِيمِ", "the Most Merciful"),
        ("الْحَمْدُ", "All praise"),
        ("لِلَّهِ", "to Allah"),
        ("رَبِّ", "Lord"),
        ("الْعَالَمِينَ", "of the worlds"),
        ("الرَّحْمَٰنِ", "the Most Gracious"),
        ("الرَّحِيمِ", "the Most Merciful"),
        ("مَالِكِ", "Master"),
        ("يَوْمِ", "of (the) Day"),
        ("الدِّينِ", "of Judgment"),
        ("إِيَّاكَ", "You alone"),
        ("نَعْبُدُ", "we worship"),
        ("وَإِيَّاكَ", "and You alone"),
        ("نَسْتَعِينُ", "we ask for help"),
        ("اهْدِنَا", "Guide us"),
        ("الصِّرَاطَ", "to the path"),
        ("الْمُسْتَقِيمَ", "the straight"),
    ];

    {
        // Prepared statements for speed
        let mut insert_node =
            tx.prepare("INSERT INTO nodes (id, content_type, created_at) VALUES (?, ?, ?)")?;
        let mut insert_meta =
            tx.prepare("INSERT INTO node_metadata (node_id, key, value) VALUES (?, ?, ?)")?;

        for (i, (arabic, translation)) in al_fatiha_data.iter().enumerate() {
            let node_id = format!("fatiha_{:02}", i + 1);

            // Insert node (integers as integers, not strings)
            insert_node.execute([&node_id, "word", &now_ms.to_string()])?;

            // Insert metadata
            insert_meta.execute([&node_id, "arabic", arabic])?;
            insert_meta.execute([&node_id, "translation", translation])?;
        }
    }

    tx.commit()?;
    println!(
        "✅ Seeded nodes and metadata with {} Al-Fatiha words",
        al_fatiha_data.len()
    );
    Ok(())
}

/// Seed database for specific user (useful for testing)
pub fn seed_database_for_user(conn: &Connection, user_id: &str) -> Result<()> {
    let tx = conn.unchecked_transaction()?;

    // Clear existing data for this user
    tx.execute(
        "DELETE FROM user_memory_states WHERE user_id = ?",
        [user_id],
    )?;

    let now_ms = chrono::Utc::now().timestamp_millis();
    let past_ms = now_ms - (24 * 60 * 60 * 1000); // 24 hours ago

    // Al-Fatiha data (must match what's in nodes table)
    let al_fatiha_data = vec![
        ("بِسْمِ", "In (the) name"),
        ("اللَّهِ", "of Allah"),
        ("الرَّحْمَٰنِ", "the Most Gracious"),
        ("الرَّحِيمِ", "the Most Merciful"),
        ("الْحَمْدُ", "All praise"),
        ("لِلَّهِ", "to Allah"),
        ("رَبِّ", "Lord"),
        ("الْعَالَمِينَ", "of the worlds"),
        ("الرَّحْمَٰنِ", "the Most Gracious"),
        ("الرَّحِيمِ", "the Most Merciful"),
        ("مَالِكِ", "Master"),
        ("يَوْمِ", "of (the) Day"),
        ("الدِّينِ", "of Judgment"),
        ("إِيَّاكَ", "You alone"),
        ("نَعْبُدُ", "we worship"),
        ("وَإِيَّاكَ", "and You alone"),
        ("نَسْتَعِينُ", "we ask for help"),
        ("اهْدِنَا", "Guide us"),
        ("الصِّرَاطَ", "to the path"),
        ("الْمُسْتَقِيمَ", "the straight"),
    ];

    {
        // Only insert memory states for this user
        let mut insert_memory = tx.prepare(
        "INSERT OR REPLACE INTO user_memory_states (user_id, node_id, stability, difficulty, energy, last_reviewed, due_at, review_count)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )?;

        for (i, _) in al_fatiha_data.iter().enumerate() {
            let node_id = format!("fatiha_{:02}", i + 1);

            insert_memory.execute([
                user_id, // Use parameter instead of hardcoded
                &node_id,
                &0.0.to_string(),
                &0.0.to_string(),
                &0.0.to_string(),
                &0.to_string(),
                &past_ms.to_string(), // Due in the past
                &0.to_string(),
            ])?;
        }
    }

    tx.commit()?;
    println!(
        "✅ Seeded database for user '{}' with {} words",
        user_id,
        al_fatiha_data.len()
    );
    Ok(())
}

/// Fast, idempotent seeding with transaction and prepared statements
pub fn seed_database(conn: &Connection) -> Result<()> {
    // First ensure nodes/metadata exist
    seed_nodes_and_metadata(conn)?;
    // Then seed for default user
    seed_database_for_user(conn, "default_user")
}
