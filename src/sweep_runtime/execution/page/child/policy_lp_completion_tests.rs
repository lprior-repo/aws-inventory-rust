mod fixture;
mod children;
mod roots;
mod proton;
mod boundaries;
mod metadata;

use fixture::*;
use inventory_core::catalog::runtime::RuntimeCatalog;
use serde_json::{Value as Json, json};
use std::collections::BTreeSet;
