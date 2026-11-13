//! KMS alias mapping. The alias ARN is the identity; the target key stays a
//! property and no alias-to-key edge is inferred without a registered relation.

use std::collections::BTreeMap;

use aws_sdk_kms::types::AliasListEntry;
use inventory_core::Resource;
use serde_json::json;

use super::arn::{is_in_scope, split_alias_arn, validate_target_key_id};
use super::key::{invalid, millis};
use crate::{AwsScanError, common::Context};

pub(super) fn map_alias(ctx: &Context, entry: &AliasListEntry) -> Result<Resource, AwsScanError> {
    let arn = entry
        .alias_arn()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| invalid("ListAliases lacks AliasArn"))?;
    let (parts, alias_name) = split_alias_arn(arn).map_err(invalid)?;
    if !is_in_scope(&parts, ctx) {
        return Err(invalid("KMS alias ARN is outside the scan scope"));
    }
    if let Some(returned) = entry.alias_name().filter(|value| !value.is_empty())
        && returned != alias_name
    {
        return Err(invalid(
            "ListAliases AliasName does not match the alias ARN",
        ));
    }
    let target = entry
        .target_key_id()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| invalid("ListAliases lacks TargetKeyId"))?;
    validate_target_key_id(target, ctx).map_err(invalid)?;
    ctx.resource(
        "aws_kms_alias",
        arn.to_owned(),
        Some(alias_name.to_owned()),
        json!({
            "alias_arn": arn,
            "alias_name": alias_name,
            "target_key_id": target,
            "creation_date": millis(entry.creation_date())?,
            "last_updated_date": millis(entry.last_updated_date())?,
            "tags_collected": false,
        }),
        BTreeMap::new(),
    )
}
