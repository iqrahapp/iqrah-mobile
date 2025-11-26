use super::models::{
    ChapterRow, ContentPackageRow, GoalRow, InstalledPackageRow,
    LanguageRow, LemmaRow, MorphologySegmentRow, PrerequisiteRow, RootRow,
    TranslatorRow, VerseRow, VerseTranslationRow, WordRow, WordTranslationRow,
};
use async_trait::async_trait;
use chrono::DateTime;
use iqrah_core::{
    domain::node_id as nid, ports::content_repository::SchedulerGoal,
    scheduler_v2::CandidateNode, Chapter, ContentPackage, ContentRepository, DistributionType, Edge,
    EdgeType, ImportedEdge, ImportedNode, InstalledPackage, KnowledgeNode, Language, Lemma,
    MorphologySegment, Node, NodeType, PackageType, Root, Translator, Verse, Word,
};
use super::node_registry::NodeRegistry;
use sqlx::{query, query_as, SqlitePool};
use std::{borrow::Cow, collections::HashMap, sync::Arc};

pub struct SqliteContentRepository {
    pool: SqlitePool,
    registry: Arc<NodeRegistry>,
}

impl SqliteContentRepository {
    pub fn new(pool: SqlitePool, registry: Arc<NodeRegistry>) -> Self {
        Self { pool, registry }
    }

    /// Helper to get the base node ID if a knowledge axis is present
    fn get_base_id<'a>(&self, node_id: &'a str) -> Cow<'a, str> {
        if let Ok((base_id, _)) = nid::parse_knowledge(node_id) {
            Cow::Owned(base_id)
        } else {
            Cow::Borrowed(node_id)
        }
    }

    /// Retrieves a node by its integer ID.
    /// This is the fast, internal method for node lookups.
    pub async fn get_node_by_id(&self, node_id: i64) -> anyhow::Result<Option<Node>> {
        let row: Option<(String, String, Option<String>, Option<String>)> = query_as(
            r#"
            SELECT
                n.ukey,
                n.node_type,
                kn.base_node_id,
                kn.axis
            FROM nodes n
            LEFT JOIN knowledge_nodes kn ON kn.node_id = n.id
            WHERE n.id = ?
            "#,
        )
        .bind(node_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some((ukey, node_type, base_node_ukey, axis)) => {
                let node_type: NodeType = node_type.parse()?;
                let knowledge_node = if node_type == NodeType::Knowledge {
                    Some(KnowledgeNode {
                        base_node_id: base_node_ukey.unwrap(), // Should be present for knowledge nodes
                        axis: axis.unwrap().parse()?, // Should be present for knowledge nodes
                        full_id: ukey.clone(),
                    })
                } else {
                    None
                };

                Ok(Some(Node {
                    id: ukey,
                    node_type,
                    knowledge_node,
                }))
            }
            None => Ok(None),
        }
    }
}

#[async_trait]
impl ContentRepository for SqliteContentRepository {
    async fn get_node(&self, ukey: &str) -> anyhow::Result<Option<Node>> {
        // 1. Lookup integer ID from string key via the registry
        let node_id_opt = self.registry.get_id(ukey).await?;

        if let Some(node_id) = node_id_opt {
            // 2. Use the fast, internal integer-based path
            self.get_node_by_id(node_id).await
        } else {
            Ok(None)
        }
    }

    async fn get_edges_from(&self, source_ukey: &str) -> anyhow::Result<Vec<Edge>> {
        let source_id = match self.registry.get_id(source_ukey).await? {
            Some(id) => id,
            None => return Ok(Vec::new()),
        };

        let rows: Vec<(String, String, i32, i32, f64, f64)> = query_as(
            r#"
            SELECT
                source_node.ukey AS source_ukey,
                target_node.ukey AS target_ukey,
                e.edge_type,
                e.distribution_type,
                e.param1,
                e.param2
            FROM edges e
            JOIN nodes source_node ON e.source_id = source_node.id
            JOIN nodes target_node ON e.target_id = target_node.id
            WHERE e.source_id = ?
            "#,
        )
        .bind(source_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(source_ukey, target_ukey, edge_type, distribution_type, param1, param2)| Edge {
                source_id: source_ukey,
                target_id: target_ukey,
                edge_type: (edge_type).try_into().unwrap_or(EdgeType::Dependency),
                distribution_type: (distribution_type)
                    .try_into()
                    .unwrap_or(DistributionType::Const),
                param1,
                param2,
            })
            .collect())
    }

    async fn get_edges_to(&self, target_ukey: &str) -> anyhow::Result<Vec<Edge>> {
        let target_id = match self.registry.get_id(target_ukey).await? {
            Some(id) => id,
            None => return Ok(Vec::new()),
        };

        let rows: Vec<(String, String, i32, i32, f64, f64)> = query_as(
            r#"
            SELECT
                source_node.ukey AS source_ukey,
                target_node.ukey AS target_ukey,
                e.edge_type,
                e.distribution_type,
                e.param1,
                e.param2
            FROM edges e
            JOIN nodes source_node ON e.source_id = source_node.id
            JOIN nodes target_node ON e.target_id = target_node.id
            WHERE e.target_id = ?
            "#,
        )
        .bind(target_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(source_ukey, target_ukey, edge_type, distribution_type, param1, param2)| Edge {
                source_id: source_ukey,
                target_id: target_ukey,
                edge_type: (edge_type).try_into().unwrap_or(EdgeType::Dependency),
                distribution_type: (distribution_type)
                    .try_into()
                    .unwrap_or(DistributionType::Const),
                param1,
                param2,
            })
            .collect())
    }

    async fn get_quran_text(&self, ukey: &str) -> anyhow::Result<Option<String>> {
        let node_id = match self.registry.get_id(ukey).await? {
            Some(id) => id,
            None => return Ok(None),
        };

        // This query joins nodes with verses/words to get the text using the integer ID
        let row: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT COALESCE(v.text_uthmani, w.text_uthmani)
            FROM nodes n
            LEFT JOIN verses v ON n.ukey = v.verse_key
            LEFT JOIN words w ON n.ukey = w.word_id_str -- Assuming word_id is stored as string in nodes.ukey
            WHERE n.id = ?
            "#,
        )
        .bind(node_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(text,)| text))
    }

    async fn get_translation(&self, ukey: &str, lang: &str) -> anyhow::Result<Option<String>> {
        let node_id = match self.registry.get_id(ukey).await? {
            Some(id) => id,
            None => return Ok(None),
        };

        let translator: Option<(i32,)> =
            query_as("SELECT translator_id FROM translators WHERE language_code = ? LIMIT 1")
                .bind(lang)
                .fetch_optional(&self.pool)
                .await?;

        let translator_id = match translator {
            Some((id,)) => id,
            None => return Ok(None),
        };

        let row: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT COALESCE(vt.translation, wt.translation)
            FROM nodes n
            LEFT JOIN verse_translations vt ON n.ukey = vt.verse_key AND vt.translator_id = ?1
            LEFT JOIN word_translations wt ON n.ukey = wt.word_id_str AND wt.translator_id = ?1 -- Assuming word_id is stored as string in nodes.ukey
            WHERE n.id = ?2
            "#,
        )
        .bind(translator_id)
        .bind(node_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(text,)| text))
    }

    async fn get_metadata(&self, ukey: &str, key: &str) -> anyhow::Result<Option<String>> {
        // For backwards compatibility, map old metadata keys to new tables
        match key {
            "arabic" => self.get_quran_text(ukey).await,
            "translation" => self.get_translation(ukey, "en").await,
            _ => {
                // Unknown key - return None
                Ok(None)
            }
        }
    }

    async fn get_all_metadata(&self, ukey: &str) -> anyhow::Result<HashMap<String, String>> {
        let mut metadata = HashMap::new();

        // Get arabic text
        if let Some(arabic) = self.get_quran_text(ukey).await? {
            metadata.insert("arabic".to_string(), arabic);
        }

        // Get translation (default to English)
        if let Some(translation) = self.get_translation(ukey, "en").await? {
            metadata.insert("translation".to_string(), translation);
        }

        Ok(metadata)
    }

    async fn node_exists(&self, ukey: &str) -> anyhow::Result<bool> {
        self.registry.get_id(ukey).await.map(|id| id.is_some())
    }

    async fn get_all_nodes(&self) -> anyhow::Result<Vec<Node>> {
        let rows: Vec<(String, String, Option<String>, Option<String>)> = query_as(
            r#"
            SELECT
                n.ukey,
                n.node_type,
                kn.base_node_id,
                kn.axis
            FROM nodes n
            LEFT JOIN knowledge_nodes kn ON kn.node_id = n.id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|(ukey, node_type, base_node_ukey, axis)| {
                let node_type: NodeType = node_type.parse()?;
                let knowledge_node = if node_type == NodeType::Knowledge {
                    Some(KnowledgeNode {
                        base_node_id: base_node_ukey.unwrap(),
                        axis: axis.unwrap().parse()?,
                        full_id: ukey.clone(),
                    })
                } else {
                    None
                };

                Ok(Node {
                    id: ukey,
                    node_type,
                    knowledge_node,
                })
            })
            .collect()
    }

    async fn get_nodes_by_type(&self, node_type: NodeType) -> anyhow::Result<Vec<Node>> {
        let rows: Vec<(String, String, Option<String>, Option<String>)> = query_as(
            r#"
            SELECT
                n.ukey,
                n.node_type,
                kn.base_node_id,
                kn.axis
            FROM nodes n
            LEFT JOIN knowledge_nodes kn ON kn.node_id = n.id
            WHERE n.node_type = ?
            "#,
        )
        .bind(node_type.to_string())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|(ukey, node_type, base_node_ukey, axis)| {
                let node_type: NodeType = node_type.parse()?;
                let knowledge_node = if node_type == NodeType::Knowledge {
                    Some(KnowledgeNode {
                        base_node_id: base_node_ukey.unwrap(),
                        axis: axis.unwrap().parse()?,
                        full_id: ukey.clone(),
                    })
                } else {
                    None
                };

                Ok(Node {
                    id: ukey,
                    node_type,
                    knowledge_node,
                })
            })
            .collect()
    }

    async fn insert_nodes_batch(&self, nodes: &[ImportedNode]) -> anyhow::Result<()> {
        // V2 schema: Content data (verses, words, chapters) is populated by migrations
        // This function is now a no-op for v2 schema since:
        // - Verses are in the 'verses' table (populated by migrations)
        // - Words are in the 'words' table (populated by migrations)
        // - Chapters are in the 'chapters' table (populated by migrations)
        // - The generic 'nodes' table no longer exists in v2 purist schema

        if !nodes.is_empty() {
            tracing::debug!(
                "insert_nodes_batch called with {} nodes (no-op in v2 schema, data populated by migrations)",
                nodes.len()
            );
        }

        Ok(())
    }

    async fn insert_edges_batch(&self, edges: &[ImportedEdge]) -> anyhow::Result<()> {
        for edge in edges {
            let source_id = self.registry.get_id(&edge.source_id).await?;
            let target_id = self.registry.get_id(&edge.target_id).await?;

            if let (Some(source_id), Some(target_id)) = (source_id, target_id) {
                let edge_type = edge.edge_type as i32;
                let dist_type = edge.distribution_type as i32;

                query(
                    "INSERT OR IGNORE INTO edges (source_id, target_id, edge_type, distribution_type, param1, param2)
                     VALUES (?, ?, ?, ?, ?, ?)",
                )
                .bind(source_id)
                .bind(target_id)
                .bind(edge_type)
                .bind(dist_type)
                .bind(edge.param1)
                .bind(edge.param2)
                .execute(&self.pool)
                .await?;
            }
        }

        Ok(())
    }

    async fn get_words_in_ayahs(&self, ayah_node_ids: &[String]) -> anyhow::Result<Vec<Node>> {
        if ayah_node_ids.is_empty() {
            return Ok(Vec::new());
        }

        use iqrah_core::domain::node_id as nid;
        let mut all_words = Vec::new();

        for ayah_id in ayah_node_ids {
            // Use parse_verse to handle both "VERSE:1:1" and "1:1" formats safely
            let (chapter, verse) = match nid::parse_verse(ayah_id) {
                Ok(cv) => cv,
                Err(_) => continue, // Skip invalid IDs
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
                    id: nid::word(word_id as i64),
                    node_type: NodeType::Word,
                    knowledge_node: None,
                });
            }
        }

        Ok(all_words)
    }

    async fn get_adjacent_words(
        &self,
        word_ukey: &str,
    ) -> anyhow::Result<(Option<Node>, Option<Node>)> {
        let word_id = match self.registry.get_id(word_ukey).await? {
            Some(id) => id,
            None => return Ok((None, None)),
        };

        let current_word =
            query_as::<_, (String, i32)>("SELECT verse_key, position FROM words WHERE word_id = (SELECT internal_id FROM nodes WHERE id = ?)")
                .bind(word_id)
                .fetch_optional(&self.pool)
                .await?;

        let (verse_key, position) = match current_word {
            Some(w) => w,
            None => return Ok((None, None)),
        };

        let prev_word_row = query_as::<_, (String,)>(
            "SELECT n.ukey FROM nodes n JOIN words w ON n.ukey = w.word_id_str WHERE w.verse_key = ? AND w.position = ?",
        )
        .bind(&verse_key)
        .bind(position - 1)
        .fetch_optional(&self.pool)
        .await?;

        let prev_word = if let Some((ukey,)) = prev_word_row {
            self.get_node(&ukey).await?
        } else {
            None
        };

        let next_word_row = query_as::<_, (String,)>(
            "SELECT n.ukey FROM nodes n JOIN words w ON n.ukey = w.word_id_str WHERE w.verse_key = ? AND w.position = ?",
        )
        .bind(&verse_key)
        .bind(position + 1)
        .fetch_optional(&self.pool)
        .await?;

        let next_word = if let Some((ukey,)) = next_word_row {
            self.get_node(&ukey).await?
        } else {
            None
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

    async fn get_scheduler_candidates(
        &self,
        _goal_id: &str,
        _user_id: &str,
        _now_ts: i64,
    ) -> anyhow::Result<Vec<CandidateNode>> {
        Ok(vec![])
    }

    async fn get_prerequisite_parents(
        &self,
        ukeys: &[String],
    ) -> anyhow::Result<HashMap<String, Vec<String>>> {
        if ukeys.is_empty() {
            return Ok(HashMap::new());
        }

        let mut node_ids = Vec::new();
        for ukey in ukeys {
            if let Some(id) = self.registry.get_id(ukey).await? {
                node_ids.push(id);
            }
        }

        if node_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let mut result: HashMap<String, Vec<String>> = HashMap::new();
        const CHUNK_SIZE: usize = 500;

        for chunk in node_ids.chunks(CHUNK_SIZE) {
            let placeholders = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
            let sql = format!(
                "SELECT n_target.ukey AS node_id, n_source.ukey AS parent_id
                 FROM edges e
                 JOIN nodes n_target ON e.target_id = n_target.id
                 JOIN nodes n_source ON e.source_id = n_source.id
                 WHERE e.edge_type = 0 AND e.target_id IN ({})",
                placeholders
            );

            let mut query = query_as::<_, PrerequisiteRow>(&sql);
            for node_id in chunk {
                query = query.bind(node_id);
            }

            let rows = query.fetch_all(&self.pool).await?;

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

    async fn get_nodes_for_goal(&self, goal_id: &str) -> anyhow::Result<Vec<String>> {
        let rows: Vec<(String,)> = query_as(
            r#"
            SELECT n.ukey
            FROM node_goals ng
            JOIN nodes n ON ng.node_id = n.id
            WHERE ng.goal_id = ?
            ORDER BY ng.priority DESC, n.ukey ASC
            "#,
        )
        .bind(goal_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.0).collect())
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
