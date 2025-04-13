use anyhow::Result;
use clap::Subcommand;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use serde_json::{Value, json};
use colored::Colorize;

use crate::errors::GoaError;
use crate::utils;

#[derive(Subcommand)]
pub enum ProjectCommands {
    New,
    
    List,
    
    Config,
    
    Build {
        #[clap(long, short)]
        output: Option<String>,
    },
}

pub fn handle_project_command(command: ProjectCommands) -> Result<()> {
    match command {
        ProjectCommands::New => create_new_project(),
        ProjectCommands::List => list_project_routes(),
        ProjectCommands::Config => configure_project(),
        ProjectCommands::Build { output } => build_project(output),
    }
}

fn create_new_project() -> Result<()> {
    utils::log_step("Creating a new Go on Airplanes project");

    
    let project_name = utils::prompt_input("Project name", None)?;
    
    
    let project_description = utils::prompt_input("Project description", Some("A modern Go web application".to_string()))?;
    
    
    let default_dir = format!("./{}", project_name);
    let project_dir = utils::prompt_input("Directory", Some(default_dir))?;
    
    
    let with_docs = utils::prompt_confirm("Include documentation?", true)?;
    
    
    utils::log_step("Running Go on Airplanes setup...");
    
    
    #[cfg(windows)]
    let setup_result = if Command::new("where").arg("bash").output().is_ok() {
        Command::new("bash")
            .args(["-c", &format!("git clone https://github.com/kleeedolinux/goonairplanes.git {}", project_dir)])
            .output()
    } else {
        Command::new("powershell")
            .args(["-Command", &format!("git clone https://github.com/kleeedolinux/goonairplanes.git {}", project_dir)])
            .output()
    };
    
    
    #[cfg(not(windows))]
    let setup_result = Command::new("bash")
        .args(["-c", &format!("git clone https://github.com/kleeedolinux/goonairplanes.git {}", project_dir)])
        .output();
    
    match setup_result {
        Ok(output) => {
            if output.status.success() {
                utils::log_success("Go on Airplanes repository cloned successfully!");
                
                
                cleanup_files(PathBuf::from(&project_dir), with_docs)?;
                
                
                let config_path = PathBuf::from(&project_dir).join("config.json");
                update_config_meta(&config_path, &project_name, &project_description)?;
                
                
                let git_init = Command::new("git")
                    .args(["init"])
                    .current_dir(&project_dir)
                    .output();
                
                if let Ok(git_output) = git_init {
                    if git_output.status.success() {
                        utils::log_success("Initialized Git repository");
                    }
                }
                
                
                let go_tidy = Command::new("go")
                    .args(["mod", "tidy"])
                    .current_dir(&project_dir)
                    .output();
                
                if let Ok(go_output) = go_tidy {
                    if go_output.status.success() {
                        utils::log_success("Go dependencies installed");
                    }
                }
                
                utils::log_success(&format!("Project '{}' created successfully!", project_name));
                utils::log_info(&format!("Your project is ready at: {}", project_dir));
                utils::log_info("To run your project:");
                utils::log_info(&format!("  cd {}", project_dir));
                utils::log_info("  go run main.go");
                
                Ok(())
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                utils::log_error(&format!("Failed to clone repository: {}", error));
                Err(GoaError::ProjectCreation(format!("Failed to clone repository: {}", error)).into())
            }
        }
        Err(e) => {
            utils::log_error(&format!("Failed to run setup: {}", e));
            Err(GoaError::ProjectCreation(format!("Failed to run setup: {}", e)).into())
        }
    }
}

fn cleanup_files(project_dir: PathBuf, keep_docs: bool) -> Result<()> {
    utils::log_step("Cleaning up unnecessary files...");
    
    
    let git_dir = project_dir.join(".git");
    if git_dir.exists() {
        if let Err(e) = fs::remove_dir_all(&git_dir) {
            utils::log_warning(&format!("Failed to remove .git directory: {}", e));
        }
    }
    
    
    let files_to_remove = vec![
        "img", "README.md", "MANIFEST.md", "BENCHMARK.MD", 
        "CODE_OF_CONDUCT.md", "ROADMAP.md", "SECURITY.md"
    ];
    
    for file in files_to_remove {
        let file_path = project_dir.join(file);
        if file_path.exists() {
            if file_path.is_dir() {
                if let Err(e) = fs::remove_dir_all(&file_path) {
                    utils::log_warning(&format!("Failed to remove directory {}: {}", file, e));
                }
            } else {
                if let Err(e) = fs::remove_file(&file_path) {
                    utils::log_warning(&format!("Failed to remove file {}: {}", file, e));
                }
            }
        }
    }
    
    
    if !keep_docs {
        let docs_dir = project_dir.join("docs");
        if docs_dir.exists() {
            if let Err(e) = fs::remove_dir_all(&docs_dir) {
                utils::log_warning(&format!("Failed to remove docs directory: {}", e));
            } else {
                utils::log_success("Documentation removed");
            }
        }
    } else {
        utils::log_success("Documentation kept");
    }
    
    
    let scripts_dir = project_dir.join("scripts");
    if scripts_dir.exists() {
        if let Err(e) = fs::remove_dir_all(&scripts_dir) {
            utils::log_warning(&format!("Failed to remove scripts directory: {}", e));
        }
    }
    
    utils::log_success("Cleanup completed");
    Ok(())
}

fn update_config_meta(config_path: &PathBuf, project_name: &str, project_description: &str) -> Result<()> {
    if !config_path.exists() {
        return Err(GoaError::Configuration(format!("Config file not found at {}", config_path.display())).into());
    }
    
    
    let config_str = fs::read_to_string(config_path)
        .map_err(|e| GoaError::Io(e))?;
    
    
    let mut config: Value = serde_json::from_str(&config_str)
        .map_err(|e| GoaError::Json(e))?;
    
    
    if let Some(meta) = config.get_mut("meta") {
        if let Some(meta_obj) = meta.as_object_mut() {
            meta_obj.insert("appName".to_string(), json!(project_name));
            
            if let Some(meta_tags) = meta_obj.get_mut("defaultMetaTags") {
                if let Some(meta_tags_obj) = meta_tags.as_object_mut() {
                    meta_tags_obj.insert("description".to_string(), json!(project_description));
                    meta_tags_obj.insert("og:title".to_string(), json!(project_name));
                }
            }
        }
    }
    
    
    let updated_config = serde_json::to_string_pretty(&config)
        .map_err(|e| GoaError::Json(e))?;
    
    fs::write(config_path, updated_config)
        .map_err(|e| GoaError::Io(e))?;
    
    utils::log_success("Updated project configuration");
    Ok(())
}

fn list_project_routes() -> Result<()> {
    utils::log_step("Analyzing project structure");
    
    let config_path = find_config_file()?;
    let config = fs::read_to_string(&config_path)
        .map_err(|e| GoaError::Io(e))?;
    
    let config: Value = serde_json::from_str(&config)
        .map_err(|e| GoaError::Json(e))?;
    
    let app_dir = if let Some(dirs) = config.get("directories") {
        if let Some(app_dir) = dirs.get("appDir") {
            app_dir.as_str().unwrap_or("app").to_string()
        } else {
            "app".to_string()
        }
    } else {
        "app".to_string()
    };
    
    let app_name = if let Some(meta) = config.get("meta") {
        if let Some(name) = meta.get("appName") {
            name.as_str().unwrap_or("Go on Airplanes").to_string()
        } else {
            "Go on Airplanes".to_string()
        }
    } else {
        "Go on Airplanes".to_string()
    };
    
    let project_dir = config_path.parent().unwrap().to_path_buf();
    let app_path = project_dir.join(&app_dir);
    
    if !app_path.exists() {
        utils::log_error(&format!("App directory not found at {}", app_path.display()));
        return Err(GoaError::InvalidPath(format!("App directory not found at {}", app_path.display())).into());
    }
    
    
    println!("\n{}", "╭───────────────────────────────────────────────────╮".cyan());
    println!("{}{:^53}{}", "│".cyan(), app_name.bold(), "│".cyan());
    println!("{}", "╰───────────────────────────────────────────────────╯".cyan());
    
    
    list_api_routes(&app_path)?;
    list_page_routes(&app_path)?;
    list_components(&app_path, &config)?;
    
    Ok(())
}

fn list_api_routes(app_path: &PathBuf) -> Result<()> {
    let api_path = app_path.join("api");
    
    if !api_path.exists() {
        println!("\n{} {}", "API ROUTES:".cyan().bold(), "(none)".dimmed());
        return Ok(());
    }
    
    println!("\n{}", format!("╭─ API ROUTES {}", "─".repeat(40)).cyan().bold());
    
    let routes = find_routes_in_directory(&api_path, "route.go", |path| {
        !path.to_string_lossy().contains("/components/")
    })?;
    
    if routes.is_empty() {
        println!("│  {}", "(none)".dimmed());
    } else {
        
        let mut route_tree: std::collections::BTreeMap<String, Vec<String>> = std::collections::BTreeMap::new();
        
        for route_path in routes {
            let relative_path = route_path.strip_prefix(&api_path).unwrap_or(&route_path);
            let parent = relative_path.parent().unwrap_or(relative_path);
            let route_str = parent.to_string_lossy().to_string();
            
            
            if route_str.is_empty() {
                continue;
            }
            
            
            let parts: Vec<&str> = route_str.split('/').filter(|s| !s.is_empty()).collect();
            if parts.is_empty() {
                route_tree.entry("/".to_string()).or_insert(vec![]);
            } else {
                let mut current_path = String::new();
                for (i, part) in parts.iter().enumerate() {
                    if i > 0 {
                        current_path.push('/');
                    }
                    current_path.push_str(part);
                    
                    if i == parts.len() - 1 {
                        route_tree.entry(format!("/{}", current_path)).or_insert(vec![]);
                    }
                }
            }
        }
        
        
        for (i, (route, _)) in route_tree.iter().enumerate() {
            let is_last = i == route_tree.len() - 1;
            let prefix = if is_last { "└─ " } else { "├─ " };
            println!("│ {}{}", prefix.cyan(), route.green().bold());
        }
    }
    
    println!("{}", format!("╰{}", "─".repeat(50)).cyan());
    Ok(())
}

fn list_page_routes(app_path: &PathBuf) -> Result<()> {
    println!("\n{}", format!("╭─ PAGE ROUTES {}", "─".repeat(38)).magenta().bold());
    
    let routes = find_routes_in_directory(app_path, "index.html", |path| {
        !path.to_string_lossy().contains("/components/") && 
        !path.to_string_lossy().contains("/api/")
    })?;
    
    if routes.is_empty() {
        println!("│  {}", "(none)".dimmed());
    } else {
        
        let mut route_tree: std::collections::BTreeMap<String, Vec<String>> = std::collections::BTreeMap::new();
        
        for route_path in routes {
            let relative_path = route_path.strip_prefix(app_path).unwrap_or(&route_path);
            let parent = relative_path.parent().unwrap_or(relative_path);
            let route_str = parent.to_string_lossy().to_string();
            
            
            if route_str.is_empty() {
                route_tree.entry("/".to_string()).or_insert(vec![]);
                continue;
            }
            
            
            let parts: Vec<&str> = route_str.split('/').filter(|s| !s.is_empty()).collect();
            if parts.is_empty() {
                route_tree.entry("/".to_string()).or_insert(vec![]);
            } else {
                let mut current_path = String::new();
                for (i, part) in parts.iter().enumerate() {
                    if i > 0 {
                        current_path.push('/');
                    }
                    current_path.push_str(part);
                    
                    if i == parts.len() - 1 {
                        let route_key = if current_path.is_empty() { 
                            "/".to_string() 
                        } else { 
                            format!("/{}", current_path) 
                        };
                        route_tree.entry(route_key).or_insert(vec![]);
                    }
                }
            }
        }
        
        
        for (route, _) in route_tree.iter_mut() {
            if route.contains('[') && route.contains(']') {
                
            }
        }
        
        
        for (i, (route, _)) in route_tree.iter().enumerate() {
            let is_last = i == route_tree.len() - 1;
            let prefix = if is_last { "└─ " } else { "├─ " };
            
            
            if route.contains('[') && route.contains(']') {
                println!("│ {}{}", prefix.magenta(), route.yellow().bold().italic());
            } else {
                println!("│ {}{}", prefix.magenta(), route.yellow().bold());
            }
        }
    }
    
    println!("{}", format!("╰{}", "─".repeat(50)).magenta());
    Ok(())
}

fn list_components(app_path: &PathBuf, config: &Value) -> Result<()> {
    println!("\n{}", format!("╭─ COMPONENTS {}", "─".repeat(39)).bright_blue().bold());
    
    let component_dir = if let Some(dirs) = config.get("directories") {
        if let Some(dir) = dirs.get("componentDir") {
            dir.as_str().unwrap_or("app/components").to_string()
        } else {
            "app/components".to_string()
        }
    } else {
        "app/components".to_string()
    };
    
    let project_dir = app_path.parent().unwrap_or(app_path);
    let components_path = project_dir.join(&component_dir);
    
    if !components_path.exists() {
        println!("│  {}", "(none)".dimmed());
        println!("{}", format!("╰{}", "─".repeat(50)).bright_blue());
        return Ok(());
    }
    
    let components = find_routes_in_directory(&components_path, ".html", |_| true)?;
    
    if components.is_empty() {
        println!("│  {}", "(none)".dimmed());
    } else {
        for (i, component_path) in components.iter().enumerate() {
            let is_last = i == components.len() - 1;
            let prefix = if is_last { "└─ " } else { "├─ " };
            
            let component_name = component_path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
                
            println!("│ {}{}", prefix.bright_blue(), component_name.bright_green().bold());
        }
    }
    
    println!("{}", format!("╰{}", "─".repeat(50)).bright_blue());
    Ok(())
}

fn find_routes_in_directory<F>(base_dir: &PathBuf, target_file: &str, filter: F) -> Result<Vec<PathBuf>>
where
    F: Fn(&PathBuf) -> bool,
{
    let mut result = Vec::new();
    let mut dirs_to_scan = vec![base_dir.clone()];
    
    while let Some(current_dir) = dirs_to_scan.pop() {
        if !current_dir.is_dir() {
            continue;
        }
        
        for entry in fs::read_dir(&current_dir).map_err(|e| GoaError::Io(e))? {
            let entry = entry.map_err(|e| GoaError::Io(e))?;
            let path = entry.path();
            
            if path.is_dir() {
                dirs_to_scan.push(path);
            } else if path.file_name()
                      .and_then(|n| n.to_str())
                      .map(|n| n == target_file)
                      .unwrap_or(false) && filter(&path) {
                result.push(path);
            }
        }
    }
    
    Ok(result)
}

fn find_config_file() -> Result<PathBuf> {
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
    
    Err(GoaError::Configuration("Could not find config.json file. Are you inside a Go on Airplanes project?".to_string()).into())
}

fn configure_project() -> Result<()> {
    utils::log_step("Configuring Go on Airplanes project");
    
    let config_path = find_config_file()?;
    let config_str = fs::read_to_string(&config_path)
        .map_err(|e| GoaError::Io(e))?;
    
    let mut config: Value = serde_json::from_str(&config_str)
        .map_err(|e| GoaError::Json(e))?;
    
    let categories = vec![
        "Server Settings",
        "Directory Paths",
        "Performance",
        "Static Site Generation (SSG)",
        "Meta Information",
        "Save and Exit"
    ];
    
    loop {
        println!("\n{}", "PROJECT CONFIGURATION".bold().underline());
        println!("Choose a category to configure:\n");
        
        for (i, category) in categories.iter().enumerate() {
            println!("  {}. {}", (i + 1).to_string().cyan(), category);
        }
        
        let selection = utils::prompt_input("Select category (1-6)", Some("6".to_string()))?;
        let selection = selection.parse::<usize>().unwrap_or(6);
        
        match selection {
            1 => configure_server_settings(&mut config)?,
            2 => configure_directory_paths(&mut config)?,
            3 => configure_performance(&mut config)?,
            4 => configure_ssg(&mut config)?,
            5 => configure_meta(&mut config)?,
            _ => break,
        }
    }
    
    let updated_config = serde_json::to_string_pretty(&config)
        .map_err(|e| GoaError::Json(e))?;
    
    fs::write(&config_path, updated_config)
        .map_err(|e| GoaError::Io(e))?;
    
    utils::log_success("Configuration saved successfully");
    Ok(())
}

fn configure_server_settings(config: &mut Value) -> Result<()> {
    println!("\n{}", "SERVER SETTINGS".bold().underline());
    
    let server = config.get_mut("server").and_then(|s| s.as_object_mut());
    
    if let Some(server) = server {
        
        let current_port = server.get("port").and_then(|v| v.as_str()).unwrap_or("5000");
        let new_port = utils::prompt_input("Port", Some(current_port.to_string()))?;
        server.insert("port".to_string(), json!(new_port));
        
        
        let current_dev_mode = server.get("devMode").and_then(|v| v.as_bool()).unwrap_or(true);
        let new_dev_mode = utils::prompt_confirm("Enable development mode", current_dev_mode)?;
        server.insert("devMode".to_string(), json!(new_dev_mode));
        
        
        let current_built = server.get("isBuiltSystem").and_then(|v| v.as_bool()).unwrap_or(false);
        let new_built = utils::prompt_confirm("Is built system", current_built)?;
        server.insert("isBuiltSystem".to_string(), json!(new_built));
        
        
        let current_live_reload = server.get("liveReload").and_then(|v| v.as_bool()).unwrap_or(true);
        let new_live_reload = utils::prompt_confirm("Enable live reload", current_live_reload)?;
        server.insert("liveReload".to_string(), json!(new_live_reload));
        
        
        let current_cors = server.get("enableCORS").and_then(|v| v.as_bool()).unwrap_or(false);
        let new_cors = utils::prompt_confirm("Enable CORS", current_cors)?;
        server.insert("enableCORS".to_string(), json!(new_cors));
        
        
        let current_rate_limit = server.get("rateLimit").and_then(|v| v.as_u64()).unwrap_or(100);
        let new_rate_limit = utils::prompt_input("Rate limit", Some(current_rate_limit.to_string()))?;
        let new_rate_limit = new_rate_limit.parse::<u64>().unwrap_or(100);
        server.insert("rateLimit".to_string(), json!(new_rate_limit));
        
        utils::log_success("Server settings updated");
    } else {
        utils::log_error("Server configuration section not found");
    }
    
    Ok(())
}

fn configure_directory_paths(config: &mut Value) -> Result<()> {
    println!("\n{}", "DIRECTORY PATHS".bold().underline());
    
    let directories = config.get_mut("directories").and_then(|d| d.as_object_mut());
    
    if let Some(directories) = directories {
        
        let current_app_dir = directories.get("appDir").and_then(|v| v.as_str()).unwrap_or("app");
        let new_app_dir = utils::prompt_input("App directory", Some(current_app_dir.to_string()))?;
        directories.insert("appDir".to_string(), json!(new_app_dir));
        
        
        let current_static_dir = directories.get("staticDir").and_then(|v| v.as_str()).unwrap_or("static");
        let new_static_dir = utils::prompt_input("Static directory", Some(current_static_dir.to_string()))?;
        directories.insert("staticDir".to_string(), json!(new_static_dir));
        
        
        let current_layout = directories.get("layoutPath").and_then(|v| v.as_str()).unwrap_or("app/layout.html");
        let new_layout = utils::prompt_input("Layout path", Some(current_layout.to_string()))?;
        directories.insert("layoutPath".to_string(), json!(new_layout));
        
        
        let current_component_dir = directories.get("componentDir").and_then(|v| v.as_str()).unwrap_or("app/components");
        let new_component_dir = utils::prompt_input("Component directory", Some(current_component_dir.to_string()))?;
        directories.insert("componentDir".to_string(), json!(new_component_dir));
        
        utils::log_success("Directory paths updated");
    } else {
        utils::log_error("Directories configuration section not found");
    }
    
    Ok(())
}

fn configure_performance(config: &mut Value) -> Result<()> {
    println!("\n{}", "PERFORMANCE SETTINGS".bold().underline());
    
    let performance = config.get_mut("performance").and_then(|p| p.as_object_mut());
    
    if let Some(performance) = performance {
        
        let current_template_cache = performance.get("templateCache").and_then(|v| v.as_bool()).unwrap_or(true);
        let new_template_cache = utils::prompt_confirm("Enable template cache", current_template_cache)?;
        performance.insert("templateCache".to_string(), json!(new_template_cache));
        
        
        let current_in_memory_js = performance.get("inMemoryJS").and_then(|v| v.as_bool()).unwrap_or(true);
        let new_in_memory_js = utils::prompt_confirm("Enable in-memory JavaScript", current_in_memory_js)?;
        performance.insert("inMemoryJS".to_string(), json!(new_in_memory_js));
        
        utils::log_success("Performance settings updated");
    } else {
        utils::log_error("Performance configuration section not found");
    }
    
    Ok(())
}

fn configure_ssg(config: &mut Value) -> Result<()> {
    println!("\n{}", "STATIC SITE GENERATION (SSG) SETTINGS".bold().underline());
    
    let ssg = config.get_mut("ssg").and_then(|s| s.as_object_mut());
    
    if let Some(ssg) = ssg {
        
        let current_enabled = ssg.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
        let new_enabled = utils::prompt_confirm("Enable SSG", current_enabled)?;
        ssg.insert("enabled".to_string(), json!(new_enabled));
        
        
        let current_cache = ssg.get("cacheEnabled").and_then(|v| v.as_bool()).unwrap_or(true);
        let new_cache = utils::prompt_confirm("Enable SSG cache", current_cache)?;
        ssg.insert("cacheEnabled".to_string(), json!(new_cache));
        
        
        let current_directory = ssg.get("directory").and_then(|v| v.as_str()).unwrap_or("static/generated");
        let new_directory = utils::prompt_input("SSG output directory", Some(current_directory.to_string()))?;
        ssg.insert("directory".to_string(), json!(new_directory));
        
        utils::log_success("SSG settings updated");
    } else {
        utils::log_error("SSG configuration section not found");
    }
    
    Ok(())
}

fn configure_meta(config: &mut Value) -> Result<()> {
    println!("\n{}", "META INFORMATION".bold().underline());
    
    let meta = config.get_mut("meta").and_then(|m| m.as_object_mut());
    
    if let Some(meta) = meta {
        
        let current_app_name = meta.get("appName").and_then(|v| v.as_str()).unwrap_or("Go on Airplanes");
        let new_app_name = utils::prompt_input("Application name", Some(current_app_name.to_string()))?;
        meta.insert("appName".to_string(), json!(new_app_name));
        
        
        let meta_tags = meta.get_mut("defaultMetaTags").and_then(|t| t.as_object_mut());
        
        if let Some(meta_tags) = meta_tags {
            println!("\n{}", "Default Meta Tags:".bold());
            
            
            let current_viewport = meta_tags.get("viewport").and_then(|v| v.as_str()).unwrap_or("width=device-width, initial-scale=1.0");
            let new_viewport = utils::prompt_input("Viewport", Some(current_viewport.to_string()))?;
            meta_tags.insert("viewport".to_string(), json!(new_viewport));
            
            
            let current_desc = meta_tags.get("description").and_then(|v| v.as_str()).unwrap_or("Go on Airplanes - A modern Go web framework");
            let new_desc = utils::prompt_input("Description", Some(current_desc.to_string()))?;
            meta_tags.insert("description".to_string(), json!(new_desc));
            
            
            let current_og_title = meta_tags.get("og:title").and_then(|v| v.as_str()).unwrap_or(&new_app_name);
            let new_og_title = utils::prompt_input("Open Graph title", Some(current_og_title.to_string()))?;
            meta_tags.insert("og:title".to_string(), json!(new_og_title));
            
            
            let current_og_type = meta_tags.get("og:type").and_then(|v| v.as_str()).unwrap_or("website");
            let new_og_type = utils::prompt_input("Open Graph type", Some(current_og_type.to_string()))?;
            meta_tags.insert("og:type".to_string(), json!(new_og_type));
            
            
            let current_twitter = meta_tags.get("twitter:card").and_then(|v| v.as_str()).unwrap_or("summary");
            let new_twitter = utils::prompt_input("Twitter card type", Some(current_twitter.to_string()))?;
            meta_tags.insert("twitter:card".to_string(), json!(new_twitter));
        }
        
        utils::log_success("Meta information updated");
    } else {
        utils::log_error("Meta configuration section not found");
    }
    
    Ok(())
}

fn build_project(output_dir: Option<String>) -> Result<()> {
    utils::log_step("Building Go on Airplanes project for production");
    
    
    let config_path = find_config_file()?;
    
    
    let project_dir = config_path.parent().unwrap().to_path_buf();
    
    
    let temp_config_path = project_dir.join("temp_production_config.json");
    
    
    let config_str = fs::read_to_string(&config_path)
        .map_err(|e| GoaError::Io(e))?;
    
    let mut config: Value = serde_json::from_str(&config_str)
        .map_err(|e| GoaError::Json(e))?;
    
    
    let server = config.get_mut("server").and_then(|s| s.as_object_mut());
    if let Some(server) = server {
        
        server.insert("devMode".to_string(), json!(false));
        
        
        server.insert("liveReload".to_string(), json!(false));
        
        
        server.insert("isBuiltSystem".to_string(), json!(true));
    } else {
        utils::log_error("Server configuration section not found");
        return Err(GoaError::Configuration("Server configuration section not found".to_string()).into());
    }
    
    
    let production_config = serde_json::to_string_pretty(&config)
        .map_err(|e| GoaError::Json(e))?;
    
    fs::write(&temp_config_path, &production_config)
        .map_err(|e| GoaError::Io(e))?;
    
    
    let backup_config_path = project_dir.join("config.json.bak");
    fs::copy(&config_path, &backup_config_path)
        .map_err(|e| GoaError::Io(e))?;
    
    
    fs::copy(&temp_config_path, &config_path)
        .map_err(|e| GoaError::Io(e))?;
    
    
    fs::remove_file(&temp_config_path).ok();
    
    utils::log_success("Temporarily updated config for production build");
    
    
    let target_dir = match output_dir {
        Some(dir) => PathBuf::from(dir),
        None => project_dir.join("build"),
    };
    
    
    if !target_dir.exists() {
        fs::create_dir_all(&target_dir)
            .map_err(|e| GoaError::Io(e))?;
    }
    
    
    let main_go_path = project_dir.join("main.go");
    if !main_go_path.exists() {
        utils::log_error("main.go not found in project directory");
        
        
        fs::copy(&backup_config_path, &config_path)
            .map_err(|e| GoaError::Io(e))?;
        fs::remove_file(&backup_config_path).ok();
        
        return Err(GoaError::ProjectCreation("main.go not found in project directory".to_string()).into());
    }
    
    
    utils::log_step("Running build process...");
    
    let executable_name = if cfg!(windows) { "app.exe" } else { "app" };
    let output_path = target_dir.join(executable_name);
    
    let build_result = Command::new("go")
        .args([
            "build",
            "-o", 
            &output_path.to_string_lossy()
        ])
        .current_dir(&project_dir)
        .output();
    
    
    fs::copy(&backup_config_path, &config_path)
        .map_err(|e| GoaError::Io(e))?;
    fs::remove_file(&backup_config_path).ok();
    
    utils::log_success("Restored original configuration");
    
    
    match build_result {
        Ok(output) => {
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                utils::log_error(&format!("Build failed: {}", error));
                return Err(GoaError::ProjectCreation(format!("Build failed: {}", error)).into());
            }
            
            utils::log_success("Build completed successfully!");
            
            
            let mut prod_config: Value = serde_json::from_str(&config_str)
                .map_err(|e| GoaError::Json(e))?;
            
            
            if !prod_config.as_object().unwrap().contains_key("server") {
                prod_config["server"] = json!({});
            }
            
            
            if let Some(server) = prod_config.get_mut("server").and_then(|s| s.as_object_mut()) {
                server.insert("devMode".to_string(), json!(false));
                server.insert("liveReload".to_string(), json!(false)); 
                server.insert("isBuiltSystem".to_string(), json!(true));
            }
            
            let prod_config_str = serde_json::to_string_pretty(&prod_config)
                .map_err(|e| GoaError::Json(e))?;
                
            fs::write(target_dir.join("config.json"), prod_config_str)
                .map_err(|e| GoaError::Io(e))?;
                
            utils::log_success("Saved production config.json to build directory");
            
            
            println!("\n{}", "╭───────────────────────────────────────────────────╮".cyan());
            println!("{}{:^53}{}", "│".cyan(), "BUILD COMPLETED SUCCESSFULLY".green().bold(), "│".cyan());
            println!("{}", "╰───────────────────────────────────────────────────╯".cyan());
            
            utils::log_info(&format!("Build output: {}", target_dir.display()));
            utils::log_info(&format!("Executable: {}", output_path.display()));
            
            Ok(())
        },
        Err(e) => {
            utils::log_error(&format!("Failed to run build: {}", e));
            Err(GoaError::ProjectCreation(format!("Failed to run build: {}", e)).into())
        }
    }
} 