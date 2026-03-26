# Configuration

## Setup

Run the interactive TUI setup:

```bash
h --setup
```

This guides you through all settings with `dialoguer` prompts. Existing values are shown as defaults — press Enter to keep them.

## Config File

Location: `~/.config/how2cli/config.toml`

```toml
base_url = "https://ds.fsmallcold.top"
api_key = "sk-..."
fast_model = "deepseek-chat"
slow_model = "deepseek-reasoner"
fast_timeout = 30
slow_timeout = 300
custom_prompt = "Always respond in Traditional Chinese"
session_limit = 100
show_token_usage = true
stream_output = true

[[modes]]
name = "talk unlimited"
flags = "-t -u"

[[modes]]
name = "global context"
flags = "-t -u -a"
```

## Settings Reference

| Setting | Default | Description |
|---------|---------|-------------|
| `base_url` | `https://ds.fsmallcold.top` | OAI-compatible API endpoint |
| `api_key` | (required) | API authentication key |
| `fast_model` | `deepseek-chat` | Model for fast mode. Search variant auto-derived as `{model}-search` |
| `slow_model` | `deepseek-reasoner` | Model for think mode (`ht`) |
| `fast_timeout` | `30` | Timeout in seconds for fast mode search request |
| `slow_timeout` | `300` | Timeout in seconds for think mode |
| `custom_prompt` | (empty) | Appended to all system prompts |
| `session_limit` | `100` | Max messages loaded into context per request |
| `show_token_usage` | `true` | Display input/output token counts after each response |
| `stream_output` | `true` | Enable SSE streaming (tokens appear in real-time) |
| `modes` | (empty) | Preset flag combinations, accessed via `h 1`, `h 2`, etc. |

## Environment Variables

All settings can be overridden via environment variables (highest priority):

```bash
export HOW2_BASE_URL=https://api.example.com
export HOW2_API_KEY=sk-...
export HOW2_FAST_MODEL=gpt-4o-mini
export HOW2_SLOW_MODEL=gpt-4o
export HOW2_FAST_TIMEOUT=60
export HOW2_SLOW_TIMEOUT=600
export HOW2_CUSTOM_PROMPT="Respond in English"
export HOW2_SESSION_LIMIT=200
```

## .env Files

Searched in order:
1. `.env` in current directory
2. `~/.config/how2cli/.env`
3. `~/.how2cli.env`

## Modes

Modes are shortcuts for flag combinations. Define in setup or config file:

```toml
[[modes]]
name = "talk"
flags = "-t"

[[modes]]
name = "deep talk"
flags = "-t -u -a"
```

Usage: `h 1 what is rust` applies mode 1's flags to the query.

Available flags for modes: `-t`, `-u`, `-a`, `--raw`, `--no-explain`
