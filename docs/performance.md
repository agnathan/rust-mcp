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

Sample size 10, warm-up 1 s, measurement 2 s. The CI column is the median from [Benchmarks run 38856477535](https://github.com/agnathan/rust-mcp/actions/runs/38856477535) (2026-10-16T05:34:43Z, Ubuntu). The local column is the same 4-vCPU Intel Xeon guest.

| Benchmark | Target | CI median | Local median |
|---|---|---|---|
| `lookup_1` | < 1 µs | 17.14 ns | 12.51 ns |
| `dispatch_1000` | < 1 ms | 82.97 µs | 59.96 µs |

`lookup_1` is one `HashMap` lookup among 1,000 tools. `dispatch_1000` deserializes and calls all 1,000. Details are in [Chapter 3](../book/chapter-03-the-fastmcp-builder-and-tools.md).

## Chapter 4 budget

Sample size 10, warm-up 1 s, measurement 2 s. The CI column is the median from [Benchmarks run 38857080367](https://github.com/agnathan/rust-mcp/actions/runs/38857080367) (2026-10-16T05:41:43Z, Ubuntu). The local column is the same 4-vCPU Intel Xeon guest. The generator is not cached.

| Benchmark | Target | CI median | Local median |
|---|---|---|---|
| `schema_1` | < 100 µs | 1.863 µs | 1.596 µs |
| `schema_1000` | < 50 ms | 1.916 ms | 1.854 ms |

`schema_1` builds one JSON Schema for a struct with a `String` and an `Option<String>`. `schema_1000` builds that schema 1,000 times. Details are in [Chapter 4](../book/chapter-04-schemas-from-types.md).

## Chapter 5 budget

Sample size 10, warm-up 1 s, measurement 2 s. The CI column is the median from [Benchmarks run 38862013588](https://github.com/agnathan/rust-mcp/actions/runs/38862013588) (2026-10-16T06:35:09Z, Ubuntu). The local column is the same 4-vCPU Intel Xeon guest.

| Benchmark | Target | CI median | Local median |
|---|---|---|---|
| `register_macro_1` | < 100 µs | 4.640 µs | 3.200 µs |
| `call_fn_1000` | < 10 µs | 342.5 ns | 263.2 ns |

`register_macro_1` is one `tool_decl` registration, including schema generation. `call_fn_1000` is a thousand direct calls of the function the macro left in place. The proc macro is not in either timer. Details are in [Chapter 5](../book/chapter-05-a-tool-macro.md).

## Chapter 6 budget

Sample size 10, warm-up 1 s, measurement 2 s. The CI column is the median from [Benchmarks run 38863264174](https://github.com/agnathan/rust-mcp/actions/runs/38863264174) (2026-10-16T06:48:21Z, Ubuntu). The local column is the same 4-vCPU Intel Xeon guest.

| Benchmark | Target | CI median | Local median |
|---|---|---|---|
| `resource_lookup_10k` | < 1 µs | 64.47 ns | 38.75 ns |
| `template_scan_10k` | < 1 ms | 308.6 µs | 264.2 µs |

`resource_lookup_10k` is one `HashMap` get among 10,000 static resources. `template_scan_10k` walks 10,000 templates and matches the last one. Details are in [Chapter 6](../book/chapter-06-resources-and-templates.md).

## Chapter 7 budget

Sample size 10, warm-up 1 s, measurement 2 s. The CI column is the median from [Benchmarks run 38864877048](https://github.com/agnathan/rust-mcp/actions/runs/38864877048) (2026-10-16T07:04:27Z, Ubuntu). The local column is the same 4-vCPU Intel Xeon guest.

| Benchmark | Target | CI median | Local median |
|---|---|---|---|
| `prompt_render_1` | < 1 µs | 97.61 ns | 62.14 ns |
| `paginate_10k` | < 50 µs | 2.371 µs | 2.482 µs |

`prompt_render_1` is one prompt render. `paginate_10k` is one page of 50 items taken from offset 5,000 of a 10,000-item list. Details are in [Chapter 7](../book/chapter-07-prompts-completions-pagination.md).

## Chapter 8 budget

Sample size 10, warm-up 1 s, measurement 2 s. The CI column is the median from [Benchmarks run 38865647838](https://github.com/agnathan/rust-mcp/actions/runs/38865647838) (2026-10-16T07:11:53Z, Ubuntu). The local column is the same 4-vCPU Intel Xeon guest.

| Benchmark | Target | CI median | Local median |
|---|---|---|---|
| `dispatch_ok_1000` | < 1 ms | 172.7 µs | 128.4 µs |
| `dispatch_missing_1000` | < 1 ms | 75.41 µs | 64.08 µs |

`dispatch_ok_1000` is a thousand successful `add` dispatches. `dispatch_missing_1000` is a thousand lookups of a name that is not registered. Details are in [Chapter 8](../book/chapter-08-errors-done-right.md).

## Chapter 9 budget

Sample size 10, warm-up 1 s, measurement 2 s. The CI column is the median from [Benchmarks run 38866727295](https://github.com/agnathan/rust-mcp/actions/runs/38866727295) (2026-10-16T07:22:12Z, Ubuntu). The local column is the same 4-vCPU Intel Xeon guest.

| Benchmark | Target | CI median | Local median |
|---|---|---|---|
| `progress_1000` | < 100 µs | 12.63 µs | 14.37 µs |

`progress_1000` is a thousand in-memory progress notes, then a clear. Details are in [Chapter 9](../book/chapter-09-context.md).

## Chapter 10 budget

Sample size 10, warm-up 1 s, measurement 2 s. The CI column is the median from [Benchmarks run 38867393927](https://github.com/agnathan/rust-mcp/actions/runs/38867393927) (2026-10-16T07:28:20Z, Ubuntu). The local column is the same 4-vCPU Intel Xeon guest.

| Benchmark | Target | CI median | Local median |
|---|---|---|---|
| `inject_1000` | < 100 µs | 18.66 µs | 12.03 µs |
| `hand_1000` | < 100 µs | 3.909 µs | 15.22 µs |

`inject_1000` is a thousand `Dep::<Db>` extractions. `hand_1000` is a thousand `Arc` clones of a `Db` the caller already holds. Details are in [Chapter 10](../book/chapter-10-dependency-injection.md).

## Chapter 11 budget

Sample size 10, warm-up 1 s, measurement 2 s. The CI column is the median from [Benchmarks run 38867880902](https://github.com/agnathan/rust-mcp/actions/runs/38867880902) (2026-10-16T07:32:55Z, Ubuntu). The local column is the same 4-vCPU Intel Xeon guest.

| Benchmark | Target | CI median | Local median |
|---|---|---|---|
| `contended_reads` | < 2 ms | 236.8 µs | 244.3 µs |

`contended_reads` spawns four threads that each read an `RwLock` 1,000 times. Details are in [Chapter 11](../book/chapter-11-lifespan-and-state.md).

## Chapter 12 budget

Sample size 10, warm-up 1 s, measurement 2 s. The CI column is the median from [Benchmarks run 38870644362](https://github.com/agnathan/rust-mcp/actions/runs/38870644362) (2026-10-16T07:59:08Z, Ubuntu). The local column is the same 4-vCPU Intel Xeon guest.

| Benchmark | Target | CI median | Local median |
|---|---|---|---|
| `http_roundtrip` | < 2 ms | 346.6 µs | 294.9 µs |
| `initialize_once` | < 1 ms | 206.2 µs | 167.2 µs |
| `discover_once` | < 500 µs | 118.7 µs | 91.22 µs |

`http_roundtrip` is initialize, `tools/list`, and `DELETE` on loopback. `initialize_once` stops after the handshake and the delete. `discover_once` is one `server/discover` for `2026-07-28`. Details are in [Chapter 12](../book/chapter-12-streamable-http.md).

## Chapter 13 budget

Sample size 10, warm-up 1 s, measurement 2 s. The CI column is the median from [Benchmarks run 38871433836](https://github.com/agnathan/rust-mcp/actions/runs/38871433836) (2026-10-16T08:06:20Z, Ubuntu). The local column is the same 4-vCPU Intel Xeon guest.

| Benchmark | Target | CI median | Local median |
|---|---|---|---|
| `elicit_round` | < 100 µs | 30.68 µs | 15.11 µs |

`elicit_round` builds a desk, asks, and submits `yes` from a spawned task. Details are in [Chapter 13](../book/chapter-13-elicitation.md).

## Chapter 14 budget

Sample size 10, warm-up 1 s, measurement 2 s. The CI column is the median from [Benchmarks run 38872049825](https://github.com/agnathan/rust-mcp/actions/runs/38872049825) (2026-10-16T08:11:48Z, Ubuntu). The local column is the same 4-vCPU Intel Xeon guest.

| Benchmark | Target | CI median | Local median |
|---|---|---|---|
| `list_once` | < 200 µs | 82.47 µs | 36.40 µs |
| `fanout_8` | < 500 µs | 184.0 µs | 78.98 µs |

`list_once` is one `tools/list` on a connected memory client. `fanout_8` lists eight servers on a `JoinSet`. Details are in [Chapter 14](../book/chapter-14-client-and-groups.md).

## Chapter 15 budget

Sample size 10, warm-up 1 s, measurement 2 s. The CI column is filled from the Benchmarks run for this chapter. The local column is the same 4-vCPU Intel Xeon guest.

| Benchmark | Target | CI median | Local median |
|---|---|---|---|
| `memory_list` | < 1 µs | 30.67 ns | 24.20 ns |
| `duplex_list` | < 200 µs | 47.07 µs | 53.27 µs |

`memory_list` collects tool names from the harness. `duplex_list` is one `tools/list` on a connected memory client. Details are in [Chapter 15](../book/chapter-15-testing-utilities.md).

## Chapter 16 budget

Sample size 10, warm-up 1 s, measurement 2 s. The CI column is filled from the Benchmarks run for this chapter. The local column is the same 4-vCPU Intel Xeon guest.

| Benchmark | Target | CI median | Local median |
|---|---|---|---|
| `layers_0` | < 2 ms | 306.7 µs | 194.0 µs |
| `layers_5` | < 2 ms | 664.2 µs | 557.2 µs |

`layers_0` is a thousand `echo` calls with no hooks. `layers_5` is the same thousand through five hooks that all continue. Details are in [Chapter 16](../book/chapter-16-middleware.md).

## Chapter 17 budget

Sample size 10, warm-up 1 s, measurement 2 s. The CI column is filled from the Benchmarks run for this chapter. The local column is the same 4-vCPU Intel Xeon guest.

| Benchmark | Target | CI median | Local median |
|---|---|---|---|
| `direct_call` | < 1 µs | pending | 15.91 ns |
| `proxy_call` | < 1 µs | pending | 17.58 ns |
| `aggregate_10` | < 5 µs | pending | 448.1 ns |

`direct_call` is one local `call`. `proxy_call` is the same call through a proxy. `aggregate_10` lists ten providers. Details are in [Chapter 17](../book/chapter-17-providers-and-composition.md).
