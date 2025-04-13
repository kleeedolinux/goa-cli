use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::process::Command;

mod commands;
mod config;
mod errors;
mod templates;
mod utils;
mod version;

#[derive(Parser)]
#[clap(name = "goa", about = "Go on Airplanes CLI - Developer-focused tooling for the Go on Airplanes framework", version, disable_version_flag = true)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,

    #[clap(long = "version", short = 'v', help = "Print version information", global = true)]
    version_flag: bool,
}

#[derive(Subcommand)]
enum Commands {
    
    #[clap(about = "Create a new Go on Airplanes project")]
    Project {
        #[clap(subcommand)]
        command: commands::project::ProjectCommands,
    },
    
    
    #[clap(about = "Generate or modify API and page routes")]
    Route {
        #[clap(subcommand)]
        command: commands::route::RouteCommands,
    },
    
    
    #[clap(about = "Generate a new component")]
    Component {
        #[clap(subcommand)]
        command: commands::component::ComponentCommands,
    },
    
    #[clap(name = "self", about = "Update the CLI to the latest version")]
    SelfCmd {
        #[clap(subcommand)]
        command: SelfCommands,
    },
}

#[derive(Subcommand)]
enum SelfCommands {
    #[clap(about = "Update the CLI to the latest version")]
    Update,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    if cli.version_flag {
        print_version_info();
        return Ok(());
    }
    
    print_banner();
    
    let _ = version::check_version();
    
    if !verify_requirements()? {
        return Ok(());
    }
    
    match cli.command {
        Some(command) => match command {
            Commands::Project { command } => {
                commands::project::handle_project_command(command)
            },
            Commands::Route { command } => {
                commands::route::handle_route_command(command)
            },
            Commands::Component { command } => {
                commands::component::handle_component_command(command)
            },
            Commands::SelfCmd { command } => {
                match command {
                    SelfCommands::Update => version::handle_self_update(),
                }
            },
        },
        None => {
            print_version_info();
            Ok(())
        }
    }
}

fn verify_requirements() -> Result<bool> {
    let mut all_requirements_met = true;
    
    
    if !is_command_available("git") {
        all_requirements_met = false;
        utils::log_error("Git is not installed on your system");
        
        if utils::prompt_confirm("Would you like to install Git now?", true)? {
            if install_git()? {
                utils::log_success("Git installed successfully");
            } else {
                utils::log_error("Failed to install Git automatically");
                utils::log_info("Please install Git manually from https://git-scm.com/downloads");
                return Ok(false);
            }
        } else {
            utils::log_info("Git is required for Go on Airplanes. Please install it from https://git-scm.com/downloads");
            return Ok(false);
        }
    }
    
    
    if !is_command_available("go") {
        all_requirements_met = false;
        utils::log_error("Go is not installed on your system");
        
        if utils::prompt_confirm("Would you like to install Go now?", true)? {
            if install_go()? {
                utils::log_success("Go installed successfully");
            } else {
                utils::log_error("Failed to install Go automatically");
                utils::log_info("Please install Go manually from https://golang.org/dl/");
                return Ok(false);
            }
        } else {
            utils::log_info("Go is required for Go on Airplanes. Please install it from https://golang.org/dl/");
            return Ok(false);
        }
    } else {
        
        if let Ok(output) = Command::new("go").arg("version").output() {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout);
                utils::log_success(&format!("Found {}", version.trim()));
            }
        }
    }
    
    Ok(all_requirements_met)
}

fn is_command_available(command: &str) -> bool {
    #[cfg(windows)]
    let check_command = Command::new("where").arg(command).output();
    
    #[cfg(not(windows))]
    let check_command = Command::new("which").arg(command).output();
    
    match check_command {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

fn install_git() -> Result<bool> {
    utils::log_step("Installing Git...");
    
    #[cfg(target_os = "windows")]
    {
        utils::log_info("Automatic installation is not supported on Windows");
        utils::log_info("Please download and install Git from https://git-scm.com/download/win");
        return Ok(false);
    }
    
    #[cfg(target_os = "macos")]
    {
        if is_command_available("brew") {
            return Ok(Command::new("brew").args(["install", "git"]).status()?.success());
        } else {
            utils::log_info("Homebrew not found. Installing via Homebrew is recommended");
            utils::log_info("Please download and install Git from https://git-scm.com/download/mac");
            return Ok(false);
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        if is_command_available("apt-get") {
            
            if let Ok(status) = Command::new("sudo").args(["apt-get", "update"]).status() {
                if status.success() {
                    return Ok(Command::new("sudo").args(["apt-get", "install", "-y", "git"]).status()?.success());
                }
            }
        } else if is_command_available("yum") {
            
            return Ok(Command::new("sudo").args(["yum", "install", "-y", "git"]).status()?.success());
        } else if is_command_available("dnf") {
            
            return Ok(Command::new("sudo").args(["dnf", "install", "-y", "git"]).status()?.success());
        } else if is_command_available("pacman") {
            
            return Ok(Command::new("sudo").args(["pacman", "-S", "--noconfirm", "git"]).status()?.success());
        }
        
        utils::log_info("No supported package manager found");
        utils::log_info("Please install Git manually from https://git-scm.com/download/linux");
        return Ok(false);
    }
    
    #[allow(unreachable_code)]
    Ok(false)
}

fn install_go() -> Result<bool> {
    utils::log_step("Installing Go...");
    
    #[cfg(target_os = "windows")]
    {
        utils::log_info("Automatic installation is not supported on Windows");
        utils::log_info("Please download and install Go from https://golang.org/dl/");
        return Ok(false);
    }
    
    #[cfg(target_os = "macos")]
    {
        if is_command_available("brew") {
            return Ok(Command::new("brew").args(["install", "go"]).status()?.success());
        } else {
            utils::log_info("Homebrew not found. Installing via Homebrew is recommended");
            utils::log_info("Please download and install Go from https://golang.org/dl/");
            return Ok(false);
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        if is_command_available("apt-get") {
            
            if let Ok(status) = Command::new("sudo").args(["apt-get", "update"]).status() {
                if status.success() {
                    return Ok(Command::new("sudo").args(["apt-get", "install", "-y", "golang"]).status()?.success());
                }
            }
        } else if is_command_available("yum") {
            
            return Ok(Command::new("sudo").args(["yum", "install", "-y", "golang"]).status()?.success());
        } else if is_command_available("dnf") {
            
            return Ok(Command::new("sudo").args(["dnf", "install", "-y", "golang"]).status()?.success());
        } else if is_command_available("pacman") {
            
            return Ok(Command::new("sudo").args(["pacman", "-S", "--noconfirm", "go"]).status()?.success());
        }
        
        utils::log_info("No supported package manager found");
        utils::log_info("Please install Go manually from https://golang.org/dl/");
        return Ok(false);
    }
    
    #[allow(unreachable_code)]
    Ok(false)
}

fn print_version_info() {
    let current_version = version::get_current_version();
    println!("GOA CLI v{}", current_version);
    
    match version::get_latest_version() {
        Ok(latest_version) => {
            if latest_version != format!("v{}", current_version) {
                println!("Latest version: {}", latest_version.bright_green());
                println!("Run {} to upgrade.", "`goa self update`".cyan());
            } else {
                println!("You are using the latest version.");
            }
        },
        Err(_) => {
            println!("Could not check for updates.");
        }
    }
}

fn print_banner() {
    let banner = r#"
   ██████╗  ██████╗      ██████╗ ███╗   ██╗     █████╗ ██╗██████╗ ██████╗ ██╗      █████╗ ███╗   ██╗███████╗███████╗
  ██╔════╝ ██╔═══██╗    ██╔═══██╗████╗  ██║    ██╔══██╗██║██╔══██╗██╔══██╗██║     ██╔══██╗████╗  ██║██╔════╝██╔════╝
  ██║  ███╗██║   ██║    ██║   ██║██╔██╗ ██║    ███████║██║██████╔╝██████╔╝██║     ███████║██╔██╗ ██║█████╗  ███████╗
  ██║   ██║██║   ██║    ██║   ██║██║╚██╗██║    ██╔══██║██║██╔══██╗██╔═══╝ ██║     ██╔══██║██║╚██╗██║██╔══╝  ╚════██║
  ╚██████╔╝╚██████╔╝    ╚██████╔╝██║ ╚████║    ██║  ██║██║██║  ██║██║     ███████╗██║  ██║██║ ╚████║███████╗███████║
   ╚═════╝  ╚═════╝      ╚═════╝ ╚═╝  ╚═══╝    ╚═╝  ╚═╝╚═╝╚═╝  ╚═╝╚═╝     ╚══════╝╚═╝  ╚═╝╚═╝  ╚═══╝╚══════╝╚══════╝
    "#;
    
    println!("{}", banner.cyan());
    println!("{} v{}", "Go on Airplanes CLI - Fly high with simple web development".bright_blue(), version::get_current_version().bright_yellow());
    println!();
}
