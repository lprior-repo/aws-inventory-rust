use super::*;
#[test]
fn given_real_list_and_detail_responses_when_selected_then_three_views_converge_on_one_identity()
-> TestResult {
    let cases = [
        (
            "apigatewayv2",
            "2018-11-29",
            "GetDomainNames",
            "GetDomainName",
            json!({"items":[{"domainName":"api.example.test","domainNameArn":"arn:aws:apigateway:us-east-1::/domainnames/api.example.test"}]}),
            json!({"domainName":"api.example.test","domainNameArn":"arn:aws:apigateway:us-east-1::/domainnames/api.example.test"}),
            "arn:aws:apigateway:us-east-1::/domainnames/api.example.test",
        ),
        (
            "appconfig",
            "2019-10-09",
            "ListExtensionAssociations",
            "GetExtensionAssociation",
            json!({"Items":[{"Id":"abc1234","ExtensionArn":"arn:aws:appconfig:us-east-1:123456789012:extension/ex1"}]}),
            json!({"Id":"abc1234","Arn":"arn:aws:appconfig:us-east-1:123456789012:extensionassociation/abc1234"}),
            "abc1234",
        ),
        (
            "cloudwatch-events",
            "2015-10-07",
            "ListArchives",
            "DescribeArchive",
            json!({"Archives":[{"ArchiveName":"nightly","EventSourceArn":"arn:aws:events:us-east-1:123456789012:event-bus/main"}]}),
            json!({"ArchiveName":"nightly","ArchiveArn":"arn:aws:events:us-east-1:123456789012:archive/nightly"}),
            "nightly",
        ),
    ];
    for (service, date, list, detail, list_body, detail_body, expected) in cases {
        let compiled = compile(service, date, &[list, detail])?;
        let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
        let list_plan = catalog.lookup(service, list).ok_or("list plan missing")?;
        let detail_plan = catalog
            .lookup(service, detail)
            .ok_or("detail plan missing")?;
        let list_id = only_id(list_plan, &list_body)?;
        let detail_id = only_id(detail_plan, &detail_body)?;
        assert_eq!(list_id, detail_id, "{service}");
        assert_eq!(list_id.as_str(), expected);
        assert_eq!(list_plan.kind(), detail_plan.kind());
        let malformed = decode(detail_plan, &json!({}))?;
        let rows = item_records(&malformed, detail_plan.items())?;
        assert!(
            resource_id(
                rows.first().ok_or("detail root absent")?,
                detail_plan.identity(),
                None
            )
            .is_err()
        );
    }
    Ok(())
}

#[test]
fn given_child_baselines_and_application_profiles_when_requested_then_the_real_wire_filters_are_complete()
-> TestResult {
    let cases = [
        (
            "controltower",
            "2018-05-10",
            "ListEnabledBaselines",
            "includeChildren",
            json!(true),
            json!({"enabledBaselines":[{"arn":"arn:aws:controltower:us-east-1:123456789012:enabledbaseline/child"}]}),
            "arn:aws:controltower:us-east-1:123456789012:enabledbaseline/child",
        ),
        (
            "bedrock",
            "2023-04-20",
            "ListInferenceProfiles",
            "type",
            json!("APPLICATION"),
            json!({"inferenceProfileSummaries":[{"inferenceProfileArn":"arn:aws:bedrock:us-east-1:123456789012:application-inference-profile/profile1"}]}),
            "arn:aws:bedrock:us-east-1:123456789012:application-inference-profile/profile1",
        ),
    ];
    for (service, date, operation, field, expected, response, id) in cases {
        let compiled = compile(service, date, &[operation])?;
        let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
        let plan = catalog
            .lookup(service, operation)
            .ok_or("filtered plan missing")?;
        let base = crate::input::root_bindings(plan, None, None)?;
        let request = crate::request::encode(plan, "https://fixture.example.test", &base, &[])?;
        let actual: Json = if request.body().is_empty() {
            let query = request.uri().query().ok_or("fixed query filter absent")?;
            let expected_pair = format!(
                "{field}={}",
                expected.as_str().ok_or("query string expected")?
            );
            assert!(query.split('&').any(|pair| pair == expected_pair));
            expected.clone()
        } else {
            serde_json::from_slice::<Json>(request.body())?
                .get(field)
                .ok_or("fixed body filter absent")?
                .clone()
        };
        assert_eq!(actual, expected);
        assert_eq!(only_id(plan, &response)?.as_str(), id);
    }
    Ok(())
}
