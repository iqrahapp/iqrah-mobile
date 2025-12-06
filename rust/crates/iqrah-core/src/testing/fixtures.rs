//! Reusable test fixtures for Quranic content.
//!
//! Provides pre-built test data and mock setup helpers to avoid duplication
//! across test files.

use crate::domain::{Chapter, MorphologySegment, Root, Verse, Word};
use crate::ports::content_repository::MockContentRepository;
use mockall::predicate::*;
use std::collections::HashMap;

// ============================================================================
// Chapter Fixtures
// ============================================================================

/// Create Al-Fatihah chapter (Chapter 1) for testing
pub fn create_chapter_al_fatihah() -> Chapter {
    Chapter {
        number: 1,
        name_arabic: "الفاتحة".to_string(),
        name_transliteration: "Al-Fatihah".to_string(),
        name_translation: "The Opening".to_string(),
        revelation_place: Some("makkah".to_string()),
        verse_count: 7,
    }
}

/// Create Al-Baqarah chapter (Chapter 2) for testing
pub fn create_chapter_al_baqarah() -> Chapter {
    Chapter {
        number: 2,
        name_arabic: "البقرة".to_string(),
        name_transliteration: "Al-Baqarah".to_string(),
        name_translation: "The Cow".to_string(),
        revelation_place: Some("madinah".to_string()),
        verse_count: 286,
    }
}

// ============================================================================
// Verse Fixtures
// ============================================================================

/// All 7 verses of Al-Fatihah with Arabic text
pub fn create_verses_al_fatihah() -> Vec<Verse> {
    let verses_data = [
        ("1:1", "بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ"),
        ("1:2", "ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ"),
        ("1:3", "ٱلرَّحْمَٰنِ ٱلرَّحِيمِ"),
        ("1:4", "مَٰلِكِ يَوْمِ ٱلدِّينِ"),
        ("1:5", "إِيَّاكَ نَعْبُدُ وَإِيَّاكَ نَسْتَعِينُ"),
        ("1:6", "ٱهْدِنَا ٱلصِّرَٰطَ ٱلْمُسْتَقِيمَ"),
        ("1:7", "صِرَٰطَ ٱلَّذِينَ أَنْعَمْتَ عَلَيْهِمْ غَيْرِ ٱلْمَغْضُوبِ عَلَيْهِمْ وَلَا ٱلضَّآلِّينَ"),
    ];

    verses_data
        .iter()
        .enumerate()
        .map(|(i, (key, text))| Verse {
            key: key.to_string(),
            chapter_number: 1,
            verse_number: (i + 1) as i32,
            text_uthmani: text.to_string(),
            text_simple: None,
            juz: 1,
            page: 1,
        })
        .collect()
}

/// Get a specific verse from Al-Fatihah by verse number (1-7)
pub fn get_verse_al_fatihah(verse_number: i32) -> Option<Verse> {
    create_verses_al_fatihah()
        .into_iter()
        .find(|v| v.verse_number == verse_number)
}

/// Create a HashMap of verse_key -> Verse for Al-Fatihah
pub fn create_verses_map_al_fatihah() -> HashMap<String, Verse> {
    create_verses_al_fatihah()
        .into_iter()
        .map(|v| (v.key.clone(), v))
        .collect()
}

// ============================================================================
// Word Fixtures
// ============================================================================

/// Words for verse 1:1 (Bismillah)
pub fn create_words_verse_1_1() -> Vec<Word> {
    vec![
        Word {
            id: 1,
            verse_key: "1:1".to_string(),
            position: 1,
            text_uthmani: "بِسْمِ".to_string(),
            text_simple: Some("بسم".to_string()),
            transliteration: Some("bismi".to_string()),
        },
        Word {
            id: 2,
            verse_key: "1:1".to_string(),
            position: 2,
            text_uthmani: "ٱللَّهِ".to_string(),
            text_simple: Some("الله".to_string()),
            transliteration: Some("al-lahi".to_string()),
        },
        Word {
            id: 3,
            verse_key: "1:1".to_string(),
            position: 3,
            text_uthmani: "ٱلرَّحْمَٰنِ".to_string(),
            text_simple: Some("الرحمن".to_string()),
            transliteration: Some("ar-rahmani".to_string()),
        },
        Word {
            id: 4,
            verse_key: "1:1".to_string(),
            position: 4,
            text_uthmani: "ٱلرَّحِيمِ".to_string(),
            text_simple: Some("الرحيم".to_string()),
            transliteration: Some("ar-rahimi".to_string()),
        },
    ]
}

/// Words for verse 1:2
pub fn create_words_verse_1_2() -> Vec<Word> {
    vec![
        Word {
            id: 5,
            verse_key: "1:2".to_string(),
            position: 1,
            text_uthmani: "ٱلْحَمْدُ".to_string(),
            text_simple: Some("الحمد".to_string()),
            transliteration: Some("al-hamdu".to_string()),
        },
        Word {
            id: 6,
            verse_key: "1:2".to_string(),
            position: 2,
            text_uthmani: "لِلَّهِ".to_string(),
            text_simple: Some("لله".to_string()),
            transliteration: Some("lillahi".to_string()),
        },
        Word {
            id: 7,
            verse_key: "1:2".to_string(),
            position: 3,
            text_uthmani: "رَبِّ".to_string(),
            text_simple: Some("رب".to_string()),
            transliteration: Some("rabbi".to_string()),
        },
        Word {
            id: 8,
            verse_key: "1:2".to_string(),
            position: 4,
            text_uthmani: "ٱلْعَٰلَمِينَ".to_string(),
            text_simple: Some("العالمين".to_string()),
            transliteration: Some("al-'alamina".to_string()),
        },
    ]
}

/// Get all words for Al-Fatihah verses 1:1 and 1:2
pub fn create_words_map() -> HashMap<String, Vec<Word>> {
    let mut map = HashMap::new();
    map.insert("1:1".to_string(), create_words_verse_1_1());
    map.insert("1:2".to_string(), create_words_verse_1_2());
    map
}

// ============================================================================
// Morphology & Root Fixtures
// ============================================================================

/// Common Arabic roots used in tests
pub fn create_common_roots() -> HashMap<String, Root> {
    let roots = vec![
        ("س-م-و", "سمو", "s-m-w"),
        ("ا-ل-ه", "اله", "'-l-h"),
        ("ر-ح-م", "رحم", "r-h-m"),
        ("ح-م-د", "حمد", "h-m-d"),
        ("ر-ب-ب", "ربب", "r-b-b"),
        ("ع-ل-م", "علم", "'-l-m"),
    ];

    roots
        .into_iter()
        .map(|(id, arabic, translit)| {
            (
                id.to_string(),
                Root {
                    root_id: id.to_string(),
                    arabic: arabic.to_string(),
                    transliteration: Some(translit.to_string()),
                    root_type: "trilateral".to_string(),
                },
            )
        })
        .collect()
}

/// Morphology segments for words in verse 1:1
pub fn create_morphology_verse_1_1() -> HashMap<i64, Vec<MorphologySegment>> {
    let mut morphology = HashMap::new();

    // بِسْمِ - root س-م-و
    morphology.insert(
        1,
        vec![MorphologySegment {
            segment_id: 1,
            word_id: 1,
            position: 1,
            lemma_id: Some("اسم".to_string()),
            root_id: Some("س-م-و".to_string()),
            pos_tag: Some("noun".to_string()),
        }],
    );

    // ٱللَّهِ - root ا-ل-ه
    morphology.insert(
        2,
        vec![MorphologySegment {
            segment_id: 2,
            word_id: 2,
            position: 1,
            lemma_id: Some("الله".to_string()),
            root_id: Some("ا-ل-ه".to_string()),
            pos_tag: Some("noun".to_string()),
        }],
    );

    // ٱلرَّحْمَٰنِ - root ر-ح-م
    morphology.insert(
        3,
        vec![MorphologySegment {
            segment_id: 3,
            word_id: 3,
            position: 1,
            lemma_id: Some("رحمن".to_string()),
            root_id: Some("ر-ح-م".to_string()),
            pos_tag: Some("noun".to_string()),
        }],
    );

    // ٱلرَّحِيمِ - root ر-ح-م
    morphology.insert(
        4,
        vec![MorphologySegment {
            segment_id: 4,
            word_id: 4,
            position: 1,
            lemma_id: Some("رحيم".to_string()),
            root_id: Some("ر-ح-م".to_string()),
            pos_tag: Some("noun".to_string()),
        }],
    );

    morphology
}

// ============================================================================
// Mock Setup Helpers
// ============================================================================

/// Configure mock for Ayah Chain exercise tests (verse-based exercises)
pub fn setup_ayah_chain_mock(mock: &mut MockContentRepository) {
    let chapters = vec![create_chapter_al_fatihah()];
    let chapters_clone = chapters.clone();
    let verses = create_verses_al_fatihah();
    let verses_map = create_verses_map_al_fatihah();

    // get_chapter
    mock.expect_get_chapter()
        .with(eq(1))
        .returning(move |_| Ok(Some(chapters[0].clone())));

    // get_chapters
    mock.expect_get_chapters()
        .returning(move || Ok(chapters_clone.clone()));

    // get_verses_for_chapter
    let verses_clone = verses.clone();
    mock.expect_get_verses_for_chapter()
        .with(eq(1))
        .returning(move |_| Ok(verses_clone.clone()));

    // get_verse
    mock.expect_get_verse()
        .returning(move |key| Ok(verses_map.get(key).cloned()));
}

/// Configure mock for word-based exercise tests (grammar, morphology)
pub fn setup_word_mock(mock: &mut MockContentRepository) {
    let words_map = create_words_map();
    let morphology = create_morphology_verse_1_1();
    let roots = create_common_roots();
    let words_1_1 = create_words_verse_1_1();

    // get_words_for_verse
    mock.expect_get_words_for_verse()
        .returning(move |key| Ok(words_map.get(key).cloned().unwrap_or_default()));

    // get_word
    let all_words: Vec<Word> = create_words_verse_1_1()
        .into_iter()
        .chain(create_words_verse_1_2())
        .collect();
    mock.expect_get_word()
        .returning(move |id| Ok(all_words.iter().find(|w| w.id == id).cloned()));

    // get_morphology_for_word
    mock.expect_get_morphology_for_word()
        .returning(move |id| Ok(morphology.get(&id).cloned().unwrap_or_default()));

    // get_root_by_id
    mock.expect_get_root_by_id()
        .returning(move |id| Ok(roots.get(id).cloned()));

    // get_quran_text (for word text)
    mock.expect_get_quran_text().returning(move |id| {
        Ok(words_1_1
            .iter()
            .find(|w| w.id == id)
            .map(|w| w.text_uthmani.clone()))
    });
}

/// Configure mock with default empty implementations for all methods.
/// Use this as a base and then add specific expectations.
pub fn setup_default_mock(mock: &mut MockContentRepository) {
    // Default empty implementations - tests can override specific methods
    mock.expect_get_node().returning(|_| Ok(None));
    mock.expect_get_node_by_ukey().returning(|_| Ok(None));
    mock.expect_get_edges_from().returning(|_| Ok(vec![]));
    mock.expect_get_quran_text().returning(|_| Ok(None));
    mock.expect_get_translation().returning(|_, _| Ok(None));
    mock.expect_get_metadata().returning(|_, _| Ok(None));
    mock.expect_get_all_metadata()
        .returning(|_| Ok(HashMap::new()));
    mock.expect_node_exists().returning(|_| Ok(false));
    mock.expect_get_all_nodes().returning(|| Ok(vec![]));
    mock.expect_get_nodes_by_type().returning(|_| Ok(vec![]));
    mock.expect_get_words_in_ayahs().returning(|_| Ok(vec![]));
    mock.expect_get_adjacent_words()
        .returning(|_| Ok((None, None)));
    mock.expect_get_chapter().returning(|_| Ok(None));
    mock.expect_get_chapters().returning(|| Ok(vec![]));
    mock.expect_get_verse().returning(|_| Ok(None));
    mock.expect_get_verses_for_chapter()
        .returning(|_| Ok(vec![]));
    mock.expect_get_words_for_verse().returning(|_| Ok(vec![]));
    mock.expect_get_word().returning(|_| Ok(None));
    mock.expect_get_languages().returning(|| Ok(vec![]));
    mock.expect_get_language().returning(|_| Ok(None));
    mock.expect_get_translators_for_language()
        .returning(|_| Ok(vec![]));
    mock.expect_get_translator().returning(|_| Ok(None));
    mock.expect_get_translator_by_slug().returning(|_| Ok(None));
    mock.expect_get_verse_translation()
        .returning(|_, _| Ok(None));
    mock.expect_get_word_translation()
        .returning(|_, _| Ok(None));
    mock.expect_insert_translator()
        .returning(|_, _, _, _, _, _, _, _, _| Ok(0));
    mock.expect_insert_verse_translation()
        .returning(|_, _, _, _| Ok(()));
    mock.expect_get_available_packages()
        .returning(|_, _| Ok(vec![]));
    mock.expect_get_package().returning(|_| Ok(None));
    mock.expect_upsert_package().returning(|_| Ok(()));
    mock.expect_delete_package().returning(|_| Ok(()));
    mock.expect_get_installed_packages()
        .returning(|| Ok(vec![]));
    mock.expect_is_package_installed().returning(|_| Ok(false));
    mock.expect_mark_package_installed().returning(|_| Ok(()));
    mock.expect_mark_package_uninstalled().returning(|_| Ok(()));
    mock.expect_enable_package().returning(|_| Ok(()));
    mock.expect_disable_package().returning(|_| Ok(()));
    mock.expect_get_enabled_packages().returning(|| Ok(vec![]));
    mock.expect_get_morphology_for_word()
        .returning(|_| Ok(vec![]));
    mock.expect_get_root_by_id().returning(|_| Ok(None));
    mock.expect_get_lemma_by_id().returning(|_| Ok(None));
    mock.expect_get_scheduler_candidates()
        .returning(|_| Ok(vec![]));
    mock.expect_get_prerequisite_parents()
        .returning(|_| Ok(HashMap::new()));
    mock.expect_get_goal().returning(|_| Ok(None));
    mock.expect_get_nodes_for_goal().returning(|_| Ok(vec![]));
    mock.expect_get_verses_batch()
        .returning(|_| Ok(HashMap::new()));
    mock.expect_get_words_batch()
        .returning(|_| Ok(HashMap::new()));
}
