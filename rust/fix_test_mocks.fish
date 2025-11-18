#!/usr/bin/env fish
# Script to add batch method implementations to all mock ContentRepository test files
# Run from: rust/ directory

set files \
    crates/iqrah-core/src/exercises/find_mistake_tests.rs \
    crates/iqrah-core/src/exercises/graph_tests.rs \
    crates/iqrah-core/src/exercises/translate_phrase_tests.rs \
    crates/iqrah-core/src/exercises/full_verse_input_tests.rs \
    crates/iqrah-core/src/exercises/grammar_tests.rs \
    crates/iqrah-core/src/exercises/ayah_sequence_tests.rs \
    crates/iqrah-core/src/exercises/reverse_cloze_tests.rs \
    crates/iqrah-core/src/exercises/translation_tests.rs \
    crates/iqrah-core/src/exercises/pos_tagging_tests.rs \
    crates/iqrah-core/src/exercises/memorization_tests.rs \
    crates/iqrah-core/src/services/session_service_tests.rs \
    crates/iqrah-core/src/services/learning_service_tests.rs

echo "Adding batch methods to test mocks..."

for file in $files
    echo "Processing $file"
    
    # Find the line with "async fn get_nodes_for_goal" and insert batch methods before the closing brace
    # This is a placeholder - manual editing required due to fish shell limitations
    
    # For manual fix: Add these two methods before the final closing brace of impl ContentRepository
end

echo ""
echo "Manual fix required for each file:"
echo "Add these methods before the closing brace of impl ContentRepository for MockContentRepo:"
echo ""
echo '    async fn get_verses_batch('
echo '        &self,'
echo '        verse_keys: &[String],'
echo '    ) -> anyhow::Result<std::collections::HashMap<String, crate::Verse>> {'
echo '        let mut result = std::collections::HashMap::new();'
echo '        for key in verse_keys {'
echo '            if let Some(verse) = self.get_verse(key).await? {'
echo '                result.insert(key.clone(), verse);'
echo '            }'
echo '        }'
echo '        Ok(result)'
echo '    }'
echo ''
echo '    async fn get_words_batch('
echo '        &self,'
echo '        word_ids: &[i32],'
echo '    ) -> anyhow::Result<std::collections::HashMap<i32, crate::Word>> {'
echo '        let mut result = std::collections::HashMap::new();'
echo '        for &id in word_ids {'
echo '            if let Some(word) = self.get_word(id).await? {'
echo '                result.insert(id, word);'
echo '            }'
echo '        }'
echo '        Ok(result)'
echo '    }'
echo ""
echo "Files to update:"
for file in $files
    echo "  - $file"
end
