# Chapter 5: A `#[tool]` macro

*Draft. Bracketed notes like [personal note] are where my own story goes.*

## Introduction

Chapter 4 can register `add` and advertise its schema. The call site is still a small pile of glue: a struct for the arguments, a closure that pulls the fields out, and a string for the name. FastMCP's Python does not ask you to write that glue. You write the function, put `@mcp.tool` above it, and the decorator records the name, the docstring, and the parameters.

This chapter builds the Rust version of that decorator. A procedural macro is a function the compiler runs while it is compiling your code. `#[tool]` reads the function, leaves the function in place so you can still call it, and emits the registration struct you would otherwise write by hand. The schema is still built when the tool is registered, which is the same moment as chapter 4. The macro's own work is finished before the program starts.

`rmcp` already ships a `#[tool]` macro. It fills an `rmcp` `ToolRouter` on a type that implements `ServerHandler` directly. This book's registry is `FastMcp::tool`, so the macro in this chapter emits a `ToolDecl` that `FastMcp::tool_decl` turns into that call. You will see both shapes. The one you use is ours.

## What you'll build

A function with a doc comment, an attribute above it, and a server that knows the function's name.

```text
add(2, 3) = {"total":5}
description: Add two integers.
direct: 5
```

`#[tool]` on `fn add` reads the doc comment `Add two integers.` and registers the tool under the name `add`. The same function is still an ordinary function: `add(2, 3).total` is `5`. A function that returns `Result<Sum, String>` and produces `Err` is a tool failure, the same `is_error` path as chapter 3.

The snapshot is `steps/ch05-1-tool-macro`. It is a full copy of chapter 4 plus a proc-macro crate, `ch05-1-fastmcp-macros`, library name `fastmcp_macros`. The server crate is `ch05-1-fastmcp-core`, library name `fastmcp_core`.

```bash
cargo run -p ch05-1-fastmcp-core --example demo
```

## Goals

- Explain what a procedural macro does, and when it runs.
- Write a `#[tool]` function whose name and description come from the function, and register it with `tool_decl`.
- Say why the return type and the generated argument struct have to be public.
- Tell `rmcp`'s `#[tool]` (a `ToolRouter`) from this chapter's macro (a `ToolDecl` for `FastMcp`).
- Read the benchmark that separates a direct call from registration, and say which cost the macro already paid at compile time.

## Roadmap

**5.1 What the attribute expands to.** The problem is the glue. The macro reads the signature and emits a struct that implements `ToolDecl`. This is the whole feature.

**5.2 Register it, and call the function directly.** `tool_decl` is a thin call to `tool`. The function the macro left behind is how you know the expansion did not hide the code.

**5.3 Build, run, and benchmark.** The tests, the demo above, and the two timings: one registration, and a thousand direct calls.

| Section | Step | What it adds |
|---|---|---|
| 5.1–5.3 | `steps/ch05-1-tool-macro` | `#[tool]`, a `ToolDecl` impl, `FastMcp::tool_decl` |

> **How FastMCP does it**
>
> Read at commit [`e2fcc412`](https://github.com/PrefectHQ/fastmcp/commit/e2fcc412001ff7414095c5dc282f94c70299a612). Paths are under `fastmcp_slim/fastmcp/`.
>
> **Adopted.** [`decorators.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/decorators.py) `get_fastmcp_meta` reads metadata the decorator attached to the function (`__fastmcp__`), including through a bound method. [`server/providers/local_provider/decorators/tools.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/providers/local_provider/decorators/tools.py) `tool` (the implementation that starts near line 211) is the decorator: `@provider.tool`, `@provider.tool()`, or `@provider.tool("custom_name")`. It builds a tool from the function and stores it. This chapter's `#[tool]` is that decorator for one synchronous function: name, description, parameters, registration.
>
> **Adapted.** The Python decorator runs when the module is imported. A Rust procedural macro runs in the compiler, and the code it emits runs later, when you call `tool_decl`. The description defaults to the doc comment. `name = "..."` overrides the function name, which is the `"custom_name"` form of the Python decorator. The return type has to be public because it appears in the public `ToolDecl` trait. Python has no such privacy rule.
>
> **Deferred.** `task`, `timeout`, `auth`, `tags`, and `enabled` on `tool()` wait. The tasks runner is chapter 20. Auth is chapter 22. `rmcp`'s own `#[tool]` / `#[tool_router]` / `#[tool_handler]` macros fill a `ToolRouter` on `ServerHandler`. They are the SDK's shape, read in the crate's `tests/test_tool_macros.rs`. This chapter does not route through them, because the registry this book is building is `FastMcp::tool`.

> **Guardrails in this chapter**
>
> The notes are [docs/fastmcp-guardrails.md](../docs/fastmcp-guardrails.md).
>
> **The macro does not change the wire.** An unknown name is still `invalid_params`. Arguments that do not match the generated struct never enter the function. A function that returns `Err` is still a tool result with `is_error: true`. Those are the chapter 3 rules, and W4's validation half, applied to code the macro wrote.
>
> **A doc comment is a description, not a schema.** The schema still comes from the types, as in chapter 4. The comment is the string `tools/list` shows beside the name.

[personal note: the first macro expansion I printed, and what surprised me in it]

## 5.1 What the attribute expands to

You want to write the function and stop. The glue from chapter 4 is an argument struct, a name, a description, and a closure that calls the function. A procedural macro is Rust code that runs inside the compiler and returns more Rust code. `syn` reads the tokens. `quote` writes the tokens that replace the attribute. The macro crate is `steps/ch05-1-tool-macro/crates/fastmcp-macros`. A proc-macro crate is its own package because the compiler loads it as a plugin. It cannot be the same crate as `fastmcp_core`.

The function you write:

```rust
/// Add two integers.
#[tool]
fn add(left: i64, right: i64) -> Sum {
    Sum { total: left + right }
}
```

`#[tool]` is an attribute macro: it receives the attribute arguments (none, here) and the function, and it returns both the original function and a struct. The struct's name is the function name in Pascal case plus `Tool`, so `add` becomes `AddTool`. The description is the doc comment, with the leading spaces trimmed and the lines joined. `description = "..."` on the attribute wins when you pass it. `name = "fail"` is how the second test registers a function under a name that is not the function's own name.

The expansion builds an argument struct from the named parameters and implements `ToolDecl`. The trait lives in `fastmcp_core`, and the macro emits the impl by that path. The part that matters is the impl, which is what `tool_decl` calls:

```rust
impl fastmcp_core::ToolDecl for AddTool {
    type Params = __add_args;
    type Output = Sum;

    fn name() -> &'static str { "add" }
    fn description() -> &'static str { "Add two integers." }
    fn invoke(args: Self::Params) -> Result<Self::Output, String> {
        Ok(add(args.left, args.right))
    }
}
```

`Params` and `Output` are associated types: the trait says "some type," and the impl names which one. If the function returns `Result<Sum, String>`, `Output` is `Sum` and `invoke` returns the `Result` the function already built, without wrapping it in another `Ok`. A function with no return type is a compile error. A function that takes `self` is a compile error. This chapter's tools are plain functions.

`AddTool` is public, and so is the generated argument struct. `ToolDecl` is a public trait, so the types in its impl are part of the public interface of the module. A private `Sum` would leak through that interface and the compiler would reject the macro. Make the return type `pub`. That is a Rust privacy rule, and it shows up here because the macro made the registration public.

`rmcp`'s macro is a different expansion. On a method it looks like `#[tool(description = "...")]` above `async fn get_weather(&self, city: Parameters<GetWeatherRequest>)`, and `#[tool_router]` collects those methods into a `ToolRouter`. That is the right tool when the server *is* the `ServerHandler`. Ours is the right tool when the server is a `FastMcp` value you build by registering functions. Both are procedural macros. They write different Rust.

## 5.2 Register it, and call the function directly

`tool_decl` is three arguments passed to `tool`: the name, the description, and `T::invoke`.

```rust
pub fn tool_decl<T: ToolDecl + 'static>(self) -> Self {
    self.tool(T::name(), T::description(), T::invoke)
}
```

`'static` is required because `tool` stores the function for the rest of the process. `T::invoke` is a function pointer, which is `'static` once `T` is. The schema for `Params` and `Output` is built inside `tool`, once, the same way chapter 4 built it. The macro did not bake the JSON document into the binary. It baked the struct the document is generated from.

The demo then calls the tool through the registry and calls `add` directly. Both return a total of `5`. The direct call is the proof that the function survived the expansion. You can unit-test `add` without a server.

A missing field still fails before `invoke`. The generated struct has `left` and `right`, both required, because neither is `Option`. Chapter 4's rules did not move.

## 5.3 Build, run, and benchmark

The integration test in `tests/macro_tool.rs` is a separate crate, which is the only place a proc macro can expand against `fastmcp_core` without the server crate trying to load itself as a plugin. It registers `AddTool` from the doc comment, checks the description, and checks `add(2, 3)`. `FailTool` returns `Err("nope")`, and `dispatch_tool` reports `ToolError::Failed`.

Chapter 4's tests still pass in this copy. Thirty-nine tests passed: the previous 37, plus these two.

```bash
cargo test -p ch05-1-fastmcp-core
cargo run -p ch05-1-fastmcp-core --example demo
cargo bench -p ch05-1-fastmcp-core --bench macro_reg -- --warm-up-time 1 --measurement-time 2 --sample-size 10
```

The demo prints:

```text
add(2, 3) = {"total":5}
description: Add two integers.
direct: 5
```

The bench times two different things, on purpose. `register_macro_1` builds a server and calls `tool_decl`, which builds the schema. `call_fn_1000` calls `add` a thousand times and never touches the registry. The macro already ran, when this bench was compiled. Nothing in the timer is the proc macro.

The slope lines, on the same 4-vCPU guest as chapter 1:

```text
register_macro_1        time:   [3.1972 µs 3.2176 µs 3.2331 µs]
call_fn_1000            time:   [262.53 ns 264.30 ns 265.91 ns]
```

Local medians from that run's `estimates.json` (the slope above is a different statistic):

| Benchmark | Median | Low | High | Target |
|---|---|---|---|---|
| `register_macro_1` | 3.200 µs | 3.186 µs | 3.217 µs | < 100 µs |
| `call_fn_1000` | 263.2 ns | 262.0 ns | 264.8 ns | < 10 µs |

**Where the timer starts and stops.**

| Benchmark | Included | Excluded | Input | Faked | A good result |
|---|---|---|---|---|---|
| `register_macro_1` | `FastMcp::new` and `tool_decl::<AddTool>`, including both schema generations inside `tool` | The proc-macro expansion, which finished at compile time. No `dispatch_tool`. | `add(left: i64, right: i64) -> Sum`, description `add two integers` | No socket | Well under 100 µs. About 3.2 µs here, in the same band as chapter 4's schema generation. |
| `call_fn_1000` | 1,000 calls of `add(n, 1)` and a running total | Registration, schema generation, JSON | `n` from 0 through 999 | No server | Well under 10 µs. About 263 ns here. |

CI runs this step's tests on Ubuntu, macOS, and Windows when the step changes. Criterion runs on Ubuntu in the [Benchmarks workflow](https://github.com/agnathan/rust-mcp/actions/workflows/benchmarks.yml). The numbers below are `median.point_estimate` and the median confidence interval from `estimates.json` on [run 38862013588](https://github.com/agnathan/rust-mcp/actions/runs/38862013588) (2026-10-16T06:35:09Z). GitHub's `github.sha` for that pull request run is the merge commit `4bf53ac06faba49c79c3c6a90cc6ca30b3b515ac`. A push to `main` appends a later run to the dashboard sources in `benchmarks/`. A slow number does not fail the build.

| Benchmark | CI median | Low | High | Target |
|---|---|---|---|---|
| `register_macro_1` | 4.640 µs | 4.632 µs | 4.652 µs | < 100 µs |
| `call_fn_1000` | 342.5 ns | 342.0 ns | 343.2 ns | < 10 µs |

On the Ubuntu runner, one registration is 4.640 µs and a thousand direct calls are 342.5 ns. Those are the medians this chapter quotes. The local table above is the machine where the command transcript was captured.

### Milestone demo

`fastmcp-rs/` grows a `macro_tool_struct_name` stub. `tests/ch05.rs` expects `"AddTool"`. The function is `todo!()`, so the test panics until you fill it in. CI does not run that test.

### Rust Playground

[`book/playground/ch05-a-tool-macro.rs`](playground/ch05-a-tool-macro.rs) is the shape the macro emits: a function, and a struct whose `call` forwards to it. The [playground](https://play.rust-lang.org/) does not run procedural macros, and it does not have this book's crate, so the attribute itself stays in `fastmcp-macros`.

[Run the expansion on the playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=a288e6200859a566678ed79359a0ad55).

I executed that gist on the playground's stable channel, debug mode, edition 2021. It exited 0 and printed:

```text
5
add
```

## Recap

`#[tool]` on `fn add` leaves `add` callable and emits `AddTool`, which implements `ToolDecl`. `FastMcp::new("fastmcp-rs").tool_decl::<AddTool>()` registers it. The description defaults to the doc comment. The schema is still built at registration, so the macro did not make startup free. It removed the glue.

`cargo run -p ch05-1-fastmcp-core --example demo` prints the sum, the description, and the direct call. Locally, one registration is 3.200 µs and a thousand direct calls are 263.2 ns. The thousand calls are the function. The registration is the schema. The compiler already ran the macro. On the Ubuntu run linked above, one registration is 4.640 µs and a thousand direct calls are 342.5 ns.

[personal note: a moment where the compiler stopped me, and how I worked out what it wanted]

## What's next

Chapter 6 gives the server something to read besides tools. A static resource is a URI and a body. A template is a URI with `{param}` in it, and the match has to say which string the parameter borrowed.
