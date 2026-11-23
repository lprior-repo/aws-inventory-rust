use super::*;
#[test]
fn given_healthimaging_summary_when_arn_is_omitted_then_canonical_fallback_matches_detail_not_raw_id()
-> TestResult {
    // Given: Smithy requires datastoreId but makes the summary ARN optional.
    let id = "0123456789abcdef0123456789abcdef";
    let arn = format!("arn:aws:medical-imaging:us-east-1:123456789012:datastore/{id}");
    let bytes = compiled(
        "medical-imaging",
        "2023-07-19",
        &["ListDatastores", "GetDatastore"],
        "AWS::HealthImaging::Datastore",
    )?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let list = runtime
        .lookup("medical-imaging", "ListDatastores")
        .ok_or("list absent")?;
    let detail = runtime
        .lookup("medical-imaging", "GetDatastore")
        .ok_or("detail absent")?;
    let listed = rows(
        list,
        json!({"datastoreSummaries":[{"datastoreId":id,"datastoreName":"images","datastoreStatus":"ACTIVE"}]}),
    )?;
    let described = rows(
        detail,
        json!({"datastoreProperties":{"datastoreId":id,"datastoreName":"images","datastoreStatus":"ACTIVE","datastoreArn":arn}}),
    )?;
    // When: the summary lacks an ARN but the metadata detail has its real canonical ARN.
    let list_key = key(list, listed.first().ok_or("summary absent")?, "us-east-1")?;
    let detail_key = key(
        detail,
        described.first().ok_or("detail absent")?,
        "us-east-1",
    )?;
    // Then: the fallback is a scoped ARN and converges with the direct detail identity.
    assert_eq!(list_key, detail_key);
    assert_eq!(list_key.id.as_str(), arn);
    assert_eq!(list_key.kind, Kind::parse("AWS::HealthImaging::Datastore")?);
    // Optional ARNs in detail obey the same absent/null fallback contract as list summaries.
    for properties in [
        json!({"datastoreId":id}),
        json!({"datastoreId":id,"datastoreArn":null}),
    ] {
        let described = rows(detail, json!({"datastoreProperties":properties}))?;
        assert_eq!(
            key(
                detail,
                described.first().ok_or("fallback detail absent")?,
                "us-east-1"
            )?,
            list_key
        );
    }
    Ok(())
}

#[test]
fn given_healthimaging_explicit_arn_when_decoded_then_actual_partition_is_preserved_byte_for_byte()
-> TestResult {
    // Given: an actual ARN exists, so construction from restricted fallback context is unnecessary.
    let id = "0123456789abcdef0123456789abcdef";
    let arn = format!("arn:aws-cn:medical-imaging:cn-north-1:123456789012:datastore/{id}");
    let bytes = compiled(
        "medical-imaging",
        "2023-07-19",
        &["ListDatastores", "GetDatastore"],
        "AWS::HealthImaging::Datastore",
    )?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    // When: both real modeled views carry the explicit ARN.
    for (name, response) in [
        (
            "ListDatastores",
            json!({"datastoreSummaries":[{"datastoreId":id,"datastoreName":"images","datastoreStatus":"ACTIVE","datastoreArn":arn}]}),
        ),
        (
            "GetDatastore",
            json!({"datastoreProperties":{"datastoreId":id,"datastoreName":"images","datastoreStatus":"ACTIVE","datastoreArn":arn}}),
        ),
    ] {
        let plan = runtime
            .lookup("medical-imaging", name)
            .ok_or("plan absent")?;
        let selected = rows(plan, response)?;
        assert_eq!(selected.len(), 1);
        // Then: the real ARN wins over any fallback construction and is never shortened to datastoreId.
        assert_eq!(
            key(plan, selected.first().ok_or("row absent")?, "cn-north-1")?
                .id
                .as_str(),
            arn
        );
    }
    Ok(())
}

#[test]
fn given_healthimaging_canonical_owner_when_tag_endpoint_decodes_then_snapshot_targets_same_resource()
-> TestResult {
    // Given: the optional summary ARN is absent; the reviewed fallback still owns the real datastore.
    let id = "0123456789abcdef0123456789abcdef";
    let bytes = compiled(
        "medical-imaging",
        "2023-07-19",
        &["ListDatastores", "GetDatastore", "ListTagsForResource"],
        "AWS::HealthImaging::Datastore",
    )?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let list = runtime
        .lookup("medical-imaging", "ListDatastores")
        .ok_or("list absent")?;
    let tags = runtime
        .lookup("medical-imaging", "ListTagsForResource")
        .ok_or("tag endpoint absent")?;
    let PlanPurpose::TagEndpoint(endpoint) = tags.purpose() else {
        return Err("tag endpoint is not a tag facet".into());
    };
    let parent = endpoint.parents.first().ok_or("tag parent absent")?;
    assert_eq!(parent.operation.as_ref(), "ListDatastores");
    let request = parent.request.as_ref().ok_or("tag request absent")?;
    assert_eq!(request.source, TagEndpointSource::OwnerIdentity);
    assert!(request.parent.is_empty());
    assert_eq!(request.input.len(), 1);
    assert_eq!(
        tags.service()
            .field_name(*request.input.first().ok_or("tag input absent")?)?,
        "resourceArn"
    );
    let summaries = rows(list, json!({"datastoreSummaries":[{"datastoreId":id}]}))?;
    let owner = key(
        list,
        summaries.first().ok_or("summary absent")?,
        "us-east-1",
    )?;
    assert_eq!(
        owner.id.as_str(),
        format!("arn:aws:medical-imaging:us-east-1:123456789012:datastore/{id}")
    );
    // When: the production wire decoder and tag extractor consume populated and empty map snapshots.
    for (response, expected) in [
        (
            json!({"tags":{"owner":"radiology"}}),
            vec![Tag::new("owner", "radiology")?],
        ),
        (json!({"tags":{}}), vec![]),
    ] {
        let response_rows = rows(tags, response)?;
        let row = response_rows.first().ok_or("tag response absent")?;
        let snapshot = tag_observation(row, &owner, &endpoint.tags)?;
        // Then: no fabricated tag resource is created; explicit emptiness is authoritative.
        assert_eq!(snapshot.resource(), &owner);
        assert_eq!(snapshot.coverage(), TagCoverage::Complete);
        assert_eq!(snapshot.tags(), expected);
    }
    Ok(())
}

#[test]
fn given_healthimaging_present_bad_arn_when_decoded_then_canonical_fallback_is_not_used()
-> TestResult {
    let id = "0123456789abcdef0123456789abcdef";
    let bytes = compiled(
        "medical-imaging",
        "2023-07-19",
        &["ListDatastores", "GetDatastore"],
        "AWS::HealthImaging::Datastore",
    )?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    for arn in ["", "not-an-arn"] {
        for (operation, response) in [
            (
                "ListDatastores",
                json!({"datastoreSummaries":[{"datastoreId":id,"datastoreArn":arn}]}),
            ),
            (
                "GetDatastore",
                json!({"datastoreProperties":{"datastoreId":id,"datastoreArn":arn}}),
            ),
        ] {
            let plan = runtime
                .lookup("medical-imaging", operation)
                .ok_or("operation absent")?;
            let selected = rows(plan, response)?;
            assert!(key(plan, selected.first().ok_or("row absent")?, "us-east-1").is_err());
        }
    }
    Ok(())
}
