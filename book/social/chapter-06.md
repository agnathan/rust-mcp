# Social posts: Chapter 6, "Resources and templates"

## X

Chapter 6 of Learning Rust by Building FastMCP is up: resources and URI templates.

A static resource is a URI and a body. A template is `res://{owner}/{repo}`. The match borrows the name from the pattern and the value from the URI, then copies both before the function runs. The function outlives the request, so it cannot hold the borrow.

Lookup among 10,000 static URIs is a HashMap get. Scanning 10,000 templates is the slower number, and the bench says which is which.

[personal note]

#rustlang #MCP

## LinkedIn

Chapter 6 of Learning Rust by Building FastMCP is out: static resources and `{param}` URI templates.

FastMCP's `match_uri_template` compiles `{var}` to one path segment and drops a fragment the template did not spell out. The Rust chapter writes that match by hand, with two lifetimes: the parameter name borrows the pattern, and the captured text borrows the URI the client sent. The server copies those slices into `String`s before it calls the template, because the template is stored on the server and the request is not.

`resources/list` and `resources/read` go out over the same session as the tools. An unknown URI is `invalid_params`. A `{param}` that would have to swallow a slash does not match.

[personal note]

Written for a reader who is new to Rust. Lifetimes show up here because the URI match needs them.

#Rust #MCP #LearningInPublic #SoftwareEngineering
