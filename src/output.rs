use std::io::IsTerminal;

use anyhow::{Context, Result};
use colored::Colorize;
use serde::Serialize;

use crate::types::{SearchResponse, UsageResponse};

pub fn print_search_human(response: &SearchResponse, show_images: bool) {
    let credit_note = if response.credits_used == 0 {
        "free daily search".green().to_string()
    } else {
        format!(
            "{} credit{} used",
            response.credits_used,
            if response.credits_used == 1 { "" } else { "s" }
        )
    };

    eprintln!();
    eprintln!(
        "  🔍 {} result{}  ·  {}  ·  {} credits remaining",
        response.results.len(),
        if response.results.len() == 1 { "" } else { "s" },
        credit_note,
        format_number(response.credits_remaining),
    );
    eprintln!();

    for (index, result) in response.results.iter().enumerate() {
        // Top border + title
        eprintln!(
            "  ┌─ {}  {}",
            format!("[{}]", index + 1).dimmed(),
            result.title.bold(),
        );

        // Metadata line
        let mut meta = Vec::new();
        meta.push(format!(
            "📊 {}",
            format!("{}% match", (result.score * 100.0) as u32).green()
        ));
        meta.push(format!(
            "🕐 {} → {}",
            format_timestamp(result.timestamp_start),
            format_timestamp(result.timestamp_end),
        ));
        if let Some(speaker) = &result.speaker {
            meta.push(format!("🎤 {speaker}"));
        }
        eprintln!("  │  {}", meta.join("  ·  ").dimmed());

        // Inline image (iTerm2 / Kitty protocol)
        if show_images {
            if let Some(url) = result.keyframe_url.as_deref().or(result.thumbnail_url.as_deref()) {
                eprintln!("  │");
                if let Some(img_data) = fetch_image_bytes(url) {
                    print_inline_image(&img_data);
                }
            }
        }

        // Transcript / snippet
        let text = result
            .transcript
            .as_deref()
            .unwrap_or(result.snippet.as_str());
        let preview = truncate_preview(text, 280);
        eprintln!("  │");
        for line in wrap_text(&preview, 64) {
            eprintln!("  │  {line}");
        }

        // URL (OSC 8 clickable link with fallback)
        eprintln!("  │");
        eprintln!("  │  🔗 {}", osc8_link(&result.url, "Open video").dimmed());

        // Bottom border
        eprintln!("  └─");
        eprintln!();
    }

    if let Some(answer) = &response.answer {
        eprintln!("  {} {}", "💡".dimmed(), "Answer".bold());
        eprintln!();
        for line in wrap_text(answer.trim(), 68) {
            eprintln!("  {line}");
        }
        eprintln!();
    }
}

pub fn print_usage_human(response: &UsageResponse) {
    eprintln!();
    eprintln!("  {}", "📊 Cerul Usage".bold());
    eprintln!();
    eprintln!(
        "  {:<12}{}",
        "Plan".dimmed(),
        response.tier.bold()
    );
    eprintln!(
        "  {:<12}{} used · {} remaining",
        "Credits".dimmed(),
        format_number(response.credits_used),
        format_number(response.credits_remaining).green(),
    );
    eprintln!(
        "  {:<12}{}",
        "Wallet".dimmed(),
        format_number(response.wallet_balance),
    );
    eprintln!(
        "  {:<12}{} / {}",
        "Daily free".dimmed(),
        format_number(response.daily_free_remaining).green(),
        format_number(response.daily_free_limit),
    );
    eprintln!(
        "  {:<12}{} req/sec",
        "Rate limit".dimmed(),
        format_number(response.rate_limit_per_sec),
    );
    eprintln!(
        "  {:<12}{} → {}",
        "Period".dimmed(),
        response.period_start,
        response.period_end,
    );
    if response.billing_hold {
        eprintln!(
            "  {:<12}{}",
            "Hold".dimmed(),
            "YES (account under review)".red().bold(),
        );
    }
    eprintln!();
}

pub fn print_json<T>(value: &T) -> Result<()>
where
    T: Serialize,
{
    let json = serde_json::to_string_pretty(value).context("Failed to serialize JSON output")?;
    println!("{json}");
    Ok(())
}

fn format_timestamp(timestamp: Option<f64>) -> String {
    let Some(timestamp) = timestamp else {
        return "—".to_string();
    };

    let total_seconds = timestamp.floor() as u64;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

fn truncate_preview(text: &str, limit: usize) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let char_count = collapsed.chars().count();
    if char_count <= limit {
        return collapsed;
    }

    let truncated: String = collapsed.chars().take(limit.saturating_sub(3)).collect();
    format!("{truncated}...")
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        if current.is_empty() {
            current = word.to_string();
        } else if current.len() + 1 + word.len() > width {
            lines.push(current);
            current = word.to_string();
        } else {
            current.push(' ');
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }

    lines
}

fn format_number(value: u64) -> String {
    let digits = value.to_string();
    let mut formatted = String::with_capacity(digits.len() + digits.len() / 3);

    for (index, ch) in digits.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            formatted.push(',');
        }
        formatted.push(ch);
    }

    formatted.chars().rev().collect()
}

/// OSC 8 hyperlink: makes text clickable in supported terminals.
/// Falls back to showing the URL as plain text in unsupported terminals.
fn osc8_link(url: &str, label: &str) -> String {
    if is_terminal_interactive() {
        // OSC 8 format: \x1b]8;;URL\x1b\\LABEL\x1b]8;;\x1b\\
        format!("\x1b]8;;{url}\x1b\\{label}\x1b]8;;\x1b\\  {url}")
    } else {
        url.to_string()
    }
}

fn is_terminal_interactive() -> bool {
    std::io::stderr().is_terminal()
}

/// Download image bytes. Returns None on any failure (non-blocking to UX).
fn fetch_image_bytes(url: &str) -> Option<Vec<u8>> {
    // Use a blocking reqwest call since we're already in an output function.
    // Short timeout to avoid slowing down results.
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .ok()?;
    let resp = client.get(url).send().ok()?;
    if !resp.status().is_success() {
        return None;
    }
    resp.bytes().ok().map(|b| b.to_vec())
}

/// Print an inline image using the iTerm2 Inline Images Protocol.
/// Falls back to nothing on unsupported terminals (the escape is simply ignored).
fn print_inline_image(data: &[u8]) {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let encoded = STANDARD.encode(data);
    // iTerm2 protocol: \x1b]1337;File=inline=1;width=40;preserveAspectRatio=1:<base64>\x07
    eprint!(
        "  │  \x1b]1337;File=inline=1;width=40;preserveAspectRatio=1:{}\x07",
        encoded
    );
    eprintln!();
}
