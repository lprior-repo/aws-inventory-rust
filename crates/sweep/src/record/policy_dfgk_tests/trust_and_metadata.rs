use super::*;
#[tokio::test]
async fn given_mixed_msk_cluster_types_when_persisted_then_both_canonical_cfn_kinds_survive()
-> TestResult {
    let catalog = catalog()?;
    let plan = plan(&catalog, "kafka", "ListClustersV2")?;
    let body = br#"{"clusterInfoList":[{"clusterArn":"arn:aws:kafka:us-east-1:123456789012:cluster/provisioned/1","clusterType":"PROVISIONED"},{"clusterArn":"arn:aws:kafka:us-east-1:123456789012:cluster/serverless/2","clusterType":"SERVERLESS"}]}"#;
    let saved = persist(
        catalog.raw_bytes(),
        plan,
        observations(plan, rows(plan, body)?)?,
    )
    .await?;
    assert_eq!(
        saved
            .iter()
            .map(|(kind, _, _)| kind.as_str())
            .collect::<Vec<_>>(),
        ["AWS::MSK::Cluster", "AWS::MSK::ServerlessCluster"]
    );
    let unknown = rows(plan, br#"{"clusterInfoList":[{"clusterArn":"arn:aws:kafka:us-east-1:123456789012:cluster/unknown/3","clusterType":"FUTURE"}]}"#)?
        .into_iter().next().ok_or("unknown cluster row absent")?;
    assert!(
        matches!(crate::resource_projection::classify(plan, &unknown),
        Err(SweepError::Catalog(inventory_core::catalog::runtime::RuntimeCatalogError::UnclassifiedKind { discriminator })) if discriminator == "FUTURE")
    );
    let missing = rows(plan, br#"{"clusterInfoList":[{"clusterArn":"arn:aws:kafka:us-east-1:123456789012:cluster/missing/4"}]}"#)?
        .into_iter().next().ok_or("missing cluster row absent")?;
    assert!(matches!(
        crate::resource_projection::classify(plan, &missing),
        Err(SweepError::Catalog(
            inventory_core::catalog::runtime::RuntimeCatalogError::Capability
        ))
    ));
    Ok(())
}

#[tokio::test]
async fn given_iam_list_and_detail_when_projected_and_stored_then_trust_values_never_reach_sqlite()
-> TestResult {
    let catalog = catalog()?;
    let document = r#"{"Statement":{"Effect":"Allow","Action":"sts:AssumeRole","Principal":{"Service":"ec2.amazonaws.com"},"Condition":{"StringEquals":{"sts:ExternalId":"must-not-persist"}}}}"#;
    let encoded = document
        .bytes()
        .map(|byte| format!("%{byte:02X}"))
        .collect::<String>();
    let role = format!(
        "<Path>/</Path><RoleName>collector</RoleName><RoleId>AROA12345678901234567</RoleId><Arn>arn:aws:iam::123456789012:role/collector</Arn><CreateDate>2020-01-01T00:00:00Z</CreateDate><AssumeRolePolicyDocument>{encoded}</AssumeRolePolicyDocument>"
    );
    let mut identities = Vec::new();
    for (operation, selected) in [
        (
            "ListRoles",
            format!("<Roles><member>{role}</member></Roles><IsTruncated>false</IsTruncated>"),
        ),
        ("GetRole", format!("<Role>{role}</Role>")),
    ] {
        let plan = plan(&catalog, "iam", operation)?;
        let response = format!(
            "<{operation}Response xmlns=\"https://iam.amazonaws.com/doc/2010-05-08/\"><{operation}Result>{selected}</{operation}Result></{operation}Response>"
        );
        let saved = persist(
            catalog.raw_bytes(),
            plan,
            observations(plan, rows(plan, response.as_bytes())?)?,
        )
        .await?;
        let (kind, id, record) = saved.first().ok_or("IAM role absent from SQLite")?;
        assert_eq!(kind, "AWS::IAM::Role");
        identities.push(id.clone());
        let stored = Record::from_bytes(record.clone())?;
        let projection = plan.projection().ok_or("IAM bounded projection absent")?;
        let Some(ValueRef::Map(summary)) = path_value(stored.root(), projection.protected_path())?
        else {
            return Err("trust summary was not stored as a structure".into());
        };
        assert!(matches!(
            summary.get_name("trust_policy_values_omitted")?,
            Some(ValueRef::Bool(true))
        ));
        for forbidden in ["must-not-persist", document, encoded.as_str()] {
            assert!(
                !record
                    .windows(forbidden.len())
                    .any(|window| window == forbidden.as_bytes())
            );
        }
        for retained in ["sts:AssumeRole", "ec2.amazonaws.com", "StringEquals"] {
            assert!(
                record
                    .windows(retained.len())
                    .any(|window| window == retained.as_bytes())
            );
        }
    }
    assert_eq!(
        identities,
        [
            "arn:aws:iam::123456789012:role/collector",
            "arn:aws:iam::123456789012:role/collector"
        ]
    );
    Ok(())
}

pub(super) fn listed_role(document: &str) -> Vec<u8> {
    let encoded = document
        .bytes()
        .map(|byte| format!("%{byte:02X}"))
        .collect::<String>();
    format!("<ListRolesResponse><ListRolesResult><Roles><member><Path>/</Path><RoleName>collector</RoleName><RoleId>AROA12345678901234567</RoleId><Arn>arn:aws:iam::123456789012:role/collector</Arn><CreateDate>2020-01-01T00:00:00Z</CreateDate><AssumeRolePolicyDocument>{encoded}</AssumeRolePolicyDocument></member></Roles><IsTruncated>false</IsTruncated></ListRolesResult></ListRolesResponse>").into_bytes()
}

#[tokio::test]
async fn given_allow_and_deny_trust_principals_when_projected_then_only_registered_allow_arn_targets_keep_foreign_owner()
-> TestResult {
    let catalog = catalog()?;
    let plan = plan(&catalog, "iam", "ListRoles")?;
    let document = r#"{"Statement":[{"Effect":"Allow","Action":"sts:AssumeRole","Principal":{"AWS":["arn:aws:iam::210987654321:role/external","arn:aws:iam::210987654321:root","*"],"Service":"ec2.amazonaws.com"},"Condition":{"StringEquals":{"sts:ExternalId":"private-value"}}},{"Effect":"Deny","Action":"sts:AssumeRole","Principal":{"AWS":"arn:aws:iam::210987654321:role/denied"}}]}"#;
    let observed = observations(plan, rows(plan, &listed_role(document))?)?;
    let observation = observed.first().ok_or("projected role absent")?;
    let references = crate::resource_projection::trust_references(
        &catalog,
        plan,
        observation.key(),
        observation.record(),
    )?;
    assert_eq!(references.len(), 1);
    let reference = references
        .first()
        .ok_or("allow principal reference absent")?;
    assert_eq!(reference.relation.as_str(), "DECLARES_TRUST");
    assert_eq!(reference.target.account().as_str(), "210987654321");
    assert_eq!(reference.target.region(), &Region::Global);
    assert_eq!(reference.target.kind().as_str(), "AWS::IAM::Role");
    assert_eq!(
        reference.target.value().as_str(),
        "arn:aws:iam::210987654321:role/external"
    );
    assert_eq!(reference.from.account.as_str(), "123456789012");
    let saved = persist(catalog.raw_bytes(), plan, observed).await?;
    let (_, _, record) = saved
        .first()
        .ok_or("role facet absent after reference commit")?;
    assert!(
        !record
            .windows("private-value".len())
            .any(|window| window == b"private-value")
    );
    Ok(())
}

#[test]
fn given_malformed_or_overbound_iam_documents_when_projected_then_the_page_fails_closed()
-> TestResult {
    let catalog = catalog()?;
    let plan = plan(&catalog, "iam", "ListRoles")?;
    for document in ["not-json".to_owned(), "x".repeat(65_537)] {
        let selected = rows(plan, &listed_role(&document))?;
        let row = selected.first().ok_or("role row absent")?;
        assert!(matches!(
            crate::resource_projection::project(plan, row),
            Err(SweepError::Operation("invalid bounded IAM trust document"))
        ));
    }
    Ok(())
}

#[tokio::test]
async fn given_dms_recursive_file_body_when_catalog_is_compiled_then_that_call_is_absent_and_safe_metadata_remains_collectable()
-> TestResult {
    let catalog = catalog()?;
    assert!(
        catalog
            .lookup("database-migration-service", "DescribeReplicationInstances")
            .is_none()
    );
    let plan = plan(
        &catalog,
        "database-migration-service",
        "DescribeInstanceProfiles",
    )?;
    let body = br#"{"InstanceProfiles":[{"InstanceProfileArn":"arn:aws:dms:us-east-1:123456789012:instance-profile:safe","InstanceProfileName":"safe-profile","NetworkType":"IPV4"}]}"#;
    let saved = persist(
        catalog.raw_bytes(),
        plan,
        observations(plan, rows(plan, body)?)?,
    )
    .await?;
    assert_eq!(saved.len(), 1);
    let (kind, id, _) = saved.first().ok_or("safe DMS metadata absent")?;
    assert_eq!(kind, "AWS::DMS::InstanceProfile");
    assert_eq!(
        id,
        "arn:aws:dms:us-east-1:123456789012:instance-profile:safe"
    );
    Ok(())
}
