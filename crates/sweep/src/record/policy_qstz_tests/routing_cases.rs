use super::*;
#[tokio::test]
async fn given_two_outposts_when_bucket_pages_fan_out_then_wire_signing_headers_and_bucket_arns_are_correct()
-> TestResult {
    use aws_wire::sign::{self, Body, Credentials, PathMode, PayloadHeader, Scope, SigningName};
    let catalog = RuntimeCatalog::from_bytes(bytes()?)?;
    let parent = catalog
        .lookup("outposts", "ListOutposts")
        .ok_or("Outposts parent absent")?;
    let plan = catalog
        .lookup("s3-control", "ListRegionalBuckets")
        .ok_or("regional buckets absent")?;
    let outposts = decode(parent, br#"{"Outposts":[{"OutpostId":"op-01234567890123456","OutpostArn":"arn:aws:outposts:us-east-1:123456789012:outpost/op-01234567890123456"},{"OutpostId":"op-fedcba98765432100","OutpostArn":"arn:aws:outposts:us-east-1:123456789012:outpost/op-fedcba98765432100"}]}"#)?;
    let parent_bindings = request_bindings(parent, &[])?;
    let owners = observations_with_request(
        parent,
        item_records(&outposts, parent.items())?,
        &parent_bindings,
    )?;
    let mut pages = Vec::new();
    let mut expected = Vec::new();
    for owner in &owners {
        let row = owner.record();
        let fields = child_fields(&catalog, parent, plan, row)?;
        let first = request(plan, &fields)?;
        assert_eq!(
            first.uri().host(),
            Some("s3-outposts.us-east-1.amazonaws.com")
        );
        assert_eq!(
            first
                .headers()
                .get("x-amz-account-id")
                .ok_or("account header absent")?,
            "123456789012"
        );
        let outpost = first
            .headers()
            .get("x-amz-outpost-id")
            .ok_or("Outpost header absent")?
            .to_str()?;
        assert!(matches!(
            outpost,
            "op-01234567890123456" | "op-fedcba98765432100"
        ));
        let endpoint =
            crate::execution::routing::resolve(plan, &RegionId::parse("us-east-1")?)?;
        let scope = Scope {
            region: sign::Region::new(&endpoint.signing_region)?,
            name: SigningName::new(&endpoint.signing_name)?,
        };
        let uri = first.uri().to_string();
        let signed = sign::sign(
            &Credentials::new("A", "secret", None)?,
            &scope,
            std::time::SystemTime::UNIX_EPOCH,
            sign::Request {
                method: first.method(),
                uri: &uri,
                headers: first.headers(),
                body: Body::Bytes(first.body()),
                payload_header: PayloadHeader::Omit,
            },
            PathMode::Standard,
        )?;
        assert!(
            signed
                .get("authorization")
                .ok_or("authorization absent")?
                .to_str()?
                .contains("/us-east-1/s3-outposts/aws4_request,")
        );
        let mut rows = Vec::new();
        for page in 1..=2 {
            let arn = format!(
                "arn:aws:s3-outposts:us-east-1:123456789012:outpost/{outpost}/bucket/bucket-{page}"
            );
            let token = if page == 1 {
                "<NextToken>page-two</NextToken>"
            } else {
                ""
            };
            let body = format!(
                "<ListRegionalBucketsResult><RegionalBucketList><RegionalBucket><Bucket>bucket-{page}</Bucket><BucketArn>{arn}</BucketArn><OutpostId>{outpost}</OutpostId><PublicAccessBlockEnabled>false</PublicAccessBlockEnabled><CreationDate>2026-01-01T00:00:00Z</CreationDate></RegionalBucket></RegionalBucketList>{token}</ListRegionalBucketsResult>"
            );
            let response = decode(plan, body.as_bytes())?;
            if page == 1 {
                let token = plan
                    .pagination()
                    .tokens
                    .first()
                    .ok_or("regional bucket cursor absent")?;
                let value = path_value(response.root(), &token.response_path)?
                    .ok_or("bucket response cursor absent")?
                    .to_value()?;
                let mut second_fields = child_fields(&catalog, parent, plan, row)?;
                second_fields.push(crate::input::InputField {
                    path: token.request_path.clone(),
                    value,
                });
                let second = request(plan, &second_fields)?;
                assert_eq!(
                    second.headers().get("x-amz-outpost-id"),
                    first.headers().get("x-amz-outpost-id")
                );
                assert!(
                    second
                        .uri()
                        .query()
                        .ok_or("bucket page query absent")?
                        .split('&')
                        .any(|pair| pair == "nextToken=page-two")
                );
            }
            rows.extend(item_records(&response, plan.items())?);
            expected.push(arn);
        }
        let bindings = request_bindings(plan, &fields)?;
        let observations = observations_with_request(plan, rows, &bindings)?;
        pages.push((plan, Some(owner.key().clone()), bindings, observations));
    }
    pages.insert(0, (parent, None, parent_bindings, owners));
    assert_eq!(expected.len(), 4);
    expected.sort();
    let (persisted, _) = persist(plan, pages).await?;
    assert_eq!(
        persisted
            .iter()
            .map(|(_, id, _)| id.clone())
            .collect::<Vec<_>>(),
        expected
    );
    assert!(
        persisted
            .iter()
            .all(|(kind, _, _)| kind == plan.kind().as_str())
    );
    Ok(())
}

#[test]
fn given_optional_route53_id_prefixes_when_child_requests_bind_then_only_request_ids_normalize()
-> TestResult {
    let catalog = RuntimeCatalog::from_bytes(bytes()?)?;
    for (list, get, field, wrapper, segment, prefix) in [
        (
            "ListHostedZones",
            "GetHostedZone",
            "HostedZones",
            "HostedZone",
            "hostedzone",
            "/hostedzone/",
        ),
        (
            "ListReusableDelegationSets",
            "GetReusableDelegationSet",
            "DelegationSets",
            "DelegationSet",
            "delegationset",
            "/delegationset/",
        ),
    ] {
        let parent = catalog
            .lookup("route-53", list)
            .ok_or("Route53 list absent")?;
        let child = catalog
            .lookup("route-53", get)
            .ok_or("Route53 detail absent")?;
        for identifier in [format!("{prefix}Z123456"), "Z123456".to_owned()] {
            let body = format!(
                "<{list}Response><{field}><{wrapper}><Id>{identifier}</Id></{wrapper}></{field}></{list}Response>"
            );
            let response = decode(parent, body.as_bytes())?;
            let row = item_records(&response, parent.items())?
                .into_iter()
                .next()
                .ok_or("Route53 row absent")?;
            let original_id = resource_id(&row, parent.identity(), None)?;
            assert_eq!(original_id.as_str(), identifier);
            let encoded = request(child, &child_fields(&catalog, parent, child, &row)?)?;
            assert_eq!(
                encoded.uri().path(),
                format!("/2013-04-01/{segment}/Z123456")
            );
            assert!(!encoded.uri().path().contains("%2F"));
            assert_eq!(resource_id(&row, parent.identity(), None)?, original_id);
            if list == "ListHostedZones" {
                let traffic = catalog
                    .lookup("route-53", "ListTrafficPolicyInstancesByHostedZone")
                    .ok_or("zone traffic instances absent")?;
                let encoded = request(traffic, &child_fields(&catalog, parent, traffic, &row)?)?;
                assert!(
                    encoded
                        .uri()
                        .query()
                        .ok_or("zone traffic query absent")?
                        .split('&')
                        .any(|pair| pair == "id=Z123456")
                );
                assert!(!encoded.uri().to_string().contains("%2F"));
            }
        }
        for identifier in [
            "".to_owned(),
            prefix.to_owned(),
            format!("{prefix}/Z123456"),
            "/wrong/Z123456".to_owned(),
        ] {
            let body = format!(
                "<{list}Response><{field}><{wrapper}><Id>{identifier}</Id></{wrapper}></{field}></{list}Response>"
            );
            let response = decode(parent, body.as_bytes())?;
            let row = item_records(&response, parent.items())?
                .into_iter()
                .next()
                .ok_or("malformed Route53 row absent")?;
            assert!(
                child_fields(&catalog, parent, child, &row).is_err(),
                "malformed identifier was accepted: {identifier}"
            );
        }
    }
    Ok(())
}

#[test]
fn given_s3_global_and_directory_buckets_when_routes_resolve_then_request_destinations_are_operation_specific()
-> TestResult {
    let catalog = RuntimeCatalog::from_bytes(bytes()?)?;
    let mrap = catalog
        .lookup("s3-control", "ListMultiRegionAccessPoints")
        .ok_or("MRAP absent")?;
    let scope = inventory_core::inventory::Region::Global;
    let request_region = crate::execution::routing::request_region(
        mrap,
        &scope,
        &RegionId::parse("eu-west-1")?,
        None,
    )?;
    assert_eq!(request_region.as_str(), "us-west-2");
    assert_eq!(
        request(mrap, &[])?.uri().host(),
        Some("123456789012.s3-control.us-west-2.amazonaws.com")
    );
    let directory = catalog
        .lookup("s3-control", "ListAccessPointsForDirectoryBuckets")
        .ok_or("directory access points absent")?;
    let dir_req = request(directory, &[])?;
    assert_eq!(
        dir_req.uri().host(),
        Some("s3express-control.us-east-1.amazonaws.com")
    );
    assert_eq!(
        dir_req
            .headers()
            .get("x-amz-account-id")
            .ok_or("account header absent")?,
        "123456789012"
    );
    assert_eq!(
        crate::execution::routing::resolve(directory, &RegionId::parse("us-east-1")?)?
            .signing_name,
        "s3express"
    );
    Ok(())
}
