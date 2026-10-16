# Social posts: Chapter 4, "Schemas from types"

## X

Chapter 4 of Learning Rust by Building FastMCP is up: schemas from types.

`#[derive(JsonSchema)]` on the argument struct. `schemars` writes the JSON Schema that `tools/list` advertises. `left` and `right` land in `required`. `label: Option<String>` does not, and its type is `["string", "null"]`.

FastMCP does this with Pydantic. `function_parsing.py` calls `TypeAdapter.json_schema()` and then `compress_schema`. The Rust step is the same idea with a derive macro, and a smaller cleanup: drop the top-level title, keep the fields.

The return struct gets an output schema too. `Sum { total }` shows `total` as an integer.

[personal note]

The Rust in this chapter is only what it needs: derive macros, `schemars`, and `Option`.

#rustlang #MCP

## LinkedIn

Chapter 4 of Learning Rust by Building FastMCP is out: schemas from types.

FastMCP builds a tool's input schema from the function signature. `tools/function_parsing.py` uses a Pydantic `TypeAdapter` and then `utilities/json_schema.py` `compress_schema` to prune titles. The output schema is the same path for the return type, and a non-object return is wrapped.

The Rust chapter derives `JsonSchema` on the argument struct and the return struct:

- `required` is `["left", "right"]`
- `label: Option<String>` stays out of `required`, with schema `{"type": ["string", "null"]}`
- omitting `label` still runs the tool
- omitting `right` fails before the closure runs
- the bench times schema generation with no cache, once and a thousand times

`status` from the earlier chapters stays on the list, without an output schema.

[personal note]

Written for a reader who is new to Rust. Derive macros, `schemars`, and `Option` show up here because the schema needs them.

#Rust #MCP #LearningInPublic #SoftwareEngineering
