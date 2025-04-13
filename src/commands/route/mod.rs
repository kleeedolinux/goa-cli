use anyhow::Result;
use clap::Subcommand;
use std::fs;
use std::path::PathBuf;

use crate::config::GoaConfig;
use crate::errors::{GoaError, GoaResult};
use crate::templates;
use crate::utils;

#[derive(Subcommand)]
pub enum RouteCommands {
    
    Api {
        #[clap(subcommand)]
        command: ApiCommands,
    },
    
    
    Page {
        #[clap(subcommand)]
        command: PageCommands,
    },
}

#[derive(Subcommand)]
pub enum ApiCommands {
    
    New {
        
        path: Option<String>,
    },
    
    
    Delete {
        
        path: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum PageCommands {
    
    New {
        
        path: Option<String>,
    },
    
    
    Delete {
        
        path: Option<String>,
    },
}

pub fn handle_route_command(command: RouteCommands) -> Result<()> {
    match command {
        RouteCommands::Api { command } => match command {
            ApiCommands::New { path } => create_api_route(path),
            ApiCommands::Delete { path } => delete_api_route(path),
        },
        RouteCommands::Page { command } => match command {
            PageCommands::New { path } => create_page_route(path),
            PageCommands::Delete { path } => delete_page_route(path),
        },
    }
}

fn create_api_route(path_option: Option<String>) -> Result<()> {
    utils::log_step("Creating a new API route");
    
    
    let route_path = match path_option {
        Some(path) => path,
        None => utils::prompt_input("Route path (e.g., users/auth/login)", None)?,
    };
    
    
    if let Err(e) = utils::validate_route_path(&route_path) {
        utils::log_error(&e);
        return Err(GoaError::RouteGeneration(e).into());
    }
    
    
    let config_path = find_config_file()?;
    let config = GoaConfig::load(&config_path)?;
    
    
    let mut api_dir = config.get_api_dir();
    let route_parts: Vec<&str> = route_path.split('/').collect();
    
    
    if route_parts.is_empty() {
        return Err(GoaError::RouteGeneration("Route path cannot be empty".to_string()).into());
    }
    
    
    for part in &route_parts {
        api_dir.push(part);
        utils::ensure_directory_exists(&api_dir)?;
    }
    
    
    let route_file_path = api_dir.join("route.go");
    if route_file_path.exists() {
        if !utils::prompt_confirm(
            &format!("Route file already exists at {}. Overwrite?", route_file_path.display()),
            false,
        )? {
            utils::log_info("Route creation cancelled");
            return Ok(());
        }
    }
    
    let package_name = config.meta.app_name.clone();
    utils::write_file(&route_file_path, &templates::api::route(&package_name))?;
    
    
    let main_path = config_path.parent().unwrap().join("main.go");
    utils::update_main_imports(&main_path, &route_path)?;
    
    utils::log_success(&format!("API route '{route_path}' created successfully!"));
    Ok(())
}

fn delete_api_route(path_option: Option<String>) -> Result<()> {
    utils::log_step("Deleting an API route");
    
    
    let route_path = match path_option {
        Some(path) => path,
        None => utils::prompt_input("Route path to delete", None)?,
    };
    
    
    if let Err(e) = utils::validate_route_path(&route_path) {
        utils::log_error(&e);
        return Err(GoaError::RouteGeneration(e).into());
    }
    
    
    let config_path = find_config_file()?;
    let config = GoaConfig::load(&config_path)?;
    
    
    let mut api_route_dir = config.get_api_dir();
    let route_parts: Vec<&str> = route_path.split('/').collect();
    
    for part in &route_parts {
        api_route_dir.push(part);
    }
    
    
    if !api_route_dir.exists() {
        utils::log_error(&format!("API route '{}' does not exist", route_path));
        return Err(GoaError::RouteGeneration(format!("API route '{}' does not exist", route_path)).into());
    }
    
    
    if !utils::prompt_confirm(
        &format!("Are you sure you want to delete the API route '{}'?", route_path),
        false,
    )? {
        utils::log_info("Route deletion cancelled");
        return Ok(());
    }
    
    
    fs::remove_dir_all(&api_route_dir)
        .map_err(|e| GoaError::Io(e))?;
    
    
    let main_path = config_path.parent().unwrap().join("main.go");
    utils::remove_main_import(&main_path, &route_path)?;
    
    utils::log_success(&format!("API route '{route_path}' deleted successfully!"));
    Ok(())
}

fn create_page_route(path_option: Option<String>) -> Result<()> {
    utils::log_step("Creating a new page route");
    
    
    let route_path = match path_option {
        Some(path) => path,
        None => utils::prompt_input("Page path (e.g., dashboard)", None)?,
    };
    
    
    if let Err(e) = utils::validate_route_path(&route_path) {
        utils::log_error(&e);
        return Err(GoaError::RouteGeneration(e).into());
    }
    
    
    let config_path = find_config_file()?;
    let config = GoaConfig::load(&config_path)?;
    
    
    let mut page_dir = config.get_app_dir();
    let route_parts: Vec<&str> = route_path.split('/').collect();
    
    
    if route_parts.is_empty() {
        return Err(GoaError::RouteGeneration("Page path cannot be empty".to_string()).into());
    }
    
    
    for part in &route_parts {
        page_dir.push(part);
        utils::ensure_directory_exists(&page_dir)?;
    }
    
    
    let is_dynamic = route_parts.iter().any(|part| part.starts_with('[') && part.ends_with(']'));
    
    
    let page_file_path = page_dir.join("index.html");
    if page_file_path.exists() {
        if !utils::prompt_confirm(
            &format!("Page file already exists at {}. Overwrite?", page_file_path.display()),
            false,
        )? {
            utils::log_info("Page creation cancelled");
            return Ok(());
        }
    }
    
    if is_dynamic {
        utils::write_file(&page_file_path, templates::page::dynamic_page())?;
    } else {
        utils::write_file(&page_file_path, templates::page::normal_page())?;
    }
    
    utils::log_success(&format!("Page route '{route_path}' created successfully!"));
    Ok(())
}

fn delete_page_route(path_option: Option<String>) -> Result<()> {
    utils::log_step("Deleting a page route");
    
    
    let route_path = match path_option {
        Some(path) => path,
        None => utils::prompt_input("Page path to delete", None)?,
    };
    
    
    if let Err(e) = utils::validate_route_path(&route_path) {
        utils::log_error(&e);
        return Err(GoaError::RouteGeneration(e).into());
    }
    
    
    let config_path = find_config_file()?;
    let config = GoaConfig::load(&config_path)?;
    
    
    let mut page_dir = config.get_app_dir();
    let route_parts: Vec<&str> = route_path.split('/').collect();
    
    for part in &route_parts {
        page_dir.push(part);
    }
    
    
    if !page_dir.exists() {
        utils::log_error(&format!("Page route '{}' does not exist", route_path));
        return Err(GoaError::RouteGeneration(format!("Page route '{}' does not exist", route_path)).into());
    }
    
    
    if !utils::prompt_confirm(
        &format!("Are you sure you want to delete the page route '{}'?", route_path),
        false,
    )? {
        utils::log_info("Page route deletion cancelled");
        return Ok(());
    }
    
    
    fs::remove_dir_all(&page_dir)
        .map_err(|e| GoaError::Io(e))?;
    
    utils::log_success(&format!("Page route '{route_path}' deleted successfully!"));
    Ok(())
}

fn find_config_file() -> GoaResult<PathBuf> {
    
    let current_dir = std::env::current_dir().map_err(|e| GoaError::Io(e))?;
    let config_path = current_dir.join("config.json");
    
    if config_path.exists() {
        return Ok(config_path);
    }
    
    
    let mut dir = current_dir;
    while let Some(parent) = dir.parent() {
        let parent_config = parent.join("config.json");
        if parent_config.exists() {
            return Ok(parent_config);
        }
        dir = parent.to_path_buf();
    }
    
    
    Err(GoaError::Configuration("Could not find config.json file. Are you inside a Go on Airplanes project?".to_string()))
} 