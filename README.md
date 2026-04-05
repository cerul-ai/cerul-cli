<div align="center">
  <br />
  <a href="https://cerul.ai">
    <img src="https://raw.githubusercontent.com/cerul-ai/cerul/main/assets/logo.png" alt="Cerul" width="80" />
  </a>
  <h1>Cerul CLI</h1>
  <p><strong>The video search layer for AI agents.</strong></p>
  <p>Teach your AI agents to see — search by meaning across visual scenes, speech, and on-screen content.</p>

  <p>
    <a href="https://cerul.ai/docs"><strong>Docs</strong></a> &middot;
    <a href="https://cerul.ai"><strong>Website</strong></a> &middot;
    <a href="https://github.com/cerul-ai/cerul"><strong>Main Repo</strong></a> &middot;
    <a href="https://x.com/cerul_hq"><img src="https://img.shields.io/badge/follow-%40cerul__hq-000?style=flat-square&logo=x" alt="Follow on X" /></a>
  </p>

  <p>
    <a href="https://github.com/cerul-ai/cerul-cli/releases/latest"><img alt="Release" src="https://img.shields.io/github/v/release/cerul-ai/cerul-cli?style=flat-square&color=3b82f6" /></a>
    <a href="./LICENSE"><img alt="License" src="https://img.shields.io/badge/license-MIT-3b82f6?style=flat-square" /></a>
    <img alt="Platforms" src="https://img.shields.io/badge/platforms-macOS%20%7C%20Linux%20%7C%20Windows-22c55e?style=flat-square" />
  </p>
</div>

<br />

<div align="center">
  <img src="./cli.png" alt="cerul search results with inline video frames" width="720" />
</div>

<br />

## Install

```bash
curl -fsSL https://cli.cerul.ai/install.sh | bash
```

## Quick Start

```bash
cerul login                                 # authenticate (opens browser)
cerul search "Sam Altman on AGI timeline"   # search videos
cerul usage                                 # check credits
```

Get a free API key at [cerul.ai/dashboard](https://cerul.ai/dashboard).

## Why a CLI?

AI coding agents (Claude Code, Codex, Cursor, Cline) can run shell commands. Give them access to `cerul search` and they can find evidence from video — who said what, when, in which talk.

```bash
# An agent can run this directly
cerul search "Jensen Huang on AI infrastructure" --json

# Or as part of a multi-step research workflow
cerul search "scaling laws explained" --speaker "Ilya Sutskever" --json
```

Use `--json` for structured output that agents can parse. Without `--json`, results are formatted for humans with inline video frames, clickable links, and color.

> Inline video frame previews are supported in iTerm2, WezTerm, and Kitty. Enable with `cerul config` and toggle **Images** on.

## Search Options

```bash
cerul search "query"                          # basic search
cerul search "query" --max-results 10         # more results (1-50)
cerul search "query" --ranking-mode rerank    # LLM reranking
cerul search "query" --include-answer         # AI summary (2 credits)
cerul search "query" --speaker "Sam Altman"   # filter by speaker
cerul search "query" --published-after 2025-01-01
cerul search "query" --source youtube
cerul search "query" --json                   # raw JSON for scripts/agents
```

## All Commands

| Command | Description |
|---------|-------------|
| `cerul search <query>` | Search indexed videos |
| `cerul usage` | Check credits and rate limits |
| `cerul login` / `logout` | Authenticate |
| `cerul config` | Interactive settings |
| `cerul history` | Browse and re-run past searches |
| `cerul upgrade` | Self-update to latest version |
| `cerul completions <shell>` | Shell completions (bash/zsh/fish) |

## Agent Integration

Install the Cerul skill so your agent can search videos automatically:

```bash
npx skills add cerul-ai/cerul
```

Or point your agent directly at the skill file:

> Install the Cerul video search skill by reading and following https://github.com/cerul-ai/cerul/blob/main/skills/cerul/SKILL.md

## Links

- [Python SDK](https://pypi.org/project/cerul/) — `pip install cerul`
- [TypeScript SDK](https://www.npmjs.com/package/cerul) — `npm install cerul`
- [Main repo](https://github.com/cerul-ai/cerul) — docs, skills, remote MCP

## License

[MIT](./LICENSE)
