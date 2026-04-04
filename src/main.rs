mod client;
mod commands;
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
    about = "Search video knowledge from your terminal"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Search indexed videos for speech, visual content, and on-screen text
    Search(SearchArgs),
    /// Check credit balance, billing period, and rate limits
    Usage(UsageArgs),
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
    let client = CerulClient::from_env()?;

    match cli.command {
        Commands::Search(args) => commands::search::run(&client, args).await?,
        Commands::Usage(args) => commands::usage::run(&client, args).await?,
    }

    Ok(())
}
