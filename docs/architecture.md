# Architecture

## System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                          User Input                             │
│                                                                 │
│  h list docker containers     h -t explain rust     ht debug    │
│  h 1 query (mode shortcut)    h -u -a query         h --raw    │
└──────────────────────┬──────────────────────────────────────────┘
                       │
                       ▼
┌──────────────────────────────────────────────────────────────────┐
│                        main.rs                                   │
│                                                                  │
│  ┌─────────────┐  ┌──────────┐  ┌─────────┐  ┌──────────────┐  │
│  │ Mode Parser │  │ Flag     │  │ argv[0] │  │ Config       │  │
│  │ h 1 → flags │  │ Resolver │  │ h vs ht │  │ Loader       │  │
│  └──────┬──────┘  └────┬─────┘  └────┬────┘  └──────┬───────┘  │
│         └──────────────┴─────────────┴───────────────┘          │
│                          │                                       │
│              ┌───────────┴───────────┐                           │
│              ▼                       ▼                           │
│     ┌─────────────┐        ┌─────────────┐                      │
│     │  Talk Mode  │        │ Command Mode│                      │
│     │  -t flag    │        │  (default)  │                      │
│     │  free chat  │        │ COMMAND/EXP │                      │
│     └──────┬──────┘        └──────┬──────┘                      │
│            └──────────┬───────────┘                              │
│                       ▼                                          │
└───────────────────────┬──────────────────────────────────────────┘
                        │
           ┌────────────┼────────────┐
           ▼            ▼            ▼
   ┌──────────┐  ┌──────────┐  ┌──────────┐
   │session.rs│  │ prompt.rs│  │  api.rs   │
   │          │  │          │  │          │
   │ Per-CWD  │  │ System   │  │ OAI SSE  │
   │ JSONL    │  │ Prompt   │  │ Streaming│
   │ History  │  │ Builder  │  │ Client   │
   │          │  │          │  │          │
   │ load()   │  │ direct() │  │ stream() │
   │ append() │  │ search() │  │ models() │
   │ clear()  │  │ talk()   │  │          │
   │ all()    │  │ parse()  │  │ Token    │
   │          │  │          │  │ Usage    │
   └──────────┘  └──────────┘  └──────────┘
                        │
                        ▼
   ┌──────────┐  ┌──────────┐  ┌──────────┐
   │config.rs │  │ shell.rs │  │format.rs │
   │          │  │          │  │          │
   │ .env     │  │ OS       │  │ termimad │
   │ TOML     │  │ Shell    │  │ Markdown │
   │ dialoguer│  │ PkgMgr   │  │ Colored  │
   │ Modes    │  │          │  │ Raw/Talk │
   │ Setup TUI│  │          │  │ Tokens   │
   └──────────┘  └──────────┘  └──────────┘
```

## Data Flow

### Fast Mode (default)
```
User query
  → Load CWD session history
  → Build messages [system + history + query]
  → Try search model (SSE streaming, timeout 30s)
  → Fallback to direct model if timeout/error
  → Parse COMMAND: / EXPLANATION:
  → Render with termimad (markdown)
  → Show token usage
  → Append to session
```

### Talk Mode (-t)
```
User query
  → Load session history
  → Build messages [custom_prompt + history + query]
  → SSE streaming (tokens printed in real-time)
  → Render markdown
  → Show token usage
  → Append to session
```

### Think Mode (ht)
```
User query
  → Load session history
  → Build messages [search_prompt + history + query]
  → SSE streaming with slow model (timeout 300s)
  → Parse COMMAND: / EXPLANATION:
  → Render with termimad
  → Show token usage
  → Append to session
```

## File Map

| File | Responsibility | Key Types |
|------|---------------|-----------|
| `main.rs` | CLI entry, mode resolution, orchestration | `Cli` (clap) |
| `config.rs` | Config loading, TUI setup wizard | `Config`, `ModeConfig`, `FileConfig` |
| `api.rs` | OAI-compatible HTTP client, SSE streaming | `TokenUsage` |
| `prompt.rs` | System prompt construction, response parsing | - |
| `session.rs` | Per-CWD conversation history (JSONL) | `Message` |
| `shell.rs` | OS/shell/package manager detection | `ShellContext` |
| `format.rs` | Terminal output with markdown rendering | `MadSkin` |
| `testbench.rs` | 100-question automated test suite | `TestCase` |

## Config Priority

```
Environment variables  >  .env files  >  ~/.config/how2cli/config.toml
```

.env search order:
1. Current directory `.env`
2. `~/.config/how2cli/.env`
3. `~/.how2cli.env`

## Session Storage

```
~/.local/share/how2cli/sessions/
├── home_joe_dev_project-a.jsonl
├── home_joe_dev_how2cli.jsonl
└── tmp.jsonl
```

Each file is JSONL with alternating `{"role":"user",...}` and `{"role":"assistant",...}` lines.
