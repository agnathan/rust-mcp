# Performance

Every chapter that adds work has a Criterion bench. The numbers exclude the network. Chapter 1 has no socket at all.

**Measure, and do not fail the build on a number.** On pull requests and on every push to `main`, [Benchmarks](https://github.com/agnathan/rust-mcp/actions/workflows/benchmarks.yml) runs the benches on Ubuntu, writes the median, low, and high to the job summary, and uploads `criterion-summary-<commit>.json`. A push to `main` appends the run to `data.json` on `gh-pages` and refreshes [benchmarks/index.html](../benchmarks/index.html). A slow result does not fail the job. The bench job is `continue-on-error`, so a bench crash does not fail the workflow either.

The median in the book is `median.point_estimate` from `target/criterion/<name>/new/estimates.json`. The `time:` line Criterion prints is the slope. This page and the chapter say which one they are quoting.

## Chapter 1 budget

Sample size 10, warm-up 1 s, measurement 2 s. The CI column is the median from [Benchmarks run 38843114413](https://github.com/agnathan/rust-mcp/actions/runs/38843114413) (2026-10-16T02:58:15Z, Ubuntu). The local column is a 4-vCPU Intel Xeon guest under KVM, 16 GiB RAM, Linux, rustc 1.99.1.

| Benchmark | Target | CI median | Local median |
|---|---|---|---|
| `parse_1_message` | < 1 ms | 1.558 µs | 1.191 µs |
| `parse_1000_messages` | < 50 ms | 1.569 ms | 1.294 ms |
| `serialize_1_message` | < 1 ms | 249.3 ns | 179.5 ns |
| `serialize_1000_messages` | < 50 ms | 249.8 µs | 178.9 µs |
| `dispatch_initialize` | < 1 ms | 863.7 ns | 748.8 ns |

One message in the microsecond range, and a thousand messages under a couple of milliseconds, is a good result. A jump into the tens of milliseconds for a single message means the envelope got heavier. Details of what the timer includes are in [Chapter 1](../book/chapter-01-speaking-json-rpc.md) and in [benchmarks/descriptions.json](../benchmarks/descriptions.json).
