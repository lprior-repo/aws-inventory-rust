use super::*;

const CONNECT_INSTANCE: &str = r#"{"Id":"instance-observed","Arn":"arn:aws:connect:us-west-2:123456789012:instance/instance-observed"}"#;
const CONNECT_DEFAULTS: &str = r#"{"ContactFlowSummaryList":[{"Arn":"flow-1"},{"Arn":"flow-2"},{"Arn":"flow-3"},{"Arn":"flow-4"},{"Arn":"flow-5"},{"Arn":"flow-6"},{"Arn":"flow-7"},{"Arn":"flow-8"},{"Arn":"flow-9"}]}"#;
const CONNECT_RECIPE: &str = include_str!("../../seeds/connect.toml");

fn defaults_response(count: usize, duplicate: bool, token: Option<&str>) -> String {
    let flows: Vec<_> = (0..count)
        .map(|index| serde_json::json!({"Arn": format!("flow-{}", if duplicate { 0 } else { index })}))
        .collect();
    match token {
        Some(token) => serde_json::json!({"ContactFlowSummaryList": flows, "NextToken": token}),
        None => serde_json::json!({"ContactFlowSummaryList": flows}),
    }
    .to_string()
}

fn observed_flows(oracle: &ScenarioSeedOracle) -> Vec<(&str, SeedIdentityEvidence)> {
    oracle
        .resources
        .iter()
        .filter(|resource| resource.kind.as_str() == "AWS::Connect::ContactFlow")
        .map(|resource| (resource.id.as_str(), resource.evidence))
        .collect()
}

#[tokio::test]
async fn given_new_connect_instance_when_seeded_then_observed_default_flows_are_in_oracle()
-> TestResult {
    let scenario = load(CONNECT_RECIPE)?;
    let (oracle, requests) =
        execute(scenario, vec![CALLER, CONNECT_INSTANCE, CONNECT_DEFAULTS]).await?;
    let oracle = oracle?;
    assert_eq!(
        observed_flows(&oracle),
        [
            "flow-1", "flow-2", "flow-3", "flow-4", "flow-5", "flow-6", "flow-7", "flow-8",
            "flow-9",
        ]
        .map(|id| (id, SeedIdentityEvidence::ServiceCreated))
    );
    let list = requests.get(2).ok_or("missing list-flows request")?;
    assert_eq!(
        parts(list)?.0.lines().next(),
        Some("GET /contact-flows-summary/instance-observed?maxResults=100 HTTP/1.1")
    );
    let rows = SeedRunReport {
        oracle,
        outcomes: vec![],
    }
    .oracle_rows();
    assert!(rows.contains(&format!(
        "resource\tAWS::Connect::ContactFlow\t{ACCOUNT}\t{REGION}\tflow-9\tservice-created"
    )));
    assert!(rows.contains(&format!(
        "resource\tAWS::Connect::Instance\t{ACCOUNT}\t{REGION}\tarn:aws:connect:{REGION}:{ACCOUNT}:instance/instance-observed"
    )));
    Ok(())
}

#[tokio::test]
async fn given_short_or_long_connect_defaults_when_seeded_then_no_partial_oracle_is_accepted()
-> TestResult {
    for observed in [10, 1, 0] {
        let defaults = defaults_response(observed, false, None);
        let (result, _) = execute(
            load(CONNECT_RECIPE)?,
            vec![CALLER, CONNECT_INSTANCE, &defaults],
        )
        .await?;
        let error = result.err().ok_or("wrong size returned an oracle")?;
        assert!(matches!(
            error,
            SeedError::ResponseListCount { expected: 9, observed: found } if found == observed
        ));
        assert_eq!(
            error.to_string(),
            format!("seed response identity list count differs: expected 9, observed {observed}")
        );
    }
    Ok(())
}

#[tokio::test]
async fn given_exact_list_with_continuation_when_seeded_then_no_partial_oracle_is_accepted()
-> TestResult {
    let defaults = defaults_response(9, false, Some("another-page"));
    let (result, _) = execute(
        load(CONNECT_RECIPE)?,
        vec![CALLER, CONNECT_INSTANCE, &defaults],
    )
    .await?;
    let error = result.err().ok_or("paginated reply returned an oracle")?;
    assert!(matches!(
        &error,
        SeedError::ResponseListContinuation { member } if member == "NextToken"
    ));
    assert_eq!(
        error.to_string(),
        "seed response identity list has continuation token NextToken"
    );
    Ok(())
}

#[tokio::test]
async fn given_duplicate_members_or_non_list_output_when_seeded_then_identity_is_rejected()
-> TestResult {
    let duplicate = defaults_response(9, true, None);
    let non_list =
        CONNECT_RECIPE.replace("list = \"ContactFlowSummaryList\"", "list = \"NextToken\"");
    for (recipe, response, expected) in [
        (
            CONNECT_RECIPE,
            duplicate.as_str(),
            "seed operation returned a duplicate resource identity",
        ),
        (
            non_list.as_str(),
            CONNECT_DEFAULTS,
            "seed response identity path must select a modeled list",
        ),
    ] {
        let (result, _) = execute(load(recipe)?, vec![CALLER, CONNECT_INSTANCE, response]).await?;
        assert!(
            matches!(result, Err(SeedError::Invalid(message)) if message == expected),
            "{expected}: {result:?}"
        );
    }
    Ok(())
}

const LIST_COUNT_OR_SCALAR: &str =
    "seed response identity list needs a count from 1 through 256 and no scalar identity";

#[test]
fn given_invalid_list_descriptor_fields_when_loading_then_each_is_rejected() {
    for (identity, expected) in [
        (
            "member = \"Arn\", count = 0",
            SeedError::Invalid(LIST_COUNT_OR_SCALAR),
        ),
        (
            "member = \"Arn\", count = 257",
            SeedError::Invalid(LIST_COUNT_OR_SCALAR),
        ),
        (
            "member = \"Arn\", count = -1",
            SeedError::Invalid("seed response identity count must be an integer"),
        ),
        (
            "count = 9",
            SeedError::Message("seed field member must be a string".into()),
        ),
        (
            "member = \"Arn\", count = 9, surplus = 1",
            SeedError::Message("unknown seed field: surplus".into()),
        ),
    ] {
        let recipe = CONNECT_RECIPE.replace("member = \"Arn\", count = 9", identity);
        let result = load(&recipe);
        let matched = match (&result, expected) {
            (Err(SeedError::Invalid(actual)), SeedError::Invalid(want)) => *actual == want,
            (Err(SeedError::Message(actual)), SeedError::Message(want)) => *actual == want,
            _ => false,
        };
        assert!(matched, "{identity}: {result:?}");
    }
}
#[test]
fn given_list_with_scalar_expectation_or_as_relationship_endpoint_when_loading_then_it_is_rejected()
-> TestResult {
    let scenario = load(CONNECT_RECIPE)?;
    let mut steps = scenario.steps().to_vec();
    let list = steps.last_mut().ok_or("missing list step")?;
    list.expected_identity = Some("not-a-scalar".into());
    assert!(matches!(
        SeedScenario::new(steps),
        Err(SeedError::Invalid(LIST_COUNT_OR_SCALAR))
    ));
    for (source, target) in [(0, 1), (1, 0)] {
        let relationship = SeedRelationship::new(
            StepId::new(source),
            StepId::new(target),
            "belongs_to".into(),
            FieldPath::new(&["Arn"])?,
        );
        assert!(matches!(
            SeedScenario::with_relationships(scenario.steps().to_vec(), vec![relationship]),
            Err(SeedError::Invalid(
                "seed relationship descriptor is invalid"
            ))
        ));
    }
    Ok(())
}

#[tokio::test]
async fn given_connect_campaign_when_seeded_then_defaults_and_the_explicit_flow_are_recorded()
-> TestResult {
    let scenario =
        inventory_harness::inventory_seed::load(include_str!("../../seeds/connectcampaigns.toml"))?;
    let flow = r#"{"ContactFlowArn":"campaign-flow","ContactFlowId":"campaign-flow-id"}"#;
    let campaign = r#"{"arn":"seeded-campaign","id":"seeded-campaign-id"}"#;
    let replies = vec![
        CALLER,
        CONNECT_INSTANCE,
        CONNECT_DEFAULTS,
        "{}",
        flow,
        campaign,
    ];
    let (oracle, requests) = execute(scenario, replies).await?;
    assert_eq!(
        observed_flows(&oracle?),
        [
            "flow-1", "flow-2", "flow-3", "flow-4", "flow-5", "flow-6", "flow-7", "flow-8",
            "flow-9",
        ]
        .map(|id| (id, SeedIdentityEvidence::ServiceCreated))
        .into_iter()
        .chain([("campaign-flow", SeedIdentityEvidence::Response)])
        .collect::<Vec<_>>()
    );
    let create = requests.get(5).ok_or("missing create-campaign request")?;
    let body: serde_json::Value = serde_json::from_str(parts(create)?.1)?;
    assert_eq!(
        body.pointer("/outboundCallConfig/connectContactFlowId")
            .and_then(serde_json::Value::as_str),
        Some("campaign-flow-id")
    );
    Ok(())
}

#[tokio::test]
async fn given_codeartifact_domain_when_seeded_then_it_uses_an_oracle_owned_key() -> TestResult {
    let scenario = load(CODEARTIFACT_RECIPE)?;
    let (oracle, requests) = execute(scenario, vec![CALLER, ARTIFACT_KEY, ARTIFACT_DOMAIN]).await?;
    let oracle = oracle?;
    let seeded_key = codeartifact_key(&oracle)?;
    assert_eq!(
        seeded_key,
        "arn:aws:kms:us-west-2:123456789012:key/observed-domain-key"
    );
    let request = requests.get(2).ok_or("missing create-domain request")?;
    let body: serde_json::Value = serde_json::from_str(parts(request)?.1)?;
    assert_eq!(
        body.get("encryptionKey")
            .and_then(serde_json::Value::as_str),
        Some(seeded_key)
    );
    Ok(())
}

const CODEARTIFACT_RECIPE: &str = include_str!("../../seeds/codeartifact.toml");
const ARTIFACT_KEY: &str =
    r#"{"KeyMetadata":{"Arn":"arn:aws:kms:us-west-2:123456789012:key/observed-domain-key"}}"#;
const ARTIFACT_DOMAIN: &str = r#"{"domain":{"arn":"arn:aws:codeartifact:us-west-2:123456789012:domain/inventory-thin-codeartifact"}}"#;

fn codeartifact_key(oracle: &ScenarioSeedOracle) -> Result<&str, &'static str> {
    let mut keys = oracle
        .resources
        .iter()
        .filter(|resource| resource.kind.as_str() == "AWS::KMS::Key");
    let key = keys.next().ok_or("missing seeded domain key")?;
    if keys.next().is_some() {
        return Err("more than one seeded KMS key");
    }
    Ok(key.id.as_str())
}

#[tokio::test]
async fn given_codeartifact_oracles_with_zero_or_two_keys_when_checked_then_both_are_rejected()
-> TestResult {
    let no_key = r#"
format = 1
service = "codeartifact"
[[step]]
id = "resource"
operation = "CreateDomain"
kind = "AWS::CodeArtifact::Domain"
identity = "domain.arn"
[step.input]
domain = "inventory-thin-codeartifact"
"#;
    let extra = r#"
[[step]]
id = "extra-key"
service = "kms"
operation = "CreateKey"
kind = "AWS::KMS::Key"
identity = "KeyMetadata.Arn"
"#;
    let two_keys = format!("{CODEARTIFACT_RECIPE}\n{extra}");
    let second = r#"{"KeyMetadata":{"Arn":"arn:aws:kms:us-west-2:123456789012:key/another-key"}}"#;
    for (recipe, replies, error) in [
        (
            no_key,
            vec![CALLER, ARTIFACT_DOMAIN],
            "missing seeded domain key",
        ),
        (
            two_keys.as_str(),
            vec![CALLER, ARTIFACT_KEY, ARTIFACT_DOMAIN, second],
            "more than one seeded KMS key",
        ),
    ] {
        let (oracle, _) = execute(load(recipe)?, replies).await?;
        assert_eq!(codeartifact_key(&oracle?), Err(error));
    }
    Ok(())
}
