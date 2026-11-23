use super::*;
use inventory_core::record::{Exclusions, Key, MapKeyExclusion};

async fn commit_masked(
    store: &BinaryStore,
    catalog: &RuntimeCatalog<'_>,
    operation: &str,
    record: Record,
    exclusions: &Exclusions,
    at: i64,
) -> TestResult<Vec<u8>> {
    let plan = catalog
        .lookup("workspaces-web", operation)
        .ok_or("mask plan absent")?;
    let observation = Observation::new(key()?, record, exclusions, None)?
        .with_facet_precedence(plan.facet_precedence());
    let hash = observation.content_hash().as_bytes().to_vec();
    let sweep = SweepId::new(format!("mask-{at}"))?;
    let scope = ScopeKey::new(
        AccountId::parse("123456789012")?,
        Region::parse("us-east-1")?,
        ServiceId::parse("workspaces-web")?,
        OperationId::parse(operation)?,
        None,
    )?;
    store
        .seed_work(WorkSpec::new(sweep.clone(), scope, None, vec![1])?)
        .await?;
    let lease = store
        .claim(&sweep, at, 10_000)
        .await?
        .ok_or("mask lease absent")?;
    store
        .commit_page(PageCommit {
            lease,
            observations: vec![observation],
            edges: Vec::new(),
            unresolved: Vec::new(),
            tags: Vec::new(),
            children: Vec::new(),
            next_token: None,
            continuation_digest: None,
            done: true,
            observed_at_ms: at,
        })
        .await?;
    Ok(hash)
}

fn masked_record(endpoint: &str, purpose: &str) -> TestResult<Record> {
    Ok(Record::encode(&Value::Map(BTreeMap::from([
        (Key::Field(FieldId::new(5)?), Value::Text(PORTAL.to_owned())),
        (
            Key::Field(FieldId::new(1000)?),
            Value::Map(BTreeMap::from([
                (
                    Key::Name("Endpoint".to_owned()),
                    Value::Text(endpoint.to_owned()),
                ),
                (
                    Key::Name("Purpose".to_owned()),
                    Value::Text(purpose.to_owned()),
                ),
            ])),
        ),
    ])))?)
}

#[tokio::test]
async fn given_named_map_hash_mask_when_facets_reload_then_only_that_exact_entry_remains_excluded()
-> TestResult {
    // Given a reviewed named-map mask whose sibling entry still affects the canonical hash.
    let bytes = compiled()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (directory, store) = store(&catalog).await?;
    let mask = Exclusions::none().with_map_keys(vec![MapKeyExclusion::new(
        vec![FieldId::new(1000)?],
        "Endpoint",
    )?])?;
    let expected = commit_masked(
        &store,
        &catalog,
        "ListPortals",
        masked_record("secret-a", "inventory")?,
        &mask,
        1,
    )
    .await?;
    assert_eq!(state(&directory, &catalog).await?.2, expected);
    // When only the excluded entry changes, and a sparse sibling commits after reopening.
    commit_masked(
        &store,
        &catalog,
        "ListPortals",
        masked_record("secret-b", "inventory")?,
        &mask,
        2,
    )
    .await?;
    drop(store);
    let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    persist(
        &reopened,
        &catalog,
        "GetPortal",
        json!({"portal":{"portalArn":PORTAL}}),
        3,
    )
    .await?;
    // Then stored mask decoding preserves hash and last_changed across the unrelated facet.
    let current = state(&directory, &catalog).await?;
    assert_eq!(current.2, expected);
    assert_eq!(current.1, 1);
    assert_eq!(
        current.0.as_bytes(),
        masked_record("secret-b", "inventory")?.as_bytes()
    );
    // When an unmasked sibling entry changes, it is not accidentally hidden.
    commit_masked(
        &reopened,
        &catalog,
        "ListPortals",
        masked_record("secret-c", "changed")?,
        &mask,
        4,
    )
    .await?;
    let changed = state(&directory, &catalog).await?;
    assert_ne!(changed.2, expected);
    assert_eq!(changed.1, 4);
    assert_eq!(
        changed.0.as_bytes(),
        masked_record("secret-c", "changed")?.as_bytes()
    );
    Ok(())
}

fn numeric_record(volatile: i64, purpose: &str) -> TestResult<Record> {
    Ok(Record::encode(&Value::Map(BTreeMap::from([
        (Key::Field(FieldId::new(5)?), Value::Text(PORTAL.to_owned())),
        (
            Key::Field(FieldId::new(1000)?),
            Value::Map(BTreeMap::from([
                (Key::Field(FieldId::new(1001)?), Value::Int(volatile)),
                (
                    Key::Field(FieldId::new(1002)?),
                    Value::Text(purpose.to_owned()),
                ),
            ])),
        ),
    ])))?)
}

#[tokio::test]
async fn given_numeric_hash_mask_when_store_reopens_then_only_exact_numeric_leaf_is_excluded()
-> TestResult {
    let bytes = compiled()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (directory, store) = store(&catalog).await?;
    let mask = Exclusions::new([vec![FieldId::new(1000)?, FieldId::new(1001)?]])?;
    let expected = commit_masked(
        &store,
        &catalog,
        "ListPortals",
        numeric_record(1, "inventory")?,
        &mask,
        1,
    )
    .await?;
    commit_masked(
        &store,
        &catalog,
        "ListPortals",
        numeric_record(2, "inventory")?,
        &mask,
        2,
    )
    .await?;
    drop(store);
    let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    persist(
        &reopened,
        &catalog,
        "GetPortal",
        json!({"portal":{"portalArn":PORTAL}}),
        3,
    )
    .await?;
    let current = state(&directory, &catalog).await?;
    assert_eq!(current.2, expected);
    assert_eq!(current.1, 1);
    assert_eq!(
        current.0.as_bytes(),
        numeric_record(2, "inventory")?.as_bytes()
    );
    commit_masked(
        &reopened,
        &catalog,
        "ListPortals",
        numeric_record(3, "changed")?,
        &mask,
        4,
    )
    .await?;
    let changed = state(&directory, &catalog).await?;
    assert_ne!(changed.2, expected);
    assert_eq!(changed.1, 4);
    assert_eq!(
        changed.0.as_bytes(),
        numeric_record(3, "changed")?.as_bytes()
    );
    Ok(())
}

fn named_leaf(name: &str, value: &str) -> TestResult<Record> {
    Ok(Record::encode(&Value::Map(BTreeMap::from([
        (Key::Field(FieldId::new(5)?), Value::Text(PORTAL.to_owned())),
        (
            Key::Field(FieldId::new(1000)?),
            Value::Map(BTreeMap::from([(
                Key::Name(name.to_owned()),
                Value::Text(value.to_owned()),
            )])),
        ),
    ])))?)
}

#[tokio::test]
async fn given_nested_sibling_owners_when_detail_omits_its_leaf_then_live_list_leaf_survives_reopen()
-> TestResult {
    let bytes = compiled()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (directory, store) = store(&catalog).await?;
    commit_masked(
        &store,
        &catalog,
        "ListPortals",
        named_leaf("ListLeaf", "live")?,
        &Exclusions::none(),
        1,
    )
    .await?;
    commit_masked(
        &store,
        &catalog,
        "GetPortal",
        named_leaf("DetailLeaf", "old")?,
        &Exclusions::none(),
        2,
    )
    .await?;
    let before = state(&directory, &catalog).await?.0;
    let ValueRef::Map(nested) = before
        .root()
        .get_field(FieldId::new(1000)?)?
        .ok_or("nested map missing")?
    else {
        return Err("nested value is not a map".into());
    };
    assert_eq!(
        nested
            .get_name("ListLeaf")?
            .map(|value| value.to_value())
            .transpose()?,
        Some(Value::Text("live".to_owned()))
    );
    assert_eq!(
        nested
            .get_name("DetailLeaf")?
            .map(|value| value.to_value())
            .transpose()?,
        Some(Value::Text("old".to_owned()))
    );
    drop(store);
    let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    persist(
        &reopened,
        &catalog,
        "GetPortal",
        json!({"portal":{"portalArn":PORTAL}}),
        3,
    )
    .await?;
    let current = state(&directory, &catalog).await?;
    assert_eq!(
        current.0.as_bytes(),
        named_leaf("ListLeaf", "live")?.as_bytes()
    );
    assert_eq!(current.1, 3);
    Ok(())
}
