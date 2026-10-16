#!/usr/bin/env python3
"""Decide a macOS CI retry, then whether that retry saved the workflow.

GitHub's hosted macOS arm64 runners sometimes cancel a job before any step
runs ("not acquired by Runner"). A step-level retry never starts in that case.
The workflow runs the macOS jobs once, and a second time only when the first
wave did not succeed. This script reads the Actions job list.

`inspect` prints `retry=true` or `retry=false` for $GITHUB_OUTPUT.
`gate` exits 0 when the first wave succeeded, when macOS was not part of this
run, or when the retry wave succeeded. Otherwise it exits 1.

Job names must be `macos / <step>` and `macos retry / <step>`.
"""

from __future__ import annotations

import json
import os
import sys
import urllib.request
from typing import Any


def latest_by_name(jobs: list[dict[str, Any]]) -> list[dict[str, Any]]:
    chosen: dict[str, dict[str, Any]] = {}
    for job in jobs:
        name = str(job.get("name") or "")
        previous = chosen.get(name)
        started = str(job.get("started_at") or "")
        if previous is None or started >= str(previous.get("started_at") or ""):
            chosen[name] = job
    return list(chosen.values())


def split_waves(jobs: list[dict[str, Any]]) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    latest = latest_by_name(jobs)
    first = [job for job in latest if str(job.get("name") or "").startswith("macos / ")]
    second = [job for job in latest if str(job.get("name") or "").startswith("macos retry / ")]
    return first, second


def should_retry(first: list[dict[str, Any]]) -> bool:
    if not first:
        return True
    return any(job.get("conclusion") != "success" for job in first)


def gate_passes(first: list[dict[str, Any]], second: list[dict[str, Any]], expect: bool) -> bool:
    if not expect:
        return True
    if first and all(job.get("conclusion") == "success" for job in first):
        return True
    if not second:
        return False
    return all(job.get("conclusion") == "success" for job in second)


def fetch_jobs() -> list[dict[str, Any]]:
    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    repo = os.environ.get("GITHUB_REPOSITORY")
    run_id = os.environ.get("GITHUB_RUN_ID")
    if not token or not repo or not run_id:
        raise SystemExit("GH_TOKEN, GITHUB_REPOSITORY, and GITHUB_RUN_ID are required")
    jobs: list[dict[str, Any]] = []
    page = 1
    while True:
        url = (
            f"https://api.github.com/repos/{repo}/actions/runs/{run_id}/jobs"
            f"?per_page=100&page={page}"
        )
        request = urllib.request.Request(
            url,
            headers={
                "Authorization": f"Bearer {token}",
                "Accept": "application/vnd.github+json",
                "User-Agent": "rust-mcp-macos-retry",
            },
        )
        with urllib.request.urlopen(request) as response:
            payload = json.load(response)
        batch = payload.get("jobs")
        if not isinstance(batch, list):
            raise SystemExit("Actions jobs response has no jobs array")
        jobs.extend(batch)
        if len(batch) < 100:
            return jobs
        page += 1


def expect_macos() -> bool:
    return os.environ.get("EXPECT_MACOS", "true") == "true"


def self_test() -> None:
    ok = {"name": "macos / ch01-1-messages", "conclusion": "success", "started_at": "1"}
    bad = {"name": "macos / ch04-4-cache", "conclusion": "cancelled", "started_at": "1"}
    retry_ok = {
        "name": "macos retry / ch04-4-cache",
        "conclusion": "success",
        "started_at": "2",
    }
    older = {"name": "macos / ch01-1-messages", "conclusion": "failure", "started_at": "0"}
    assert should_retry([ok]) is False
    assert should_retry([ok, bad]) is True
    assert should_retry([]) is True
    assert gate_passes([ok], [], expect=True) is True
    assert gate_passes([ok, bad], [], expect=True) is False
    assert gate_passes([ok, bad], [retry_ok, {"name": "macos retry / ch01-1-messages", "conclusion": "success"}], expect=True)
    assert gate_passes([], [], expect=False) is True
    assert gate_passes([bad], [retry_ok], expect=True) is True
    assert (
        gate_passes(
            [bad],
            [{"name": "macos retry / ch04-4-cache", "conclusion": "cancelled"}],
            expect=True,
        )
        is False
    )
    assert gate_passes([], [], expect=True) is False
    kept = latest_by_name([older, ok])
    assert len(kept) == 1 and kept[0]["conclusion"] == "success"
    first, second = split_waves([ok, bad, retry_ok])
    assert len(first) == 2 and len(second) == 1


def main(argv: list[str]) -> None:
    if len(argv) != 2 or argv[1] not in {"inspect", "gate", "self-test"}:
        raise SystemExit("usage: macos_retry.py inspect|gate|self-test")
    if argv[1] == "self-test":
        self_test()
        print("ok")
        return
    expect = expect_macos()
    if argv[1] == "gate" and not expect:
        return
    first, second = split_waves(fetch_jobs())
    if argv[1] == "inspect":
        retry = expect and should_retry(first)
        print(f"retry={'true' if retry else 'false'}")
        if first:
            failed = [job["name"] for job in first if job.get("conclusion") != "success"]
            if failed:
                print("first wave not all success: " + ", ".join(failed), file=sys.stderr)
        elif expect:
            print("no macos / jobs were visible; scheduling a retry", file=sys.stderr)
        return
    if gate_passes(first, second, expect):
        return
    print(
        f"macOS gate failed: first={len(first)} retry={len(second)} expect={expect}",
        file=sys.stderr,
    )
    for job in first + second:
        print(f"  {job.get('name')}: {job.get('conclusion')}", file=sys.stderr)
    raise SystemExit(1)


if __name__ == "__main__":
    main(sys.argv)
