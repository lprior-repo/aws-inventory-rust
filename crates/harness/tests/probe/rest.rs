use super::{
    children::assert_row,
    context::Context,
    fixture,
    server::{Server, TestResult, header},
};
use catalog_tool::{kind_map, names, overlay, runtime_catalog, smithy};
use inventory_core::catalog::runtime::RuntimeCatalog;
use std::{collections::BTreeMap, fs};

const MODEL: &str = r#"{"smithy":"2.0","shapes":{
"n#Service":{"type":"service","version":"1","operations":[{"target":"n#ListParents"},{"target":"n#ListChildren"},{"target":"n#ListTags"}],"traits":{"aws.api#service":{"sdkId":"Bindings","endpointPrefix":"bindings"},"aws.auth#sigv4":{"name":"bindings"},"aws.protocols#restJson1":{}}},
"n#ListParents":{"type":"operation","output":{"target":"n#Output"},"traits":{"smithy.api#http":{"method":"GET","uri":"/parents"}}},
"n#ListChildren":{"type":"operation","input":{"target":"n#ChildInput"},"output":{"target":"n#Output"},"traits":{"smithy.api#http":{"method":"GET","uri":"/parents/{EntityId}/children"}}},
"n#ListTags":{"type":"operation","input":{"target":"n#TagInput"},"output":{"target":"n#TagOutput"},"traits":{"smithy.api#http":{"method":"GET","uri":"/tags/{EntityId}"}}},
"n#ChildInput":{"type":"structure","members":{"EntityId":{"target":"smithy.api#String","traits":{"smithy.api#required":{},"smithy.api#httpLabel":{}}},"Filter":{"target":"smithy.api#String","traits":{"smithy.api#httpQuery":"filter"}},"Marker":{"target":"smithy.api#String","traits":{"smithy.api#httpHeader":"X-Probe-Marker"}}}},
"n#TagInput":{"type":"structure","members":{"EntityId":{"target":"smithy.api#String","traits":{"smithy.api#required":{},"smithy.api#httpLabel":{}}}}},
"n#Output":{"type":"structure","members":{"Items":{"target":"n#Items"}}},
"n#Item":{"type":"structure","members":{"Id":{"target":"smithy.api#String"},"Filter":{"target":"smithy.api#String"},"Marker":{"target":"smithy.api#String"}}},
"n#Items":{"type":"list","member":{"target":"n#Item"}},
"n#TagOutput":{"type":"structure","members":{"Tags":{"target":"n#Tags"}}},
"n#Tags":{"type":"map","key":{"target":"smithy.api#String"},"value":{"target":"smithy.api#String"}}
}}"#;
const PARENT: &str = r#"{"Items":[{"Id":"first/id","Filter":"blue sky","Marker":"from-parent"}]}"#;
const PARENT_LINE: &str = "GET /parents HTTP/1.1";
const CHILD_LINE: &str = "GET /parents/first%2Fid/children?filter=blue%20sky HTTP/1.1";

#[tokio::test]
async fn given_rest_child_when_probed_then_model_places_bound_values_on_path_query_and_header()
-> TestResult {
    let (report, requests) = probe().await?;
    assert_eq!(request_lines(&requests), [PARENT_LINE, CHILD_LINE]);
    let child = requests.get(1).ok_or("child request missing")?;
    assert_eq!(header(child, "X-Probe-Marker")?, "from-parent");
    assert_row(&report, "bindings ListChildren restJson1 items 1");
    Ok(())
}

#[tokio::test]
async fn given_tag_plan_when_probed_then_dedicated_binding_is_skipped_without_a_request()
-> TestResult {
    let (report, requests) = probe().await?;
    assert_eq!(request_lines(&requests), [PARENT_LINE, CHILD_LINE]);
    assert_row(
        &report,
        "bindings ListTags restJson1 skipped child tag endpoint uses dedicated resource tag binding",
    );
    assert_row(&report, "total skipped-child 1");
    assert_row(
        &report,
        "protocol restJson1 items 2 empty 0 unsupported 0 failed 0 skipped-child 1",
    );
    Ok(())
}

fn request_lines(requests: &[String]) -> Vec<&str> {
    requests
        .iter()
        .filter_map(|request| request.lines().next())
        .collect()
}

async fn probe() -> TestResult<(String, Vec<String>)> {
    let bytes = compiled()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let server = Server::new().await?;
    let context = Context::new(&server)?;
    server
        .capture_with(context.report(&catalog), |request| {
            let uri = request
                .lines()
                .next()
                .and_then(|line| line.split_whitespace().nth(1));
            match uri {
                Some("/parents") => Ok((200, PARENT)),
                Some(uri) if uri.starts_with("/tags/") => Ok((200, r#"{"Tags":{"env":"test"}}"#)),
                _ => Ok((200, r#"{"Items":[{"Id":"child"}]}"#)),
            }
        })
        .await
}

fn compiled() -> TestResult<Vec<u8>> {
    let temp = tempfile::tempdir()?;
    let model = temp.path().join("models/bindings/service/2020-01-01");
    fs::create_dir_all(&model)?;
    fs::write(model.join("bindings.json"), MODEL)?;
    let policy = overlay::ServiceOverlay::new(
        "bindings",
        vec![
            ("ListParents", items(overlay::OverlayClass::Enumerate)?),
            ("ListChildren", child()?),
            ("ListTags", tags()?),
        ],
    )?;
    let checked = overlay::check_policies(
        temp.path(),
        &[policy],
        &catalog_tool::classes::DenyList::builtin(),
    )?;
    let fields = smithy::Registry::parse("EntityId\nFilter\nId\nItems\nMarker\nTags\n")?;
    let base = smithy::build_catalog(temp.path(), &fields, "rest-child-probe-fixture")?;
    let kinds = kind_map::MappingSource {
        operations: &[
            kind_map::OperationMapping {
                service: "bindings",
                operation: "ListParents",
                kind: "AWS::ApiOnly::BindingParent",
            },
            kind_map::OperationMapping {
                service: "bindings",
                operation: "ListChildren",
                kind: "AWS::ApiOnly::BindingChild",
            },
        ],
        api_only: &[
            kind_map::ApiOnlyMapping {
                kind: "AWS::ApiOnly::BindingParent",
                reason: "REST child parent fixture",
            },
            kind_map::ApiOnlyMapping {
                kind: "AWS::ApiOnly::BindingChild",
                reason: "REST child binding fixture",
            },
        ],
        ..kind_map::MappingSource::default()
    }
    .compile(&names::Exceptions::none())?;
    let endpoints = BTreeMap::from([("bindings".to_owned(), fixture::endpoint()?)]);
    Ok(runtime_catalog::augment(
        &base, &checked, &kinds, &endpoints,
    )?)
}

fn items(class: overlay::OverlayClass) -> TestResult<overlay::OperationOverlay> {
    let mut plan = overlay::OperationOverlay::default();
    plan.class = Some(class);
    plan.scope = Some(overlay::Scope::Region);
    plan.items = Some(overlay::ItemSelector::parse("Items")?);
    plan.paginator = Some(overlay::Paginator::None);
    plan.identity = Some(overlay::Identity::Key(vec![overlay::MemberPath::new(
        "Id",
    )?]));
    Ok(plan)
}

fn child() -> TestResult<overlay::OperationOverlay> {
    let mut child = items(overlay::OverlayClass::Expand)?;
    for (input, parent) in [
        ("EntityId", "Id"),
        ("Filter", "Filter"),
        ("Marker", "Marker"),
    ] {
        child.bind.insert(
            overlay::MemberPath::new(input)?,
            overlay::Binding::From {
                operation: overlay::OperationName::new("ListParents")?,
                path: overlay::MemberPath::selected(parent)?,
            },
        );
    }
    Ok(child)
}

fn tags() -> TestResult<overlay::OperationOverlay> {
    Ok(overlay::TagEndpoint {
        parents: vec![overlay::TagEndpointParent {
            operation: overlay::OperationName::new("ListParents")?,
            request: overlay::TagEndpointRequest::Member {
                input: overlay::MemberPath::new("EntityId")?,
                parent: overlay::MemberPath::new("Id")?,
            },
        }],
        selector: overlay::TagEndpointSelector::Map {
            path: overlay::MemberPath::new("Tags")?,
        },
    }
    .operation(overlay::Scope::Region, overlay::Paginator::None)?)
}
