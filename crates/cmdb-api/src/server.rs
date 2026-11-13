//! Startup: read one export, parse it once, then serve until shutdown.

use crate::{AppState, Config, ConfigError, build_router};
use cmdb_view::{Snapshot, ViewError};
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Parse arguments, load the snapshot, and serve the read-only API.
pub async fn run<I>(args: I) -> Result<(), StartupError>
where
    I: IntoIterator<Item = String>,
{
    let config = Config::parse(args)?;
    let snapshot = load_snapshot(&config.export)?;
    let state = AppState::new(snapshot);
    let listener = tokio::net::TcpListener::bind(config.bind)
        .await
        .map_err(StartupError::Bind)?;
    let local = listener.local_addr().map_err(StartupError::Bind)?;
    log_ready(&config, state.fingerprint(), local);
    axum::serve(listener, build_router(state))
        .await
        .map_err(StartupError::Serve)
}

/// Everything that can stop startup, reported as one structured JSON line.
#[derive(Debug, Error)]
pub enum StartupError {
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error("could not read export {path}: {source}")]
    Read { path: PathBuf, source: io::Error },
    #[error("export parse failed: {0}")]
    Parse(#[from] ViewError),
    #[error("could not bind the listener: {0}")]
    Bind(io::Error),
    #[error("server terminated with an error: {0}")]
    Serve(io::Error),
}

impl StartupError {
    /// Stable machine-readable category.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Config(_) => "configuration",
            Self::Read { .. } => "read_export",
            Self::Parse(_) => "parse_export",
            Self::Bind(_) => "bind",
            Self::Serve(_) => "serve",
        }
    }

    /// Whether this is the successful `--help` path rather than a failure.
    pub fn is_help(&self) -> bool {
        matches!(self, Self::Config(ConfigError::Help))
    }
}

/// Print `{"event":"startup_failed",...}` to stderr.
pub fn report(error: &StartupError) {
    let payload = serde_json::json!({
        "event": "startup_failed",
        "kind": error.kind(),
        "message": error.to_string(),
    });
    if let Ok(line) = serde_json::to_string(&payload) {
        eprintln!("{line}");
    }
}

fn load_snapshot(path: &Path) -> Result<Snapshot, StartupError> {
    let bytes = std::fs::read(path).map_err(|source| StartupError::Read {
        path: path.to_owned(),
        source,
    })?;
    Snapshot::parse(&bytes).map_err(StartupError::Parse)
}

fn log_ready(config: &Config, fingerprint: &str, local: std::net::SocketAddr) {
    let payload = serde_json::json!({
        "event": "ready",
        "bind": local.to_string(),
        "export": config.export.display().to_string(),
        "fingerprint": fingerprint,
    });
    if let Ok(line) = serde_json::to_string(&payload) {
        println!("{line}");
    }
}
