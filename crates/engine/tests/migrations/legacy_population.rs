//! Legacy source fixture population.
use super::*;

pub(super) fn source_page(source: Source, ids: &[&str], time: i64) -> CollectionPage {
    let mut resources = Vec::new();
    let mut edges = Vec::new();
    for id in ids {
        let mut resource = fixture(
            id,
            "aws_ec2_instance",
            json!({"instance_id":id,"marker":source.as_str(),"literal":"line\n\tcloud ☁",
                "nested":{"nullable":null,"array":[1,true,"string"]},
                "cloudformation_type":"AWS::EC2::Instance","resource_identifier":id,
                "provider_properties":{"InstanceId":id}}),
            time,
        );
        resource.source = source;
        resource
            .tags
            .insert("custom-tag".into(), "quotes\" and spaces".into());
        resources.push(resource);
        edges.push(Edge {
            from_id: (*id).into(),
            to_id: "unobserved-vpc".into(),
            relation: "IN_VPC".into(),
            source,
            observed_at: time,
            properties: json!({"marker":source.as_str(),"evidence":[{"raw":"space\tline\n"}]}),
        });
    }
    let mut input = page(resources, edges);
    input.scope.collector = format!("fixture_{}", source.as_str());
    input.scope.source = source;
    input
}

/// Seed historical rows directly: opening the current Store would migrate them
/// before the test could establish the old priority and replay contract.
pub(super) async fn insert_legacy_page(
    connection: &mut SqliteConnection,
    input: &CollectionPage,
) -> TestResult {
    let old_priority = match input.scope.source {
        Source::Direct => 3_i64,
        Source::AwsConfig => 2,
        Source::ResourceExplorer => 1,
        Source::CloudControl => return Err("Cloud Control is not a historical source".into()),
    };
    let key = input.scope.key()?;
    let fingerprint = Sha256::digest(serde_json::to_vec(input)?).to_vec();
    sqlx::query(
        "INSERT INTO scope_checkpoints VALUES('populated',?1,?2,?3,?4,?5,?6,?7,?8,?9,0)
        ON CONFLICT(scan_id,scope_key) DO UPDATE SET last_page=excluded.last_page,
            next_token=excluded.next_token,complete=excluded.complete",
    )
    .bind(&key)
    .bind(&input.scope.account_id)
    .bind(&input.scope.region)
    .bind(&input.scope.collector)
    .bind(&input.scope.parent_id)
    .bind(input.scope.source.as_str())
    .bind(i64::from(input.page_number))
    .bind(&input.next_token)
    .bind(input.next_token.is_none())
    .execute(&mut *connection)
    .await?;
    sqlx::query("INSERT INTO collection_pages VALUES('populated',?1,?2,?3,?4,?5,?6)")
        .bind(&key)
        .bind(i64::from(input.page_number))
        .bind(fingerprint)
        .bind(&input.next_token)
        .bind(i64::try_from(input.resources.len())?)
        .bind(i64::try_from(input.edges.len())?)
        .execute(&mut *connection)
        .await?;
    for resource in &input.resources {
        // Noncanonical whitespace makes an accidental deserialize/reserialize
        // detectable, while the page digest still uses the v1/v2 wire format.
        sqlx::query("INSERT INTO resource_observations VALUES('populated',?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)")
            .bind(&key).bind(i64::from(input.page_number)).bind(&resource.id).bind(&resource.account_id).bind(&resource.region)
            .bind(&resource.kind).bind(&resource.name).bind(resource.source.as_str())
            .bind(old_priority).bind(resource.observed_at).bind(serde_json::to_string_pretty(resource)?)
            .execute(&mut *connection).await?;
    }
    for edge in &input.edges {
        sqlx::query("INSERT INTO edge_observations VALUES('populated',?1,?2,?3,?4,?5,?6,?7,?8,?9)")
            .bind(&key)
            .bind(i64::from(input.page_number))
            .bind(&edge.from_id)
            .bind(&edge.to_id)
            .bind(&edge.relation)
            .bind(edge.source.as_str())
            .bind(old_priority)
            .bind(edge.observed_at)
            .bind(serde_json::to_string_pretty(edge)?)
            .execute(&mut *connection)
            .await?;
    }
    Ok(())
}

pub(super) async fn populated_legacy(
    path: &Path,
    version: i64,
) -> Result<Vec<CollectionPage>, TestError> {
    let mut connection = create_legacy_schema(path, version).await?;
    sqlx::query("INSERT INTO scans VALUES('populated',1,40,'partial')")
        .execute(&mut connection)
        .await?;
    let mut direct = source_page(Source::Direct, &["preferred-direct"], 10);
    direct.next_token = Some("historical-next-page".into());
    let mut terminal = source_page(Source::Direct, &[], 11);
    terminal.page_number = 2;
    let mut pending = source_page(Source::Direct, &[], 35);
    pending.scope.collector = "pending_native_scope".into();
    pending.next_token = Some("saved-resume-token".into());
    let pages = vec![
        direct,
        terminal,
        source_page(
            Source::AwsConfig,
            &["preferred-direct", "preferred-config", "preferred-cloud"],
            20,
        ),
        source_page(
            Source::ResourceExplorer,
            &[
                "preferred-direct",
                "preferred-config",
                "preferred-cloud",
                "preferred-explorer",
            ],
            30,
        ),
        pending,
    ];
    for input in &pages {
        insert_legacy_page(&mut connection, input).await?;
    }
    sqlx::raw_sql("UPDATE scope_checkpoints SET failed=1 WHERE source='aws_config';
        INSERT INTO collection_failures SELECT scan_id,scope_key,'fixture_partial','preserve failure text'
            FROM scope_checkpoints WHERE source='aws_config';
        INSERT INTO graph_export_checkpoints VALUES('populated','falkordb','preserved-snapshot',4,4,1);")
        .execute(&mut connection).await?;
    connection.close().await?;
    Ok(pages)
}
