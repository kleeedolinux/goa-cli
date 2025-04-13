use anyhow::Result;
use clap::Subcommand;
use std::path::PathBuf;

use crate::config::GoaConfig;
use crate::errors::{GoaError, GoaResult};
use crate::templates;
use crate::utils;

#[derive(Subcommand)]
pub enum ComponentCommands {
    
    New {
        
        name: Option<String>,
    },
    
    
    Delete {
        
        name: Option<String>,
    },
}

pub fn handle_component_command(command: ComponentCommands) -> Result<()> {
    match command {
        ComponentCommands::New { name } => create_component(name),
        ComponentCommands::Delete { name } => delete_component(name),
    }
}

fn create_component(name_option: Option<String>) -> Result<()> {
    utils::log_step("Creating a new component");
    
    
    let component_name = match name_option {
        Some(name) => name,
        None => utils::prompt_input("Component name", None)?,
    };
    
    
    if let Err(e) = utils::validate_project_name(&component_name) {
        utils::log_error(&e);
        return Err(GoaError::ComponentGeneration(e).into());
    }
    
    
    let config_path = find_config_file()?;
    let config = GoaConfig::load(&config_path)?;
    
    
    let components_dir = config.get_components_dir();
    
    
    let component_file_path = components_dir.join(format!("{}.html", component_name));
    
    
    if component_file_path.exists() {
        if !utils::prompt_confirm(
            &format!("Component '{}' already exists. Overwrite?", component_name),
            false,
        )? {
            utils::log_info("Component creation cancelled");
            return Ok(());
        }
    }
    
    
    utils::ensure_directory_exists(&components_dir)?;
    
    
    let component_content = templates::component::basic_component()
        .replace("card", &component_name);
    
    
    utils::write_file(&component_file_path, &component_content)?;
    
    utils::log_success(&format!("Component '{}' created successfully!", component_name));
    Ok(())
}

fn delete_component(name_option: Option<String>) -> Result<()> {
    utils::log_step("Deleting a component");
    
    
    let component_name = match name_option {
        Some(name) => name,
        None => utils::prompt_input("Component name to delete", None)?,
    };
    
    
    let config_path = find_config_file()?;
    let config = GoaConfig::load(&config_path)?;
    
    
    let components_dir = config.get_components_dir();
    
    
    let component_file_path = components_dir.join(format!("{}.html", component_name));
    
    
    if !component_file_path.exists() {
        utils::log_error(&format!("Component '{}' does not exist", component_name));
        return Err(GoaError::ComponentGeneration(format!("Component '{}' does not exist", component_name)).into());
    }
    
    
    if !utils::prompt_confirm(
        &format!("Are you sure you want to delete the component '{}'?", component_name),
        false,
    )? {
        utils::log_info("Component deletion cancelled");
        return Ok(());
    }
    
    
    std::fs::remove_file(&component_file_path)
        .map_err(|e| GoaError::Io(e))?;
    
    utils::log_success(&format!("Component '{}' deleted successfully!", component_name));
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