use serde_json::json;
use std::io::Write;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Non-streaming API call (for testbench)
pub async fn chat_completion(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    model: &str,
    messages: &[serde_json::Value],
) -> Result<String> {
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

    Ok(content)
}

/// Streaming API call via SSE. Prints tokens in real-time if `print` is true.
/// Returns the full accumulated response.
pub async fn chat_completion_stream(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    model: &str,
    messages: &[serde_json::Value],
    print: bool,
) -> Result<String> {
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
    let mut buffer = String::new();
    let mut resp = resp;

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
                    return Ok(full_response);
                }

                if let Ok(data) = serde_json::from_str::<serde_json::Value>(json_str) {
                    if let Some(content) = data["choices"][0]["delta"]["content"].as_str() {
                        full_response.push_str(content);
                        if print {
                            print!("{}", content);
                            let _ = std::io::stdout().flush();
                        }
                    }
                }
            }
        }
    }

    if print && !full_response.is_empty() {
        println!();
    }

    Ok(full_response)
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
