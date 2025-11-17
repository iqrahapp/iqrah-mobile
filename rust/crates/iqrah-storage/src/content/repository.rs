use super::models::{
    ChapterRow, ContentPackageRow, EdgeRow, InstalledPackageRow, LanguageRow, LemmaRow,
    MorphologySegmentRow, NodeRow, QuranTextRow, RootRow, TranslationRow, TranslatorRow, VerseRow,
    VerseTranslationRow, WordRow, WordTranslationRow,
};
use async_trait::async_trait;
use chrono::DateTime;
use iqrah_core::{
    Chapter, ContentPackage, ContentRepository, DistributionType, Edge, EdgeType, ImportedEdge,
    ImportedNode, InstalledPackage, KnowledgeNode, Language, Lemma, MorphologySegment, Node,
    NodeType, PackageType, Root, Translator, Verse, Word,
};
use sqlx::{query, query_as, SqlitePool};
use std::collections::HashMap;

pub struct SqliteContentRepository {
    pool: SqlitePool,
}

impl SqliteContentRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Helper to construct a Node from a NodeRow, parsing knowledge_node if applicable
    fn node_from_row(row: NodeRow) -> Node {
        let node_type = NodeType::from(row.node_type.clone());
        let knowledge_node = if node_type == NodeType::Knowledge {
            KnowledgeNode::parse(&row.id)
        } else {
            None
        };

        Node {
            id: row.id,
            node_type,
            knowledge_node,
        }
    }
}

#[async_trait]
impl ContentRepository for SqliteContentRepository {
    async fn get_node(&self, node_id: &str) -> anyhow::Result<Option<Node>> {
        let row =
            query_as::<_, NodeRow>("SELECT id, node_type, created_at FROM nodes WHERE id = ?")
                .bind(node_id)
                .fetch_optional(&self.pool)
                .await?;

        Ok(row.map(Self::node_from_row))
    }

    async fn get_edges_from(&self, source_id: &str) -> anyhow::Result<Vec<Edge>> {
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

    async fn get_quran_text(&self, node_id: &str) -> anyhow::Result<Option<String>> {
        let row =
            query_as::<_, QuranTextRow>("SELECT node_id, arabic FROM quran_text WHERE node_id = ?")
                .bind(node_id)
                .fetch_optional(&self.pool)
                .await?;

        Ok(row.map(|r| r.arabic))
    }

    async fn get_translation(&self, node_id: &str, lang: &str) -> anyhow::Result<Option<String>> {
        let row = query_as::<_, TranslationRow>(
            "SELECT node_id, language_code, translation
             FROM translations
             WHERE node_id = ? AND language_code = ?",
        )
        .bind(node_id)
        .bind(lang)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.translation))
    }

    async fn get_metadata(&self, node_id: &str, key: &str) -> anyhow::Result<Option<String>> {
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

    async fn get_all_metadata(&self, node_id: &str) -> anyhow::Result<HashMap<String, String>> {
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

    async fn node_exists(&self, node_id: &str) -> anyhow::Result<bool> {
        let count: (i64,) = query_as("SELECT COUNT(*) FROM nodes WHERE id = ?")
            .bind(node_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(count.0 > 0)
    }

    async fn get_all_nodes(&self) -> anyhow::Result<Vec<Node>> {
        let rows = query_as::<_, NodeRow>("SELECT id, node_type, created_at FROM nodes")
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Self::node_from_row).collect())
    }

    async fn get_nodes_by_type(&self, node_type: NodeType) -> anyhow::Result<Vec<Node>> {
        let type_str = node_type.to_string();

        let rows = query_as::<_, NodeRow>(
            "SELECT id, node_type, created_at FROM nodes WHERE node_type = ?",
        )
        .bind(&type_str)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Self::node_from_row).collect())
    }

    async fn insert_nodes_batch(&self, nodes: &[ImportedNode]) -> anyhow::Result<()> {
        // Batch insert nodes
        for node in nodes {
            let node_type_str = node.node_type.to_string();
            query("INSERT OR IGNORE INTO nodes (id, node_type, created_at) VALUES (?, ?, ?)")
                .bind(&node.id)
                .bind(&node_type_str)
                .bind(node.created_at)
                .execute(&self.pool)
                .await?;

            // Insert metadata into quran_text and translations tables
            if let Some(arabic) = node.metadata.get("arabic") {
                query("INSERT OR IGNORE INTO quran_text (node_id, arabic) VALUES (?, ?)")
                    .bind(&node.id)
                    .bind(arabic)
                    .execute(&self.pool)
                    .await?;
            }

            if let Some(translation) = node.metadata.get("translation") {
                query(
                    "INSERT OR IGNORE INTO translations (node_id, language_code, translation) VALUES (?, ?, ?)"
                )
                .bind(&node.id)
                .bind("en")
                .bind(translation)
                .execute(&self.pool)
                .await?;
            }
        }

        Ok(())
    }

    async fn insert_edges_batch(&self, edges: &[ImportedEdge]) -> anyhow::Result<()> {
        // Batch insert edges
        for edge in edges {
            let edge_type = match edge.edge_type {
                EdgeType::Dependency => 0,
                EdgeType::Knowledge => 1,
            };

            let dist_type = match edge.distribution_type {
                DistributionType::Const => 0,
                DistributionType::Normal => 1,
                DistributionType::Beta => 2,
            };

            query(
                "INSERT OR IGNORE INTO edges (source_id, target_id, edge_type, distribution_type, param1, param2)
                 VALUES (?, ?, ?, ?, ?, ?)"
            )
            .bind(&edge.source_id)
            .bind(&edge.target_id)
            .bind(edge_type)
            .bind(dist_type)
            .bind(edge.param1)
            .bind(edge.param2)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    async fn get_words_in_ayahs(&self, ayah_node_ids: &[String]) -> anyhow::Result<Vec<Node>> {
        if ayah_node_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Parse ayah IDs to get chapter:verse pairs
        // Expected format: "VERSE:chapter:verse"
        let mut word_ids = Vec::new();
        for ayah_id in ayah_node_ids {
            let parts: Vec<&str> = ayah_id.split(':').collect();
            if parts.len() >= 3 && parts[0] == "VERSE" {
                let chapter = parts[1];
                let verse = parts[2];

                // Get all WORD nodes for this verse by querying with LIKE pattern
                // Format: "WORD:chapter:verse:position"
                let pattern = format!("WORD:{}:{}:%", chapter, verse);
                let rows = query_as::<_, NodeRow>(
                    "SELECT id, node_type, created_at FROM nodes WHERE id LIKE ? AND node_type = 'word' ORDER BY id"
                )
                .bind(&pattern)
                .fetch_all(&self.pool)
                .await?;

                for row in rows {
                    word_ids.push(Self::node_from_row(row));
                }
            }
        }

        Ok(word_ids)
    }

    async fn get_adjacent_words(
        &self,
        word_node_id: &str,
    ) -> anyhow::Result<(Option<Node>, Option<Node>)> {
        // Parse word ID: "WORD:chapter:verse:position"
        let parts: Vec<&str> = word_node_id.split(':').collect();
        if parts.len() != 4 || parts[0] != "WORD" {
            return Ok((None, None));
        }

        let chapter: i32 = parts[1].parse()?;
        let verse: i32 = parts[2].parse()?;
        let position: i32 = parts[3].parse()?;

        // Try to get previous word (position - 1 in same verse)
        let prev_word_id = format!("WORD:{}:{}:{}", chapter, verse, position - 1);
        let prev_word = self.get_node(&prev_word_id).await?;

        // If no previous word in current verse, try last word of previous verse
        let prev_word = if prev_word.is_none() && verse > 1 {
            // Find the last word of the previous verse by querying in reverse order
            let pattern = format!("WORD:{}:{}:%", chapter, verse - 1);
            let row = query_as::<_, NodeRow>(
                "SELECT id, node_type, created_at FROM nodes WHERE id LIKE ? AND node_type = 'word' ORDER BY id DESC LIMIT 1"
            )
            .bind(&pattern)
            .fetch_optional(&self.pool)
            .await?;

            row.map(Self::node_from_row)
        } else {
            prev_word
        };

        // Try to get next word (position + 1 in same verse)
        let next_word_id = format!("WORD:{}:{}:{}", chapter, verse, position + 1);
        let next_word = self.get_node(&next_word_id).await?;

        // If no next word in current verse, try first word of next verse
        let next_word = if next_word.is_none() {
            // Find the first word of the next verse by querying in order
            let pattern = format!("WORD:{}:{}:%", chapter, verse + 1);
            let row = query_as::<_, NodeRow>(
                "SELECT id, node_type, created_at FROM nodes WHERE id LIKE ? AND node_type = 'word' ORDER BY id ASC LIMIT 1"
            )
            .bind(&pattern)
            .fetch_optional(&self.pool)
            .await?;

            row.map(Self::node_from_row)
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
}
