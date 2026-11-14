pub mod assertions;
pub mod audit;
pub mod catalog;
pub mod emulator;
mod files;
pub mod install;
pub mod json;
pub mod package;
pub mod provenance;
pub mod proxy;
pub mod seed;
pub mod snapshot;

#[derive(Debug, thiserror::Error)]
pub enum HarnessError {
    #[error("{0}")]
    Invalid(String),

    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Sqlx(#[from] sqlx::Error),
}

impl HarnessError {
    pub fn invalid(reason: impl Into<String>) -> Self {
        Self::Invalid(reason.into())
    }
}
