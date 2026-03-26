<p align="center">
  <h1 align="center">how2cli</h1>
  <p align="center">
    <b>Natural language to shell command, instantly.</b><br>
    Type what you want to do, get the exact command + explanation.
  </p>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/language-Rust-orange?style=flat-square" alt="Rust">
  <img src="https://img.shields.io/badge/binary_size-5.4MB-green?style=flat-square" alt="Size">
  <img src="https://img.shields.io/badge/testbench-100%2F100-brightgreen?style=flat-square" alt="Testbench">
  <img src="https://img.shields.io/badge/platform-Linux-blue?style=flat-square" alt="Platform">
</p>

```
$ h list all docker containers

  docker ps -a
  ────────────────────────────────────────
  Lists all Docker containers (running + stopped).
  `ps` = process status, `-a` = all states.
```

## Highlights

- **SSE streaming** - responses appear token-by-token in real-time. Configurable in setup.
- **Session memory** - per-directory conversation history. AI remembers your previous questions. `-u` for unlimited, `-a` for cross-directory.
- **Talk mode (`-t`)** - free conversation beyond just shell commands. Ask anything.
- **Mode shortcuts** - define preset flag combos (e.g. `h 1 query` = talk + unlimited). Up to 9 modes.
- **Markdown rendering** - AI responses rendered with proper formatting (bold, code, lists, headers) via termimad.
- **Token usage** - shows input/output token counts after each response. Toggleable in setup.
- **Two commands** - `h` for fast answers (~3s), `ht` for deep thinking with reasoning models.
- **Single binary** - zero runtime dependencies, any OAI-compatible API endpoint.
- **TUI setup** - interactive setup wizard with dialoguer. All settings persist.
- **Shell integration** - `noglob` wrapper for zsh prevents `?*[]` glob issues.
- **100/100 testbench** - validated against 100 real-world queries.

## Installation

### From source (recommended)

```bash
git clone https://github.com/123hi123/how2cli.git
cd how2cli
cargo build --release
```

Copy binaries to your PATH:

```bash
cp target/release/h ~/.local/bin/
cp target/release/ht ~/.local/bin/
```

> Note: If your cargo uses a custom target directory, find the binary with `cargo metadata --no-deps 2>/dev/null | grep target_directory`.

### Arch Linux (AUR)

```bash
yay -S how2cli
```

### First-time setup

```bash
h --setup
```

You'll be prompted to enter:
- API Base URL (any OpenAI-compatible endpoint)
- API Key
- Fast model name (default: `deepseek-chat`)
- Slow model name (default: `deepseek-reasoner`)
- Timeout settings
- Custom prompt (optional, appended to system prompt)
- Session history limit (default: 100 messages)

Config is saved to `~/.config/how2cli/config.toml` and persists across sessions. Running `--setup` again loads your existing values as defaults — nothing gets lost.

You can also use a `.env` file or environment variables:

```bash
export HOW2_BASE_URL=https://your-api-endpoint.com
export HOW2_API_KEY=sk-your-key
export HOW2_FAST_MODEL=deepseek-chat
export HOW2_SLOW_MODEL=deepseek-reasoner
```

## Usage

### `h` - Fast mode (default)

```bash
h list all docker containers          # no quotes needed
h "find files larger than 100MB"      # quotes work too
h show disk usage of current dir
h compress a folder with tar
h --no-explain restart nginx          # command only, skip explanation
h -m gpt-4o what is my ip            # use a specific model
```

### `ht` - Think mode (reasoning model)

```bash
ht explain how iptables chains work
ht write a complex find command to delete old logs
ht debug why my ssh connection is refused
```

`ht` uses the slow/reasoning model (e.g. `deepseek-reasoner`) with a 5-minute timeout for complex questions that benefit from deeper analysis.

### `-t` - Talk mode (free conversation)

```bash
h -t what is Rust                     # free-form answer, no command format
h -t explain kubernetes in simple terms
h -u -t tell me about linux history   # unlimited history + talk
```

Talk mode uses only your custom prompt (from `--setup`). No `COMMAND:/EXPLANATION:` format — AI responds freely.

### `--raw` - Pipe-friendly output

```bash
h --raw list all docker containers    # outputs only: docker ps -a
h --raw compress this folder | sh     # pipe to shell for execution
h --raw find large files > script.sh  # save to script
```

### Session memory

Conversations are remembered per working directory. Default: last 100 messages.

```bash
h list docker containers              # asks a question
h what was my last question            # AI remembers: "list docker containers"
h -u show me more                     # -u = unlimited history (send all)
h --clear                             # clear current directory's session
```

Session files are stored at `~/.local/share/how2cli/sessions/`.

### Other options

```bash
h --setup          # (re)configure settings
h --list-models    # show available models from your API
h --help           # show help
h --version        # show version
```

## How it works

```
User: h list all docker containers

  ┌─ Request 1 (direct) ────────────────────────┐
  │  Model: deepseek-chat                        │
  │  Prompt: "answer directly from knowledge"    │
  │  Timeout: none (guaranteed baseline)         │
  └──────────────────────────────────────────────┘
                                                    → Best result wins
  ┌─ Request 2 (search-enhanced) ────────────────┐
  │  Model: deepseek-chat-search                 │
  │  Prompt: "search and verify before answering"│
  │  Timeout: 30s (configurable)                 │
  └──────────────────────────────────────────────┘

Output:
  docker ps -a
  ────────────────────────────────────────
  `docker ps` lists containers. `-a` includes stopped ones.
```

Both requests fire **simultaneously** via async Rust. If the search-enhanced request finishes in time, its (usually better) result is used. If it times out, the direct answer is already waiting. You always get an answer.

## Architecture

```
src/
├── main.rs      Entry point + dual-request parallel logic
│                Detects h vs ht via argv[0], session integration
├── config.rs    Config loading (.env → env vars → config.toml)
│                Interactive setup wizard (h --setup)
├── api.rs       OpenAI-compatible API client
│                POST /v1/chat/completions + GET /v1/models
├── prompt.rs    System prompt construction (direct + search + talk)
│                Response parsing (COMMAND: / EXPLANATION:)
├── session.rs   Per-CWD conversation history (JSONL)
│                Load/append/clear session files
├── shell.rs     OS, shell, package manager detection
│                Provides context to LLM for accurate commands
└── format.rs    Terminal output (colored, raw, talk modes)
```

Config priority: **environment variables > `.env` file > `~/.config/how2cli/config.toml`**

## Cross-platform

| | Linux | macOS | Windows |
|---|---|---|---|
| Status | Supported | Should work (untested) | Planned |
| Shell detection | `$SHELL` | `$SHELL` | PowerShell/cmd |
| Config path | `~/.config/how2cli/` | `~/Library/Application Support/how2cli/` | `%APPDATA%/how2cli/` |
| Package manager | pacman/apt/dnf/zypper | brew | - |

## Documentation

| Doc | Contents |
|-----|----------|
| [docs/architecture.md](docs/architecture.md) | System overview, data flow diagram, file map |
| [docs/configuration.md](docs/configuration.md) | All settings, env vars, modes, .env files |
| [docs/development.md](docs/development.md) | Build, test, contribute, add features |

## License

MIT
