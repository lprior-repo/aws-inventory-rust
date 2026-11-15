use super::{inventory, path, paths, bind, token};
use crate::overlay::{OperationOverlay, NameError, Identity, Paginator, Scope, FixedValue};

pub(super) fn get_byte_match_set() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "ByteMatchSet",
        Identity::Key(paths(&["ByteMatchSetId"])?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "ByteMatchSetId",
        "Name",
    ])?;
    bind(&mut operation, "ByteMatchSetId", "ListByteMatchSets", "ByteMatchSetId")?;
    Ok(operation)
}

pub(super) fn get_geo_match_set() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "GeoMatchSet",
        Identity::Key(paths(&["GeoMatchSetId"])?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "GeoMatchSetId",
        "Name",
    ])?;
    bind(&mut operation, "GeoMatchSetId", "ListGeoMatchSets", "GeoMatchSetId")?;
    Ok(operation)
}

pub(super) fn get_ip_set() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "IPSet",
        Identity::Key(paths(&["IPSetId"])?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "IPSetId",
        "Name",
    ])?;
    bind(&mut operation, "IPSetId", "ListIPSets", "IPSetId")?;
    Ok(operation)
}

pub(super) fn get_rate_based_rule() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "Rule",
        Identity::Key(paths(&["RuleId"])?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "RuleId",
        "Name",
        "MetricName",
        "RateKey",
        "RateLimit",
    ])?;
    bind(&mut operation, "RuleId", "ListRateBasedRules", "RuleId")?;
    Ok(operation)
}

pub(super) fn get_regex_match_set() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "RegexMatchSet",
        Identity::Key(paths(&["RegexMatchSetId"])?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "RegexMatchSetId",
        "Name",
    ])?;
    bind(&mut operation, "RegexMatchSetId", "ListRegexMatchSets", "RegexMatchSetId")?;
    Ok(operation)
}

pub(super) fn get_regex_pattern_set() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "RegexPatternSet",
        Identity::Key(paths(&["RegexPatternSetId"])?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "RegexPatternSetId",
        "Name",
    ])?;
    bind(&mut operation, "RegexPatternSetId", "ListRegexPatternSets", "RegexPatternSetId")?;
    Ok(operation)
}

pub(super) fn get_rule() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "Rule",
        Identity::Key(paths(&["RuleId"])?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "RuleId",
        "Name",
        "MetricName",
    ])?;
    bind(&mut operation, "RuleId", "ListRules", "RuleId")?;
    Ok(operation)
}

pub(super) fn get_rule_group() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "RuleGroup",
        Identity::Key(paths(&["RuleGroupId"])?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "RuleGroupId",
        "Name",
        "MetricName",
    ])?;
    bind(&mut operation, "RuleGroupId", "ListRuleGroups", "RuleGroupId")?;
    Ok(operation)
}

pub(super) fn get_size_constraint_set() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "SizeConstraintSet",
        Identity::Key(paths(&["SizeConstraintSetId"])?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "SizeConstraintSetId",
        "Name",
    ])?;
    bind(&mut operation, "SizeConstraintSetId", "ListSizeConstraintSets", "SizeConstraintSetId")?;
    Ok(operation)
}

pub(super) fn get_sql_injection_match_set() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "SqlInjectionMatchSet",
        Identity::Key(paths(&["SqlInjectionMatchSetId"])?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "SqlInjectionMatchSetId",
        "Name",
    ])?;
    bind(&mut operation, "SqlInjectionMatchSetId", "ListSqlInjectionMatchSets", "SqlInjectionMatchSetId")?;
    Ok(operation)
}

pub(super) fn get_web_acl() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "WebACL",
        Identity::Key(paths(&["WebACLId"])?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "WebACLId",
        "Name",
        "MetricName",
        "WebACLArn",
    ])?;
    bind(&mut operation, "WebACLId", "ListWebACLs", "WebACLId")?;
    Ok(operation)
}

pub(super) fn get_xss_match_set() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "XssMatchSet",
        Identity::Key(paths(&["XssMatchSetId"])?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "XssMatchSetId",
        "Name",
    ])?;
    bind(&mut operation, "XssMatchSetId", "ListXssMatchSets", "XssMatchSetId")?;
    Ok(operation)
}

pub(super) fn list_byte_match_sets() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "ByteMatchSets",
        Identity::Key(paths(&["ByteMatchSetId"])?),
        token("NextMarker", "NextMarker")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "ByteMatchSetId",
        "Name",
    ])?;
    operation.fixed.insert(path("Limit")?, FixedValue::Integer(100));
    Ok(operation)
}

pub(super) fn list_geo_match_sets() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "GeoMatchSets",
        Identity::Key(paths(&["GeoMatchSetId"])?),
        token("NextMarker", "NextMarker")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "GeoMatchSetId",
        "Name",
    ])?;
    operation.fixed.insert(path("Limit")?, FixedValue::Integer(100));
    Ok(operation)
}

pub(super) fn list_ip_sets() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "IPSets",
        Identity::Key(paths(&["IPSetId"])?),
        token("NextMarker", "NextMarker")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "IPSetId",
        "Name",
    ])?;
    operation.fixed.insert(path("Limit")?, FixedValue::Integer(100));
    Ok(operation)
}

pub(super) fn list_rate_based_rules() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "Rules",
        Identity::Key(paths(&["RuleId"])?),
        token("NextMarker", "NextMarker")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "RuleId",
        "Name",
    ])?;
    operation.fixed.insert(path("Limit")?, FixedValue::Integer(100));
    Ok(operation)
}
