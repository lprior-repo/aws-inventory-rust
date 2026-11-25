use super::*;

const SOURCES: [(EdgeSource, i64); 2] = [(EdgeSource::Model, 0), (EdgeSource::ReviewedRule, 1)];

fn declared_rule(code: i64) -> TestResult<Value> {
    let mut value = rule(0, &[1], vec![], None)?;
    let Value::Map(fields) = &mut value else {
        return Err("relationship fixture is not a map".into());
    };
    fields.insert(Key::Field(FieldId::new(7)?), Value::Int(code));
    Ok(value)
}

async fn prepared_page(
    store: &BinaryStore,
    source: EdgeSource,
) -> TestResult<inventory_engine::inventory::PageCommit> {
    store
        .seed_work(relationship_work(
            "provenance",
            "us-east-1",
            Some("target-name"),
            None,
        )?)
        .await?;
    let lease = store
        .claim(&SweepId::new("provenance".into())?, 100, 20)
        .await?
        .ok_or("provenance lease missing")?;
    let mut result = page(
        lease,
        vec![
            observation("source", "source-name")?,
            observation("target", "target-name")?,
        ],
        true,
        None,
        101,
    );
    let mut reference = reference(
        RelationTargetMode::SelectedFields,
        &[1],
        "target-name",
        None,
    )?;
    reference.source = source;
    result.unresolved.push(reference);
    Ok(result)
}

#[tokio::test]
async fn given_explicit_catalog_provenance_when_equal_evidence_commits_then_its_source_survives_reopen()
-> TestResult {
    for (source, code) in SOURCES {
        let (directory, store) = fixture_for(declared_rule(code)?, false).await?;
        store
            .commit_page(prepared_page(&store, source).await?)
            .await?;
        let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
        assert_edges(&reopened, &["target"], source).await?;
    }
    Ok(())
}

#[tokio::test]
async fn given_explicit_catalog_provenance_when_another_source_is_claimed_then_every_mismatch_is_refused()
-> TestResult {
    for (declared, code) in SOURCES {
        for actual in [
            EdgeSource::Model,
            EdgeSource::ReviewedRule,
            EdgeSource::InferredExact,
        ]
        .into_iter()
        .filter(|actual| *actual != declared)
        {
            let (directory, store) = fixture_for(declared_rule(code)?, false).await?;
            let commit = prepared_page(&store, actual).await?;
            let error = store
                .commit_page(commit)
                .await
                .err()
                .ok_or("expected an error")?;
            assert!(matches!(
                error,
                StoreError::Invalid(
                    "relationship evidence is not permitted by the checked catalog"
                )
            ));
            let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
            assert_edges(&reopened, &[], actual).await?;
            assert!(
                reopened
                    .projection_resource(&resource("source")?)
                    .await?
                    .is_none()
            );
        }
    }
    Ok(())
}
