# Social posts: Chapter 10, "Dependency injection"

## X

Chapter 10 of Learning Rust by Building FastMCP is up: dependencies the schema does not list.

`provide` stores a database and a token by type. The tool asks for `Dep<Db>` and `Dep<Token>`. The client sends `{q: "widgets"}`. `required` is `q`. A `PhantomData<fn() -> T>` keeps `Dep` invariant in `T`.

A thousand extractions and a thousand hand-passed Arc clones are both small. The bench says which loop did what.

[personal note]

#rustlang #MCP

## LinkedIn

Chapter 10 of Learning Rust by Building FastMCP is out: dependency injection.

FastMCP's `Depends` removes a parameter from the signature the schema is built from. The Rust chapter keys a map by `TypeId` and extracts `Dep<T>` before the tool runs. A tuple implements `Extract`, so one tool can take the database and the token together. Neither type appears in `required`.

[personal note]

Written for a reader who is new to Rust. `PhantomData` shows up here because the dependency's type has to stay exact.

#Rust #MCP #LearningInPublic #SoftwareEngineering
