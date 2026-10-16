#!/usr/bin/env bash
# Run every cargo package that belongs to one step directory.
# Usage: test_step.sh <step-dir-name>
set -euo pipefail

step=${1:?step directory name}
root=$(cd "$(dirname "$0")/../.." && pwd)
cd "$root"

found=0
while IFS= read -r package; do
  package=${package//$'\r'/}
  if [[ -z "$package" ]]; then
    continue
  fi
  found=1
  echo "::group::${package}"
  if command -v cargo-nextest >/dev/null 2>&1; then
    # A CLI package with no test targets is a success. Nextest's default
    # is to fail that case (exit 4), which cargo test does not.
    cargo nextest run --locked -p "$package" --no-tests=pass
  else
    echo "cargo-nextest is not installed; falling back to cargo test for ${package}"
    cargo test -p "$package" --locked
  fi
  echo "::endgroup::"
done < <(python3 "$root/.github/scripts/list_steps.py" --root "$root" --packages "$step")

if [[ "$found" -ne 1 ]]; then
  echo "step ${step} has no packages" >&2
  exit 1
fi
