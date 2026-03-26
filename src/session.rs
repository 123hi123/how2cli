use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

fn session_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("how2cli")
        .join("sessions")
}

fn sanitize_path(path: &str) -> String {
    path.replace('/', "_")
        .replace('\\', "_")
        .trim_start_matches('_')
        .to_string()
}

fn session_path() -> PathBuf {
    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    session_dir().join(format!("{}.jsonl", sanitize_path(&cwd)))
}

/// Load conversation history as (user, assistant) pairs.
/// `limit`: max number of messages (user+assistant each count as 1).
/// None = unlimited.
pub fn load_history(limit: Option<usize>) -> Vec<(String, String)> {
    let path = session_path();
    if !path.exists() {
        return vec![];
    }

    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let mut pairs: Vec<(String, String)> = Vec::new();
    let mut pending_user: Option<String> = None;

    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(msg) = serde_json::from_str::<Message>(line) {
            match msg.role.as_str() {
                "user" => {
                    pending_user = Some(msg.content);
                }
                "assistant" => {
                    if let Some(user_msg) = pending_user.take() {
                        pairs.push((user_msg, msg.content));
                    }
                }
                _ => {}
            }
        }
    }

    // Apply limit
    match limit {
        Some(n) => {
            let msg_limit = n / 2; // n messages = n/2 pairs
            let skip = pairs.len().saturating_sub(msg_limit.max(1));
            pairs.into_iter().skip(skip).collect()
        }
        None => pairs,
    }
}

/// Append a user+assistant exchange to the session file.
pub fn append(user_msg: &str, assistant_msg: &str) {
    let path = session_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }

    let user = Message {
        role: "user".to_string(),
        content: user_msg.to_string(),
    };
    let assistant = Message {
        role: "assistant".to_string(),
        content: assistant_msg.to_string(),
    };

    let mut data = String::new();
    if let Ok(json) = serde_json::to_string(&user) {
        data.push_str(&json);
        data.push('\n');
    }
    if let Ok(json) = serde_json::to_string(&assistant) {
        data.push_str(&json);
        data.push('\n');
    }

    use std::io::Write;
    if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = file.write_all(data.as_bytes());
    }
}

/// Clear the session for the current CWD.
pub fn clear() {
    let path = session_path();
    if path.exists() {
        fs::remove_file(&path).ok();
    }
}
