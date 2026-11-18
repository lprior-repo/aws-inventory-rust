use super::*;
use catalog_tool::{endpoints, kind_map, overlay, runtime_catalog, smithy};
use std::sync::LazyLock;

pub(super) const OPERATIONS: &[&str] = &[
    "ListAdConfigurations",
    "ListFailoverSlateConfigurations",
    "ListChannels",
    "ListStreamKeys",
    "GetAdConfiguration",
    "GetFailoverSlateConfiguration",
    "GetChannel",
    "GetStreamKey",
    "BatchGetChannel",
    "BatchGetStreamKey",
    "GetStream",
    "GetStreamSession",
    "ListStreamSessions",
    "ListStreams",
    "CreateAdConfiguration",
    "CreateFailoverSlateConfiguration",
    "InsertAdBreak",
    "PutMetadata",
];
static CATALOG: LazyLock<Result<Vec<u8>, String>> =
    LazyLock::new(|| compile().map_err(|error| error.to_string()));

pub(super) fn catalog() -> TestResult<RuntimeCatalog<'static>> {
    match &*CATALOG {
        Ok(bytes) => Ok(RuntimeCatalog::from_bytes(bytes)?),
        Err(error) => Err(format!("pinned IVS roots fixture failed: {error}").into()),
    }
}

fn compile() -> TestResult<Vec<u8>> {
    let directory = tempfile::tempdir()?;
    let source = directory.path().join("source");
    crate::sweep_runtime::pinned_model::install(
        &source.join("models"),
        "ivs",
        "2020-07-14",
        OPERATIONS,
    )?;
    let embedded = overlay::embedded()?;
    let policy = embedded
        .iter()
        .find(|policy| policy.service() == "ivs")
        .ok_or("IVS policy absent")?;
    let operations = OPERATIONS
        .iter()
        .map(|operation| {
            let name = overlay::OperationName::new(operation)?;
            Ok((
                *operation,
                policy
                    .operations()
                    .get(&name)
                    .ok_or("IVS operation absent")?
                    .clone(),
            ))
        })
        .collect::<TestResult<Vec<_>>>()?;
    let policies = [overlay::ServiceOverlay::new("ivs", operations)?];
    let checked = overlay::check_policies(
        &source,
        &policies,
        &catalog_tool::classes::DenyList::builtin(),
    )?;
    let fields = fields()?;
    let base = smithy::build_catalog(&source, &fields, "46135f8468cd5cb60d983b97290c9f6661e359c6")?;
    let kinds = kind_map::embedded(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/catalog/cloudcontrol-universal.json"
    )))?;
    Ok(runtime_catalog::augment(
        &base,
        &checked,
        &kinds,
        &endpoints::verified_endpoints(&source)?,
    )?)
}

pub(super) fn fields() -> TestResult<smithy::Registry> {
    Ok(smithy::Registry::parse(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/catalog/field-ids.txt"
    )))?)
}

// Independent conversion of the persisted numeric record into member names lets the tests
// compare every descendant to literal metadata, rather than reusing the wire decoder as oracle.
pub(super) fn json_value(value: Value, fields: &smithy::Registry) -> TestResult<Json> {
    match value {
        Value::Text(value) => Ok(json!(value)),
        Value::Int(value) => Ok(json!(value)),
        Value::Bool(value) => Ok(json!(value)),
        Value::List(values) => Ok(Json::Array(
            values
                .into_iter()
                .map(|value| json_value(value, fields))
                .collect::<TestResult<_>>()?,
        )),
        Value::Map(values) => Ok(Json::Object(
            values
                .into_iter()
                .map(|(key, value)| {
                    let name = match key {
                        Key::Field(id) => {
                            fields.name(id).ok_or("unknown persisted field")?.to_owned()
                        }
                        Key::Name(name) => name,
                    };
                    Ok((name, json_value(value, fields)?))
                })
                .collect::<TestResult<_>>()?,
        )),
        _ => Err("unexpected non-metadata value in IVS resource".into()),
    }
}
