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
#[command(name = "h", version, about = "Natural language to shell command\n\n  h  = fast mode (dual-request parallel)\n  ht = think mode (slow reasoning model)")]
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

    let user_query = cli.query.join(" ");

    // Load session history
    let history_limit = if cli.unlimited {
        None
    } else {
        Some(cfg.session_limit as usize)
    };
    let history = session::load_history(history_limit);

    if cli.talk {
        // Talk mode: free conversation, streaming
        let system_prompt = prompt::build_talk_prompt(&cfg.custom_prompt);
        let messages = build_messages(&system_prompt, &history, &user_query);
        let model = cli.model.as_deref().unwrap_or(&cfg.fast_model);

        print!("\n  ");
        match api::chat_completion_stream(
            &client, &cfg.base_url, &cfg.api_key, model, &messages, true,
        ).await {
            Ok(response) => {
                println!();
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

        // Stream but buffer (don't print) — parse COMMAND/EXPLANATION after completion
        let result = tokio::time::timeout(
            timeout_dur,
            api::chat_completion_stream(
                &client, &cfg.base_url, &cfg.api_key, model, &messages, false,
            ),
        ).await;

        match result {
            Ok(Ok(response)) => {
                let (cmd, exp) = prompt::parse_response(&response);
                if cli.raw {
                    format::format_raw(&cmd);
                } else {
                    format::format_result(&cmd, &exp, !cli.no_explain);
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
                &client, &cfg.base_url, &cfg.api_key, &search_model, &search_messages, false,
            ),
        ).await;

        let response = match result {
            Ok(Ok(resp)) => resp,
            _ => {
                // Fallback to direct model (streaming, buffered)
                match api::chat_completion_stream(
                    &client, &cfg.base_url, &cfg.api_key, model, &messages, false,
                ).await {
                    Ok(resp) => resp,
                    Err(e) => {
                        eprintln!("API 請求失敗: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        };

        let (cmd, exp) = prompt::parse_response(&response);
        if cli.raw {
            format::format_raw(&cmd);
        } else {
            format::format_result(&cmd, &exp, !cli.no_explain);
        }
        session::append(&user_query, &response);
    }
}
