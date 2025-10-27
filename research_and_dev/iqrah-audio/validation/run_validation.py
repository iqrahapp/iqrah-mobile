#!/usr/bin/env python3
"""
Validation and Benchmarking Harness for Iqrah Audio

This script runs validation tests and benchmarks for all modules.
Designed for both local development and CI/CD environments.

Usage:
    python validation/run_validation.py               # Run all tests
    python validation/run_validation.py --unit        # Unit tests only
    python validation/run_validation.py --benchmark   # Benchmarks only
    python validation/run_validation.py --module m1   # Specific module
    python validation/run_validation.py --verbose     # Detailed output
"""

import argparse
import json
import sys
import time
from pathlib import Path
from typing import Dict, List, Optional

import pytest


class ValidationRunner:
    """Orchestrates validation tests and benchmarks."""

    def __init__(self, verbose: bool = False):
        self.verbose = verbose
        self.results: Dict = {
            "timestamp": time.time(),
            "tests": {},
            "benchmarks": {},
            "summary": {},
        }

    def run_unit_tests(self, module: Optional[str] = None) -> int:
        """Run unit tests using pytest.

        Args:
            module: Specific module to test (e.g., 'm1'), or None for all

        Returns:
            Exit code (0 = success, non-zero = failure)
        """
        print("\n" + "=" * 70)
        print("RUNNING UNIT TESTS")
        print("=" * 70)

        args = ["tests/", "-v", "--tb=short"]

        if module:
            # Filter to specific module tests
            args.append(f"-k {module}")

        if self.verbose:
            args.append("-vv")

        # Run pytest and capture results
        exit_code = pytest.main(args)

        self.results["tests"]["exit_code"] = exit_code
        self.results["tests"]["status"] = "passed" if exit_code == 0 else "failed"

        return exit_code

    def run_benchmarks(self, module: Optional[str] = None) -> int:
        """Run benchmark tests using pytest-benchmark or custom timing.

        Args:
            module: Specific module to benchmark (e.g., 'm1'), or None for all

        Returns:
            Exit code (0 = success, non-zero = failure)
        """
        print("\n" + "=" * 70)
        print("RUNNING BENCHMARKS")
        print("=" * 70)

        args = ["validation/benchmarks/", "-v", "--tb=short"]

        if module:
            args.append(f"-k {module}")

        if self.verbose:
            args.append("-vv")

        # Run pytest on benchmark files
        exit_code = pytest.main(args)

        self.results["benchmarks"]["exit_code"] = exit_code
        self.results["benchmarks"]["status"] = "passed" if exit_code == 0 else "failed"

        return exit_code

    def run_validation_tests(self, module: Optional[str] = None) -> int:
        """Run validation tests against reference data.

        Args:
            module: Specific module to validate (e.g., 'm1'), or None for all

        Returns:
            Exit code (0 = success, non-zero = failure)
        """
        print("\n" + "=" * 70)
        print("RUNNING VALIDATION TESTS")
        print("=" * 70)

        validation_tests = Path("validation/tests")
        if not validation_tests.exists():
            print("⚠️  No validation tests found (this is OK for early development)")
            return 0

        args = [str(validation_tests), "-v", "--tb=short"]

        if module:
            args.append(f"-k {module}")

        if self.verbose:
            args.append("-vv")

        exit_code = pytest.main(args)

        self.results["validation"] = {
            "exit_code": exit_code,
            "status": "passed" if exit_code == 0 else "failed",
        }

        return exit_code

    def save_results(self, output_file: str = "validation/results.json"):
        """Save validation results to JSON file.

        Args:
            output_file: Path to output JSON file
        """
        output_path = Path(output_file)
        output_path.parent.mkdir(parents=True, exist_ok=True)

        with open(output_path, "w") as f:
            json.dump(self.results, f, indent=2)

        print(f"\n📊 Results saved to: {output_file}")

    def print_summary(self):
        """Print summary of validation results."""
        print("\n" + "=" * 70)
        print("VALIDATION SUMMARY")
        print("=" * 70)

        for test_type in ["tests", "benchmarks", "validation"]:
            if test_type in self.results:
                status = self.results[test_type].get("status", "skipped")
                icon = "✅" if status == "passed" else "❌" if status == "failed" else "⊘"
                print(f"{icon}  {test_type.upper()}: {status}")

        # Overall status
        all_passed = all(
            self.results.get(t, {}).get("status") in ["passed", None]
            for t in ["tests", "benchmarks", "validation"]
        )

        print("\n" + "=" * 70)
        if all_passed:
            print("✅ ALL VALIDATIONS PASSED")
        else:
            print("❌ SOME VALIDATIONS FAILED")
        print("=" * 70 + "\n")


def main():
    """Main entry point for validation script."""
    parser = argparse.ArgumentParser(
        description="Run Iqrah Audio validation and benchmarks",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )

    parser.add_argument(
        "--unit",
        action="store_true",
        help="Run unit tests only",
    )
    parser.add_argument(
        "--benchmark",
        action="store_true",
        help="Run benchmarks only",
    )
    parser.add_argument(
        "--validation",
        action="store_true",
        help="Run validation tests only",
    )
    parser.add_argument(
        "--module",
        type=str,
        help="Run tests for specific module (e.g., m1, m2)",
    )
    parser.add_argument(
        "--verbose",
        "-v",
        action="store_true",
        help="Verbose output",
    )
    parser.add_argument(
        "--save-results",
        type=str,
        default="validation/results.json",
        help="Save results to JSON file",
    )

    args = parser.parse_args()

    runner = ValidationRunner(verbose=args.verbose)

    # Determine what to run
    run_all = not (args.unit or args.benchmark or args.validation)

    exit_codes = []

    try:
        if args.unit or run_all:
            exit_codes.append(runner.run_unit_tests(module=args.module))

        if args.benchmark or run_all:
            exit_codes.append(runner.run_benchmarks(module=args.module))

        if args.validation or run_all:
            exit_codes.append(runner.run_validation_tests(module=args.module))

    finally:
        # Always save results and print summary
        runner.save_results(args.save_results)
        runner.print_summary()

    # Exit with failure if any test suite failed
    sys.exit(max(exit_codes) if exit_codes else 0)


if __name__ == "__main__":
    main()
