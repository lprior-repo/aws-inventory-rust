use super::{IdentityPart, IdentityPlan, PlanParts, RuntimeCatalogError, scalar, selected, tags, text};
use crate::catalog::runtime::ResourceEmissionPlan;

pub(in crate::catalog::runtime::decode::plans) fn validate_resource(parts: &PlanParts<'_>,
    emission: &ResourceEmissionPlan,
) -> Result<(), RuntimeCatalogError> {
    let item = tags::selected_item(parts.service, parts.operation, emission.items())?;
    match emission.identity() {
        IdentityPlan::Arn(path) => text(parts.service, selected(parts.service, item, path)?),
        IdentityPlan::Key(paths) => paths.iter().try_for_each(|path|
            scalar(parts.service, selected(parts.service, item, path)?)),
        IdentityPlan::Template(fields) => fields.iter().try_for_each(|part| match part {
            IdentityPart::Field(path) => scalar(parts.service, selected(parts.service, item, path)?),
            IdentityPart::Literal(_) | IdentityPart::Partition | IdentityPart::Account | IdentityPart::Region => Ok(()),
        }),
        IdentityPlan::Choice(_) | IdentityPlan::CorrelatedKey(_) | IdentityPlan::MapArn { .. } =>
            super::super::selection::validate_identity(parts, emission),
    }
}
