use std::env;

use anyhow::{bail, Context, Result};
use colored::Colorize;

const REPO: &str = "cerul-ai/cerul-cli";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub async fn run() -> Result<()> {
    eprintln!();
    eprintln!("  {}", "⬆️  Cerul Upgrade".bold());
    eprintln!();
    eprint!("  Checking for updates... ");

    let latest = fetch_latest_version().await?;

    if latest == CURRENT_VERSION {
        eprintln!("{}", "already up to date.".green());
        eprintln!();
        return Ok(());
    }

    eprintln!();
    eprintln!(
        "  {:<12}v{}",
        "Current".dimmed(),
        CURRENT_VERSION
    );
    eprintln!(
        "  {:<12}v{}",
        "Latest".dimmed(),
        latest.green()
    );
    eprintln!();

    let artifact_name = artifact_for_current_platform()?;
    let url = format!(
        "https://github.com/{REPO}/releases/download/v{latest}/{artifact_name}"
    );

    eprint!("  Downloading {artifact_name}... ");
    let bytes = download_url(&url).await?;

    let current_exe = env::current_exe().context("Cannot determine current executable path")?;

    // Extract from tar.gz and replace
    let decompressed = flate2_decompress(&bytes)?;
    let binary = extract_tar_entry(&decompressed, "cerul")?;

    // Write to temp file next to current exe, then rename (atomic on same filesystem)
    let tmp_path = current_exe.with_extension("tmp");
    std::fs::write(&tmp_path, &binary).with_context(|| {
        format!(
            "Permission denied writing to {}.\n\n  Try: sudo cerul upgrade\n  Or reinstall to a user directory: CERUL_INSTALL_DIR=~/.local/bin",
            tmp_path.display()
        )
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o755)).ok();
    }

    std::fs::rename(&tmp_path, &current_exe).with_context(|| {
        format!(
            "Failed to replace {}. Try: sudo cerul upgrade",
            current_exe.display()
        )
    })?;

    eprintln!("{}", "✓".green().bold());
    eprintln!();
    eprintln!("  {} Updated to v{latest}.", "✅".green());
    eprintln!();
    Ok(())
}

/// Check latest version without blocking. Returns Some(version) if newer.
pub async fn check_update_background() -> Option<String> {
    // Skip if checked recently (cache for 24h)
    let cache_path = cache_dir().ok()?.join("last_update_check");
    if let Ok(metadata) = std::fs::metadata(&cache_path) {
        if let Ok(modified) = metadata.modified() {
            if modified.elapsed().unwrap_or_default().as_secs() < 86400 {
                let cached = std::fs::read_to_string(&cache_path).ok()?;
                let latest = cached.trim().to_string();
                if !latest.is_empty() && is_newer(&latest, CURRENT_VERSION) {
                    return Some(latest);
                }
                return None;
            }
        }
    }

    let latest = fetch_latest_version().await.ok()?;

    // Cache the result
    if let Ok(dir) = cache_dir() {
        std::fs::create_dir_all(&dir).ok();
        std::fs::write(dir.join("last_update_check"), &latest).ok();
    }

    if is_newer(&latest, CURRENT_VERSION) {
        Some(latest)
    } else {
        None
    }
}

/// Get latest version by following the GitHub releases/latest redirect.
/// Does NOT use the GitHub API (no rate limit).
async fn fetch_latest_version() -> Result<String> {
    let client = build_http_client(10)?;

    // GitHub redirects /releases/latest to /releases/tag/vX.Y.Z
    // We follow the redirect and parse the version from the final URL.
    let url = format!("https://github.com/{REPO}/releases/latest");
    let resp = client
        .get(&url)
        .send()
        .await
        .context("Failed to check for updates")?;

    // The final URL after redirect contains the tag
    let final_url = resp.url().as_str();
    let version = final_url
        .rsplit("/v")
        .next()
        .filter(|v| !v.is_empty())
        .context("Could not parse version from GitHub release URL")?;

    Ok(version.to_string())
}

async fn download_url(url: &str) -> Result<Vec<u8>> {
    let client = build_http_client(120)?;
    let resp = client.get(url).send().await.context("Failed to download")?;
    if !resp.status().is_success() {
        bail!("Download failed: HTTP {}", resp.status());
    }
    resp.bytes()
        .await
        .map(|b| b.to_vec())
        .context("Failed to read download")
}

fn build_http_client(timeout_secs: u64) -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent(format!("cerul-cli/{CURRENT_VERSION}"))
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build()
        .context("Failed to build HTTP client")
}

fn artifact_for_current_platform() -> Result<String> {
    let os = if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        bail!("Unsupported OS");
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        bail!("Unsupported architecture");
    };

    if os == "windows" {
        Ok(format!("cerul-{os}-{arch}.zip"))
    } else {
        Ok(format!("cerul-{os}-{arch}.tar.gz"))
    }
}

fn flate2_decompress(gz_data: &[u8]) -> Result<Vec<u8>> {
    use std::io::Read;
    let mut decoder = flate2::read::GzDecoder::new(gz_data);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .context("Failed to decompress")?;
    Ok(decompressed)
}

fn extract_tar_entry(tar_data: &[u8], entry_name: &str) -> Result<Vec<u8>> {
    use std::io::Read;
    let mut archive = tar::Archive::new(tar_data);
    for entry in archive.entries().context("Failed to read tar archive")? {
        let mut entry = entry.context("Failed to read tar entry")?;
        let path = entry.path().context("Failed to read entry path")?;
        if path.file_name().and_then(|n| n.to_str()) == Some(entry_name) {
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .context("Failed to extract binary")?;
            return Ok(buf);
        }
    }
    bail!("Binary '{entry_name}' not found in archive");
}

/// Compare semver strings: is `latest` strictly newer than `current`?
fn is_newer(latest: &str, current: &str) -> bool {
    let parse = |s: &str| -> (u64, u64, u64) {
        let parts: Vec<u64> = s.split('.').filter_map(|p| p.parse().ok()).collect();
        (
            parts.first().copied().unwrap_or(0),
            parts.get(1).copied().unwrap_or(0),
            parts.get(2).copied().unwrap_or(0),
        )
    };
    parse(latest) > parse(current)
}

fn cache_dir() -> Result<std::path::PathBuf> {
    let base = if cfg!(target_os = "macos") {
        env::var("HOME")
            .map(|h| std::path::PathBuf::from(h).join("Library/Caches"))
            .context("HOME not set")?
    } else if cfg!(target_os = "windows") {
        env::var("LOCALAPPDATA")
            .map(std::path::PathBuf::from)
            .context("LOCALAPPDATA not set")?
    } else {
        env::var("XDG_CACHE_HOME")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| {
                env::var("HOME")
                    .map(|h| std::path::PathBuf::from(h).join(".cache"))
                    .unwrap_or_else(|_| std::path::PathBuf::from(".cache"))
            })
    };
    Ok(base.join("cerul"))
}
