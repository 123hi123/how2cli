use std::path::PathBuf;

use dialoguer::{Confirm, Input, Password};
use dialoguer::theme::ColorfulTheme;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default)]
pub struct ModeConfig {
    pub name: Option<String>,
    pub flags: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub base_url: String,
    pub api_key: String,
    pub fast_model: String,
    pub slow_model: String,
    pub fast_timeout: u64,
    pub slow_timeout: u64,
    pub custom_prompt: String,
    pub session_limit: u64,
    pub modes: Vec<ModeConfig>,
    pub show_token_usage: bool,
    pub stream_output: bool,
}

impl Config {
    /// Derived search model: fast_model + "-search"
    pub fn search_model(&self) -> String {
        format!("{}-search", self.fast_model)
    }
}

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("how2cli")
        .join("config.toml")
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
struct FileConfig {
    base_url: Option<String>,
    api_key: Option<String>,
    fast_model: Option<String>,
    slow_model: Option<String>,
    fast_timeout: Option<u64>,
    slow_timeout: Option<u64>,
    custom_prompt: Option<String>,
    session_limit: Option<u64>,
    modes: Option<Vec<ModeConfig>>,
    show_token_usage: Option<bool>,
    stream_output: Option<bool>,
}

fn load_file_config() -> FileConfig {
    let path = config_path();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(cfg) = toml::from_str::<FileConfig>(&content) {
                return cfg;
            }
        }
    }
    FileConfig::default()
}

pub fn load_config() -> Result<Config, String> {
    // Try loading .env from multiple locations
    // 1. Current directory
    let _ = dotenvy::dotenv();
    // 2. ~/.config/how2cli/.env
    if let Some(config_dir) = dirs::config_dir() {
        let env_path = config_dir.join("how2cli").join(".env");
        let _ = dotenvy::from_path(&env_path);
    }
    // 3. Home directory
    if let Some(home) = dirs::home_dir() {
        let _ = dotenvy::from_path(home.join(".how2cli.env"));
    }

    let file_cfg = load_file_config();

    let base_url = std::env::var("HOW2_BASE_URL")
        .ok()
        .or(file_cfg.base_url)
        .unwrap_or_else(|| "https://ds.fsmallcold.top".to_string());

    let api_key = std::env::var("HOW2_API_KEY")
        .ok()
        .or(file_cfg.api_key)
        .unwrap_or_default();

    let fast_model = std::env::var("HOW2_FAST_MODEL")
        .ok()
        .or(file_cfg.fast_model)
        .unwrap_or_else(|| "deepseek-chat".to_string());

    let slow_model = std::env::var("HOW2_SLOW_MODEL")
        .ok()
        .or(file_cfg.slow_model)
        .unwrap_or_else(|| "deepseek-reasoner".to_string());

    let fast_timeout = std::env::var("HOW2_FAST_TIMEOUT")
        .ok()
        .and_then(|v| v.parse().ok())
        .or(file_cfg.fast_timeout)
        .unwrap_or(30);

    let slow_timeout = std::env::var("HOW2_SLOW_TIMEOUT")
        .ok()
        .and_then(|v| v.parse().ok())
        .or(file_cfg.slow_timeout)
        .unwrap_or(300);

    let custom_prompt = std::env::var("HOW2_CUSTOM_PROMPT")
        .ok()
        .or(file_cfg.custom_prompt)
        .unwrap_or_default();

    let session_limit = std::env::var("HOW2_SESSION_LIMIT")
        .ok()
        .and_then(|v| v.parse().ok())
        .or(file_cfg.session_limit)
        .unwrap_or(100);

    let modes = file_cfg.modes.unwrap_or_default();
    let show_token_usage = file_cfg.show_token_usage.unwrap_or(true);
    let stream_output = file_cfg.stream_output.unwrap_or(true);

    if api_key.is_empty() {
        return Err("未找到 API key。請執行 `h --setup` 進行設定。".to_string());
    }

    Ok(Config {
        base_url,
        api_key,
        fast_model,
        slow_model,
        fast_timeout,
        slow_timeout,
        custom_prompt,
        session_limit,
        modes,
        show_token_usage,
        stream_output,
    })
}

pub fn interactive_setup() {
    let theme = ColorfulTheme::default();

    println!("=== how2cli Setup ===\n");

    // Load existing values as defaults
    let _ = dotenvy::dotenv();
    let file_cfg = load_file_config();

    let default_url = std::env::var("HOW2_BASE_URL")
        .ok()
        .or(file_cfg.base_url)
        .unwrap_or_else(|| "https://ds.fsmallcold.top".to_string());

    let default_key = std::env::var("HOW2_API_KEY")
        .ok()
        .or(file_cfg.api_key)
        .unwrap_or_default();

    let default_fast = std::env::var("HOW2_FAST_MODEL")
        .ok()
        .or(file_cfg.fast_model)
        .unwrap_or_else(|| "deepseek-chat".to_string());

    let default_slow = std::env::var("HOW2_SLOW_MODEL")
        .ok()
        .or(file_cfg.slow_model)
        .unwrap_or_else(|| "deepseek-reasoner".to_string());

    let default_ft = std::env::var("HOW2_FAST_TIMEOUT")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .or(file_cfg.fast_timeout)
        .unwrap_or(30);

    let default_st = std::env::var("HOW2_SLOW_TIMEOUT")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .or(file_cfg.slow_timeout)
        .unwrap_or(300);

    let default_custom = std::env::var("HOW2_CUSTOM_PROMPT")
        .ok()
        .or(file_cfg.custom_prompt)
        .unwrap_or_default();

    let default_sl = std::env::var("HOW2_SESSION_LIMIT")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .or(file_cfg.session_limit)
        .unwrap_or(100);

    let default_show_token = file_cfg.show_token_usage.unwrap_or(true);
    let default_stream = file_cfg.stream_output.unwrap_or(true);
    let existing_modes = file_cfg.modes.unwrap_or_default();

    // 1. API Base URL
    let base_url: String = Input::with_theme(&theme)
        .with_prompt("API Base URL")
        .default(default_url)
        .interact_text()
        .unwrap();

    // 2. API Key
    let api_key: String = if default_key.is_empty() {
        Password::with_theme(&theme)
            .with_prompt("API Key")
            .allow_empty_password(false)
            .interact()
            .unwrap()
    } else {
        Input::with_theme(&theme)
            .with_prompt("API Key")
            .default(default_key)
            .interact_text()
            .unwrap()
    };

    // 3. Fast model
    let fast_model: String = Input::with_theme(&theme)
        .with_prompt("Fast model")
        .default(default_fast)
        .interact_text()
        .unwrap();

    // 4. Slow model
    let slow_model: String = Input::with_theme(&theme)
        .with_prompt("Slow model")
        .default(default_slow)
        .interact_text()
        .unwrap();

    // 5. Fast timeout
    let fast_timeout: u64 = Input::with_theme(&theme)
        .with_prompt("Fast timeout (秒)")
        .default(default_ft)
        .interact_text()
        .unwrap();

    // 6. Slow timeout
    let slow_timeout: u64 = Input::with_theme(&theme)
        .with_prompt("Slow timeout (秒)")
        .default(default_st)
        .interact_text()
        .unwrap();

    // 7. Custom prompt
    println!("\n自訂提示詞 (會附加在系統提示詞後面，直接 Enter 跳過)");
    println!("例: \"Always respond in Traditional Chinese\" 或 \"Prefer pacman over apt\"");
    let custom_prompt: String = Input::with_theme(&theme)
        .with_prompt("Custom prompt")
        .default(default_custom)
        .allow_empty(true)
        .interact_text()
        .unwrap();

    // 8. Session history limit
    let session_limit: u64 = Input::with_theme(&theme)
        .with_prompt("Session history limit (條)")
        .default(default_sl)
        .interact_text()
        .unwrap();

    // 9. Show token usage
    let show_token_usage: bool = Confirm::with_theme(&theme)
        .with_prompt("回應後顯示 token 用量？")
        .default(default_show_token)
        .interact()
        .unwrap();

    // 10. Stream output
    let stream_output: bool = Confirm::with_theme(&theme)
        .with_prompt("啟用串流輸出？(逐字顯示回應)")
        .default(default_stream)
        .interact()
        .unwrap();

    // 11. Mode setup
    println!("\n--- Mode Setup ---");
    let mode_count: u64 = Input::with_theme(&theme)
        .with_prompt("要定義幾個模式？(0-9)")
        .default(existing_modes.len() as u64)
        .validate_with(|input: &u64| -> Result<(), String> {
            if *input <= 9 {
                Ok(())
            } else {
                Err("最多只能定義 9 個模式".to_string())
            }
        })
        .interact_text()
        .unwrap();

    let mut modes: Vec<ModeConfig> = Vec::new();
    for i in 0..mode_count as usize {
        println!("\n  模式 {} :", i + 1);

        let default_name = existing_modes
            .get(i)
            .and_then(|m| m.name.clone())
            .unwrap_or_default();

        let default_flags = existing_modes
            .get(i)
            .and_then(|m| m.flags.clone())
            .unwrap_or_default();

        let name: String = Input::with_theme(&theme)
            .with_prompt("  Mode name")
            .default(default_name)
            .allow_empty(true)
            .interact_text()
            .unwrap();

        let flags: String = Input::with_theme(&theme)
            .with_prompt("  Mode flags")
            .default(default_flags)
            .allow_empty(true)
            .interact_text()
            .unwrap();

        modes.push(ModeConfig {
            name: if name.is_empty() { None } else { Some(name) },
            flags: if flags.is_empty() { None } else { Some(flags) },
        });
    }

    // Build and save config
    let cfg = FileConfig {
        base_url: Some(base_url),
        api_key: Some(api_key),
        fast_model: Some(fast_model),
        slow_model: Some(slow_model),
        fast_timeout: Some(fast_timeout),
        slow_timeout: Some(slow_timeout),
        custom_prompt: if custom_prompt.is_empty() { None } else { Some(custom_prompt) },
        session_limit: Some(session_limit),
        modes: if modes.is_empty() { None } else { Some(modes) },
        show_token_usage: Some(show_token_usage),
        stream_output: Some(stream_output),
    };

    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let content = toml::to_string_pretty(&cfg).unwrap();
    match std::fs::write(&path, content) {
        Ok(_) => println!("\n設定已儲存至 {}", path.display()),
        Err(e) => eprintln!("\n儲存設定失敗: {}", e),
    }
}
