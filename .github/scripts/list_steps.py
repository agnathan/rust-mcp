#!/usr/bin/env python3
"""List step directories and the cargo packages inside each one."""

from __future__ import annotations

import argparse
import json
import sys
import tomllib
from pathlib import Path


def step_dirs(root: Path) -> list[Path]:
    steps = root / "steps"
    if not steps.is_dir():
        return []
    found = []
    for path in sorted(steps.iterdir()):
        if path.is_dir() and path.name.startswith(("ch", "gw")) and any(path.rglob("Cargo.toml")):
            found.append(path)
    return found


def packages(step: Path) -> list[str]:
    names: list[str] = []
    for manifest in sorted(step.rglob("Cargo.toml")):
        data = tomllib.loads(manifest.read_text(encoding="utf-8"))
        package = data.get("package")
        if isinstance(package, dict) and isinstance(package.get("name"), str):
            names.append(package["name"])
    if not names:
        raise SystemExit(f"no packages in {step}")
    return names


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path("."))
    parser.add_argument("--json", action="store_true", help="Print step names as a JSON array")
    parser.add_argument("--packages", metavar="STEP", help="Print package names for one step")
    parser.add_argument("--github-output", type=Path, help="Append steps=[...] for Actions")
    args = parser.parse_args()
    root = args.root.resolve()
    if args.packages:
        step = root / "steps" / args.packages
        if not step.is_dir():
            raise SystemExit(f"unknown step {args.packages}")
        for name in packages(step):
            sys.stdout.write(name + "\n")
        return
    names = [path.name for path in step_dirs(root)]
    if not names:
        raise SystemExit(f"no steps found under {root / 'steps'}")
    payload = json.dumps(names)
    if args.github_output:
        with args.github_output.open("a", encoding="utf-8") as handle:
            handle.write(f"steps={payload}\n")
    if args.json or args.github_output:
        print(payload)
    else:
        for name in names:
            print(name)


if __name__ == "__main__":
    try:
        main()
    except BrokenPipeError:
        sys.exit(0)
