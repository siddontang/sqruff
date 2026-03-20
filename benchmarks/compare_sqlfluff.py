#!/usr/bin/env python3
"""Compare sqruff vs sqlfluff performance on the same SQL files.

Prerequisites:
    pip install sqlfluff
    cargo build --release  (in the sqruff workspace)

Usage:
    python compare_sqlfluff.py [--sql-file large_test.sql] [--iterations 3]
"""

import argparse
import os
import subprocess
import sys
import time
import tempfile


def find_sqruff_binary():
    """Find the sqruff binary."""
    candidates = [
        os.path.join(os.path.dirname(__file__), "..", "target", "release", "sqruff"),
        os.path.join(os.path.dirname(__file__), "..", "target", "debug", "sqruff"),
        "sqruff",  # in PATH
    ]
    for path in candidates:
        if os.path.isfile(path) and os.access(path, os.X_OK):
            return path
    return None


def time_command(cmd, iterations=3):
    """Run a command multiple times and return (mean_time, min_time, max_time) in seconds."""
    times = []
    for i in range(iterations):
        start = time.perf_counter()
        try:
            result = subprocess.run(
                cmd, capture_output=True, text=True, timeout=300
            )
        except subprocess.TimeoutExpired:
            return (float("inf"), float("inf"), float("inf"))
        elapsed = time.perf_counter() - start
        times.append(elapsed)
    return (sum(times) / len(times), min(times), max(times))


def check_sqlfluff():
    """Check if sqlfluff is installed."""
    try:
        result = subprocess.run(
            ["sqlfluff", "version"], capture_output=True, text=True, timeout=30
        )
        return result.returncode == 0
    except (FileNotFoundError, subprocess.TimeoutExpired):
        return False


def main():
    parser = argparse.ArgumentParser(description="Compare sqruff vs sqlfluff performance")
    parser.add_argument("--sql-file", type=str, default=None, help="SQL file to benchmark")
    parser.add_argument("--iterations", type=int, default=3, help="Number of iterations")
    parser.add_argument("--generate", type=int, default=None,
                        help="Generate a SQL file with N statements first")
    args = parser.parse_args()

    # Generate SQL file if needed
    sql_file = args.sql_file
    if sql_file is None:
        if args.generate:
            sql_file = tempfile.mktemp(suffix=".sql")
            subprocess.run([
                sys.executable,
                os.path.join(os.path.dirname(__file__), "generate_large_sql.py"),
                "--lines", str(args.generate),
                "--output", sql_file,
            ], check=True)
        else:
            # Default: generate 1000 statements
            sql_file = tempfile.mktemp(suffix=".sql")
            subprocess.run([
                sys.executable,
                os.path.join(os.path.dirname(__file__), "generate_large_sql.py"),
                "--lines", "1000",
                "--output", sql_file,
            ], check=True)

    if not os.path.isfile(sql_file):
        print(f"Error: SQL file not found: {sql_file}")
        sys.exit(1)

    file_size_kb = os.path.getsize(sql_file) / 1024
    print(f"\n{'='*70}")
    print(f"  sqruff vs sqlfluff Performance Comparison")
    print(f"{'='*70}")
    print(f"  SQL file:    {sql_file}")
    print(f"  File size:   {file_size_kb:.1f} KB")
    print(f"  Iterations:  {args.iterations}")
    print(f"{'='*70}\n")

    results = []

    # Benchmark sqruff
    sqruff_bin = find_sqruff_binary()
    if sqruff_bin:
        print(f"  Benchmarking sqruff ({sqruff_bin})...")
        mean, mn, mx = time_command([sqruff_bin, "check", sql_file], args.iterations)
        results.append(("sqruff check", mean, mn, mx))
        print(f"    lint:   {mean:.3f}s (min={mn:.3f}s, max={mx:.3f}s)")

        mean, mn, mx = time_command([sqruff_bin, "format", "--check", sql_file], args.iterations)
        results.append(("sqruff format", mean, mn, mx))
        print(f"    format: {mean:.3f}s (min={mn:.3f}s, max={mx:.3f}s)")
    else:
        print("  sqruff binary not found. Run: cargo build --release")

    # Benchmark sqlfluff
    if check_sqlfluff():
        print(f"\n  Benchmarking sqlfluff...")
        mean, mn, mx = time_command(["sqlfluff", "lint", "--dialect", "ansi", sql_file], args.iterations)
        results.append(("sqlfluff lint", mean, mn, mx))
        print(f"    lint:   {mean:.3f}s (min={mn:.3f}s, max={mx:.3f}s)")

        mean, mn, mx = time_command(["sqlfluff", "fix", "--dialect", "ansi", "--check", sql_file], args.iterations)
        results.append(("sqlfluff fix", mean, mn, mx))
        print(f"    format: {mean:.3f}s (min={mn:.3f}s, max={mx:.3f}s)")
    else:
        print("\n  sqlfluff not installed. Run: pip install sqlfluff")

    # Summary table
    if len(results) >= 2:
        print(f"\n{'='*70}")
        print(f"  {'Tool':<20} {'Mean':>10} {'Min':>10} {'Max':>10}")
        print(f"  {'-'*50}")
        for name, mean, mn, mx in results:
            if mean == float("inf"):
                print(f"  {name:<20} {'timeout':>10} {'timeout':>10} {'timeout':>10}")
            else:
                print(f"  {name:<20} {mean:>9.3f}s {mn:>9.3f}s {mx:>9.3f}s")

        # Speedup calculation
        sqruff_times = [r for r in results if r[0].startswith("sqruff")]
        sqlfluff_times = [r for r in results if r[0].startswith("sqlfluff")]
        if sqruff_times and sqlfluff_times:
            speedup = sqlfluff_times[0][1] / sqruff_times[0][1] if sqruff_times[0][1] > 0 else float("inf")
            print(f"\n  🚀 sqruff is {speedup:.1f}x faster than sqlfluff for linting")

        print(f"{'='*70}\n")


if __name__ == "__main__":
    main()
