use aws_sdk_sfn::operation::{
    list_activities::ListActivitiesOutput, list_state_machines::ListStateMachinesOutput,
};

use super::*;

#[test]
fn state_machine_page_keeps_the_token_and_counts_attempted_items() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    let item = state_machine_item(STATE_MACHINE_ARN, STATE_MACHINE_NAME)?;
    let described = state_machine_description(STATE_MACHINE_ARN, STATE_MACHINE_NAME, None)?;
    let observation = map_state_machine(&ctx, &item, Some(&described), None)?;
    let output = ListStateMachinesOutput::builder()
        .state_machines(item.clone())
        .state_machines(item)
        .next_token("cursor-2")
        .build()?;
    let page = state_machine_page_body(&output, dedupe(vec![observation.clone(), observation]));
    assert_eq!(page.next_token.as_deref(), Some("cursor-2"));
    assert_eq!(page.listed_count, 2);
    assert_eq!(page.resources.len(), 1);
    assert_eq!(page.edges.len(), 1);

    let terminal = state_machine_page_body(
        &ListStateMachinesOutput::builder()
            .state_machines(state_machine_item(STATE_MACHINE_ARN, STATE_MACHINE_NAME)?)
            .build()?,
        Vec::new(),
    );
    assert!(terminal.next_token.is_none());
    assert_eq!(terminal.listed_count, 1);
    assert!(terminal.resources.is_empty());
    Ok(())
}

#[test]
fn activity_page_keeps_the_token_and_counts_attempted_items() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    let item = activity_item(ACTIVITY_ARN, ACTIVITY_NAME)?;
    let described = activity_description(ACTIVITY_ARN, ACTIVITY_NAME, None)?;
    let observation = map_activity(&ctx, &item, &described, None)?;
    let output = ListActivitiesOutput::builder()
        .activities(item)
        .next_token("cursor-3")
        .build()?;
    let page = activity_page_body(&output, vec![observation]);
    assert_eq!(page.next_token.as_deref(), Some("cursor-3"));
    assert_eq!(page.listed_count, 1);
    assert_eq!(page.resources.len(), 1);
    assert!(page.edges.is_empty());
    Ok(())
}

#[test]
fn duplicate_identities_on_one_page_are_collapsed() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    let item = state_machine_item(STATE_MACHINE_ARN, STATE_MACHINE_NAME)?;
    let described = state_machine_description(STATE_MACHINE_ARN, STATE_MACHINE_NAME, None)?;
    let first = map_state_machine(&ctx, &item, Some(&described), None)?;
    let second = map_state_machine(&ctx, &item, Some(&described), None)?;
    let deduped = dedupe(vec![first, second]);
    assert_eq!(deduped.len(), 1);
    assert_eq!(
        deduped
            .first()
            .map(|observation| observation.resource.id.as_str()),
        Some(STATE_MACHINE_ARN)
    );
    Ok(())
}

#[test]
fn list_page_size_is_clamped_to_the_service_bounds() {
    assert_eq!(page_limit(0), 1);
    assert_eq!(page_limit(100), 100);
    assert_eq!(page_limit(5_000), LIST_MAX_RESULTS);
}
