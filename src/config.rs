use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub base_url: String,
    pub api_key: String,
    pub fast_model: String,
    pub slow_model: String,
    pub fast_timeout: u64,
    pub slow_timeout: u64,
    pub custom_prompt: String,
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
    })
}

fn prompt_input(prompt: &str, default: &str) -> String {
    if default.is_empty() {
        print!("{}: ", prompt);
    } else {
        print!("{} [{}]: ", prompt, default);
    }
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim().to_string();
    if input.is_empty() {
        default.to_string()
    } else {
        input
    }
}

pub fn interactive_setup() {
    println!("=== how2cli 設定精靈 ===\n");

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

    let base_url = prompt_input("API Base URL", &default_url);
    let api_key = prompt_input("API Key", &default_key);
    let fast_model = prompt_input("Fast model", &default_fast);
    let slow_model = prompt_input("Slow model", &default_slow);
    let fast_timeout = prompt_input("Fast timeout (秒)", &default_ft.to_string());
    let slow_timeout = prompt_input("Slow timeout (秒)", &default_st.to_string());

    println!("\n自訂提示詞 (會附加在系統提示詞後面，直接 Enter 跳過)");
    println!("例: \"Always respond in Traditional Chinese\" 或 \"Prefer pacman over apt\"");
    let custom_prompt = prompt_input("Custom prompt", &default_custom);

    let cfg = FileConfig {
        base_url: Some(base_url),
        api_key: Some(api_key),
        fast_model: Some(fast_model),
        slow_model: Some(slow_model),
        fast_timeout: fast_timeout.parse().ok(),
        slow_timeout: slow_timeout.parse().ok(),
        custom_prompt: if custom_prompt.is_empty() { None } else { Some(custom_prompt) },
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
