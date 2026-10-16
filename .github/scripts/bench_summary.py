#!/usr/bin/env python3
"""Render Criterion estimates into a GitHub job summary. Never fails the job."""

from __future__ import annotations

import json
import sys
from pathlib import Path


def human(ns: float) -> str:
    absolute = abs(ns)
    if absolute < 1_000:
        return f"{ns:.2f} ns" if absolute < 10 else f"{ns:.1f} ns"
    if absolute < 1_000_000:
        return f"{ns / 1_000:.2f} µs"
    if absolute < 1_000_000_000:
        return f"{ns / 1_000_000:.2f} ms"
    return f"{ns / 1_000_000_000:.2f} s"


def from_run(path: Path) -> list[tuple[str, float, float, float]]:
    data = json.loads(path.read_text(encoding="utf-8"))
    benches = data.get("benches")
    if not isinstance(benches, dict):
        return []
    rows = []
    for name in sorted(benches):
        sample = benches[name]
        if not isinstance(sample, dict):
            continue
        try:
            rows.append(
                (
                    name,
                    float(sample["median_ns"]),
                    float(sample["low_ns"]),
                    float(sample["high_ns"]),
                )
            )
        except (KeyError, TypeError, ValueError):
            continue
    return rows


def from_criterion(root: Path) -> list[tuple[str, float, float, float]]:
    rows = []
    for path in sorted(root.glob("*/new/estimates.json")):
        name = path.parts[-3]
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
            median = data["median"]
            interval = median["confidence_interval"]
            rows.append(
                (
                    name,
                    float(median["point_estimate"]),
                    float(interval["lower_bound"]),
                    float(interval["upper_bound"]),
                )
            )
        except (OSError, KeyError, TypeError, ValueError, json.JSONDecodeError) as err:
            print(f"| `{name}` | unreadable ({err}) | | |")
    return rows


def main() -> None:
    print("## Benchmarks")
    print()
    print("Criterion medians for the written chapters. Informational only: no fail threshold.")
    print("The median and its low/high bounds come from `estimates.json`. The printed `time:` line is the slope and is not used here.")
    print("A push to `main` appends this run to the dashboard on `gh-pages`.")
    print()
    run_path = Path("target/bench-run.json")
    rows = from_run(run_path) if run_path.is_file() else []
    if not rows:
        root = Path("target/criterion")
        rows = from_criterion(root) if root.is_dir() else []
    custom: list[dict[str, object]] = []
    recorded: list[dict[str, object]] = []
    if not rows:
        print("No Criterion estimates were written.")
    else:
        print("| Benchmark | Median | Low | High |")
        print("|---|---|---|---|")
        for name, median, low, high in rows:
            print(f"| `{name}` | {human(median)} | {human(low)} | {human(high)} |")
            custom.append({"name": name, "unit": "ns", "value": median})
            recorded.append(
                {"name": name, "median_ns": median, "low_ns": low, "high_ns": high}
            )
    if custom:
        target = Path("target")
        target.mkdir(exist_ok=True)
        (target / "criterion-summary.json").write_text(
            json.dumps(custom, indent=2) + "\n", encoding="utf-8"
        )
        commit = __import__("os").environ.get("GITHUB_SHA", "")
        (target / "criterion-run.json").write_text(
            json.dumps({"commit": commit, "benches": recorded}, indent=2) + "\n",
            encoding="utf-8",
        )


if __name__ == "__main__":
    try:
        main()
    except BrokenPipeError:
        sys.exit(0)
