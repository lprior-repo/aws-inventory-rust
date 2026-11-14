//! # Reviewed service overlays: format 1
//!
//! One `catalog/overlay/<service>.toml` per reviewed service. Service stems match
//! `[a-z0-9][a-z0-9-]*`. Presence records review; it does not alone grant permission.
//! Files are at most 1 MiB with at most 4,096 operations. Directory loading visits
//! at most 65,536 entries and accepts at most 4,096 TOML files, in filename order.
//! Non-TOML entries below the traversal cap are ignored, including `must-deny.txt`.
//! Distinct filenames cannot share a stem with the exact `.toml` extension.
//!
//! Required root keys: `format` is integer `1`; `reviewed` is a quoted real
//! Gregorian `YYYY-MM-DD` date (years 0001..9999); `reviewer` is a nonempty string
//! of at most 64 UTF-8 bytes. Optional `operation` is a table keyed by exact
//! `[A-Za-z][A-Za-z0-9]*` names; absence means no explicit overrides.
//!
//! ## Operation keys and absence semantics
//!
//! Unknown keys and wrong types are errors at every structured table boundary.
//! Member paths use `[A-Za-z][A-Za-z0-9_]*` segments separated by dots, at most
//! eight segments and 256 bytes. Lists/maps are traversed implicitly, not indexed.
//! All arrays reject duplicates and contain at most 64 entries; tighter bounds
//! below take precedence. `fixed` and `bind` maps are bounded by the file budget.
//!
//! | Key | Type and bounds | When absent |
//! |---|---|---|
//! | `class` | `enumerate`, `expand`, `deny`, `defer` string | Model-derived class |
//! | `deny_reason` | `content`, `secret`, `paid`, `mutating`, `reference-data` | Required exactly for `deny`; forbidden otherwise |
//! | `scope` | `region`, `account`, `home-region` string | Model-derived scope |
//! | `items` | Output member path string | Model-derived list; otherwise required before callable |
//! | `page_size` | Integer 1..=100000 | Largest model-derived allowed size |
//! | `exclude_from_hash` | Array of item member paths | No additional exclusions beyond model-derived values |
//! | `skip` | Array of operation names, not self | No additional skips beyond model-derived values |
//! | `quota_group` | `[a-z][a-z0-9-]*` string, <=32 bytes | Service-wide quota group |
//! | `queryable` | Array of item member paths | Model-derived SQL columns |
//! | `tags` | Item member path string | Model-derived tags; otherwise required before callable when tags needed |
//! | `fixed` | Input member path -> string, i64 integer, boolean, or string array | No additional fixed parameters; required inputs must be resolved before callable |
//! | `paginator` | Table described below | Model-derived paginator; otherwise reviewed override required before callable |
//! | `identity` | Table described below | Model-derived identity; otherwise required before callable |
//! | `bind` | Input member path -> binding table below | Model-derived bindings; unresolved required inputs prevent callability |
//! | `batch` | Table described below | Model-derived batch read, otherwise no batch replacement |
//!
//! `deny`/`defer` entries carry no plan keys, including explicitly empty tables or
//! arrays. `deny_reason` is stored in the typed `OverlayClass::Deny` variant.
//!
//! ## Nested tables
//!
//! `paginator.kind` is required: `token` requires `input` and `output` member paths
//! and forbids other keys; `tokens` requires `pairs` (1..=8 `{input, output}` tables,
//! distinct input paths) and optionally `truncated` (output boolean member path;
//! absence means no truncation-flag override); `none` forbids every other key.
//!
//! `identity` contains exactly one of `arn` (item member path), `template` (string),
//! or `key` (1..=8 item paths). Templates require balanced, nonempty placeholders:
//! `partition`, `region`, `account` or an item member path; at least one is required.
//!
//! A `bind` value is exactly `{from = "Operation", path = "Item.Path"}`,
//! `{account = true}`, or `{region = true}`. Context flags cannot be false. Self
//! bindings and in-file cycles are rejected iteratively, bounded by operation count.
//! `batch` requires `member` (input list member path), `max` (integer 1..=10000),
//! and `replaces` (operation name, not self). No nested required key has a default.
//!
//! ## Model-aware checks belong to the next package
//!
//! Operation existence, path resolution, reviewed-service completeness and callable
//! plan compilation are not checked here. An override may deny anything, but may
//! only make callable a read-prefixed operation not denied by the must-deny list.
//! Every reviewed-service candidate must then be complete, denied or deferred.
//!
//! ## Complete example (parsed by `documented_example_has_exact_typed_fields`)
//!
//! The illustrative proposal placed `deny_reason` beside `enumerate` to show the
//! key; that combination is invalid. The example places it on a denied operation.
#![doc = concat!("```toml\n", include_str!("../tests/fixtures/overlay/complete.toml"), "\n```\n")]

mod directory;
mod error;
mod fields;
mod operation;
mod pagination;
mod parse;
mod plan;
mod raw;
mod types;

pub use directory::load_dir;
pub use error::{NameError, OverlayError, Rule};
pub use types::{
    Batch, Binding, FixedValue, Identity, MemberPath, OperationName, OperationOverlay,
    OverlayClass, Paginator, Scope, ServiceOverlay, TokenPair,
};

/// Inclusive maximum UTF-8 bytes in one overlay, checked before TOML parsing.
pub const MAX_FILE_BYTES: usize = 1_048_576;
/// Inclusive maximum operations per file.
pub const MAX_OPERATIONS: usize = 4_096;
/// Inclusive maximum matching TOML files per directory load.
pub const MAX_FILES: usize = 4_096;
/// Inclusive maximum visited directory entries, including ignored files.
pub const MAX_DIRECTORY_ENTRIES: usize = 65_536;

fn valid_member(text: &str) -> bool {
    text.starts_with(|character: char| character.is_ascii_alphabetic())
        && text
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn valid_path(text: &str) -> bool {
    text.len() <= 256 && text.split('.').count() <= 8 && text.split('.').all(valid_member)
}
