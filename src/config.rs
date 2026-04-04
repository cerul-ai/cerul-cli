use std::{env, fs, io, path::PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const CONFIG_DIR: &str = "cerul";
const CREDENTIALS_FILE: &str = "credentials";
const CONFIG_FILE: &str = "config.toml";

// ── API Key ──────────────────────────────────────────────────────────

/// Resolve API key: config file → env var → None
pub fn resolve_api_key() -> Option<String> {
    if let Some(key) = read_saved_key() {
        return Some(key);
    }
    if let Ok(key) = env::var("CERUL_API_KEY") {
        if !key.is_empty() {
            return Some(key);
        }
    }
    None
}

pub fn require_api_key() -> Result<String> {
    resolve_api_key().ok_or_else(|| {
        anyhow::anyhow!(
            "Not logged in.\n\n\
             Run `cerul login` to authenticate, or set CERUL_API_KEY.\n\
             Get your API key at https://cerul.ai/dashboard"
        )
    })
}

pub fn save_api_key(key: &str) -> Result<()> {
    let dir = config_dir()?;
    fs::create_dir_all(&dir)
        .with_context(|| format!("Failed to create config directory: {}", dir.display()))?;
    let path = dir.join(CREDENTIALS_FILE);
    fs::write(&path, key).with_context(|| format!("Failed to write {}", path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).ok();
    }

    Ok(())
}

pub fn clear_api_key() -> Result<bool> {
    let path = config_dir()?.join(CREDENTIALS_FILE);
    if path.exists() {
        fs::remove_file(&path)
            .with_context(|| format!("Failed to remove {}", path.display()))?;
        return Ok(true);
    }
    Ok(false)
}

fn read_saved_key() -> Option<String> {
    let path = config_dir().ok()?.join(CREDENTIALS_FILE);
    let key = fs::read_to_string(path).ok()?.trim().to_string();
    if key.is_empty() {
        None
    } else {
        Some(key)
    }
}

// ── User Preferences ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    #[serde(default = "default_false")]
    pub images: bool,
    #[serde(default = "default_max_results")]
    pub max_results: u32,
    #[serde(default = "default_ranking_mode")]
    pub ranking_mode: String,
    #[serde(default = "default_false")]
    pub include_answer: bool,
}

fn default_false() -> bool {
    false
}
fn default_max_results() -> u32 {
    5
}
fn default_ranking_mode() -> String {
    "embedding".to_string()
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            images: false,
            max_results: 5,
            ranking_mode: "embedding".to_string(),
            include_answer: false,
        }
    }
}

pub fn load_config() -> UserConfig {
    let path = match config_dir() {
        Ok(dir) => dir.join(CONFIG_FILE),
        Err(_) => return UserConfig::default(),
    };
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return UserConfig::default(),
    };
    toml::from_str(&content).unwrap_or_default()
}

pub fn save_config(config: &UserConfig) -> Result<()> {
    let dir = config_dir()?;
    fs::create_dir_all(&dir)?;
    let path = dir.join(CONFIG_FILE);
    let content = toml::to_string_pretty(config).context("Failed to serialize config")?;
    fs::write(&path, content).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

pub fn config_file_path() -> Result<PathBuf> {
    Ok(config_dir()?.join(CONFIG_FILE))
}

// ── Shared ──────────────────────────────────────────────────────────

fn config_dir() -> Result<PathBuf> {
    let base = if cfg!(target_os = "macos") {
        env::var("HOME")
            .map(|h| PathBuf::from(h).join(".config"))
            .context("HOME not set")?
    } else if cfg!(target_os = "windows") {
        env::var("APPDATA")
            .map(PathBuf::from)
            .context("APPDATA not set")?
    } else {
        env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                env::var("HOME")
                    .map(|h| PathBuf::from(h).join(".config"))
                    .unwrap_or_else(|_| PathBuf::from(".config"))
            })
    };
    Ok(base.join(CONFIG_DIR))
}

/// Read a line from stdin (for interactive prompts)
pub fn read_line(prompt: &str) -> Result<String> {
    eprint!("{prompt}");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("Failed to read input")?;
    Ok(input.trim().to_string())
}
