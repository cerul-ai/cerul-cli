mod client;
mod commands;
mod config;
mod error;
mod output;
mod types;

use anyhow::Result;
use clap::{Parser, Subcommand};
use types::RankingMode;

use crate::client::CerulClient;

#[derive(Parser, Debug)]
#[command(
    name = "cerul",
    version,
    about = "Search video knowledge from your terminal",
    long_about = "Cerul CLI — search what was said, shown, or presented in tech talks,\n\
                  podcasts, conference presentations, and earnings calls.\n\n\
                  Get started:\n\
                  \x20 cerul login                           Authenticate with your API key\n\
                  \x20 cerul search \"sam altman agi\"          Search indexed videos\n\
                  \x20 cerul usage                            Check your credit balance",
    after_help = "Examples:\n\
                  \x20 cerul search \"attention mechanism explained\"\n\
                  \x20 cerul search \"jensen huang\" --speaker \"Jensen Huang\" --max-results 10\n\
                  \x20 cerul search \"AI safety\" --ranking-mode rerank --include-answer\n\
                  \x20 cerul search \"scaling laws\" --published-after 2025-01-01 --json\n\n\
                  Documentation: https://cerul.ai/docs\n\
                  Dashboard:     https://cerul.ai/dashboard"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Authenticate with your Cerul API key
    Login(LoginArgs),
    /// Remove saved API key
    Logout,
    /// Search indexed videos for speech, visual content, and on-screen text
    Search(SearchArgs),
    /// Check credit balance, billing period, and rate limits
    Usage(UsageArgs),
}

#[derive(clap::Args, Debug, Clone)]
pub struct LoginArgs {
    /// API key to save (skip interactive prompt)
    #[arg(long)]
    pub api_key: Option<String>,
}

#[derive(clap::Args, Debug, Clone)]
pub struct SearchArgs {
    /// Natural language search query (max 400 chars)
    pub query: String,
    /// Maximum number of results (1-50)
    #[arg(long, default_value_t = 5, value_parser = clap::value_parser!(u32).range(1..=50))]
    pub max_results: u32,
    /// Ranking mode: embedding or rerank
    #[arg(long, value_enum, default_value_t = RankingMode::Embedding)]
    pub ranking_mode: RankingMode,
    /// Include AI-generated summary answer (costs 2 credits)
    #[arg(long)]
    pub include_answer: bool,
    /// Filter by speaker name
    #[arg(long)]
    pub speaker: Option<String>,
    /// Filter videos published after date (YYYY-MM-DD)
    #[arg(long)]
    pub published_after: Option<String>,
    /// Filter by minimum duration in seconds
    #[arg(long)]
    pub min_duration: Option<u64>,
    /// Filter by maximum duration in seconds
    #[arg(long)]
    pub max_duration: Option<u64>,
    /// Filter by source (e.g. youtube)
    #[arg(long)]
    pub source: Option<String>,
    /// Output raw JSON instead of human-readable format
    #[arg(long)]
    pub json: bool,
}

#[derive(clap::Args, Debug, Clone)]
pub struct UsageArgs {
    /// Output raw JSON instead of human-readable format
    #[arg(long)]
    pub json: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(err) = run().await {
        error::exit_with_error(&err);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Login(args) => commands::login::run(args.api_key).await?,
        Commands::Logout => commands::login::run_logout()?,
        Commands::Search(args) => {
            let client = CerulClient::from_config()?;
            commands::search::run(&client, args).await?;
        }
        Commands::Usage(args) => {
            let client = CerulClient::from_config()?;
            commands::usage::run(&client, args).await?;
        }
    }

    Ok(())
}
