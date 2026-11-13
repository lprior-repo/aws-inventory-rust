use aws_sdk_sns::operation::{
    list_subscriptions::ListSubscriptionsOutput, list_topics::ListTopicsOutput,
};
use serde_json::json;

use super::*;

#[test]
fn topic_page_keeps_the_terminal_token_and_counts_attempted_topics() -> TestResult {
    let output = ListTopicsOutput::builder()
        .topics(topic(TOPIC_ARN))
        .topics(topic("arn:aws:sns:us-east-1:111111111111:audit"))
        .next_token("cursor-2")
        .build();
    let page = topic_page_body(&output, Vec::new());
    assert_eq!(page.next_token.as_deref(), Some("cursor-2"));
    assert_eq!(page.listed_count, 2);
    assert!(page.resources.is_empty());

    let terminal = topic_page_body(
        &ListTopicsOutput::builder().topics(topic(TOPIC_ARN)).build(),
        Vec::new(),
    );
    assert!(terminal.next_token.is_none());
    assert_eq!(terminal.listed_count, 1);
    Ok(())
}

#[test]
fn subscription_page_counts_filtered_rows_and_keeps_edges() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    let item = subscription(
        SUBSCRIPTION_ARN,
        "sqs",
        Some("arn:aws:sqs:us-east-1:111111111111:orders"),
        Some(TOPIC_ARN),
    );
    let observation = map_subscription(&ctx, &item)?.ok_or("in-scope subscription filtered")?;
    let output = ListSubscriptionsOutput::builder()
        .subscriptions(item.clone())
        .subscriptions(item)
        .next_token("cursor-3")
        .build();
    let page = subscription_page_body(&output, vec![observation]);
    assert_eq!(page.next_token.as_deref(), Some("cursor-3"));
    assert_eq!(page.listed_count, 2);
    assert_eq!(page.resources.len(), 1);
    assert_eq!(page.edges.len(), 1);
    let edge = page.edges.first().ok_or("missing subscription edge")?;
    assert_eq!(edge.relation, "SUBSCRIBES_TO");
    assert_eq!(edge.properties.get("evidence"), Some(&json!("topic_arn")));

    let terminal = subscription_page_body(&ListSubscriptionsOutput::builder().build(), Vec::new());
    assert!(terminal.next_token.is_none());
    assert_eq!(terminal.listed_count, 0);
    Ok(())
}
