# Iqrah Knowledge Graph

Research and development environment for the Iqrah knowledge graph.

## Setup

```bash
pip install -r requirements.yml
```

## Building the Knowledge Graph

```bash
python -m iqrah_cli build knowledge-graph \
    --from-scratch \
    --morphology data/morphology.csv \
    --preset full \
    -o output/knowledge_graph.cbor.zst
```

## Node ID Stability Validation

All graph builds are automatically validated to prevent breaking user progress.

### How It Works

1. **First Build:** Graph is saved as baseline (`baseline_graph.cbor.zst`)
2. **Subsequent Builds:** New graph is validated against baseline
3. **Validation Checks:** All node IDs from baseline must exist in new graph
4. **On Success:** New graph becomes the baseline
5. **On Failure:** Build exits with error

### Running Builds

```bash
# Normal build (validation enabled by default)
python -m iqrah_cli build knowledge-graph \
    --from-scratch \
    --morphology path/to/corpus.csv \
    --preset full \
    -o output/graph.cbor.zst

# Skip validation (DANGEROUS - only for major version bumps)
python -m iqrah_cli build knowledge-graph \
    --from-scratch \
    --morphology path/to/corpus.csv \
    --skip-validation \
    -o output/graph.cbor.zst

# Use custom baseline
python -m iqrah_cli build knowledge-graph \
    --from-scratch \
    --morphology path/to/corpus.csv \
    --baseline path/to/baseline.cbor.zst \
    -o output/graph.cbor.zst
```

### Manual Validation

You can also run validation manually:

```bash
python validate_stability.py old_graph.cbor.zst new_graph.cbor.zst
```

### Handling Breaking Changes

If you MUST remove/change node IDs:

1. Document the change in `CHANGELOG.md`
2. Bump major version (2.0.0 → 3.0.0)
3. Create migration mapping (see `docs/migration-strategy.md`)
4. Run with `--skip-validation` flag
5. Communicate breaking change to users

### CI/CD Integration

In CI pipeline:

```yaml
- name: Build and validate knowledge graph
  run: |
    cd research_and_dev/iqrah-knowledge-graph2
    python -m iqrah_cli build knowledge-graph \
      --from-scratch \
      --morphology data/corpus.csv \
      --preset ci-test \
      -o output/graph.cbor.zst
    # Validation runs automatically - will fail build if broken
```

Validation failures will cause CI to fail, preventing merge.
