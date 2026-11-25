#[path = "postgres_support/pages.rs"]
mod pages;
mod postgres_support;

#[path = "postgres_page/facets.rs"]
mod facets;
#[path = "postgres_page/scalars.rs"]
mod scalars;
#[path = "postgres_page/scale.rs"]
mod scale;
#[path = "postgres_page/tags.rs"]
mod tags;
#[path = "postgres_page/transactions.rs"]
mod transactions;

use postgres_support::{TestResult, registry};
