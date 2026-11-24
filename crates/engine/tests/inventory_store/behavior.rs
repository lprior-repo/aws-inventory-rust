#[path = "behavior/lifecycle.rs"]
mod lifecycle;
#[path = "behavior/outcome_detail.rs"]
mod outcome_detail;
use inventory_core::inventory::{Edge, EdgeSource, Relation, ScopeOutcome};
use inventory_engine::inventory::{BinaryStore, StoreError, SweepId, WorkSpec};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

use super::support::*;
