use anyhow::Result;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use dirs;

const VERSION_CHECK_URL: &str = "https://re.juliaklee.wtf/goa-cli/version";
const VERSION_CHECK_INTERVAL: Duration = Duration::from_secs(1 * 60 * 60); 

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
    let cache_path = get_cache_path();
    
    let latest_version = if should_check_for_updates(&cache_path)? {
        let client = reqwest::blocking::Client::new();
        let response = client.get(VERSION_CHECK_URL).send()?;
        
        if response.status().is_success() {
            let version_info: VersionResponse = response.json()?;
            
            let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs();
            let cache = VersionCache {
                last_checked: now,
                latest_version: version_info.version.clone(),
            };
            
            let cache_json = serde_json::to_string(&cache)?;
            fs::write(&cache_path, cache_json)?;
            
            version_info.version
        } else {
            format!("v{}", get_current_version())
        }
    } else {
        let cache_data = fs::read_to_string(&cache_path)?;
        let cache: VersionCache = serde_json::from_str(&cache_data)?;
        cache.latest_version
    };
    
    if latest_version != format!("v{}", get_current_version()) {
        println!();
        println!("{} {} → {}", 
            "A new version of GOA CLI is available:".yellow(),
            format!("v{}", get_current_version()).bright_red(),
            latest_version.bright_green()
        );
        println!("Run {} to upgrade.", "`goa self update`".cyan());
        println!();
    }
    
    Ok(())
}

fn should_check_for_updates(cache_path: &PathBuf) -> Result<bool> {
    if !cache_path.exists() {
        return Ok(true);
    }
    
    
    let cache_data = fs::read_to_string(cache_path)?;
    let cache: VersionCache = serde_json::from_str(&cache_data)?;
    
    
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();
    
    
    Ok(now - cache.last_checked > VERSION_CHECK_INTERVAL.as_secs())
}

pub fn handle_self_update() -> Result<()> {
    println!("Checking for updates...");
    
    let current_version = format!("v{}", get_current_version());
    
    let client = reqwest::blocking::Client::new();
    let response = client.get(VERSION_CHECK_URL).send()?;
    
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Failed to check for updates"));
    }
    
    let version_info: VersionResponse = response.json()?;
    let latest_version = version_info.version;
    
    if latest_version == current_version {
        println!("You already have the latest version ({}).", current_version);
        return Ok(());
    }
    
    println!("{} {} → {}", 
        "Updating GOA CLI:".yellow(),
        current_version.bright_red(),
        latest_version.bright_green()
    );
    
    #[cfg(target_os = "windows")]
    {
        println!("Downloading Windows installer...");
        
        let temp_dir = std::env::temp_dir();
        let installer_path = temp_dir.join("goa_install.ps1");
        
        let installer_url = "https://raw.githubusercontent.com/kleeedolinux/goa-cli/master/scripts/install.ps1";
        let installer_content = reqwest::blocking::get(installer_url)?.text()?;
        fs::write(&installer_path, installer_content)?;
        
        println!("Running installer...");
        
        let ps_status = Command::new("powershell")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-File")
            .arg(&installer_path)
            .status()?;
        
        if !ps_status.success() {
            return Err(anyhow::anyhow!("Failed to run the installer"));
        }
        
        fs::remove_file(installer_path).ok();
    }
    
    #[cfg(target_os = "macos")]
    {
        println!("Downloading macOS installer...");
        
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
        println!("Downloading Linux installer...");
        
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

pub fn get_latest_version() -> Result<String> {
    let cache_path = get_cache_path();
    
    if !cache_path.exists() || should_check_for_updates(&cache_path)? {
        let client = reqwest::blocking::Client::new();
        let response = client.get(VERSION_CHECK_URL).send()?;
        
        if response.status().is_success() {
            let version_info: VersionResponse = response.json()?;
            
            let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs();
            let cache = VersionCache {
                last_checked: now,
                latest_version: version_info.version.clone(),
            };
            
            let cache_json = serde_json::to_string(&cache)?;
            fs::write(&cache_path, cache_json)?;
            
            Ok(version_info.version)
        } else {
            Ok(format!("v{}", get_current_version()))
        }
    } else {
        let cache_data = fs::read_to_string(&cache_path)?;
        let cache: VersionCache = serde_json::from_str(&cache_data)?;
        Ok(cache.latest_version)
    }
} 