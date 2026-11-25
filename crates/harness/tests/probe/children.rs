use super::{
    cli,
    context::Context,
    fixture,
    server::{Server, TestResult, response, target},
};
use catalog_tool::overlay::{Binding, ContextSource, MemberPath, OperationName, OverlayClass};
use inventory_core::catalog::runtime::RuntimeCatalog;
use serde_json::{Value, json};
use std::time::Duration;

#[tokio::test]
async fn given_parent_item_when_probed_then_child_sends_item_and_request_context() -> TestResult {
    let bytes = fixture::compiled(1, |_| {})?;
    let (report, requests) = run(&bytes, 2, response).await?;
    assert_eq!(targets(&requests)?, ["ListParents0", "ListChildren"]);
    assert_eq!(
        body(requests.get(1).ok_or("child not called")?)?,
        json!({"OrganizationId":"org-0", "EntityId":"one", "Regions":["us-east-1"]})
    );
    assert!(report.contains("ListChildren awsJson1_1 items 1\n"));
    Ok(())
}

#[tokio::test]
async fn given_empty_parent_when_probed_then_child_has_no_parent_items() -> TestResult {
    let bindings = bindings_from("ListParents1")?;
    let bytes = fixture::compiled(2, move |child| child.bind.extend(bindings))?;
    let (report, requests) = run(&bytes, 2, response).await?;
    assert!(!targets(&requests)?.contains(&"ListChildren"));
    assert!(report.contains("ListChildren awsJson1_1 skipped child no parent items\n"));
    Ok(())
}

#[tokio::test]
async fn given_a_failing_child_sample_when_probed_then_other_samples_still_run() -> TestResult {
    let bytes = fixture::compiled(1, |_| {})?;
    let (report, requests) = cli::run(&bytes, &["--child-samples", "2"], 3, |request| {
        if target(request)? == "ListChildren"
            && body(request)?.get("EntityId") == Some(&json!("one"))
        {
            return Ok((200, "malformed JSON"));
        }
        response(request)
    })
    .await?;
    assert_eq!(
        targets(&requests)?,
        ["ListParents0", "ListChildren", "ListChildren"]
    );
    assert!(
        requests
            .iter()
            .any(|request| request.contains(r#""EntityId":"two""#))
    );
    assert!(report.contains("ListChildren awsJson1_1 failed decode shape="));
    Ok(())
}

#[tokio::test]
async fn given_child_items_when_probed_then_grandchild_sends_item_and_ancestry() -> TestResult {
    let bytes = chain("ListParents0")?;
    let (report, requests) = run(&bytes, 3, response).await?;
    assert_eq!(
        targets(&requests)?,
        ["ListParents0", "ListChildren", "ListParents1"]
    );
    assert_eq!(
        body(requests.get(2).ok_or("grandchild not called")?)?,
        json!({"OrganizationId":"org-0", "EntityId":"child", "Regions":["us-east-1"]})
    );
    assert!(report.contains("ListParents1 awsJson1_1 empty\n"));
    Ok(())
}

#[tokio::test]
async fn given_empty_child_page_when_probed_then_grandchild_is_not_called() -> TestResult {
    for (context_source, reason) in [
        ("ListChildren", "no parent items"),
        (
            "ListParents0",
            "no sampled parent item satisfies all bindings and row eligibility",
        ),
    ] {
        let bytes = chain(context_source)?;
        let (report, requests) = run(&bytes, 2, |request| match target(request)? {
            "ListChildren" => Ok((200, r#"{"Items":[],"NextToken":"do-not-fetch"}"#)),
            _ => response(request),
        })
        .await?;
        assert_eq!(targets(&requests)?, ["ListParents0", "ListChildren"]);
        assert!(
            report.contains(&format!("ListParents1 awsJson1_1 skipped child {reason}\n")),
            "{report}"
        );
    }
    Ok(())
}

#[tokio::test]
async fn given_independent_producers_when_probed_then_inputs_are_not_spliced() -> TestResult {
    let entity = MemberPath::new("EntityId")?;
    let org = MemberPath::new("OrganizationId")?;
    let first = from("ListParents0")?;
    let second = from("ListParents1")?;
    let bytes = fixture::compiled(2, move |child| {
        child.bind.insert(entity, first);
        child.bind.insert(org, second);
    })?;
    let (report, requests) = run(&bytes, 2, |request| match target(request)? {
        "ListParents1" => Ok((200, r#"{"Items":[{"Id":"unrelated"}]}"#)),
        _ => response(request),
    })
    .await?;
    assert!(!targets(&requests)?.contains(&"ListChildren"));
    assert!(report.contains(
        "ListChildren awsJson1_1 skipped child no sampled parent item satisfies all bindings"
    ));
    Ok(())
}

#[tokio::test]
async fn given_two_samples_when_probed_then_both_first_pages_contribute_items() -> TestResult {
    let bytes = fixture::compiled(1, |_| {})?;
    let report = cli::report(&bytes, &["--child-samples", "2"], 3).await?;
    assert!(report.contains("ListChildren awsJson1_1 items 2\n"));
    Ok(())
}

fn chain(context_source: &str) -> TestResult<Vec<u8>> {
    let child = bindings_from("ListParents0")?;
    let org = MemberPath::new("OrganizationId")?;
    let second_parent = catalog_tool::overlay::FixedValue::String("org-1".into());
    let grandchild = [
        (MemberPath::new("EntityId")?, from("ListChildren")?),
        (org.clone(), request_from(context_source)?),
    ];
    fixture::compiled_with_parents(
        2,
        move |parent| {
            if parent.fixed.get(&org) == Some(&second_parent) {
                parent.class = Some(OverlayClass::Expand);
                parent.fixed.clear();
                parent.bind.extend(grandchild.clone());
            }
        },
        move |plan| plan.bind.extend(child),
    )
}

fn bindings_from(operation: &str) -> TestResult<[(MemberPath, Binding); 2]> {
    Ok([
        (MemberPath::new("EntityId")?, from(operation)?),
        (MemberPath::new("OrganizationId")?, request_from(operation)?),
    ])
}

fn from(operation: &str) -> TestResult<Binding> {
    Ok(Binding::From {
        operation: OperationName::new(operation)?,
        path: MemberPath::selected("Id")?,
    })
}

fn request_from(operation: &str) -> TestResult<Binding> {
    Ok(Binding::Context {
        operation: OperationName::new(operation)?,
        source: ContextSource::Request,
        path: MemberPath::new("OrganizationId")?,
    })
}

async fn run(
    bytes: &[u8],
    count: usize,
    respond: impl Fn(&str) -> TestResult<(u16, &'static str)>,
) -> TestResult<(String, Vec<String>)> {
    let catalog = RuntimeCatalog::from_bytes(bytes)?;
    let server = Server::new().await?;
    let context = Context::new(&server)?;
    tokio::time::timeout(Duration::from_secs(10), async {
        tokio::try_join!(context.report(&catalog), server.serve_with(count, respond))
    })
    .await?
}

fn targets(requests: &[String]) -> TestResult<Vec<&str>> {
    requests.iter().map(|request| target(request)).collect()
}

fn body(request: &str) -> TestResult<Value> {
    let (_, json) = request
        .split_once("\r\n\r\n")
        .ok_or("request has no body")?;
    Ok(serde_json::from_str(json)?)
}
