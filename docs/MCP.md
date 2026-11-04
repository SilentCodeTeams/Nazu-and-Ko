# MCP

Nazu&Ko exposes a small stdio MCP server for local, read-only inspection.

## Start

```sh
nazuko mcp
```

Transport: newline-delimited JSON-RPC 2.0 on stdin/stdout. Do not write human
output to stdout; diagnostics belong on stderr.

## Client configuration

```json
{
  "mcpServers": {
    "nazuko": {
      "command": "nazuko",
      "args": ["mcp"]
    }
  }
}
```

## Protocol

- Supported versions: `2025-06-18`, `2025-03-26`, `2024-11-05`
- Capabilities: tools
- Resources and prompts are intentionally empty in this slice

## Tools

### `nazuko_version`

Returns the name, version, license and repository.

### `nazuko_analyze_pcap`

Arguments:

| Name | Type | Default | Meaning |
| --- | --- | --- | --- |
| `path` | string | required | Classic pcap path |
| `limit` | integer | 100 | Maximum packets returned (1–10000) |
| `hex` | boolean | false | Include payload hex |
| `dissect` | boolean | true | Detect protocol and version |

The result is a packet summary object and is also returned as
`structuredContent`.

## CLI JSON

```sh
nazuko version --json
nazuko analyze capture.pcap --json --limit 100
```

Both commands write one JSON object to stdout. Errors go to stderr with a
non-zero exit code.

## Safety boundary

This slice is read-only. It does not start proxies, capture live traffic, change
network interfaces, or write files. Write-capable MCP tools should be added only
after the control plane has explicit authorization, dry-run and audit semantics.
