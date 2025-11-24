"""
Pipeline validation integration for knowledge graph builds.

This module integrates the standalone validate_stability.py script
into the automated build pipeline.
"""

import subprocess
import sys
import shutil
import tempfile
import zstandard as zstd
from pathlib import Path
from typing import Optional
from loguru import logger


class GraphValidationError(Exception):
    """Raised when graph validation fails."""
    pass


def _decompress_graph(input_path: Path, output_path: Path) -> None:
    """
    Decompress a .zst graph file to a plain CBOR file.

    Args:
        input_path: Path to the compressed .zst file
        output_path: Path to write the decompressed CBOR file
    """
    if not input_path.exists():
        raise FileNotFoundError(f"Compressed file not found: {input_path}")

    logger.info(f"Decompressing {input_path} to {output_path}...")

    with open(input_path, 'rb') as ifh, open(output_path, 'wb') as ofh:
        dctx = zstd.ZstdDecompressor()
        dctx.copy_stream(ifh, ofh)


def validate_graph_stability(
    new_graph_path: Path,
    baseline_path: Optional[Path] = None,
    skip_validation: bool = False,
) -> bool:
    """
    Validate graph stability against baseline.

    Args:
        new_graph_path: Path to newly built graph (.cbor.zst)
        baseline_path: Path to baseline graph (if None, uses default)
        skip_validation: If True, skip validation (dangerous!)

    Returns:
        True if validation passed or skipped

    Raises:
        GraphValidationError: If validation fails
    """
    if skip_validation:
        logger.warning("⚠️  VALIDATION SKIPPED (--skip-validation flag)")
        return True

    # Determine baseline path
    if baseline_path is None:
        baseline_path = new_graph_path.parent / "baseline_graph.cbor.zst"

    # If no baseline exists, this is the first build
    if not baseline_path.exists():
        logger.info("No baseline graph found - first build detected")
        logger.info(f"Saving baseline: {baseline_path}")
        _save_baseline(new_graph_path, baseline_path)
        return True

    # Run validation script
    logger.info(f"Validating against baseline: {baseline_path}")

    validate_script = Path(__file__).parent.parent.parent.parent / "validate_stability.py"

    if not validate_script.exists():
        raise FileNotFoundError(f"Validation script not found: {validate_script}")

    # Use a temporary directory for decompressed files
    with tempfile.TemporaryDirectory() as tmp_dir_str:
        tmp_dir = Path(tmp_dir_str)

        # Decompress files if they are compressed
        # Handle new graph
        if str(new_graph_path).endswith('.zst'):
            decompressed_new = tmp_dir / "new_graph.cbor"
            _decompress_graph(new_graph_path, decompressed_new)
            new_graph_arg = decompressed_new
        else:
            new_graph_arg = new_graph_path

        # Handle baseline
        if str(baseline_path).endswith('.zst'):
            decompressed_baseline = tmp_dir / "baseline.cbor"
            _decompress_graph(baseline_path, decompressed_baseline)
            baseline_arg = decompressed_baseline
        else:
            baseline_arg = baseline_path

        result = subprocess.run(
            [sys.executable, str(validate_script), str(baseline_arg), str(new_graph_arg)],
            capture_output=True,
            text=True,
        )

    # Print validation output
    print(result.stdout)

    if result.returncode != 0:
        # Validation failed
        logger.error("❌ Graph validation FAILED!")
        logger.error(result.stderr)
        raise GraphValidationError(
            "Node ID stability check failed. "
            "Use --skip-validation to override (not recommended)."
        )

    logger.success("✅ Graph validation PASSED")

    # Update baseline for next build
    logger.info(f"Updating baseline: {baseline_path}")
    _save_baseline(new_graph_path, baseline_path)

    return True


def _save_baseline(source: Path, target: Path) -> None:
    """Copy graph file to baseline location."""
    import shutil
    shutil.copy2(source, target)
    logger.info(f"Baseline saved: {target}")
