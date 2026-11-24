# Security

The supported version is the latest release.

Report a vulnerability privately through the repository's "Report a vulnerability" button
(GitHub private vulnerability reporting), not in a public issue.

The MCP server (`beatcraft-cli mcp`) is unconfined unless you pass `--automation-read-root` and
`--automation-write-root`: without them it reads and writes any path its client names, so confine it
and run it only with clients you trust. See [docs/wiki/MCP-Setup.md](docs/wiki/MCP-Setup.md).
