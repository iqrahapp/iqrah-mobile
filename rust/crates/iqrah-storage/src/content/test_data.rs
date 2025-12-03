use crate::Result;
use sqlx::SqlitePool;

/// Seeds the sample data used by integration tests.
///
/// **Data Seeded:**
/// - 3 chapters (Al-Fatihah, Al-Baqarah, Al-Imran)
/// - 493 verses (7 + 286 + 200)
/// - 4 words for verse 1:1
/// - 2 script resources (uthmani, simple)
/// - Text content for all verses and words
/// - 7 languages (English, Arabic, French, Urdu, Indonesian, Turkish, Spanish)
/// - 5 translators (Sahih International, Yusuf Ali, Pickthall, Khattab, Hilali-Khan)
/// - Verse translations for 1:1 from all translators
/// - Word-by-word translations for Sahih International
/// - Test goal: "memorization:chapters-1-3"
/// - Node metadata and edges for scheduler
///
/// **Order of Operations:**
/// 1. Chapters
/// 2. Verses (Al-Fatihah full, chapters 2-3 placeholders)
/// 3. Words (verse 1:1 only)
/// 4. Nodes (populate registry for verses and words)
/// 5. Script Resources (uthmani, simple, transliteration)
/// 6. Script Contents (link resources to nodes)
/// 7. Languages and Translators
/// 8. Translations (verse and word)
/// 9. Goal and node_goals
/// 10. Node metadata and edges
pub async fn seed_sample_data(pool: &SqlitePool) -> Result<()> {
    // Step 1: Insert Chapters
    // Clear tables to prevent duplicates if DB is reused
    sqlx::query("DELETE FROM word_translations")
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM verse_translations")
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM words").execute(pool).await.ok();
    sqlx::query("DELETE FROM verses").execute(pool).await.ok();
    sqlx::query("DELETE FROM chapters").execute(pool).await.ok();

    sqlx::query(
        "INSERT OR IGNORE INTO chapters (chapter_number, name_arabic, name_transliteration, name_translation,
            revelation_place, revelation_order, bismillah_pre, verse_count, page_start, page_end)
            VALUES
            (1, 'الفاتحة', 'Al-Fatihah', 'The Opening', 'makkah', 5, 1, 7, 1, 1),
            (2, 'البقرة', 'Al-Baqarah', 'The Cow', 'madinah', 87, 1, 286, 2, 49),
            (3, 'آل عمران', 'Al-Imran', 'The Family of Imran', 'madinah', 89, 1, 200, 50, 76)",
    )
    .execute(pool)
    .await?;

    // Step 2: Insert Verses (Al-Fatihah with full details)
    sqlx::query(
        "INSERT OR IGNORE INTO verses (verse_key, chapter_number, verse_number, juz, hizb, rub_el_hizb, page, manzil, word_count)
            VALUES
            ('1:1', 1, 1, 1, 1, 1, 1, 1, 4),
            ('1:2', 1, 2, 1, 1, 1, 1, 1, 4),
            ('1:3', 1, 3, 1, 1, 1, 1, 1, 2),
            ('1:4', 1, 4, 1, 1, 1, 1, 1, 3),
            ('1:5', 1, 5, 1, 1, 1, 1, 1, 4),
            ('1:6', 1, 6, 1, 1, 1, 1, 1, 3),
            ('1:7', 1, 7, 1, 1, 1, 1, 1, 10)"
    )
    .execute(pool)
    .await?;

    // Generate placeholder verses for chapters 2-3 using CTE
    sqlx::query(
        "WITH RECURSIVE chapter_verses(chapter_num, verse_num, max_verses) AS (
            SELECT 2, 1, 286
            UNION ALL
            SELECT
                CASE WHEN verse_num < max_verses THEN chapter_num ELSE chapter_num + 1 END,
                CASE WHEN verse_num < max_verses THEN verse_num + 1 ELSE 1 END,
                CASE WHEN verse_num < max_verses THEN max_verses WHEN chapter_num = 2 THEN 200 ELSE 0 END
            FROM chapter_verses
            WHERE chapter_num < 3 OR (chapter_num = 3 AND verse_num < 200)
        )
        INSERT OR IGNORE INTO verses (verse_key, chapter_number, verse_number, juz, hizb, rub_el_hizb, page, manzil, word_count)
        SELECT chapter_num || ':' || verse_num, chapter_num, verse_num,
                CASE WHEN chapter_num = 2 THEN (verse_num / 25) + 1 ELSE 15 + (verse_num / 20) END,
                1, 1, 1, 1, 1
        FROM chapter_verses"
    )
    .execute(pool)
    .await?;

    // Step 3: Insert Words (verse 1:1 only)
    sqlx::query("DELETE FROM words WHERE verse_key = '1:1'")
        .execute(pool)
        .await?;
    sqlx::query(
        "INSERT OR IGNORE INTO words (verse_key, position, letter_count) VALUES
            ('1:1', 1, 3),
            ('1:1', 2, 4),
            ('1:1', 3, 6),
            ('1:1', 4, 6)",
    )
    .execute(pool)
    .await?;

    // Step 4: Populate Nodes Registry
    // Encode verse nodes: (TYPE_VERSE << 56) | (chapter << 16) | verse
    sqlx::query(
        "INSERT OR IGNORE INTO nodes (id, ukey, node_type)
            SELECT (CAST(2 AS INTEGER) << 56) | (chapter_number << 16) | verse_number,
                'VERSE:' || verse_key,
                1
            FROM verses",
    )
    .execute(pool)
    .await?;

    // Encode word nodes: (TYPE_WORD << 56) | word_id
    sqlx::query(
        "INSERT OR IGNORE INTO nodes (id, ukey, node_type)
            SELECT (CAST(3 AS INTEGER) << 56) | word_id,
                'WORD:' || word_id,
                3
            FROM words",
    )
    .execute(pool)
    .await?;

    // Step 5: Insert Script Resources
    let uthmani_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO script_resources (slug, name, type, direction, description)
            VALUES ('uthmani', 'Uthmani Script', 1, 'rtl', 'Standard Uthmani Quranic text')
            RETURNING resource_id",
    )
    .fetch_one(pool)
    .await?;

    let simple_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO script_resources (slug, name, type, direction, description)
            VALUES ('simple', 'Simple Script', 1, 'rtl', 'Simplified Quranic text without diacritics')
            RETURNING resource_id"
    )
    .fetch_one(pool)
    .await?;

    let translit_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO script_resources (slug, name, type, direction, description)
            VALUES ('transliteration', 'Transliteration', 1, 'ltr', 'Romanized pronunciation guide')
            RETURNING resource_id",
    )
    .fetch_one(pool)
    .await?;

    // Step 6: Insert Script Contents (Uthmani text for verses)
    // For verse 1:1 through 1:7
    let verse_texts = [
        ("1:1", "بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ", "بسم الله الرحمن الرحيم"),
        ("1:2", "ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ", "الحمد لله رب العالمين"),
        ("1:3", "ٱلرَّحْمَٰنِ ٱلرَّحِيمِ", "الرحمن الرحيم"),
        ("1:4", "مَٰلِكِ يَوْمِ ٱلدِّينِ", "مالك يوم الدين"),
        ("1:5", "إِيَّاكَ نَعْبُدُ وَإِيَّاكَ نَسْتَعِينُ", "اياك نعبد واياك نستعين"),
        ("1:6", "ٱهْدِنَا ٱلصِّرَٰطَ ٱلْمُسْتَقِيمَ", "اهدنا الصراط المستقيم"),
        (
            "1:7",
            "صِرَٰطَ ٱلَّذِينَ أَنْعَمْتَ عَلَيْهِمْ غَيْرِ ٱلْمَغْضُوبِ عَلَيْهِمْ وَلَا ٱلضَّآلِّينَ",
            "صراط الذين انعمت عليهم غير المغضوب عليهم ولا الضالين",
        ),
    ];

    for (verse_key, uthmani, simple) in verse_texts.iter() {
        let node_id =
            sqlx::query_scalar::<_, i64>("SELECT id FROM nodes WHERE ukey = 'VERSE:' || ?")
                .bind(verse_key)
                .fetch_one(pool)
                .await?;

        // Insert uthmani text
        sqlx::query(
            "INSERT INTO script_contents (resource_id, node_id, text_content) VALUES (?, ?, ?)",
        )
        .bind(uthmani_id)
        .bind(node_id)
        .bind(uthmani)
        .execute(pool)
        .await?;

        // Insert simple text
        sqlx::query(
            "INSERT INTO script_contents (resource_id, node_id, text_content) VALUES (?, ?, ?)",
        )
        .bind(simple_id)
        .bind(node_id)
        .bind(simple)
        .execute(pool)
        .await?;
    }

    // Insert word text content (verse 1:1 words)
    let word_texts = [
        (1, "بِسْمِ", "بسم", "bismi"),
        (2, "ٱللَّهِ", "الله", "Allāhi"),
        (3, "ٱلرَّحْمَٰنِ", "الرحمن", "al-Raḥmāni"),
        (4, "ٱلرَّحِيمِ", "الرحيم", "al-Raḥīmi"),
    ];

    for (pos, uthmani, simple, translit) in word_texts.iter() {
        let word_id = sqlx::query_scalar::<_, i64>(
            "SELECT word_id FROM words WHERE verse_key = '1:1' AND position = ?",
        )
        .bind(pos)
        .fetch_one(pool)
        .await?;

        let node_id =
            sqlx::query_scalar::<_, i64>("SELECT id FROM nodes WHERE ukey = 'WORD:' || ?")
                .bind(word_id)
                .fetch_one(pool)
                .await?;

        // Uthmani
        sqlx::query(
            "INSERT INTO script_contents (resource_id, node_id, text_content) VALUES (?, ?, ?)",
        )
        .bind(uthmani_id)
        .bind(node_id)
        .bind(uthmani)
        .execute(pool)
        .await?;

        // Simple
        sqlx::query(
            "INSERT INTO script_contents (resource_id, node_id, text_content) VALUES (?, ?, ?)",
        )
        .bind(simple_id)
        .bind(node_id)
        .bind(simple)
        .execute(pool)
        .await?;

        // Transliteration
        sqlx::query(
            "INSERT INTO script_contents (resource_id, node_id, text_content) VALUES (?, ?, ?)",
        )
        .bind(translit_id)
        .bind(node_id)
        .bind(translit)
        .execute(pool)
        .await?;
    }

    // Step 7: Insert Languages
    sqlx::query(
        "INSERT INTO languages (language_code, english_name, native_name, direction) VALUES
            ('en', 'English', 'English', 'ltr'),
            ('ar', 'Arabic', 'العربية', 'rtl'),
            ('fr', 'French', 'Français', 'ltr'),
            ('ur', 'Urdu', 'اردو', 'rtl'),
            ('id', 'Indonesian', 'Indonesia', 'ltr'),
            ('tr', 'Turkish', 'Türkçe', 'ltr'),
            ('es', 'Spanish', 'Español', 'ltr')",
    )
    .execute(pool)
    .await?;

    // Step 8: Insert Translators
    sqlx::query(
        "INSERT INTO translators (slug, full_name, language_code, description, license, website, version) VALUES
            ('sahih-intl', 'Sahih International', 'en', 'Clear and modern English translation', 'Public Domain', 'https://quran.com', '1.0'),
            ('yusuf-ali', 'Abdullah Yusuf Ali', 'en', 'Classic English translation', 'Public Domain', 'https://www.al-islam.org', '1.0'),
            ('pickthall', 'Marmaduke Pickthall', 'en', 'First English translation by a Muslim', 'Public Domain', NULL, '1.0'),
            ('khattab', 'Dr. Mustafa Khattab', 'en', 'The Clear Quran', 'CC BY-NC-ND 4.0', 'https://theclearquran.org', '1.0'),
            ('hilali-khan', 'Dr. Muhsin Khan & Dr. Taqi-ud-Din al-Hilali', 'en', 'Noble Quran', 'Public Domain', NULL, '1.0')"
    )
    .execute(pool)
    .await?;

    // Step 9: Insert Verse Translations (verse 1:1 only)
    sqlx::query(
        "INSERT INTO verse_translations (verse_key, translator_id, translation)
            SELECT '1:1', translator_id, 'In the name of Allah, the Entirely Merciful, the Especially Merciful.' FROM translators WHERE slug = 'sahih-intl'
            UNION ALL SELECT '1:1', translator_id, 'In the name of God, Most Gracious, Most Merciful.' FROM translators WHERE slug = 'yusuf-ali'
            UNION ALL SELECT '1:1', translator_id, 'In the name of Allah, the Beneficent, the Merciful.' FROM translators WHERE slug = 'pickthall'
            UNION ALL SELECT '1:1', translator_id, 'In the Name of Allah—the Most Compassionate, Most Merciful.' FROM translators WHERE slug = 'khattab'
            UNION ALL SELECT '1:1', translator_id, 'In the Name of Allah, the Most Gracious, the Most Merciful.' FROM translators WHERE slug = 'hilali-khan'"
    )
    .execute(pool)
    .await?;

    // Step 10: Insert Word Translations (verse 1:1, Sahih International only)
    sqlx::query(
        "INSERT INTO word_translations (word_id, translator_id, translation)
            SELECT w.word_id, t.translator_id, 'In the name' FROM words w, translators t WHERE w.verse_key = '1:1' AND w.position = 1 AND t.slug = 'sahih-intl'
            UNION ALL SELECT w.word_id, t.translator_id, 'of Allah' FROM words w, translators t WHERE w.verse_key = '1:1' AND w.position = 2 AND t.slug = 'sahih-intl'
            UNION ALL SELECT w.word_id, t.translator_id, 'the Entirely Merciful' FROM words w, translators t WHERE w.verse_key = '1:1' AND w.position = 3 AND t.slug = 'sahih-intl'
            UNION ALL SELECT w.word_id, t.translator_id, 'the Especially Merciful' FROM words w, translators t WHERE w.verse_key = '1:1' AND w.position = 4 AND t.slug = 'sahih-intl'"
    )
    .execute(pool)
    .await?;

    // Step 11: Insert Test Goal
    sqlx::query(
        "INSERT OR IGNORE INTO goals (goal_id, goal_type, goal_group, label, description) VALUES
            ('memorization:chapters-1-3', 'custom', 'memorization', 'Memorize Chapters 1-3',
            'Master all 493 verses from Al-Fatihah, Al-Baqarah, and Al-Imran')",
    )
    .execute(pool)
    .await?;

    // Add all verses from chapters 1-3 to the goal
    sqlx::query(
        "INSERT OR IGNORE INTO node_goals (goal_id, node_id, priority)
            SELECT 'memorization:chapters-1-3', id, 1001000
            FROM nodes
            WHERE ukey LIKE 'VERSE:1:%' OR ukey LIKE 'VERSE:2:%' OR ukey LIKE 'VERSE:3:%'",
    )
    .execute(pool)
    .await?;

    // Step 12: Insert Node Metadata (foundational, influence, difficulty scores)
    sqlx::query(
        "INSERT OR IGNORE INTO node_metadata (node_id, key, value)
            SELECT n.id, 'foundational_score',
            CASE WHEN v.chapter_number = 1 AND v.verse_number = 1 THEN 0.85
                ELSE 0.1 + (CAST(v.chapter_number AS REAL) * 0.01) + (CAST(v.verse_number AS REAL) * 0.001)
            END
            FROM nodes n
            JOIN verses v ON n.ukey = 'VERSE:' || v.verse_key
            WHERE n.ukey LIKE 'VERSE:%'
            UNION ALL
            SELECT n.id, 'influence_score',
            CASE WHEN v.chapter_number = 1 AND v.verse_number = 1 THEN 0.90
                ELSE 0.1 + (CAST(v.chapter_number AS REAL) * 0.01) + (CAST(v.verse_number AS REAL) * 0.001)
            END
            FROM nodes n
            JOIN verses v ON n.ukey = 'VERSE:' || v.verse_key
            WHERE n.ukey LIKE 'VERSE:%'
            UNION ALL
            SELECT n.id, 'difficulty_score', 0.3 + (CAST(v.verse_number AS REAL) * 0.001)
            FROM nodes n
            JOIN verses v ON n.ukey = 'VERSE:' || v.verse_key
            WHERE n.ukey LIKE 'VERSE:%'
            UNION ALL
            SELECT n.id, 'quran_order', CAST(v.chapter_number AS INTEGER) * 1000 + CAST(v.verse_number AS INTEGER)
            FROM nodes n
            JOIN verses v ON n.ukey = 'VERSE:' || v.verse_key
            WHERE n.ukey LIKE 'VERSE:%'"
    )
    .execute(pool)
    .await?;

    // Step 13: Insert Sequential Prerequisite Edges
    sqlx::query(
        "INSERT OR IGNORE INTO edges (source_id, target_id, edge_type, distribution_type)
            SELECT curr.id AS source_id, next.id AS target_id, 0 AS edge_type, 0 AS distribution_type
            FROM nodes curr
            JOIN verses curr_v ON curr.ukey = 'VERSE:' || curr_v.verse_key
            JOIN verses next_v ON curr_v.chapter_number = next_v.chapter_number
                            AND next_v.verse_number = curr_v.verse_number + 1
            JOIN nodes next ON next.ukey = 'VERSE:' || next_v.verse_key
            WHERE curr.ukey LIKE 'VERSE:%' AND next.ukey LIKE 'VERSE:%'"
    )
    .execute(pool)
    .await?;

    Ok(())
}
