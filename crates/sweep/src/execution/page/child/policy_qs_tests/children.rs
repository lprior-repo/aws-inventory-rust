use super::*;

#[tokio::test]
async fn given_two_applications_when_index_and_datasource_fanout_resumes_then_matching_parent_pairs_and_all_children_persist()
-> TestResult {
    let catalog = fixture::catalog()?;
    let scenario = Scenario::new(&catalog, "qbusiness-owned-descendants", "us-east-1").await?;
    scenario
        .seed(fixture::plan(&catalog, "qbusiness", "ListApplications")?)
        .await?;
    let mut requests = BTreeSet::new();
    scenario.run(&catalog, 25, |plan, lease, wire| {
        let operation = plan.operation_name();
        let second = lease.token().is_some();
        if operation == "ListApplications" { return bytes(json!({"applications":[{"applicationId":"app-a"},{"applicationId":"app-b"}]})); }
        let path = wire.uri().path();
        let application = if path.contains("/app-a/") { "app-a" } else { "app-b" };
        assert!(path.contains(&format!("/applications/{application}/")));
        assert_eq!(wire.uri().query().is_some_and(|query| query.contains("nextToken=page-two")), second);
        assert!(requests.insert((operation.to_owned(), path.to_owned(), second)));
        let page = if second { "b" } else { "a" };
        let value = match operation {
            "ListIndices" => json!({"indices":[{"indexId":format!("index-{page}"),"displayName":"shared"}]}),
            "ListPlugins" => json!({"plugins":[{"pluginId":format!("plugin-{page}"),"displayName":"shared","type":"CUSTOM","state":"ACTIVE"}]}),
            "ListWebExperiences" => json!({"webExperiences":[{"webExperienceId":format!("web-{page}"),"status":"ACTIVE"}]}),
            "ListRetrievers" => json!({"retrievers":[{"retrieverId":format!("retriever-{page}"),"type":"NATIVE_INDEX"}]}),
            "ListDataSources" => {
                assert!(path.contains("/indices/index-a/") || path.contains("/indices/index-b/"));
                json!({"dataSources":[{"dataSourceId":format!("source-{page}"),"displayName":"shared","status":"ACTIVE"}]})
            }
            _ => return Err("unexpected content or write operation".into()),
        };
        paginated(value, "nextToken", second)
    }).await?;
    assert_eq!(requests.len(), 24);
    let saved = scenario.saved().await?;
    for kind in ["Index", "Plugin", "WebExperience", "Retriever"] {
        assert_eq!(count(&saved, &format!("AWS::QBusiness::{kind}")), 4);
    }
    assert_eq!(count(&saved, "AWS::QBusiness::DataSource"), 8);
    let edges = scenario.edges().await?;
    assert_eq!(
        edges
            .iter()
            .filter(
                |(kind, _, _, _, relation)| kind == "AWS::QBusiness::DataSource"
                    && relation == "FOR_INDEX"
            )
            .count(),
        8
    );
    assert_eq!(
        edges
            .iter()
            .filter(
                |(_, _, kind, _, relation)| kind == "AWS::QBusiness::Application"
                    && relation == "FOR_APPLICATION"
            )
            .count(),
        20
    );
    for application in ["app-a", "app-b"] {
        for index in ["index-a", "index-b"] {
            let expected = saved
                .iter()
                .find(|(kind, id, _)| {
                    kind == "AWS::QBusiness::Index"
                        && id.contains(application)
                        && id.contains(index)
                })
                .ok_or("actual persisted owning index is missing")?;
            let matches = edges.iter().filter(|(kind, source, _, _, relation)| {
                kind == "AWS::QBusiness::DataSource"
                    && relation == "FOR_INDEX"
                    && source.contains(application)
                    && source.contains(index)
            });
            let mut matched = 0;
            for (_, _, target_kind, target, _) in matches {
                matched += 1;
                assert_eq!(target_kind, "AWS::QBusiness::Index");
                assert_eq!(target, &expected.1);
            }
            assert_eq!(matched, 2);
        }
    }
    Ok(())
}

#[tokio::test]
async fn given_two_owners_per_family_when_safe_child_pages_resume_then_accesspoints_versions_certificates_and_window_targets_remain_distinct()
-> TestResult {
    let catalog = fixture::catalog()?;
    let scenario = Scenario::new(&catalog, "qs-metadata-children", "us-east-1").await?;
    for (service, operation) in [
        ("s3files", "ListFileSystems"),
        ("sagemaker", "ListImages"),
        ("sesv2", "ListEmailIdentities"),
        ("ssm", "DescribeMaintenanceWindows"),
        ("ssm", "ListDocuments"),
    ] {
        scenario
            .seed(fixture::plan(&catalog, service, operation)?)
            .await?;
    }
    scenario.run(&catalog, 22, |plan, lease, wire| {
        let second = lease.token().is_some();
        let page = if second { "b" } else { "a" };
        let value = match plan.operation_name() {
            "ListFileSystems" => return bytes(json!({"fileSystems":[filesystem("fs-a"),filesystem("fs-b")]})),
            "ListAccessPoints" => {
                let query = wire.uri().query().ok_or("filesystem selector absent")?;
                let fs = if query.contains("fs-a") { "fs-a" } else { "fs-b" };
                assert!(query.contains("fileSystemId=arn%3Aaws%3As3files"));
                json!({"accessPoints":[{"accessPointArn":format!("arn:aws:s3files:us-east-1:123456789012:access-point/{fs}-{page}"),"accessPointId":format!("{fs}-{page}"),"fileSystemId":fs,"status":"available","ownerId":"123456789012"}]})
            }
            "ListImages" => return bytes(json!({"Images":[image("image-a"),image("image-b")]})),
            "ListImageVersions" => {
                let request = body(wire)?;
                let name = text(&request, "ImageName")?;
                let versions = if second { vec![11] } else { (1..=10).collect::<Vec<_>>() };
                json!({"ImageVersions":versions.into_iter().map(|version| json!({"CreationTime":0,"LastModifiedTime":0,"ImageArn":image_arn(name),"ImageVersionArn":format!("arn:aws:sagemaker:us-east-1:123456789012:image-version/{name}/{version}"),"ImageVersionStatus":"CREATED","Version":version})).collect::<Vec<_>>()})
            }
            "ListEmailIdentities" => return bytes(json!({"EmailIdentities":[{"IdentityName":"a.example.test","IdentityType":"DOMAIN"},{"IdentityName":"b.example.test","IdentityType":"DOMAIN"}]})),
            "ListEmailIdentityCertificates" => {
                let request = body(wire)?;
                assert!(matches!(text(&request,"EmailIdentity")?, "a.example.test" | "b.example.test"));
                let certificate = "arn:aws:acm:us-east-1:123456789012:certificate/11111111-2222-3333-4444-555555555555";
                let rows = if second { vec![json!({"CertificateArn":certificate,"Status":"INACTIVE"})] }
                    else { vec![json!({"FromAddress":"alice@example.test","CertificateArn":certificate,"Status":"ACTIVE"}),json!({"FromAddress":"bob@example.test","CertificateArn":certificate,"Status":"PENDING_PROVISIONING"})] };
                json!({"Certificates":rows})
            }
            "DescribeMaintenanceWindows" => return bytes(json!({"WindowIdentities":[{"WindowId":"mw-a","Name":"a"},{"WindowId":"mw-b","Name":"b"}]})),
            "DescribeMaintenanceWindowTargets" => {
                let request = body(wire)?;
                let window = text(&request,"WindowId")?;
                assert!(request.get("Filters").is_none());
                json!({"Targets":[{"WindowId":window,"WindowTargetId":format!("same-{page}"),"ResourceType":"INSTANCE","Targets":[{"Key":if second {"tag:Team"} else {"InstanceIds"},"Values":[if second {"inventory"} else {"i-0123456789abcdef0"}]}]}]})
            }
            "ListDocuments" => {
                let request = body(wire)?;
                assert_eq!(request.get("Filters"), Some(&json!([{"Key":"Owner","Values":["Self"]}])));
                assert!(request.get("DocumentFilterList").is_none());
                json!({"DocumentIdentifiers":[{"Name":format!("owned-{page}"),"Owner":"123456789012","DocumentType":"Command","DocumentFormat":"JSON","Tags":[]}]})
            }
            _ => return Err("unreviewed child/content call".into()),
        };
        paginated(value, if plan.service_name() == "s3files" { "nextToken" } else { "NextToken" }, second)
    }).await?;
    let saved = scenario.saved().await?;
    for (kind, expected) in [
        ("AWS::S3Files::AccessPoint", 4),
        ("AWS::SageMaker::ImageVersion", 22),
        ("AWS::SES::EmailIdentityCertificate", 6),
        ("AWS::SSM::MaintenanceWindowTarget", 4),
        ("AWS::SSM::Document", 2),
    ] {
        assert_eq!(count(&saved, kind), expected, "lost {kind}");
    }
    let edges = scenario.edges().await?;
    assert_eq!(
        edges
            .iter()
            .filter(
                |(kind, _, _, _, relation)| kind == "AWS::SageMaker::ImageVersion"
                    && relation == "VERSION_OF"
            )
            .count(),
        22
    );
    assert_eq!(
        edges
            .iter()
            .filter(
                |(kind, _, _, _, relation)| kind == "AWS::SSM::MaintenanceWindowTarget"
                    && relation == "FOR_WINDOW"
            )
            .count(),
        4
    );
    Ok(())
}

fn filesystem(id: &str) -> Json {
    json!({"creationTime":0,"fileSystemArn":format!("arn:aws:s3files:us-east-1:123456789012:file-system/{id}"),"fileSystemId":id,"bucket":"owned","status":"available","roleArn":"arn:aws:iam::123456789012:role/files","ownerId":"123456789012"})
}
fn image_arn(name: &str) -> String {
    format!("arn:aws:sagemaker:us-east-1:123456789012:image/{name}")
}
fn image(name: &str) -> Json {
    json!({"CreationTime":0,"ImageArn":image_arn(name),"ImageName":name,"ImageStatus":"CREATED","LastModifiedTime":0})
}
