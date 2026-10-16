#!/usr/bin/env python3
"""Turn dorny/paths-filter `changes` into the step directories that exist.

`shared` expands to every steps/ch* and steps/gw* directory. A planned
chapter whose directory is not in the tree yet is ignored. Writes `steps=[...]` for
$GITHUB_OUTPUT when that variable is set, and always prints the JSON array.
"""

from __future__ import annotations

import json
import os
import sys
from pathlib import Path


def existing_steps(root: Path) -> list[str]:
    steps = root / "steps"
    if not steps.is_dir():
        return []
    return sorted(
        path.name
        for path in steps.iterdir()
        if path.is_dir() and path.name.startswith(("ch", "gw"))
    )


def select(changes: list[str], existing: list[str]) -> list[str]:
    have = set(existing)
    chosen: list[str] = []
    if "shared" in changes:
        chosen.extend(existing)
    for name in changes:
        if name == "shared" or name in chosen or name not in have:
            continue
        chosen.append(name)
    return chosen


def main() -> None:
    raw = os.environ.get("CHANGES", "[]").strip() or "[]"
    changes = json.loads(raw)
    if not isinstance(changes, list) or not all(isinstance(item, str) for item in changes):
        raise SystemExit("CHANGES must be a JSON array of strings")
    root = Path(os.environ.get("ROOT", ".")).resolve()
    selected = select(changes, existing_steps(root))
    payload = json.dumps(selected)
    output = os.environ.get("GITHUB_OUTPUT")
    if output:
        with open(output, "a", encoding="utf-8") as handle:
            handle.write(f"steps={payload}\n")
    print(payload)


if __name__ == "__main__":
    try:
        main()
    except BrokenPipeError:
        sys.exit(0)
