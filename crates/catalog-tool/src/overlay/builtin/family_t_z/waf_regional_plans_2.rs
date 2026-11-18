use super::{inventory, path, paths, token};
use crate::overlay::{FixedValue, Identity, NameError, OperationOverlay, Scope};

pub(super) fn list_regex_match_sets() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "RegexMatchSets",
        Identity::Key(paths(&["RegexMatchSetId"])?),
        token("NextMarker", "NextMarker")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&["RegexMatchSetId", "Name"])?;
    operation
        .fixed
        .insert(path("Limit")?, FixedValue::Integer(100));
    Ok(operation)
}

pub(super) fn list_regex_pattern_sets() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "RegexPatternSets",
        Identity::Key(paths(&["RegexPatternSetId"])?),
        token("NextMarker", "NextMarker")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&["RegexPatternSetId", "Name"])?;
    operation
        .fixed
        .insert(path("Limit")?, FixedValue::Integer(100));
    Ok(operation)
}

pub(super) fn list_rule_groups() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "RuleGroups",
        Identity::Key(paths(&["RuleGroupId"])?),
        token("NextMarker", "NextMarker")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&["RuleGroupId", "Name"])?;
    operation
        .fixed
        .insert(path("Limit")?, FixedValue::Integer(100));
    Ok(operation)
}

pub(super) fn list_rules() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "Rules",
        Identity::Key(paths(&["RuleId"])?),
        token("NextMarker", "NextMarker")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&["RuleId", "Name"])?;
    operation
        .fixed
        .insert(path("Limit")?, FixedValue::Integer(100));
    Ok(operation)
}

pub(super) fn list_size_constraint_sets() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "SizeConstraintSets",
        Identity::Key(paths(&["SizeConstraintSetId"])?),
        token("NextMarker", "NextMarker")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&["SizeConstraintSetId", "Name"])?;
    operation
        .fixed
        .insert(path("Limit")?, FixedValue::Integer(100));
    Ok(operation)
}

pub(super) fn list_sql_injection_match_sets() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "SqlInjectionMatchSets",
        Identity::Key(paths(&["SqlInjectionMatchSetId"])?),
        token("NextMarker", "NextMarker")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&["SqlInjectionMatchSetId", "Name"])?;
    operation
        .fixed
        .insert(path("Limit")?, FixedValue::Integer(100));
    Ok(operation)
}

pub(super) fn list_web_ac_ls() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "WebACLs",
        Identity::Key(paths(&["WebACLId"])?),
        token("NextMarker", "NextMarker")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&["WebACLId", "Name"])?;
    operation
        .fixed
        .insert(path("Limit")?, FixedValue::Integer(100));
    Ok(operation)
}

pub(super) fn list_xss_match_sets() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "XssMatchSets",
        Identity::Key(paths(&["XssMatchSetId"])?),
        token("NextMarker", "NextMarker")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&["XssMatchSetId", "Name"])?;
    operation
        .fixed
        .insert(path("Limit")?, FixedValue::Integer(100));
    Ok(operation)
}
