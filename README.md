<div align="center">
  <h1>cerul</h1>
  <p><strong>The video search layer for AI agents — CLI.</strong></p>
  <p>Search what was said, shown, or presented in any video. From your terminal.</p>

  <p>
    <a href="https://cerul.ai/docs"><strong>Docs</strong></a> &middot;
    <a href="https://cerul.ai"><strong>Website</strong></a> &middot;
    <a href="https://github.com/cerul-ai/cerul"><strong>Main Repo</strong></a>
  </p>

  <p>
    <a href="https://github.com/cerul-ai/cerul-cli/releases/latest"><img alt="Release" src="https://img.shields.io/github/v/release/cerul-ai/cerul-cli?style=flat-square&color=3b82f6" /></a>
    <a href="./LICENSE"><img alt="License" src="https://img.shields.io/badge/license-MIT-3b82f6?style=flat-square" /></a>
    <img alt="Platforms" src="https://img.shields.io/badge/platforms-macOS%20%7C%20Linux%20%7C%20Windows-22c55e?style=flat-square" />
  </p>
</div>

<br />

## Install

```bash
# macOS / Linux
curl -fsSL https://raw.githubusercontent.com/cerul-ai/cerul-cli/main/install.sh | bash

# Homebrew
brew tap cerul-ai/tap && brew install cerul

# Self-update
cerul upgrade
```

## Quick Start

```bash
cerul login                                   # authenticate
cerul search "Sam Altman on AGI timeline"     # search videos
cerul usage                                   # check credits
```

## What It Does

Cerul indexes tech talks, podcasts, conference presentations, and earnings calls. The CLI lets you search across all of them by meaning — speech, visuals, slides, and on-screen text.

```
$ cerul search "transformer attention explained"

  🔍 5 results  ·  free daily search  ·  7,550 credits remaining

  ┌─ [1]  Attention Is All You Need — Paper Explained
  │  📊 92% match  ·  🕐 14:32 → 16:45  ·  🎤 Yannic Kilcher
  │
  │  "So the key insight of attention is that instead of compressing
  │   the entire input into a fixed-size vector, we allow the decoder
  │   to look back at all encoder hidden states..."
  │
  │  🔗 https://cerul.ai/v/a8f3k2x
  └─
```

## Commands

| Command | Description |
|---------|-------------|
| `cerul search <query>` | Search indexed videos |
| `cerul usage` | Check credit balance and rate limits |
| `cerul login` | Authenticate with your API key |
| `cerul logout` | Remove saved API key |
| `cerul config` | Configure defaults (arrow keys to navigate) |
| `cerul history` | View recent searches |
| `cerul upgrade` | Update to latest version |
| `cerul completions <shell>` | Generate shell completions (bash/zsh/fish) |

## Search Options

| Flag | Description |
|------|-------------|
| `--max-results N` | Number of results (1-50, default 5) |
| `--ranking-mode MODE` | `embedding` (default) or `rerank` |
| `--include-answer` | Include AI summary (2 credits) |
| `--speaker NAME` | Filter by speaker |
| `--published-after DATE` | Filter by date (YYYY-MM-DD) |
| `--source SOURCE` | Filter by source (e.g. youtube) |
| `--json` | Output raw JSON for scripts and agents |

## Configuration

`cerul config` opens an interactive settings editor:

```
  ⚙️  Cerul Configuration
  ↑↓ navigate  ←→ change  Enter save  Esc quit

> Images in results      ◀ off ▶
  Default max results    5
  Default ranking mode   embedding
  Include AI answer      off

    Save and exit
```

Settings persist to `~/.config/cerul/config.toml`. CLI flags always override config.

## Shell Completions

```bash
# zsh
cerul completions zsh > ~/.zfunc/_cerul

# bash
cerul completions bash > /etc/bash_completion.d/cerul

# fish
cerul completions fish > ~/.config/fish/completions/cerul.fish
```

## Ecosystem

| Package | Description |
|---------|-------------|
| [`cerul`](https://github.com/cerul-ai/cerul) | Main repo — API, docs, skills, remote MCP |
| [`cerul`](https://www.npmjs.com/package/cerul) | TypeScript SDK |
| [`cerul`](https://pypi.org/project/cerul/) | Python SDK |

## License

[MIT](./LICENSE)
