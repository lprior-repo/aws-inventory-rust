mod binding_types;
mod directory;
pub mod generate;
mod merge;
mod parse;
mod skip;
mod skip_consumers;
pub use binding_types::{Batch, Binding, GenerationBlocker};
pub use directory::{OverlayError, load, load_generated_dir};
pub use skip::SkipEvidence;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    Hand,
    Generated,
}

mod builtin;
mod check;
pub(crate) use check::orchestration::{check_model_policies, read_model_values};
pub(crate) use generate::selection::deny_text;
mod emission;
mod error;
mod policy;
pub use emission::{MAX_RESOURCE_EMISSIONS, ResourceEmission};
mod capabilities;
mod fixed_input;
mod map_key_exclusion;
pub use capabilities::{KindClassification, ResourceProjection};
pub use fixed_input::FixedValue;
pub use map_key_exclusion::MapKeyExclusion;
mod selection;
pub use selection::{
    ArnResourceKind, BindingAlternative, BindingTransform, ContextSource, Emission, IdentityBranch,
    IdentityChoice, IdentityField, IdentityLeaf, IdentityPredicate, SelectionKind, UrlResourceKind,
};
mod tag_endpoint;
pub use tag_endpoint::{TagEndpoint, TagEndpointParent, TagEndpointRequest, TagEndpointSelector};
mod routing;
pub use routing::{ExecutionBranch, RequestRouting};
mod row_policy;
mod types;
pub use row_policy::{
    ChildEligibility, MissingSelected, RowOperand, RowPolicy, RowPredicate, RowValueType,
};
mod emit_rule;
pub use emit_rule::{ExpansionBinding, expansion_conditions_met, infer_work_only};

pub use check::{
    CallableOperation, CheckError, CheckedOverlays, EffectiveClass, ReviewDisposition,
    ReviewedDecision, ReviewedService, Violation, ViolationRule, build_report, check,
    check_policies, run,
};
pub use error::NameError;
pub use types::{
    Identity, ItemSelector, MemberPath, OperationName, OperationOverlay, OverlayClass, Paginator,
    ReviewEvidence, Scope, ServiceOverlay, TokenPair,
};

pub fn embedded() -> Result<Vec<ServiceOverlay>, NameError> {
    builtin::overlays()
}

pub const MAX_FILE_BYTES: usize = 1_048_576;

pub const MAX_OPERATIONS: usize = 4_096;

fn valid_member(text: &str) -> bool {
    text.starts_with(|character: char| character.is_ascii_alphabetic())
        && text
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn valid_path(text: &str) -> bool {
    text.len() <= 256 && text.split('.').count() <= 8 && text.split('.').all(valid_member)
}
