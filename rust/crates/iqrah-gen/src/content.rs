use anyhow::{Context, Result};
use iqrah_core::domain::node_id as nid;
use rusqlite::{Connection, params};
use std::path::Path;

use crate::data_loader::{MorphologyData, QuranData, load_morphology_data, load_quran_data};

pub async fn build(data_dir: &Path, morphology: &Path, output_db: &Path) -> Result<()> {
    // 1. Initialize DB Schema using iqrah-storage
    // This ensures we use the exact same schema as the runtime application
    if output_db.exists() {
        std::fs::remove_file(output_db)?;
    }

    // Initialize schema using iqrah-storage (uses sqlx)
    let db_path_str = output_db.to_str().context("Invalid DB path")?;
    let pool = iqrah_storage::init_content_db(db_path_str).await?;
    pool.close().await; // Close connection to allow rusqlite to open it exclusively if needed

    // 2. Open with rusqlite for bulk population
    // We use rusqlite here for the generator because it was easier to set up for bulk inserts
    // in the initial implementation, but we could switch to sqlx later.
    let conn = Connection::open(output_db)?;

    // Enable optimizations
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA foreign_keys = ON;",
    )?;

    // 3. Load Data
    println!("Loading Quran data...");
    let quran_data = load_quran_data(data_dir)?;
    let morphology_data = load_morphology_data(morphology)?;

    println!("Populating content database...");
    populate_content(&conn, &quran_data, &morphology_data)?;

    println!("Content database created at {:?}", output_db);

    Ok(())
}

fn populate_content(
    conn: &Connection,
    quran: &QuranData,
    morphology: &MorphologyData,
) -> Result<()> {
    // Use a transaction for performance
    conn.execute_batch("BEGIN TRANSACTION;")?;

    // First, we need to register nodes in the nodes table
    let mut node_stmt =
        conn.prepare("INSERT OR REPLACE INTO nodes (id, ukey, node_type) VALUES (?1, ?2, ?3)")?;

    // Insert Chapters (node_type = 1 for Chapter)
    let mut chapter_stmt = conn.prepare(
        "INSERT OR REPLACE INTO chapters (chapter_number, name_arabic, name_transliteration, name_translation, revelation_place, verse_count)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
    )?;

    for chapter in &quran.chapters {
        let id = nid::encode_chapter(chapter.number as u8);
        let ukey = nid::chapter(chapter.number as u8);

        // Insert into nodes table
        node_stmt.execute(params![id, ukey, 1])?; // 1 = Chapter

        // Insert into chapters table
        chapter_stmt.execute(params![
            chapter.number,
            chapter.name_arabic,
            chapter.name_transliteration,
            chapter.name_translation,
            chapter.revelation_place,
            chapter.verse_count,
        ])?;
    }
    println!("  Inserted {} chapters", quran.chapters.len());

    // Insert Verses (node_type = 2 for Verse)
    let mut verse_stmt = conn.prepare(
        "INSERT OR REPLACE INTO verses (verse_key, chapter_number, verse_number, juz, hizb, rub_el_hizb, page, manzil)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
    )?;

    for verse in &quran.verses {
        let id = nid::encode_verse(verse.chapter_number as u8, verse.verse_number as u16);
        let ukey = verse.key.clone();

        // Insert into nodes table
        node_stmt.execute(params![id, ukey, 2])?; // 2 = Verse

        // Insert into verses table (using placeholder values for missing fields)
        verse_stmt.execute(params![
            verse.key,
            verse.chapter_number,
            verse.verse_number,
            verse.juz,  // juz (placeholder)
            0,          // hizb (placeholder)
            0,          // rub_el_hizb (placeholder)
            verse.page, // page
            0,          // manzil (placeholder)
        ])?;
    }
    println!("  Inserted {} verses", quran.verses.len());

    // Create script resource for Uthmani text
    conn.execute(
        "INSERT OR IGNORE INTO script_resources (slug, name, type, direction) VALUES ('uthmani', 'Uthmani', 1, 'rtl')",
        [],
    )?;
    let uthmani_resource_id: i64 = conn.query_row(
        "SELECT resource_id FROM script_resources WHERE slug = 'uthmani'",
        [],
        |row| row.get(0),
    )?;

    // Insert verse text content
    let mut content_stmt = conn.prepare(
        "INSERT OR REPLACE INTO script_contents (resource_id, node_id, text_content) VALUES (?1, ?2, ?3)",
    )?;

    for verse in &quran.verses {
        let id = nid::encode_verse(verse.chapter_number as u8, verse.verse_number as u16);
        content_stmt.execute(params![uthmani_resource_id, id, verse.text_uthmani])?;
    }
    println!("  Inserted {} verse texts", quran.verses.len());

    // Insert Words (node_type = 4 for WordInstance)
    let mut word_stmt = conn.prepare(
        "INSERT OR REPLACE INTO words (word_id, verse_key, position) VALUES (?1, ?2, ?3)",
    )?;

    for word in &quran.words {
        let parts: Vec<&str> = word.verse_key.split(':').collect();
        if parts.len() < 2 {
            continue;
        }
        let ch_num: u8 = parts[0].parse().unwrap_or(0);
        let v_num: u16 = parts[1].parse().unwrap_or(0);

        let id = nid::encode_word_instance(ch_num, v_num, word.position as u8);
        let ukey = nid::word_instance(ch_num, v_num, word.position as u8);

        // Insert into nodes table
        node_stmt.execute(params![id, ukey, 4])?; // 4 = WordInstance

        // Insert into words table
        word_stmt.execute(params![id, word.verse_key, word.position])?;

        // Insert word text content
        content_stmt.execute(params![uthmani_resource_id, id, word.text_uthmani])?;
    }
    println!("  Inserted {} words", quran.words.len());

    // Insert Morphology (Lemmas and Roots)
    let mut lemmas: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut roots: std::collections::HashSet<String> = std::collections::HashSet::new();

    for segment in &morphology.segments {
        if let Some(ref lemma) = segment.lemma {
            lemmas.insert(lemma.clone());
        }
        if let Some(ref root) = segment.root {
            roots.insert(root.clone());
        }
    }

    // Insert Roots
    let mut root_stmt =
        conn.prepare("INSERT OR REPLACE INTO roots (root_id, arabic) VALUES (?1, ?2)")?;
    for root in &roots {
        let id = nid::encode_root(root);
        let ukey = nid::root(root);
        // Insert into nodes table (node_type = 6 for Root)
        let _ = node_stmt.execute(params![id, ukey, 6]);
        root_stmt.execute(params![ukey, root])?;
    }
    println!("  Inserted {} roots", roots.len());

    // Insert Lemmas
    let mut lemma_stmt =
        conn.prepare("INSERT OR REPLACE INTO lemmas (lemma_id, arabic) VALUES (?1, ?2)")?;
    for lemma in &lemmas {
        let id = nid::encode_lemma(lemma);
        let ukey = nid::lemma(lemma);
        // Insert into nodes table (node_type = 5 for Lemma)
        let _ = node_stmt.execute(params![id, ukey, 5]);
        lemma_stmt.execute(params![ukey, lemma])?;
    }
    println!("  Inserted {} lemmas", lemmas.len());

    conn.execute_batch("COMMIT;")?;

    Ok(())
}
