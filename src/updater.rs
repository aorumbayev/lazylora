use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

const GITHUB_REPO: &str = "aorumbayev/lazylora";

#[derive(Debug, Deserialize, Serialize)]
struct Release {
    tag_name: String,
    html_url: String,
    assets: Vec<Asset>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

pub fn check_for_updates(current_version: &str) -> Result<bool, String> {
    println!("Checking for updates...");

    let latest_release = fetch_latest_release()?;
    let latest_version = latest_release.tag_name.trim_start_matches('v');

    println!("Current version: {}", current_version);
    println!("Latest version: {}", latest_version);

    if latest_version != current_version {
        println!(
            "Update available: {} -> {}",
            current_version, latest_version
        );
        return Ok(true);
    }

    println!("You are using the latest version.");
    Ok(false)
}

pub fn update_app() -> Result<(), String> {
    // Get current executable path
    let current_exe =
        env::current_exe().map_err(|e| format!("Failed to get current executable path: {}", e))?;

    // Get OS and architecture info
    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "apple-darwin"
    } else {
        "unknown-linux-gnu"
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        return Err("Unsupported architecture".to_string());
    };

    // Fetch latest release info
    let latest_release = fetch_latest_release()?;
    let latest_version = latest_release.tag_name.trim_start_matches('v');

    println!("Updating to version {}...", latest_version);

    // Find the right asset for this platform
    let asset_pattern = format!("lazylora-{}-{}-{}", latest_version, arch, os);
    let asset = latest_release
        .assets
        .iter()
        .find(|asset| asset.name.contains(&asset_pattern))
        .ok_or_else(|| format!("No release found for your platform ({})", asset_pattern))?;

    println!("Downloading {}...", asset.browser_download_url);

    // Create a temporary directory
    let temp_dir =
        tempfile::tempdir().map_err(|e| format!("Failed to create temporary directory: {}", e))?;

    // Download the archive
    let archive_path = temp_dir.path().join(&asset.name);
    download_file(&asset.browser_download_url, &archive_path)?;

    // Extract the archive
    extract_archive(&archive_path, temp_dir.path())?;

    // Find the binary in the extracted files
    let binary_path = temp_dir.path().join("lazylora");
    if !binary_path.exists() {
        return Err("Binary not found in the extracted archive".to_string());
    }

    // On macOS, remove quarantine attribute
    #[cfg(target_os = "macos")]
    {
        Command::new("xattr")
            .args(&["-d", "com.apple.quarantine", binary_path.to_str().unwrap()])
            .output()
            .ok();
    }

    // Replace the current executable
    fs::rename(&binary_path, &current_exe)
        .map_err(|e| format!("Failed to replace executable: {}", e))?;

    println!("Update completed successfully!");
    Ok(())
}

fn fetch_latest_release() -> Result<Release, String> {
    let client = Client::new();
    let url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        GITHUB_REPO
    );

    client
        .get(&url)
        .header("User-Agent", "lazylora-updater")
        .send()
        .map_err(|e| format!("Failed to contact GitHub API: {}", e))?
        .json::<Release>()
        .map_err(|e| format!("Failed to parse GitHub API response: {}", e))
}

fn download_file(url: &str, dest: &Path) -> Result<(), String> {
    let client = Client::new();
    let mut response = client
        .get(url)
        .header("User-Agent", "lazylora-updater")
        .send()
        .map_err(|e| format!("Failed to download file: {}", e))?;

    let mut file = fs::File::create(dest).map_err(|e| format!("Failed to create file: {}", e))?;

    std::io::copy(&mut response, &mut file).map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(())
}

fn extract_archive(archive_path: &Path, dest_dir: &Path) -> Result<(), String> {
    if archive_path.extension().unwrap_or_default() == "zip" {
        // Extract zip archive
        let file = fs::File::open(archive_path)
            .map_err(|e| format!("Failed to open zip archive: {}", e))?;

        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| format!("Failed to read zip archive: {}", e))?;

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| format!("Failed to read zip entry: {}", e))?;

            let outpath = dest_dir.join(file.name());

            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p)
                            .map_err(|e| format!("Failed to create parent directory: {}", e))?;
                    }
                }

                let mut outfile = fs::File::create(&outpath)
                    .map_err(|e| format!("Failed to create file: {}", e))?;

                std::io::copy(&mut file, &mut outfile)
                    .map_err(|e| format!("Failed to write file: {}", e))?;
            }
        }
    } else {
        // Extract tar.gz archive
        let file = fs::File::open(archive_path)
            .map_err(|e| format!("Failed to open tar.gz archive: {}", e))?;

        let tar = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(tar);

        archive
            .unpack(dest_dir)
            .map_err(|e| format!("Failed to extract tar.gz archive: {}", e))?;
    }

    Ok(())
}
