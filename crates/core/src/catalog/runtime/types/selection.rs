use crate::record::FieldId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionKind {
    MapEntries,
    MapKeys,
    MapValues,
    List,
    Structure,
    Scalar,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Emission {
    #[default]
    EmitMetadata,
    DiscoveryOnly,
    WorkOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContextSource {
    SelectedItem,
    Response,
    Request,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IdentityFieldPlan {
    Selected(Box<[FieldId]>),
    SelectedDigest(Box<[FieldId]>),
    Request(Box<[FieldId]>),
    SelectedArn(Box<[FieldId]>),
    RequestArn(Box<[FieldId]>),
    SelectedInteger {
        path: Box<[FieldId]>,
        minimum: i64,
        maximum: i64,
    },
    RequestInteger {
        path: Box<[FieldId]>,
        minimum: i64,
        maximum: i64,
    },
    SelectedStringList {
        path: Box<[FieldId]>,
        max_items: u32,
        max_item_bytes: u32,
    },
    Context {
        service: Box<str>,
        operation: Box<str>,
        source: ContextSource,
        path: Box<[FieldId]>,
    },
    MapValue {
        path: Box<[FieldId]>,
        key: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IdentityLeafPlan {
    Arn(Box<[FieldId]>),
    /// ARN whose account segment may belong to another account; the selected
    /// row's `owner` member names the account that owns the resource.
    OwnedArn {
        path: Box<[FieldId]>,
        owner: Box<[FieldId]>,
        resource_kind: super::ArnResourceKind,
    },
    Template(Box<[super::IdentityPart]>),
    Key(Box<[Box<[FieldId]>]>),
    Fields(Box<[IdentityFieldPlan]>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IdentityPredicatePlan {
    Present(Box<[FieldId]>),
    TextEquals { path: Box<[FieldId]>, value: String },
    RequestTextEquals { path: Box<[FieldId]>, value: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdentityBranchPlan {
    pub predicate: IdentityPredicatePlan,
    pub identity: IdentityLeafPlan,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IdentityChoicePlan {
    PreferArn {
        path: Box<[FieldId]>,
        fallback: IdentityLeafPlan,
        review: String,
    },
    Discriminated(Box<[IdentityBranchPlan]>),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ContextProjectionPlan {
    pub selected: Box<[Box<[FieldId]>]>,
    pub response: Box<[Box<[FieldId]>]>,
    pub request: Box<[Box<[FieldId]>]>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BindingAlternativePlan {
    pub service: Box<str>,
    pub operation: Box<str>,
    pub source: ContextSource,
    pub path: Box<[FieldId]>,
}
