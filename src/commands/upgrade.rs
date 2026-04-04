use std::env;

use anyhow::{bail, Context, Result};

const REPO: &str = "cerul-ai/cerul-cli";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(serde::Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(serde::Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

pub async fn run() -> Result<()> {
    eprintln!("Current version: v{CURRENT_VERSION}");
    eprint!("Checking for updates...");

    let release = fetch_latest_release().await?;
    let latest = release.tag_name.trim_start_matches('v');

    if latest == CURRENT_VERSION {
        eprintln!(" up to date.");
        return Ok(());
    }

    eprintln!(" v{latest} available.\n");

    let artifact_name = artifact_for_current_platform()?;
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == artifact_name)
        .with_context(|| format!("No binary found for this platform ({artifact_name})"))?;

    eprintln!("Downloading {artifact_name}...");
    let bytes = download_asset(&asset.browser_download_url).await?;

    let current_exe = env::current_exe().context("Cannot determine current executable path")?;

    // Extract from tar.gz and replace
    let decompressed = flate2_decompress(&bytes)?;
    let binary = extract_tar_entry(&decompressed, "cerul")?;

    // Write to temp file next to current exe, then rename (atomic on same filesystem)
    let tmp_path = current_exe.with_extension("tmp");
    std::fs::write(&tmp_path, &binary)
        .with_context(|| format!("Failed to write {}", tmp_path.display()))?;

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

    eprintln!("Updated to v{latest}.");
    Ok(())
}

/// Check latest version without blocking. Returns Some(version) if newer.
pub async fn check_update_background() -> Option<String> {
    // Skip if checked recently (cache for 24h)
    let cache_path = cache_dir().ok()?.join("last_update_check");
    if let Ok(metadata) = std::fs::metadata(&cache_path) {
        if let Ok(modified) = metadata.modified() {
            if modified.elapsed().unwrap_or_default().as_secs() < 86400 {
                // Read cached latest version
                let cached = std::fs::read_to_string(&cache_path).ok()?;
                let latest = cached.trim();
                if latest != CURRENT_VERSION && !latest.is_empty() {
                    return Some(latest.to_string());
                }
                return None;
            }
        }
    }

    let release = fetch_latest_release().await.ok()?;
    let latest = release.tag_name.trim_start_matches('v').to_string();

    // Cache the result
    if let Ok(dir) = cache_dir() {
        std::fs::create_dir_all(&dir).ok();
        std::fs::write(dir.join("last_update_check"), &latest).ok();
    }

    if latest != CURRENT_VERSION {
        Some(latest)
    } else {
        None
    }
}

async fn fetch_latest_release() -> Result<GitHubRelease> {
    let client = reqwest::Client::builder()
        .user_agent(format!("cerul-cli/{CURRENT_VERSION}"))
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let url = format!("https://api.github.com/repos/{REPO}/releases/latest");
    let resp = client.get(&url).send().await.context("Failed to check for updates")?;

    if !resp.status().is_success() {
        bail!("GitHub API returned {}", resp.status());
    }

    resp.json().await.context("Failed to parse release info")
}

async fn download_asset(url: &str) -> Result<Vec<u8>> {
    let client = reqwest::Client::builder()
        .user_agent(format!("cerul-cli/{CURRENT_VERSION}"))
        .timeout(std::time::Duration::from_secs(120))
        .build()?;

    let resp = client.get(url).send().await.context("Failed to download")?;
    if !resp.status().is_success() {
        bail!("Download failed: HTTP {}", resp.status());
    }

    resp.bytes().await.map(|b| b.to_vec()).context("Failed to read download")
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
    decoder.read_to_end(&mut decompressed).context("Failed to decompress")?;
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
            entry.read_to_end(&mut buf).context("Failed to extract binary")?;
            return Ok(buf);
        }
    }
    bail!("Binary '{entry_name}' not found in archive");
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
