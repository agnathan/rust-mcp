#!/usr/bin/env bash
# Run one chapter section's Criterion bench and record its median before the
# next section overwrites a shared function name such as loop_iteration.
set -euo pipefail

section_id=${1:?section id, for example 3.3}
section_label=${2:?section label, for example "3.3 dispatch"}
package=${3:?cargo package}
bench=${4:?bench target name}

root=$(cd "$(dirname "$0")/../.." && pwd)
cd "$root"

rm -rf target/criterion
cargo bench -p "$package" --bench "$bench" --locked -- --warm-up-time 1 --measurement-time 2 --sample-size 10
python3 "$root/.github/scripts/record_bench_section.py" \
  --section-id "$section_id" \
  --section "$section_label"
