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

## Chapter 2 budget

Sample size 10, warm-up 1 s, measurement 2 s. The CI column is the median from [Benchmarks run 38856055391](https://github.com/agnathan/rust-mcp/actions/runs/38856055391) (2026-10-16T05:29:54Z, Ubuntu). The local column is the same 4-vCPU Intel Xeon guest under KVM, 16 GiB RAM, Linux, rustc 1.99.1.

| Benchmark | Target | CI median | Local median |
|---|---|---|---|
| `negotiate_1` | < 1 µs | 16.66 ns | 10.46 ns |
| `negotiate_1000` | < 1 ms | 21.83 µs | 11.78 µs |
| `stdio_initialize_list` | < 5 ms | 236.2 µs | 113.9 µs |

`negotiate_1` is one version choice. `negotiate_1000` is a thousand choices, with every fifth request asking for `2026-07-28`. `stdio_initialize_list` is one in-memory duplex session: serve, `initialize`, `tools/list`, cancel. It does not spawn a process. Details are in [Chapter 2](../book/chapter-02-negotiating-the-protocol.md).

## Chapter 3 budget

Sample size 10, warm-up 1 s, measurement 2 s. The local column is the same 4-vCPU Intel Xeon guest. The CI column is filled from this chapter's Benchmarks run once that job has finished.

| Benchmark | Target | CI median | Local median |
|---|---|---|---|
| `lookup_1` | < 1 µs | pending | 12.51 ns |
| `dispatch_1000` | < 1 ms | pending | 59.96 µs |

`lookup_1` is one `HashMap` lookup among 1,000 tools. `dispatch_1000` deserializes and calls all 1,000. Details are in [Chapter 3](../book/chapter-03-the-fastmcp-builder-and-tools.md).
