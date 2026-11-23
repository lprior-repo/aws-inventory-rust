use super::*;
#[test]
fn given_license_tag_endpoint_when_decoded_then_authoritative_snapshot_preserves_discovered_configuration_owner()
-> TestResult {
    let arn = "arn:aws:license-manager:us-east-1:123456789012:license-configuration:lic-123";
    let bytes = compiled(
        "license-manager",
        "2018-08-01",
        &[
            "ListLicenseConfigurations",
            "GetLicenseConfiguration",
            "ListTagsForResource",
        ],
        "AWS::LicenseManager::LicenseConfiguration",
    )?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let list = runtime
        .lookup("license-manager", "ListLicenseConfigurations")
        .ok_or("list absent")?;
    let tags = runtime
        .lookup("license-manager", "ListTagsForResource")
        .ok_or("tag endpoint absent")?;
    let PlanPurpose::TagEndpoint(endpoint) = tags.purpose() else {
        return Err("standalone tags must be a facet".into());
    };
    let parent = endpoint.parents.first().ok_or("tag parent absent")?;
    assert_eq!(parent.operation.as_ref(), "ListLicenseConfigurations");
    let request = parent.request.as_ref().ok_or("tag request absent")?;
    assert_eq!(request.source, TagEndpointSource::Response);
    assert_eq!(
        list.service()
            .field_name(*request.parent.first().ok_or("parent ARN path absent")?)?,
        "LicenseConfigurationArn"
    );
    assert_eq!(
        tags.service()
            .field_name(*request.input.first().ok_or("request ARN path absent")?)?,
        "ResourceArn"
    );
    let summaries = rows(
        list,
        json!({"LicenseConfigurations":[{"LicenseConfigurationArn":arn}]}),
    )?;
    let owner = key(
        list,
        summaries.first().ok_or("summary absent")?,
        "us-east-1",
    )?;
    for (response, coverage, expected) in [
        (
            json!({"Tags":[{"Key":"owner","Value":"platform"}]}),
            TagCoverage::Complete,
            vec![Tag::new("owner", "platform")?],
        ),
        (json!({"Tags":[]}), TagCoverage::Complete, vec![]),
        (json!({}), TagCoverage::Unobserved, vec![]),
    ] {
        let response_rows = rows(tags, response)?;
        let row = response_rows.first().ok_or("tag response absent")?;
        let snapshot = tag_observation(row, &owner, &endpoint.tags)?;
        assert_eq!(snapshot.resource(), &owner);
        assert_eq!(snapshot.coverage(), coverage);
        assert_eq!(snapshot.tags(), expected);
    }
    Ok(())
}
