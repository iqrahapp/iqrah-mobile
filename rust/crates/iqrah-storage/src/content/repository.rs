use super::models::{
    CandidateNodeRow, ChapterRow, ContentPackageRow, EdgeRow, GoalRow, InstalledPackageRow,
    LanguageRow, LemmaRow, MorphologySegmentRow, NodeGoalRow, PrerequisiteRow, RootRow,
    TranslatorRow, VerseRow, VerseTranslationRow, WordRow, WordTranslationRow,
};
use async_trait::async_trait;
use chrono::DateTime;
use iqrah_core::{
    ports::content_repository::SchedulerGoal, scheduler_v2::CandidateNode,
    Chapter, ContentPackage, ContentRepository, DistributionType, Edge, EdgeType, InstalledPackage,
    Language, Lemma, MorphologySegment, Node, NodeType, PackageType, Root, Translator, Verse, Word,
};
use sqlx::{query, query_as, SqlitePool};
use std::collections::HashMap;
use std::sync::Arc;

use super::node_registry::NodeRegistry;

pub struct SqliteContentRepository {
    pool: SqlitePool,
    registry: Arc<NodeRegistry>,
}

impl SqliteContentRepository {
    pub fn new(pool: SqlitePool, registry: Arc<NodeRegistry>) -> Self {
        Self { pool, registry }
    }

    /// Internal method: Get node by integer ID
    /// This bypasses the registry and directly queries content tables
    async fn get_node_by_id_internal(&self, node_id: i64) -> anyhow::Result<Option<Node>> {
        // V2 schema: Parse node_id to determine type and query appropriate table
        use iqrah_core::domain::node_id as nid;

        // Detect type first
        let node_type = match nid::decode_type(node_id) {
            Some(t) => t,
            None => return Ok(None), // Invalid ID format means node doesn't exist
        };

        match node_type {
            NodeType::Verse => {
                let (chapter, verse) = nid::decode_verse(node_id)
                    .ok_or_else(|| anyhow::anyhow!("Invalid verse ID"))?;
                // Ensure we use the standard verse_key format "chapter:verse" for DB query
                let verse_key = format!("{}:{}", chapter, verse);

                let row = query_as::<_, (String, i64)>(
                    "SELECT verse_key, created_at FROM verses WHERE verse_key = ?",
                )
                .bind(&verse_key)
                .fetch_optional(&self.pool)
                .await?;

                Ok(row.map(|(_vk, _)| Node {
                    id: node_id,
                    ukey: nid::verse(chapter, verse),
                    node_type: NodeType::Verse,
                }))
            }
            NodeType::Chapter => {
                let chapter_num = nid::decode_chapter(node_id)
                    .ok_or_else(|| anyhow::anyhow!("Invalid chapter ID"))?;

                let row = query_as::<_, (i32, i64)>(
                    "SELECT chapter_number, created_at FROM chapters WHERE chapter_number = ?",
                )
                .bind(chapter_num)
                .fetch_optional(&self.pool)
                .await?;

                Ok(row.map(|(num, _)| Node {
                    id: node_id,
                    ukey: nid::chapter(num as u8),
                    node_type: NodeType::Chapter,
                }))
            }
            NodeType::Word => {
                let word_id =
                    nid::decode_word(node_id).ok_or_else(|| anyhow::anyhow!("Invalid word ID"))?;

                let row = query_as::<_, (i32, i64)>(
                    "SELECT word_id, created_at FROM words WHERE word_id = ?",
                )
                .bind(word_id)
                .fetch_optional(&self.pool)
                .await?;

                Ok(row.map(|(wid, _)| Node {
                    id: node_id,
                    ukey: nid::word(wid as i64),
                    node_type: NodeType::Word,
                }))
            }
            NodeType::WordInstance => {
                // Word instances map to words in the DB
                let (chapter, verse, position) = nid::decode_word_instance(node_id)
                    .ok_or_else(|| anyhow::anyhow!("Invalid word instance ID"))?;
                let verse_key = format!("{}:{}", chapter, verse);

                let row = query_as::<_, (i32, i64)>(
                    "SELECT word_id, created_at FROM words WHERE verse_key = ? AND position = ?",
                )
                .bind(&verse_key)
                .bind(position)
                .fetch_optional(&self.pool)
                .await?;

                Ok(row.map(|(_wid, _)| Node {
                    id: node_id,
                    ukey: nid::word_instance(chapter, verse, position),
                    node_type: NodeType::WordInstance,
                }))
            }
            NodeType::Knowledge => {
                // Knowledge nodes are virtual in V2 schema
                Ok(None)
            }
            _ => Ok(None), // Other types not supported yet
        }
    }
}

#[async_trait]
impl ContentRepository for SqliteContentRepository {
    async fn get_node(&self, node_id: i64) -> anyhow::Result<Option<Node>> {
        // Delegate to internal method which queries content tables directly
        self.get_node_by_id_internal(node_id).await
    }

    async fn get_node_by_ukey(&self, ukey: &str) -> anyhow::Result<Option<Node>> {
        use iqrah_core::domain::node_id as nid;

        // Two-level lookup strategy:
        // 1. Check registry for cached ukey -> id mapping
        // 2. If not in registry, parse ukey and encode to i64
        // 3. Query content tables with the ID
        // 4. If node exists, register it for future fast lookups

        // Step 1: Try registry first (fast path with caching)
        if let Some(id) = self.registry.get_id(ukey).await? {
            return self.get_node(id).await;
        }

        // Step 2: Parse ukey to determine type and encode to i64
        let node_type = match nid::node_type(ukey) {
            Ok(t) => t,
            Err(_) => return Ok(None),
        };

        let (id, type_code) = match node_type {
            NodeType::Verse => {
                let (chapter, verse) = nid::parse_verse(ukey)?;
                (nid::encode_verse(chapter, verse), 1)
            }
            NodeType::Chapter => {
                let num = nid::parse_chapter(ukey)?;
                (nid::encode_chapter(num), 0)
            }
            NodeType::Word => {
                let word_id = nid::parse_word(ukey)?;
                (nid::encode_word(word_id), 2)
            }
            NodeType::WordInstance => {
                let (ch, v, pos) = nid::parse_word_instance(ukey)?;
                (nid::encode_word_instance(ch, v, pos), 3)
            }
            _ => return Ok(None),
        };

        // Step 3: Query content tables
        let node = self.get_node(id).await?;

        // Step 4: If node exists, register it in the registry for future lookups
        if node.is_some() {
            // Register asynchronously, ignore errors (registry is optimization, not critical)
            let _ = self.registry.register(id, ukey.to_string(), type_code).await;
        }

        Ok(node)
    }

    async fn get_edges_from(&self, source_id: i64) -> anyhow::Result<Vec<Edge>> {
        let rows = query_as::<_, EdgeRow>(
            "SELECT source_id, target_id, edge_type, distribution_type, param1, param2
             FROM edges
             WHERE source_id = ?",
        )
        .bind(source_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| Edge {
                source_id: r.source_id,
                target_id: r.target_id,
                edge_type: if r.edge_type == 0 {
                    EdgeType::Dependency
                } else {
                    EdgeType::Knowledge
                },
                distribution_type: match r.distribution_type {
                    0 => DistributionType::Const,
                    1 => DistributionType::Normal,
                    _ => DistributionType::Beta,
                },
                param1: r.param1,
                param2: r.param2,
            })
            .collect())
    }

    async fn get_quran_text(&self, node_id: i64) -> anyhow::Result<Option<String>> {
        // V2 schema: Query text from verses or words tables
        use iqrah_core::domain::node_id as nid;

        // Detect type first
        let node_type = match nid::decode_type(node_id) {
            Some(t) => t,
            None => return Ok(None),
        };

        match node_type {
            NodeType::Verse => {
                let (chapter, verse) = nid::decode_verse(node_id)
                    .ok_or_else(|| anyhow::anyhow!("Invalid verse ID"))?;
                let verse_key = format!("{}:{}", chapter, verse);

                let row =
                    query_as::<_, (String,)>("SELECT text_uthmani FROM verses WHERE verse_key = ?")
                        .bind(&verse_key)
                        .fetch_optional(&self.pool)
                        .await?;

                Ok(row.map(|(text,)| text))
            }
            NodeType::Word => {
                let word_id =
                    nid::decode_word(node_id).ok_or_else(|| anyhow::anyhow!("Invalid word ID"))?;

                let row =
                    query_as::<_, (String,)>("SELECT text_uthmani FROM words WHERE word_id = ?")
                        .bind(word_id)
                        .fetch_optional(&self.pool)
                        .await?;

                Ok(row.map(|(text,)| text))
            }
            NodeType::WordInstance => {
                let (chapter, verse, position) = nid::decode_word_instance(node_id)
                    .ok_or_else(|| anyhow::anyhow!("Invalid word instance ID"))?;
                let verse_key = format!("{}:{}", chapter, verse);

                let row = query_as::<_, (String,)>(
                    "SELECT text_uthmani FROM words WHERE verse_key = ? AND position = ?",
                )
                .bind(&verse_key)
                .bind(position)
                .fetch_optional(&self.pool)
                .await?;

                Ok(row.map(|(text,)| text))
            }
            _ => Ok(None),
        }
    }

    async fn get_translation(&self, node_id: i64, lang: &str) -> anyhow::Result<Option<String>> {
        // V2 schema: Query from verse_translations or word_translations
        use iqrah_core::domain::node_id as nid;

        // First, find a translator for the given language
        let translator = query_as::<_, (i32,)>(
            "SELECT translator_id FROM translators WHERE language_code = ? LIMIT 1",
        )
        .bind(lang)
        .fetch_optional(&self.pool)
        .await?;

        let translator_id = match translator {
            Some((id,)) => id,
            None => return Ok(None), // No translator found for this language
        };

        // Detect type
        let node_type = match nid::decode_type(node_id) {
            Some(t) => t,
            None => return Ok(None),
        };

        match node_type {
            NodeType::Verse => {
                let (chapter, verse) = nid::decode_verse(node_id)
                    .ok_or_else(|| anyhow::anyhow!("Invalid verse ID"))?;
                let verse_key = format!("{}:{}", chapter, verse);

                let row = query_as::<_, (String,)>(
                    "SELECT translation FROM verse_translations WHERE verse_key = ? AND translator_id = ?"
                )
                .bind(&verse_key)
                .bind(translator_id)
                .fetch_optional(&self.pool)
                .await?;

                Ok(row.map(|(text,)| text))
            }
            NodeType::Word => {
                let word_id =
                    nid::decode_word(node_id).ok_or_else(|| anyhow::anyhow!("Invalid word ID"))?;

                let row = query_as::<_, (String,)>(
                    "SELECT translation FROM word_translations WHERE word_id = ? AND translator_id = ?"
                )
                .bind(word_id)
                .bind(translator_id)
                .fetch_optional(&self.pool)
                .await?;

                Ok(row.map(|(text,)| text))
            }
            NodeType::WordInstance => {
                let (chapter, verse, position) = nid::decode_word_instance(node_id)
                    .ok_or_else(|| anyhow::anyhow!("Invalid word instance ID"))?;
                let verse_key = format!("{}:{}", chapter, verse);

                // Need to find word_id first for word instance
                let word_row = query_as::<_, (i32,)>(
                    "SELECT word_id FROM words WHERE verse_key = ? AND position = ?",
                )
                .bind(&verse_key)
                .bind(position)
                .fetch_optional(&self.pool)
                .await?;

                if let Some((word_id,)) = word_row {
                    let row = query_as::<_, (String,)>(
                        "SELECT translation FROM word_translations WHERE word_id = ? AND translator_id = ?"
                    )
                    .bind(word_id)
                    .bind(translator_id)
                    .fetch_optional(&self.pool)
                    .await?;

                    Ok(row.map(|(text,)| text))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    async fn get_metadata(&self, node_id: i64, key: &str) -> anyhow::Result<Option<String>> {
        // For backwards compatibility, map old metadata keys to new tables
        match key {
            "arabic" => self.get_quran_text(node_id).await,
            "translation" => self.get_translation(node_id, "en").await,
            _ => {
                // Unknown key - return None
                Ok(None)
            }
        }
    }

    async fn get_all_metadata(&self, node_id: i64) -> anyhow::Result<HashMap<String, String>> {
        let mut metadata = HashMap::new();

        // Get arabic text
        if let Some(arabic) = self.get_quran_text(node_id).await? {
            metadata.insert("arabic".to_string(), arabic);
        }

        // Get translation (default to English)
        if let Some(translation) = self.get_translation(node_id, "en").await? {
            metadata.insert("translation".to_string(), translation);
        }

        Ok(metadata)
    }

    async fn node_exists(&self, node_id: i64) -> anyhow::Result<bool> {
        // Fast path: Check registry first
        if self.registry.exists_by_id(node_id).await? {
            return Ok(true);
        }

        // V2 schema: Check existence in appropriate table
        use iqrah_core::domain::node_id as nid;

        // Detect type first
        let node_type = match nid::decode_type(node_id) {
            Some(t) => t,
            None => return Ok(false),
        };

        match node_type {
            NodeType::Verse => {
                let (chapter, verse) = nid::decode_verse(node_id)
                    .ok_or_else(|| anyhow::anyhow!("Invalid verse ID"))?;
                let verse_key = format!("{}:{}", chapter, verse);

                let count: (i64,) = query_as("SELECT COUNT(*) FROM verses WHERE verse_key = ?")
                    .bind(&verse_key)
                    .fetch_one(&self.pool)
                    .await?;
                Ok(count.0 > 0)
            }
            NodeType::Chapter => {
                let chapter_num = nid::decode_chapter(node_id)
                    .ok_or_else(|| anyhow::anyhow!("Invalid chapter ID"))?;

                let count: (i64,) =
                    query_as("SELECT COUNT(*) FROM chapters WHERE chapter_number = ?")
                        .bind(chapter_num)
                        .fetch_one(&self.pool)
                        .await?;
                Ok(count.0 > 0)
            }
            NodeType::Word => {
                let word_id =
                    nid::decode_word(node_id).ok_or_else(|| anyhow::anyhow!("Invalid word ID"))?;

                let count: (i64,) = query_as("SELECT COUNT(*) FROM words WHERE word_id = ?")
                    .bind(word_id)
                    .fetch_one(&self.pool)
                    .await?;
                Ok(count.0 > 0)
            }
            NodeType::WordInstance => {
                let (chapter, verse, position) = nid::decode_word_instance(node_id)
                    .ok_or_else(|| anyhow::anyhow!("Invalid word instance ID"))?;
                let verse_key = format!("{}:{}", chapter, verse);

                let count: (i64,) =
                    query_as("SELECT COUNT(*) FROM words WHERE verse_key = ? AND position = ?")
                        .bind(&verse_key)
                        .bind(position)
                        .fetch_one(&self.pool)
                        .await?;
                Ok(count.0 > 0)
            }
            _ => Ok(false),
        }
    }

    async fn get_all_nodes(&self) -> anyhow::Result<Vec<Node>> {
        // V2 schema: Query from actual content tables
        // For import checking, we primarily care about verses
        use iqrah_core::domain::node_id as nid;

        let verse_rows = query_as::<_, (String, i64)>("SELECT verse_key, created_at FROM verses")
            .fetch_all(&self.pool)
            .await?;

        let nodes: Vec<Node> = verse_rows
            .into_iter()
            .filter_map(|(verse_key, _created_at)| {
                // Parse verse_key to ensure we construct a valid standardized ID
                nid::parse_verse(&verse_key).ok().map(|(ch, v)| Node {
                    id: nid::encode_verse(ch, v),
                    ukey: nid::verse(ch, v),
                    node_type: NodeType::Verse,
                })
            })
            .collect();

        Ok(nodes)
    }

    async fn get_nodes_by_type(&self, node_type: NodeType) -> anyhow::Result<Vec<Node>> {
        // V2 schema: Query from appropriate table based on node_type
        use iqrah_core::domain::node_id as nid;

        match node_type {
            NodeType::Verse => {
                let rows = query_as::<_, (String, i64)>("SELECT verse_key, created_at FROM verses")
                    .fetch_all(&self.pool)
                    .await?;

                Ok(rows
                    .into_iter()
                    .filter_map(|(verse_key, _created_at)| {
                        nid::parse_verse(&verse_key).ok().map(|(ch, v)| Node {
                            id: nid::encode_verse(ch, v),
                            ukey: nid::verse(ch, v),
                            node_type: NodeType::Verse,
                        })
                    })
                    .collect())
            }
            NodeType::Word => {
                let rows = query_as::<_, (i32, i64)>("SELECT word_id, created_at FROM words")
                    .fetch_all(&self.pool)
                    .await?;

                Ok(rows
                    .into_iter()
                    .map(|(word_id, _created_at)| Node {
                        id: nid::encode_word(word_id as i64),
                        ukey: nid::word(word_id as i64),
                        node_type: NodeType::Word,
                    })
                    .collect())
            }
            NodeType::Chapter => {
                let rows =
                    query_as::<_, (i32, i64)>("SELECT chapter_number, created_at FROM chapters")
                        .fetch_all(&self.pool)
                        .await?;

                Ok(rows
                    .into_iter()
                    .map(|(chapter_number, _created_at)| Node {
                        id: nid::encode_chapter(chapter_number as u8),
                        ukey: nid::chapter(chapter_number as u8),
                        node_type: NodeType::Chapter,
                    })
                    .collect())
            }
            _ => {
                // Other types (Root, Lemma, WordInstance, Knowledge) not yet supported in v2
                Ok(Vec::new())
            }
        }
    }

    async fn get_words_in_ayahs(&self, ayah_node_ids: &[i64]) -> anyhow::Result<Vec<Node>> {
        if ayah_node_ids.is_empty() {
            return Ok(Vec::new());
        }

        use iqrah_core::domain::node_id as nid;
        let mut all_words = Vec::new();

        for &ayah_id in ayah_node_ids {
            // Use decode_verse to handle "VERSE:1:1" encoded as i64
            let (chapter, verse) = match nid::decode_verse(ayah_id) {
                Some(cv) => cv,
                None => continue, // Skip invalid IDs
            };

            let verse_key = format!("{}:{}", chapter, verse);

            // Query words for this verse_key
            let rows = query_as::<_, (i32, i64)>(
                "SELECT word_id, created_at FROM words WHERE verse_key = ? ORDER BY position",
            )
            .bind(&verse_key)
            .fetch_all(&self.pool)
            .await?;

            for (word_id, _created_at) in rows {
                all_words.push(Node {
                    id: nid::encode_word(word_id as i64),
                    ukey: nid::word(word_id as i64),
                    node_type: NodeType::Word,
                });
            }
        }

        Ok(all_words)
    }

    async fn get_adjacent_words(
        &self,
        word_node_id: i64,
    ) -> anyhow::Result<(Option<Node>, Option<Node>)> {
        // V2 schema: Use words table with verse_key and position
        use iqrah_core::domain::node_id as nid;

        // Decode word_id
        let word_id = nid::decode_word(word_node_id)
            .ok_or_else(|| anyhow::anyhow!("Invalid word ID"))? as i32;

        // Get current word's verse_key and position
        let current_word =
            query_as::<_, (String, i32)>("SELECT verse_key, position FROM words WHERE word_id = ?")
                .bind(word_id)
                .fetch_optional(&self.pool)
                .await?;

        let (verse_key, position) = match current_word {
            Some(w) => w,
            None => return Ok((None, None)),
        };

        // Try to get previous word (position - 1 in same verse)
        let prev_word = query_as::<_, (i32, i64)>(
            "SELECT word_id, created_at FROM words WHERE verse_key = ? AND position = ?",
        )
        .bind(&verse_key)
        .bind(position - 1)
        .fetch_optional(&self.pool)
        .await?
        .map(|(wid, _)| Node {
            id: nid::encode_word(wid as i64),
            ukey: nid::word(wid as i64),
            node_type: NodeType::Word,
        });

        // If no previous word in current verse, try last word of previous verse
        let prev_word = if prev_word.is_none() {
            // Parse verse_key to get chapter and verse numbers
            // verse_key is "chapter:verse" e.g. "1:1"
            // parse_verse handles "1:1"
            if let Ok((chapter, verse_num)) = nid::parse_verse(&verse_key) {
                if verse_num > 1 {
                    let prev_verse_key = format!("{}:{}", chapter, verse_num - 1);
                    query_as::<_, (i32, i64)>(
                        "SELECT word_id, created_at FROM words WHERE verse_key = ? ORDER BY position DESC LIMIT 1"
                    )
                    .bind(&prev_verse_key)
                    .fetch_optional(&self.pool)
                    .await?
                    .map(|(wid, _)| Node {
                        id: nid::encode_word(wid as i64),
                        ukey: nid::word(wid as i64),
                        node_type: NodeType::Word,
                    })
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            prev_word
        };

        // Try to get next word (position + 1 in same verse)
        let next_word = query_as::<_, (i32, i64)>(
            "SELECT word_id, created_at FROM words WHERE verse_key = ? AND position = ?",
        )
        .bind(&verse_key)
        .bind(position + 1)
        .fetch_optional(&self.pool)
        .await?
        .map(|(wid, _)| Node {
            id: nid::encode_word(wid as i64),
            ukey: nid::word(wid as i64),
            node_type: NodeType::Word,
        });

        // If no next word in current verse, try first word of next verse
        let next_word = if next_word.is_none() {
            if let Ok((chapter, verse_num)) = nid::parse_verse(&verse_key) {
                let next_verse_key = format!("{}:{}", chapter, verse_num + 1);

                query_as::<_, (i32, i64)>(
                    "SELECT word_id, created_at FROM words WHERE verse_key = ? ORDER BY position ASC LIMIT 1"
                )
                .bind(&next_verse_key)
                .fetch_optional(&self.pool)
                .await?
                .map(|(wid, _)| Node {
                    id: nid::encode_word(wid as i64),
                    ukey: nid::word(wid as i64),
                    node_type: NodeType::Word,
                })
            } else {
                None
            }
        } else {
            next_word
        };

        Ok((prev_word, next_word))
    }

    // ========================================================================
    // V2 Methods (Purist relational schema)
    // ========================================================================

    async fn get_chapter(&self, chapter_number: i32) -> anyhow::Result<Option<Chapter>> {
        let row = query_as::<_, ChapterRow>(
            "SELECT chapter_number, name_arabic, name_transliteration, name_translation,
                    revelation_place, revelation_order, bismillah_pre, verse_count,
                    page_start, page_end, created_at
             FROM chapters
             WHERE chapter_number = ?",
        )
        .bind(chapter_number)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Chapter {
            number: r.chapter_number,
            name_arabic: r.name_arabic,
            name_transliteration: r.name_transliteration,
            name_translation: r.name_translation,
            revelation_place: r.revelation_place,
            verse_count: r.verse_count,
        }))
    }

    async fn get_chapters(&self) -> anyhow::Result<Vec<Chapter>> {
        let rows = query_as::<_, ChapterRow>(
            "SELECT chapter_number, name_arabic, name_transliteration, name_translation,
                    revelation_place, revelation_order, bismillah_pre, verse_count,
                    page_start, page_end, created_at
             FROM chapters
             ORDER BY chapter_number",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| Chapter {
                number: r.chapter_number,
                name_arabic: r.name_arabic,
                name_transliteration: r.name_transliteration,
                name_translation: r.name_translation,
                revelation_place: r.revelation_place,
                verse_count: r.verse_count,
            })
            .collect())
    }

    async fn get_verse(&self, verse_key: &str) -> anyhow::Result<Option<Verse>> {
        let row = query_as::<_, VerseRow>(
            "SELECT verse_key, chapter_number, verse_number, text_uthmani, text_simple,
                    juz, hizb, rub_el_hizb, page, manzil, ruku, sajdah_type, sajdah_number,
                    letter_count, word_count, created_at
             FROM verses
             WHERE verse_key = ?",
        )
        .bind(verse_key)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Verse {
            key: r.verse_key,
            chapter_number: r.chapter_number,
            verse_number: r.verse_number,
            text_uthmani: r.text_uthmani,
            text_simple: r.text_simple,
            juz: r.juz,
            page: r.page,
        }))
    }

    async fn get_verses_for_chapter(&self, chapter_number: i32) -> anyhow::Result<Vec<Verse>> {
        let rows = query_as::<_, VerseRow>(
            "SELECT verse_key, chapter_number, verse_number, text_uthmani, text_simple,
                    juz, hizb, rub_el_hizb, page, manzil, ruku, sajdah_type, sajdah_number,
                    letter_count, word_count, created_at
             FROM verses
             WHERE chapter_number = ?
             ORDER BY verse_number",
        )
        .bind(chapter_number)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| Verse {
                key: r.verse_key,
                chapter_number: r.chapter_number,
                verse_number: r.verse_number,
                text_uthmani: r.text_uthmani,
                text_simple: r.text_simple,
                juz: r.juz,
                page: r.page,
            })
            .collect())
    }

    async fn get_words_for_verse(&self, verse_key: &str) -> anyhow::Result<Vec<Word>> {
        let rows = query_as::<_, WordRow>(
            "SELECT word_id, verse_key, position, text_uthmani, text_simple, transliteration,
                    letter_count, created_at
             FROM words
             WHERE verse_key = ?
             ORDER BY position",
        )
        .bind(verse_key)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| Word {
                id: r.word_id,
                verse_key: r.verse_key,
                position: r.position,
                text_uthmani: r.text_uthmani,
                text_simple: r.text_simple,
                transliteration: r.transliteration,
            })
            .collect())
    }

    async fn get_word(&self, word_id: i32) -> anyhow::Result<Option<Word>> {
        let row = query_as::<_, WordRow>(
            "SELECT word_id, verse_key, position, text_uthmani, text_simple, transliteration,
                    letter_count, created_at
             FROM words
             WHERE word_id = ?",
        )
        .bind(word_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Word {
            id: r.word_id,
            verse_key: r.verse_key,
            position: r.position,
            text_uthmani: r.text_uthmani,
            text_simple: r.text_simple,
            transliteration: r.transliteration,
        }))
    }

    async fn get_languages(&self) -> anyhow::Result<Vec<Language>> {
        let rows = query_as::<_, LanguageRow>(
            "SELECT language_code, english_name, native_name, direction
             FROM languages
             ORDER BY english_name",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| Language {
                code: r.language_code,
                english_name: r.english_name,
                native_name: r.native_name,
                direction: r.direction,
            })
            .collect())
    }

    async fn get_language(&self, code: &str) -> anyhow::Result<Option<Language>> {
        let row = query_as::<_, LanguageRow>(
            "SELECT language_code, english_name, native_name, direction
             FROM languages
             WHERE language_code = ?",
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Language {
            code: r.language_code,
            english_name: r.english_name,
            native_name: r.native_name,
            direction: r.direction,
        }))
    }

    async fn get_translators_for_language(
        &self,
        language_code: &str,
    ) -> anyhow::Result<Vec<Translator>> {
        let rows = query_as::<_, TranslatorRow>(
            "SELECT translator_id, slug, full_name, language_code, description, copyright_holder,
                    license, website, version, package_id, created_at
             FROM translators
             WHERE language_code = ?
             ORDER BY full_name",
        )
        .bind(language_code)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| Translator {
                id: r.translator_id,
                slug: r.slug,
                full_name: r.full_name,
                language_code: r.language_code,
                description: r.description,
                license: r.license,
                package_id: r.package_id,
            })
            .collect())
    }

    async fn get_translator(&self, translator_id: i32) -> anyhow::Result<Option<Translator>> {
        let row = query_as::<_, TranslatorRow>(
            "SELECT translator_id, slug, full_name, language_code, description, copyright_holder,
                    license, website, version, package_id, created_at
             FROM translators
             WHERE translator_id = ?",
        )
        .bind(translator_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Translator {
            id: r.translator_id,
            slug: r.slug,
            full_name: r.full_name,
            language_code: r.language_code,
            description: r.description,
            license: r.license,
            package_id: r.package_id,
        }))
    }

    async fn get_translator_by_slug(&self, slug: &str) -> anyhow::Result<Option<Translator>> {
        let row = query_as::<_, TranslatorRow>(
            "SELECT translator_id, slug, full_name, language_code, description, copyright_holder,
                    license, website, version, package_id, created_at
             FROM translators
             WHERE slug = ?",
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Translator {
            id: r.translator_id,
            slug: r.slug,
            full_name: r.full_name,
            language_code: r.language_code,
            description: r.description,
            license: r.license,
            package_id: r.package_id,
        }))
    }

    async fn get_verse_translation(
        &self,
        verse_key: &str,
        translator_id: i32,
    ) -> anyhow::Result<Option<String>> {
        let row = query_as::<_, VerseTranslationRow>(
            "SELECT verse_key, translator_id, translation, footnotes, created_at
             FROM verse_translations
             WHERE verse_key = ? AND translator_id = ?",
        )
        .bind(verse_key)
        .bind(translator_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.translation))
    }

    async fn get_word_translation(
        &self,
        word_id: i32,
        translator_id: i32,
    ) -> anyhow::Result<Option<String>> {
        let row = query_as::<_, WordTranslationRow>(
            "SELECT word_id, translator_id, translation, created_at
             FROM word_translations
             WHERE word_id = ? AND translator_id = ?",
        )
        .bind(word_id)
        .bind(translator_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.translation))
    }

    async fn insert_translator(
        &self,
        slug: &str,
        full_name: &str,
        language_code: &str,
        description: Option<&str>,
        copyright_holder: Option<&str>,
        license: Option<&str>,
        website: Option<&str>,
        version: Option<&str>,
        package_id: Option<&str>,
    ) -> anyhow::Result<i32> {
        let result = query(
            "INSERT INTO translators (slug, full_name, language_code, description, copyright_holder, license, website, version, package_id)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(slug)
        .bind(full_name)
        .bind(language_code)
        .bind(description)
        .bind(copyright_holder)
        .bind(license)
        .bind(website)
        .bind(version)
        .bind(package_id)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid() as i32)
    }

    async fn insert_verse_translation(
        &self,
        verse_key: &str,
        translator_id: i32,
        translation: &str,
        footnotes: Option<&str>,
    ) -> anyhow::Result<()> {
        query(
            "INSERT INTO verse_translations (verse_key, translator_id, translation, footnotes)
             VALUES (?, ?, ?, ?)
             ON CONFLICT(verse_key, translator_id) DO UPDATE SET
                translation = excluded.translation,
                footnotes = excluded.footnotes",
        )
        .bind(verse_key)
        .bind(translator_id)
        .bind(translation)
        .bind(footnotes)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // ========================================================================
    // Package Management Implementation
    // ========================================================================

    async fn get_available_packages(
        &self,
        package_type: Option<PackageType>,
        language_code: Option<&str>,
    ) -> anyhow::Result<Vec<ContentPackage>> {
        let mut sql = String::from(
            "SELECT package_id, package_type, name, language_code, author, version, description, \
             file_size, download_url, checksum, license, created_at, updated_at \
             FROM content_packages WHERE 1=1",
        );

        if package_type.is_some() {
            sql.push_str(" AND package_type = ?");
        }
        if language_code.is_some() {
            sql.push_str(" AND language_code = ?");
        }
        sql.push_str(" ORDER BY name");

        let mut query = query_as::<_, ContentPackageRow>(&sql);

        if let Some(pt) = &package_type {
            query = query.bind(pt.to_string());
        }
        if let Some(lang) = language_code {
            query = query.bind(lang);
        }

        let rows = query.fetch_all(&self.pool).await?;

        rows.into_iter()
            .map(|r| {
                Ok(ContentPackage {
                    package_id: r.package_id,
                    package_type: r.package_type.parse()?,
                    name: r.name,
                    language_code: r.language_code,
                    author: r.author,
                    version: r.version,
                    description: r.description,
                    file_size: r.file_size,
                    download_url: r.download_url,
                    checksum: r.checksum,
                    license: r.license,
                })
            })
            .collect()
    }

    async fn get_package(&self, package_id: &str) -> anyhow::Result<Option<ContentPackage>> {
        let row = query_as::<_, ContentPackageRow>(
            "SELECT package_id, package_type, name, language_code, author, version, description, \
             file_size, download_url, checksum, license, created_at, updated_at \
             FROM content_packages WHERE package_id = ?",
        )
        .bind(package_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(ContentPackage {
                package_id: r.package_id,
                package_type: r.package_type.parse()?,
                name: r.name,
                language_code: r.language_code,
                author: r.author,
                version: r.version,
                description: r.description,
                file_size: r.file_size,
                download_url: r.download_url,
                checksum: r.checksum,
                license: r.license,
            })),
            None => Ok(None),
        }
    }

    async fn upsert_package(&self, package: &ContentPackage) -> anyhow::Result<()> {
        query(
            "INSERT INTO content_packages \
             (package_id, package_type, name, language_code, author, version, description, \
              file_size, download_url, checksum, license, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, unixepoch()) \
             ON CONFLICT(package_id) DO UPDATE SET \
                package_type = excluded.package_type, \
                name = excluded.name, \
                language_code = excluded.language_code, \
                author = excluded.author, \
                version = excluded.version, \
                description = excluded.description, \
                file_size = excluded.file_size, \
                download_url = excluded.download_url, \
                checksum = excluded.checksum, \
                license = excluded.license, \
                updated_at = unixepoch()",
        )
        .bind(&package.package_id)
        .bind(package.package_type.to_string())
        .bind(&package.name)
        .bind(&package.language_code)
        .bind(&package.author)
        .bind(&package.version)
        .bind(&package.description)
        .bind(package.file_size)
        .bind(&package.download_url)
        .bind(&package.checksum)
        .bind(&package.license)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_package(&self, package_id: &str) -> anyhow::Result<()> {
        query("DELETE FROM content_packages WHERE package_id = ?")
            .bind(package_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn get_installed_packages(&self) -> anyhow::Result<Vec<InstalledPackage>> {
        let rows = query_as::<_, InstalledPackageRow>(
            "SELECT package_id, installed_at, enabled FROM installed_packages ORDER BY installed_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|r| {
                DateTime::from_timestamp(r.installed_at, 0)
                    .map(|dt| InstalledPackage {
                        package_id: r.package_id,
                        installed_at: dt,
                        enabled: r.enabled != 0,
                    })
                    .ok_or_else(|| {
                        anyhow::anyhow!("Invalid installed_at timestamp: {}", r.installed_at)
                    })
            })
            .collect()
    }

    async fn is_package_installed(&self, package_id: &str) -> anyhow::Result<bool> {
        let count: i32 =
            query_as("SELECT COUNT(*) as count FROM installed_packages WHERE package_id = ?")
                .bind(package_id)
                .fetch_one(&self.pool)
                .await
                .map(|(count,): (i32,)| count)?;

        Ok(count > 0)
    }

    async fn mark_package_installed(&self, package_id: &str) -> anyhow::Result<()> {
        query(
            "INSERT INTO installed_packages (package_id, installed_at, enabled) \
             VALUES (?, unixepoch(), 1) \
             ON CONFLICT(package_id) DO UPDATE SET \
                enabled = 1, \
                installed_at = unixepoch()",
        )
        .bind(package_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn mark_package_uninstalled(&self, package_id: &str) -> anyhow::Result<()> {
        query("DELETE FROM installed_packages WHERE package_id = ?")
            .bind(package_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn enable_package(&self, package_id: &str) -> anyhow::Result<()> {
        query("UPDATE installed_packages SET enabled = 1 WHERE package_id = ?")
            .bind(package_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn disable_package(&self, package_id: &str) -> anyhow::Result<()> {
        query("UPDATE installed_packages SET enabled = 0 WHERE package_id = ?")
            .bind(package_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn get_enabled_packages(&self) -> anyhow::Result<Vec<InstalledPackage>> {
        let rows = query_as::<_, InstalledPackageRow>(
            "SELECT package_id, installed_at, enabled FROM installed_packages WHERE enabled = 1 ORDER BY installed_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|r| {
                DateTime::from_timestamp(r.installed_at, 0)
                    .map(|dt| InstalledPackage {
                        package_id: r.package_id,
                        installed_at: dt,
                        enabled: true,
                    })
                    .ok_or_else(|| {
                        anyhow::anyhow!("Invalid installed_at timestamp: {}", r.installed_at)
                    })
            })
            .collect()
    }

    // ========================================================================
    // Morphology Methods (for grammar exercises)
    // ========================================================================

    async fn get_morphology_for_word(
        &self,
        word_id: i32,
    ) -> anyhow::Result<Vec<MorphologySegment>> {
        let rows = query_as::<_, MorphologySegmentRow>(
            "SELECT segment_id, word_id, position, lemma_id, root_id, pos_tag
             FROM morphology_segments
             WHERE word_id = ?
             ORDER BY position",
        )
        .bind(word_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| MorphologySegment {
                segment_id: r.segment_id,
                word_id: r.word_id,
                position: r.position,
                lemma_id: r.lemma_id,
                root_id: r.root_id,
                pos_tag: r.pos_tag,
            })
            .collect())
    }

    async fn get_root_by_id(&self, root_id: &str) -> anyhow::Result<Option<Root>> {
        let row = query_as::<_, RootRow>(
            "SELECT root_id, arabic, transliteration, root_type, meaning, created_at
             FROM roots
             WHERE root_id = ?",
        )
        .bind(root_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Root {
            root_id: r.root_id,
            arabic: r.arabic,
            transliteration: r.transliteration,
            root_type: r.root_type.unwrap_or_else(|| "trilateral".to_string()),
        }))
    }

    async fn get_lemma_by_id(&self, lemma_id: &str) -> anyhow::Result<Option<Lemma>> {
        let row = query_as::<_, LemmaRow>(
            "SELECT lemma_id, arabic, transliteration, root_id, description, created_at
             FROM lemmas
             WHERE lemma_id = ?",
        )
        .bind(lemma_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Lemma {
            lemma_id: r.lemma_id,
            arabic: r.arabic,
            root_id: r.root_id,
            transliteration: r.transliteration,
        }))
    }

    // ========================================================================
    // Scheduler v2.0 Methods
    // ========================================================================

    async fn get_scheduler_candidates(&self, goal_id: &str) -> anyhow::Result<Vec<CandidateNode>> {
        // Fetch node metadata from content.db
        // Note: energy and next_due_ts are set to defaults here
        // The caller should fetch memory states from user repository and merge
        let rows = query_as::<_, CandidateNodeRow>(
            "SELECT
                ng.node_id AS node_id,
                COALESCE(m_found.value, 0.0) AS foundational_score,
                COALESCE(m_infl.value, 0.0) AS influence_score,
                COALESCE(m_diff.value, 0.0) AS difficulty_score,
                CAST(COALESCE(m_quran.value, 0) AS INTEGER) AS quran_order
            FROM node_goals ng
            LEFT JOIN node_metadata m_found
                ON ng.node_id = m_found.node_id AND m_found.key = 'foundational_score'
            LEFT JOIN node_metadata m_infl
                ON ng.node_id = m_infl.node_id AND m_infl.key = 'influence_score'
            LEFT JOIN node_metadata m_diff
                ON ng.node_id = m_diff.node_id AND m_diff.key = 'difficulty_score'
            LEFT JOIN node_metadata m_quran
                ON ng.node_id = m_quran.node_id AND m_quran.key = 'quran_order'
            WHERE ng.goal_id = ?
            ORDER BY ng.priority DESC, ng.node_id ASC",
        )
        .bind(goal_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| CandidateNode {
                id: r.node_id,
                foundational_score: r.foundational_score,
                influence_score: r.influence_score,
                difficulty_score: r.difficulty_score,
                energy: 0.0,    // Default - caller should merge from user repo
                next_due_ts: 0, // Default - caller should merge from user repo
                quran_order: r.quran_order,
            })
            .collect())
    }

    async fn get_prerequisite_parents(
        &self,
        node_ids: &[i64],
    ) -> anyhow::Result<HashMap<i64, Vec<i64>>> {
        if node_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let mut result: HashMap<i64, Vec<i64>> = HashMap::new();

        // SQLite parameter limit is ~999, so chunk into batches of 500
        const CHUNK_SIZE: usize = 500;

        for chunk in node_ids.chunks(CHUNK_SIZE) {
            // Build parameterized query
            let placeholders = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
            let sql = format!(
                "SELECT target_id AS node_id, source_id AS parent_id
                 FROM edges
                 WHERE edge_type = 0 AND target_id IN ({})",
                placeholders
            );

            let mut query = query_as::<_, PrerequisiteRow>(&sql);
            for node_id in chunk {
                query = query.bind(node_id);
            }

            let rows = query.fetch_all(&self.pool).await?;

            // Group by node_id
            for row in rows {
                result.entry(row.node_id).or_default().push(row.parent_id);
            }
        }

        Ok(result)
    }

    async fn get_goal(&self, goal_id: &str) -> anyhow::Result<Option<SchedulerGoal>> {
        let row = query_as::<_, GoalRow>(
            "SELECT goal_id, goal_type, goal_group, label, description
             FROM goals
             WHERE goal_id = ?",
        )
        .bind(goal_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| SchedulerGoal {
            goal_id: r.goal_id,
            goal_type: r.goal_type,
            goal_group: r.goal_group,
            label: r.label,
            description: r.description,
        }))
    }

    async fn get_nodes_for_goal(&self, goal_id: &str) -> anyhow::Result<Vec<i64>> {
        let rows = query_as::<_, NodeGoalRow>(
            "SELECT node_id
             FROM node_goals
             WHERE goal_id = ?
             ORDER BY priority DESC, node_id ASC",
        )
        .bind(goal_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.node_id).collect())
    }

    async fn get_verses_batch(
        &self,
        verse_keys: &[String],
    ) -> anyhow::Result<HashMap<String, Verse>> {
        if verse_keys.is_empty() {
            return Ok(HashMap::new());
        }

        // Build placeholders for IN clause
        let placeholders = vec!["?"; verse_keys.len()].join(", ");
        let query_str = format!(
            "SELECT verse_key, chapter_number, verse_number, text_uthmani, text_simple,
                    juz, page
             FROM verses
             WHERE verse_key IN ({})",
            placeholders
        );

        let mut query = sqlx::query_as::<_, VerseRow>(&query_str);
        for key in verse_keys {
            query = query.bind(key);
        }

        let rows = query.fetch_all(&self.pool).await?;

        let mut result = HashMap::new();
        for row in rows {
            result.insert(
                row.verse_key.clone(),
                Verse {
                    key: row.verse_key,
                    chapter_number: row.chapter_number,
                    verse_number: row.verse_number,
                    text_uthmani: row.text_uthmani,
                    text_simple: row.text_simple,
                    juz: row.juz,
                    page: row.page,
                },
            );
        }

        Ok(result)
    }

    async fn get_words_batch(&self, word_ids: &[i32]) -> anyhow::Result<HashMap<i32, Word>> {
        if word_ids.is_empty() {
            return Ok(HashMap::new());
        }

        // Build placeholders for IN clause
        let placeholders = vec!["?"; word_ids.len()].join(", ");
        let query_str = format!(
            "SELECT word_id, verse_key, position, text_uthmani, text_simple, transliteration
             FROM words
             WHERE word_id IN ({})",
            placeholders
        );

        let mut query = sqlx::query_as::<_, WordRow>(&query_str);
        for id in word_ids {
            query = query.bind(id);
        }

        let rows = query.fetch_all(&self.pool).await?;

        let mut result = HashMap::new();
        for row in rows {
            result.insert(
                row.word_id,
                Word {
                    id: row.word_id,
                    verse_key: row.verse_key,
                    position: row.position,
                    text_uthmani: row.text_uthmani,
                    text_simple: row.text_simple,
                    transliteration: row.transliteration,
                },
            );
        }

        Ok(result)
    }
}
