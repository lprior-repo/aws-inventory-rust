use super::*;

#[test]
fn key_page_keeps_the_marker_and_counts_unenriched_keys() -> TestResult {
    let output = ListKeysOutput::builder()
        .keys(key_entry(Some(KEY_ARN), Some(KEY_ID)))
        .keys(key_entry(
            Some("arn:aws:kms:us-east-1:111111111111:key/2234abcd-12ab-34cd-56ef-1234567890ab"),
            None,
        ))
        .next_marker("cursor-2")
        .build();
    let page = key_page_body(&output, Vec::new());
    assert_eq!(page.next_token.as_deref(), Some("cursor-2"));
    assert_eq!(page.listed_count, 2);
    assert!(page.resources.is_empty());

    let terminal = key_page_body(
        &ListKeysOutput::builder()
            .keys(key_entry(Some(KEY_ARN), None))
            .build(),
        Vec::new(),
    );
    assert!(terminal.next_token.is_none());
    assert_eq!(terminal.listed_count, 1);
    Ok(())
}

#[test]
fn alias_page_keeps_the_marker_and_counts_mapped_aliases() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    let resource = map_alias(
        &ctx,
        &alias_entry(Some(ALIAS_ARN), Some(ALIAS_NAME), Some(KEY_ID)),
    )?;
    let output = ListAliasesOutput::builder()
        .aliases(alias_entry(Some(ALIAS_ARN), Some(ALIAS_NAME), Some(KEY_ID)))
        .next_marker("cursor-3")
        .build();
    let page = alias_page_body(&output, vec![resource]);
    assert_eq!(page.next_token.as_deref(), Some("cursor-3"));
    assert_eq!(page.listed_count, 1);
    assert_eq!(page.resources.len(), 1);

    let terminal = alias_page_body(&ListAliasesOutput::builder().build(), Vec::new());
    assert!(terminal.next_token.is_none());
    assert_eq!(terminal.listed_count, 0);
    Ok(())
}

#[test]
fn duplicate_identities_on_one_page_are_collapsed() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    let entry = alias_entry(Some(ALIAS_ARN), Some(ALIAS_NAME), Some(KEY_ID));
    let first = map_alias(&ctx, &entry)?;
    let second = map_alias(&ctx, &entry)?;
    let deduped = dedupe(vec![first, second]);
    assert_eq!(deduped.len(), 1);
    assert_eq!(
        deduped.first().map(|resource| resource.id.as_str()),
        Some(ALIAS_ARN)
    );
    Ok(())
}

#[test]
fn list_page_size_is_clamped_to_the_service_bounds() {
    assert_eq!(page_limit(0), 1);
    assert_eq!(page_limit(100), 100);
    assert_eq!(page_limit(5_000), LIST_MAX_RESULTS);
    assert_eq!(MAX_TAGS_PER_KEY, 50);
}
