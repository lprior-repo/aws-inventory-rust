mod boundaries;
mod children;
mod fixture;
mod metadata;
mod proton;
mod roots;

use fixture::*;
use inventory_core::catalog::runtime::RuntimeCatalog;
use serde_json::{Value as Json, json};
use std::collections::BTreeSet;
