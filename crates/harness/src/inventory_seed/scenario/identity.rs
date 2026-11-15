use super::{FieldPath, SeedError};
use inventory_core::inventory::Region;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SeedScope {
    Account,
    #[default]
    Regional,
}

impl SeedScope {
    pub(in crate::inventory_seed) fn region(self, configured: &Region) -> Region {
        match self { Self::Account => Region::Global, Self::Regional => configured.clone() }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeedIdentityEvidence { Response, Request }

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SeedIdentity {
    Response(FieldPath),
    Request { input_field: FieldPath, template: Option<String> },
}

impl From<FieldPath> for SeedIdentity {
    fn from(path: FieldPath) -> Self { Self::Response(path) }
}

pub(in crate::inventory_seed) fn validate_template(template: &str) -> Result<(), SeedError> {
    if template.len() > 1024 || !template.contains("{value}") {
        return Err(SeedError::Invalid("identity template must contain {value} and fit its bound"));
    }
    template.split('{').enumerate().try_for_each(|(index, part)| {
        let literal = if index == 0 { part } else {
            let (token, literal) = part.split_once('}')
                .ok_or(SeedError::Invalid("identity template has an unclosed token"))?;
            if !matches!(token, "value" | "account" | "region" | "partition") {
                return Err(SeedError::Invalid("identity template contains an unknown token"));
            }
            literal
        };
        if literal.contains('}') { return Err(SeedError::Invalid("identity template has an unmatched brace")); }
        Ok(())
    })
}
