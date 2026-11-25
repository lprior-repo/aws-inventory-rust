use super::*;
#[path = "../../../engine/tests/inventory_store/support.rs"]
#[allow(
    dead_code,
    reason = "shared via #[path]; each includer uses a subset of the helpers"
)]
mod support;
use inventory_core::record::Exclusions;
use inventory_engine::inventory::SweepId;
use sqlx::{Connection, Row, SqliteConnection};

type TestResult = Result<(), Box<dyn std::error::Error>>;
fn raw(document: &str) -> Result<Record, SweepError> {
    Ok(Record::encode(&Value::Map(
        [
            (
                Key::Field(FieldId::new(1)?),
                Value::Text("role-name".into()),
            ),
            (Key::Field(FieldId::new(2)?), Value::Text(document.into())),
        ]
        .into_iter()
        .collect(),
    ))?)
}

#[tokio::test]
async fn given_encoded_trust_when_projected_and_committed_then_only_summary_reaches_sqlite()
-> TestResult {
    let document = r#"%7B%22Statement%22%3A%7B%22Effect%22%3A%22Allow%22%2C%22Action%22%3A%22sts%3AAssumeRole%22%2C%22Principal%22%3A%7B%22AWS%22%3A%22arn%3Aaws%3Aiam%3A%3A222222222222%3Arole%2Ftarget%22%7D%2C%22Condition%22%3A%7B%22StringEquals%22%3A%7B%22sts%3AExternalId%22%3A%22private-secret-value%22%7D%7D%7D%7D"#;
    let projection = ResourceProjectionPlan::IamTrustSummary {
        path: vec![FieldId::new(2)?].into_boxed_slice(),
    };
    let (directory, store, _) = support::fixture().await?;
    let projected = project_with(&projection, &raw(document)?)?;
    let observation = Observation::new(
        support::resource("role-one")?,
        projected,
        &Exclusions::none(),
        None,
    )?;
    store
        .seed_work(support::work("trust-projection", "us-east-1")?)
        .await?;
    let lease = store
        .claim(&SweepId::new("trust-projection".into())?, 1, 100)
        .await?
        .ok_or("missing lease")?;
    store
        .commit_page(support::page(lease, vec![observation], true, None, 2))
        .await?;
    let mut connection = SqliteConnection::connect(&format!(
        "sqlite://{}",
        directory.path().join("inventory.sqlite").display()
    ))
    .await?;
    let stored: Vec<u8> = sqlx::query("SELECT record FROM resource_facets WHERE id='role-one'")
        .fetch_one(&mut connection)
        .await?
        .try_get(0)?;
    assert!(
        !stored
            .windows(document.len())
            .any(|window| window == document.as_bytes())
    );
    assert!(
        !stored
            .windows(20)
            .any(|window| window == b"private-secret-value")
    );
    let record = Record::from_bytes(stored)?;
    let Some(ValueRef::Map(summary)) = record.root().get_field(FieldId::new(2)?)? else {
        return Err("summary missing".into());
    };
    assert!(matches!(
        summary.get_name("trust_policy_values_omitted")?,
        Some(ValueRef::Bool(true))
    ));
    let Some(ValueRef::List(statements)) = summary.get_name("trust_statements")? else {
        return Err("statements missing".into());
    };
    assert_eq!(statements.count()?, 1);
    connection.close().await?;
    Ok(())
}

#[test]
fn given_untrusted_policies_when_projected_then_malformed_and_oversized_inputs_fail_closed()
-> TestResult {
    let projection = ResourceProjectionPlan::IamTrustSummary {
        path: vec![FieldId::new(2)?].into_boxed_slice(),
    };
    [
        "%FF".to_owned(),
        "{".to_owned(),
        "x".repeat(65_537),
        serde_json::json!({"Statement":vec![serde_json::json!({});129]}).to_string(),
        serde_json::json!({"Statement":{"Principal":{"AWS":vec!["123456789012";257]}}}).to_string(),
    ]
    .iter()
    .try_for_each(|document| -> TestResult {
        assert!(project_with(&projection, &raw(document)?).is_err());
        Ok(())
    })?;
    Ok(())
}

#[test]
fn given_sensitive_endpoint_when_projected_then_digest_is_stable_and_other_identity_fields_survive()
-> TestResult {
    let projection = ResourceProjectionPlan::SensitiveDigest {
        path: vec![FieldId::new(2)?].into_boxed_slice(),
    };
    let row = raw("https://user:password@example.test/path")?;
    let first = project_with(&projection, &row)?;
    let second = project_with(&projection, &row)?;
    assert_eq!(first, second);
    assert!(matches!(
        first.root().get_field(FieldId::new(1)?)?,
        Some(ValueRef::Text("role-name"))
    ));
    let Some(ValueRef::Text(digest)) = first.root().get_field(FieldId::new(2)?)? else {
        return Err("digest absent".into());
    };
    assert!(digest.starts_with("sha256:"));
    assert_eq!(digest.len(), 71);
    assert!(!digest.contains("password"));
    assert!(project_with(&projection, &raw(&"x".repeat(16_385))?).is_err());
    Ok(())
}

#[test]
fn given_sensitive_map_entry_when_projected_then_only_reviewed_entry_is_digested() -> TestResult {
    let field = FieldId::new(2)?;
    let projection = ResourceProjectionPlan::SensitiveMapDigest {
        path: vec![field].into_boxed_slice(),
        key: "Endpoint".into(),
    };
    let row = Record::encode(&Value::Map(
        [(
            Key::Field(field),
            Value::Map(
                [
                    (
                        Key::Name("Endpoint".into()),
                        Value::Text("https://user:password@example.test".into()),
                    ),
                    (
                        Key::Name("SubscriptionArn".into()),
                        Value::Text("arn:aws:sns:us-east-1:123456789012:topic:sub".into()),
                    ),
                ]
                .into_iter()
                .collect(),
            ),
        )]
        .into_iter()
        .collect(),
    ))?;
    let projected = project_with(&projection, &row)?;
    let Some(ValueRef::Map(attributes)) = projected.root().get_field(field)? else {
        return Err("attributes missing".into());
    };
    assert!(matches!(
        attributes.get_name("SubscriptionArn")?,
        Some(ValueRef::Text(
            "arn:aws:sns:us-east-1:123456789012:topic:sub"
        ))
    ));
    let Some(ValueRef::Text(endpoint)) = attributes.get_name("Endpoint")? else {
        return Err("endpoint missing".into());
    };
    assert!(endpoint.starts_with("sha256:"));
    assert_eq!(endpoint.len(), 71);
    Ok(())
}

#[test]
fn given_bounded_multiple_statements_when_principal_total_exceeds_limit_then_projection_rejects()
-> TestResult {
    let projection = ResourceProjectionPlan::IamTrustSummary {
        path: vec![FieldId::new(2)?].into_boxed_slice(),
    };
    let statement =
        serde_json::json!({"Effect":"Allow","Principal":{"AWS":vec!["123456789012";129]}});
    let document = serde_json::json!({"Statement":[statement.clone(),statement]}).to_string();
    assert!(project_with(&projection, &raw(&document)?).is_err());
    Ok(())
}

#[test]
fn given_global_source_when_arn_region_relation_projects_owner_then_region_is_read_from_the_arn_and_non_regional_values_fail()
-> TestResult {
    use inventory_core::{
        catalog::runtime::{RelationPathMode, RelationSource, RelationTargetMode},
        inventory::{Relation, ResourceId, ResourceKey},
    };
    let kind = Kind::parse("AWS::Route53::KeySigningKey")?;
    let target = Kind::parse("AWS::KMS::Key")?;
    let relation = RelationshipPlan {
        source_kind: kind.clone(),
        source: RelationSource::SelectedOutput,
        edge_source: inventory_core::inventory::EdgeSource::ReviewedRule,
        source_path: vec![FieldId::new(1)?].into_boxed_slice(),
        target_kind: target.clone(),
        target_kinds: vec![target].into_boxed_slice(),
        target_path: vec![FieldId::new(1)?].into_boxed_slice(),
        relation: Relation::parse("USES_KEY")?,
        scope: RelationScope::ArnRegion,
        owner: ReferenceOwnerRule::ExecutionAccount,
        source_path_mode: RelationPathMode::Fields,
        target_mode: RelationTargetMode::SelectedFields,
        guard: None,
        components: Box::new([]),
    };
    let key = ResourceKey {
        account: AccountId::parse("123456789012")?,
        region: Region::Global,
        kind,
        id: ResourceId::parse("Z1/ksk")?,
    };
    let observation = Observation::new(key, raw("{}")?, &Exclusions::none(), None)?;
    let home = RegionId::parse("us-west-2")?;
    let kms = "arn:aws:kms:us-east-1:123456789012:key/11111111-2222-3333-4444-555555555555";
    let (account, region) = reference_owner(&relation, &observation, kms, &home)?;
    assert_eq!(account.as_str(), "123456789012");
    assert_eq!(region, Region::parse("us-east-1")?);
    for invalid in [
        "11111111-2222-3333-4444-555555555555",
        "arn:aws:iam::123456789012:role/r",
        "arn:aws:kms",
    ] {
        assert!(
            reference_owner(&relation, &observation, invalid, &home).is_err(),
            "{invalid}"
        );
    }
    Ok(())
}
