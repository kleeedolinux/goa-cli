use thiserror::Error;

#[derive(Error, Debug)]
pub enum GoaError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Project creation failed: {0}")]
    ProjectCreation(String),

    #[error("Route generation failed: {0}")]
    RouteGeneration(String),

    #[error("Component generation failed: {0}")]
    ComponentGeneration(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("{0}")]
    Other(String),
}

pub type GoaResult<T> = Result<T, GoaError>; 