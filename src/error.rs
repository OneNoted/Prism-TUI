use thiserror::Error;

#[derive(Error, Debug)]
pub enum PrismError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("NBT parse error: {0}")]
    Nbt(#[from] hematite_nbt::Error),

    #[error("Config parse error: {0}")]
    Config(String),

    #[error("PrismLauncher data directory not found")]
    DataDirNotFound,

    #[error("Launch failed: {0}")]
    LaunchFailed(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, PrismError>;
