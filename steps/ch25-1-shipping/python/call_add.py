"""Time one in-process FastMCP tool call.

Run with a virtualenv that has the server extra of fastmcp-slim at e2fcc412.
This script does not open a socket. Middleware is off, matching the Rust
`dispatch_tool` path.
"""

import asyncio
import time

from fastmcp import FastMCP

LOOPS = 1000
SAMPLES = 10


def server() -> FastMCP:
    mcp = FastMCP("widgets")

    @mcp.tool
    def add(a: int, b: int) -> int:
        return a + b

    return mcp


async def main() -> None:
    mcp = server()
    once = await mcp.call_tool("add", {"a": 2, "b": 3}, run_middleware=False)
    text = once.content[0].text
    for _ in range(50):
        await mcp.call_tool("add", {"a": 2, "b": 3}, run_middleware=False)
    samples = []
    for _ in range(SAMPLES):
        started = time.perf_counter_ns()
        for _ in range(LOOPS):
            await mcp.call_tool("add", {"a": 2, "b": 3}, run_middleware=False)
        samples.append((time.perf_counter_ns() - started) / LOOPS)
    samples.sort()
    median = samples[len(samples) // 2]
    print(f"python: {text}")
    print("middleware: off")
    print(f"per_call_ns: {median:.1f}")
    print(f"low_ns: {samples[0]:.1f}")
    print(f"high_ns: {samples[-1]:.1f}")
    print(f"samples: {SAMPLES}")
    print(f"loops: {LOOPS}")


if __name__ == "__main__":
    asyncio.run(main())
