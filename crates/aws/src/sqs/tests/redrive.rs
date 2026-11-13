use std::collections::BTreeMap;

use aws_sdk_sqs::types::QueueAttributeName;
use serde_json::json;

use super::*;

#[test]
fn redrive_policy_emits_only_identifiable_delivery_edges() -> TestResult {
    let (ctx, _receiver) = context()?;
    let url = queue_url()?;
    for (policy, expected_edges, expected_count) in [
        (
            format!(r#"{{"deadLetterTargetArn":"{DLQ}","maxReceiveCount":"5"}}"#),
            1,
            Some(5),
        ),
        (
            format!(r#"{{"deadLetterTargetArn":"{DLQ}","maxReceiveCount":7}}"#),
            1,
            Some(7),
        ),
        ("not-json".to_owned(), 0, None),
        (
            r#"{"deadLetterTargetArn":"not-an-arn","maxReceiveCount":"5"}"#.to_owned(),
            0,
            Some(5),
        ),
        (format!(r#"{{"deadLetterTargetArn":"{ARN}"}}"#), 0, None),
    ] {
        let attrs = attributes(&[
            (QueueAttributeName::QueueArn, ARN),
            (QueueAttributeName::RedrivePolicy, &policy),
        ]);
        let observation = map_queue(&ctx, URL, &url, &attrs, Some(BTreeMap::new()))?;
        assert_eq!(observation.edges.len(), expected_edges, "policy: {policy}");
        let actual_count = observation
            .resources
            .first()
            .and_then(|resource| resource.properties.get("redrive_max_receive_count"))
            .and_then(serde_json::Value::as_u64);
        assert_eq!(actual_count, expected_count, "policy: {policy}");
        if let Some(edge) = observation.edges.first() {
            assert_eq!(edge.relation, "DELIVERS_TO");
            assert_eq!(edge.to_id, DLQ);
            assert_eq!(
                edge.properties.get("evidence"),
                Some(&json!("redrive_policy"))
            );
            let expected_edge_count =
                expected_count.map_or(serde_json::Value::Null, |count| json!(count));
            assert_eq!(
                edge.properties.get("max_receive_count"),
                Some(&expected_edge_count)
            );
        }
    }
    Ok(())
}
