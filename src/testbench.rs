mod api;
mod config;
mod format;
mod prompt;
mod session;
mod shell;

use serde::Deserialize;
use std::fs;
use std::time::Instant;
use colored::Colorize;

#[derive(Deserialize)]
struct TestCase {
    id: u32,
    category: String,
    query: String,
    expected_commands: Vec<String>,
}

/// Extract base command(s) from a single pipeline segment.
/// "docker ps -a" → ["docker", "ps"]
/// "sudo ss -tulpn" → ["ss"]
/// "find . -name '*.txt'" → ["find"]
fn extract_base_from_segment(segment: &str) -> Vec<String> {
    let compound_prefixes = ["docker", "git", "systemctl", "apt", "pacman", "brew",
                              "dnf", "zypper", "cargo", "npm", "pip", "kubectl",
                              "docker-compose"];

    let tokens: Vec<&str> = segment.split_whitespace().collect();
    if tokens.is_empty() {
        return vec![];
    }

    // Skip "sudo" prefix
    let start = if tokens[0].to_lowercase() == "sudo" { 1 } else { 0 };
    if start >= tokens.len() {
        return vec![];
    }

    let first = tokens[start].to_lowercase();

    // Check for compound commands
    if tokens.len() > start + 1 && compound_prefixes.contains(&first.as_str()) {
        let second = tokens[start + 1].to_lowercase();
        if !second.starts_with('-') {
            return vec![first, second];
        }
    }

    vec![first]
}

/// Extract all base commands from a full command (handling pipes).
/// "echo hello | tr a-z A-Z" → [["echo"], ["tr"]]
/// "cat /dev/urandom | tr -dc ... | head -c 16" → [["cat"], ["tr"], ["head"]]
fn extract_all_bases(cmd: &str) -> Vec<Vec<String>> {
    // Also handle ; as command separator
    cmd.split('|')
        .flat_map(|part| part.split(';'))
        .map(|seg| extract_base_from_segment(seg.trim()))
        .filter(|b| !b.is_empty())
        .collect()
}

/// Check if the AI's command matches any expected command.
/// Matching strategy:
/// 1. Compare base commands of each pipeline segment
/// 2. If any segment's base command matches, it's a pass
/// 3. Handle sudo prefix transparently
fn command_matches(ai_cmd: &str, expected_commands: &[String]) -> bool {
    let ai_bases = extract_all_bases(ai_cmd);
    if ai_bases.is_empty() {
        return false;
    }

    for expected in expected_commands {
        let exp_bases = extract_all_bases(expected);
        if exp_bases.is_empty() {
            continue;
        }

        // Check if any AI base matches any expected base
        for ai_base in &ai_bases {
            for exp_base in &exp_bases {
                if ai_base == exp_base {
                    return true;
                }
            }
        }
    }

    false
}

#[tokio::main]
async fn main() {
    // Load .env
    let _ = dotenvy::dotenv();

    let cfg = match config::load_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("設定載入失敗: {}", e);
            std::process::exit(1);
        }
    };

    // Read testbench
    let testbench_path = "testbench.json";
    let content = match fs::read_to_string(testbench_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("無法讀取 {}: {}", testbench_path, e);
            std::process::exit(1);
        }
    };

    let test_cases: Vec<TestCase> = match serde_json::from_str(&content) {
        Ok(tc) => tc,
        Err(e) => {
            eprintln!("JSON 解析失敗: {}", e);
            std::process::exit(1);
        }
    };

    let total = test_cases.len();
    let mut passed = 0;
    let mut failed_cases: Vec<(u32, String, String, String)> = Vec::new();
    let client = reqwest::Client::new();
    let shell_ctx = shell::ShellContext::detect();
    let system_prompt = prompt::build_direct_prompt(&shell_ctx, "");

    println!("{}", format!("=== how2cli Testbench ({} 題) ===\n", total).bold());

    let start_time = Instant::now();

    for (i, tc) in test_cases.iter().enumerate() {
        let query = &tc.query;
        print!("[{:>3}/{}] {} ... ", i + 1, total, query);

        let messages = vec![
            serde_json::json!({"role": "system", "content": &system_prompt}),
            serde_json::json!({"role": "user", "content": query}),
        ];
        match api::chat_completion(
            &client,
            &cfg.base_url,
            &cfg.api_key,
            &cfg.fast_model,
            &messages,
        )
        .await
        {
            Ok((response, _usage)) => {
                let (cmd, _) = prompt::parse_response(&response);

                if command_matches(&cmd, &tc.expected_commands) {
                    passed += 1;
                    println!("{} ({})", "PASS".green().bold(), cmd.dimmed());
                } else {
                    println!("{} (got: {})", "FAIL".red().bold(), cmd.yellow());
                    failed_cases.push((
                        tc.id,
                        tc.category.clone(),
                        query.clone(),
                        cmd,
                    ));
                }
            }
            Err(e) => {
                println!("{} ({})", "ERROR".red().bold(), e);
                failed_cases.push((tc.id, tc.category.clone(), query.clone(), format!("ERROR: {}", e)));
            }
        }

        // Small delay to avoid rate limiting
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }

    let elapsed = start_time.elapsed();

    println!("\n{}", "=== 測試結果 ===".bold());
    println!("通過: {}/{} ({:.1}%)", passed, total, (passed as f64 / total as f64) * 100.0);
    println!("耗時: {:.1}秒", elapsed.as_secs_f64());

    if !failed_cases.is_empty() {
        println!("\n{}", "失敗題目:".red().bold());
        for (id, cat, query, got) in &failed_cases {
            println!("  #{} [{}] \"{}\" → {}", id, cat, query, got);
        }
    }

    // Exit with error code if not all passed
    if passed < total {
        std::process::exit(1);
    }
}
