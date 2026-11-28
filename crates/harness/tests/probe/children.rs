use super::{
    cli,
    context::Context,
    fixture,
    server::{Server, TestResult, response, target},
};
use catalog_tool::overlay::{
    Binding, ChildEligibility, ContextSource, FixedValue, MemberPath, MissingSelected,
    OperationName, OverlayClass, RowPredicate, RowValueType,
};
use inventory_core::catalog::runtime::RuntimeCatalog;
use serde_json::{Value, json};

#[tokio::test]
async fn given_parent_item_when_probed_then_child_sends_item_and_request_context() -> TestResult {
    let bytes = fixture::compiled(1, |_| {})?;
    let (report, requests) = run(&bytes, response).await?;
    assert_eq!(targets(&requests)?, ["ListParents0", "ListChildren"]);
    assert_eq!(
        body(requests.get(1).ok_or("child not called")?)?,
        json!({"OrganizationId":"org-0", "EntityId":"one", "Regions":["us-east-1"]})
    );
    assert_row(&report, "bindings ListChildren awsJson1_1 items 1");
    Ok(())
}

#[tokio::test]
async fn given_parent_without_items_when_probed_then_child_has_no_parent_items() -> TestResult {
    let bindings = bindings_from("ListParents1")?;
    let bytes = fixture::compiled(2, move |child| child.bind.extend(bindings))?;
    for (reply, outcome) in [
        ((200, r#"{"Items":[]}"#), "empty"),
        ((400, UNSUPPORTED), UNSUPPORTED_OUTCOME),
        (DENIED, DENIED_OUTCOME),
    ] {
        let (report, requests) = run(&bytes, |request| match target(request)? {
            "ListParents1" => Ok(reply),
            _ => response(request),
        })
        .await?;
        assert_eq!(sorted_targets(&requests)?, ["ListParents0", "ListParents1"]);
        assert_row(
            &report,
            &format!("bindings ListParents1 awsJson1_1 {outcome}"),
        );
        assert_row(
            &report,
            "bindings ListChildren awsJson1_1 skipped child no parent items",
        );
    }
    Ok(())
}

#[tokio::test]
async fn given_ineligible_first_parent_item_when_probed_then_default_admits_the_next_item()
-> TestResult {
    let eligibility = ChildEligibility {
        operation: OperationName::new("ListParents0")?,
        predicate: RowPredicate::SelectedRequestEquals {
            selected: MemberPath::new("Id")?,
            request: MemberPath::new("OrganizationId")?,
            value_type: RowValueType::String,
            missing: MissingSelected::Reject,
        },
    };
    let bytes = fixture::compiled(1, move |child| child.row_policy.child = Some(eligibility))?;
    let (report, requests) = run(&bytes, |request| match target(request)? {
        "ListParents0" => Ok((200, r#"{"Items":[{"Id":"pending"},{"Id":"org-0"}]}"#)),
        _ => response(request),
    })
    .await?;
    assert_eq!(targets(&requests)?, ["ListParents0", "ListChildren"]);
    assert_eq!(
        body(requests.get(1).ok_or("child not called")?)?,
        json!({"OrganizationId":"org-0", "EntityId":"org-0", "Regions":["us-east-1"]})
    );
    assert_row(&report, "bindings ListParents0 awsJson1_1 items 2");
    assert_row(&report, "bindings ListChildren awsJson1_1 items 1");
    Ok(())
}

const UNSUPPORTED: &str = r#"{"__type":"UnsupportedOperation"}"#;
const UNSUPPORTED_OUTCOME: &str = "unsupported API executor error: provider does not support this type: AWS service failure: NotSupported (UnsupportedOperation)";
const DENIED: (u16, &str) = (403, "not a JSON error");
const DENIED_OUTCOME: &str = "failed denied API executor error: access denied: HTTP 403 Forbidden: invalid response: service error JSON is not an object";

#[tokio::test]
async fn given_child_http_errors_when_probed_then_unsupported_and_denied_are_distinct() -> TestResult
{
    let bytes = fixture::compiled(1, |_| {})?;
    for (reply, outcome, totals) in [
        (
            (400, UNSUPPORTED),
            UNSUPPORTED_OUTCOME,
            "total unsupported 1",
        ),
        (DENIED, DENIED_OUTCOME, "total failed 1"),
    ] {
        let (report, requests) = run(&bytes, |request| match target(request)? {
            "ListChildren" => Ok(reply),
            _ => response(request),
        })
        .await?;
        assert_eq!(targets(&requests)?, ["ListParents0", "ListChildren"]);
        assert_row(
            &report,
            &format!("bindings ListChildren awsJson1_1 {outcome}"),
        );
        assert_row(&report, totals);
    }
    Ok(())
}

#[tokio::test]
async fn given_three_sample_slots_and_two_items_when_probed_then_both_distinct_children_are_called()
-> TestResult {
    let bytes = fixture::compiled(1, |_| {})?;
    let (report, requests) = cli::run(&bytes, &["--child-samples", "3"], response).await?;
    assert_eq!(
        targets(&requests)?,
        ["ListParents0", "ListChildren", "ListChildren"]
    );
    let mut children = requests
        .iter()
        .skip(1)
        .map(|request| body(request))
        .collect::<TestResult<Vec<_>>>()?;
    children.sort_by(|left, right| {
        left.get("EntityId")
            .and_then(Value::as_str)
            .cmp(&right.get("EntityId").and_then(Value::as_str))
    });
    assert_eq!(
        children,
        [
            json!({"OrganizationId":"org-0", "EntityId":"one", "Regions":["us-east-1"]}),
            json!({"OrganizationId":"org-0", "EntityId":"two", "Regions":["us-east-1"]})
        ]
    );
    assert_row(&report, "bindings ListChildren awsJson1_1 items 2");
    Ok(())
}

#[tokio::test]
async fn given_two_producers_with_items_when_probed_then_default_admits_one_request_for_the_operation()
-> TestResult {
    let bytes = fixture::compiled(2, |_| {})?;
    let (report, requests) = run(&bytes, |request| match target(request)? {
        "ListParents1" => Ok((200, r#"{"Items":[{"Id":"other"}]}"#)),
        _ => response(request),
    })
    .await?;
    assert_eq!(
        sorted_targets(&requests)?,
        ["ListChildren", "ListParents0", "ListParents1"]
    );
    assert_eq!(
        body(requests.last().ok_or("child not called")?)?,
        json!({"OrganizationId":"org-0", "EntityId":"one", "Regions":["us-east-1"]})
    );
    assert_row(&report, "bindings ListChildren awsJson1_1 items 1");
    assert_row(&report, "bindings ListParents1 awsJson1_1 items 1");
    Ok(())
}

#[tokio::test]
async fn given_one_failed_sibling_when_probed_then_successful_sibling_still_feeds_grandchild()
-> TestResult {
    let bytes = siblings()?;
    let (report, requests) = run(&bytes, |request| match target(request)? {
        "ListParents1" => Ok(DENIED),
        "ListParents2" => Ok((200, r#"{"Items":[{"Id":"grandchild"}]}"#)),
        _ => response(request),
    })
    .await?;
    assert_eq!(
        sorted_targets(&requests)?,
        [
            "ListChildren",
            "ListParents0",
            "ListParents1",
            "ListParents2"
        ]
    );
    assert_eq!(
        target(requests.first().ok_or("root not called")?)?,
        "ListParents0"
    );
    let grandchild = requests.last().ok_or("grandchild not called")?;
    assert_eq!(target(grandchild)?, "ListParents2");
    assert_eq!(
        body(grandchild)?,
        json!({"OrganizationId":"org-0", "EntityId":"child", "Regions":["us-east-1"]})
    );
    assert_row(
        &report,
        &format!("bindings ListParents1 awsJson1_1 {DENIED_OUTCOME}"),
    );
    assert_row(&report, "bindings ListChildren awsJson1_1 items 1");
    assert_row(&report, "bindings ListParents2 awsJson1_1 items 1");
    assert_row(
        &report,
        "protocol awsJson1_1 items 3 empty 0 not-applicable 0 unsupported 0 failed 1 skipped-child 0",
    );
    assert_row(&report, "total failed 1");
    Ok(())
}

fn siblings() -> TestResult<Vec<u8>> {
    let org = MemberPath::new("OrganizationId")?;
    let failing = FixedValue::String("org-1".into());
    let surviving_child = FixedValue::String("org-2".into());
    let failed_sibling_bindings = bindings_from("ListParents0")?;
    let grandchild_bindings = bindings_from("ListChildren")?;
    let child_bindings = bindings_from("ListParents0")?;
    fixture::compiled_with_parents(
        3,
        move |parent| {
            let bindings = match parent.fixed.get(&org) {
                Some(value) if *value == failing => &failed_sibling_bindings,
                Some(value) if *value == surviving_child => &grandchild_bindings,
                _ => return,
            };
            parent.class = Some(OverlayClass::Expand);
            parent.fixed.clear();
            parent.bind.extend(bindings.clone());
        },
        move |plan| plan.bind.extend(child_bindings),
    )
}

pub(super) fn assert_row(report: &str, row: &str) {
    assert!(
        report.lines().any(|line| line == row),
        "missing {row:?} from:\n{report}"
    );
}

fn sorted_targets(requests: &[String]) -> TestResult<Vec<&str>> {
    let mut values = targets(requests)?;
    values.sort_unstable();
    Ok(values)
}

#[tokio::test]
async fn given_a_failing_child_sample_when_probed_then_other_samples_still_run() -> TestResult {
    let bytes = fixture::compiled(1, |_| {})?;
    let (report, requests) = cli::run(&bytes, &["--child-samples", "2"], |request| {
        if target(request)? == "ListChildren"
            && body(request)?.get("EntityId") == Some(&json!("one"))
        {
            return Ok(DENIED);
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
    assert_row(
        &report,
        &format!("bindings ListChildren awsJson1_1 {DENIED_OUTCOME}"),
    );
    Ok(())
}

#[tokio::test]
async fn given_child_items_when_probed_then_grandchild_sends_item_and_ancestry() -> TestResult {
    let bytes = chain("ListParents0")?;
    let (report, requests) = run(&bytes, response).await?;
    assert_eq!(
        targets(&requests)?,
        ["ListParents0", "ListChildren", "ListParents1"]
    );
    assert_eq!(
        body(requests.get(2).ok_or("grandchild not called")?)?,
        json!({"OrganizationId":"org-0", "EntityId":"child", "Regions":["us-east-1"]})
    );
    assert_row(&report, "bindings ListParents1 awsJson1_1 empty");
    Ok(())
}

#[tokio::test]
async fn given_empty_child_page_when_probed_then_no_pagination_or_grandchild_request_is_sent()
-> TestResult {
    for (context_source, reason) in [
        ("ListChildren", "no parent items"),
        (
            "ListParents0",
            "no sampled parent item satisfies all bindings and row eligibility",
        ),
    ] {
        let bytes = chain(context_source)?;
        let (report, requests) = run(&bytes, |request| match target(request)? {
            "ListChildren" => Ok((200, r#"{"Items":[],"NextToken":"do-not-fetch"}"#)),
            _ => response(request),
        })
        .await?;
        assert_eq!(targets(&requests)?, ["ListParents0", "ListChildren"]);
        assert_row(
            &report,
            &format!("bindings ListParents1 awsJson1_1 skipped child {reason}"),
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
    let (report, requests) = run(&bytes, |request| match target(request)? {
        "ListParents1" => Ok((200, r#"{"Items":[{"Id":"unrelated"}]}"#)),
        _ => response(request),
    })
    .await?;
    assert_eq!(sorted_targets(&requests)?, ["ListParents0", "ListParents1"]);
    assert_row(
        &report,
        "bindings ListChildren awsJson1_1 skipped child no sampled parent item satisfies all bindings and row eligibility",
    );
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
    respond: impl Fn(&str) -> TestResult<(u16, &'static str)>,
) -> TestResult<(String, Vec<String>)> {
    let catalog = RuntimeCatalog::from_bytes(bytes)?;
    let server = Server::new().await?;
    let context = Context::new(&server)?;
    server.capture_with(context.report(&catalog), respond).await
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
