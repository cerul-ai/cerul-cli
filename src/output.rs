use anyhow::{Context, Result};
use serde::Serialize;

use crate::types::{SearchResponse, UsageResponse};

pub fn print_search_human(response: &SearchResponse) {
    let credit_note = if response.credits_used == 0 {
        "(free daily search)".to_string()
    } else {
        format!(
            "({} credit{} used)",
            response.credits_used,
            if response.credits_used == 1 { "" } else { "s" }
        )
    };

    println!(
        "\n  {} result{} found  {}  |  {} credits remaining\n",
        response.results.len(),
        if response.results.len() == 1 { "" } else { "s" },
        credit_note,
        format_number(response.credits_remaining),
    );

    println!("{}", "─".repeat(72));

    for (index, result) in response.results.iter().enumerate() {
        if index > 0 {
            println!("{}", "─".repeat(72));
        }

        // Title line
        println!(
            "  [{}/{}]  {}",
            index + 1,
            response.results.len(),
            result.title,
        );

        // Metadata line
        let mut meta_parts = Vec::new();
        meta_parts.push(format!("Score: {:.0}%", result.score * 100.0));
        if let Some(rs) = result.rerank_score {
            meta_parts.push(format!("Rerank: {:.0}%", rs * 100.0));
        }
        meta_parts.push(format!(
            "Time: {} - {}",
            format_timestamp(result.timestamp_start),
            format_timestamp(result.timestamp_end),
        ));
        if let Some(speaker) = &result.speaker {
            meta_parts.push(format!("Speaker: {speaker}"));
        }
        println!("        {}", meta_parts.join("  |  "));

        // Transcript / snippet
        let text = result
            .transcript
            .as_deref()
            .unwrap_or(result.snippet.as_str());
        let preview = truncate_preview(text, 280);
        println!();
        for line in wrap_text(&preview, 68) {
            println!("        {line}");
        }

        // URL
        println!();
        println!("        {}", result.url);
        println!();
    }

    println!("{}", "─".repeat(72));

    if let Some(answer) = &response.answer {
        println!();
        println!("  Answer:");
        for line in wrap_text(answer.trim(), 68) {
            println!("    {line}");
        }
        println!();
    }
}

pub fn print_usage_human(response: &UsageResponse) {
    println!();
    println!("  Cerul API Usage");
    println!("  {}", "─".repeat(40));
    println!(
        "  Tier:              {}",
        response.tier
    );
    println!(
        "  Credits:           {} used / {} remaining",
        format_number(response.credits_used),
        format_number(response.credits_remaining),
    );
    println!(
        "  Wallet balance:    {}",
        format_number(response.wallet_balance)
    );
    println!(
        "  Daily free:        {} / {} remaining",
        format_number(response.daily_free_remaining),
        format_number(response.daily_free_limit)
    );
    println!(
        "  Rate limit:        {} req/sec",
        format_number(response.rate_limit_per_sec)
    );
    println!(
        "  Billing period:    {} to {}",
        response.period_start, response.period_end
    );
    if response.billing_hold {
        println!("  Billing hold:      YES (account under review)");
    }
    println!();
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
