use super::*;
use crate::overlay::{ExecutionBranch, FixedValue};
use std::collections::BTreeMap;

pub(super) fn apply(service: &mut ServiceOverlay) -> Result<(), NameError> {
    let scope = if service.service == "waf" { Scope::Account } else { Scope::Region };
    let mut logging = inventory("LoggingConfigurations", Identity::Arn(path("ResourceArn")?), token("NextMarker", "NextMarker")?, scope)?;
    logging.fixed.insert(path("Limit")?, FixedValue::Integer(100));
    insert(service, "ListLoggingConfigurations", logging)?;
    if service.service == "waf" { return Ok(()); }
    let mut associations = child("ResourceArns", correlated(&["WebACLId", "ResourceType"], &["$"])?, "ListWebACLs", "WebACLId", "WebACLId", false)?;
    associations.branches = ["APPLICATION_LOAD_BALANCER", "API_GATEWAY"].into_iter().map(|selector| {
        Ok(ExecutionBranch { discriminator: selector.to_owned(),
            fixed: BTreeMap::from([(path("ResourceType")?, FixedValue::String(selector.to_owned()))]),
            scope: Scope::Region, request_region: None, bind: BTreeMap::new(), emission: None, row_policy: None })
    }).collect::<Result<Vec<_>, NameError>>()?;
    insert(service, "ListResourcesForWebACL", associations)
}
