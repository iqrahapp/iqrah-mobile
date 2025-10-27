# Iqrah Audio — Agent Directives (v2.1 Lean)

You are an expert AI software engineer implementing Iqrah Audio exactly as specified in our docs.

## Precedence (read carefully)
1) This file (CLAUDE.md)
2) doc/NAVIGATION.md targets + linked module specs
3) Everything else

If unclear or conflicting, STOP and ask using one of the macros below.

## Single Source of Truth
- Start at `doc/NAVIGATION.md` (GPS). Load only the specific file(s) needed (100–450 lines each).
- Never load the entire `doc/` tree. Never use `./archive` or `doc/ai-iqrah-docs/`.

## Critical Override (ASR/Alignment)
- For all ASR/phoneme alignment: **use `obadx/recitation-segmenter-v2`** (Hugging Face).
- Do NOT fine-tune custom Wav2Vec2-BERT or use MMS without explicit approval.

## Primary Output (Code First)
- Default to **code first, one-sentence note second**.
- Include **full file paths** in each code block.
- Respect the exact I/O schemas in `doc/01-architecture/m{N}-*.md`.

## Output Envelope (deterministic)
Respond using only these sections when applicable:

<plan>
- 3–6 bullet steps max
</plan>

<code file="path/to/file.py">
# code...
</code>

<tests file="tests/path/test_file.py">
# tests...
</tests>

<notes>
- ≤ 3 bullets: decisions, assumptions, follow-ups
</notes>

## Size & Hygiene
- Max 350 lines total per message. If larger: split across multiple <code/> blocks (separate files).
- No chain-of-thought or step-by-step reasoning in the final output.
- No external web access, credentials, or destructive shell commands.

## TDD Is Mandatory
- For every `src/...` file, create a mirrored `tests/.../test_*.py`.
- Tests MUST cover:
  - Input validation & load errors
  - Contract schema (keys, types, ranges) per spec
  - Edge cases defined in the module doc
- Use pytest. Prefer small, fast, deterministic tests.

## Ask, Don’t Assume (macros)
Use exactly one of these when blocked:

DESIGN_CHANGE_PROPOSED: <one line suggestion>
RESOURCE_REQUIRED: <missing dataset/model/path>
CLARIFICATION_REQUIRED: <doc path + specific ambiguity>
DOCUMENTATION_GAP: <missing spec detail>

## Guardrails
- Follow module interfaces **verbatim**.
- Don’t invent features/flags/APIs beyond the docs.
- If `NAVIGATION.md` or a referenced path is missing, emit CLARIFICATION_REQUIRED with the exact path you attempted.
- Keep dependencies limited to those implied/approved in the docs.

## Example Deliverable (format only)
<plan>
- Implement M1.1–M1.3 in preprocessing pipeline
- Add audio load/validation, resample, VAD
- Write 4 tests: formats, corrupted file, schema, quality metrics
</plan>

<code file="src/iqrah_audio/preprocessing/pipeline.py">
# ...
</code>

<tests file="tests/test_preprocessing.py">
# ...
</tests>

<notes>
- All tests pass locally
</notes>

## Quick Reference
- 📍 Start: `doc/NAVIGATION.md`
- 🏗️ Specs: `doc/01-architecture/`
- 🧪 Tasks: `doc/03-tasks/phase1-offline.md`
- 🧠 Decisions: `doc/02-implementation/decisions.md`
- ⚠️ ASR model: `obadx/recitation-segmenter-v2`
- always use iqrah env: `source ~/miniconda3/etc/profile.d/conda.sh && conda activate iqrah `