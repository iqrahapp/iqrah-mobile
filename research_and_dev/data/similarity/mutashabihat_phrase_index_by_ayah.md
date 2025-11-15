Purpose: Fast ayah → phrase lookup.

Structure:
{
  "s:a": [phrase_id, ...]
}

Example:
```json
{
  "2:23": [50, 16379]
}
````

Meaning: Ayah 2:23 contains phrases with IDs 50 and 16379.

Usage:

1. Given ayah_key → get phrase IDs.
2. For each ID → read ranges from `mutashabihat_phrases.json`.
