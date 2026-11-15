mod builtin;
mod check;
mod error;
mod policy;
mod fixed_input;
mod capabilities;
mod map_key_exclusion;
pub use map_key_exclusion::MapKeyExclusion;
pub use fixed_input::FixedValue;
pub use capabilities::{ResourceProjection, KindClassification};
mod selection;
pub use selection::{ContextSource, Emission, SelectionKind, IdentityField, IdentityLeaf,
    IdentityPredicate, IdentityBranch, IdentityChoice, BindingAlternative, BindingTransform, ArnResourceKind, UrlResourceKind};
mod tag_endpoint;
pub use tag_endpoint::{TagEndpoint, TagEndpointParent, TagEndpointRequest, TagEndpointSelector};
mod routing;
pub use routing::{ExecutionBranch, RequestRouting};
mod types;
mod row_policy;
pub use row_policy::{RowPolicy, RowPredicate, RowOperand, RowValueType, MissingSelected, ChildEligibility};

pub use check::{
    CallableOperation, CheckError, CheckedOverlays, EffectiveClass, ReviewDisposition,
    ReviewedDecision, ReviewedService, Violation, ViolationRule, build_report, check, check_policies, run,
};
pub use error::NameError;
pub use types::{
    Batch, Binding, Identity, ItemSelector, MemberPath, OperationName,
    OperationOverlay, OverlayClass, Paginator, Scope, ServiceOverlay, TokenPair,
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
