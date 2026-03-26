use serde_json::json;
use std::io::Write;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Count how many terminal lines a string occupies (accounting for wrapping + 2-char indent).
fn count_display_lines(text: &str, term_width: usize) -> u16 {
    let usable = term_width.saturating_sub(2).max(1); // 2-char indent
    let mut lines: u16 = 0;
    for line in text.split('\n') {
        let len = line.len();
        lines += ((len / usable) + 1) as u16;
    }
    // Subtract 1 because the cursor sits on the last line (no extra newline)
    lines.saturating_sub(1)
}

/// Token usage info from API response
#[derive(Debug, Default)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

/// Non-streaming API call (for testbench)
pub async fn chat_completion(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    model: &str,
    messages: &[serde_json::Value],
) -> Result<(String, TokenUsage)> {
    let url = format!("{}/v1/chat/completions", base_url.trim_end_matches('/'));

    let body = json!({
        "model": model,
        "messages": messages,
    });

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("API 請求失敗 ({}): {}", status, text).into());
    }

    let data: serde_json::Value = resp.json().await?;

    let content = data["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("無法解析 API 回應")?
        .to_string();

    let usage = TokenUsage {
        input_tokens: data["usage"]["prompt_tokens"].as_u64().unwrap_or(0),
        output_tokens: data["usage"]["completion_tokens"].as_u64().unwrap_or(0),
    };

    Ok((content, usage))
}

/// Streaming API call via SSE. Prints tokens in real-time if `print` is true.
/// Returns the full accumulated response and token usage.
pub async fn chat_completion_stream(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    model: &str,
    messages: &[serde_json::Value],
    print: bool,
) -> Result<(String, TokenUsage)> {
    let url = format!("{}/v1/chat/completions", base_url.trim_end_matches('/'));

    let body = json!({
        "model": model,
        "messages": messages,
        "stream": true,
    });

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("API 請求失敗 ({}): {}", status, text).into());
    }

    let mut full_response = String::new();
    let mut token_usage = TokenUsage::default();
    let mut buffer = String::new();
    let mut resp = resp;
    let mut printed_lines: u16 = 0;

    while let Some(chunk) = resp.chunk().await? {
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        // Process complete SSE lines from buffer
        while let Some(newline_pos) = buffer.find('\n') {
            let line = buffer[..newline_pos].trim().to_string();
            buffer = buffer[newline_pos + 1..].to_string();

            if line.is_empty() {
                continue;
            }

            if let Some(json_str) = line.strip_prefix("data: ") {
                if json_str.trim() == "[DONE]" {
                    if print {
                        println!();
                    }
                    return Ok((full_response, token_usage));
                }

                if let Ok(data) = serde_json::from_str::<serde_json::Value>(json_str) {
                    if let Some(content) = data["choices"][0]["delta"]["content"].as_str() {
                        full_response.push_str(content);
                        if print {
                            use crossterm::{cursor, terminal, execute};
                            // Move cursor back to start of our output
                            if printed_lines > 0 {
                                let _ = execute!(std::io::stdout(), cursor::MoveUp(printed_lines));
                            }
                            // Always clear current line and everything below
                            let _ = execute!(
                                std::io::stdout(),
                                cursor::MoveToColumn(0),
                                terminal::Clear(terminal::ClearType::FromCursorDown)
                            );
                            // Reprint full accumulated response with indent
                            let display = full_response.replace('\n', "\n  ");
                            print!("  {}", display);
                            let _ = std::io::stdout().flush();
                            // Count lines for next rewrite
                            let term_width = crossterm::terminal::size()
                                .map(|(w, _)| w as usize)
                                .unwrap_or(80);
                            printed_lines = count_display_lines(&full_response, term_width);
                        }
                    }
                    if let Some(usage) = data.get("usage") {
                        token_usage.input_tokens = usage["prompt_tokens"].as_u64().unwrap_or(0);
                        token_usage.output_tokens = usage["completion_tokens"].as_u64().unwrap_or(0);
                    }
                }
            }
        }
    }

    if print && !full_response.is_empty() {
        println!();
    }

    Ok((full_response, token_usage))
}

pub async fn list_models(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
) -> Result<Vec<String>> {
    let url = format!("{}/v1/models", base_url.trim_end_matches('/'));

    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("API 請求失敗 ({}): {}", status, text).into());
    }

    let data: serde_json::Value = resp.json().await?;

    let models = data["data"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m["id"].as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    Ok(models)
}
