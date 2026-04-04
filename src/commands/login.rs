use anyhow::{bail, Result};

use crate::client::CerulClient;
use crate::config;

pub async fn run(api_key: Option<String>) -> Result<()> {
    // If key provided via flag, use it directly
    let key = if let Some(k) = api_key {
        k
    } else {
        // Interactive flow
        eprintln!("Log in to Cerul\n");
        eprintln!("  1. Go to https://cerul.ai/dashboard/api-keys");
        eprintln!("  2. Create or copy an API key\n");

        // Try to open browser
        open_browser("https://cerul.ai/dashboard/api-keys");

        config::read_line("Paste your API key: ")?
    };

    if key.is_empty() {
        bail!("No API key provided.");
    }

    if !key.starts_with("cerul_") {
        bail!("Invalid API key format. Keys start with \"cerul_\".");
    }

    // Verify key works
    eprint!("Verifying...");
    let client = CerulClient::with_key(key.clone())?;
    let usage = client.usage().await?;
    eprintln!(" OK\n");

    // Save
    config::save_api_key(&key)?;

    eprintln!("Logged in successfully.");
    eprintln!("  Tier:              {}", usage.tier);
    eprintln!(
        "  Credits remaining: {}",
        usage.credits_remaining
    );
    eprintln!(
        "  Daily free:        {} / {}",
        usage.daily_free_remaining, usage.daily_free_limit
    );
    eprintln!("\nAPI key saved. You can now use `cerul search`.");

    Ok(())
}

pub fn run_logout() -> Result<()> {
    if config::clear_api_key()? {
        eprintln!("Logged out. API key removed.");
    } else {
        eprintln!("No saved API key found.");
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
