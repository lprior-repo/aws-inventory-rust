use super::{
    ItemSelector, OperationName, OperationOverlay, OverlayClass, OverlayError, Rule, Scope,
    error::Context, fields, pagination, plan, raw,
};
use toml::Value;

pub(super) fn parse(context: &Context<'_>, value: Value) -> Result<OperationOverlay, OverlayError> {
    let raw: raw::Operation = raw::decode(context, value)?;
    let class = class(context, &raw)?;
    no_plan(context, &raw, class)?;
    let skip = skip(context, raw.skip.as_ref())?;
    let mut overlay = simple(context, &raw, class)?;
    overlay.skip = skip;
    nested(context, raw, overlay)
}
fn nested(
    context: &Context<'_>,
    raw: raw::Operation,
    mut overlay: OperationOverlay,
) -> Result<OperationOverlay, OverlayError> {
    overlay.fixed = fields::map(context, "fixed", raw.fixed, plan::fixed)?;
    overlay.bind = fields::map(context, "bind", raw.bind, plan::binding)?;
    overlay.paginator = raw
        .paginator
        .map(|value| pagination::paginator(&context.child("paginator"), value))
        .transpose()?;
    overlay.identity = raw
        .identity
        .map(|value| plan::identity(&context.child("identity"), value))
        .transpose()?;
    overlay.batch = raw
        .batch
        .map(|value| plan::batch(&context.child("batch"), value))
        .transpose()?;
    Ok(overlay)
}
fn simple(
    context: &Context<'_>,
    raw: &raw::Operation,
    class: Option<OverlayClass>,
) -> Result<OperationOverlay, OverlayError> {
    Ok(OperationOverlay {
        class,
        scope: scope(context, raw.scope.as_ref())?,
        items: item_selector(context, raw.items.as_ref())?,
        page_size: page_size(context, raw.page_size.as_ref())?,
        exclude_from_hash: exclusions(context, raw)?,
        skip: Vec::new(),
        quota_group: quota(context, raw.quota_group.as_ref())?,
        queryable: fields::paths(context, "queryable", raw.queryable.as_ref())?,
        tags: fields::optional_path(context, "tags", raw.tags.as_ref())?,
        fixed: Default::default(),
        paginator: None,
        identity: None,
        bind: Default::default(),
        batch: None,
    })
}
fn item_selector(
    context: &Context<'_>,
    value: Option<&Value>,
) -> Result<Option<ItemSelector>, OverlayError> {
    value
        .map(|value| {
            fields::string(context, "items", value).and_then(|text| {
                ItemSelector::parse(text)
                    .map_err(|_| context.error("items", text, Rule::MemberPath))
            })
        })
        .transpose()
}
fn exclusions(
    context: &Context<'_>,
    raw: &raw::Operation,
) -> Result<Vec<super::MemberPath>, OverlayError> {
    fields::paths(context, "exclude_from_hash", raw.exclude_from_hash.as_ref())
}
fn skip(context: &Context<'_>, value: Option<&Value>) -> Result<Vec<OperationName>, OverlayError> {
    let operations = fields::list(context, "skip", value, 0, 64, fields::operation)?;
    operations
        .iter()
        .try_for_each(|name| fields::not_self(context, "skip", name))?;
    Ok(operations)
}
fn page_size(context: &Context<'_>, value: Option<&Value>) -> Result<Option<i64>, OverlayError> {
    value
        .map(|value| fields::number(context, "page_size", value, 1, 100_000))
        .transpose()
}
fn class(
    context: &Context<'_>,
    raw: &raw::Operation,
) -> Result<Option<OverlayClass>, OverlayError> {
    let name = raw
        .class
        .as_ref()
        .map(|value| fields::string(context, "class", value))
        .transpose()?;
    if name != Some("deny") && raw.deny_reason.is_some() {
        let value = raw
            .deny_reason
            .as_ref()
            .map_or_else(String::new, fields::spelling);
        return Err(context.error("deny_reason", value, Rule::DenyReason));
    }
    match name {
        None => Ok(None),
        Some("enumerate") => Ok(Some(OverlayClass::Enumerate)),
        Some("expand") => Ok(Some(OverlayClass::Expand)),
        Some("defer") => Ok(Some(OverlayClass::Defer)),
        Some("deny") => deny_class(context, raw.deny_reason.as_ref()).map(Some),
        Some(name) => Err(context.error("class", name, Rule::Value)),
    }
}
fn deny_class(context: &Context<'_>, value: Option<&Value>) -> Result<OverlayClass, OverlayError> {
    let value = value.ok_or_else(|| context.error("deny_reason", "", Rule::DenyReason))?;
    let text = fields::string(context, "deny_reason", value)?;
    let reason = text
        .parse()
        .map_err(|_| context.error("deny_reason", text, Rule::Value))?;
    Ok(OverlayClass::Deny(reason))
}
fn no_plan(
    context: &Context<'_>,
    raw: &raw::Operation,
    class: Option<OverlayClass>,
) -> Result<(), OverlayError> {
    if !matches!(class, Some(OverlayClass::Deny(_) | OverlayClass::Defer)) {
        return Ok(());
    }
    if let Some((key, Some(value))) = plan_fields(raw)
        .into_iter()
        .find(|(_, value)| value.is_some())
    {
        return Err(context.error(key, fields::spelling(value), Rule::NoPlan));
    }
    Ok(())
}
fn plan_fields(raw: &raw::Operation) -> [(&'static str, &Option<Value>); 13] {
    [
        ("scope", &raw.scope),
        ("items", &raw.items),
        ("page_size", &raw.page_size),
        ("exclude_from_hash", &raw.exclude_from_hash),
        ("skip", &raw.skip),
        ("quota_group", &raw.quota_group),
        ("queryable", &raw.queryable),
        ("tags", &raw.tags),
        ("fixed", &raw.fixed),
        ("paginator", &raw.paginator),
        ("identity", &raw.identity),
        ("bind", &raw.bind),
        ("batch", &raw.batch),
    ]
}
fn scope(context: &Context<'_>, value: Option<&Value>) -> Result<Option<Scope>, OverlayError> {
    value
        .map(|value| match fields::string(context, "scope", value)? {
            "region" => Ok(Scope::Region),
            "account" => Ok(Scope::Account),
            "home-region" => Ok(Scope::HomeRegion),
            text => Err(context.error("scope", text, Rule::Value)),
        })
        .transpose()
}
fn quota(context: &Context<'_>, value: Option<&Value>) -> Result<Option<String>, OverlayError> {
    let value = fields::text(context, "quota_group", value)?;
    if let Some(text) = &value
        && (text.len() > 32
            || !text.starts_with(|character: char| character.is_ascii_lowercase())
            || !text
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'))
    {
        return Err(context.error("quota_group", text, Rule::Value));
    }
    Ok(value)
}
