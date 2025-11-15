# Quran Morphology Data – Overview

This directory contains three morphology datasets derived from QUL, exported as JSON (originally SQLite):

1. **Lemma dataset** – canonical dictionary forms.

2. **Root dataset** – trilateral Arabic roots.

3. **Stem dataset** – intermediate morphological base forms.

All files use `word_location` in the format:

```
SURAH:AYAH:WORD
```

(e.g., `2:3:5` = Surah 2, Ayah 3, Word 5)


---

# Lemma Dataset

## Lemma Dataset

A **lemma** = the canonical dictionary form of an Arabic word. It represents the normalized lexical form from which conjugations are derived.

Example:

* Lemma: **رَحِمَ**
* Derived forms: يرحم، نرحم، ارْحَم

Lemmas are **not** the same as roots:

* Root = abstract triliteral pattern (ر-ح-م)
* Lemma = normalized lexical form (رَحِمَ)

## Files

### `word-lemma.lemmas.json`

Structure (each line is a JSON object):

```
{
  "id": <int>,              // lemma ID
  "text": <str>,            // lemma with tashkeel
  "text_clean": <str>,      // lemma without tashkeel
  "words_count": <int|null>,
  "uniq_words_count": <int|null>
}
```

### `word-lemma.lemma_words.json`

Mapping from lemma → individual word occurrences.

```
{
  "lemma_id": <int>,        // FK to lemmas.id
  "word_location": "s:a:w" // location in Quran
}
```

Usage pattern:

1. Lookup lemma by ID in `lemmas.json`.
2. Fetch all occurrences via `lemma_words.json`.

---

# Root Dataset

## Root Dataset

A **root** is a set of typically three consonants that encode a core semantic field.

Example:
Root: **ر ح م** (r-ḥ-m) → الرحمن، الرحيم، رحمة، يرحم

Roots group semantically related words together.

## Files

### `word-root.roots.json`

Structure:

```
{
  "id": <int>,
  "arabic_trilateral": <str>,   // Arabic root letters
  "english_trilateral": <str>,  // transliteration
  "words_count": <int>,         // total derived word occurrences
  "uniq_words_count": <int>     // number of distinct derived forms
}
```

### `word-root.root_words.json`

Word occurrences for a given root.

```
{
  "root_id": <int>,              // FK to roots.id
  "word_location": "s:a:w"
}
```

Usage:

1. Lookup root metadata.
2. Fetch every Quranic occurrence via `root_words.json`.

---

# Stem Dataset

## Stem Dataset

A **stem** = intermediate morphological reduction between lemma and surface form.
Used for grouping close morphological variants.

Example:

* Lemma: **رَحِمَ**
* Stem: **رَحْم**
* Related: رحمة، الرحيم، يرحم

Stems may not correspond to actual standalone Arabic words.

## Files

### `word-stem.stems.json`

```
{
  "id": <int>,
  "text": <str>,         // stem with tashkeel
  "text_clean": <str>,   // stem without tashkeel
  "words_count": <int|null>,
  "uniq_words_count": <int|null>
}
```

### `word-stem.stem_words.json`

Mapping stem → occurrences.

```
{
  "stem_id": <int>,
  "word_location": "s:a:w"
}
```

Usage:

* Useful for clustering words that share morphological structure but differ in exact lemma.

---

# Agent Quick Rules

* Always treat each file as **line-delimited JSON objects**.
* Primary key lookups:

  * lemma: `id`
  * root: `id`
  * stem: `id`
* To retrieve all occurrences of any morphological class, use its corresponding `*_words.json` file.
* `word_location` always resolves to: Surah → Ayah → Word index.

