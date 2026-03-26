use crate::shell::ShellContext;

pub fn build_talk_prompt(custom_prompt: &str) -> String {
    if custom_prompt.is_empty() {
        "You are a helpful assistant. Answer concisely.".to_string()
    } else {
        custom_prompt.to_string()
    }
}

pub fn build_direct_prompt(shell_ctx: &ShellContext, custom_prompt: &str) -> String {
    let mut prompt = format!(
        "You are a command-line expert. The user is on {os} using {shell} shell with {pm} package manager.\n\
         The user will describe what they want to do. Give the exact shell command and a brief explanation.\n\
         Respond ONLY in this format:\n\
         COMMAND: <the command>\n\
         EXPLANATION: <brief explanation of what the command does, what each flag means, help the user understand and remember>",
        os = shell_ctx.os,
        shell = shell_ctx.shell,
        pm = shell_ctx.package_manager,
    );
    if !custom_prompt.is_empty() {
        prompt.push_str("\n\nAdditional instructions from user:\n");
        prompt.push_str(custom_prompt);
    }
    prompt
}

pub fn build_search_prompt(shell_ctx: &ShellContext, custom_prompt: &str) -> String {
    let mut prompt = format!(
        "You are a command-line expert. The user is on {os} using {shell} shell with {pm} package manager.\n\
         The user will describe what they want to do. Think carefully and verify your answer. \
         Search for the most accurate and up-to-date command. Consider edge cases and common pitfalls.\n\
         Respond ONLY in this format:\n\
         COMMAND: <the command>\n\
         EXPLANATION: <brief explanation of what the command does, what each flag means, help the user understand and remember>",
        os = shell_ctx.os,
        shell = shell_ctx.shell,
        pm = shell_ctx.package_manager,
    );
    if !custom_prompt.is_empty() {
        prompt.push_str("\n\nAdditional instructions from user:\n");
        prompt.push_str(custom_prompt);
    }
    prompt
}

pub fn parse_response(response: &str) -> (String, String) {
    let mut command = String::new();
    let mut explanation = String::new();

    let text = response.trim();
    let lower = text.to_lowercase();

    let cmd_marker = lower.find("command:");
    let exp_marker = lower.find("explanation:");

    match (cmd_marker, exp_marker) {
        (Some(cmd_pos), Some(exp_pos)) => {
            let cmd_content_start = cmd_pos + "command:".len();
            if cmd_pos < exp_pos {
                // COMMAND: ... EXPLANATION: ...
                command = text[cmd_content_start..exp_pos].trim().to_string();
            } else {
                // EXPLANATION came first (unusual), just take command to end
                command = text[cmd_content_start..].trim().to_string();
            }
            let exp_content_start = exp_pos + "explanation:".len();
            if exp_pos < cmd_pos {
                explanation = text[exp_content_start..cmd_pos].trim().to_string();
            } else {
                explanation = text[exp_content_start..].trim().to_string();
            }
        }
        (Some(cmd_pos), None) => {
            let cmd_content_start = cmd_pos + "command:".len();
            command = text[cmd_content_start..].trim().to_string();
        }
        (None, Some(exp_pos)) => {
            let exp_content_start = exp_pos + "explanation:".len();
            // Try to extract command from text before EXPLANATION:
            let before = text[..exp_pos].trim();
            if !before.is_empty() {
                command = extract_first_code_line(before);
            }
            explanation = text[exp_content_start..].trim().to_string();
        }
        (None, None) => {
            // No markers found - treat whole thing as explanation, try to find a command
            command = extract_first_code_line(text);
            explanation = text.to_string();
        }
    }

    // Clean up: remove backticks wrapping
    command = command.trim().trim_matches('`').trim().to_string();

    (command, explanation)
}

/// Try to extract the first code-like line from text
fn extract_first_code_line(text: &str) -> String {
    // Look for lines inside ``` blocks
    let lines: Vec<&str> = text.lines().collect();
    let mut in_code_block = false;
    for line in &lines {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            if in_code_block {
                break;
            }
            in_code_block = true;
            continue;
        }
        if in_code_block && !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    // Fall back: find first line that looks like a command
    for line in &lines {
        let trimmed = line.trim();
        if trimmed.starts_with('$') {
            return trimmed[1..].trim().to_string();
        }
        if !trimmed.is_empty()
            && !trimmed.starts_with('#')
            && !trimmed.starts_with("//")
            && trimmed.len() > 2
        {
            return trimmed.to_string();
        }
    }

    text.lines().next().unwrap_or("").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_standard_response() {
        let resp = "COMMAND: docker ps -a\nEXPLANATION: Lists all containers including stopped ones.";
        let (cmd, exp) = parse_response(resp);
        assert_eq!(cmd, "docker ps -a");
        assert!(exp.contains("Lists all containers"));
    }

    #[test]
    fn test_parse_with_backticks() {
        let resp = "COMMAND: `docker ps -a`\nEXPLANATION: Lists all docker containers.";
        let (cmd, _) = parse_response(resp);
        assert_eq!(cmd, "docker ps -a");
    }
}
