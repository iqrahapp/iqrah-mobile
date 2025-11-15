# Quran Similarity Data – Overview

This directory contains two independent similarity datasets:

1. **Mutashabihat (Shared Phrases)**  
   Repeated word-phrases across ayahs.  
   → See: `mutashabihat_phrases.md`  
   → See: `mutashabihat_phrase_index_by_ayah.md`

2. **Similar Ayah Pairs (QUL)**  
   Ayahs sharing similar wording/expressions.  
   → See: `similar_ayah_pairs_json.md`  
   → See: `similar_ayah_pairs_sqlite.md`

Use cases:
- highlight shared phrases in ayahs,  
- detect confusingly similar ayahs for memorization,  
- explore repeated themes/wording.

All systems assume a **word-by-word Quran script**, where ayah words = array of tokens.
