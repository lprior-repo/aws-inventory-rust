use catalog_tool::{
    classes::DenyList, endpoints, kind_map, names, overlay, runtime_catalog, smithy,
};
use inventory_core::record::{FieldId, Key, MapRef, Record, Value, ValueRef};
use serde_json::json;
use std::{collections::BTreeMap, fs, path::Path};

pub type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
pub const COUNTER: &str = "SubscriptionsConfirmed";
pub const FIELDS: &str = "Attributes\nEnvelope\nId\nItems\nNoise\nOtherAttributes\nScalar\nStruct\nLists\nIntKeys\nAggregate\nListValues\nDocumentValues\n";

const MODEL: &str = r#"{"smithy":"2.0","shapes":{
"n#Service":{"type":"service","version":"1","operations":[{"target":"n#ListThings"}],"traits":{"aws.api#service":{"sdkId":"Things","endpointPrefix":"things","arnNamespace":"things"},"aws.auth#sigv4":{"name":"things"},"aws.protocols#restJson1":{}}},
"n#ListThings":{"type":"operation","output":{"target":"n#Output"},"traits":{"smithy.api#http":{"method":"GET","uri":"/things"}}},
"n#Output":{"type":"structure","members":{"Items":{"target":"n#Items"},"Attributes":{"target":"smithy.api#String"}}},
"n#Items":{"type":"list","member":{"target":"n#Thing"}},
"n#Thing":{"type":"structure","members":{"Id":{"target":"smithy.api#String"},"Attributes":{"target":"n#Attributes"},"Envelope":{"target":"n#Container"},"Noise":{"target":"smithy.api#Long"},"OtherAttributes":{"target":"n#Attributes"},"Scalar":{"target":"smithy.api#String"},"Struct":{"target":"n#Container"},"Lists":{"target":"n#Containers"},"IntKeys":{"target":"n#IntKeys"},"Aggregate":{"target":"n#Aggregate"},"ListValues":{"target":"n#ListValues"},"DocumentValues":{"target":"n#DocumentValues"}}},
"n#Container":{"type":"structure","members":{"Attributes":{"target":"n#Attributes"}}},
"n#Containers":{"type":"list","member":{"target":"n#Container"}},
"n#Attributes":{"type":"map","key":{"target":"smithy.api#String"},"value":{"target":"smithy.api#String"}},
"n#IntKeys":{"type":"map","key":{"target":"smithy.api#Long"},"value":{"target":"smithy.api#String"}},
"n#Aggregate":{"type":"map","key":{"target":"smithy.api#String"},"value":{"target":"n#Container"}},
"n#ListValues":{"type":"map","key":{"target":"smithy.api#String"},"value":{"target":"n#Containers"}},
"n#DocumentValues":{"type":"map","key":{"target":"smithy.api#String"},"value":{"target":"smithy.api#Document"}}
}}"#;

pub fn masks(entries: &[(&str, &str)]) -> TestResult<Vec<overlay::MapKeyExclusion>> {
    entries
        .iter()
        .map(|(path, key)| {
            Ok(overlay::MapKeyExclusion {
                path: overlay::MemberPath::new(path)?,
                key: (*key).to_owned(),
            })
        })
        .collect()
}

fn endpoint() -> TestResult<endpoints::VerifiedEndpoint> {
    let properties = json!({"authSchemes":[{"name":"sigv4","signingName":"things","signingRegion":"us-east-1"}]});
    let document = json!({"version":"1.0","parameters":{"Region":{"type":"string"}},"rules":[{"type":"endpoint","conditions":[],"endpoint":{"url":"https://things.test","properties":properties}}]});
    let rules = endpoints::RuleSet::parse(Path::new("map-exclusion-endpoint.json"), &document)?;
    let cases: Vec<endpoints::Case> = serde_json::from_value(
        json!([{"params":{"Region":"us-east-1"},"expect":{"endpoint":{"url":"https://things.test","properties":properties}}}]),
    )?;
    rules
        .run_cases(&cases)?
        .verified_endpoint(Some("things"))
        .map_err(|error| format!("fixture endpoint evidence failed: {error:?}").into())
}

pub fn compiled(exclusions: Vec<overlay::MapKeyExclusion>, numeric: bool) -> TestResult<Vec<u8>> {
    let directory = tempfile::tempdir()?;
    let source = directory.path().join("source");
    let models = source.join("models/things/service/2020-01-01");
    fs::create_dir_all(&models)?;
    fs::write(models.join("model.json"), MODEL)?;
    let mut plan = overlay::OperationOverlay::default();
    plan.class = Some(overlay::OverlayClass::Enumerate);
    plan.scope = Some(overlay::Scope::Region);
    plan.items = Some(overlay::ItemSelector::parse("Items")?);
    plan.identity = Some(overlay::Identity::Key(vec![overlay::MemberPath::new(
        "Id",
    )?]));
    plan.paginator = Some(overlay::Paginator::None);
    plan.exclude_map_keys = exclusions;
    if numeric {
        plan.exclude_from_hash = vec![overlay::MemberPath::new("Noise")?];
    }
    let policies = [overlay::ServiceOverlay::new(
        "things",
        vec![("ListThings", plan)],
    )?];
    let checked = overlay::check_policies(&source, &policies, &DenyList::builtin())?;
    let fields = smithy::Registry::parse(FIELDS)?;
    let base = smithy::build_catalog(&source, &fields, "map-exclusion-fixture")?;
    let kinds = kind_map::MappingSource {
        api_only: &[kind_map::ApiOnlyMapping {
            kind: "AWS::ApiOnly::MapExclusion",
            reason: "Reviewed map hash exclusion fixture",
        }],
        operations: &[kind_map::OperationMapping {
            service: "things",
            operation: "ListThings",
            kind: "AWS::ApiOnly::MapExclusion",
        }],
        ..kind_map::MappingSource::default()
    }
    .compile(&names::Exceptions::none())?;
    Ok(runtime_catalog::augment(
        &base,
        &checked,
        &kinds,
        &BTreeMap::from([("things".to_owned(), endpoint()?)]),
    )?)
}

pub fn field(id: u32) -> TestResult<Key> {
    Ok(Key::Field(FieldId::new(id)?))
}
pub fn name(key: &str) -> Key {
    Key::Name(key.to_owned())
}
pub fn text(value: &str) -> Value {
    Value::Text(value.to_owned())
}

pub fn map(entries: impl IntoIterator<Item = (u32, Value)>) -> TestResult<Value> {
    Ok(Value::Map(
        entries
            .into_iter()
            .map(|(id, value)| Ok((field(id)?, value)))
            .collect::<TestResult<BTreeMap<_, _>>>()?,
    ))
}

pub fn wire_mask(path: &[u32], key: &str) -> TestResult<Value> {
    map([
        (
            1,
            Value::List(path.iter().map(|id| Value::Int(i64::from(*id))).collect()),
        ),
        (2, text(key)),
    ])
}

pub fn row() -> TestResult<Value> {
    let attributes = Value::Map(BTreeMap::from([
        (name(COUNTER), text("1")),
        (name("SubscriptionsPending"), text("2")),
        (name("SubscriptionsDeleted"), text("3")),
        (name("DisplayName"), text("topic")),
        (name("subscriptionsconfirmed"), text("lower")),
        (name("SubscriptionsConfirmed.extra"), text("suffix")),
        (name("65535"), text("literal")),
        (field(65_535)?, text("numeric")),
    ]));
    let Value::Map(mut fields) = map([
        (1, attributes.clone()),
        (2, map([(1, attributes.clone())])?),
        (3, text("topic-1")),
        (5, Value::Int(10)),
        (6, attributes.clone()),
        (65_535, text("numeric root")),
    ])?
    else {
        return Err("row fixture is not a map".into());
    };
    fields.insert(name("Attributes"), attributes);
    fields.insert(name("65535"), text("literal root"));
    Ok(Value::Map(fields))
}

pub fn replaced(value: &Value, path: &[Key], replacement: Value) -> TestResult<Record> {
    let (last, parents) = path.split_last().ok_or("replacement path is empty")?;
    let mut changed = value.clone();
    let mut target = &mut changed;
    for key in parents {
        let Value::Map(fields) = target else {
            return Err("replacement parent is not a map".into());
        };
        target = fields.get_mut(key).ok_or("replacement parent absent")?;
    }
    let Value::Map(fields) = target else {
        return Err("replacement target is not a map".into());
    };
    fields
        .insert(last.clone(), replacement)
        .ok_or("replacement key absent")?;
    Ok(Record::encode(&changed)?)
}

pub fn rewritten(bytes: &[u8], replacement: Value) -> TestResult<Vec<u8>> {
    let mut root = ValueRef::Map(MapRef::from_bytes_bounded(bytes)?).to_value()?;
    let Value::Map(fields) = &mut root else {
        return Err("catalog root is not a map".into());
    };
    let Some(Value::Map(extension)) = fields.get_mut(&field(60_000)?) else {
        return Err("extension absent".into());
    };
    let Some(Value::List(plans)) = extension.get_mut(&field(2)?) else {
        return Err("plans absent".into());
    };
    let Some(Value::Map(plan)) = plans.first_mut() else {
        return Err("plan absent".into());
    };
    plan.insert(field(21)?, replacement);
    crate::runtime_provenance::reseal(&mut root, bytes)?;
    Ok(Record::encode(&root)?.into_bytes())
}
