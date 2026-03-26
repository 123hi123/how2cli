use std::path::Path;
use base64::Engine;
use serde_json::json;

/// Detect MIME type from file extension
fn detect_mime(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        _ => "image/png", // default fallback
    }
}

/// Load an image file and return it as a serde_json::Value for OAI vision API.
/// Format: {"type": "image_url", "image_url": {"url": "data:image/png;base64,..."}}
pub fn load_image(path: &Path) -> Result<serde_json::Value, String> {
    let data = std::fs::read(path)
        .map_err(|e| format!("無法讀取圖片 {}: {}", path.display(), e))?;

    let mime = detect_mime(path);
    let b64 = base64::engine::general_purpose::STANDARD.encode(&data);

    Ok(json!({
        "type": "image_url",
        "image_url": {
            "url": format!("data:{};base64,{}", mime, b64)
        }
    }))
}

/// Grab image from system clipboard. Returns Ok(Some(json)) if image found, Ok(None) if no image.
/// Currently supports Wayland (wl-paste). TODO: macOS (pbpaste), Windows (PowerShell).
pub fn grab_clipboard() -> Result<Option<serde_json::Value>, String> {
    // Try Wayland first
    if cfg!(target_os = "linux") {
        // Check if image is in clipboard by trying wl-paste with image type
        let check = std::process::Command::new("wl-paste")
            .args(["--list-types"])
            .output();

        if let Ok(output) = check {
            let types = String::from_utf8_lossy(&output.stdout);
            let has_png = types.lines().any(|t| t.trim() == "image/png");
            let has_jpeg = types.lines().any(|t| t.trim() == "image/jpeg");

            let (mime, type_arg) = if has_png {
                ("image/png", "image/png")
            } else if has_jpeg {
                ("image/jpeg", "image/jpeg")
            } else {
                // No image in clipboard
                return Ok(None);
            };

            let result = std::process::Command::new("wl-paste")
                .args(["-t", type_arg])
                .output()
                .map_err(|e| format!("wl-paste 失敗: {}", e))?;

            if result.status.success() && !result.stdout.is_empty() {
                let b64 = base64::engine::general_purpose::STANDARD.encode(&result.stdout);
                return Ok(Some(json!({
                    "type": "image_url",
                    "image_url": {
                        "url": format!("data:{};base64,{}", mime, b64)
                    }
                })));
            }
        }

        // Fallback: try xclip for X11
        let result = std::process::Command::new("xclip")
            .args(["-selection", "clipboard", "-t", "image/png", "-o"])
            .output();

        if let Ok(output) = result {
            if output.status.success() && !output.stdout.is_empty() {
                let b64 = base64::engine::general_purpose::STANDARD.encode(&output.stdout);
                return Ok(Some(json!({
                    "type": "image_url",
                    "image_url": {
                        "url": format!("data:image/png;base64,{}", b64)
                    }
                })));
            }
        }
    }

    // TODO: macOS — use `osascript` or `pbpaste` to get clipboard image
    // TODO: Windows — use PowerShell Get-Clipboard -Format Image

    Ok(None)
}

/// Build a user message content array with text + optional images.
/// OAI format: content is an array of {"type": "text"/"image_url", ...}
pub fn build_content_with_images(
    text: &str,
    images: &[serde_json::Value],
) -> serde_json::Value {
    if images.is_empty() {
        // Simple text content (string, not array) for compatibility
        json!(text)
    } else {
        // Array content with images + text
        let mut content: Vec<serde_json::Value> = images.to_vec();
        content.push(json!({"type": "text", "text": text}));
        json!(content)
    }
}
