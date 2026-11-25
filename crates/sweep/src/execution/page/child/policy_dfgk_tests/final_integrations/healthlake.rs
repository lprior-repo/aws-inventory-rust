use super::*;

fn profile(format: &str, version: i64) -> Json {
    let id = if format == "CCDA" {
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    } else {
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
    };
    json!({"ProfileId":id,"Version":version,"SourceFormat":format,"TargetFormat":"FHIR_R4","ProfileName":format!("{format} profile")})
}

async fn seed_formats(scenario: &Scenario<'_, '_>, root: &CallablePlan<'_>) -> TestResult {
    for branch in root.branches() {
        let region = Region::Regional(scenario.home.clone());
        let scope = ScopeKey::new(
            scenario.account.clone(),
            region.clone(),
            ServiceId::parse("healthlake")?,
            OperationId::parse(root.operation_name())?,
            None,
        )?
        .with_branch(branch.discriminator.clone())?;
        let bindings = routing::branch_bindings(root, branch, &scenario.account, &region)?;
        scenario
            .store
            .seed_work(WorkSpec::new(
                scenario.sweep.clone(),
                scope,
                None,
                bindings.into_bytes(),
            )?)
            .await?;
    }
    Ok(())
}

fn response(
    root: &CallablePlan<'_>,
    plan: &CallablePlan<'_>,
    lease: &WorkLease,
    wire: &http::Request<bytes::Bytes>,
    format: &str,
    pages: u32,
) -> TestResult<Json> {
    match plan.operation_name() {
        "ListDataTransformationProfiles" => {
            let body: Json = serde_json::from_slice(wire.body())?;
            assert_eq!(
                body.get("SourceFormat").and_then(Json::as_str),
                Some(format)
            );
            assert_eq!(lease.scope().parent(), None);
            Ok(if pages == 0 {
                page("Items", Vec::new(), Some("secondpage"), "NextToken")
            } else {
                page("Items", vec![profile(format, 1)], None, "NextToken")
            })
        }
        "ListDataTransformationProfileVersions" => {
            let id = if format == "CCDA" {
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            } else {
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
            };
            let body: Json = serde_json::from_slice(wire.body())?;
            assert_eq!(body.get("ProfileId").and_then(Json::as_str), Some(id));
            let parent = lease
                .scope()
                .parent()
                .ok_or("profile version lost its parent")?;
            assert_eq!(parent.kind, *root.kind());
            assert_eq!(
                parent.id.as_str(),
                format!("{}:{format}|{}:{id}", format.len(), id.len())
            );
            Ok(match pages {
                0 => page("Items", Vec::new(), Some("secondpage"), "NextToken"),
                1 => page(
                    "Items",
                    vec![profile(format, 0)],
                    Some("lastpage"),
                    "NextToken",
                ),
                2 => page("Items", vec![profile(format, 1)], None, "NextToken"),
                _ => return Err("unexpected version cursor".into()),
            })
        }
        _ => Err("profile content/transformation call scheduled".into()),
    }
}

fn source_format(lease: &WorkLease) -> TestResult<String> {
    if let Some(branch) = lease.scope().branch() {
        return Ok(branch.to_owned());
    }
    let parent = lease
        .scope()
        .parent()
        .ok_or("source-format branch and parent both absent")?;
    let (length, rest) = parent
        .id
        .as_str()
        .split_once(':')
        .ok_or("parent identity lacks a format prefix")?;
    let length: usize = length.parse()?;
    Ok(rest
        .get(..length)
        .ok_or("parent format prefix overruns identity")?
        .to_owned())
}

fn expected_requests() -> Vec<(String, String, u32)> {
    let mut requests = ["CCDA", "CSV"]
        .into_iter()
        .flat_map(|format| {
            [
                (
                    "ListDataTransformationProfiles".to_owned(),
                    format.to_owned(),
                    0,
                ),
                (
                    "ListDataTransformationProfiles".to_owned(),
                    format.to_owned(),
                    1,
                ),
                (
                    "ListDataTransformationProfileVersions".to_owned(),
                    format.to_owned(),
                    0,
                ),
                (
                    "ListDataTransformationProfileVersions".to_owned(),
                    format.to_owned(),
                    1,
                ),
                (
                    "ListDataTransformationProfileVersions".to_owned(),
                    format.to_owned(),
                    2,
                ),
            ]
        })
        .collect::<Vec<_>>();
    requests.sort();
    requests
}

#[tokio::test]
async fn given_both_source_formats_when_empty_profile_and_version_pages_resume_then_draft_and_published_versions_keep_their_profile()
-> TestResult {
    let catalog = catalog()?;
    let root = fixture::plan(&catalog, "healthlake", "ListDataTransformationProfiles")?;
    let versions = fixture::plan(
        &catalog,
        "healthlake",
        "ListDataTransformationProfileVersions",
    )?;
    assert_eq!(
        root.kind().as_str(),
        "AWS::HealthLake::DataTransformationProfile"
    );
    assert_eq!(
        versions.kind().as_str(),
        "AWS::ApiOnly::HealthLakeDataTransformationProfileVersion"
    );
    assert_eq!(
        root.branches()
            .iter()
            .map(|branch| branch.discriminator.as_str())
            .collect::<Vec<_>>(),
        ["CCDA", "CSV"]
    );
    let mut scenario = Scenario::new(&catalog, "healthlake-source-format-versions").await?;
    seed_formats(&scenario, root).await?;
    let mut requested = Vec::new();
    for _ in 0..10 {
        let lease = scenario
            .store
            .claim(&scenario.sweep, 1001, 10000)
            .await?
            .ok_or("profile/version continuation absent")?;
        let plan = fixture::plan(&catalog, "healthlake", lease.scope().operation().as_str())?;
        let format = source_format(&lease)?;
        assert!(matches!(format.as_str(), "CCDA" | "CSV"));
        let (request, wire) = scenario.prepared_request(plan, &lease)?;
        assert!(
            wire.uri()
                .host()
                .ok_or("HealthLake endpoint host absent")?
                .starts_with("datatransformation.")
        );
        let cursor = cursor::Cursor::decode(lease.token())?;
        let expected_token = match cursor.pages {
            0 => None,
            1 => Some("secondpage"),
            2 => Some("lastpage"),
            _ => return Err("unexpected HealthLake page".into()),
        };
        assert_wire_token(&wire, expected_token, "NextToken")?;
        requested.push((
            plan.operation_name().to_owned(),
            format.clone(),
            cursor.pages,
        ));
        let response = response(root, plan, &lease, &wire, &format, cursor.pages)?;
        scenario
            .apply(&catalog, plan, lease, &request, response)
            .await?;
        scenario = reopen(scenario).await?;
    }
    assert!(
        scenario
            .store
            .claim(&scenario.sweep, 1001, 10000)
            .await?
            .is_none()
    );
    requested.sort();
    let expected_requests = expected_requests();
    assert_eq!(requested, expected_requests);
    let profiles = saved(&scenario, root.kind().as_str()).await?;
    assert_eq!(
        profiles
            .iter()
            .map(|(id, _)| id.as_str())
            .collect::<Vec<_>>(),
        [
            "3:CSV|32:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "4:CCDA|32:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        ]
    );
    let versions = saved(&scenario, versions.kind().as_str()).await?;
    assert_eq!(
        versions
            .iter()
            .map(|(id, _)| id.as_str())
            .collect::<Vec<_>>(),
        [
            "3:CSV|32:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb|1:0",
            "3:CSV|32:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb|1:1",
            "4:CCDA|32:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa|1:0",
            "4:CCDA|32:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa|1:1",
        ]
    );
    let edges = scenario.store.projection_edges(None, 100).await?;
    assert_eq!(edges.edges.len(), 4);
    for edge in edges.edges {
        assert_eq!(edge.relation.as_str(), "VERSION_OF");
        assert!(
            edge.from
                .id
                .as_str()
                .starts_with(&format!("{}|", edge.to.id.as_str()))
        );
    }
    Ok(())
}
