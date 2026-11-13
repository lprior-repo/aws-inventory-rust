//! Typed Cloud Control resource-type selection.
//!
//! CLI selectors are parsed once into [`Selection`]; collection, planning and
//! reporting APIs accept the enum instead of re-interpreting the magic `all`
//! string or carrying a parallel `universal` boolean.

use std::fmt;

use thiserror::Error;

/// One CloudFormation type name selected for Cloud Control collection.
///
/// This is only the requested name. Whether the pinned catalog can collect it
/// is decided by the catalog-aware validation in `inventory-aws` and by the
/// offline plan catalog, so their existing error messages stay authoritative.
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResourceType(String);

impl ResourceType {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// What a Cloud Control scan collects: explicit types or the whole catalog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Selection {
    /// Exactly the named types; empty means the pinned catalog default.
    Types(Vec<ResourceType>),
    /// Every list-capable type in the pinned universal catalog.
    Universal,
}

impl Selection {
    /// Parse resource-type selectors at the CLI boundary.
    ///
    /// `all` selects the universal catalog and must be the only selector.
    pub fn parse(values: &[String]) -> Result<Self, SelectionError> {
        if !values.iter().any(|value| value == "all") {
            return Ok(Self::Types(
                values
                    .iter()
                    .map(|value| ResourceType::new(value.clone()))
                    .collect(),
            ));
        }
        if values.len() != 1 {
            return Err(SelectionError::MixedUniversal);
        }
        Ok(Self::Universal)
    }

    /// The named types; empty for a universal selection.
    pub fn types(&self) -> &[ResourceType] {
        match self {
            Self::Types(types) => types,
            Self::Universal => &[],
        }
    }

    pub fn is_universal(&self) -> bool {
        matches!(self, Self::Universal)
    }

    pub fn is_empty(&self) -> bool {
        self.types().is_empty()
    }
}

/// A resource-type selector that cannot be interpreted safely.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum SelectionError {
    #[error("`all` must be the only Cloud Control resource-type selector")]
    MixedUniversal,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn selection(values: &[&str]) -> Result<Selection, SelectionError> {
        let values = values
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<Vec<_>>();
        Selection::parse(&values)
    }

    #[test]
    fn empty_selection_is_the_default_catalog() {
        let parsed = selection(&[]);
        assert_eq!(parsed, Ok(Selection::Types(Vec::new())));
    }

    #[test]
    fn explicit_names_keep_their_order_and_duplicates() {
        let parsed = selection(&["AWS::EC2::Instance", "AWS::EKS::Cluster"]);
        let names = parsed
            .map(|selection| {
                selection
                    .types()
                    .iter()
                    .map(|value| value.as_str().to_owned())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        assert_eq!(names, ["AWS::EC2::Instance", "AWS::EKS::Cluster"]);
    }

    #[test]
    fn all_alone_is_universal_and_never_mixes() {
        assert_eq!(selection(&["all"]), Ok(Selection::Universal));
        assert_eq!(
            selection(&["all", "AWS::EC2::Instance"]),
            Err(SelectionError::MixedUniversal)
        );
        assert_eq!(
            selection(&["AWS::EC2::Instance", "all"]),
            Err(SelectionError::MixedUniversal)
        );
    }

    #[test]
    fn universal_has_no_named_types() {
        assert_eq!(
            selection(&["all"]).map(|value| value.is_universal()),
            Ok(true)
        );
        assert_eq!(selection(&["all"]).map(|value| value.is_empty()), Ok(true));
    }
}
