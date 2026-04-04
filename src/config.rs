use std::{env, fs, io, path::PathBuf};

use anyhow::{Context, Result};

const CONFIG_DIR: &str = "cerul";
const CREDENTIALS_FILE: &str = "credentials";

/// Resolve API key: config file → env var → None
pub fn resolve_api_key() -> Option<String> {
    // 1. Config file
    if let Some(key) = read_saved_key() {
        return Some(key);
    }
    // 2. Environment variable
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

    // Restrict permissions on Unix
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
    if key.is_empty() { None } else { Some(key) }
}

fn config_dir() -> Result<PathBuf> {
    // XDG on Linux, ~/Library/Application Support on macOS, AppData on Windows
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
