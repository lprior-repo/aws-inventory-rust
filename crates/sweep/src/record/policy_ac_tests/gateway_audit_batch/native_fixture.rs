use super::*;
use inventory_core::inventory::ResourceKey;

pub(super) fn child_body(
    family: &Family,
    case: &ChildCase,
    parent: &str,
    resumed: bool,
) -> TestResult<Value> {
    let suffix = if resumed { "shared" } else { "early" };
    let values = if case.operation == "ListJobs" {
        let states: &[&str] = if resumed {
            &["FAILED"]
        } else {
            &[
                "SUBMITTED",
                "PENDING",
                "RUNNABLE",
                "STARTING",
                "RUNNING",
                "SUCCEEDED",
            ]
        };
        states.iter().map(|state| json!({"jobId":format!("{parent}-{state}"),"jobName":"same","status":state})).collect::<Vec<_>>()
    } else if case.operation == "ListQuotaShares" {
        let (_, queue) = parent
            .rsplit_once('/')
            .ok_or("native queue ARN has no queue name")?;
        vec![
            json!({"quotaShareArn":format!("arn:aws:batch:us-east-1:123456789012:quota-share/{queue}/{suffix}"), "quotaShareName":suffix,"jobQueueArn":parent}),
        ]
    } else {
        vec![json!({case.id:suffix})]
    };
    let mut body = json!({case.items:values});
    if !resumed {
        body.as_object_mut()
            .ok_or("native body is not an object")?
            .insert(family.token.to_owned(), json!("later"));
    }
    Ok(body)
}

pub(super) fn page_keys(
    case: &ChildCase,
    parent: &str,
    native: &Value,
    account: &AccountId,
    region: &Region,
) -> TestResult<BTreeSet<ResourceKey>> {
    native
        .get(case.items)
        .and_then(Value::as_array)
        .ok_or("native child summaries absent")?
        .iter()
        .map(|row| {
            let id = row
                .get(case.id)
                .and_then(Value::as_str)
                .ok_or("native resource identity absent")?;
            let id = if matches!(case.operation, "ListJobs" | "ListQuotaShares") {
                id.to_owned()
            } else {
                format!("{}:{parent}|{}:{id}", parent.len(), id.len())
            };
            resource_key(account, region, case.kind, &id)
        })
        .collect()
}
