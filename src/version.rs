use anyhow::Result;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use dirs;

const VERSION_CHECK_URL: &str = "https://re.juliaklee.wtf/goa-cli/version";
const VERSION_CHECK_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60); // 24 hours

#[derive(Debug, Serialize, Deserialize)]
struct VersionResponse {
    version: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct VersionCache {
    last_checked: u64,
    latest_version: String,
}

pub fn get_current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn get_cache_path() -> PathBuf {
    let mut path = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("goa-cli");
    fs::create_dir_all(&path).ok();
    path.push("version-cache.json");
    path
}

pub fn check_version() -> Result<()> {
    let current_version = format!("v{}", get_current_version());
    let cache_path = get_cache_path();
    
    // Check if we need to fetch the latest version
    let latest_version = if should_check_for_updates(&cache_path)? {
        // Fetch latest version from server
        let client = reqwest::blocking::Client::new();
        let response = client.get(VERSION_CHECK_URL).send()?;
        
        if response.status().is_success() {
            let version_info: VersionResponse = response.json()?;
            
            // Update cache
            let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs();
            let cache = VersionCache {
                last_checked: now,
                latest_version: version_info.version.clone(),
            };
            
            let cache_json = serde_json::to_string(&cache)?;
            fs::write(&cache_path, cache_json)?;
            
            version_info.version
        } else {
            // Use current version if request fails
            current_version.clone()
        }
    } else {
        // Use cached version
        let cache_data = fs::read_to_string(&cache_path)?;
        let cache: VersionCache = serde_json::from_str(&cache_data)?;
        cache.latest_version
    };
    
    // Compare versions
    if latest_version != current_version {
        println!();
        println!("{} {} → {}", 
            "A new version of GOA CLI is available:".yellow(),
            current_version.bright_red(),
            latest_version.bright_green()
        );
        println!("Run {} to upgrade.", "`goa self update`".cyan());
        println!();
    }
    
    Ok(())
}

fn should_check_for_updates(cache_path: &PathBuf) -> Result<bool> {
    // If cache file doesn't exist, we should check
    if !cache_path.exists() {
        return Ok(true);
    }
    
    // Read and parse cache
    let cache_data = fs::read_to_string(cache_path)?;
    let cache: VersionCache = serde_json::from_str(&cache_data)?;
    
    // Get current time
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();
    
    // Check if the cache is older than the check interval
    Ok(now - cache.last_checked > VERSION_CHECK_INTERVAL.as_secs())
}

pub fn handle_self_update() -> Result<()> {
    println!("Starting GOA CLI self-update...");
    
    #[cfg(target_os = "windows")]
    {
        // Download and run the PowerShell installer
        println!("Downloading Windows installer...");
        
        let temp_dir = std::env::temp_dir();
        let installer_path = temp_dir.join("goa_install.ps1");
        
        // Download the installer
        let installer_url = "https://raw.githubusercontent.com/kleeedolinux/goa-cli/master/scripts/install.ps1";
        let installer_content = reqwest::blocking::get(installer_url)?.text()?;
        fs::write(&installer_path, installer_content)?;
        
        println!("Running installer...");
        
        // Run PowerShell with the installer
        let ps_status = Command::new("powershell")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-File")
            .arg(&installer_path)
            .status()?;
        
        if !ps_status.success() {
            return Err(anyhow::anyhow!("Failed to run the installer"));
        }
        
        // Clean up
        fs::remove_file(installer_path).ok();
    }
    
    #[cfg(target_os = "macos")]
    {
        // Download and run the macOS installer
        println!("Downloading macOS installer...");
        
        // Use bash to download and run the installer
        let status = Command::new("bash")
            .arg("-c")
            .arg("curl -sSL https://raw.githubusercontent.com/kleeedolinux/goa-cli/master/scripts/macuser.sh | bash")
            .status()?;
        
        if !status.success() {
            return Err(anyhow::anyhow!("Failed to run the installer"));
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        // Download and run the Linux installer
        println!("Downloading Linux installer...");
        
        // Use bash to download and run the installer
        let status = Command::new("bash")
            .arg("-c")
            .arg("curl -sSL https://raw.githubusercontent.com/kleeedolinux/goa-cli/master/scripts/install.sh | bash")
            .status()?;
        
        if !status.success() {
            return Err(anyhow::anyhow!("Failed to run the installer"));
        }
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        return Err(anyhow::anyhow!("Self-update is not supported on this platform"));
    }
    
    println!("Self-update completed successfully!");
    Ok(())
} 