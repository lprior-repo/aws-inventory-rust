use super::{OverlayError, Rule, error::Context};
use serde::{Deserialize, de::DeserializeOwned};
use toml::Value;

// Declaration-only macro: one field list drives Serde and contextual unknown-key checks.
macro_rules! shape {
    ($name:ident { $($field:ident),* $(,)? }) => {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        pub(super) struct $name { $(pub $field: Option<Value>),* }
        impl Shape for $name { const KEYS: &'static [&'static str] = &[$(stringify!($field)),*]; }
    };
}

pub(super) trait Shape: DeserializeOwned {
    const KEYS: &'static [&'static str];
}
shape!(Root {
    format,
    reviewed,
    reviewer,
    operation
});
shape!(Operation {
    class,
    deny_reason,
    scope,
    items,
    page_size,
    exclude_from_hash,
    skip,
    quota_group,
    queryable,
    tags,
    fixed,
    paginator,
    identity,
    bind,
    batch
});
shape!(Identity { arn, template, key });
shape!(Paginator {
    kind,
    input,
    output,
    pairs,
    truncated
});
shape!(Pair { input, output });
shape!(Binding {
    from,
    path,
    account,
    region
});
shape!(Batch {
    member,
    max,
    replaces
});

pub(super) fn decode<T: Shape>(context: &Context<'_>, value: Value) -> Result<T, OverlayError> {
    let table = value.as_table().ok_or_else(|| {
        context.error(
            "",
            super::fields::spelling(&value),
            Rule::WrongType { expected: "table" },
        )
    })?;
    if let Some((key, value)) = table
        .iter()
        .find(|(key, _)| !T::KEYS.contains(&key.as_str()))
    {
        return Err(context.error(key, super::fields::spelling(value), Rule::UnknownKey));
    }
    value.try_into().map_err(|error: toml::de::Error| {
        context.error("", error.message(), Rule::WrongType { expected: "table" })
    })
}
