# Development Guide

## Prerequisites

- Rust 1.85+ (2024 edition)
- An OAI-compatible API key

## Build

```bash
cargo build --release
```

Binaries output to cargo's target directory. Find it with:

```bash
cargo metadata --no-deps 2>/dev/null | grep target_directory
```

## Install locally

```bash
cp target/release/h ~/.local/bin/
cp target/release/ht ~/.local/bin/
```

## Project Structure

```
src/
├── main.rs        CLI entry point, mode resolution, orchestration
├── config.rs      Config loading (.env/TOML), TUI setup with dialoguer
├── api.rs         OAI-compatible API client with SSE streaming
├── prompt.rs      System prompt builder + response parser
├── session.rs     Per-CWD conversation history (JSONL)
├── shell.rs       OS/shell/package manager detection
├── format.rs      Terminal output with termimad markdown rendering
└── testbench.rs   100-question automated test runner
```

## Running testbench

The testbench sends 100 real queries to the API and validates the command responses:

```bash
cargo build --release --bin testbench
./target/release/testbench    # run from project root (needs testbench.json)
```

Requires `.env` or config with valid API credentials.

## Adding a new CLI flag

1. Add the flag to `Cli` struct in `main.rs`
2. If it should be available in modes, add parsing in the mode flags matcher
3. Implement the behavior in the appropriate section of `main()`

## Adding a new config option

1. Add field to `Config` struct in `config.rs`
2. Add field to `FileConfig` struct (as `Option<T>`)
3. Add loading logic in `load_config()`
4. Add TUI prompt in `interactive_setup()`
5. Add to the `FileConfig` construction before save

## Shell integration

For zsh (recommended — prevents glob expansion of `?`, `*`):

```bash
# Add to ~/.zshrc
source /path/to/how2cli/shell/h.zsh
```

This wraps `h` and `ht` with `noglob` so special characters work without quoting.

## Documentation

| Doc | Contents |
|-----|----------|
| [architecture.md](architecture.md) | System overview, data flow, file map |
| [configuration.md](configuration.md) | All settings, env vars, modes |
| [development.md](development.md) | This file — build, test, contribute |
