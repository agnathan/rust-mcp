# Performance

Every chapter that adds work has a Criterion bench. The numbers exclude the network. Chapter 1 has no socket at all.

**Measure, and do not fail the build on a number.** On pull requests and on every push to `main`, [Benchmarks](https://github.com/agnathan/rust-mcp/actions/workflows/benchmarks.yml) runs the benches on Ubuntu, writes the median, low, and high to the job summary, and uploads `criterion-summary-<commit>.json`. A push to `main` appends the run to `data.json` on `gh-pages` and refreshes [benchmarks/index.html](../benchmarks/index.html). A slow result does not fail the job. The bench job is `continue-on-error`, so a bench crash does not fail the workflow either.

The median in the book is `median.point_estimate` from `target/criterion/<name>/new/estimates.json`. The `time:` line Criterion prints is the slope. This page and the chapter say which one they are quoting.

## Chapter 1 budget

Machine for the local run recorded below: 4 vCPU Intel Xeon (KVM), 16 GiB RAM, Linux, rustc 1.99.1. Sample size 10, warm-up 1 s, measurement 2 s. CI remeasures on its own runner; the chapter table is updated from that run.

| Benchmark | Target | Local median |
|---|---|---|
| `parse_1_message` | < 1 ms | 1.191 µs |
| `parse_1000_messages` | < 50 ms | 1.294 ms |
| `serialize_1_message` | < 1 ms | 179.5 ns |
| `serialize_1000_messages` | < 50 ms | 178.9 µs |
| `dispatch_initialize` | < 1 ms | 748.8 ns |

One message in the microsecond range, and a thousand messages under a couple of milliseconds, is a good result on this kind of machine. A jump into the tens of milliseconds for a single message means the envelope got heavier. Details of what the timer includes are in [Chapter 1](../book/chapter-01-speaking-json-rpc.md) and in [benchmarks/descriptions.json](../benchmarks/descriptions.json).
