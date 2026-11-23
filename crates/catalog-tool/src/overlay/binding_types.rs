use super::{MemberPath, OperationName};

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Binding {
    Context {
        operation: OperationName,
        source: super::ContextSource,
        path: MemberPath,
    },
    FromBranch {
        operation: OperationName,
        branch: String,
        path: MemberPath,
    },
    ContextFromBranch {
        operation: OperationName,
        branch: String,
        source: super::ContextSource,
        path: MemberPath,
    },
    From {
        operation: OperationName,
        path: MemberPath,
    },
    FromArn {
        operation: OperationName,
        path: MemberPath,
        arn_service: String,
        resource_kind: super::ArnResourceKind,
    },
    FromUrl {
        operation: OperationName,
        path: MemberPath,
        resource_kind: super::UrlResourceKind,
    },
    FromService {
        service: String,
        operation: OperationName,
        path: MemberPath,
    },
    FromAny(Vec<super::BindingAlternative>),
    Transformed {
        binding: Box<Binding>,
        transform: super::BindingTransform,
    },
    Account,
    Region,
    RegionList,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Batch {
    pub member: MemberPath,
    pub max: i64,
    pub replaces: OperationName,
}

impl Batch {
    #[must_use]
    pub const fn new(member: MemberPath, max: i64, replaces: OperationName) -> Self {
        Self {
            member,
            max,
            replaces,
        }
    }
}
