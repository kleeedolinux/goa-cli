use crate::errors::{GoaError, GoaResult};
use colored::Colorize;
use dialoguer::{Confirm, Input, Select};
use fs_extra::dir::CopyOptions;
use regex::Regex;
use std::fs;
use std::path::Path;

pub fn ensure_directory_exists(path: impl AsRef<Path>) -> GoaResult<()> {
    let path = path.as_ref();
    if !path.exists() {
        fs::create_dir_all(path)
            .map_err(|e| GoaError::Io(e))?;
        log_success(&format!("Created directory: {}", path.display()));
    }
    Ok(())
}

#[allow(dead_code)]
pub fn copy_directory(from: impl AsRef<Path>, to: impl AsRef<Path>) -> GoaResult<()> {
    let from = from.as_ref();
    let to = to.as_ref();

    if !from.exists() {
        return Err(GoaError::InvalidPath(format!(
            "Source directory does not exist: {}",
            from.display()
        )));
    }

    let options = CopyOptions::new();
    fs_extra::dir::copy(from, to, &options)
        .map_err(|e| GoaError::Other(format!("Failed to copy directory: {}", e)))?;

    Ok(())
}

pub fn write_file(path: impl AsRef<Path>, contents: &str) -> GoaResult<()> {
    let path = path.as_ref();
    let parent = path.parent().ok_or_else(|| {
        GoaError::InvalidPath(format!("Invalid file path: {}", path.display()))
    })?;

    ensure_directory_exists(parent)?;

    fs::write(path, contents)
        .map_err(|e| GoaError::Io(e))?;

    log_success(&format!("Created file: {}", path.display()));
    Ok(())
}

pub fn validate_project_name(name: &str) -> Result<(), String> {
    let name_regex = Regex::new(r"^[a-z][a-z0-9_-]*$").unwrap();
    if name.is_empty() {
        Err("Project name cannot be empty".to_string())
    } else if name.contains(" ") {
        Err("Project name cannot contain spaces".to_string())
    } else if !name_regex.is_match(name) {
        Err("Project name must start with a lowercase letter and contain only lowercase letters, numbers, underscores, and hyphens".to_string())
    } else {
        Ok(())
    }
}

pub fn validate_route_path(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("Route path cannot be empty".to_string());
    }

    let parts: Vec<&str> = path.split('/').collect();
    for part in parts {
        if part.is_empty() {
            return Err("Route path parts cannot be empty".to_string());
        }

        
        if part.starts_with('[') && part.ends_with(']') {
            let param_name = &part[1..part.len() - 1];
            if param_name.is_empty() {
                return Err("Parameter name cannot be empty".to_string());
            }
            continue;
        }

        
        let name_regex = Regex::new(r"^[a-z][a-z0-9_-]*$").unwrap();
        if !name_regex.is_match(part) {
            return Err(format!(
                "Path segment '{}' must start with a lowercase letter and contain only lowercase letters, numbers, underscores, and hyphens",
                part
            ));
        }
    }

    Ok(())
}

pub fn prompt_input<T: AsRef<str>>(prompt: T, default: Option<String>) -> GoaResult<String> {
    let input = Input::new();
    let input_with_prompt = input.with_prompt(prompt.as_ref());
    
    let input_with_default = if let Some(default_value) = default {
        input_with_prompt.default(default_value)
    } else {
        input_with_prompt
    };
    
    input_with_default.interact()
        .map_err(|e| GoaError::Other(format!("Input prompt failed: {}", e)))
}

pub fn prompt_confirm<T: AsRef<str>>(prompt: T, default: bool) -> GoaResult<bool> {
    Confirm::new()
        .with_prompt(prompt.as_ref())
        .default(default)
        .interact()
        .map_err(|e| GoaError::Other(format!("Confirmation prompt failed: {}", e)))
}

#[allow(dead_code)]
pub fn prompt_select<T: AsRef<str>>(prompt: T, options: &[String]) -> GoaResult<usize> {
    Select::new()
        .with_prompt(prompt.as_ref())
        .items(options)
        .default(0)
        .interact()
        .map_err(|e| GoaError::Other(format!("Selection prompt failed: {}", e)))
}

pub fn log_error(message: &str) {
    eprintln!("{} {}", "[ERROR]".red().bold(), message);
}

pub fn log_warning(message: &str) {
    eprintln!("{} {}", "[WARNING]".yellow().bold(), message);
}

pub fn log_info(message: &str) {
    println!("{} {}", "[INFO]".blue().bold(), message);
}

pub fn log_success(message: &str) {
    println!("{} {}", "[SUCCESS]".green().bold(), message);
}

pub fn log_step(message: &str) {
    println!("{} {}", "[STEP]".cyan().bold(), message);
}

pub fn update_main_imports(main_path: &Path, api_route: &str) -> GoaResult<()> {
    if !main_path.exists() {
        return Err(GoaError::InvalidPath(format!(
            "Main file does not exist: {}",
            main_path.display()
        )));
    }

    let content = fs::read_to_string(main_path)
        .map_err(|e| GoaError::Io(e))?;

    
    let import_line = format!("\t_ \"goonairplanes/app/api/{}\"", api_route);

    
    if content.contains(&import_line) {
        log_info(&format!("Import for {} already exists in main.go", api_route));
        return Ok(());
    }

    
    let re = Regex::new(r"import \(\n([\s\S]*?)\)").unwrap();
    if let Some(caps) = re.captures(&content) {
        let imports = &caps[1];
        let new_imports = format!("{}\n{}", imports, import_line);
        let new_content = content.replace(imports, &new_imports);

        fs::write(main_path, new_content)
            .map_err(|e| GoaError::Io(e))?;

        log_success(&format!("Added import for {} to main.go", api_route));
        return Ok(());
    }

    Err(GoaError::Other("Failed to parse main.go imports".to_string()))
}

pub fn remove_main_import(main_path: &Path, api_route: &str) -> GoaResult<()> {
    if !main_path.exists() {
        return Err(GoaError::InvalidPath(format!(
            "Main file does not exist: {}",
            main_path.display()
        )));
    }

    let content = fs::read_to_string(main_path)
        .map_err(|e| GoaError::Io(e))?;

    
    let import_line = format!("\t_ \"goonairplanes/app/api/{}\"", api_route);

    
    if !content.contains(&import_line) {
        log_info(&format!("Import for {} not found in main.go", api_route));
        return Ok(());
    }

    
    let new_content = content.replace(&format!("{}\n", import_line), "");

    fs::write(main_path, new_content)
        .map_err(|e| GoaError::Io(e))?;

    log_success(&format!("Removed import for {} from main.go", api_route));
    Ok(())
} 