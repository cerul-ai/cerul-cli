# cerul

Search video knowledge from your terminal. Find what was said, shown, or presented in tech talks, podcasts, and conference presentations.

## Status

This repository currently implements Phase 1 of the Cerul CLI: the core Rust CLI commands only.

Not included yet:

- release CI
- `install.sh`
- Homebrew distribution

## Requirements

- Rust toolchain
- `CERUL_API_KEY`

## Build

```bash
cargo build --release
```

The binary will be available at `./target/release/cerul`.

## Setup

```bash
export CERUL_API_KEY=cerul_sk_...
```

Get your API key at [cerul.ai/dashboard](https://cerul.ai/dashboard).

## Usage

```bash
# Search videos
./target/release/cerul search "sam altman agi timeline"
./target/release/cerul search "transformer attention" --max-results 10 --ranking-mode rerank --json

# Check credits
./target/release/cerul usage
./target/release/cerul usage --json
```

## Options

| Flag | Description |
|------|-------------|
| `--max-results N` | Number of results (1-50, default 5) |
| `--ranking-mode MODE` | `embedding` (default) or `rerank` |
| `--include-answer` | Include AI summary (2 credits) |
| `--speaker NAME` | Filter by speaker |
| `--published-after DATE` | Filter by date (`YYYY-MM-DD`) |
| `--min-duration N` | Filter by minimum duration in seconds |
| `--max-duration N` | Filter by maximum duration in seconds |
| `--source SOURCE` | Filter by source (for example `youtube`) |
| `--json` | Output raw JSON |
