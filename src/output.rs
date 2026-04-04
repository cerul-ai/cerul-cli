use anyhow::{Context, Result};
use serde::Serialize;

use crate::types::{SearchResponse, UsageResponse};

pub fn print_search_human(response: &SearchResponse) {
    println!(
        "\nFound {} results ({} credit{} used, {} remaining):\n",
        response.results.len(),
        response.credits_used,
        if response.credits_used == 1 { "" } else { "s" },
        format_number(response.credits_remaining),
    );

    for (index, result) in response.results.iter().enumerate() {
        println!(
            "[{}] Score: {:.2}  |  {}  |  {} -> {}",
            index + 1,
            result.score,
            result.title,
            format_timestamp(result.timestamp_start),
            format_timestamp(result.timestamp_end),
        );

        let preview = truncate_preview(
            result
                .transcript
                .as_deref()
                .unwrap_or(result.snippet.as_str()),
            200,
        );
        println!("    \"{}\"", preview);
        println!("    URL: {}\n", result.url);
    }

    if let Some(answer) = &response.answer {
        println!("Answer: {}\n", answer.trim());
    }
}

pub fn print_usage_human(response: &UsageResponse) {
    println!("\nCerul API Usage");
    println!("  Tier:              {}", response.tier);
    println!("  Plan code:         {}", plan_code_label(response));
    println!(
        "  Credits used:      {} / {}",
        format_number(response.credits_used),
        format_number(response.credits_limit),
    );
    println!(
        "  Credits remaining: {}",
        format_number(response.credits_remaining)
    );
    println!(
        "  Wallet balance:    {}",
        format_number(response.wallet_balance)
    );
    println!(
        "  Daily free:        {} / {}",
        format_number(response.daily_free_remaining),
        format_number(response.daily_free_limit)
    );
    println!(
        "  Rate limit:        {} req/sec",
        format_number(response.rate_limit_per_sec)
    );
    println!(
        "  Billing period:    {} -> {}",
        response.period_start, response.period_end
    );
    println!(
        "  Billing hold:      {}",
        if response.billing_hold { "yes" } else { "no" }
    );
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
        return "-".to_string();
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

fn plan_code_label(response: &UsageResponse) -> &'static str {
    match response.plan_code {
        crate::types::PlanCode::Free => "free",
        crate::types::PlanCode::Pro => "pro",
        crate::types::PlanCode::Enterprise => "enterprise",
    }
}
