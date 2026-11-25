use super::{MemberPath, NameError, OperationName};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
pub enum IdentityField {
    Selected(MemberPath),
    SelectedDigest(MemberPath),
    Request(MemberPath),
    SelectedArn(MemberPath),
    RequestArn(MemberPath),
    SelectedInteger {
        path: MemberPath,
        minimum: i64,
        maximum: i64,
    },
    RequestInteger {
        path: MemberPath,
        minimum: i64,
        maximum: i64,
    },
    SelectedStringList {
        path: MemberPath,
        max_items: u32,
        max_item_bytes: u32,
    },
    Context {
        operation: OperationName,
        source: ContextSource,
        path: MemberPath,
    },
    MapValue {
        path: MemberPath,
        key: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IdentityLeaf {
    Arn(MemberPath),
    /// An ARN whose account segment need not be the scanning account. The
    /// scanning account owns the resource only when the owner member of the
    /// same row names it.
    OwnedArn {
        path: MemberPath,
        owner: MemberPath,
        resource_kind: ArnResourceKind,
    },
    Template(String),
    Key(Vec<MemberPath>),
    Fields(Vec<IdentityField>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IdentityPredicate {
    Present(MemberPath),
    TextEquals { path: MemberPath, value: String },
    RequestTextEquals { path: MemberPath, value: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdentityBranch {
    pub predicate: IdentityPredicate,
    pub identity: IdentityLeaf,
}

impl IdentityBranch {
    pub fn new(predicate: IdentityPredicate, identity: IdentityLeaf) -> Result<Self, NameError> {
        if let IdentityPredicate::TextEquals { value, .. }
        | IdentityPredicate::RequestTextEquals { value, .. } = &predicate
        {
            valid_text(value, 4096)?;
        }
        validate_leaf(&identity)?;
        Ok(Self {
            predicate,
            identity,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IdentityChoice {
    PreferArn {
        path: MemberPath,
        fallback: IdentityLeaf,
        review: String,
    },
    Discriminated(Vec<IdentityBranch>),
}

impl IdentityChoice {
    pub fn validate(&self) -> Result<(), NameError> {
        match self {
            Self::PreferArn {
                fallback, review, ..
            } => {
                valid_text(review, 1024)?;
                validate_leaf(fallback)
            }
            Self::Discriminated(branches) => {
                if branches.is_empty() || branches.len() > 16 {
                    return Err(NameError {
                        value: "identity branches".to_owned(),
                    });
                }
                branches.iter().try_for_each(|branch| {
                    IdentityBranch::new(branch.predicate.clone(), branch.identity.clone())
                        .map(|_| ())
                })
            }
        }
    }
}

pub(super) fn validate_leaf(leaf: &IdentityLeaf) -> Result<(), NameError> {
    match leaf {
        IdentityLeaf::Arn(_) => Ok(()),
        IdentityLeaf::OwnedArn { resource_kind, .. }
            if !matches!(resource_kind, ArnResourceKind::SqsQueue) =>
        {
            Ok(())
        }
        IdentityLeaf::Template(template) => valid_text(template, 4096),
        IdentityLeaf::Key(fields) if !fields.is_empty() && fields.len() <= 8 => Ok(()),
        IdentityLeaf::Fields(fields) => validate_fields(fields),
        _ => Err(NameError {
            value: "identity fields".to_owned(),
        }),
    }
}

pub(super) fn validate_fields(fields: &[IdentityField]) -> Result<(), NameError> {
    if fields.is_empty() || fields.len() > 64 {
        return Err(NameError {
            value: "identity fields".to_owned(),
        });
    }
    fields.iter().try_for_each(|field| match field {
        IdentityField::MapValue { key, .. } => valid_text(key, 4096),
        IdentityField::SelectedInteger {
            minimum, maximum, ..
        }
        | IdentityField::RequestInteger {
            minimum, maximum, ..
        } if minimum > maximum => Err(NameError {
            value: "identity integer bounds".to_owned(),
        }),
        IdentityField::SelectedStringList {
            max_items,
            max_item_bytes,
            ..
        } if !(1..=64).contains(max_items) || !(1..=4096).contains(max_item_bytes) => {
            Err(NameError {
                value: "identity string list bounds".to_owned(),
            })
        }
        _ => Ok(()),
    })
}

fn valid_text(text: &str, bound: usize) -> Result<(), NameError> {
    if text.is_empty() || text.len() > bound || text.chars().any(char::is_control) {
        return Err(NameError {
            value: text.to_owned(),
        });
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BindingTransform {
    StripPrefix(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BindingAlternative {
    pub operation: OperationName,
    pub source: ContextSource,
    pub path: MemberPath,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArnResourceKind {
    SnsSubscription,
    SqsQueue,
    SnsTopic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UrlResourceKind {
    SqsQueue,
}
