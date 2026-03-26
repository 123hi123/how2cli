mod api;
mod config;
mod format;
mod prompt;
mod session;
mod shell;

use clap::Parser;
use serde_json::json;
use std::path::Path;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "h", version, about = "Natural language to shell command\n\n  h  = fast mode\n  ht = think mode (slow reasoning model)\n  h 1 = mode 1 shortcut")]
struct Cli {
    /// The query in natural language
    #[arg(trailing_var_arg = true)]
    query: Vec<String>,

    /// Use specific model
    #[arg(short, long)]
    model: Option<String>,

    /// Don't show explanation
    #[arg(long)]
    no_explain: bool,

    /// Run setup wizard
    #[arg(long)]
    setup: bool,

    /// List available models
    #[arg(long)]
    list_models: bool,

    /// Unlimited session history
    #[arg(short = 'u', long)]
    unlimited: bool,

    /// Talk mode (free conversation, no command format)
    #[arg(short = 't', long)]
    talk: bool,

    /// Raw output (command only, for piping)
    #[arg(long)]
    raw: bool,

    /// Clear session history for current directory
    #[arg(long)]
    clear: bool,

    /// Load all directories' session history (requires -u)
    #[arg(short = 'a', long)]
    all: bool,
}

/// Detect if invoked as "ht" (think/slow mode)
fn is_think_mode() -> bool {
    std::env::args()
        .next()
        .and_then(|arg0| {
            Path::new(&arg0)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
        })
        .map(|name| name == "ht")
        .unwrap_or(false)
}

/// Build messages array with session history
fn build_messages(
    system_prompt: &str,
    history: &[(String, String)],
    user_query: &str,
) -> Vec<serde_json::Value> {
    let mut messages = vec![json!({"role": "system", "content": system_prompt})];

    for (user_msg, assistant_msg) in history {
        messages.push(json!({"role": "user", "content": user_msg}));
        messages.push(json!({"role": "assistant", "content": assistant_msg}));
    }

    messages.push(json!({"role": "user", "content": user_query}));
    messages
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let slow = is_think_mode();

    // Handle --setup
    if cli.setup {
        config::interactive_setup();
        return;
    }

    // Handle --clear
    if cli.clear {
        session::clear();
        println!("已清空當前目錄的會話歷史。");
        return;
    }

    // Load config
    let cfg = match config::load_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let client = reqwest::Client::new();

    // Handle --list-models
    if cli.list_models {
        match api::list_models(&client, &cfg.base_url, &cfg.api_key).await {
            Ok(models) => {
                println!("可用模型:");
                for m in &models {
                    println!("  - {}", m);
                }
            }
            Err(e) => {
                eprintln!("取得模型列表失敗: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    // Check for query
    if cli.query.is_empty() {
        Cli::parse_from(["h", "--help"]);
        return;
    }

    // Parse mode number from first argument
    let (mode_num, user_query) = {
        let first = &cli.query[0];
        if first.len() == 1 {
            if let Some(c) = first.chars().next() {
                if c.is_ascii_digit() && c != '0' {
                    let num: usize = first.parse().unwrap();
                    (Some(num), cli.query[1..].join(" "))
                } else {
                    (None, cli.query.join(" "))
                }
            } else {
                (None, cli.query.join(" "))
            }
        } else {
            (None, cli.query.join(" "))
        }
    };

    if user_query.is_empty() {
        Cli::parse_from(["h", "--help"]);
        return;
    }

    // Resolve flags (CLI flags + mode overrides)
    let mut talk = cli.talk;
    let mut unlimited = cli.unlimited;
    let mut all_sessions = cli.all;
    let mut raw = cli.raw;
    let mut no_explain = cli.no_explain;

    if let Some(num) = mode_num {
        if let Some(mode) = cfg.modes.get(num - 1) {
            if let Some(flags) = &mode.flags {
                for flag in flags.split_whitespace() {
                    match flag {
                        "-t" | "--talk" => talk = true,
                        "-u" | "--unlimited" => unlimited = true,
                        "-a" | "--all" => all_sessions = true,
                        "--raw" => raw = true,
                        "--no-explain" => no_explain = true,
                        _ => {}
                    }
                }
            }
        }
    }

    // Load session history
    let history = if all_sessions && unlimited {
        session::load_all_history(None)
    } else if unlimited {
        session::load_history(None)
    } else {
        session::load_history(Some(cfg.session_limit as usize))
    };

    // Execute based on mode
    if talk {
        // Talk mode: free conversation, streaming
        let system_prompt = prompt::build_talk_prompt(&cfg.custom_prompt);
        let messages = build_messages(&system_prompt, &history, &user_query);
        let model = cli.model.as_deref().unwrap_or(&cfg.fast_model);

        let stream_print = cfg.stream_output;
        if stream_print { print!("\n  "); }
        match api::chat_completion_stream(
            &client, &cfg.base_url, &cfg.api_key, model, &messages, stream_print,
        )
        .await
        {
            Ok((response, usage)) => {
                if stream_print {
                    println!();
                } else {
                    format::format_talk(&response);
                }
                if cfg.show_token_usage && !raw {
                    format::format_token_usage(usage.input_tokens, usage.output_tokens);
                }
                session::append(&user_query, &response);
            }
            Err(e) => {
                eprintln!("\nAPI 請求失敗: {}", e);
                std::process::exit(1);
            }
        }
    } else if slow {
        // Think mode: streaming with slow model + timeout
        let shell_ctx = shell::ShellContext::detect();
        let model = cli.model.as_deref().unwrap_or(&cfg.slow_model);
        let system_prompt = prompt::build_search_prompt(&shell_ctx, &cfg.custom_prompt);
        let messages = build_messages(&system_prompt, &history, &user_query);
        let timeout_dur = Duration::from_secs(cfg.slow_timeout);

        let result = tokio::time::timeout(
            timeout_dur,
            api::chat_completion_stream(
                &client, &cfg.base_url, &cfg.api_key, model, &messages, false,
            ),
        )
        .await;

        match result {
            Ok(Ok((response, usage))) => {
                let (cmd, exp) = prompt::parse_response(&response);
                if raw {
                    format::format_raw(&cmd);
                } else {
                    format::format_result(&cmd, &exp, !no_explain);
                }
                if cfg.show_token_usage && !raw {
                    format::format_token_usage(usage.input_tokens, usage.output_tokens);
                }
                session::append(&user_query, &response);
            }
            Ok(Err(e)) => {
                eprintln!("API 請求失敗: {}", e);
                std::process::exit(1);
            }
            Err(_) => {
                eprintln!("請求逾時 ({}秒)", cfg.slow_timeout);
                std::process::exit(1);
            }
        }
    } else {
        // Fast mode: try search model streaming, fallback to direct model
        let shell_ctx = shell::ShellContext::detect();
        let model = cli.model.as_deref().unwrap_or(&cfg.fast_model);
        let search_model = if cli.model.is_some() {
            format!("{}-search", model)
        } else {
            cfg.search_model()
        };

        let system_prompt = prompt::build_direct_prompt(&shell_ctx, &cfg.custom_prompt);
        let messages = build_messages(&system_prompt, &history, &user_query);
        let timeout_dur = Duration::from_secs(cfg.fast_timeout);

        // Try search model first (streaming, buffered, with timeout)
        let search_messages = {
            let search_prompt = prompt::build_search_prompt(&shell_ctx, &cfg.custom_prompt);
            build_messages(&search_prompt, &history, &user_query)
        };

        let result = tokio::time::timeout(
            timeout_dur,
            api::chat_completion_stream(
                &client,
                &cfg.base_url,
                &cfg.api_key,
                &search_model,
                &search_messages,
                false,
            ),
        )
        .await;

        let (response, usage) = match result {
            Ok(Ok(r)) => r,
            _ => {
                // Fallback to direct model (streaming, buffered)
                match api::chat_completion_stream(
                    &client, &cfg.base_url, &cfg.api_key, model, &messages, false,
                )
                .await
                {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("API 請求失敗: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        };

        let (cmd, exp) = prompt::parse_response(&response);
        if raw {
            format::format_raw(&cmd);
        } else {
            format::format_result(&cmd, &exp, !no_explain);
        }
        if cfg.show_token_usage && !raw {
            format::format_token_usage(usage.input_tokens, usage.output_tokens);
        }
        session::append(&user_query, &response);
    }
}
