Purpose: JSON dataset from QUL showing ayah-to-ayah similarity based on word overlap.

Top-level:
{
  "source_ayah": [SimilarityRecord, ...]
}

### SimilarityRecord
```jsonc
{
  "matched_ayah_key": "s:a",   // ayah with similar wording
  "matched_words_count": <int>,// number of overlapping words
  "coverage": <int>,           // % of source ayah words matched (0-100)
  "score": <int>,              // overall similarity score (0-100)
  "match_words_range": [start, end] // matching segment in matched ayah (1-based)
}
````

Notes:

* `coverage` measures match relative to the *source* ayah.
* `match_words_range` refers to token positions in the *matched* ayah.
* Multiple matches per source possible.

Usage:

* List all ayahs similar to a given ayah.
* Sort by: `score`, or `coverage` for memorization difficulty.

Example:
1:1 → matches 27:30 with range [5,8].
