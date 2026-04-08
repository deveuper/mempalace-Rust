# MemPalace Rust Edition — AI Memory System

MemPalace Rust Edition — High-performance local AI memory with 96.6% recall. Semantic search over past conversations, all stored locally on your machine. Zero cloud, zero API calls.

## Architecture

**Wings** = people or projects (e.g., `wing_alice`, `wing_myproject`)
**Halls** = categories (facts, events, preferences, advice)
**Rooms** = specific topics (e.g., `database-design`, `api-refactor`)
**Drawers** = individual memory chunks (verbatim text)

## Protocol — FOLLOW THIS EVERY SESSION

**ON WAKE-UP**: Call `mempalace_status` to load palace overview.

**BEFORE RESPONDING** about any person, project, or past event: call `mempalace_search` FIRST. Never guess from memory — verify from the palace.

**IF UNSURE** about a fact (name, age, relationship, preference): say "let me check" and query. Wrong is worse than slow.

## Available Tools

### Search & Browse
- `mempalace_search` — Semantic search across all memories. Always start here.
- `mempalace_status` — Palace overview: total documents, wings, rooms
- `mempalace_wake_up` — Get wake-up context (identity + essential story)

### Write
- `mempalace_mine` — Mine files/conversations into the palace
- `mempalace_init` — Initialize a new palace for a directory

### Setup

The user needs to initialize and populate the palace first:

```bash
# Windows
mempalace.exe init C:\Users\Name\Documents\my-convos
mempalace.exe mine C:\Users\Name\Documents\my-convos

# macOS/Linux
./mempalace init ~/my-convos
./mempalace mine ~/my-convos
```

Then connect via MCP (for Claude Code, Cursor, Codex, etc.):

**Claude Code:**
```bash
claude mcp add mempalace -- C:\path\to\mempalace.exe mcp --transport stdio
```

**Cursor:**
Add to `~/.cursor/mcp.json`:
```json
{
  "mcpServers": {
    "mempalace": {
      "command": "C:\\path\\to\\mempalace.exe",
      "args": ["mcp", "--transport", "stdio"]
    }
  }
}
```

**OpenClaw:**
Add to MCP config:
```json
{
  "mcpServers": {
    "mempalace": {
      "command": "mempalace.exe",
      "args": ["mcp", "--transport", "stdio"]
    }
  }
}
```

## Tips

- Search is semantic (meaning-based), not keyword. "What did we discuss about database performance?" works better than "database".
- Wings auto-detect from directory names during mining.
- All data stays on your machine — no cloud, no API calls.

## Differences from Python Version

| Feature | Python | Rust Edition |
|---------|--------|--------------|
| Binary Size | ~50MB+ | <10MB |
| Startup Time | ~500ms | <50ms |
| Memory Usage | ~100MB | ~10MB |
| File Scanning | ~100 files/s | ~2000 files/s |
| Vector DB | ChromaDB | SQLite (built-in) |
| Dependencies | Many | Minimal |

## License

MIT License — See LICENSE file for details.
