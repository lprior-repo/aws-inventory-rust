mod behavior;
use super::*;
use inventory_core::record::{FieldId, ValueRef};
use crate::overlay::ServiceOverlay;

type TestResult = Result<(), Box<dyn std::error::Error>>;

const MODEL: &str = r#"{"shapes":{
"n#Svc":{"type":"service","version":"2020-01-01","traits":{"aws.protocols#restXml":{}}},
"n#ListThings":{"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#Output"},"traits":{"smithy.api#readonly":{},"smithy.api#http":{"method":"GET","uri":"/things"}}},
"n#Input":{"type":"structure","members":{
"StartName":{"target":"smithy.api#String","traits":{"smithy.api#httpQuery":"name"}},
"StartType":{"target":"n#Kind","traits":{"smithy.api#httpQuery":"type"}},
"StartId":{"target":"smithy.api#String","traits":{"smithy.api#httpQuery":"identifier"}}}},
"n#Kind":{"type":"enum","members":{"A":{"target":"smithy.api#Unit","traits":{"smithy.api#enumValue":"A"}}}},
"n#Output":{"type":"structure","members":{
"Items":{"target":"n#Rows"},"NextName":{"target":"smithy.api#String"},
"NextType":{"target":"n#Kind"},"NextId":{"target":"smithy.api#String"},"IsTruncated":{"target":"smithy.api#Boolean"}}},
"n#Rows":{"type":"list","member":{"target":"n#Row"}},
"n#Row":{"type":"structure","members":{"Id":{"target":"smithy.api#String"}}}
}}"#;
const PAIRS: &str = "[{input='StartName',output='NextName'}, {input='StartType',output='NextType'}, {input='StartId',output='NextId',optional=true}]";
const REGISTRY: &str = "A\nId\nIsTruncated\nItems\nNextId\nNextName\nNextType\nStartId\nStartName\nStartType\n";

fn compile_fixture(model: &str, pairs: &str) -> Result<Value, Box<dyn std::error::Error>> {
    compile_fixture_with_registry(model, pairs, REGISTRY)
}

fn compile_fixture_with_registry(model: &str, pairs: &str, registry: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let service_dir = directory.path().join("models/example/service/2020-01-01");
    std::fs::create_dir_all(&service_dir)?;
    std::fs::write(service_dir.join("example.json"), model)?;
    let registry = crate::smithy::Registry::parse(registry)?;
    let bytes = crate::smithy::build_catalog(directory.path(), &registry, "fixture")?;
    let catalog = inventory_core::catalog::Catalog::borrow(&bytes)?;
    let services = crate::runtime_catalog::ServiceIndex::build(catalog)?;
    let service = catalog.service("example")?.ok_or("service")?;
    let operation = service.operations()?.next().ok_or("operation")??;
    let overlay = ServiceOverlay::parse("example", &format!(
        "format=1\nreviewed='2026-11-15'\nreviewer='pagination'\n[operation.ListThings]\nclass='enumerate'\nscope='account'\nitems='Items'\nidentity={{key=['Id']}}\npaginator={{kind='tokens',pairs={pairs},truncated='IsTruncated'}}\n"
    ))?;
    let plan = overlay.operations().values().next().ok_or("review")?;
    Ok(compile(plan, service, operation, 6, &services)?)
}

fn tokens(value: &Value) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let record = inventory_core::record::Record::encode(value)?;
    match record.root().get_field(FieldId::new(2)?)? {
        Some(ValueRef::List(values)) => values.iter()?.map(|value| value?.to_value()).collect::<Result<_, _>>().map_err(Into::into),
        _ => Err("missing compiled tokens".into()),
    }
}
