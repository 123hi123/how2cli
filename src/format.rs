use termimad::MadSkin;
use colored::Colorize;

/// Create a configured MadSkin for terminal markdown rendering
fn make_skin() -> MadSkin {
    let mut skin = MadSkin::default();
    // Simplify inline code style: just bold, no background color
    skin.inline_code.set_fg(termimad::crossterm::style::Color::Yellow);
    skin.inline_code.set_bg(termimad::crossterm::style::Color::Reset);
    skin
}

/// Format command mode output: green command + markdown explanation
pub fn format_result(command: &str, explanation: &str, show_explanation: bool) {
    if !command.is_empty() {
        println!("\n  {}", command.green().bold());
    }
    if show_explanation && !explanation.is_empty() {
        println!("  {}", "─".repeat(40).dimmed());
        let skin = make_skin();
        // Render markdown explanation with 2-space indent
        let rendered = skin.term_text(explanation);
        for line in rendered.to_string().lines() {
            println!("  {}", line);
        }
    }
    println!();
}

/// Raw output for piping
pub fn format_raw(command: &str) {
    println!("{}", command);
}

/// Talk mode: render full response as markdown
pub fn format_talk(response: &str) {
    println!();
    let skin = make_skin();
    let rendered = skin.term_text(response);
    for line in rendered.to_string().lines() {
        println!("  {}", line);
    }
    println!();
}

/// Show token usage stats
pub fn format_token_usage(input_tokens: u64, output_tokens: u64) {
    println!("  {} input: {} | output: {}", "tokens".dimmed(), input_tokens.to_string().dimmed(), output_tokens.to_string().dimmed());
}
