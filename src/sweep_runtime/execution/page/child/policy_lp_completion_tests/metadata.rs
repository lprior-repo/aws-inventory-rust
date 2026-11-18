use super::*;
use inventory_core::record::{Record, ValueRef};

const REFERENCE_KIND: &str = "AWS::ApiOnly::OmicsReference";
const STORE: &str = "arn:aws:omics:us-east-1:123456789012:referenceStore/1234567890";
const DIGESTS: [&str; 2] = [
    "d41d8cd98f00b204e9800998ecf8427e",
    "0cc175b9c0f1b6a831c399e269772661",
];

#[tokio::test]
async fn given_omics_reference_integrity_metadata_when_native_pages_resume_then_md5_survives_reopen_while_readsets_are_denied()
-> TestResult {
    // Given actual policy compilation includes the clinical ReadSets exclusion.
    let policies = catalog_tool::overlay::embedded()?;
    let omics = policies
        .iter()
        .find(|service| service.service() == "omics")
        .ok_or("Omics policy absent")?;
    let read_sets = omics
        .operations()
        .iter()
        .find(|(name, _)| name.as_str() == "ListReadSets")
        .map(|(_, policy)| policy)
        .ok_or("ReadSets decision absent")?;
    assert_eq!(
        read_sets.class,
        Some(catalog_tool::overlay::OverlayClass::Deny(
            catalog_tool::classes::DenyReason::Content
        ))
    );
    let bytes = compiled(
        "omics",
        "2022-11-28",
        &["ListReferenceStores", "ListReferences", "ListReadSets"],
        "AWS::Omics::ReferenceStore",
    )?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    assert!(catalog.lookup("omics", "ListReadSets").is_none());
    let scenario =
        Scenario::new(&catalog, "omics-reference-integrity-not-clinical-content").await?;
    scenario
        .seed(plan(&catalog, "omics", "ListReferenceStores")?)
        .await?;
    // When resource-only references, including their native integrity digests, traverse real work.
    for _ in 0..3 {
        let lease = scenario.claim().await?;
        let operation = lease.scope().operation().as_str().to_owned();
        let callable = plan(&catalog, "omics", &operation)?;
        let second = lease.token().is_some();
        let (request, wire) = scenario.request(callable, &lease)?;
        assert_cursor(&wire, second)?;
        let body = if operation == "ListReferenceStores" {
            json!({"referenceStores":[{"arn":STORE,"id":"1234567890"}]})
        } else if operation == "ListReferences" {
            assert_parent(&wire, "1234567890")?;
            let id = if second { "1234567892" } else { "1234567891" };
            let digest = DIGESTS
                .get(usize::from(second))
                .ok_or("reference digest fixture absent")?;
            page(
                json!({"references":[{"arn":format!("{STORE}/reference/{id}"),"id":id,"referenceStoreId":"1234567890","md5":digest,"name":"reference","status":"ACTIVE","creationTime":"2024-01-01T00:00:00Z","updateTime":"2024-01-02T00:00:00Z"}]}),
                "nextToken",
                second,
            )?
        } else {
            return Err("non-resource Omics work was scheduled".into());
        };
        scenario
            .apply(&catalog, callable, lease, &request, body)
            .await?;
    }
    // Then exact owned resources, store edges and both md5 values remain after reopening SQLite.
    let scenario = scenario.reopen().await?;
    let ids: BTreeSet<_> = ["1234567891", "1234567892"]
        .into_iter()
        .map(|id| format!("{STORE}/reference/{id}"))
        .collect();
    let edges = ids
        .iter()
        .map(|id| {
            (
                id.clone(),
                STORE.to_owned(),
                "IN_REFERENCE_STORE".to_owned(),
            )
        })
        .collect();
    assert_eq!(scenario.stored(REFERENCE_KIND).await?, (ids, edges));
    assert_reference_digests(&scenario).await?;
    Ok(())
}

async fn assert_reference_digests(scenario: &Scenario<'_, '_>) -> TestResult {
    let reader = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(scenario.directory.path().join("inventory.db"))
                .read_only(true),
        )
        .await?;
    let rows: Vec<(String, Vec<u8>)> =
        sqlx::query_as("SELECT id,record FROM resource_facets WHERE kind=? ORDER BY id")
            .bind(REFERENCE_KIND)
            .fetch_all(&reader)
            .await?;
    reader.close().await;
    let registry = catalog_tool::smithy::Registry::parse(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/catalog/field-ids.txt"
    )))?;
    let md5 = registry.id("md5").ok_or("md5 field ID absent")?;
    assert_eq!(rows.len(), 2);
    for ((id, bytes), digest) in rows.into_iter().zip(DIGESTS) {
        assert!(id.starts_with(&format!("{STORE}/reference/")));
        let row = Record::from_bytes(bytes)?;
        assert!(
            matches!(crate::sweep_runtime::record::path_value(row.root(), &[md5])?, Some(ValueRef::Text(value)) if value == digest)
        );
    }
    Ok(())
}

#[tokio::test]
async fn given_organization_transfers_without_a_smithy_page_size_trait_when_native_tokens_resume_then_both_direction_unions_survive_reopen()
-> TestResult {
    let operations = [
        "ListInboundResponsibilityTransfers",
        "ListOutboundResponsibilityTransfers",
    ];
    let kind = "AWS::ApiOnly::OrganizationsResponsibilityTransfer";
    let bytes = compiled("organizations", "2016-11-28", &operations, kind)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let mut scenario =
        Scenario::new(&catalog, "organizations-native-transfer-continuations").await?;
    for operation in operations {
        scenario
            .seed(plan(&catalog, "organizations", operation)?)
            .await?;
    }
    let mut seen = BTreeSet::new();
    for _ in 0..4 {
        let lease = scenario.claim().await?;
        let operation = lease.scope().operation().as_str().to_owned();
        let callable = plan(&catalog, "organizations", &operation)?;
        let second = lease.token().is_some();
        let (request, wire) = scenario.request(callable, &lease)?;
        let native: Json = serde_json::from_slice(wire.body())?;
        assert_eq!(native.get("Type"), Some(&json!("BILLING")));
        assert!(
            callable.pagination().page_size.is_none(),
            "no synthetic page-size trait may be invented"
        );
        assert_eq!(native.get("MaxResults"), Some(&json!(20)));
        assert!(
            native.get("Id").is_none(),
            "inbound collection must remain unfiltered"
        );
        assert_cursor(&wire, second)?;
        assert!(seen.insert((operation.clone(), second)));
        let direction = if operation == "ListInboundResponsibilityTransfers" {
            "inbound"
        } else if operation == "ListOutboundResponsibilityTransfers" {
            "outbound"
        } else {
            return Err("unexpected transfer work".into());
        };
        let id = if second { "rt-22222222" } else { "rt-11111111" };
        let arn = format!(
            "arn:aws:organizations::123456789012:transfer/o-1234567890/billing/{direction}/{id}"
        );
        let response = page(
            json!({"ResponsibilityTransfers":[{"Arn":arn,"Id":id,"Type":"BILLING"}]}),
            "NextToken",
            second,
        )?;
        scenario
            .apply(&catalog, callable, lease, &request, response)
            .await?;
        scenario = scenario.reopen().await?;
    }
    assert_eq!(seen.len(), 4);
    let expected = ["inbound", "outbound"].into_iter().flat_map(|direction| ["rt-11111111", "rt-22222222"].into_iter().map(move |id| format!("arn:aws:organizations::123456789012:transfer/o-1234567890/billing/{direction}/{id}"))).collect();
    assert_eq!(scenario.stored(kind).await?, (expected, BTreeSet::new()));
    Ok(())
}
