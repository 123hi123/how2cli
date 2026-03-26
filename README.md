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

- **Dual-request parallel architecture** - fires two AI requests simultaneously (direct + search-enhanced). If search times out, the direct answer is always there as fallback. You never wait for nothing.
- **Two modes, two commands** - `h` for instant answers (~3s), `ht` for deep thinking with reasoning models.
- **5.4 MB single binary** - zero runtime dependencies, smaller than navi (~8MB), aichat (~10MB), tgpt (~12MB).
- **Any OAI-compatible API** - works with DeepSeek, OpenAI, Groq, local vLLM, or any OpenAI-compatible endpoint. Not locked to a single provider.
- **Explains to help you remember** - doesn't just give you the command, explains what each flag does so you learn and won't need to ask again.
- **Custom prompts** - add your own instructions (e.g. "always respond in Chinese", "prefer pacman") via `h --setup`.
- **100/100 testbench** - validated against 100 real-world queries across file ops, Docker, Git, networking, text processing, and more.

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
│                Detects h vs ht via argv[0]
├── config.rs    Config loading (.env → env vars → config.toml)
│                Interactive setup wizard (h --setup)
├── api.rs       OpenAI-compatible API client
│                POST /v1/chat/completions + GET /v1/models
├── prompt.rs    System prompt construction (direct + search)
│                Response parsing (COMMAND: / EXPLANATION:)
├── shell.rs     OS, shell, package manager detection
│                Provides context to LLM for accurate commands
└── format.rs    Terminal output formatting with colors
```

Config priority: **environment variables > `.env` file > `~/.config/how2cli/config.toml`**

## Cross-platform

| | Linux | macOS | Windows |
|---|---|---|---|
| Status | Supported | Should work (untested) | Planned |
| Shell detection | `$SHELL` | `$SHELL` | PowerShell/cmd |
| Config path | `~/.config/how2cli/` | `~/Library/Application Support/how2cli/` | `%APPDATA%/how2cli/` |
| Package manager | pacman/apt/dnf/zypper | brew | - |

## License

MIT
