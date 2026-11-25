use catalog_tool::{
    classes::DenyList, endpoints, kind_map, names, overlay, runtime_catalog, smithy,
};
use serde_json::json;
use std::{collections::BTreeMap, fs, path::Path};

pub type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

pub fn compiled(
    count: usize,
    change: impl FnOnce(&mut overlay::OperationOverlay),
) -> TestResult<Vec<u8>> {
    compiled_with_parents(count, |_| {}, change)
}

pub fn compiled_with_parents(
    count: usize,
    change_parent: impl FnMut(&mut overlay::OperationOverlay),
    change: impl FnOnce(&mut overlay::OperationOverlay),
) -> TestResult<Vec<u8>> {
    compiled_with_model(count, change_parent, change, |_| {})
}

pub fn compiled_with_model(
    count: usize,
    change_parent: impl FnMut(&mut overlay::OperationOverlay),
    change: impl FnOnce(&mut overlay::OperationOverlay),
    change_model: impl FnOnce(&mut serde_json::Map<String, serde_json::Value>),
) -> TestResult<Vec<u8>> {
    let source = tempfile::tempdir()?;
    let directory = source.path().join("models/bindings/service/2020-01-01");
    fs::create_dir_all(&directory)?;
    let producers = (0..count)
        .map(|index| format!("ListParents{index}"))
        .collect::<Vec<_>>();
    let operations = producers
        .iter()
        .map(String::as_str)
        .chain(std::iter::once("ListChildren"))
        .collect::<Vec<_>>();
    let mut shapes = model_shapes(&operations);
    change_model(&mut shapes);
    fs::write(
        directory.join("bindings.json"),
        serde_json::to_vec(&json!({"smithy":"2.0","shapes":shapes}))?,
    )?;
    let policy = policy(&producers, change_parent, change)?;
    compile_source(source.path(), &operations, policy)
}

fn model_shapes(operations: &[&str]) -> serde_json::Map<String, serde_json::Value> {
    let mut shapes = serde_json::Map::from_iter([
        (
            "n#Service".to_owned(),
            json!({"type":"service","version":"1","operations":operations.iter().map(|name|json!({"target":format!("n#{name}")})).collect::<Vec<_>>(),"traits":{"aws.api#service":{"sdkId":"Bindings","endpointPrefix":"bindings"},"aws.auth#sigv4":{"name":"bindings"},"aws.protocols#awsJson1_1":{}}}),
        ),
        (
            "n#Input".to_owned(),
            json!({"type":"structure","members":{"OrganizationId":{"target":"smithy.api#String","traits":{"smithy.api#required":{}}},"Regions":{"target":"n#Regions"},"EntityId":{"target":"smithy.api#String"},"NextToken":{"target":"smithy.api#String"}}}),
        ),
        (
            "n#Output".to_owned(),
            json!({"type":"structure","members":{"Items":{"target":"n#Items"},"NextToken":{"target":"smithy.api#String"},"ResponseId":{"target":"smithy.api#String"}}}),
        ),
        (
            "n#Regions".to_owned(),
            json!({"type":"list","member":{"target":"smithy.api#String"}}),
        ),
        (
            "n#Items".to_owned(),
            json!({"type":"list","member":{"target":"n#Item"}}),
        ),
        (
            "n#Item".to_owned(),
            json!({"type":"structure","members":{"Id":{"target":"smithy.api#String"},"Count":{"target":"smithy.api#Long"}}}),
        ),
    ]);
    operations.iter().for_each(|name| {
        shapes.insert(
            format!("n#{name}"),
            json!({"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#Output"}}),
        );
    });
    shapes
}

fn policy(
    producers: &[String],
    mut change_parent: impl FnMut(&mut overlay::OperationOverlay),
    change: impl FnOnce(&mut overlay::OperationOverlay),
) -> TestResult<overlay::ServiceOverlay> {
    let mut plans = producers
        .iter()
        .enumerate()
        .map(|(index, name)| -> TestResult<_> {
            let mut plan = plan()?;
            plan.fixed.insert(
                overlay::MemberPath::new("OrganizationId")?,
                overlay::FixedValue::String(format!("org-{index}")),
            );
            change_parent(&mut plan);
            Ok((name.as_str(), plan))
        })
        .collect::<TestResult<Vec<_>>>()?;
    let mut child = plan()?;
    child.class = Some(overlay::OverlayClass::Expand);
    child.bind.insert(
        overlay::MemberPath::new("EntityId")?,
        alternatives(producers, overlay::ContextSource::SelectedItem, "Id")?,
    );
    child.bind.insert(
        overlay::MemberPath::new("OrganizationId")?,
        alternatives(producers, overlay::ContextSource::Request, "OrganizationId")?,
    );
    change(&mut child);
    plans.push(("ListChildren", child));
    Ok(overlay::ServiceOverlay::new("bindings", plans)?)
}

fn compile_source(
    source: &Path,
    operations: &[&str],
    policy: overlay::ServiceOverlay,
) -> TestResult<Vec<u8>> {
    let checked = overlay::check_policies(source, &[policy], &DenyList::builtin())?;
    let fields = smithy::Registry::parse(
        "Count\nEntityId\nId\nItems\nNextToken\nOrganizationId\nRegions\nResponseId\n",
    )?;
    let base = smithy::build_catalog(source, &fields, "typed-binding-fixture")?;
    let kinds = kinds(operations)?;
    let endpoints = BTreeMap::from([("bindings".to_owned(), endpoint()?)]);
    Ok(runtime_catalog::augment(
        &base, &checked, &kinds, &endpoints,
    )?)
}

fn kinds(operations: &[&str]) -> TestResult<kind_map::KindMap> {
    let mappings = operations
        .iter()
        .map(|operation| kind_map::OperationMapping {
            service: "bindings",
            operation,
            kind: if *operation == "ListChildren" {
                "AWS::ApiOnly::BindingChild"
            } else {
                "AWS::ApiOnly::BindingParent"
            },
        })
        .collect::<Vec<_>>();
    Ok(kind_map::MappingSource {
        relations: &[kind_map::RelationMapping {
            source_kind: "AWS::ApiOnly::BindingChild",
            source: kind_map::RelationSource::Request(kind_map::RelationPath::Fields("EntityId")),
            target_kind: "AWS::ApiOnly::BindingParent",
            target: kind_map::RelationTarget::SelectedOutput(kind_map::RelationPath::Fields("Id")),
            relation: "IN_PARENT",
            scope: "account_region",
            guard: None,
        }],
        api_only: &[
            kind_map::ApiOnlyMapping {
                kind: "AWS::ApiOnly::BindingParent",
                reason: "Typed binding parent fixture",
            },
            kind_map::ApiOnlyMapping {
                kind: "AWS::ApiOnly::BindingChild",
                reason: "Typed binding child fixture",
            },
        ],
        operations: &mappings,
        ..kind_map::MappingSource::default()
    }
    .compile(&names::Exceptions::none())?)
}

fn plan() -> TestResult<overlay::OperationOverlay> {
    let mut plan = overlay::OperationOverlay::default();
    plan.class = Some(overlay::OverlayClass::Enumerate);
    plan.scope = Some(overlay::Scope::Region);
    plan.items = Some(overlay::ItemSelector::parse("Items")?);
    plan.identity = Some(overlay::Identity::Key(vec![overlay::MemberPath::new(
        "Id",
    )?]));
    plan.paginator = Some(overlay::Paginator::Token(overlay::TokenPair::new(
        overlay::MemberPath::new("NextToken")?,
        overlay::MemberPath::new("NextToken")?,
    )));
    plan.bind.insert(
        overlay::MemberPath::new("Regions")?,
        overlay::Binding::RegionList,
    );
    Ok(plan)
}

pub fn alternatives(
    producers: &[String],
    source: overlay::ContextSource,
    path: &str,
) -> TestResult<overlay::Binding> {
    Ok(overlay::Binding::FromAny(
        producers
            .iter()
            .map(|operation| {
                Ok(overlay::BindingAlternative {
                    operation: overlay::OperationName::new(operation)?,
                    source,
                    path: overlay::MemberPath::selected(path)?,
                })
            })
            .collect::<Result<Vec<_>, overlay::NameError>>()?,
    ))
}

fn endpoint() -> TestResult<endpoints::VerifiedEndpoint> {
    let properties = json!({"authSchemes":[{"name":"sigv4","signingName":"bindings","signingRegion":"us-east-1"}]});
    let document = json!({"version":"1.0","parameters":{"Region":{"type":"string"}},"rules":[{"type":"endpoint","conditions":[],"endpoint":{"url":"https://bindings.test","properties":properties}}]});
    let rules = endpoints::RuleSet::parse(Path::new("binding-endpoint.json"), &document)?;
    let cases: Vec<endpoints::Case> = serde_json::from_value(
        json!([{"params":{"Region":"us-east-1"},"expect":{"endpoint":{"url":"https://bindings.test","properties":properties}}}]),
    )?;
    rules
        .run_cases(&cases)?
        .verified_endpoint(Some("bindings"))
        .map_err(|failure| format!("fixture endpoint evidence failed: {failure:?}").into())
}
