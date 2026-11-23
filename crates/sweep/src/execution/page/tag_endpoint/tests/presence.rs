use super::*;

fn decode(
    plan: &CallablePlan<'_>,
    body: &str,
    limits: aws_wire::response::DecodeLimits,
) -> TestResult<Record> {
    Ok(aws_wire::response::decode(
        aws_wire::response::ResponseContext {
            service: plan.service(),
            operation: plan.operation(),
            protocol: aws_wire::request::Protocol::AwsQuery,
            body: body.as_bytes(),
            headers: &http::HeaderMap::new(),
            status: http::StatusCode::OK,
            limits,
        },
    )?)
}

#[test]
fn given_required_tag_collection_when_absent_then_snapshot_mode_preserves_absence_without_changing_sdk_defaults()
-> TestResult {
    let compiled = iam()?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog
        .lookup("iam", "ListRoleTags")
        .ok_or("tag endpoint missing")?;
    let tags = compiled.fields.id("Tags").ok_or("Tags field missing")?;
    let truncated = compiled
        .fields
        .id("IsTruncated")
        .ok_or("IsTruncated field missing")?;
    let absent = "<ListRoleTagsResponse><ListRoleTagsResult/></ListRoleTagsResponse>";
    // Given: the pinned Smithy response marks Tags as required and defaults IsTruncated=false.
    let sdk = decode(plan, absent, aws_wire::response::DecodeLimits::default())?;
    let Some(ValueRef::List(sdk_tags)) = sdk.root().get_field(tags)? else {
        return Err("normal SDK collection correction changed".into());
    };
    assert_eq!(sdk_tags.count()?, 0);
    // When: the dedicated endpoint decoder opts into preserving collection presence.
    let strict = decode(
        plan,
        absent,
        aws_wire::response::DecodeLimits::new(65_536, 1024, 16)?.preserve_field_presence(),
    )?;
    // Then: neither tag content nor pagination evidence is invented.
    assert!(strict.root().get_field(tags)?.is_none());
    assert!(strict.root().get_field(truncated)?.is_none());
    assert!(matches!(
        sdk.root().get_field(truncated)?,
        Some(ValueRef::Bool(false))
    ));
    let present = decode(
        plan,
        "<ListRoleTagsResponse><ListRoleTagsResult><Tags/></ListRoleTagsResult></ListRoleTagsResponse>",
        aws_wire::response::DecodeLimits::default().preserve_field_presence(),
    )?;
    let Some(ValueRef::List(explicit_empty)) = present.root().get_field(tags)? else {
        return Err("explicit empty Tags was lost".into());
    };
    assert_eq!(explicit_empty.count()?, 0);
    Ok(())
}

#[tokio::test]
async fn given_missing_required_tag_scalars_or_completion_flag_when_decoded_then_known_snapshot_survives()
-> TestResult {
    let compiled = iam()?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog
        .lookup("iam", "ListRoleTags")
        .ok_or("tag endpoint missing")?;
    // Malformed tag pairs are skipped (ES-01); the snapshot stays stale.
    // Only the completion-flag case still hard-fails at decode time.
    let skip = [
        (
            "value",
            "<Tags><member><Key>Owner</Key></member></Tags><IsTruncated>false</IsTruncated>",
        ),
        (
            "key",
            "<Tags><member><Value>value</Value></member></Tags><IsTruncated>false</IsTruncated>",
        ),
    ];
    for (name, body) in skip {
        let (directory, store, owner, sweep) =
            seeded(&compiled, &format!("missing-{name}")).await?;
        let lease = claim(&store, &sweep, 2).await?;
        let body = format!(
            "<ListRoleTagsResponse><ListRoleTagsResult>{body}</ListRoleTagsResult></ListRoleTagsResponse>"
        );
        let record = decode(
            plan,
            &body,
            aws_wire::response::DecodeLimits::default().preserve_field_presence(),
        )?;
        let response = call::DecodedResponse {
            http_version: http::Version::HTTP_11,
            record,
            headers: http::HeaderMap::new(),
        };
        // Malformed pair is skipped; consume returns Ok with Partial coverage.
        let result = consume(&catalog, &lease, response, 10);
        assert!(
            matches!(result, Ok(PageParts { .. })),
            "malformed tag pair should be skipped, not fail: {name}"
        );
        store.finish_scope(lease, ScopeOutcome::Failed, 3).await?;
        assert_eq!(
            tags(&store, &owner).await?,
            vec![("known".into(), "old".into())],
            "snapshot unchanged after skipped malformed pair: {name}"
        );
        drop(store);
        directory.close()?;
    }
    // Completion flag still hard-fails.
    let (directory, store, owner, sweep) = seeded(&compiled, "missing-completion").await?;
    let lease = claim(&store, &sweep, 2).await?;
    let body =
        "<ListRoleTagsResponse><ListRoleTagsResult><Tags/></ListRoleTagsResult></ListRoleTagsResponse>".to_owned();
    let record = decode(
        plan,
        &body,
        aws_wire::response::DecodeLimits::default().preserve_field_presence(),
    )?;
    let response = call::DecodedResponse {
        http_version: http::Version::HTTP_11,
        record,
        headers: http::HeaderMap::new(),
    };
    assert!(
        matches!(consume(&catalog, &lease, response, 10), Err(error) if matches!(error.downcast_ref::<SweepError>(), Some(SweepError::Operation(message)) if *message == "tag response omitted explicit pagination completion evidence"))
    );
    store.finish_scope(lease, ScopeOutcome::Failed, 3).await?;
    assert_eq!(
        tags(&store, &owner).await?,
        vec![("known".into(), "old".into())]
    );
    drop(store);
    directory.close()?;
    Ok(())
}
