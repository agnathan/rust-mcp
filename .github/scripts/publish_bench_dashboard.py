#!/usr/bin/env python3
"""Append this run to data.json on gh-pages and refresh the dashboard files.

Called only for a push to main. A pull request still uploads the artifact;
it does not publish. Older seeded points stay. A sha already present is left
as it was, so a re-run does not duplicate the point.
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path


REPO = "agnathan/rust-mcp"
SITE_FILES = ("index.html", "descriptions.json", "data.json")


def load_json(path: Path) -> dict:
    data = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(data, dict) or not isinstance(data.get("runs"), list):
        raise SystemExit(f"{path} must be an object with a runs array")
    return data


def append_run(history: dict, run: dict) -> bool:
    sha = str(run.get("sha") or "")
    if not sha:
        raise SystemExit("this run has no sha")
    benches = run.get("benches")
    if not isinstance(benches, dict) or not benches:
        raise SystemExit("this run recorded no benches")
    for existing in history["runs"]:
        if isinstance(existing, dict) and existing.get("sha") == sha:
            return False
    history["runs"].append(run)
    history["runs"].sort(key=lambda item: str(item.get("date") or ""))
    return True


def git(*args: str, cwd: Path) -> None:
    subprocess.run(["git", *args], cwd=cwd, check=True)


def main() -> None:
    root = Path(__file__).resolve().parents[2]
    run_path = root / "target" / "bench-run.json"
    if not run_path.is_file():
        raise SystemExit(f"missing {run_path}")
    run = json.loads(run_path.read_text(encoding="utf-8"))
    source = root / "benchmarks"

    remote = os.environ.get("BENCH_PAGES_REMOTE", "origin")
    branch = os.environ.get("BENCH_PAGES_BRANCH", "gh-pages")
    fetch = subprocess.run(
        ["git", "fetch", remote, branch],
        cwd=root,
        check=False,
        capture_output=True,
        text=True,
    )
    has_remote = fetch.returncode == 0

    parent = Path(tempfile.mkdtemp(prefix="gh-pages-"))
    page = parent / "site"
    try:
        if has_remote:
            git("worktree", "add", "--detach", str(page), f"{remote}/{branch}", cwd=root)
        else:
            git("worktree", "add", "--detach", str(page), "HEAD", cwd=root)
            git("checkout", "--orphan", branch, cwd=page)
            git("rm", "-rf", ".", cwd=page)

        data_path = page / "data.json"
        if data_path.is_file():
            history = load_json(data_path)
        else:
            history = load_json(source / "data.json")
        added = append_run(history, run)
        for name in ("index.html", "descriptions.json"):
            (page / name).write_text((source / name).read_text(encoding="utf-8"), encoding="utf-8")
        data_path.write_text(json.dumps(history, indent=2) + "\n", encoding="utf-8")
        (page / ".nojekyll").write_text("", encoding="utf-8")

        git("add", "--", *SITE_FILES, ".nojekyll", cwd=page)
        dirty = subprocess.run(["git", "diff", "--cached", "--quiet"], cwd=page)
        sha = str(run.get("sha") or "")[:12]
        if dirty.returncode == 0:
            print("gh-pages already has this dashboard revision")
            return
        message = f"Record benchmark medians for {sha}" if added else "Refresh the benchmark dashboard"
        git(
            "-c",
            "user.name=github-actions[bot]",
            "-c",
            "user.email=41898282+github-actions[bot]@users.noreply.github.com",
            "commit",
            "-m",
            message,
            cwd=page,
        )
        git("push", remote, f"HEAD:{branch}", cwd=page)
        print(f"published {sha} to {branch}")
    finally:
        subprocess.run(["git", "worktree", "remove", "--force", str(page)], cwd=root, check=False)
        shutil.rmtree(parent, ignore_errors=True)


if __name__ == "__main__":
    main()
