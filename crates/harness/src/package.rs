//! Release packaging and SDK catalog reproduction.
//!
//! Release archives use `catalog-tool`'s single deterministic implementation.
//! `sdk_*` port the offline modes of `scripts/generate_sdk_catalog.py`: strict
//! snapshot validation, deterministic generation and Cargo cache verification.
//! Network refresh is deliberately unreachable and reports a typed failure.

mod sdk_cache;
mod sdk_catalog;
mod sdk_generate;
mod sdk_validate;

pub use catalog_tool::package::{PackageReceipt, package};
pub use sdk_cache::verify_cache as verify_catalog_cache;
pub use sdk_catalog::{
    CacheReport, SdkMode, SdkReport, sdk_check, sdk_refresh, sdk_reproduce, sdk_run, sdk_run_in,
    sdk_verify_cache,
};
