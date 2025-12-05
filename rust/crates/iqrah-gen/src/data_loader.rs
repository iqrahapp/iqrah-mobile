use anyhow::{Context, Result};
use iqrah_core::domain::models::{Chapter, Verse, Word};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct QuranData {
    pub chapters: Vec<Chapter>,
    pub verses: Vec<Verse>,
    pub words: Vec<Word>,
}

pub fn load_quran_data(data_dir: &Path) -> Result<QuranData> {
    let text_dir = data_dir.join("text/wbw");
    let metadata_dir = data_dir.join("structural-metadata");

    // 1. Load Uthmani Text (Word by Word)
    let uthmani_path = text_dir.join("uthmani.json");
    let uthmani_data: HashMap<String, WordJson> = load_json(&uthmani_path)?;

    // 2. Load Metadata
    let surah_info_path = metadata_dir.join("quran-metadata-surah-name.json");
    let surah_info: HashMap<String, SurahInfoJson> = load_json(&surah_info_path)?;

    // 3. Construct Models
    let mut chapters = Vec::new();
    let mut verses = Vec::new();
    let mut words = Vec::new();

    // Build Chapters
    for i in 1..=114 {
        let chapter_num = i.to_string();
        if let Some(info) = surah_info.get(&chapter_num) {
            chapters.push(Chapter {
                number: i,
                name_arabic: info.name_arabic.clone(),
                name_transliteration: info.name_complex.clone(),
                name_translation: info.name_translated.clone(),
                revelation_place: Some(info.revelation_place.clone()),
                verse_count: info.verses_count,
            });
        }
    }

    // Group words by verse
    let mut verse_words: HashMap<String, Vec<Word>> = HashMap::new();

    for (key, word_json) in uthmani_data {
        let parts: Vec<&str> = key.split(':').collect();
        if parts.len() < 3 {
            continue;
        }

        let chapter_num: i32 = parts[0].parse()?;
        let verse_num: i32 = parts[1].parse()?;
        let position: i32 = parts[2].parse()?;
        let verse_key = format!("{}:{}", chapter_num, verse_num);

        let word = Word {
            id: 0,
            verse_key: verse_key.clone(),
            position,
            text_uthmani: word_json.text_uthmani,
            text_simple: None,
            transliteration: None,
        };

        words.push(word.clone());
        verse_words.entry(verse_key).or_default().push(word);
    }

    // Build Verses
    for (verse_key, words) in &verse_words {
        let parts: Vec<&str> = verse_key.split(':').collect();
        let chapter_num: i32 = parts[0].parse()?;
        let verse_num: i32 = parts[1].parse()?;

        // Sort words by position
        let mut sorted_words = words.clone();
        sorted_words.sort_by_key(|w| w.position);

        let text_uthmani = sorted_words
            .iter()
            .map(|w| w.text_uthmani.clone())
            .collect::<Vec<_>>()
            .join(" ");

        verses.push(Verse {
            key: verse_key.clone(),
            chapter_number: chapter_num,
            verse_number: verse_num,
            text_uthmani,
            text_simple: None,
            juz: 0,
            page: 0,
        });
    }

    // Sort for consistency
    chapters.sort_by_key(|c| c.number);
    verses.sort_by_key(|v| (v.chapter_number, v.verse_number));
    // Words are already in order per verse, but verses might be out of order

    Ok(QuranData {
        chapters,
        verses,
        words,
    })
}

fn load_json<T: for<'a> Deserialize<'a>>(path: &Path) -> Result<T> {
    let file = File::open(path).with_context(|| format!("Failed to open {:?}", path))?;
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).with_context(|| format!("Failed to parse {:?}", path))
}

// JSON mapping structs
#[derive(Deserialize)]
struct WordJson {
    #[serde(rename = "text")]
    text_uthmani: String,
}

#[derive(Deserialize)]
struct SurahInfoJson {
    #[serde(rename = "name_arabic")]
    name_arabic: String,
    #[serde(rename = "name_simple")]
    name_complex: String, // Mapping simple to complex for now or rename field in Chapter
    #[serde(rename = "name")]
    name_translated: String,
    #[serde(rename = "revelation_place")]
    revelation_place: String,
    #[serde(rename = "verses_count")]
    verses_count: i32,
}

// ... (previous code)

#[derive(Debug, Clone)]
pub struct MorphologyData {
    pub segments: Vec<MorphologySegment>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MorphologySegment {
    #[serde(rename = "LOCATION")]
    pub location: String, // "1:1:1:1"
    #[serde(rename = "FORM")]
    #[allow(dead_code)]
    pub text: String,
    #[serde(rename = "TAG")]
    pub tag: String,
    #[serde(rename = "FEATURES")]
    pub features: String,
    #[serde(skip)]
    pub root: Option<String>,
    #[serde(skip)]
    pub lemma: Option<String>,
    #[serde(skip)]
    pub pos: Option<String>,
}

pub fn load_morphology_data(path: &Path) -> Result<MorphologyData> {
    let file = File::open(path)?;
    let mut rdr = csv::ReaderBuilder::new().delimiter(b'\t').from_reader(file);

    let mut segments = Vec::new();
    for result in rdr.deserialize() {
        let mut segment: MorphologySegment = result?;

        // Parse features to extract root and lemma
        // FEATURES: "P|PREF|LEM:ب" or "N|LEM:ٱِسْم|ROOT:smw|M|GEN"
        for feature in segment.features.split('|') {
            if feature.starts_with("ROOT:") {
                segment.root = Some(feature.trim_start_matches("ROOT:").to_string());
            } else if feature.starts_with("LEM:") {
                segment.lemma = Some(feature.trim_start_matches("LEM:").to_string());
            }
        }
        segment.pos = Some(segment.tag.clone());

        segments.push(segment);
    }

    Ok(MorphologyData { segments })
}

// ... (rest of the file)
