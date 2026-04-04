use anyhow::{bail, Result};
use colored::Colorize;

use crate::client::CerulClient;
use crate::config;

pub async fn run(api_key: Option<String>) -> Result<()> {
    let key = if let Some(k) = api_key {
        k
    } else {
        eprintln!();
        eprintln!("  {}", "🔑 Cerul Login".bold());
        eprintln!();
        eprintln!("  Opening {} ...", "https://cerul.ai/dashboard".dimmed());
        eprintln!();

        open_browser("https://cerul.ai/dashboard");

        config::read_line("  Paste your API key: ")?
    };

    if key.is_empty() {
        bail!("No API key provided.");
    }

    if !key.starts_with("cerul_") {
        bail!("Invalid API key format. Keys start with \"cerul_\".");
    }

    eprint!("  Verifying... ");
    let client = CerulClient::with_key(key.clone())?;
    let usage = client.usage().await?;
    eprintln!("{}", "✓".green().bold());
    eprintln!();

    config::save_api_key(&key)?;

    eprintln!("  {}", "✅ Logged in".green().bold());
    eprintln!(
        "  {:<12}{}",
        "Plan".dimmed(),
        usage.tier.bold()
    );
    eprintln!(
        "  {:<12}{} remaining",
        "Credits".dimmed(),
        format!("{}", usage.credits_remaining).green(),
    );
    eprintln!(
        "  {:<12}{} / {}",
        "Daily free".dimmed(),
        format!("{}", usage.daily_free_remaining).green(),
        usage.daily_free_limit,
    );
    eprintln!();
    eprintln!(
        "  You're all set! Try: {}",
        "cerul search \"sam altman agi\"".green()
    );
    eprintln!();

    Ok(())
}

pub fn run_logout() -> Result<()> {
    if config::clear_api_key()? {
        eprintln!();
        eprintln!("  {} Logged out. API key removed.", "✅".green());
        eprintln!();
    } else {
        eprintln!();
        eprintln!("  No saved API key found.");
        eprintln!();
    }
    Ok(())
}

fn open_browser(url: &str) {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).spawn().ok();
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(url).spawn().ok();
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", url])
            .spawn()
            .ok();
    }
}
