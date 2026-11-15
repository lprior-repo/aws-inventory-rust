mod check;
mod directory;
mod error;
mod fields;
mod operation;
mod pagination;
mod parse;
mod plan;
mod raw;
mod types;

pub use check::{
    CallableOperation, CheckError, CheckedOverlays, EffectiveClass, ReviewDisposition,
    ReviewedDecision, ReviewedService, Violation, ViolationRule, build_report, check, run,
};
pub use directory::load_dir;
pub use error::{NameError, OverlayError, Rule};
pub use types::{
    Batch, Binding, FixedValue, Identity, ItemSelector, MemberPath, OperationName,
    OperationOverlay, OverlayClass, Paginator, Scope, ServiceOverlay, TokenPair,
};

pub const MAX_FILE_BYTES: usize = 1_048_576;

pub const MAX_OPERATIONS: usize = 4_096;

pub const MAX_FILES: usize = 4_096;

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
