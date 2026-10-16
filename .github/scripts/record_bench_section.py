#!/usr/bin/env python3
"""Append one section's Criterion medians to target/bench-run.json.

Times are nanoseconds. median/low/high are the median point estimate and the
median confidence interval from estimates.json, not the printed slope.
"""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path


def read_median(path: Path) -> dict[str, float]:
    data = json.loads(path.read_text(encoding="utf-8"))
    median = data["median"]
    interval = median["confidence_interval"]
    return {
        "median_ns": float(median["point_estimate"]),
        "low_ns": float(interval["lower_bound"]),
        "high_ns": float(interval["upper_bound"]),
    }


def harvest(criterion_dir: Path, section_id: str) -> dict[str, dict[str, float]]:
    found: dict[str, dict[str, float]] = {}
    if not criterion_dir.is_dir():
        return found
    for path in sorted(criterion_dir.glob("*/new/estimates.json")):
        name = path.parts[-3]
        found[f"{section_id}/{name}"] = read_median(path)
    return found


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--section-id", required=True)
    parser.add_argument("--section", required=True)
    parser.add_argument("--criterion", type=Path, default=Path("target/criterion"))
    parser.add_argument("--out", type=Path, default=Path("target/bench-run.json"))
    args = parser.parse_args()

    fresh = harvest(args.criterion, args.section_id)
    if not fresh:
        raise SystemExit(f"no estimates under {args.criterion} for section {args.section_id}")

    run: dict[str, object] = {
        "sha": os.environ.get("GITHUB_SHA", ""),
        "date": os.environ.get("BENCH_DATE", ""),
        "run_url": os.environ.get("BENCH_RUN_URL", ""),
        "estimate": "median",
        "sections": {},
        "benches": {},
    }
    if args.out.is_file():
        run = json.loads(args.out.read_text(encoding="utf-8"))
    sections = run.setdefault("sections", {})
    if not isinstance(sections, dict):
        raise SystemExit("bench-run sections must be an object")
    sections[args.section_id] = args.section
    benches = run.setdefault("benches", {})
    if not isinstance(benches, dict):
        raise SystemExit("bench-run benches must be an object")
    benches.update(fresh)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(run, indent=2) + "\n", encoding="utf-8")
    print(f"recorded {len(fresh)} benches for {args.section}")


if __name__ == "__main__":
    main()
