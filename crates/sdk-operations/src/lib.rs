//! Inventory of every operation exposed by the generated AWS SDK crates.
//!
//! The archive catalog only covers reviewed resource types. This crate reads
//! the pinned crates in the package registry instead, so the operations that
//! exist in the shipped SDK are enumerable without a review subset.

pub mod digest;
pub mod inventory;
pub mod lockfile;
pub mod model;
pub mod registry;

use std::path::PathBuf;

/// Bounded inventory failures. Messages are operator-facing and name the
/// exact crate, path or lockfile entry that broke the contract.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Filesystem failure, annotated with the path that was being read.
    #[error("cannot access {path}: {source}")]
    Io {
        /// Path that could not be accessed.
        path: PathBuf,
        /// Underlying filesystem failure.
        #[source]
        source: std::io::Error,
    },
    /// A directory in the registry is not a valid crate identifier.
    #[error("crate directory `{name}` is not named aws-sdk-<service>-<version>")]
    InvalidCrateDirectory {
        /// Directory name that could not be parsed.
        name: String,
    },
    /// A registry entry has a name that is not valid UTF-8.
    #[error("crate directory {path} has a non-UTF-8 name")]
    NonUtf8CrateName {
        /// Offending directory.
        path: PathBuf,
    },
    /// The same crate name and version appears twice in the registry.
    #[error("crate `{name}-{version}` appears twice in the registry")]
    DuplicateCrate {
        /// Crate package name.
        name: String,
        /// Crate version.
        version: String,
    },
    /// A crate has no `src/operation` directory.
    #[error("crate `{name}` has no src/operation directory at {path}")]
    MissingOperations {
        /// Crate package name.
        name: String,
        /// Expected operation directory.
        path: PathBuf,
    },
    /// A crate has an empty `src/operation` directory.
    #[error("crate `{name}` has no operation modules in {path}")]
    EmptyOperations {
        /// Crate package name.
        name: String,
        /// Operation directory that carried no modules.
        path: PathBuf,
    },
    /// The bounded fingerprint found no files.
    #[error("source fingerprint for {path} is empty")]
    EmptyTree {
        /// Crate root that carried no files.
        path: PathBuf,
    },
    /// The bounded fingerprint exceeded its file or byte budget.
    #[error("source fingerprint for {path} exceeds its bounded file/byte budget")]
    TreeBudget {
        /// Crate root that exceeded the budget.
        path: PathBuf,
    },
    /// A discovered file escaped its crate root.
    #[error("source path {path} escaped its crate root")]
    TreeEscape {
        /// Root the path was expected to stay under.
        path: PathBuf,
    },
    /// The registry carried no `aws-sdk-*` crate directories.
    #[error("registry {path} contains no aws-sdk-* crates")]
    EmptyRegistry {
        /// Registry that was scanned.
        path: PathBuf,
    },
    /// The lockfile had no `aws-sdk-*` packages.
    #[error("Cargo.lock at {path} lists no aws-sdk packages")]
    EmptyLockfile {
        /// Lockfile that was parsed.
        path: PathBuf,
    },
    /// The lockfile pins SDK crates that the registry does not contain.
    #[error(
        "Cargo.lock at {path} lists aws-sdk packages absent from registry {registry}: {missing}"
    )]
    MissingFromRegistry {
        /// Lockfile that was parsed.
        path: PathBuf,
        /// Registry that was scanned.
        registry: PathBuf,
        /// Comma-separated `name-version` list of missing crates.
        missing: String,
    },
    /// The checked-in inventory is byte-different from a fresh render.
    #[error(
        "{path} is stale; regenerate it with `sdk-operations inventory --registry <dir> --out {path}`"
    )]
    StaleOutput {
        /// Checked-in inventory that drifted.
        path: PathBuf,
    },
    /// JSON encoding failed.
    #[error("JSON encoding failed: {0}")]
    Json(#[from] serde_json::Error),
    /// Command line was not a supported invocation.
    #[error("{0}")]
    Usage(String),
}

impl Error {
    /// Attach a path to a filesystem failure.
    pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}
