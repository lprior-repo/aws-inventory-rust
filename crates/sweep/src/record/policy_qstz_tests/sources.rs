use super::*;
#[tokio::test]
async fn given_all_source_access_points_on_two_pages_when_wire_decoded_then_every_canonical_arn_persists()
-> TestResult {
    let catalog = RuntimeCatalog::from_bytes(bytes()?)?;
    let plan = catalog
        .lookup("s3-control", "ListAccessPoints")
        .ok_or("access-point plan absent")?;
    let first_request = request(plan, &[])?;
    assert!(
        first_request
            .uri()
            .query()
            .ok_or("query absent")?
            .split('&')
            .any(|pair| pair == "dataSourceType=ALL")
    );
    assert_eq!(
        first_request.uri().host(),
        Some("123456789012.s3-control.us-east-1.amazonaws.com")
    );
    assert_eq!(
        first_request
            .headers()
            .get("x-amz-account-id")
            .ok_or("account header absent")?,
        "123456789012"
    );
    let first = decode(plan, br#"<ListAccessPointsResult><AccessPointList><AccessPoint><Name>ordinary</Name><NetworkOrigin>Internet</NetworkOrigin><Bucket>bucket</Bucket><AccessPointArn>arn:aws:s3:us-east-1:123456789012:accesspoint/ordinary</AccessPointArn><DataSourceType>S3</DataSourceType></AccessPoint></AccessPointList><NextToken>second-page</NextToken></ListAccessPointsResult>"#)?;
    let token = plan
        .pagination()
        .tokens
        .first()
        .ok_or("compiled cursor absent")?;
    let cursor = path_value(first.root(), &token.response_path)?
        .ok_or("continuation absent")?
        .to_value()?;
    let next_request = request(
        plan,
        &[crate::input::InputField {
            path: token.request_path.clone(),
            value: cursor,
        }],
    )?;
    assert!(
        next_request
            .uri()
            .query()
            .ok_or("next query absent")?
            .split('&')
            .any(|pair| pair == "nextToken=second-page")
    );
    let second = decode(plan, br#"<ListAccessPointsResult><AccessPointList><AccessPoint><Name>filesystem</Name><NetworkOrigin>VPC</NetworkOrigin><Bucket>filesystem</Bucket><AccessPointArn>arn:aws:s3:us-east-1:123456789012:accesspoint/filesystem</AccessPointArn><DataSourceType>FSX_OPENZFS</DataSourceType></AccessPoint></AccessPointList></ListAccessPointsResult>"#)?;
    let mut rows = item_records(&first, plan.items())?;
    rows.extend(item_records(&second, plan.items())?);
    let bindings = request_bindings(plan, &[])?;
    let observations = observations_with_request(plan, rows, &bindings)?;
    let (persisted, _) = persist(plan, vec![(plan, None, bindings, observations)]).await?;
    assert_eq!(
        persisted
            .iter()
            .map(|(kind, id, _)| (kind.as_str(), id.as_str()))
            .collect::<Vec<_>>(),
        [
            (
                "AWS::S3::AccessPoint",
                "arn:aws:s3:us-east-1:123456789012:accesspoint/filesystem"
            ),
            (
                "AWS::S3::AccessPoint",
                "arn:aws:s3:us-east-1:123456789012:accesspoint/ordinary"
            ),
        ]
    );
    Ok(())
}

#[tokio::test]
async fn given_agreements_on_two_servers_when_details_bind_then_exact_pairs_safe_fields_and_tags_persist()
-> TestResult {
    let catalog = RuntimeCatalog::from_bytes(bytes()?)?;
    let list = catalog
        .lookup("transfer", "ListAgreements")
        .ok_or("agreement list absent")?;
    let detail = catalog
        .lookup("transfer", "DescribeAgreement")
        .ok_or("agreement detail absent")?;
    let page = decode(list, br#"{"Agreements":[{"Arn":"arn:aws:transfer:us-east-1:123456789012:agreement/s-one/a-one","AgreementId":"a-one","ServerId":"s-one"},{"Arn":"arn:aws:transfer:us-east-1:123456789012:agreement/s-two/a-two","AgreementId":"a-two","ServerId":"s-two"}],"NextToken":"next"}"#)?;
    assert!(
        path_value(
            page.root(),
            &list
                .pagination()
                .tokens
                .first()
                .ok_or("list token absent")?
                .response_path
        )?
        .is_some()
    );
    let mut pairs = Vec::new();
    let mut first_bindings = None;
    for row in item_records(&page, list.items())? {
        let fields = child_fields(&catalog, list, detail, &row)?;
        let encoded = request(detail, &fields)?;
        if first_bindings.is_none() {
            first_bindings = Some(request_bindings(detail, &fields)?);
        }
        let body: serde_json::Value = serde_json::from_slice(encoded.body())?;
        pairs.push((
            body.get("ServerId")
                .and_then(serde_json::Value::as_str)
                .ok_or("server absent")?
                .to_owned(),
            body.get("AgreementId")
                .and_then(serde_json::Value::as_str)
                .ok_or("agreement absent")?
                .to_owned(),
        ));
    }
    assert_eq!(
        pairs,
        [
            ("s-one".to_owned(), "a-one".to_owned()),
            ("s-two".to_owned(), "a-two".to_owned())
        ]
    );
    let response = decode(detail, br#"{"Agreement":{"Arn":"arn:aws:transfer:us-east-1:123456789012:agreement/s-one/a-one","AgreementId":"a-one","ServerId":"s-one","BaseDirectory":"/bucket/inbound","AccessRole":"arn:aws:iam::123456789012:role/transfer","Tags":[{"Key":"team","Value":"inventory"}]}}"#)?;
    let rows = item_records(&response, detail.items())?;
    let expected = rows.first().ok_or("detail row absent")?.as_bytes().to_vec();
    let bindings = first_bindings.ok_or("detail request absent")?;
    let observations = observations_with_request(detail, rows, &bindings)?;
    let (persisted, tags) = persist(detail, vec![(detail, None, bindings, observations)]).await?;
    let (kind, id, record) = persisted.first().ok_or("detail not persisted")?;
    assert_eq!(persisted.len(), 1);
    assert_eq!(kind, "AWS::Transfer::Agreement");
    assert_eq!(
        id,
        "arn:aws:transfer:us-east-1:123456789012:agreement/s-one/a-one"
    );
    assert_eq!(record, &expected);
    assert_eq!(tags, [("team".to_owned(), "inventory".to_owned())]);
    Ok(())
}
