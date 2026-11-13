use std::collections::BTreeMap;

use inventory_core::{CollectionPage, CollectionScope, Resource, ScanStatus, Source};
use inventory_engine::store::Store;
use serde_json::{Value, json};
use sqlx::{Connection, SqliteConnection, sqlite::SqliteConnectOptions};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const CURRENT_SCHEMA_VERSION: i64 = 6;
const NEW_VIEWS: [&str; 10] = [
    "aws_s3_bucket",
    "aws_iam_role",
    "aws_iam_instance_profile",
    "aws_iam_policy",
    "aws_route53_zone",
    "aws_route53_record",
    "aws_cloudfront_distribution",
    "aws_cloudfront_origin",
    "aws_iam_instance_profile_role",
    "aws_route53_zone_vpc",
];

fn resource(id: &str, kind: &str, properties: Value) -> Resource {
    Resource {
        id: id.into(),
        account_id: "123456789012".into(),
        region: "global".into(),
        kind: kind.into(),
        name: Some(id.into()),
        properties,
        tags: BTreeMap::new(),
        source: Source::Direct,
        observed_at: 2,
    }
}

async fn fixture(store: &Store) -> TestResult {
    store.begin_scan("global", 1).await?;
    store.apply_page("global",CollectionPage {
        scope:CollectionScope{account_id:"123456789012".into(),region:"global".into(),collector:"global_fixture".into(),parent_id:None,source:Source::Direct},
        page_number:1,next_token:None,edges:Vec::new(),resources:vec![
            resource("bucket", "aws_s3_bucket",json!({"bucket_name":"public-test","bucket_region":"us-west-2","policy_status":{"configured":true,"is_public":true},"encryption":null,"unavailable_sections":["encryption"]})),
            resource("role", "aws_iam_role",json!({"role_name":"reader","path":"/audit/","details_available":true,"trust_statements":[{"effect":"Allow","condition_present":true}]})),
            resource("profile", "aws_iam_instance_profile",json!({"instance_profile_name":"reader-profile","role_arns":["role"],"details_available":true})),
            resource("policy", "aws_iam_policy",json!({"policy_name":"ReadOnly","policy_scope":"Local","policy_document_collected":false,"details_available":true})),
            resource("zone", "aws_route53_zone",json!({"zone_id":"ZTEST","private_zone":true,"vpcs":[{"vpc_id":"vpc-example","vpc_region":"us-west-2","owner_resolution":"assumed_scanning_account"}]})),
            resource("record", "aws_route53_record",json!({"zone_id":"ZTEST","type":"TXT","ttl":60,"records":[],"rdata_count":1,"rdata_omitted":true})),
            resource("distribution", "aws_cloudfront_distribution",json!({"distribution_id":"ETEST","enabled":true,"domain_name":"test.cloudfront.net","origins":[{"id":"bucket-origin","domain_name":"public-test.s3.us-west-2.amazonaws.com","s3_origin":true,"custom_header_count":1,"custom_header_values_omitted":true}]})),
        ]
    }).await?;
    store.finish_scan("global", ScanStatus::Complete, 3).await?;
    Ok(())
}

#[tokio::test]
async fn global_views_expose_metadata_without_inventing_unknown_values() -> TestResult {
    let store = Store::open(":memory:").await?;
    fixture(&store).await?;
    assert_eq!(store.query("SELECT bucket_name,bucket_region,policy_is_public,encryption,unavailable_sections FROM aws_s3_bucket",10).await?,json!([{"bucket_name":"public-test","bucket_region":"us-west-2","policy_is_public":1,"encryption":null,"unavailable_sections":"[\"encryption\"]"}]));
    assert_eq!(
        store
            .query(
                "SELECT policy_document_collected,policy_scope FROM aws_iam_policy",
                10
            )
            .await?,
        json!([{"policy_document_collected":0,"policy_scope":"Local"}])
    );
    assert_eq!(
        store
            .query(
                "SELECT type,records,rdata_count,rdata_omitted FROM aws_route53_record",
                10
            )
            .await?,
        json!([{"type":"TXT","records":"[]","rdata_count":1,"rdata_omitted":1}])
    );
    assert_eq!(store.query("SELECT distribution_id,origin_id,domain_name,s3_origin,custom_header_count FROM aws_cloudfront_origin",10).await?,json!([{"distribution_id":"ETEST","origin_id":"bucket-origin","domain_name":"public-test.s3.us-west-2.amazonaws.com","s3_origin":1,"custom_header_count":1}]));
    assert_eq!(
        store
            .query(
                "SELECT instance_profile_arn,role_arn FROM aws_iam_instance_profile_role",
                10
            )
            .await?,
        json!([{"instance_profile_arn":"profile","role_arn":"role"}])
    );
    assert_eq!(
        store
            .query(
                "SELECT zone_id,vpc_id,vpc_region,owner_resolution FROM aws_route53_zone_vpc",
                10
            )
            .await?,
        json!([{"zone_id":"ZTEST","vpc_id":"vpc-example","vpc_region":"us-west-2","owner_resolution":"assumed_scanning_account"}])
    );
    store.close().await;
    Ok(())
}

async fn make_v3(path: &std::path::Path) -> TestResult {
    let store = Store::open(path).await?;
    fixture(&store).await?;
    store.close().await;
    let mut connection =
        SqliteConnection::connect_with(&SqliteConnectOptions::new().filename(path)).await?;
    // Store::open creates the current schema. A genuine v3 database predates
    // both the global views and the v5 Cloud Control expansion, so remove only
    // the reviewed view identifiers declared by that later migration.
    let mut removed_expansion_views = 0;
    for view in [
        include_str!("../sql/005_cloudcontrol_expansion.sql"),
        include_str!("../sql/006_cloudcontrol_parents.sql"),
    ]
    .into_iter()
    .flat_map(str::lines)
    .filter_map(|line| line.strip_prefix("CREATE VIEW "))
    .filter_map(|line| line.strip_suffix(" AS"))
    {
        assert!(view.starts_with("aws_cc_"));
        assert!(
            view.bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        );
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!("DROP VIEW {view}")))
            .execute(&mut connection)
            .await?;
        removed_expansion_views += 1;
    }
    assert_eq!(removed_expansion_views, 218);
    let legacy_cloud_control_views: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM sqlite_schema WHERE type='view' AND name GLOB 'aws_cc_*'",
    )
    .fetch_one(&mut connection)
    .await?;
    assert_eq!(legacy_cloud_control_views, 200);
    for view in NEW_VIEWS {
        // Only reviewed, constant identifiers enter this migration fixture.
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!("DROP VIEW {view}")))
            .execute(&mut connection)
            .await?;
    }
    sqlx::raw_sql(
        "PRAGMA user_version=3; CREATE VIEW owner_view AS SELECT id,kind FROM latest_resources;",
    )
    .execute(&mut connection)
    .await?;
    connection.close().await?;
    Ok(())
}

#[tokio::test]
async fn schema_four_upgrades_existing_snapshots_and_preserves_owner_objects() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("v3.sqlite");
    make_v3(&path).await?;
    let mut before =
        SqliteConnection::connect_with(&SqliteConnectOptions::new().filename(&path)).await?;
    let fingerprint: Vec<u8> = sqlx::query_scalar("SELECT fingerprint FROM collection_pages")
        .fetch_one(&mut before)
        .await?;
    before.close().await?;
    let store = Store::open(&path).await?;
    assert_eq!(
        store
            .query("SELECT user_version AS version FROM pragma_user_version", 1)
            .await?,
        json!([{"version":CURRENT_SCHEMA_VERSION}])
    );
    assert_eq!(
        store
            .query("SELECT count(*) AS total FROM owner_view", 1)
            .await?,
        json!([{"total":7}])
    );
    assert_eq!(store.scan_status("global").await?, ScanStatus::Complete);
    assert_eq!(
        store
            .query("SELECT role_name,path FROM aws_iam_role", 1)
            .await?,
        json!([{"role_name":"reader","path":"/audit/"}])
    );
    store.close().await;
    let mut after =
        SqliteConnection::connect_with(&SqliteConnectOptions::new().filename(&path)).await?;
    let actual: Vec<u8> = sqlx::query_scalar("SELECT fingerprint FROM collection_pages")
        .fetch_one(&mut after)
        .await?;
    assert_eq!(actual, fingerprint);
    after.close().await?;
    Ok(())
}

#[tokio::test]
async fn schema_four_name_conflict_rolls_back_all_new_views() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("conflict.sqlite");
    make_v3(&path).await?;
    let mut connection =
        SqliteConnection::connect_with(&SqliteConnectOptions::new().filename(&path)).await?;
    sqlx::raw_sql("CREATE TABLE aws_route53_zone(note TEXT); INSERT INTO aws_route53_zone VALUES('owner data');").execute(&mut connection).await?;
    connection.close().await?;
    assert!(Store::open(&path).await.is_err());
    let mut connection =
        SqliteConnection::connect_with(&SqliteConnectOptions::new().filename(&path)).await?;
    let version: i64 = sqlx::query_scalar("PRAGMA user_version")
        .fetch_one(&mut connection)
        .await?;
    let partial: i64 =
        sqlx::query_scalar("SELECT count(*) FROM sqlite_schema WHERE name='aws_s3_bucket'")
            .fetch_one(&mut connection)
            .await?;
    let note: String = sqlx::query_scalar("SELECT note FROM aws_route53_zone")
        .fetch_one(&mut connection)
        .await?;
    assert_eq!((version, partial, note), (3, 0, "owner data".into()));
    connection.close().await?;
    Ok(())
}
