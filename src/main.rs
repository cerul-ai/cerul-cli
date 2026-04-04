mod client;
mod commands;
mod config;
mod error;
mod output;
mod types;

use anyhow::Result;
use clap::{Parser, Subcommand};
use clap_complete::Shell;
use colored::Colorize;
use types::RankingMode;

use crate::client::CerulClient;

#[derive(Parser, Debug)]
#[command(
    name = "cerul",
    version,
    disable_version_flag = true,
    about = "The video search layer for AI agents",
    long_about = "Cerul CLI — teach your AI agents to see.\n\
                  Search what was said, shown, or presented in tech talks,\n\
                  podcasts, conference presentations, and earnings calls.\n\n\
                  Get started:\n\
                  \x20 cerul login                           Authenticate with your API key\n\
                  \x20 cerul search \"sam altman agi\"          Search indexed videos\n\
                  \x20 cerul usage                            Check your credit balance",
    after_help = "Examples:\n\
                  \x20 cerul search \"how does attention mechanism work\"\n\
                  \x20 cerul search \"Jensen Huang\" --speaker \"Jensen Huang\" --max-results 10\n\
                  \x20 cerul search \"Dario Amodei AI safety\" --ranking-mode rerank --include-answer\n\
                  \x20 cerul search \"scaling laws\" --published-after 2025-01-01 --json\n\
                  \x20 cerul search \"what is RLHF\" --include-answer\n\
                  \x20 cerul search \"Mark Zuckerberg open source\" --source youtube\n\n\
                  Documentation: https://cerul.ai/docs\n\
                  Dashboard:     https://cerul.ai/dashboard"
)]
pub struct Cli {
    /// Print version
    #[arg(short = 'v', short_alias = 'V', long = "version", global = true)]
    version: bool,

    #[command(subcommand)]
    command: Option<Commands>,
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
    /// Configure default settings (images, max_results, ranking_mode, etc.)
    Config(ConfigArgs),
    /// View recent search history
    History(HistoryArgs),
    /// Generate shell completions (bash, zsh, fish, powershell)
    Completions(CompletionsArgs),
    /// Update cerul to the latest version
    Upgrade,
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
    /// Maximum number of results (1-50, default from config)
    #[arg(long)]
    pub max_results: Option<u32>,
    /// Ranking mode: embedding or rerank (default from config)
    #[arg(long, value_enum)]
    pub ranking_mode: Option<RankingMode>,
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

#[derive(clap::Args, Debug, Clone)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub action: Option<ConfigAction>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ConfigAction {
    /// Show current configuration
    List,
    /// Set a config value (e.g. cerul config set images on)
    Set {
        /// Config key
        key: String,
        /// Config value
        value: String,
    },
    /// Reset all settings to defaults
    Reset,
}

#[derive(clap::Args, Debug, Clone)]
pub struct HistoryArgs {
    /// Number of recent searches to show
    #[arg(long, default_value_t = 20)]
    pub limit: usize,
}

#[derive(clap::Args, Debug, Clone)]
pub struct CompletionsArgs {
    /// Shell to generate completions for
    pub shell: Shell,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // No args → show welcome banner instead of clap help
    if std::env::args().count() == 1 {
        print_welcome();
        return;
    }

    let cli = Cli::parse();

    if cli.version {
        println!("cerul {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    if let Err(err) = run(cli).await {
        error::exit_with_error(&err);
    }
}

const EXAMPLE_QUERIES: &[&str] = &[
    "Sam Altman on AGI timeline",
    "how does attention mechanism work",
    "Jensen Huang on AI infrastructure",
    "Dario Amodei AI safety approach",
    "scaling laws explained",
    "Mark Zuckerberg open source AI strategy",
    "what is chain of thought reasoning",
    "Andrej Karpathy on LLM training",
    "Demis Hassabis on protein folding",
    "Yann LeCun self-supervised learning",
    "how transformers replaced RNNs",
    "Satya Nadella on Copilot and AI agents",
    "Geoffrey Hinton on AI existential risk",
    "Ilya Sutskever on unsupervised learning",
    "why retrieval augmented generation works",
    "Emad Mostaque on open source generative AI",
    "how diffusion models generate images",
    "Jim Fan on foundation agents",
    "Harrison Chase on LangChain and agent frameworks",
    "what is RLHF and why it matters",
];

fn pick_example_query() -> &'static str {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    // Mix bits for better distribution: XOR high and low, add PID
    let pid = std::process::id() as u128;
    let seed = ((nanos ^ (nanos >> 17)).wrapping_add(pid)) as usize;
    EXAMPLE_QUERIES[seed % EXAMPLE_QUERIES.len()]
}

fn print_welcome() {
    let version = env!("CARGO_PKG_VERSION");
    let example = pick_example_query();
    eprintln!();
    eprintln!(
        "  {} {}",
        "🔍 cerul".bold(),
        format!("v{version}").dimmed()
    );
    eprintln!("  {}", "The video search layer for AI agents.".dimmed());
    eprintln!();
    eprintln!("  {}", "Quick start:".bold());
    eprintln!(
        "    {}                          Set up your API key",
        "cerul login".green()
    );
    eprintln!(
        "    {}",
        format!("cerul search \"{example}\"").green()
    );
    eprintln!(
        "    {}                          Check credits",
        "cerul usage".green()
    );
    eprintln!();
    eprintln!("  Run {} for all options.", "cerul --help".dimmed());
    eprintln!();
}

async fn run(cli: Cli) -> Result<()> {
    let Some(command) = cli.command else {
        print_welcome();
        return Ok(());
    };

    // Check for updates on non-upgrade commands (non-blocking, cached 24h)
    let is_upgrade = matches!(command, Commands::Upgrade);
    if !is_upgrade {
        if let Some(latest) = commands::upgrade::check_update_background().await {
            eprintln!(
                "  {}  Update available: v{} → v{latest}. Run {} to update.\n",
                "⬆️".dimmed(),
                env!("CARGO_PKG_VERSION"),
                "cerul upgrade".yellow(),
            );
        }
    }

    match command {
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
        Commands::Config(args) => match args.action {
            None => commands::config::run_interactive()?,
            Some(ConfigAction::List) => commands::config::run_list()?,
            Some(ConfigAction::Set { key, value }) => commands::config::run_set(&key, &value)?,
            Some(ConfigAction::Reset) => commands::config::run_reset()?,
        },
        Commands::History(args) => commands::history::run(args.limit)?,
        Commands::Completions(args) => commands::completions::run(args.shell)?,
        Commands::Upgrade => commands::upgrade::run().await?,
    }

    Ok(())
}
